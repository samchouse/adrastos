use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    /// Apply all pending migrations
    Migrate,
}
