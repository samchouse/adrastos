use clap::Parser;

use self::commands::Command;

pub mod commands;

#[derive(Parser)]
#[command(author, version, about, long_about = None, propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}
