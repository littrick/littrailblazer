use crate::program::run_or_sudo;
use anyhow::{Context, Ok, Result, anyhow};
use lazy_static::lazy_static;
use log::info;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use which::which;

type AptIst = Mutex<Option<AptOp>>;

lazy_static! {
    static ref APT: AptIst = Default::default();
}

pub struct AptOp {
    apt_path: PathBuf,
    list: HashSet<String>,
}

#[allow(unused)]
impl AptOp {
    pub fn try_get() -> Result<&'static AptIst> {
        let mut apt = APT.lock().unwrap();

        if apt.is_none() {
            *apt = Some(Self {
                apt_path: which("apt")?,
                list: Default::default(),
            });

            let apt = apt.as_mut().unwrap();
            apt.update().context("apt update fail")?;
            apt.get_list().context("apt list fail")?;
        }
        Ok(&APT)
    }

    pub fn check<S>(&self, name: S) -> Result<()>
    where
        S: AsRef<OsStr>,
    {
        let name = name.as_ref().to_string_lossy().to_string();

        info!(target: "APT", "Checking package: {name}...");

        /* 创建命令 */
        self.list
            .get(&name)
            .ok_or(anyhow!(format!("Con not find package: {name}")))?;
        Ok(())
    }

    pub fn install<S: AsRef<OsStr>>(&self, name: S) -> Result<()> {
        /* 创建命令 */
        let mut cmd = Command::new(&self.apt_path);
        cmd.arg("install").arg("-y").arg(name);
        cmd.env("DEBIAN_FRONTEND", "noninteractive");

        run_or_sudo(cmd).context("apt install fail")?;

        Ok(())
    }

    pub fn remove<S: AsRef<OsStr>>(&self, name: S) -> Result<()> {
        let name = name.as_ref();
        info!( target: "APT", "Removeing {} ...", name.to_string_lossy());

        /* 创建命令 */
        let mut cmd = Command::new(&self.apt_path);
        cmd.arg("remove")
            .arg("-y")
            .arg("--autoremove")
            .arg("--purge")
            .arg(name);

        run_or_sudo(cmd).context(format!(
            "Fail to remove package: {}",
            name.to_string_lossy()
        ))?;

        Ok(())
    }

    fn update(&mut self) -> Result<()> {
        let mut cmd = Command::new(&self.apt_path);
        cmd.arg("update");

        run_or_sudo(cmd).context("Fail to update package list")?;
        Ok(())
    }

    fn get_list(&mut self) -> Result<()> {
        let mut cmd = Command::new(&self.apt_path);
        cmd.arg("list");

        let list_content = run_or_sudo(cmd).context("apt list fail")?;
        self.list = list_content
            .lines()
            .filter_map(|s| s.split_once('/').map(|(before, _)| before.to_string()))
            .collect();

        Ok(())
    }
}

impl Debug for AptOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AptOp")
            .field("apt_path", &self.apt_path)
            .finish()
    }
}
