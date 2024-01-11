//! A non-thread-safe implementation of an S3-FIFO
//! Paper here: https://jasony.me/publication/sosp23-s3fifo.pdf

use std::collections::hash_map::DefaultHasher;
use std::collections::VecDeque;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

///
/// S3FIFO requires a key. This is a
/// convenience struct that will generate
/// a key from a value that is Hash.
///
#[derive(Clone)]
pub struct S3FIFOKey<V: Hash> {
    hash: u64,
    _phantom: PhantomData<V>,
}

/// S3FIFO is a non-thread-safe implementation of an S3-FIFO
///
/// Paper here: https://jasony.me/publication/sosp23-s3fifo.pdf
///
/// S3FIFO is a cache that is split into three parts:
/// 1. A small cache that holds the most recently used items
/// 2. A main cache that holds the most frequently used items
/// 3. A ghost cache that holds keys that have been evicted from the main cache
///
/// ```
/// use s3_fifo::{S3FIFO, S3FIFOKey};
///
/// // The cached value must be Clone.
/// // Hash is optional and allows using the S3FIFOKey struct
/// #[derive(Clone, Hash)]
/// struct Foobar { a: i32 }
///
/// // Create a cache with a capacity of 128 (small: 12, main: 115, ghost: 115)
/// let mut cache = S3FIFO::new(128);
/// let value = Foobar { a: 1 };
/// let key = S3FIFOKey::new(&value);
///
/// // Check if the item is in the cache before inserting
/// if let None = cache.get(&key) {
///     cache.put(key.clone(), value);
///     assert!(cache.get(&key).is_some());
/// }
/// ````
pub struct S3FIFO<K, V> {
    small: VecDeque<Item<K, V>>,
    main: VecDeque<Item<K, V>>,
    ghost: VecDeque<Key<K>>,
}

impl<K: PartialEq + Clone, V> S3FIFO<K, V> {
    ///
    /// Create a new S3FIFO cache with 10% of the capacity for
    /// the small cache and 90% of the capacity for the main cache.
    ///
    /// The ghost cache is also 90% of the capacity but only holds
    /// keys and not values.
    ///
    pub fn new(capacity: usize) -> Self {
        let small_capacity = capacity / 10;
        let main_capacity = capacity * 9 / 10;
        S3FIFO {
            small: VecDeque::with_capacity(small_capacity),
            main: VecDeque::with_capacity(main_capacity),
            ghost: VecDeque::with_capacity(main_capacity),
        }
    }

    /// Read an item from the cache.
    /// If the item is present, then its frequency is incremented and a reference is returned.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        // Check item in small
        if let Some(item) = self.small.iter_mut().find(|item| item.key == *key) {
            item.freq = (item.freq + 1) & 0b11;
            return Some(&item.value);
        }

        // Check item in main
        if let Some(item) = self.main.iter_mut().find(|item| item.key == *key) {
            item.freq = (item.freq + 1) & 0b11;
            return Some(&item.value);
        }

        None
    }

    /// Write an item to the cache.
    /// This may evict items from the cache.
    pub fn put(&mut self, key: K, value: V) -> &V {
        // Check if item is in ghost to decide where to insert
        if let Some(key) = self.ghost.iter().find(|k| k.key == key) {
            let item = Item {
                key: key.key.clone(),
                value,
                freq: key.freq,
            };
            if self.main.capacity() == self.main.len() {
                self.evict_main();
            }
            self.main.push_front(item);
            return &self.main.front().unwrap().value;
        } else {
            let item = Item {
                key,
                value,
                freq: 0,
            };
            if self.small.capacity() == self.small.len() {
                self.evict_small();
            }
            self.small.push_front(item);
            return &self.small.front().unwrap().value;
        }
    }

    fn evict_small(&mut self) {
        let mut evicted = false;
        while !evicted && !self.small.is_empty() {
            let item = self.small.pop_back().unwrap();
            if item.freq > 1 {
                if self.main.capacity() == self.main.len() {
                    self.evict_main();
                }
                self.main.push_front(item);
            } else {
                self.ghost.push_front(item.into());
                evicted = true;
            }
        }
    }

    fn evict_main(&mut self) {
        let mut evicted = false;
        while !evicted && !self.main.is_empty() {
            let mut item = self.main.pop_back().unwrap();
            if item.freq > 0 {
                item.freq -= 1;
                self.main.push_front(item);
            } else {
                evicted = true;
            }
        }
    }
}

struct Item<K, V> {
    key: K,
    value: V,
    freq: u8,
}

struct Key<K> {
    key: K,
    freq: u8,
}

impl<K, V> From<Item<K, V>> for Key<K> {
    fn from(item: Item<K, V>) -> Self {
        Key {
            key: item.key,
            freq: item.freq,
        }
    }
}

impl<V: Hash> PartialEq for S3FIFOKey<V> {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl<V: Hash> S3FIFOKey<V> {
    ///
    /// Create a new S3FIFOKey from a value that is Hash.
    ///
    /// This will generate a hash from the value and act as the key.
    ///
    pub fn new(value: &V) -> Self {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        S3FIFOKey {
            hash: hasher.finish(),
            _phantom: PhantomData,
        }
    }
}

impl<V: Hash> std::fmt::Debug for S3FIFOKey<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("S3FIFOKey")
            .field("hash", &self.hash)
            .finish()
    }
}

impl<V: Hash> Display for S3FIFOKey<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#x}", self.hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    #[derive(Hash, Clone)]
    struct Abc {
        a: u8,
        b: u16,
        c: u32,
    }

    #[test]
    fn can_use_cache() {
        let test_value = Abc { a: 1, b: 2, c: 3 };
        let mut hasher = DefaultHasher::new();
        test_value.hash(&mut hasher);
        let test_key = hasher.finish();

        // Create a cache with a capacity of 10
        let mut cache = S3FIFO::new(10);
        cache.put(test_key, test_value);
        assert!(cache.get(&test_key).is_some());
    }

    #[test]
    fn can_fill_cache() {
        // Create a cache with a capacity of 10
        let mut cache = S3FIFO::new(10);
        for i in 0..10 {
            let test_value = Abc {
                a: i as u8,
                b: i as u16,
                c: i as u32,
            };
            let test_key = S3FIFOKey::new(&test_value);

            assert!(cache.get(&test_key).is_none());
            cache.put(test_key.clone(), test_value);
            assert!(cache.get(&test_key).is_some());
        }

        assert_eq!(cache.small.len(), 1);
        assert_eq!(cache.ghost.len(), 9);

        // Promote to main
        let repeat_value = Abc { a: 0, b: 0, c: 0 };
        let repeat_key = S3FIFOKey::new(&repeat_value);
        assert!(cache.get(&repeat_key).is_none());
        cache.put(repeat_key, repeat_value);

        assert_eq!(cache.small.len(), 1);
        assert_eq!(cache.ghost.len(), 9);
        assert_eq!(cache.main.len(), 1);

        // Increment main
        let repeat_value = Abc { a: 0, b: 0, c: 0 };
        let repeat_key = S3FIFOKey::new(&repeat_value);
        assert!(cache.get(&repeat_key).is_some());
        // Do not insert again or else duplicate keys will be in the cache
        // cache.put(repeat_key, repeat_value);

        assert_eq!(cache.small.len(), 1);
        assert_eq!(cache.ghost.len(), 9);
        assert_eq!(cache.main.len(), 1);
    }
}