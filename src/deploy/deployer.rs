use super::command;
use crate::deploy::InstallItem;
use crate::deploy::Installed;
use crate::deploy::alias::Alias;
use crate::deploy::apt::Apt;
use crate::deploy::env::Env;
use crate::deploy::envrc::Envrc;
use crate::deploy::file::File;
use crate::op::file::FileOp;
use crate::types::config::Config;
use anyhow::Ok;
use anyhow::anyhow;
use anyhow::ensure;
use anyhow::{Context, Result};
use derive_more::Debug;
use dirs::home_dir;
use log::debug;
use log::info;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const DEPLOY_DIR: &str = ".distro";

#[derive(Debug)]
#[allow(unused)]
pub struct Deployer {
    config_list: Vec<ConfigInfo>,
    install_dir: PathBuf,

    #[debug(ignore)]
    installers: Vec<InstallInfo>,
}

#[derive(Debug)]
struct ConfigInfo {
    file: PathBuf,
    config: Config,
}

#[derive(Debug)]
struct InstallInfo {
    config_file: PathBuf,
    install_items: Box<dyn InstallItem>,
}

impl Deployer {
    const BEGIN_TAG: &str = "# Config Start";
    const END_TAG: &str = "# Config End";

    pub fn from_list<P: AsRef<Path>>(list_paths: &[P]) -> Result<Self> {
        let mut config_list = Vec::new();

        list_paths.iter().try_for_each(|path| {
            let content = fs::read_to_string(path)
                .context(format!("Fail to read {}", path.as_ref().to_string_lossy()))?;

            let config: Config = toml::from_str(&content).context(format!(
                "File {} is invaild",
                path.as_ref().to_string_lossy()
            ))?;

            config_list.push(ConfigInfo {
                file: path.as_ref().to_path_buf(),
                config,
            });
            Ok(())
        })?;

        let install_path = Self::deploy_dir();

        let mut installers = Vec::new();

        for config_info in &config_list {
            let config = &config_info.config;
            let config_path = config_info.file.parent().unwrap_or(Path::new("."));

            if let Some(command) = &config.infomation.install_while {
                info!(target: "Deployer", "Checking the precondition for {}: {command}", config.infomation.name);

                let check = Command::new("bash").arg("-c").arg(command).status();

                if !check.is_ok_and(|code| code.success()) {
                    info!(target: "Deployer", "Checking fail: {command}, next...");
                    continue;
                }
            }

            let new_installers: Vec<_> = Self::dispath_config(
                config,
                config_path,
                install_path.join(&config.infomation.name),
            )
            .into_iter()
            .map(|installer| InstallInfo {
                config_file: config_info.file.clone(),
                install_items: installer,
            })
            .collect();

            installers.extend(new_installers);
        }

        info!(target: "Deployer", "got {} installers", installers.len());

        Ok(Self {
            config_list,
            install_dir: install_path.to_path_buf(),
            installers,
        })
    }

    pub fn deploy(&self) -> Result<()> {
        self.check_all()?;
        self.deploy_all()?;

        Ok(())
    }

    pub fn deploy_dir() -> PathBuf {
        home_dir().unwrap().join(DEPLOY_DIR)
    }

    pub fn unset_bashrc() -> Result<()> {
        let bashrc_path = home_dir()
            .ok_or(anyhow!("Fail to get home dir"))?
            .join(".bashrc");

        let bashrc = fs::read_to_string(&bashrc_path).context("Fail to read .bashrc")?;

        let re = Regex::new(format!(r"(?s){}\n.*?{}\n", Self::BEGIN_TAG, Self::END_TAG).as_str())?;

        let replaced = re.replace(&bashrc, "").to_string();

        fs::write(bashrc_path, replaced).context("Fail to write .bashrc")?;

        Ok(())
    }

    fn check_all(&self) -> Result<()> {
        self.installers.iter().try_for_each(
            |InstallInfo {
                 config_file,
                 install_items,
             }| {
                debug!(target: "Deployer", "Checking for {}", config_file.to_string_lossy());

                install_items.check().context(format!(
                    "{} -> {:?} checking fail",
                    config_file.to_string_lossy(),
                    install_items
                ))
            },
        )?;

        ensure!(
            FileOp::is_dir(&self.install_dir) || !FileOp::exist(&self.install_dir),
            "{} is a directory",
            self.install_dir.to_string_lossy()
        );

        Ok(())
    }

    fn deploy_all(&self) -> Result<()> {
        let mut installed_list = Vec::new();

        for InstallInfo {
            config_file,
            install_items,
        } in &self.installers
        {
            debug!(target: "Deployer", "Installing a item for {}", config_file.to_string_lossy());

            let installed = install_items.install().context(format!(
                "Fail to install item: {} -> {:?}",
                config_file.to_string_lossy(),
                install_items
            ))?;

            debug!("new installed item: {installed:?}");

            installed_list.push(installed);
        }

        let rc_content: String = installed_list
            .iter()
            .filter_map(|installed| match installed {
                super::Installed::Rc { command } => Some(command.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");

        let path_content = installed_list
            .iter()
            .filter_map(|installed| match installed {
                Installed::Path { path } => Some(path.clone()),
                _ => None,
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
            .join(":");

        let uninstall_rc = format!(
            "alias uninstall='rm -rf {}'",
            self.install_dir.to_string_lossy()
        );

        let allrc = format!(
            "#!/bin/bash\n# This file is auto-generated by {}. Do not modify it to avoid invalidation.\nexport PATH={}:$PATH\n{}\n{}\n",
            env!("CARGO_PKG_NAME"),
            path_content,
            rc_content,
            uninstall_rc
        );

        let allrc_file = self.install_dir.join("allrc");

        FileOp::write(&allrc_file, allrc, None).context(format!(
            "Fail to create file: {}",
            allrc_file.to_string_lossy()
        ))?;

        Self::unset_bashrc().context("Fail to unset .bashrc")?;
        Self::setup_bash(allrc_file).context("Fail to setup allrc")?;

        Ok(())
    }

    fn setup_bash(rc_file: PathBuf) -> Result<()> {
        let source_rc = format!("test -f {0} && source {0}", rc_file.to_string_lossy());
        let new_block = format!("\n{}\n{}\n{}\n", Self::BEGIN_TAG, source_rc, Self::END_TAG);

        let bashrc_path = home_dir()
            .ok_or(anyhow!("Fail to get home dir"))?
            .join(".bashrc");

        let mut bashrc = fs::read_to_string(&bashrc_path).context("Fail to read .bashrc")?;

        bashrc.push_str(&new_block);

        fs::write(bashrc_path, bashrc).context("Fail to write .bashrc")?;

        Ok(())
    }

    fn dispath_config<P1, P2>(
        config: &Config,
        config_path: P1,
        install_path: P2,
    ) -> Vec<Box<dyn InstallItem>>
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let mut installers: Vec<Box<dyn InstallItem>> = Vec::new();
        let (apt, alias, command, env, envrc, files) = (
            config.install.apt.as_ref(),
            config.install.alias.as_ref(),
            config.install.command.as_ref(),
            config.install.env.as_ref(),
            config.install.envrc.as_ref(),
            config.install.files.as_ref(),
        );

        if let Some(softwares) = apt {
            for sw in softwares {
                installers.push(Box::new(Apt::from_sw(sw.clone())));
            }
        }

        if let Some(alias_list) = alias {
            for alias in alias_list {
                installers.push(Box::new(Alias::from_pair(alias.0, alias.1)))
            }
        }

        if let Some(commands) = command {
            for command in commands {
                installers.push(Box::new(command::Command::from_content(
                    command.0,
                    command.1,
                    config_path.as_ref(),
                    install_path.as_ref().join("bin"),
                )));
            }
        }

        if let Some(env_lsit) = env {
            for env in env_lsit {
                installers.push(Box::new(Env::from_kv(env.0, env.1)));
            }
        }

        if let Some(rc_list) = envrc {
            for rc in rc_list {
                installers.push(Box::new(Envrc::from_content(rc, config_path.as_ref())));
            }
        }

        if let Some(files) = files {
            for file in files {
                installers.push(Box::new(File::from_content(
                    file.0,
                    file.1,
                    config_path.as_ref(),
                    install_path.as_ref(),
                )));
            }
        }

        installers
    }
}
