//! Example: Scan with LLM analysis
//!
//! This example demonstrates how to use LLM analysis for flagged files.
//!
//! Run with: `cargo run --example with_llm`
//!
//! Note: Requires LLM API credentials:
//! - GLASSWARE_LLM_BASE_URL
//! - GLASSWARE_LLM_API_KEY
//! - GLASSWARE_LLM_MODEL (optional)

use orchestrator_core::{Orchestrator, OrchestratorConfig, ScannerConfig};
#[cfg(feature = "llm")]
use orchestrator_core::llm::{LlmAnalyzer, LlmAnalyzerConfig};
use tracing::{info, error, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    orchestrator_core::tracing::init_tracing(
        &orchestrator_core::tracing::TracingConfig::default()
    )?;

    info!("Starting LLM analysis example");

    // Check for LLM credentials
    let llm_base_url = std::env::var("GLASSWARE_LLM_BASE_URL").ok();
    let llm_api_key = std::env::var("GLASSWARE_LLM_API_KEY").ok();

    if llm_base_url.is_none() || llm_api_key.is_none() {
        warn!("LLM credentials not set. Set GLASSWARE_LLM_BASE_URL and GLASSWARE_LLM_API_KEY");
        warn!("Running without LLM analysis...");
    }

    // Create orchestrator
    let config = OrchestratorConfig {
        scanner: ScannerConfig {
            max_concurrent: 5,
            enable_semantic: true,
            ..Default::default()
        },
        #[cfg(feature = "llm")]
        enable_llm: llm_base_url.is_some() && llm_api_key.is_some(),
        #[cfg(feature = "llm")]
        llm_config: if let (Some(base_url), Some(api_key)) = (llm_base_url, llm_api_key) {
            Some(LlmAnalyzerConfig {
                base_url,
                api_key,
                model: std::env::var("GLASSWARE_LLM_MODEL")
                    .unwrap_or_else(|_| "llama-3.3-70b".to_string()),
                ..Default::default()
            })
        } else {
            None
        },
        ..Default::default()
    };

    let orchestrator = Orchestrator::with_config(config).await?;

    // Packages to scan
    let packages = vec![
        "express".to_string(),
        "lodash".to_string(),
    ];

    info!("Scanning {} packages", packages.len());

    // Scan packages
    let results = orchestrator.scan_npm_packages(&packages).await;

    // Process results
    for result in results {
        match result {
            Ok(scan_result) => {
                info!(
                    "Package: {} v{} - Threat score: {:.2}",
                    scan_result.package_name,
                    scan_result.version,
                    scan_result.threat_score
                );

                if !scan_result.findings.is_empty() {
                    info!("  Found {} security issues", scan_result.findings.len());

                    #[cfg(feature = "llm")]
                    if let Some(ref analyzer) = orchestrator.llm_analyzer {
                        info!("  Running LLM analysis on findings...");

                        for finding in &scan_result.findings {
                            match analyzer.analyze_finding(finding).await {
                                Ok(verdict) => {
                                    info!(
                                        "    LLM verdict: {} (confidence: {:.1}%)",
                                        verdict.intent,
                                        verdict.confidence * 100.0
                                    );

                                    if !verdict.reasoning.is_empty() {
                                        info!("    Reasoning: {}", verdict.reasoning);
                                    }
                                }
                                Err(e) => {
                                    warn!("    LLM analysis failed: {}", e);
                                }
                            }
                        }
                    }

                    // Print findings
                    for finding in &scan_result.findings {
                        info!(
                            "    [{}] {}:{} - {}",
                            finding.severity,
                            finding.file,
                            finding.line,
                            finding.description
                        );
                    }
                }
            }
            Err(e) => {
                error!("Failed to scan package: {}", e);
            }
        }
    }

    info!("✅ Scan complete");

    Ok(())
}
