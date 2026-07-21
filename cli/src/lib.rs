//! Command-line interface definition using clap.

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "labwc-rs",
    version,
    about = "A Wayland window-stacking compositor (Rust rewrite)"
)]
pub struct Cli {
    #[arg(short = 'c', long = "config")]
    pub config: Option<String>,

    #[arg(short = 'C', long = "config-dir")]
    pub config_dir: Option<String>,

    #[arg(short = 'd', long = "debug")]
    pub debug: bool,

    #[arg(short = 'e', long = "exit")]
    pub exit: bool,

    #[arg(short = 'm', long = "merge-config")]
    pub merge_config: bool,

    #[arg(short = 'r', long = "reconfigure")]
    pub reconfigure: bool,

    #[arg(short = 's', long = "startup")]
    pub startup_cmd: Option<String>,

    #[arg(short = 'S', long = "session")]
    pub session_cmd: Option<String>,

    #[arg(short = 't', long = "title")]
    pub title: Option<String>,

    #[arg(short = 'v', long = "version")]
    pub version: bool,

    #[arg(short = 'V', long = "verbose")]
    pub verbose: bool,
}
