use deadpool_postgres::{
    tokio_postgres::{Config, NoTls},
    Manager, ManagerConfig, Pool, RecyclingMethod,
};

use crate::config;

pub fn create_pool(config: config::Config) -> Pool {
    let pg_config = config.cockroach_url.unwrap().parse::<Config>().unwrap();
    let mgr = Manager::from_config(
        pg_config,
        NoTls,
        ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        },
    );

    deadpool_postgres::Pool::builder(mgr)
        .max_size(16)
        .build()
        .unwrap()
}
