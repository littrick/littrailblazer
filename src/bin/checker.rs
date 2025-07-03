use anyhow::{Context, Result};
use clap::{Parser, command};
use distro_pioneer::types::config::Config;
use glob::glob;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 需要检查的文件
    #[arg(name = "toml file")]
    configs: Vec<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let all_toml = if args.configs.len() == 0 {
        glob("*.toml")
            .context("Fail to find toml file")?
            .filter_map(Result::ok)
            .collect()
    } else {
        args.configs
    };

    for config_file in all_toml {
        let config = fs::read_to_string(&config_file).context(format!(
            "Fail to read file: {}",
            config_file.to_string_lossy()
        ))?;

        let _: Config = toml::from_str(&config)
            .context(format!("File {} is invaild", config_file.to_string_lossy()))?;
    }

    Ok(())
}
