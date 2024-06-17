use std::sync::Arc;

use adrastos_core::{config::Config, db::postgres::Databases, s3::S3, task_queue::TaskQueue};
use fred::clients::SubscriberClient;
use tokio::sync::RwLock;

#[derive(PartialEq, Clone, Debug)]
pub enum Flag {
    AllowAuthParam,
    AllowProjectIdParam,
}

#[derive(Clone)]
pub struct AppState {
    pub s3: Arc<S3>,
    pub config: Config,
    pub databases: Arc<Databases>,
    pub subscriber: SubscriberClient,
    pub flags: Vec<(String, Vec<Flag>)>,
    pub task_queue: Arc<RwLock<TaskQueue>>,
    pub redis_pool: fred::clients::RedisPool,
}
