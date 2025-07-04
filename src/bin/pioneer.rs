use anyhow::{Context, Result};
use clap::{Parser, command};
use distro_pioneer::{deploy::deployer::Deployer, log::log_init};
use log::info;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
enum Args {
    /// 开始pioneer，检查和安装软件、环境变量、rc脚本等
    Install {
        /// 用于pioneer的配置文件列表
        #[arg(required = true, num_args = 1..)]
        configs: Vec<PathBuf>,
    },

    /// 卸载删除所有已安装的items
    Uninstall {},
}

fn main() -> Result<()> {
    log_init(None);

    let args = Args::parse();

    match &args {
        Args::Install { configs } => install(configs)?,
        Args::Uninstall {} => uninstall()?,
    }

    Ok(())
}

fn install(configs: &[PathBuf]) -> Result<()> {
    info!(target: "install", "config files: \n{}", configs.iter().map(|path|path.to_string_lossy()).collect::<Vec<_>>().join("\n"));

    let deployer = Deployer::from_list(configs)?;

    deployer.deploy()?;

    info!(target: "uninstall", "Install all done");
    Ok(())
}

fn uninstall() -> Result<()> {
    let deploy_dir = Deployer::deploy_dir();
    info!(target: "uninstall", "removing {}", deploy_dir.to_string_lossy());

    fs::remove_dir_all(&deploy_dir)
        .context(format!("Fail to remove {}", deploy_dir.to_string_lossy()))?;

    info!(target: "uninstall", "unseting bashrc");

    Deployer::unset_bashrc().context("Fail to unset .bashrc")?;

    info!(target: "uninstall", "all done");

    Ok(())
}
