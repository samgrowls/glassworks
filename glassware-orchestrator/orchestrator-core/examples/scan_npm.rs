//! Example: Scan npm packages
//!
//! This example demonstrates how to scan npm packages using the orchestrator.
//!
//! Run with: `cargo run --example scan_npm`

use orchestrator_core::{Orchestrator, OrchestratorConfig, ScannerConfig};
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    orchestrator_core::tracing::init_tracing(
        &orchestrator_core::tracing::TracingConfig::default()
    )?;

    info!("Starting npm package scan example");

    // Create orchestrator with custom config
    let config = OrchestratorConfig {
        scanner: ScannerConfig {
            max_concurrent: 5,
            threat_threshold: 5.0,
            ..Default::default()
        },
        cache_ttl_days: 7,
        enable_cache: true,
        enable_checkpoint: true,
        ..Default::default()
    };

    let orchestrator = Orchestrator::with_config(config).await?;

    // Packages to scan
    let packages = vec![
        "express".to_string(),
        "lodash".to_string(),
        "axios".to_string(),
    ];

    info!("Scanning {} packages", packages.len());

    // Scan packages
    let results = orchestrator.scan_npm_packages(&packages).await;

    // Process results
    let mut malicious_count = 0;
    let mut total_findings = 0;

    for result in results {
        match result {
            Ok(scan_result) => {
                info!(
                    "Package: {} v{} - Threat score: {:.2}",
                    scan_result.package_name,
                    scan_result.version,
                    scan_result.threat_score
                );

                if scan_result.is_malicious {
                    error!("🚨 Malicious package detected: {}", scan_result.package_name);
                    malicious_count += 1;
                }

                total_findings += scan_result.findings.len();

                // Print findings
                for finding in &scan_result.findings {
                    info!(
                        "  [{}] {}:{} - {}",
                        finding.severity,
                        finding.file,
                        finding.line,
                        finding.description
                    );
                }
            }
            Err(e) => {
                error!("Failed to scan package: {}", e);
            }
        }
    }

    // Print summary
    info!("");
    info!("=" .repeat(60));
    info!("SCAN SUMMARY");
    info!("=" .repeat(60));
    info!("Total packages: {}", packages.len());
    info!("Malicious packages: {}", malicious_count);
    info!("Total findings: {}", total_findings);

    if malicious_count > 0 {
        error!("🚨 Malicious packages detected!");
        std::process::exit(1);
    } else {
        info!("✅ No malicious packages detected");
    }

    Ok(())
}
