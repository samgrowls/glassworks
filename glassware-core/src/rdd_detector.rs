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

/// Convert byte offset to (line, column) position (1-indexed)
fn byte_offset_to_position(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    
    for (idx, ch) in source.char_indices() {
        if idx >= offset {
            return (line, col);
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    
    (line, col)
}

/// Find the byte offset of a dependency value (the URL part)
fn find_dependency_value_offset(content: &str, dep_name: &str, url: &str) -> Option<usize> {
    // Search for the pattern: "dep_name": "url"
    let pattern = format!("\"{}\": \"{}\"", dep_name, url);
    if let Some(start) = content.find(&pattern) {
        // Return offset to the start of the URL value (after the quote)
        let url_start = start + format!("\"{}\": \"", dep_name).len();
        return Some(url_start);
    }
    None
}

/// Find the byte offset of the author name value
fn find_author_name_offset(content: &str, author_name: &str) -> Option<usize> {
    // Search for the pattern: "name": "JPD" or "name": "JPD"
    let pattern = format!("\"name\": \"{}\"", author_name);
    if let Some(start) = content.find(&pattern) {
        // Return offset to the start of the author name value
        let name_start = start + format!("\"name\": \"").len();
        return Some(name_start);
    }
    None
}

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
                // Find the actual position of the author name in the content
                let (line, column) = find_author_name_offset(&ctx.content, author)
                    .map(|offset| byte_offset_to_position(&ctx.content, offset))
                    .unwrap_or((1, 1));

                findings.push(
                    Finding::new(
                        &path.to_string_lossy(),
                        line,
                        column,
                        0,
                        '\0',
                        DetectionCategory::JpdAuthor,
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

                        // Find the actual position of the URL value in the content
                        let (line, column) = find_dependency_value_offset(&ctx.content, name, url)
                            .map(|offset| byte_offset_to_position(&ctx.content, offset))
                            .unwrap_or((1, 1));

                        findings.push(
                            Finding::new(
                                &path.to_string_lossy(),
                                line,
                                column,
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

                        // Find the actual position of the URL value in the content
                        let (line, column) = find_dependency_value_offset(&ctx.content, name, url)
                            .map(|offset| byte_offset_to_position(&ctx.content, offset))
                            .unwrap_or((1, 1));

                        findings.push(
                            Finding::new(
                                &path.to_string_lossy(),
                                line,
                                column,
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
        // Verify line number is not hardcoded to 1 (should be line 5 where the URL is)
        assert!(findings[0].line > 1, "Line number should be actual line, not hardcoded 1");
        assert_eq!(findings[0].line, 5, "URL dependency should be reported at line 5");
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
        assert_eq!(findings[0].category, DetectionCategory::JpdAuthor);
        assert_eq!(findings[0].severity, Severity::Critical);
        // Verify line number is not hardcoded to 1 (should be line 5 where "JPD" is)
        assert!(findings[0].line > 1, "Line number should be actual line, not hardcoded 1");
        assert_eq!(findings[0].line, 5, "JPD author should be reported at line 5");
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
        // Verify line number is not hardcoded to 1 (should be line 5 where the URL is)
        assert!(findings[0].line > 1, "Line number should be actual line, not hardcoded 1");
        assert_eq!(findings[0].line, 5, "URL devDependency should be reported at line 5");
    }

    #[test]
    fn test_line_number_accuracy() {
        let detector = RddDetector::new();
        // Test with known line numbers
        let content = r#"{
  "name": "test-package",
  "version": "1.0.0",
  "dependencies": {
    "legit-pkg": "^1.0.0",
    "malicious-pkg": "https://evil.com/pkg"
  }
}"#;

        let findings = detector.scan(Path::new("package.json"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert_eq!(findings[0].line, 6, "URL dependency should be reported at line 6");
        // Column points to the first character of the URL value (after the opening quote)
        assert_eq!(findings[0].column, 23, "URL dependency should be reported at column 23");
    }

    #[test]
    fn test_jpd_author_line_number() {
        let detector = RddDetector::new();
        // Test with known line numbers
        let content = r#"{
  "name": "test-package",
  "version": "1.0.0",
  "author": {
    "name": "JPD"
  }
}"#;

        let findings = detector.scan(Path::new("package.json"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert_eq!(findings[0].category, DetectionCategory::JpdAuthor);
        // The author name is on line 5 (not 6) in this JSON structure
        assert_eq!(findings[0].line, 5, "JPD author should be reported at line 5");
        assert_eq!(findings[0].column, 14, "JPD author should be reported at column 14");
    }

    #[test]
    fn test_multiple_rdd_findings() {
        let detector = RddDetector::new();
        let content = r#"{
            "name": "test-package",
            "version": "1.0.0",
            "dependencies": {
                "pkg1": "http://evil.com/pkg1",
                "pkg2": "https://bad.com/pkg2"
            },
            "devDependencies": {
                "pkg3": "http://malicious.com/pkg3"
            }
        }"#;

        let findings = detector.scan(Path::new("package.json"), content, &UnicodeConfig::default());
        assert_eq!(findings.len(), 3, "Should detect 3 URL dependencies");
        
        // Verify each finding has correct line numbers
        assert!(findings.iter().all(|f| f.line > 1), "All findings should have actual line numbers");
        
        // Verify line numbers are different for different dependencies
        let lines: Vec<usize> = findings.iter().map(|f| f.line).collect();
        assert!(lines[0] != lines[1] || lines[1] != lines[2], "Different dependencies should be on different lines");
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
