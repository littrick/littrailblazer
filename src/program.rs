use anyhow::{Ok, Result, ensure};
use log::info;
use std::{collections::HashMap, ffi::OsStr, fs, io::Read, path::PathBuf, process::Command};
use tempfile::NamedTempFile;
use which::which;

#[derive(Debug)]
pub struct Program {
    program_path: PathBuf,
    args: Vec<String>,
}

impl Program {
    pub fn from_name<T: AsRef<OsStr>>(name: T) -> Result<Self> {
        Ok(Self {
            program_path: which(name)?,
            args: Default::default(),
        })
    }

    pub fn sudo(&mut self) -> Result<&Self> {
        let old_program: PathBuf = self.program_path.clone();
        self.program_path = which("sudo")?;
        self.args.push(old_program.to_string_lossy().to_string());
        Ok(self)
    }

    pub fn new_command(&self) -> Command {
        let mut command = Command::new(self.program_path.as_path());
        command
            .env("DEBIAN_FRONTEND", "noninteractive")
            .args(self.args.clone());
        command
    }
}

pub fn run_or_sudo(mut cmd: Command) -> Result<String> {
    let stdout_file = NamedTempFile::new()?;
    let stderr_file = NamedTempFile::new()?;
    cmd.stdout(stdout_file.reopen()?)
        .stderr(stderr_file.reopen()?);

    let status = cmd.spawn()?.wait()?;
    let (status, stdout, stderr) = match status.code() {
        Some(100) => {
            info!("Command Fail: {cmd:?}, re-run with sudo...");

            let stdout_file = NamedTempFile::new()?;
            let stderr_file = NamedTempFile::new()?;

            let mut sudo_cmd = Command::new("sudo");
            sudo_cmd
                .arg("--")
                .arg(cmd.get_program())
                .args(cmd.get_args())
                .stdout(stdout_file.reopen()?)
                .stderr(stderr_file.reopen()?);

            let cmd_envs: HashMap<_, _> = cmd
                .get_envs()
                .filter_map(|(k, v)| v.map(|v| (k, v)))
                .collect();
            sudo_cmd.envs(cmd_envs);

            (
                sudo_cmd.status()?,
                fs::read_to_string(stdout_file)?,
                fs::read_to_string(stderr_file)?,
            )
        }
        _ => (
            status,
            fs::read_to_string(stdout_file)?,
            fs::read_to_string(stderr_file)?,
        ),
    };

    ensure!(
        status.success(),
        "COMMAND:\n {cmd:?}\n Out: {stdout} \n Err: {stderr} \n"
    );

    Ok(stdout)
}

pub fn run_command(mut cmd: Command) -> Result<String> {
    let mut stdout_file = NamedTempFile::new()?;
    let mut stderr_file = NamedTempFile::new()?;
    let mut child = cmd
        .stdout(stdout_file.reopen()?)
        .stderr(stderr_file.reopen()?)
        .spawn()?;
    let status = child.wait()?;

    let mut stdout = String::new();
    let mut stderr = String::new();

    stdout_file.read_to_string(&mut stdout)?;
    stderr_file.read_to_string(&mut stderr)?;

    ensure!(
        status.success(),
        "COMMAND:\n {cmd:?}\n Out: {stdout} \n Err: {stderr} \n"
    );

    Ok(stdout)
}
