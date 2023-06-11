use clap::{Parser, Subcommand};

#[derive(Subcommand, PartialEq)]
pub enum Command {
    /// Apply all pending migrations
    Migrate,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None, propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}
