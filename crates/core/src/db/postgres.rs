use deadpool_postgres::{
    tokio_postgres::{Config, NoTls},
    Manager, ManagerConfig, Pool, RecyclingMethod,
};

use crate::config::{self, ConfigKey};

pub fn create_pool(config: config::Config) -> Pool {
    let pg_config = config
        .get(ConfigKey::CockroachUrl)
        .unwrap()
        .parse::<Config>()
        .unwrap();
    let mgr = Manager::from_config(
        pg_config,
        NoTls,
        ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        },
    );

    Pool::builder(mgr).max_size(16).build().unwrap()
}
