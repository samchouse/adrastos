use adrastos_core::entities::{
    custom_table::schema::CustomTableSchema, Connection, RefreshTokenTree, User,
};
use clap::Parser;

use crate::commands::Commands;

mod commands;

#[derive(Parser)]
#[command(author, version, about, long_about = None, propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Apply => {
            // pub async fn migrate(db_pool: &deadpool_postgres::Pool) {
            //     let conn = db_pool.get().await.unwrap();
            //     let migrations = vec![
            //         User::migrate(),
            //         Connection::migrate(),
            //         RefreshTokenTree::migrate(),
            //         CustomTableSchema::migrate(),
            //     ];

            //     for migration in migrations {
            //         conn.execute(migration.as_str(), &[]).await.unwrap();
            //     }
            // }
        }
    }
}
