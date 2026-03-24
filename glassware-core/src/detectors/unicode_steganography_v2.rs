//! Unicode Steganography V2 Detector (GlassWorm-specific)
//!
//! Detects GlassWorm-specific steganography patterns using zero-width characters
//! for binary encoding and data hiding.
//!
//! ## Detection Logic
//!
//! This detector emits findings when:
//! 1. ZWSP/ZWNJ binary encoding with balanced ratio (0.5-2.0) detected
//! 2. Hidden data in package.json fields (description, keywords, author, repository)
//! 3. Base64 in comments with invisible characters
//!
//! ## GlassWorm Encoding Scheme
//!
//! GlassWorm uses a binary encoding scheme:
//! - U+200B (ZWSP) = Binary 0
//! - U+200C (ZWNJ) = Binary 1
//! - U+200D (ZWJ) = Padding/separator
//! - U+FEFF (BOM) = Marker/start delimiter
//!
//! The balanced ratio (0.5-2.0) of ZWSP to ZWNJ suggests actual binary data
//! rather than random invisible character insertion.
//!
//! ## Severity
//!
//! Critical: Balanced ZWSP/ZWNJ ratio with high count (>50 each)
//! High: Hidden data in package.json fields (>5 invisible chars)
//! High: Base64 in comments with invisible chars

use crate::detector::{Detector, DetectorMetadata, DetectorTier};
use crate::finding::{DetectionCategory, Finding, Severity};
use crate::ir::FileIR;
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

/// Minimum count for ZWSP/ZWNJ to consider binary encoding
const MIN_BINARY_ENCODING_COUNT: usize = 50;

/// Balanced ratio range for ZWSP/ZWNJ (GlassWorm signature)
const RATIO_MIN: f32 = 0.5;
const RATIO_MAX: f32 = 2.0;

/// Minimum invisible chars in package.json field to flag
const MIN_PACKAGE_JSON_INVISIBLE: usize = 5;

/// Minimum invisible chars in base64 comment to flag
const MIN_BASE64_INVISIBLE: usize = 3;

/// Patterns for base64 detection in comments (allows invisible chars within base64)
static BASE64_COMMENT_PATTERN: Lazy<Regex> = Lazy::new(|| {
    // Match comments with base64-like content (allowing invisible chars within)
    Regex::new(r"(?://|/\*|\*).*?[A-Za-z0-9+/=\u{200B}\u{200C}\u{200D}\u{FEFF}]{50,}").unwrap()
});

/// Detector for GlassWorm-style unicode steganography
pub struct UnicodeSteganographyV2Detector;

impl UnicodeSteganographyV2Detector {
    /// Create a new unicode steganography v2 detector
    pub fn new() -> Self {
        Self
    }
}

impl Default for UnicodeSteganographyV2Detector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for UnicodeSteganographyV2Detector {
    fn name(&self) -> &str {
        "unicode_steganography_v2"
    }

    fn tier(&self) -> DetectorTier {
        DetectorTier::Tier2Secondary
    }

    fn cost(&self) -> u8 {
        4  // Medium cost - multiple passes for ratio calculation
    }

    fn signal_strength(&self) -> u8 {
        9  // High signal - balanced ZWSP/ZWNJ is strong GlassWorm indicator
    }

    fn prerequisites(&self) -> Vec<&'static str> {
        vec!["invisible_char"]  // Run after basic invisible char detection
    }

    fn detect(&self, ir: &FileIR) -> Vec<Finding> {
        let mut findings = Vec::new();
        let content = ir.content();
        let path = &ir.metadata.path;

        // Check if this is a package.json file
        let is_package_json = path.to_lowercase().ends_with("package.json");

        // Pattern 1: ZWSP/ZWNJ binary encoding detection
        findings.extend(self.detect_binary_encoding(content, path));

        // Pattern 2: Hidden data in package.json fields
        if is_package_json {
            findings.extend(self.detect_package_json_stego(content, path));
        }

        // Pattern 3: Base64 in comments with invisible chars
        findings.extend(self.detect_base64_comments(content, path));

        findings
    }

    fn metadata(&self) -> DetectorMetadata {
        DetectorMetadata {
            name: "unicode_steganography_v2".to_string(),
            version: "1.0.0".to_string(),
            description: "Detects GlassWorm-style unicode steganography including binary encoding and hidden data in JSON fields".to_string(),
        }
    }
}

impl UnicodeSteganographyV2Detector {
    /// Detect ZWSP/ZWNJ binary encoding with balanced ratio
    fn detect_binary_encoding(&self, content: &str, path: &str) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Count ZWSP (U+200B) and ZWNJ (U+200C) occurrences
        let zwsp_count = content.matches('\u{200B}').count();
        let zwnj_count = content.matches('\u{200C}').count();

        // Check if both exceed minimum threshold
        if zwsp_count >= MIN_BINARY_ENCODING_COUNT && zwnj_count >= MIN_BINARY_ENCODING_COUNT {
            // Calculate ratio
            let ratio = zwsp_count as f32 / zwnj_count as f32;

            // Check for balanced ratio (GlassWorm signature)
            if ratio >= RATIO_MIN && ratio <= RATIO_MAX {
                // Find first occurrence line for reporting
                let first_line = content
                    .lines()
                    .position(|line| line.contains('\u{200B}') || line.contains('\u{200C}'))
                    .unwrap_or(0)
                    + 1;

                findings.push(
                    Finding::new(
                        path,
                        first_line,
                        1,
                        0x200B,
                        '\u{200B}',
                        DetectionCategory::SteganoPayload,
                        Severity::Critical,
                        &format!(
                            "GlassWorm binary encoding detected: {} ZWSP + {} ZWNJ (ratio: {:.2})",
                            zwsp_count, zwnj_count, ratio
                        ),
                        "CRITICAL: Balanced ZWSP/ZWNJ ratio is a strong GlassWorm steganography signature. \
                         These characters encode binary data (ZWSP=0, ZWNJ=1). Extract and decode the hidden payload.",
                    )
                    .with_cwe_id("CWE-359")
                    .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
                    .with_confidence(0.90),
                );
            }
        }

        findings
    }

    /// Detect hidden data in package.json fields
    fn detect_package_json_stego(&self, content: &str, path: &str) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Fields commonly abused for steganography
        let target_fields = ["description", "keywords", "author", "repository", "name", "version"];

        // Simple JSON field extraction (handles basic cases)
        for field in target_fields.iter() {
            // Look for "field": "value" or "field": "value"
            let patterns = [
                format!(r#""{}"\s*:\s*"([^"]*)""#, field),
                format!(r#"'{}'\s*:\s*'([^']*)'"#, field),
            ];

            for pattern in &patterns {
                if let Ok(re) = Regex::new(pattern) {
                    for cap in re.captures_iter(content) {
                        if let Some(value) = cap.get(1) {
                            let field_value = value.as_str();

                            // Count invisible characters in field value
                            let invisible_count = field_value
                                .matches(|c: char| {
                                    matches!(
                                        c as u32,
                                        0x200B | 0x200C | 0x200D | 0xFEFF | 0x2060
                                    )
                                })
                                .count();

                            if invisible_count >= MIN_PACKAGE_JSON_INVISIBLE {
                                let line = content[..cap.get(0).unwrap().start()]
                                    .lines()
                                    .count()
                                    + 1;

                                findings.push(
                                    Finding::new(
                                        path,
                                        line,
                                        1,
                                        0x200B,
                                        '\u{200B}',
                                        DetectionCategory::SteganoPayload,
                                        Severity::High,
                                        &format!(
                                            "Hidden data in package.json '{}' field ({} invisible chars)",
                                            field, invisible_count
                                        ),
                                        &format!(
                                            "GlassWorm hides C2 addresses and commands in package.json fields. \
                                             Review the '{}' field for steganographic encoding.",
                                            field
                                        ),
                                    )
                                    .with_cwe_id("CWE-359")
                                    .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
                                    .with_confidence(0.88)
                                    .with_context(&format!(
                                        "Field: {}, Invisible chars: {}",
                                        field, invisible_count
                                    )),
                                );
                            }
                        }
                    }
                }
            }
        }

        findings
    }

    /// Detect base64 data in comments with invisible characters
    fn detect_base64_comments(&self, content: &str, path: &str) -> Vec<Finding> {
        let mut findings = Vec::new();

        for mat in BASE64_COMMENT_PATTERN.find_iter(content) {
            let comment = mat.as_str();
            let start_pos = mat.start();

            // Count invisible characters in the comment
            let invisible_count = comment
                .matches(|c: char| {
                    matches!(
                        c as u32,
                        0x200B | 0x200C | 0x200D | 0xFEFF | 0x2060
                    )
                })
                .count();

            if invisible_count >= MIN_BASE64_INVISIBLE {
                let line = content[..start_pos].lines().count() + 1;

                findings.push(
                    Finding::new(
                        path,
                        line,
                        1,
                        0x200B,
                        '\u{200B}',
                        DetectionCategory::SteganoPayload,
                        Severity::High,
                        "Base64 data with invisible characters in comment",
                        "GlassWorm hides encoded payloads in comments using invisible characters as delimiters or encoding. \
                         Decode the base64 and analyze for malicious content.",
                    )
                    .with_cwe_id("CWE-359")
                    .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
                    .with_confidence(0.85)
                    .with_context(comment),
                );
            }
        }

        findings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::UnicodeConfig;

    #[test]
    fn test_detect_balanced_binary_encoding() {
        let detector = UnicodeSteganographyV2Detector::new();

        // Create content with balanced ZWSP/ZWNJ ratio (GlassWorm signature)
        let mut binary_data = String::new();
        for _ in 0..60 {
            binary_data.push('\u{200B}'); // ZWSP = 0
            binary_data.push('\u{200C}'); // ZWNJ = 1
        }

        let content = format!(
            r#"
            // Hidden data encoded below
            const hidden = "{}";
            function decode(data) {{
                return atob(data);
            }}
            "#,
            binary_data
        );

        let ir = FileIR::build(Path::new("test.js"), &content);
        let findings = detector.detect(&ir);

        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::Critical);
        assert!(findings[0].description.contains("binary encoding"));
        assert!(findings[0].confidence.unwrap_or(0.0) >= 0.85);
    }

    #[test]
    fn test_no_detect_unbalanced_ratio() {
        let detector = UnicodeSteganographyV2Detector::new();

        // Create content with unbalanced ratio (not GlassWorm)
        let mut unbalanced = String::new();
        for _ in 0..100 {
            unbalanced.push('\u{200B}'); // Mostly ZWSP
        }
        for _ in 0..5 {
            unbalanced.push('\u{200C}'); // Very few ZWNJ
        }

        let content = format!("const data = \"{}\";", unbalanced);

        let ir = FileIR::build(Path::new("test.js"), &content);
        let findings = detector.detect(&ir);

        // Should not flag unbalanced ratio
        assert!(findings.is_empty());
    }

    #[test]
    fn test_detect_package_json_stego() {
        let detector = UnicodeSteganographyV2Detector::new();

        // Create package.json with hidden data in description (10 invisible chars)
        let mut invisible_data = String::new();
        for _ in 0..5 {
            invisible_data.push('\u{200B}'); // ZWSP
            invisible_data.push('\u{200C}'); // ZWNJ
        }
        let content = format!(r#"{{
            "name": "test-package",
            "version": "1.0.0",
            "description": "A normal{} package",
            "author": "Test Author",
            "keywords": ["test", "package"]
        }}"#, invisible_data);

        let ir = FileIR::build(Path::new("package.json"), &content);
        let findings = detector.detect(&ir);

        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::High);
        assert!(findings[0].description.contains("description"));
    }

    #[test]
    fn test_detect_base64_comment_with_invisible() {
        let detector = UnicodeSteganographyV2Detector::new();

        // Create content with base64 in comment + invisible chars interspersed
        // The invisible chars are embedded within the base64 string (GlassWorm technique)
        let mut base64_with_invisible = String::new();
        let base64_chars = "EAGSbG9iYWwudGVzdCA9ICJleGVjdXRlZCI7CgABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        for (i, ch) in base64_chars.chars().enumerate() {
            base64_with_invisible.push(ch);
            // Insert invisible chars every 10 characters
            if i > 0 && i % 10 == 0 {
                base64_with_invisible.push('\u{200B}'); // ZWSP
                base64_with_invisible.push('\u{200C}'); // ZWNJ
            }
        }
        
        let content = format!(r#"
            // {}==
            const config = {{}};
        "#, base64_with_invisible);

        let ir = FileIR::build(Path::new("test.js"), &content);
        let findings = detector.detect(&ir);

        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::High);
        assert!(findings[0].description.contains("Base64"));
    }

    #[test]
    fn test_no_detect_clean_content() {
        let detector = UnicodeSteganographyV2Detector::new();

        let content = r#"
            // Normal comment
            const data = "Hello World";
            console.log(data);
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);

        assert!(findings.is_empty());
    }

    #[test]
    fn test_detect_below_threshold() {
        let detector = UnicodeSteganographyV2Detector::new();

        // Create content with few invisible chars (below threshold)
        let content = r#"
            const data = "\u{200B}\u{200C}\u{200B}";
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);

        // Should not flag - below minimum count
        assert!(findings.is_empty());
    }
}
