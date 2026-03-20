//! Example: Adversarial testing
//!
//! This example demonstrates how to use the adversarial testing framework.
//!
//! Run with: `cargo run --example adversarial_test`

use orchestrator_core::adversarial::{AdversarialTester, MutationEngineConfig, FuzzerEngineConfig};
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    orchestrator_core::tracing::init_tracing(
        &orchestrator_core::tracing::TracingConfig::debug()
    )?;

    info!("Starting adversarial testing example");

    // Create adversarial tester with custom config
    let mutation_config = MutationEngineConfig {
        mutations_per_test: 10,
        mutation_probability: 0.3,
        enable_invisible: true,
        enable_homoglyph: true,
        enable_bidi: true,
        enable_stego: true,
        enable_noise: false,
    };

    let fuzz_config = FuzzerEngineConfig {
        fuzz_cases: 50,
        enable_unicode: true,
        enable_length: true,
        enable_structure: false,
        enable_syntax: false,
    };

    let tester = AdversarialTester::with_configs(mutation_config, fuzz_config)?;

    // Create a test file with some code
    let temp_dir = tempfile::tempdir()?;
    let test_file = temp_dir.path().join("test.js");
    std::fs::write(
        &test_file,
        r#"
const config = {
    apiKey: "secret-key-123",
    endpoint: "https://api.example.com"
};

function processData(data) {
    return JSON.parse(data);
}

module.exports = { config, processData };
"#,
    )?;

    info!("Testing file: {}", test_file.display());

    // Run adversarial tests
    let report = tester.test_package(test_file.to_str().unwrap()).await?;

    // Print report
    info!("");
    info!("ADVERSARIAL TESTING REPORT");
    info!("=".repeat(60));
    info!("Package: {}", report.package_name);
    info!("Risk level: {:?}", report.risk_level());
    info!("");
    info!("Mutation Tests:");
    info!("  Total mutations: {}", report.total_mutations);
    info!("  Evaded mutations: {}", report.evaded_mutations);
    info!("  Evasion rate: {:.2}%", report.evasion_rate * 100.0);
    info!("");
    info!("Fuzz Tests:");
    info!("  Total fuzz cases: {}", report.total_fuzz_cases);
    info!("  Erroring cases: {}", report.erroring_fuzz_cases);
    info!("");

    if !report.high_risk_evasions.is_empty() {
        error!("High-risk evasions detected:");
        for evasion in &report.high_risk_evasions {
            error!("  - {}", evasion);
        }
    }

    if !report.recommendations.is_empty() {
        info!("Recommendations:");
        for rec in &report.recommendations {
            info!("  - {}", rec);
        }
    }

    // Exit with error if high risk
    if report.is_high_risk() {
        error!("🚨 Package flagged as high-risk!");
        std::process::exit(1);
    } else {
        info!("✅ Package passed adversarial testing");
    }

    Ok(())
}
