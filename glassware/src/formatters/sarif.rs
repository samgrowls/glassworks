//! SARIF 2.1.0 output formatter for GitHub Advanced Security compatibility.
//!
//! This module provides SARIF (Static Analysis Results Interchange Format) output
//! compatible with GitHub Advanced Security and other SARIF consumers.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::error::{OrchestratorError, Result};
use crate::scanner::PackageScanResult;

/// SARIF log structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifLog {
    /// SARIF schema version.
    #[serde(rename = "$schema")]
    pub schema: String,
    /// SARIF version.
    pub version: String,
    /// Runs in the log.
    pub runs: Vec<SarifRun>,
}

/// SARIF run structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifRun {
    /// Tool information.
    pub tool: SarifTool,
    /// Results of the analysis.
    pub results: Vec<SarifResult>,
    /// Invocations that produced the run.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub invocations: Vec<SarifInvocation>,
    /// Artifacts analyzed in the run.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<SarifArtifact>,
}

/// SARIF tool structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifTool {
    /// Tool driver information.
    pub driver: SarifToolComponent,
}

/// SARIF tool component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifToolComponent {
    /// Tool name.
    pub name: String,
    /// Tool version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Information URI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub information_uri: Option<String>,
    /// Rules supported by the tool.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<SarifReportingDescriptor>,
}

/// SARIF reporting descriptor (rule definition).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifReportingDescriptor {
    /// Rule ID.
    pub id: String,
    /// Rule name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Short description.
    pub short_description: SarifMultiformatMessageString,
    /// Full description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_description: Option<SarifMultiformatMessageString>,
    /// Help URI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help_uri: Option<String>,
    /// Default configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_configuration: Option<SarifReportingConfiguration>,
}

/// SARIF reporting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifReportingConfiguration {
    /// Default level.
    pub level: String,
}

/// SARIF multiformat message string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifMultiformatMessageString {
    /// Text content.
    pub text: String,
    /// Markdown content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown: Option<String>,
}

/// SARIF result structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifResult {
    /// Rule ID.
    pub rule_id: String,
    /// Result level (error, warning, note, none).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    /// Result message.
    pub message: SarifMessage,
    /// Location of the result.
    pub locations: Vec<SarifLocation>,
    /// Partial fingerprints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_fingerprints: Option<SarifPartialFingerprints>,
}

/// SARIF message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifMessage {
    /// Message text.
    pub text: String,
}

/// SARIF location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifLocation {
    /// Physical location.
    pub physical_location: SarifPhysicalLocation,
}

/// SARIF physical location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifPhysicalLocation {
    /// Artifact location.
    pub artifact_location: SarifArtifactLocation,
    /// Region within the artifact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<SarifRegion>,
}

/// SARIF artifact location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifArtifactLocation {
    /// URI of the artifact.
    pub uri: String,
}

/// SARIF region.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifRegion {
    /// Start line (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_line: Option<usize>,
    /// Start column (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_column: Option<usize>,
    /// Snippet of the source code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<SarifArtifactContent>,
}

/// SARIF artifact content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifArtifactContent {
    /// Text content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// SARIF partial fingerprints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifPartialFingerprints {
    /// Primary location hash.
    #[serde(rename = "primaryLocationLineHash", skip_serializing_if = "Option::is_none")]
    pub primary_location_line_hash: Option<String>,
}

/// SARIF invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifInvocation {
    /// Execution successful.
    pub execution_successful: bool,
    /// Start time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time_utc: Option<DateTime<Utc>>,
    /// End time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time_utc: Option<DateTime<Utc>>,
}

/// SARIF artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifArtifact {
    /// Artifact location.
    pub location: SarifArtifactLocation,
    /// Artifact length.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<usize>,
}

/// SARIF output formatter.
pub struct SarifFormatter {
    /// Enable pretty printing.
    pub pretty: bool,
    /// Base URI for artifact locations.
    pub base_uri: Option<String>,
    /// Tool version.
    pub tool_version: Option<String>,
}

impl SarifFormatter {
    /// Create a new SARIF formatter.
    pub fn new() -> Self {
        Self {
            pretty: true,
            base_uri: None,
            tool_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }
    }

    /// Set base URI for artifact locations.
    pub fn with_base_uri(mut self, base_uri: &str) -> Self {
        self.base_uri = Some(base_uri.to_string());
        self
    }

    /// Set tool version.
    pub fn with_tool_version(mut self, version: &str) -> Self {
        self.tool_version = Some(version.to_string());
        self
    }

    /// Enable or disable pretty printing.
    pub fn with_pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }
}

impl Default for SarifFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl SarifFormatter {
    /// Format scan results to SARIF.
    pub fn format(&self, results: &[PackageScanResult]) -> Result<String> {
        let sarif_log = self.create_sarif_log(results);

        if self.pretty {
            serde_json::to_string_pretty(&sarif_log).map_err(|e| {
                OrchestratorError::cache_error(format!("Failed to serialize SARIF: {}", e))
            })
        } else {
            serde_json::to_string(&sarif_log).map_err(|e| {
                OrchestratorError::cache_error(format!("Failed to serialize SARIF: {}", e))
            })
        }
    }

    /// Format scan results to SARIF and write to file.
    pub fn format_to_file(&self, results: &[PackageScanResult], path: &std::path::Path) -> Result<()> {
        let sarif = self.format(results)?;
        std::fs::write(path, sarif).map_err(|e| {
            OrchestratorError::io(e)
        })?;
        Ok(())
    }

    /// Create SARIF log from scan results.
    fn create_sarif_log(&self, results: &[PackageScanResult]) -> SarifLog {
        let mut sarif_results = Vec::new();
        let mut rules = Vec::new();
        let mut seen_categories = std::collections::HashSet::new();

        // Collect all results and rules
        for package_result in results {
            for finding in &package_result.findings {
                let category = format!("{:?}", finding.category);
                
                // Add rule if not seen before
                if seen_categories.insert(category.clone()) {
                    rules.push(self.create_rule(&category, &finding));
                }

                sarif_results.push(self.create_sarif_result(finding, package_result));
            }
        }

        let tool = SarifTool {
            driver: SarifToolComponent {
                name: "glassware".to_string(),
                version: self.tool_version.clone(),
                information_uri: Some("https://github.com/glassware/glassworks".to_string()),
                rules,
            },
        };

        let run = SarifRun {
            tool,
            results: sarif_results,
            invocations: vec![SarifInvocation {
                execution_successful: true,
                start_time_utc: Some(Utc::now()),
                end_time_utc: None,
            }],
            artifacts: Vec::new(),
        };

        SarifLog {
            schema: "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json".to_string(),
            version: "2.1.0".to_string(),
            runs: vec![run],
        }
    }

    /// Create SARIF rule from finding category.
    fn create_rule(&self, category: &str, finding: &glassware_core::Finding) -> SarifReportingDescriptor {
        let (level, severity_text) = match finding.severity {
            glassware_core::Severity::Critical => ("error", "Critical"),
            glassware_core::Severity::High => ("error", "High"),
            glassware_core::Severity::Medium => ("warning", "Medium"),
            glassware_core::Severity::Low => ("note", "Low"),
            glassware_core::Severity::Info => ("none", "Info"),
        };

        SarifReportingDescriptor {
            id: format!("GLASSWARE-{}", category.to_uppercase().replace('_', "-")),
            name: Some(category.replace('_', " ")),
            short_description: SarifMultiformatMessageString {
                text: format!("{}: {}", category, finding.description),
                markdown: Some(format!("**{}**: {}", category, finding.description)),
            },
            full_description: Some(SarifMultiformatMessageString {
                text: finding.description.clone(),
                markdown: None,
            }),
            help_uri: Some(finding.remediation.clone()),
            default_configuration: Some(SarifReportingConfiguration {
                level: level.to_string(),
            }),
        }
    }

    /// Create SARIF result from finding.
    fn create_sarif_result(&self, finding: &glassware_core::Finding, package: &PackageScanResult) -> SarifResult {
        let level = match finding.severity {
            glassware_core::Severity::Critical => Some("error".to_string()),
            glassware_core::Severity::High => Some("error".to_string()),
            glassware_core::Severity::Medium => Some("warning".to_string()),
            glassware_core::Severity::Low => Some("note".to_string()),
            glassware_core::Severity::Info => Some("none".to_string()),
        };

        let uri = if let Some(ref base_uri) = self.base_uri {
            format!("{}/{}", base_uri, finding.file)
        } else {
            finding.file.clone()
        };

        SarifResult {
            rule_id: format!("GLASSWARE-{:?}", finding.category).to_uppercase().replace('_', "-"),
            level,
            message: SarifMessage {
                text: format!(
                    "{}\nPackage: {} ({})\nSeverity: {:?}",
                    finding.description,
                    package.package_name,
                    package.version,
                    finding.severity
                ),
            },
            locations: vec![SarifLocation {
                physical_location: SarifPhysicalLocation {
                    artifact_location: SarifArtifactLocation { uri },
                    region: Some(SarifRegion {
                        start_line: Some(finding.line),
                        start_column: Some(finding.column),
                        snippet: finding.context.as_ref().map(|ctx| SarifArtifactContent {
                            text: Some(ctx.clone()),
                        }),
                    }),
                },
            }],
            partial_fingerprints: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glassware_core::{Finding, Severity, DetectionCategory};

    fn create_test_finding() -> Finding {
        Finding {
            file: "src/index.js".to_string(),
            line: 10,
            column: 5,
            code_point: 0xFE00,
            character: "\u{FE00}".to_string(),
            raw_bytes: None,
            severity: Severity::High,
            category: DetectionCategory::InvisibleCharacter,
            description: "Invisible character detected".to_string(),
            remediation: "Remove invisible character".to_string(),
            cwe_id: None,
            references: vec![],
            context: Some("const x = 1;".to_string()),
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
    fn test_sarif_formatter_new() {
        let formatter = SarifFormatter::new();
        assert!(formatter.pretty);
        assert!(formatter.tool_version.is_some());
    }

    #[test]
    fn test_sarif_formatter_format() {
        let formatter = SarifFormatter::new();
        let results = vec![create_test_package_result()];

        let sarif = formatter.format(&results).unwrap();
        assert!(sarif.contains("\"$schema\""));
        assert!(sarif.contains("\"version\": \"2.1.0\""));
        assert!(sarif.contains("glassware"));
    }

    #[test]
    fn test_sarif_log_structure() {
        let formatter = SarifFormatter::new();
        let results = vec![create_test_package_result()];
        let log = formatter.create_sarif_log(&results);

        assert_eq!(log.version, "2.1.0");
        assert_eq!(log.runs.len(), 1);
        assert_eq!(log.runs[0].results.len(), 1);
    }

    #[test]
    fn test_sarif_empty_results() {
        let formatter = SarifFormatter::new();
        let results: Vec<PackageScanResult> = vec![];
        let sarif = formatter.format(&results).unwrap();

        assert!(sarif.contains("\"runs\": ["));
    }

    #[test]
    fn test_sarif_with_base_uri() {
        let formatter = SarifFormatter::new().with_base_uri("https://github.com/owner/repo/blob/main");
        let results = vec![create_test_package_result()];

        let sarif = formatter.format(&results).unwrap();
        assert!(sarif.contains("https://github.com/owner/repo/blob/main"));
    }

    #[test]
    fn test_sarif_rule_generation() {
        let finding = create_test_finding();
        let formatter = SarifFormatter::new();
        let rule = formatter.create_rule("InvisibleCharacter", &finding);

        assert!(rule.id.starts_with("GLASSWARE-"));
        assert_eq!(rule.default_configuration.unwrap().level, "error");
    }

    #[test]
    fn test_sarif_result_generation() {
        let finding = create_test_finding();
        let package = create_test_package_result();
        let formatter = SarifFormatter::new();
        let result = formatter.create_sarif_result(&finding, &package);

        assert_eq!(result.level, Some("error".to_string()));
        assert_eq!(result.locations.len(), 1);
        assert_eq!(result.locations[0].physical_location.region.as_ref().unwrap().start_line, Some(10));
    }
}
