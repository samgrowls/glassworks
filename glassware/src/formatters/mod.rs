//! Output formatters for scan results.
//!
//! This module provides different output formats:
//! - JSON: Compatible with Python harness
//! - SARIF: GitHub Advanced Security compatible

pub mod json;
pub mod sarif;

pub use json::{JsonFormatter, JsonOutput, JsonPackageResult, JsonFinding, JsonSummary};
pub use sarif::{SarifFormatter, SarifLog, SarifRun, SarifResult};

/// Output formatter trait.
pub trait OutputFormatter: Send + Sync {
    /// Format scan results to a string.
    fn format(&self, results: &[crate::scanner::PackageScanResult]) -> crate::error::Result<String>;
}

impl OutputFormatter for JsonFormatter {
    fn format(&self, results: &[crate::scanner::PackageScanResult]) -> crate::error::Result<String> {
        self.format(results)
    }
}

impl OutputFormatter for SarifFormatter {
    fn format(&self, results: &[crate::scanner::PackageScanResult]) -> crate::error::Result<String> {
        self.format(results)
    }
}
