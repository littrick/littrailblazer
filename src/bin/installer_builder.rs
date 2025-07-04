use anyhow::{Context, Result, anyhow};
use clap::{ArgGroup, Parser, command};
use distro_pioneer::{
    builder::{base64_encode, build_target, unique_string},
    types::config::Config,
};
use std::{fs, path::PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(group = ArgGroup::new("bin-target").required(true).multiple(false))]
#[command(about = "打包安装脚本")]
struct Args {
    /// 编译需要cargo打包的包名
    #[arg(name = "package name", group = "bin-target")]
    package: Option<String>,

    /// 选择已编译好的二进制/可执行文件
    #[arg(short, long, name = "binary path", group = "bin-target")]
    bin: Option<PathBuf>,

    /// 编译目标
    #[arg(short, name = "triple", default_value = "x86_64-unknown-linux-musl")]
    target: String,

    /// 编译release版本
    #[arg(short, default_value_t = true)]
    release: bool,

    /// 需要一起打包到script的配置文件，至少提供一个
    #[arg(short, required = true, name=".toml", num_args = 1..)]
    configs: Vec<PathBuf>,

    /// 输出脚本的文件名
    #[arg(short, default_value = "installer.sh")]
    output: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let bin = {
        if let Some(package) = &args.package {
            build_target(package, &args.target, args.release)?
        } else {
            args.bin
                .ok_or(anyhow!("no binary executable path provided"))?
        }
    };

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

    let eof = unique_string();

    let mut mk_configs = "config_files=()\n".to_string();
    for config in configs {
        let msg = format!("echo making {}.toml...", config.infomation.name);
        let mk_file = format!(
            "config_file=$(mktemp --suffix=.{}.toml)",
            config.infomation.name
        );
        let config_file = format!(
            "cat > ${{config_file}} <<'{eof}'\n{}\n{eof}",
            toml::to_string_pretty(&config)?
        );
        let append_configs = "config_files=(${config_files[@]} $config_file)";

        mk_configs.push_str(&format!(
            "\n{}\n{}\n{}\n{}\n",
            msg, mk_file, config_file, append_configs
        ));
    }

    let mk_bin = {
        let mk_file = format!("bin_exe=$(mktemp --suffix=.bin)");
        let decode = format!("base64 -d > ${{bin_exe}} <<'{eof}'\n{}\n{eof}", base64);
        let chmod = "chmod +x ${bin_exe}";
        format!("\n{}\n{}\n{}\n", mk_file, decode, chmod)
    };

    let run_bin = "RUST_LOG=info ${bin_exe} install ${config_files[@]}";
    let pass_run = format!("test $# -eq 0 && {run_bin} || ${{bin_exe}} ${{@:1}}");

    let rm_bin = "rm ${bin_exe}";
    let rm_configs = "rm ${config_files[@]}";

    let script = format!(
        "#!/bin/bash\n{}\n{}\n{}\n{}\n{}\n",
        mk_configs, mk_bin, pass_run, rm_bin, rm_configs,
    );

    fs::write(&args.output, script).context(format!(
        "Fail to create install script: {}",
        args.output.to_string_lossy()
    ))?;

    println!(
        "installing script create at {}",
        args.output.to_string_lossy()
    );

    Ok(())
}
