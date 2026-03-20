//! JPD Author Detector
//!
//! Detects npm packages authored by "JPD" - a signature of the PhantomRaven campaign.
//! All 126+ PhantomRaven packages use this author name.
//!
//! ## Detection Logic
//!
//! Scans package.json for author field matching "JPD"
//!
//! ## Severity
//!
//! Critical - strong indicator of PhantomRaven campaign

use crate::detector::{Detector, DetectorMetadata};
use crate::finding::{DetectionCategory, Finding, Severity};
use crate::ir::FileIR;
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

/// Find the byte offset of the author name value
fn find_author_name_offset(content: &str, author_name: &str) -> Option<usize> {
    // Search for the pattern: "name": "JPD" or "author": "JPD"
    let pattern_name = format!("\"name\": \"{}\"", author_name);
    let pattern_author = format!("\"author\": \"{}\"", author_name);
    
    if let Some(start) = content.find(&pattern_name) {
        // Return offset to the first character of the author name value
        let name_start = start + "\"name\": \"".len();
        return Some(name_start);
    }
    
    if let Some(start) = content.find(&pattern_author) {
        // Return offset to the first character of the author name value
        let name_start = start + "\"author\": \"".len();
        return Some(name_start);
    }
    
    None
}

/// Detector for JPD author signature
pub struct JpdAuthorDetector;

impl JpdAuthorDetector {
    /// Create a new JPD author detector
    pub fn new() -> Self {
        Self
    }
}

impl Default for JpdAuthorDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for JpdAuthorDetector {
    fn name(&self) -> &str {
        "jpd_author"
    }

    fn detect(&self, ir: &FileIR) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Only scan package.json files
        if !ir.is_package_json() {
            return findings;
        }

        // Use pre-parsed JSON from IR
        let json = match ir.json() {
            Some(v) => v,
            None => return findings,
        };

        // Check for "JPD" author in various formats
        let author_matches = [
            json.get("author").and_then(|a| a.get("name")).and_then(|n| n.as_str()),
            json.get("author").and_then(|a| a.as_str()),
            json.get("maintainers").and_then(|m| m.as_array()).and_then(|arr| {
                arr.iter().find_map(|m| m.get("name").and_then(|n| n.as_str()))
            }),
        ];

        for author_opt in author_matches.iter().flatten() {
            if author_opt == &"JPD" || author_opt == &"jpd" || author_opt == &"Jpd" {
                // Find the actual position of the author name in the content
                let (line, column) = find_author_name_offset(ir.content(), author_opt)
                    .map(|offset| byte_offset_to_position(ir.content(), offset))
                    .unwrap_or((1, 1));

                findings.push(
                    Finding::new(
                        &ir.metadata.path,
                        line,
                        column,
                        0,
                        '\0',
                        DetectionCategory::JpdAuthor,
                        Severity::Critical,
                        "PhantomRaven campaign signature: 'JPD' author detected",
                        "All 126+ PhantomRaven malicious packages use the 'JPD' author signature. This is a strong indicator of malicious intent. Immediate investigation required.",
                    )
                    .with_cwe_id("CWE-506")
                    .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode"),
                );
            }
        }

        findings
    }

    fn metadata(&self) -> DetectorMetadata {
        DetectorMetadata {
            name: "jpd_author".to_string(),
            version: "1.0.0".to_string(),
            description: "Detects PhantomRaven campaign signature: 'JPD' author field in package.json".to_string().to_string(),
        }
    }
}

impl JpdAuthorDetector {
    /// Backward compatibility method for tests
    pub fn scan(&self, path: &Path, content: &str, _config: &crate::config::UnicodeConfig) -> Vec<Finding> {
        // Build IR and call detect (for backward compatibility)
        let ir = FileIR::build(path, content);
        self.detect(&ir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::UnicodeConfig;

    #[test]
    fn test_detect_jpd_author_object() {
        let detector = JpdAuthorDetector::new();
        let content = r#"{
            "name": "test-package",
            "author": {
                "name": "JPD"
            }
        }"#;

        let findings = detector.scan(Path::new("package.json"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert_eq!(findings[0].category, DetectionCategory::JpdAuthor);
        assert_eq!(findings[0].severity, Severity::Critical);
        // Verify line number is accurate (line 4 where "JPD" appears in this formatting)
        assert_eq!(findings[0].line, 4, "JPD author should be reported at line 4");
    }

    #[test]
    fn test_detect_jpd_author_string() {
        let detector = JpdAuthorDetector::new();
        let content = r#"{
            "name": "test-package",
            "author": "JPD"
        }"#;

        let findings = detector.scan(Path::new("package.json"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::Critical);
        // Verify line number is accurate (line 3 where "JPD" appears)
        assert_eq!(findings[0].line, 3, "JPD author should be reported at line 3");
    }

    #[test]
    fn test_jpd_author_line_number_accuracy() {
        let detector = JpdAuthorDetector::new();
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
        assert_eq!(findings[0].line, 5, "JPD author should be reported at line 5");
        assert!(findings[0].column > 1, "Column should be accurate, not hardcoded to 1");
    }

    #[test]
    fn test_no_detect_legitimate_author() {
        let detector = JpdAuthorDetector::new();
        let content = r#"{
            "name": "legitimate-package",
            "author": {
                "name": "John Doe"
            }
        }"#;

        let findings = detector.scan(Path::new("package.json"), content, &UnicodeConfig::default());
        assert!(findings.is_empty());
    }

    #[test]
    fn test_no_detect_non_package_json() {
        let detector = JpdAuthorDetector::new();
        let content = r#"{"author": "JPD"}"#;

        let findings = detector.scan(Path::new("config.json"), content, &UnicodeConfig::default());
        assert!(findings.is_empty());
    }
}
