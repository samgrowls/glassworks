//! JSON output formatter for scan results.
//!
//! This module provides JSON formatting compatible with the Python harness.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::error::{OrchestratorError, Result};
use crate::scanner::PackageScanResult;

/// JSON output structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonOutput {
    /// Scan timestamp.
    pub timestamp: DateTime<Utc>,
    /// Total packages scanned.
    pub total_packages: usize,
    /// Total findings.
    pub total_findings: usize,
    /// Packages flagged as malicious.
    pub malicious_packages: usize,
    /// Average threat score.
    pub average_threat_score: f32,
    /// Individual package results.
    pub packages: Vec<JsonPackageResult>,
    /// Scan summary.
    pub summary: JsonSummary,
}

/// JSON package result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonPackageResult {
    /// Package name.
    pub package_name: String,
    /// Source type (npm, github, file).
    pub source_type: String,
    /// Version or commit hash.
    pub version: String,
    /// Path to scanned content.
    pub path: String,
    /// Content hash.
    pub content_hash: String,
    /// Threat score (0.0-10.0).
    pub threat_score: f32,
    /// Whether the package is malicious.
    pub is_malicious: bool,
    /// Scan findings.
    pub findings: Vec<JsonFinding>,
    /// Scan timestamp.
    pub timestamp: DateTime<Utc>,
}

/// JSON finding structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonFinding {
    /// File path.
    pub file: String,
    /// Line number.
    pub line: usize,
    /// Column number.
    pub column: usize,
    /// Code point (for Unicode issues).
    pub code_point: u32,
    /// Character representation.
    pub character: String,
    /// Severity level.
    pub severity: String,
    /// Detection category.
    pub category: String,
    /// Description of the issue.
    pub description: String,
    /// Remediation suggestion.
    pub remediation: String,
    /// CWE ID if applicable.
    pub cwe_id: Option<String>,
    /// References.
    pub references: Vec<String>,
    /// Code context.
    pub context: Option<String>,
    /// Decoded payload if applicable.
    pub decoded_payload: Option<String>,
    /// Confidence level (0.0-1.0).
    pub confidence: Option<f32>,
}

/// JSON summary structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSummary {
    /// Findings by severity.
    pub findings_by_severity: std::collections::HashMap<String, usize>,
    /// Findings by category.
    pub findings_by_category: std::collections::HashMap<String, usize>,
    /// Scan duration in seconds.
    pub scan_duration_secs: f64,
}

/// JSON output formatter.
pub struct JsonFormatter {
    /// Enable pretty printing.
    pub pretty: bool,
}

impl JsonFormatter {
    /// Create a new JSON formatter.
    pub fn new() -> Self {
        Self { pretty: true }
    }

    /// Create a new JSON formatter with pretty printing disabled.
    pub fn new_compact() -> Self {
        Self { pretty: false }
    }

    /// Enable or disable pretty printing.
    pub fn with_pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonFormatter {
    /// Format scan results to JSON.
    pub fn format(&self, results: &[PackageScanResult]) -> Result<String> {
        let output = self.create_output(results);

        if self.pretty {
            serde_json::to_string_pretty(&output).map_err(|e| {
                OrchestratorError::cache_error(format!("Failed to serialize JSON: {}", e))
            })
        } else {
            serde_json::to_string(&output).map_err(|e| {
                OrchestratorError::cache_error(format!("Failed to serialize JSON: {}", e))
            })
        }
    }

    /// Format scan results to JSON and write to file.
    pub fn format_to_file(&self, results: &[PackageScanResult], path: &std::path::Path) -> Result<()> {
        let json = self.format(results)?;
        std::fs::write(path, json).map_err(|e| {
            OrchestratorError::io(e)
        })?;
        Ok(())
    }

    /// Create JSON output from scan results.
    fn create_output(&self, results: &[PackageScanResult]) -> JsonOutput {
        let packages: Vec<JsonPackageResult> = results
            .iter()
            .map(|r| JsonPackageResult {
                package_name: r.package_name.clone(),
                source_type: r.source_type.clone(),
                version: r.version.clone(),
                path: r.path.clone(),
                content_hash: r.content_hash.clone(),
                threat_score: r.threat_score,
                is_malicious: r.is_malicious,
                findings: r.findings.iter().map(|f| JsonFinding::from(f)).collect(),
                timestamp: Utc::now(),
            })
            .collect();

        let total_findings: usize = packages.iter().map(|p| p.findings.len()).sum();
        let malicious_packages = packages.iter().filter(|p| p.is_malicious).count();
        
        let average_threat_score = if results.is_empty() {
            0.0
        } else {
            results.iter().map(|r| r.threat_score).sum::<f32>() / results.len() as f32
        };

        let mut findings_by_severity = std::collections::HashMap::new();
        let mut findings_by_category = std::collections::HashMap::new();

        for package in &packages {
            for finding in &package.findings {
                *findings_by_severity.entry(finding.severity.clone()).or_insert(0) += 1;
                *findings_by_category.entry(finding.category.clone()).or_insert(0) += 1;
            }
        }

        let summary = JsonSummary {
            findings_by_severity,
            findings_by_category,
            scan_duration_secs: 0.0, // Would need to track this externally
        };

        JsonOutput {
            timestamp: Utc::now(),
            total_packages: results.len(),
            total_findings,
            malicious_packages,
            average_threat_score,
            packages,
            summary,
        }
    }
}

impl From<&glassware_core::Finding> for JsonFinding {
    fn from(finding: &glassware_core::Finding) -> Self {
        Self {
            file: finding.file.clone(),
            line: finding.line,
            column: finding.column,
            code_point: finding.code_point,
            character: finding.character.clone(),
            severity: format!("{:?}", finding.severity),
            category: format!("{:?}", finding.category),
            description: finding.description.clone(),
            remediation: finding.remediation.clone(),
            cwe_id: finding.cwe_id.clone(),
            references: finding.references.clone(),
            context: finding.context.clone(),
            decoded_payload: finding.decoded_payload.as_ref().map(|p| {
                if let Some(ref text) = p.decoded_text {
                    text.clone()
                } else {
                    format!("[Binary payload, entropy: {:.2}]", p.entropy)
                }
            }),
            confidence: finding.confidence.map(|c| c as f32),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glassware_core::{Finding, Severity, DetectionCategory};

    fn create_test_finding() -> Finding {
        Finding {
            file: "test.js".to_string(),
            line: 1,
            column: 5,
            code_point: 0xFE00,
            character: "\u{FE00}".to_string(),
            raw_bytes: None,
            severity: Severity::High,
            category: DetectionCategory::InvisibleCharacter,
            description: "Test finding".to_string(),
            remediation: "Remove it".to_string(),
            cwe_id: None,
            references: vec![],
            context: None,
            decoded_payload: None,
            confidence: None,
        }
    }

    fn create_test_package_result() -> PackageScanResult {
        PackageScanResult {
            package_name: "test-pkg".to_string(),
            source_type: "npm".to_string(),
            version: "1.0.0".to_string(),
            path: "/path/to/pkg".to_string(),
            content_hash: "abc123".to_string(),
            findings: vec![create_test_finding()],
            threat_score: 5.0,
            is_malicious: true,
            llm_verdict: None,
        }
    }

    #[test]
    fn test_json_formatter_new() {
        let formatter = JsonFormatter::new();
        assert!(formatter.pretty);
    }

    #[test]
    fn test_json_formatter_compact() {
        let formatter = JsonFormatter::new_compact();
        assert!(!formatter.pretty);
    }

    #[test]
    fn test_json_formatter_format() {
        let formatter = JsonFormatter::new();
        let results = vec![create_test_package_result()];

        let json = formatter.format(&results).unwrap();
        assert!(json.contains("test-pkg"));
        assert!(json.contains("Test finding"));
    }

    #[test]
    fn test_json_formatter_format_compact() {
        let formatter = JsonFormatter::new_compact();
        let results = vec![create_test_package_result()];

        let json = formatter.format(&results).unwrap();
        assert!(json.contains("test-pkg"));
        // Compact format should not have newlines
        assert!(!json.contains('\n'));
    }

    #[test]
    fn test_json_finding_from_finding() {
        let finding = create_test_finding();
        let json_finding = JsonFinding::from(&finding);

        assert_eq!(json_finding.file, "test.js");
        assert_eq!(json_finding.line, 1);
        assert_eq!(json_finding.severity, "High");
    }

    #[test]
    fn test_json_output_structure() {
        let formatter = JsonFormatter::new();
        let results = vec![create_test_package_result()];
        let output = formatter.create_output(&results);

        assert_eq!(output.total_packages, 1);
        assert_eq!(output.total_findings, 1);
        assert_eq!(output.malicious_packages, 1);
        assert!(!output.packages.is_empty());
    }

    #[test]
    fn test_json_empty_results() {
        let formatter = JsonFormatter::new();
        let results: Vec<PackageScanResult> = vec![];
        let json = formatter.format(&results).unwrap();

        assert!(json.contains("\"total_packages\": 0"));
        assert!(json.contains("\"total_findings\": 0"));
    }
}
