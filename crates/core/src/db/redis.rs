use deadpool_redis::{Manager, Pool};

use crate::config::{self, ConfigKey};

pub fn create_pool(config: config::Config) -> Pool {
    let mgr = Manager::new(config.get(ConfigKey::DragonflyUrl).unwrap()).unwrap();

    Pool::builder(mgr).max_size(16).build().unwrap()
}
