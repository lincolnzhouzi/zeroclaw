use crate::error::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub struct CacheEntry {
    pub key: String,
    pub tokens: Vec<i32>,
    pub embedding: Vec<f32>,
    pub timestamp: std::time::Instant,
}

pub struct KVContextCache {
    max_entries: usize,
    cache: Arc<RwLock<Vec<CacheEntry>>>,
    kv_cache: Arc<RwLock<HashMap<String, KVCache>>>,
}

#[derive(Clone, Debug)]
pub struct KVCache {
    pub key_cache: Vec<Vec<f32>>,
    pub value_cache: Vec<Vec<f32>>,
    pub layer_count: usize,
    pub head_count: usize,
    pub head_dim: usize,
}

impl KVCache {
    pub fn new(layer_count: usize, head_count: usize, head_dim: usize) -> Self {
        Self {
            key_cache: Vec::with_capacity(layer_count),
            value_cache: Vec::with_capacity(layer_count),
            layer_count,
            head_count,
            head_dim,
        }
    }

    pub fn seq_len(&self) -> usize {
        if self.key_cache.is_empty() {
            0
        } else {
            self.key_cache[0].len() / (self.head_count * self.head_dim)
        }
    }

    pub fn clear(&mut self) {
        self.key_cache.clear();
        self.value_cache.clear();
    }

    pub fn append(&mut self, keys: Vec<Vec<f32>>, values: Vec<Vec<f32>>) {
        self.key_cache.extend(keys);
        self.value_cache.extend(values);
    }
}

impl KVContextCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            max_entries,
            cache: Arc::new(RwLock::new(Vec::new())),
            kv_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, key: &str) -> Option<Vec<i32>> {
        let cache = self.cache.read().await;
        cache
            .iter()
            .find(|e| e.key == key)
            .map(|e| e.tokens.clone())
    }

    pub async fn put(&self, key: String, tokens: Vec<i32>) {
        let mut cache = self.cache.write().await;

        if cache.len() >= self.max_entries {
            cache.remove(0);
        }

        cache.push(CacheEntry {
            key,
            tokens,
            embedding: Vec::new(),
            timestamp: std::time::Instant::now(),
        });
    }

    pub async fn put_with_embedding(&self, key: String, tokens: Vec<i32>, embedding: Vec<f32>) {
        let mut cache = self.cache.write().await;

        if cache.len() >= self.max_entries {
            cache.remove(0);
        }

        cache.push(CacheEntry {
            key,
            tokens,
            embedding,
            timestamp: std::time::Instant::now(),
        });
    }

    pub async fn get_kv_cache(&self, session_id: &str) -> Option<KVCache> {
        let kv = self.kv_cache.read().await;
        kv.get(session_id).cloned()
    }

    pub async fn set_kv_cache(&self, session_id: String, cache: KVCache) {
        let mut kv = self.kv_cache.write().await;
        kv.insert(session_id, cache);
    }

    pub async fn clear_kv_cache(&self, session_id: &str) {
        let mut kv = self.kv_cache.write().await;
        kv.remove(session_id);
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();

        let mut kv = self.kv_cache.write().await;
        kv.clear();
    }

    pub async fn len(&self) -> usize {
        self.cache.read().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.cache.read().await.is_empty()
    }

    pub async fn evict_expired(&self, max_age_secs: u64) {
        let mut cache = self.cache.write().await;
        let now = std::time::Instant::now();
        cache.retain(|e| now.duration_since(e.timestamp).as_secs() < max_age_secs);
    }
}

pub struct ContextManager {
    cache: KVContextCache,
    active_session: Arc<RwLock<Option<String>>>,
}

impl ContextManager {
    pub fn new(cache_size: usize) -> Self {
        Self {
            cache: KVContextCache::new(cache_size),
            active_session: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn create_session(&self) -> String {
        let session_id = uuid::Uuid::new_v4().to_string();
        let mut session = self.active_session.write().await;
        *session = Some(session_id.clone());
        session_id
    }

    pub async fn get_active_session(&self) -> Option<String> {
        self.active_session.read().await.clone()
    }

    pub async fn set_active_session(&self, session_id: String) {
        let mut session = self.active_session.write().await;
        *session = Some(session_id);
    }

    pub async fn clear_session(&self) {
        let mut session = self.active_session.write().await;
        *session = None;
    }

    pub fn cache(&self) -> &KVContextCache {
        &self.cache
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_put_get() {
        let cache = KVContextCache::new(10);
        cache.put("key1".to_string(), vec![1, 2, 3]).await;

        let result = cache.get("key1").await;
        assert!(result.is_some());
        assert_eq!(result.unwrap(), vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_cache_eviction() {
        let cache = KVContextCache::new(2);
        cache.put("key1".to_string(), vec![1]).await;
        cache.put("key2".to_string(), vec![2]).await;
        cache.put("key3".to_string(), vec![3]).await;

        assert_eq!(cache.len().await, 2);
        assert!(cache.get("key1").await.is_none());
    }

    #[tokio::test]
    async fn test_kv_cache() {
        let kv_cache = KVCache::new(12, 8, 64);
        assert_eq!(kv_cache.seq_len(), 0);
        assert_eq!(kv_cache.layer_count, 12);
    }

    #[tokio::test]
    async fn test_context_manager() {
        let manager = ContextManager::new(10);
        let session_id = manager.create_session().await;

        assert!(manager.get_active_session().await.is_some());
        assert_eq!(manager.get_active_session().await, Some(session_id));
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = KVContextCache::new(10);
        cache.put("key1".to_string(), vec![1]).await;
        cache.put("key2".to_string(), vec![2]).await;

        cache.clear().await;
        assert!(cache.is_empty().await);
    }
}
