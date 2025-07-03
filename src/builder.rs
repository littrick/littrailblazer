use anyhow::{Result, ensure};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::NamedTempFile;

pub fn build_target(package_name: &str, target: &str, release: bool) -> Result<PathBuf> {
    let mut build_cmd = Command::new(env!("CARGO"));

    build_cmd
        .arg("build")
        .arg("--bin")
        .arg(package_name)
        .arg("--target")
        .arg(target);

    let artifact_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("artifact");

    build_cmd
        .arg("-Z")
        .arg("unstable-options")
        .arg("--artifact-dir")
        .arg(artifact_dir.to_string_lossy().to_string());

    if release {
        build_cmd.arg("--release");
    }

    let status = build_cmd.status();
    ensure!(status.unwrap().success(), "Fail to run cargo build");

    Ok(artifact_dir.join(package_name))
}

pub fn base64_encode<P>(input: P) -> Result<String>
where
    P: AsRef<Path>,
{
    let output_file = NamedTempFile::new()?;

    let status = Command::new("base64")
        .arg(input.as_ref().to_string_lossy().to_string())
        .stdout(output_file.reopen()?)
        .status()?;

    ensure!(status.success(), "base64 encoding fail");

    Ok(fs::read_to_string(output_file)?)
}
