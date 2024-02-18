use std::{collections::HashMap, hash::Hash, sync::Arc};

use chrono::{DateTime, Duration, Utc};
use tokio::{sync::RwLock, time};

struct ExpiringMapEntry<T> {
    value: Arc<T>,
    expires_at: DateTime<Utc>,
}

pub struct ExpiringMap<K, V>(HashMap<K, ExpiringMapEntry<V>>);

impl<K: Eq + PartialEq + Hash + Send + Sync + 'static, V: Send + Sync + 'static> ExpiringMap<K, V> {
    pub fn new() -> Self {
        ExpiringMap(HashMap::new())
    }

    pub fn insert(&mut self, key: K, value: V, expires_in: Duration) {
        self.0.insert(
            key,
            ExpiringMapEntry {
                value: Arc::new(value),
                expires_at: Utc::now() + expires_in,
            },
        );
    }

    pub fn get(&self, key: &K) -> Option<Arc<V>> {
        self.0.get(key).and_then(|entry| {
            if entry.expires_at > Utc::now() {
                Some(entry.value.clone())
            } else {
                None
            }
        })
    }

    pub fn reset_expiry(&mut self, key: &K, expires_in: Duration) {
        if let Some(entry) = self.0.get_mut(key) {
            entry.expires_at = Utc::now() + expires_in;
        }
    }

    pub fn clear_expired(&mut self) {
        let now = Utc::now();
        self.0.retain(|_, entry| entry.expires_at > now);
    }

    pub async fn start_expiry_worker(
        map: Arc<RwLock<Self>>,
        duration: tokio::time::Duration,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = time::interval(duration);

            loop {
                interval.tick().await;
                map.write().await.clear_expired();
            }
        })
    }
}

impl<K: Eq + PartialEq + Hash + Send + Sync + 'static, V: Send + Sync + 'static> Default
    for ExpiringMap<K, V>
{
    fn default() -> Self {
        Self::new()
    }
}
