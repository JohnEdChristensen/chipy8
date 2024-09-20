use std::path::PathBuf;

use clap::{command, Parser};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    pub rom_path: PathBuf,

    #[arg(short, long)]
    pub paused: bool,
}
