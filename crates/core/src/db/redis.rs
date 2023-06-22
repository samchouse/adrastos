use deadpool_redis::{Manager, Pool};

use crate::config;

pub fn create_pool(config: &config::Config) -> Pool {
    let mgr = Manager::new(config.redis_url.clone()).unwrap();
    Pool::builder(mgr).max_size(16).build().unwrap()
}
