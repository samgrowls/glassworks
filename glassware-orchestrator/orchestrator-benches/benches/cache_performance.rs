//! Cache performance benchmarks for the orchestrator.
//!
//! Measures cache efficiency:
//! - Cache hit/miss rates
//! - Cache lookup speed
//! - Cache write speed
//! - Cache cleanup performance

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use orchestrator_core::{Cacher, CacheEntry};
use tempfile::TempDir;
use chrono::Utc;

/// Create a test cacher.
async fn create_test_cacher() -> (Cacher, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test_cache.db");
    let cacher = Cacher::with_path_and_ttl(&db_path, 7).await.unwrap();
    (cacher, temp_dir)
}

/// Generate a test cache entry.
fn create_test_entry(key: &str, source_type: &str) -> CacheEntry {
    let now = Utc::now();
    let expires_at = now + chrono::Duration::days(7);

    CacheEntry {
        key: key.to_string(),
        source_type: source_type.to_string(),
        result: r#"{"findings": [], "threat_score": 0.0}"#.to_string(),
        created_at: now,
        expires_at,
        content_hash: Some(format!("hash-{}", key)),
    }
}

/// Benchmark cache write performance.
fn bench_cache_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_write");

    let rt = tokio::runtime::Runtime::new().unwrap();

    group.bench_function("write_single_entry", |b| {
        b.to_async(&rt).iter(|| async {
            let (cacher, _temp_dir) = create_test_cacher().await;
            let entry = create_test_entry("test-key", "npm");
            let result = cacher.set(entry).await;
            assert!(result.is_ok());
        });
    });

    group.bench_function("write_100_entries", |b| {
        b.to_async(&rt).iter(|| async {
            let (cacher, _temp_dir) = create_test_cacher().await;

            for i in 0..100 {
                let entry = create_test_entry(&format!("key-{}", i), "npm");
                let result = cacher.set(entry).await;
                assert!(result.is_ok());
            }
        });
    });

    group.bench_function("write_1000_entries", |b| {
        b.to_async(&rt).iter(|| async {
            let (cacher, _temp_dir) = create_test_cacher().await;

            for i in 0..1000 {
                let entry = create_test_entry(&format!("key-{}", i), "npm");
                let result = cacher.set(entry).await;
                assert!(result.is_ok());
            }
        });
    });

    group.finish();
}

/// Benchmark cache read performance.
fn bench_cache_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_read");

    let rt = tokio::runtime::Runtime::new().unwrap();

    // Setup: create cacher with entries
    let setup_cacher = || async {
        let (cacher, temp_dir) = create_test_cacher().await;

        // Pre-populate with 100 entries
        for i in 0..100 {
            let entry = create_test_entry(&format!("key-{}", i), "npm");
            cacher.set(entry).await.unwrap();
        }

        (cacher, temp_dir)
    };

    group.bench_function("read_single_hit", |b| {
        b.to_async(&rt).iter(|| async {
            let (cacher, _temp_dir) = setup_cacher().await;
            let result = cacher.get("key-50").await;
            assert!(result.is_ok());
            assert!(result.unwrap().is_some());
        });
    });

    group.bench_function("read_single_miss", |b| {
        b.to_async(&rt).iter(|| async {
            let (cacher, _temp_dir) = setup_cacher().await;
            let result = cacher.get("nonexistent-key").await;
            assert!(result.is_ok());
            assert!(result.unwrap().is_none());
        });
    });

    group.bench_function("read_100_entries", |b| {
        b.to_async(&rt).iter(|| async {
            let (cacher, _temp_dir) = setup_cacher().await;

            for i in 0..100 {
                let result = cacher.get(&format!("key-{}", i)).await;
                assert!(result.is_ok());
            }
        });
    });

    group.finish();
}

/// Benchmark cache hit/miss rate.
fn bench_cache_hit_rate(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_hit_rate");

    let rt = tokio::runtime::Runtime::new().unwrap();

    group.bench_function("hit_rate_50_percent", |b| {
        b.to_async(&rt).iter(|| async {
            let (cacher, _temp_dir) = create_test_cacher().await;

            // Pre-populate with 50 entries
            for i in 0..50 {
                let entry = create_test_entry(&format!("key-{}", i), "npm");
                cacher.set(entry).await.unwrap();
            }

            // Query with 50% hit rate
            let mut hits = 0;
            let mut misses = 0;

            for i in 0..100 {
                let key = if i % 2 == 0 {
                    format!("key-{}", i % 50) // Hit
                } else {
                    format!("miss-{}", i) // Miss
                };

                let result = cacher.get(&key).await.unwrap();
                if result.is_some() {
                    hits += 1;
                } else {
                    misses += 1;
                }
            }

            assert_eq!(hits, 50);
            assert_eq!(misses, 50);
        });
    });

    group.bench_function("hit_rate_90_percent", |b| {
        b.to_async(&rt).iter(|| async {
            let (cacher, _temp_dir) = create_test_cacher().await;

            // Pre-populate with 90 entries
            for i in 0..90 {
                let entry = create_test_entry(&format!("key-{}", i), "npm");
                cacher.set(entry).await.unwrap();
            }

            // Query with 90% hit rate
            let mut hits = 0;
            let mut misses = 0;

            for i in 0..100 {
                let key = if i < 90 {
                    format!("key-{}", i) // Hit
                } else {
                    format!("miss-{}", i) // Miss
                };

                let result = cacher.get(&key).await.unwrap();
                if result.is_some() {
                    hits += 1;
                } else {
                    misses += 1;
                }
            }

            assert_eq!(hits, 90);
            assert_eq!(misses, 10);
        });
    });

    group.finish();
}

/// Benchmark cache cleanup performance.
fn bench_cache_cleanup(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_cleanup");

    let rt = tokio::runtime::Runtime::new().unwrap();

    group.bench_function("cleanup_expired", |b| {
        b.to_async(&rt).iter(|| async {
            let (cacher, _temp_dir) = create_test_cacher().await;

            // Add entries with various TTLs
            for i in 0..100 {
                let mut entry = create_test_entry(&format!("key-{}", i), "npm");
                if i % 2 == 0 {
                    // Make half of them expired
                    entry.expires_at = Utc::now() - chrono::Duration::seconds(1);
                }
                cacher.set(entry).await.unwrap();
            }

            // Run cleanup
            let removed = cacher.cleanup_expired().await.unwrap();
            assert_eq!(removed, 50);
        });
    });

    group.bench_function("cleanup_stats", |b| {
        b.to_async(&rt).iter(|| async {
            let (cacher, _temp_dir) = create_test_cacher().await;

            // Add entries
            for i in 0..100 {
                let entry = create_test_entry(&format!("key-{}", i), "npm");
                cacher.set(entry).await.unwrap();
            }

            // Get stats
            let stats = cacher.stats().await.unwrap();
            assert_eq!(stats.total_entries, 100);
        });
    });

    group.finish();
}

/// Benchmark cache with different source types.
fn bench_cache_source_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_source_types");

    let rt = tokio::runtime::Runtime::new().unwrap();

    group.bench_function("npm_entries", |b| {
        b.to_async(&rt).iter(|| async {
            let (cacher, _temp_dir) = create_test_cacher().await;

            for i in 0..100 {
                let entry = create_test_entry(&format!("npm-pkg-{}", i), "npm");
                cacher.set(entry).await.unwrap();
            }

            let stats = cacher.stats().await.unwrap();
            assert_eq!(stats.npm_entries, 100);
        });
    });

    group.bench_function("github_entries", |b| {
        b.to_async(&rt).iter(|| async {
            let (cacher, _temp_dir) = create_test_cacher().await;

            for i in 0..100 {
                let entry = create_test_entry(&format!("github-repo-{}", i), "github");
                cacher.set(entry).await.unwrap();
            }

            let stats = cacher.stats().await.unwrap();
            assert_eq!(stats.github_entries, 100);
        });
    });

    group.bench_function("mixed_entries", |b| {
        b.to_async(&rt).iter(|| async {
            let (cacher, _temp_dir) = create_test_cacher().await;

            for i in 0..100 {
                let source_type = if i % 2 == 0 { "npm" } else { "github" };
                let entry = create_test_entry(&format!("{}-pkg-{}", source_type, i), source_type);
                cacher.set(entry).await.unwrap();
            }

            let stats = cacher.stats().await.unwrap();
            assert_eq!(stats.npm_entries, 50);
            assert_eq!(stats.github_entries, 50);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_cache_write,
    bench_cache_read,
    bench_cache_hit_rate,
    bench_cache_cleanup,
    bench_cache_source_types,
);

criterion_main!(benches);
