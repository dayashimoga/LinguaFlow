use std::collections::HashMap;
use std::collections::VecDeque;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
}

#[derive(Clone)]
pub struct BoundedLruCache<K, V> {
    inner: Arc<Mutex<InnerLru<K, V>>>,
}

struct InnerLru<K, V> {
    map: HashMap<K, V>,
    order: VecDeque<K>,
    capacity: usize,
    stats: CacheStats,
}

impl<K: Clone + Eq + Hash, V: Clone> BoundedLruCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "LRU capacity must be strictly greater than 0");
        Self {
            inner: Arc::new(Mutex::new(InnerLru {
                map: HashMap::with_capacity(capacity),
                order: VecDeque::with_capacity(capacity),
                capacity,
                stats: CacheStats::default(),
            })),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let mut inner = self.inner.lock().ok()?;
        if let Some(val) = inner.map.get(key).cloned() {
            inner.stats.hits += 1;
            // Promote key to most-recently used (back of deque)
            if let Some(pos) = inner.order.iter().position(|k| k == key) {
                inner.order.remove(pos);
                inner.order.push_back(key.clone());
            }
            Some(val)
        } else {
            inner.stats.misses += 1;
            None
        }
    }

    pub fn insert(&self, key: K, value: V) {
        if let Ok(mut inner) = self.inner.lock() {
            if inner.map.contains_key(&key) {
                inner.map.insert(key.clone(), value);
                if let Some(pos) = inner.order.iter().position(|k| k == &key) {
                    inner.order.remove(pos);
                }
                inner.order.push_back(key);
            } else {
                if inner.map.len() >= inner.capacity {
                    // Evict least-recently used (front of deque)
                    if let Some(oldest_key) = inner.order.pop_front() {
                        inner.map.remove(&oldest_key);
                        inner.stats.evictions += 1;
                    }
                }
                inner.map.insert(key.clone(), value);
                inner.order.push_back(key);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.inner.lock().map(|i| i.map.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn capacity(&self) -> usize {
        self.inner.lock().map(|i| i.capacity).unwrap_or(0)
    }

    pub fn set_capacity(&self, new_cap: usize) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.capacity = new_cap.max(1);
            while inner.map.len() > inner.capacity {
                if let Some(oldest_key) = inner.order.pop_front() {
                    inner.map.remove(&oldest_key);
                    inner.stats.evictions += 1;
                }
            }
        }
    }

    pub fn stats(&self) -> CacheStats {
        self.inner.lock().map(|i| i.stats).unwrap_or_default()
    }

    pub fn clear(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.map.clear();
            inner.order.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache_bounds_and_eviction() {
        let cache = BoundedLruCache::new(3);
        assert_eq!(cache.capacity(), 3);
        assert_eq!(cache.len(), 0);

        cache.insert("a", 1);
        cache.insert("b", 2);
        cache.insert("c", 3);
        assert_eq!(cache.len(), 3);

        // Access "a" to make it more recently used than "b"
        assert_eq!(cache.get(&"a"), Some(1));

        // Insert "d" -> "b" should be evicted
        cache.insert("d", 4);
        assert_eq!(cache.len(), 3);
        assert_eq!(cache.get(&"b"), None);
        assert_eq!(cache.get(&"a"), Some(1));
        assert_eq!(cache.get(&"c"), Some(3));
        assert_eq!(cache.get(&"d"), Some(4));

        let stats = cache.stats();
        assert_eq!(stats.hits, 4);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.evictions, 1);
    }

    #[test]
    fn test_lru_capacity_resizing() {
        let cache = BoundedLruCache::new(4);
        cache.insert("1", 10);
        cache.insert("2", 20);
        cache.insert("3", 30);
        cache.insert("4", 40);

        cache.set_capacity(2);
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.capacity(), 2);
        assert_eq!(cache.stats().evictions, 2);
    }
}
