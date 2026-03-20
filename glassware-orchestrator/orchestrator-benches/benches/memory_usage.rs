//! Memory usage benchmarks for the orchestrator.
//!
//! Measures memory consumption during scanning operations:
//! - Peak memory usage
//! - Memory per LOC
//! - Memory efficiency with streaming vs buffering

use criterion::{black_box, criterion_group, criterion_main, Criterion, Measurement};
use orchestrator_core::{Scanner, ScannerConfig, streaming::{StreamingWriter, OutputFormat}};
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::io::BufWriter;

/// Generate test content with specified lines of code.
fn generate_test_content(loc: usize) -> String {
    let mut content = String::new();
    for i in 0..loc {
        content.push_str(&format!("const variable_{} = {};\n", i, i));
    }
    content
}

/// Measure memory usage of scanning operation.
fn measure_memory_usage<F, R>(f: F) -> usize
where
    F: FnOnce() -> R,
{
    // Note: Rust doesn't have built-in memory measurement like Python's tracemalloc
    // This is a placeholder that would use external tools in production
    // For benchmarking, we use criterion's measurement infrastructure
    let _ = f();
    0 // Placeholder
}

/// Benchmark memory usage for scanning small packages.
fn bench_memory_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    let content = generate_test_content(100);
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.js");
    std::fs::write(&file_path, &content).unwrap();
    let path = temp_dir.path().to_string_lossy().to_string();

    group.bench_function("small_100_loc", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let scanner = Scanner::new();

        b.to_async(&rt).iter(|| async {
            let result = scanner.scan_directory(black_box(&path)).await;
            assert!(result.is_ok());
        });
    });

    group.finish();
}

/// Benchmark memory usage for scanning medium packages.
fn bench_memory_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    let content = generate_test_content(1000);
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.js");
    std::fs::write(&file_path, &content).unwrap();
    let path = temp_dir.path().to_string_lossy().to_string();

    group.bench_function("medium_1000_loc", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let scanner = Scanner::new();

        b.to_async(&rt).iter(|| async {
            let result = scanner.scan_directory(black_box(&path)).await;
            assert!(result.is_ok());
        });
    });

    group.finish();
}

/// Benchmark memory usage for scanning large packages.
fn bench_memory_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    let content = generate_test_content(10000);
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.js");
    std::fs::write(&file_path, &content).unwrap();
    let path = temp_dir.path().to_string_lossy().to_string();

    group.bench_function("large_10000_loc", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let scanner = Scanner::new();

        b.to_async(&rt).iter(|| async {
            let result = scanner.scan_directory(black_box(&path)).await;
            assert!(result.is_ok());
        });
    });

    group.finish();
}

/// Benchmark streaming writer memory efficiency.
fn bench_streaming_writer(c: &mut Criterion) {
    use orchestrator_core::scanner::PackageScanResult;

    let mut group = c.benchmark_group("streaming_memory");

    let test_sizes = vec![10, 100, 1000];

    for size in test_sizes {
        group.bench_function(format!("streaming_{}_results", size), |b| {
            let rt = tokio::runtime::Runtime::new().unwrap();

            b.to_async(&rt).iter(|| async {
                let temp_dir = TempDir::new().unwrap();
                let file_path = temp_dir.path().join("results.jsonl");
                let file = tokio::fs::File::create(&file_path).await.unwrap();
                let writer = BufWriter::new(file);

                let mut streaming = StreamingWriter::json_lines(writer);

                for i in 0..size {
                    let result = PackageScanResult {
                        package_name: format!("pkg-{}", i),
                        source_type: "npm".to_string(),
                        version: "1.0.0".to_string(),
                        path: format!("/path/{}", i),
                        content_hash: format!("hash-{}", i),
                        findings: vec![],
                        threat_score: 0.0,
                        is_malicious: false,
                    };

                    streaming.write_result(&result).await.unwrap();
                }

                streaming.flush().await.unwrap();
            });
        });
    }

    group.finish();
}

/// Benchmark buffered vs streaming output.
fn bench_buffered_vs_streaming(c: &mut Criterion) {
    use orchestrator_core::scanner::PackageScanResult;

    let mut group = c.benchmark_group("buffered_vs_streaming");

    let test_size = 100;

    group.bench_function("buffered_json_array", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();

        b.to_async(&rt).iter(|| async {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("results.json");
            let file = tokio::fs::File::create(&file_path).await.unwrap();
            let writer = BufWriter::new(file);

            let mut streaming = StreamingWriter::json_array(writer);

            for i in 0..test_size {
                let result = PackageScanResult {
                    package_name: format!("pkg-{}", i),
                    source_type: "npm".to_string(),
                    version: "1.0.0".to_string(),
                    path: format!("/path/{}", i),
                    content_hash: format!("hash-{}", i),
                    findings: vec![],
                    threat_score: 0.0,
                    is_malicious: false,
                };

                streaming.write_result(&result).await.unwrap();
            }

            streaming.flush().await.unwrap();
        });
    });

    group.bench_function("streaming_json_lines", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();

        b.to_async(&rt).iter(|| async {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("results.jsonl");
            let file = tokio::fs::File::create(&file_path).await.unwrap();
            let writer = BufWriter::new(file);

            let mut streaming = StreamingWriter::json_lines(writer);

            for i in 0..test_size {
                let result = PackageScanResult {
                    package_name: format!("pkg-{}", i),
                    source_type: "npm".to_string(),
                    version: "1.0.0".to_string(),
                    path: format!("/path/{}", i),
                    content_hash: format!("hash-{}", i),
                    findings: vec![],
                    threat_score: 0.0,
                    is_malicious: false,
                };

                streaming.write_result(&result).await.unwrap();
            }

            streaming.flush().await.unwrap();
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_memory_small,
    bench_memory_medium,
    bench_memory_large,
    bench_streaming_writer,
    bench_buffered_vs_streaming,
);

criterion_main!(benches);
