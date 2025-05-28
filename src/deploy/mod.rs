use anyhow::Result;
use std::path::PathBuf;

pub mod deployer;

mod alias;
mod apt;
mod command;
mod env;
mod envrc;
mod file;

#[derive(Debug)]
#[allow(unused)]
enum Installed {
    Apt { name: String },
    Rc { command: String },
    Path { path: String },
    File { file: PathBuf },
}

trait InstallItem: std::fmt::Debug {
    /// 安装前的检查
    fn check(&self) -> Result<()>;

    /// 正式的安装操作，尽可能在check内将所有失败因素排除掉
    fn install(&self) -> Result<Installed>;
}
