[![Crate](https://img.shields.io/crates/v/s3-fifo)](https://crates.io/crates/s3-fifo)

# S3-FIFO Cache


This library contains an implementation for a non-thread-safe S3-FIFO cache as per the paper at https://jasony.me/publication/sosp23-s3fifo.pdf

Here is the abstract from the paper.

> As a cache eviction algorithm, FIFO has a lot of attractive properties, such as simplicity, speed, scalability, and flash- friendliness. The most prominent criticism of FIFO is its low efficiency (high miss ratio).
In this work, we demonstrate a simple, scalable FIFO- based algorithm with three static queues (S3-FIFO). Evalu- ated on 6594 cache traces from 14 datasets, we show that S3- FIFO has lower miss ratios than state-of-the-art algorithms across traces. Moreover, S3-FIFO’s efficiency is robust — it has the lowest mean miss ratio on 10 of the 14 datasets. FIFO queues enable S3-FIFO to achieve good scalability with 6× higher throughput compared to optimized LRU at 16 threads.
Our insight is that most objects in skewed workloads will only be accessed once in a short window, so it is critical to evict them early (also called quick demotion). The key of S3-FIFO is a small FIFO queue that filters out most objects from entering the main cache, which provides a guaranteed demotion speed and high demotion precision.


```rust
use s3_fifo::{S3FIFO, S3FIFOKey};

// The cached value must be Clone.
// Hash is optional and allows using the S3FIFOKey struct
#[derive(Clone, Hash)]
struct Foobar { a: i32 }

// Create a cache with a capacity of 128 (small: 12, main: 115, ghost: 115)
let mut cache = S3FIFO::new(128);
let value = Foobar { a: 1 };
let key = S3FIFOKey::new(&value);

// Check if the item is in the cache before inserting
if let None = cache.get(&key) {
    cache.put(key.clone(), value);
    assert!(cache.get(&key).is_some());
}

// If your value is not Hash, then supply a key yourself that implements PartialEq
// let mut hash_cache: S3FIFO<S3FIFOKey<Foobar>, Foobar> = S3Fifo::new(1234);
#[derive(Clone)]
struct Abc {}
let mut custom_cache: S3FIFO<u32, Abc> = S3Fifo::new(1234)
cache.put(42, Abc);
cache.get(&42);
```