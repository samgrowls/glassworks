//! Example: Scan GitHub repositories
//!
//! This example demonstrates how to scan GitHub repositories using the orchestrator.
//!
//! Run with: `cargo run --example scan_github`
//!
//! Note: Set GITHUB_TOKEN environment variable for private repos or higher rate limits.

use orchestrator_core::{Orchestrator, OrchestratorConfig};
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    orchestrator_core::tracing::init_tracing(
        &orchestrator_core::tracing::TracingConfig::default()
    )?;

    info!("Starting GitHub repository scan example");

    // Get GitHub token from environment (optional)
    let github_token = std::env::var("GITHUB_TOKEN").ok();

    // Create orchestrator with GitHub token
    let config = OrchestratorConfig {
        github_token: github_token.clone(),
        github_rate_limit: if github_token.is_some() { 5.0 } else { 1.0 },
        enable_checkpoint: true,
        ..Default::default()
    };

    let orchestrator = Orchestrator::with_config(config).await?;

    // Repositories to scan
    let repos = vec![
        ("tokio-rs".to_string(), "tokio".to_string()),
        ("serde-rs".to_string(), "serde".to_string()),
    ];

    info!("Scanning {} repositories", repos.len());

    // Scan repositories
    let results = orchestrator.scan_github_repos(&repos, Some("main")).await;

    // Process results
    let mut malicious_count = 0;
    let mut total_findings = 0;

    for result in results {
        match result {
            Ok(scan_result) => {
                info!(
                    "Repository: {} - Threat score: {:.2}",
                    scan_result.package_name,
                    scan_result.threat_score
                );

                if scan_result.is_malicious {
                    error!("🚨 Malicious repository detected: {}", scan_result.package_name);
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
                error!("Failed to scan repository: {}", e);
            }
        }
    }

    // Print summary
    info!("");
    info!("SCAN SUMMARY");
    info!("Total repositories: {}", repos.len());
    info!("Malicious repositories: {}", malicious_count);
    info!("Total findings: {}", total_findings);

    if malicious_count > 0 {
        error!("🚨 Malicious repositories detected!");
        std::process::exit(1);
    } else {
        info!("✅ No malicious repositories detected");
    }

    Ok(())
}
