use fred::{clients::SubscriberClient, prelude::*};

use crate::config;

pub async fn create_pool_and_subscriber(config: &config::Config) -> (RedisPool, SubscriberClient) {
    let builder = Builder::from_config(RedisConfig::from_url(&config.redis_url).unwrap());

    let pool = builder.build_pool(16).unwrap();
    pool.init().await.unwrap();

    let subscriber = builder.build_subscriber_client().unwrap();
    subscriber.init().await.unwrap();

    (pool, subscriber)
}

pub fn build_key(config: &config::Config, key: String) -> String {
    config
        .redis_prefix
        .clone()
        .map(|p| format!("{p}:{key}"))
        .unwrap_or(key)
}
