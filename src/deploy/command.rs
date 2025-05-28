use std::path::{Path, PathBuf};

use anyhow::{Context, Ok, ensure};
use log::info;
use regex::Regex;

use crate::{
    deploy::{InstallItem, Installed},
    op::file::FileOp,
    types::config::{Content, StringOr},
};

#[derive(Debug)]
pub struct Command {
    name: String,
    content: StringOr<Content>,
    config_path: PathBuf,
    install_file: PathBuf,
}

impl Command {
    pub fn from_content<N, P1, P2>(
        name: N,
        content: &StringOr<Content>,
        config_path: P1,
        install_path: P2,
    ) -> Self
    where
        N: Into<String>,
        P1: AsRef<Path>,
        P2: AsRef<Path>,
    {
        let name = name.into();
        Self {
            name: name.clone(),
            content: content.clone(),
            config_path: config_path.as_ref().to_path_buf(),
            install_file: install_path.as_ref().join(name),
        }
    }
}

impl InstallItem for Command {
    fn check(&self) -> anyhow::Result<()> {
        info!(target: "Command", "Checking command {}", self.name);

        let re = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_-]*$").unwrap();
        anyhow::ensure!(
            !self.name.is_empty() && !self.name.contains('=') && re.is_match(&self.name),
            "Name only allows letters, numbers, underscores(_), and hyphens(-)."
        );

        match &self.content {
            StringOr::String(_) | StringOr::Object(Content::Raw(_)) => {}
            StringOr::Object(Content::File(path)) => {
                let path = self.config_path.join(path);
                ensure!(
                    FileOp::exist(&path),
                    "{} is not exist",
                    path.to_string_lossy()
                );
                ensure!(
                    FileOp::exist(&path),
                    "{} is not a file",
                    path.to_string_lossy()
                );
            }
            StringOr::Object(Content::Url(_)) => todo!(),
        }

        ensure!(
            FileOp::is_file(&self.install_file) || !FileOp::is_dir(&self.install_file),
            "{} is a directory",
            self.install_file.to_string_lossy()
        );

        // FileOp::write(&self.install_file, "", None).context(format!(
        //     "Fail to create file{}",
        //     self.install_file.to_string_lossy()
        // ))?;

        Ok(())
    }

    fn install(&self) -> anyhow::Result<super::Installed> {
        if let Some(dir) = self.install_file.parent() {
            FileOp::mkdir(dir).context(format!("Fail to mkdir {}", dir.to_string_lossy()))?;
        }

        match &self.content {
            StringOr::String(content) | StringOr::Object(Content::Raw(content)) => {
                FileOp::write(&self.install_file, content, Some(0o755)).context(format!(
                    "fail to install file {}",
                    self.install_file.to_string_lossy()
                ))?;
            }
            StringOr::Object(Content::File(path)) => {
                FileOp::copy(path, &self.install_file).context(format!(
                    "fail to install file {}",
                    self.install_file.to_string_lossy()
                ))?;
            }
            StringOr::Object(Content::Url(_)) => todo!(),
        }

        Ok(Installed::Path {
            path: self
                .install_file
                .parent()
                .unwrap()
                .to_string_lossy()
                .to_string(),
        })
    }
}
