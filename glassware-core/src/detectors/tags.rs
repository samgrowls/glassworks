//! Unicode Tag Detector
//!
//! Detects Unicode tag characters that can be used for metadata injection.
//!
//! Unicode Range:
//! - U+E0000-U+E007F: Tags (language tags, etc.)

use crate::config::UnicodeConfig;
use crate::context_filter::{classify_file_by_path, FileClassification};
use crate::detector::{Detector, DetectorMetadata, DetectorTier, ScanContext};
use crate::finding::{DetectionCategory, Finding, Severity};
use crate::ir::FileIR;
use std::path::Path;

/// Detector for Unicode tag attacks
pub struct UnicodeTagDetector {
    #[allow(dead_code)]
    config: UnicodeConfig,
}

impl UnicodeTagDetector {
    /// Create a new Unicode tag detector
    pub fn new(config: UnicodeConfig) -> Self {
        Self { config }
    }

    /// Create with default config
    pub fn with_default_config() -> Self {
        Self::new(UnicodeConfig::default())
    }

    /// Scan content for Unicode tag attacks
    pub fn detect_with_content(&self, content: &str, file_path: &str) -> Vec<Finding> {
        // Build IR and call detect (for backward compatibility)
        use crate::ir::FileIR;
        let ir = FileIR::build(Path::new(file_path), content);
        self.detect(&ir)
    }

    /// Internal implementation of detection logic
    fn detect_impl(&self, content: &str, file_path: &str) -> Vec<Finding> {
        let mut findings = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            for (col_num, ch) in line.chars().enumerate() {
                let code_point = ch as u32;

                // Check if this is in the tags range (U+E0000-U+E007F)
                if (0xE0000..=0xE007F).contains(&code_point) {
                    let tag_name = Self::get_tag_name(code_point);

                    let finding = Finding::new(
                        file_path,
                        line_num + 1,
                        col_num + 1,
                        code_point,
                        ch,
                        DetectionCategory::UnicodeTag,
                        Severity::Medium,
                        &format!(
                            "Unicode tag character detected: {} (U+{:04X})",
                            tag_name, code_point
                        ),
                        "Remove the Unicode tag character. These are rarely used in legitimate \
                         code and can be used to inject hidden metadata or bypass security checks. \
                         If this appears in a string literal, it may be an attempt to hide data.",
                    )
                    .with_cwe_id("CWE-172")
                    .with_context(&Self::get_context(line, col_num));

                    findings.push(finding);
                }
            }
        }

        findings
    }

    /// Get human-readable name for a tag character
    fn get_tag_name(code_point: u32) -> String {
        match code_point {
            0xE0001 => "Language Tag".to_string(),
            0xE007F => "Cancel Tag".to_string(),
            0xE0020..=0xE007E => {
                let ascii = (code_point - 0xE0000) as u8;
                if (0x20..=0x7E).contains(&ascii) {
                    format!("Tag: {}", ascii as char)
                } else {
                    format!("Tag (U+{:04X})", code_point)
                }
            }
            _ => format!("Tag (U+{:04X})", code_point),
        }
    }

    /// Get context around the character position (Unicode-safe)
    fn get_context(line: &str, char_pos: usize) -> String {
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        let start = char_pos.saturating_sub(20);
        let end = (char_pos + 20).min(len);

        let prefix = if start > 0 { "..." } else { "" };
        let suffix = if end < len { "..." } else { "" };

        let context: String = chars[start..end].iter().collect();
        format!("{}{}{}", prefix, context, suffix)
    }
}

impl Detector for UnicodeTagDetector {
    fn name(&self) -> &str {
        "unicode_tag"
    }

    fn tier(&self) -> DetectorTier {
        DetectorTier::Tier1Primary
    }

    fn detect(&self, ir: &FileIR) -> Vec<Finding> {
        // Skip test/data/build files to reduce false positives
        match classify_file_by_path(Path::new(&ir.metadata.path)) {
            FileClassification::Test => {
                tracing::debug!("UnicodeTag: Skipping test file: {:?}", ir.metadata.path);
                return vec![];
            }
            FileClassification::Data => {
                tracing::debug!("UnicodeTag: Skipping data file: {:?}", ir.metadata.path);
                return vec![];
            }
            FileClassification::BuildOutput => {
                tracing::debug!("UnicodeTag: Skipping build output: {:?}", ir.metadata.path);
                return vec![];
            }
            FileClassification::Production => {}  // Continue detection
        }

        self.detect_impl(ir.content(), &ir.metadata.path)
    }

    fn cost(&self) -> u8 {
        1  // Very cheap - single pass range check
    }

    fn signal_strength(&self) -> u8 {
        7  // High signal - tag characters are rare in legitimate code
    }

    fn metadata(&self) -> DetectorMetadata {
        DetectorMetadata {
            name: "unicode_tag".to_string(),
            version: "1.0.0".to_string(),
            description: "Detects Unicode tag characters (U+E0000-U+E007F) used for metadata injection".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_detection() {
        let detector = UnicodeTagDetector::with_default_config();

        // Language tag character
        let content = "const x = \"test\u{E0001}value\";";
        let findings = detector.detect_with_content(content, "test.js");

        assert!(!findings.is_empty());
        assert_eq!(findings[0].code_point, 0xE0001);
        assert_eq!(findings[0].category, DetectionCategory::UnicodeTag);
    }

    #[test]
    fn test_clean_content() {
        let detector = UnicodeTagDetector::with_default_config();

        let content = "const normal = 'hello world';";
        let findings = detector.detect_with_content(content, "test.js");

        assert!(findings.is_empty());
    }
}
