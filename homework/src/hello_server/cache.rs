//! Thread-safe key/value cache.

use std::collections::hash_map::{Entry, HashMap, DefaultHasher};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, RwLock};

const SHARD_COUNT: usize = 16;
/// Cache that remembers the result for each key.
#[derive(Debug)]
pub struct Cache<K, V> {
    // todo! This is an example cache type. Build your own cache type that satisfies the
    // specification for `get_or_insert_with`.
    inner: Vec<RwLock<HashMap<K, V>>>,
}

impl<K, V> Default for Cache<K, V> {
    fn default() -> Self {
        let mut cache = Vec::with_capacity(SHARD_COUNT);
        for _ in 0..SHARD_COUNT {
            cache.push(RwLock::new(HashMap::new()));
        }
        Self {
            inner: cache,
        }
    }
}

impl<K: Eq + Hash + Clone, V: Clone> Cache<K, V> {
    /// Retrieve the value or insert a new one created by `f`.
    ///
    /// An invocation to this function should not block another invocation with a different key. For
    /// example, if a thread calls `get_or_insert_with(key1, f1)` and another thread calls
    /// `get_or_insert_with(key2, f2)` (`key1≠key2`, `key1,key2∉cache`) concurrently, `f1` and `f2`
    /// should run concurrently.
    ///
    /// On the other hand, since `f` may consume a lot of resource (= money), it's undesirable to
    /// duplicate the work. That is, `f` should be run only once for each key. Specifically, even
    /// for concurrent invocations of `get_or_insert_with(key, f)`, `f` is called only once per key.
    ///
    /// Hint: the [`Entry`] API may be useful in implementing this function.
    ///
    /// [`Entry`]: https://doc.rust-lang.org/stable/std/collections/hash_map/struct.HashMap.html#method.entry
    pub fn get_or_insert_with<F: FnOnce(K) -> V>(&self, key: K, f: F) -> V {
        let shard_index = self.get_shard(&key);

        if let Some(value) = self.inner[shard_index].read().unwrap().get(&key).cloned() {
            return value;
        }

        let mut shard = self.inner[shard_index].write().unwrap();
        shard.entry(key.clone()).or_insert_with(|| f(key.clone())).clone()
    }

    fn get_shard(&self, key: &K) -> usize {
        let hash = DefaultHasher::default();

        let mut hasher = hash;
        key.hash(&mut hasher);
        (hasher.finish() as usize) % SHARD_COUNT
    }
}
