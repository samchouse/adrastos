use crate::config::Config;
use crate::db::create_pool;

use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use std::process::exit;

mod config;
mod db;
mod handlers;
mod id;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config = Config::new();
    match config {
        Ok(config) => {
            let cfg = config.clone();
            let server = HttpServer::new(move || {
                App::new()
                    .app_data(web::Data::new(cfg.clone()))
                    .app_data(web::Data::new(create_pool(cfg.clone())))
                    .service(handlers::index)
            })
            .bind(config.url.as_ref().unwrap().to_string())?
            .run();

            let (server, _) = tokio::join!(
                server,
                server_started(config.url.unwrap())
            );

            server
        }
        Err(errors) => {
            println!("Missing required environment variables: {:#?}", errors);
            exit(1)
        }
    }
}

async fn server_started(url: String) {
    println!("Server started at http://{}", url);
}
