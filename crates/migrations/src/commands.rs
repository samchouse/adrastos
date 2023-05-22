use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Apply all pending migrations
    Apply,
}

// fn apply() {}
