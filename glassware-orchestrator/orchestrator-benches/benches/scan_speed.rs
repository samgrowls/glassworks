//! Scan speed benchmarks for the orchestrator.
//!
//! Measures the scanning speed of the orchestrator across different scenarios:
//! - Small packages (< 100 LOC)
//! - Medium packages (100-1000 LOC)
//! - Large packages (> 1000 LOC)
//! - Comparison with Python harness

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use orchestrator_core::{Scanner, ScannerConfig};
use std::path::PathBuf;
use tempfile::TempDir;

/// Generate test content with specified lines of code.
fn generate_test_content(loc: usize) -> String {
    let mut content = String::new();
    for i in 0..loc {
        content.push_str(&format!("const variable_{} = {};\n", i, i));
    }
    content
}

/// Generate test content with invisible characters.
fn generate_content_with_invisible(loc: usize) -> String {
    let mut content = String::new();
    for i in 0..loc {
        // Insert invisible character every 10 lines
        if i % 10 == 0 {
            content.push('\u{200B}'); // Zero-width space
        }
        content.push_str(&format!("const variable_{} = {};\n", i, i));
    }
    content
}

/// Generate test content with homoglyphs.
fn generate_content_with_homoglyphs(loc: usize) -> String {
    let mut content = String::new();
    for i in 0..loc {
        // Replace some 'a' with Cyrillic 'а' every 20 lines
        if i % 20 == 0 {
            content.push_str(&format!("const vаriable_{} = {};\n", i, i)); // Cyrillic 'а'
        } else {
            content.push_str(&format!("const variable_{} = {};\n", i, i));
        }
    }
    content
}

/// Create a temporary directory with test files.
fn create_test_directory(loc: usize, content: &str) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.js");
    std::fs::write(&file_path, content).unwrap();
    temp_dir
}

/// Benchmark scanning speed for clean content.
fn bench_scan_clean(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan_speed");

    let test_cases = vec![
        ("small", 100),
        ("medium", 1000),
        ("large", 10000),
    ];

    for (name, loc) in test_cases {
        let content = generate_test_content(loc);
        let temp_dir = create_test_directory(loc, &content);
        let path = temp_dir.path().to_string_lossy().to_string();

        group.throughput(Throughput::Elements(loc as u64));

        group.bench_function(format!("clean_{}_loc", name), |b| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let scanner = Scanner::new();

            b.to_async(&rt).iter(|| async {
                let result = scanner.scan_directory(black_box(&path)).await;
                assert!(result.is_ok());
            });
        });
    }

    group.finish();
}

/// Benchmark scanning speed for content with invisible characters.
fn bench_scan_invisible(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan_speed_invisible");

    let test_cases = vec![
        ("small", 100),
        ("medium", 1000),
        ("large", 10000),
    ];

    for (name, loc) in test_cases {
        let content = generate_content_with_invisible(loc);
        let temp_dir = create_test_directory(loc, &content);
        let path = temp_dir.path().to_string_lossy().to_string();

        group.throughput(Throughput::Elements(loc as u64));

        group.bench_function(format!("invisible_{}_loc", name), |b| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let scanner = Scanner::new();

            b.to_async(&rt).iter(|| async {
                let result = scanner.scan_directory(black_box(&path)).await;
                assert!(result.is_ok());
            });
        });
    }

    group.finish();
}

/// Benchmark scanning speed for content with homoglyphs.
fn bench_scan_homoglyphs(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan_speed_homoglyphs");

    let test_cases = vec![
        ("small", 100),
        ("medium", 1000),
        ("large", 10000),
    ];

    for (name, loc) in test_cases {
        let content = generate_content_with_homoglyphs(loc);
        let temp_dir = create_test_directory(loc, &content);
        let path = temp_dir.path().to_string_lossy().to_string();

        group.throughput(Throughput::Elements(loc as u64));

        group.bench_function(format!("homoglyphs_{}_loc", name), |b| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let scanner = Scanner::new();

            b.to_async(&rt).iter(|| async {
                let result = scanner.scan_directory(black_box(&path)).await;
                assert!(result.is_ok());
            });
        });
    }

    group.finish();
}

/// Benchmark scanning speed with different concurrency levels.
fn bench_scan_concurrency(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan_speed_concurrency");

    let content = generate_test_content(1000);
    let temp_dir = create_test_directory(1000, &content);
    let path = temp_dir.path().to_string_lossy().to_string();

    let concurrency_levels = vec![1, 5, 10, 20];

    for concurrency in concurrency_levels {
        group.bench_function(format!("concurrency_{}", concurrency), |b| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let config = ScannerConfig {
                max_concurrent: concurrency,
                ..Default::default()
            };
            let scanner = Scanner::with_config(config);

            b.to_async(&rt).iter(|| async {
                let result = scanner.scan_directory(black_box(&path)).await;
                assert!(result.is_ok());
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_scan_clean,
    bench_scan_invisible,
    bench_scan_homoglyphs,
    bench_scan_concurrency,
);

criterion_main!(benches);
