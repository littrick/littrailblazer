use anyhow::{Context, Ok, Result, ensure};
use clap::{Parser, command};
use distro_pioneer::types::config::Config;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::NamedTempFile;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    package: String,

    #[arg(short, default_value = "x86_64-unknown-linux-musl")]
    target: String,

    #[arg(short, default_value_t = true)]
    release: bool,

    #[arg(short, required = true, num_args = 1..)]
    configs: Vec<PathBuf>,

    #[arg(short, default_value = "installer.sh")]
    output: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let bin = build_target(&args.package, &args.target, args.release)?;

    let base64 = base64_encode(bin)?;

    let mut configs = Vec::new();

    for config_file in &args.configs {
        let config_str = fs::read_to_string(config_file).context(format!(
            "Fail to read file: {}",
            config_file.to_string_lossy()
        ))?;

        let config: Config = toml::from_str(config_str.as_str())?;

        configs.push(config);
    }

    let mut mk_configs = "config_files=()\n".to_string();
    for config in configs {
        let msg = format!("echo making {}.toml...", config.infomation.name);
        let mk_file = format!(
            "config_file=$(mktemp --suffix=.{}.toml)",
            config.infomation.name
        );
        let config_file = format!(
            "cat > ${{config_file}} <<'EOF'\n{}\nEOF",
            toml::to_string_pretty(&config)?
        );
        let append_configs = "config_files=(${config_files[@]} $config_file)";

        mk_configs.push_str(&format!(
            "\n{}\n{}\n{}\n{}\n",
            msg, mk_file, config_file, append_configs
        ));
    }

    let mk_bin = {
        let mk_file = format!("bin_exe=$(mktemp --suffix=.{}.bin)", args.package);
        let decode = format!("base64 -d > ${{bin_exe}} <<'EOF'\n{}\nEOF", base64);
        let chmod = "chmod +x ${bin_exe}";
        format!("\n{}\n{}\n{}\n", mk_file, decode, chmod)
    };

    let run_bin = "RUST_LOG=info ${bin_exe} install ${config_files[@]}";
    let rm_bin = "rm ${bin_exe}";
    let rm_configs = "rm ${config_files[@]}";

    let script = format!(
        "#!/bin/bash\n{}\n{}\n{}\n{}\n{}",
        mk_configs, mk_bin, run_bin, rm_bin, rm_configs,
    );

    fs::write(&args.output, script).context(format!(
        "Fail to create install script: {}",
        args.output.to_string_lossy()
    ))?;

    println!("install script create at {}", args.output.to_string_lossy());

    Ok(())
}

fn build_target(package_name: &str, target: &str, release: bool) -> Result<PathBuf> {
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

fn base64_encode<P>(input: P) -> Result<String>
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
