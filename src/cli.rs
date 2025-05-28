use clap::{Parser, command};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub enum Args {
    Install {
        #[arg(required = true, value_name = "CONFIG", num_args = 1..)]
        configs: Vec<PathBuf>,
    },
    Uninstall {},
}
