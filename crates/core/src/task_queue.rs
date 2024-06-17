use std::{collections::HashMap, sync::Arc};

use futures::future::BoxFuture;
use tokio::{sync::RwLock, time};

use crate::id::Id;

pub struct TaskQueue(
    HashMap<
        Id,
        Box<
            dyn Fn(Box<dyn FnOnce() -> BoxFuture<'static, ()> + Send + Sync>) -> BoxFuture<'static, ()>
                + Send
                + Sync,
        >,
    >,
);

impl TaskQueue {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add_task<
        T: Fn(Box<dyn FnOnce() -> BoxFuture<'static, ()> + Send + Sync>) -> BoxFuture<'static, ()>
            + Send
            + Sync
            + 'static,
    >(
        &mut self,
        task: T,
    ) {
        self.0.insert(Id::new(), Box::new(task));
    }

    pub fn clear_task(&mut self, id: Id) {
        self.0.remove(&id);
    }

    pub fn run(queue: Arc<RwLock<Self>>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = time::interval(tokio::time::Duration::from_millis(500));

            loop {
                interval.tick().await;

                let tasks = queue.read().await;
                let tasks = tasks.0.iter();
                for (id, task) in tasks {
                    let id = id.clone();
                    let queue = Arc::clone(&queue);

                    task(Box::new(move || {
                        Box::pin(async move {
                            queue.write().await.clear_task(id);
                        })
                    }))
                    .await;
                }
            }
        })
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}
