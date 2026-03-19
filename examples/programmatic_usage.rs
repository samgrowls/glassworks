//! Example: Programmatic usage of glassware-core with ScanConfig
//!
//! This example demonstrates how to embed the GlassWare scanner in another application
//! using the new ScanConfig for configuration.

use glassware_core::{ScanConfig, ScanEngine, Severity};
use std::path::Path;

fn main() {
    // Create a custom scan configuration
    let config = ScanConfig::default()
        .with_extensions(vec!["js".to_string(), "ts".to_string()])
        .with_exclude_patterns(vec!["node_modules".to_string(), "dist".to_string()])
        .with_max_file_size(5 * 1024 * 1024) // 5MB
        .with_parallel(true)
        .with_parallel_workers(4)
        .with_deduplication(true)
        .with_min_severity(Severity::Low);

    #[cfg(feature = "llm")]
    let config = config.with_llm(false);

    // Create engine with config and default detectors
    let engine = ScanEngine::default_detectors_with_config(config);

    // Scan some content
    let test_content = r#"
        const secretKey = 'value';
        const password = 'test';
    "#;

    let findings = engine.scan(Path::new("example.js"), test_content);

    println!("Scan complete!");
    println!("Found {} issues", findings.len());

    for finding in &findings {
        println!(
            "  [{}] {}:{}:{} - {}",
            finding.severity,
            finding.file,
            finding.line,
            finding.column,
            finding.description
        );
    }
}
