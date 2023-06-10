use clap::Subcommand;

#[derive(Subcommand, PartialEq)]
pub enum Command {
    /// Apply all pending migrations
    Migrate,
}
