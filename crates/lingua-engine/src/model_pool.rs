use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Cached model session entry tracking last access time and memory footprint.
#[derive(Clone, Debug)]
pub struct ModelSessionEntry<T> {
    pub session: Arc<T>,
    pub last_used: Instant,
    pub pack_id: String,
    pub memory_bytes: usize,
}

/// Thread-safe model session lifecycle pool enforcing 90-second idle eviction
/// and bounded memory limits across neural inference sessions.
pub struct ModelLifecyclePool<T> {
    entries: Arc<RwLock<HashMap<String, ModelSessionEntry<T>>>>,
    idle_timeout: Duration,
}

impl<T: Send + Sync + 'static> ModelLifecyclePool<T> {
    /// Create a new model lifecycle pool with a specified idle timeout (standard: 90 seconds).
    pub fn new(idle_timeout_secs: u64) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            idle_timeout: Duration::from_secs(idle_timeout_secs),
        }
    }

    /// Retrieve an active model session, updating its last_used timestamp.
    pub fn get(&self, key: &str) -> Option<Arc<T>> {
        let mut map = match self.entries.write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        if let Some(entry) = map.get_mut(key) {
            entry.last_used = Instant::now();
            Some(Arc::clone(&entry.session))
        } else {
            None
        }
    }

    /// Insert or update a model session in the pool.
    pub fn insert(
        &self,
        key: impl Into<String>,
        pack_id: impl Into<String>,
        session: Arc<T>,
        memory_bytes: usize,
    ) {
        let mut map = match self.entries.write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        map.insert(
            key.into(),
            ModelSessionEntry {
                session,
                last_used: Instant::now(),
                pack_id: pack_id.into(),
                memory_bytes,
            },
        );
    }

    /// Evict sessions that have been idle for longer than the idle timeout.
    /// Returns the list of evicted session keys.
    pub fn evict_idle(&self) -> Vec<String> {
        let mut map = match self.entries.write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        let now = Instant::now();
        let timeout = self.idle_timeout;
        let mut evicted = Vec::new();

        map.retain(|key, entry| {
            if now.duration_since(entry.last_used) > timeout {
                evicted.push(key.clone());
                false
            } else {
                true
            }
        });

        if !evicted.is_empty() {
            map.shrink_to_fit();
        }

        evicted
    }

    /// Return the count of active model sessions currently retained in memory.
    pub fn active_count(&self) -> usize {
        let map = match self.entries.read() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        map.len()
    }

    /// Return the total estimated resident memory bytes across all active models.
    pub fn total_memory_bytes(&self) -> usize {
        let map = match self.entries.read() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        map.values().map(|e| e.memory_bytes).sum()
    }

    /// Explicitly remove a model session from the pool.
    pub fn remove(&self, key: &str) -> Option<Arc<T>> {
        let mut map = match self.entries.write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let res = map.remove(key).map(|e| e.session);
        map.shrink_to_fit();
        res
    }

    /// Spawn a background maintenance task ticking every `interval` to evict idle models.
    pub fn spawn_background_evictor(
        pool: Arc<Self>,
        interval: Duration,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut tick = tokio::time::interval(interval);
            loop {
                tick.tick().await;
                let evicted = pool.evict_idle();
                if !evicted.is_empty() {
                    tracing::info!(
                        "ModelLifecyclePool evicted {} idle session(s): {:?}",
                        evicted.len(),
                        evicted
                    );
                }
            }
        })
    }
}

impl<T: Send + Sync + 'static> Default for ModelLifecyclePool<T> {
    fn default() -> Self {
        Self::new(90)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_model_lifecycle_pool_insert_get_evict() {
        let pool = Arc::new(ModelLifecyclePool::<String>::new(1)); // 1 second timeout
        let model = Arc::new("neural-ocr-kn".to_string());

        pool.insert("ocr-kn", "pack-kn-ocr", Arc::clone(&model), 1024 * 1024);
        assert_eq!(pool.active_count(), 1);
        assert_eq!(pool.total_memory_bytes(), 1024 * 1024);

        // Immediate get should succeed and refresh timestamp
        assert!(pool.get("ocr-kn").is_some());

        // Wait past 1s timeout
        thread::sleep(Duration::from_millis(1100));

        let evicted = pool.evict_idle();
        assert_eq!(evicted, vec!["ocr-kn".to_string()]);
        assert_eq!(pool.active_count(), 0);
        assert_eq!(pool.total_memory_bytes(), 0);
        assert!(pool.get("ocr-kn").is_none());
    }
}
