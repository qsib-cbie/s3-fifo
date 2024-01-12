use std::collections::hash_map::DefaultHasher;
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