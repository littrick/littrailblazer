use std::{fs, path::PathBuf};

use anyhow::{Context, Result, anyhow};
use clap::{ArgGroup, Parser, command};
use distro_pioneer::builder::{base64_encode, build_target};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(group = ArgGroup::new("bin-target").required(true).multiple(false))]
#[command(about = "打包检查脚本")]
struct Args {
    /// 编译需要cargo打包的包名
    #[arg(short, name = "package name", group = "bin-target")]
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

    /// 输出脚本的文件名
    #[arg(short, default_value = "checker.sh")]
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

    let mk_bin = {
        let mk_file = format!("bin_exe=$(mktemp --suffix=.bin)");
        let decode = format!("base64 -d > ${{bin_exe}} <<'EOF'\n{}\nEOF", base64);
        let chmod = "chmod +x ${bin_exe}";
        format!("\n{}\n{}\n{}\n", mk_file, decode, chmod)
    };

    let pass_run = "${bin_exe} ${@:1}";
    let rm_bin = "rm ${bin_exe}";

    let script = format!(
        "#!/bin/bash\n{}\n{}\n{}\n{}\n",
        mk_bin, mk_bin, pass_run, rm_bin,
    );

    fs::write(&args.output, script).context(format!(
        "Fail to create install script: {}",
        args.output.to_string_lossy()
    ))?;

    println!(
        "checking script create at {}",
        args.output.to_string_lossy()
    );

    Ok(())
}
