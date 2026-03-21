//! Typo Attribution Detector (G3)
//!
//! Detects known typo fingerprints unique to GlassWorm payloads.
//! These are not typos humans make independently -- they are compiler/developer
//! fingerprints that uniquely identify GlassWorm campaign code.
//!
//! ## Verified Typo Fingerprints (from PART5.md)
//!
//! 1. **`LoadLibararyFail`** - from memexec 0.2.0 crate source
//!    - Location: index_ia32.node, index_x64.node
//!    - Context: Error enum variant name
//!
//! 2. **`Invlaid`** - in data.dll V10 key extraction path
//!    - Location: Debug log format string
//!    - Context: `"Decoded Key Is Invlaid!"` (V10 path only, V20 spells correctly)
//!
//! 3. **`NtAllocVmErr`** - in index loaders
//!    - Location: Error string
//!    - Context: NT status error handling
//!
//! ## Severity
//!
//! HIGH for any match -- these are unique fingerprints, not plausible human typos.
//! A single match is strong attribution evidence.
//!
//! ## False Positive Mitigation
//!
//! Only use strings verified in intel source. Do NOT add plausible-looking typos.
//! Do NOT add strings that "might be" GlassWorm-related.
//! Only add strings explicitly documented in PART5.md or other intel sources.

use crate::detector::{Detector, DetectorTier};
use crate::finding::{DetectionCategory, Finding, Severity};
use crate::ir::FileIR;
use std::path::Path;

/// Verified GlassWorm typo fingerprints from PART5.md
/// 
/// Format: (typo_string, description, source_location)
const TYPO_FINGERPRINTS: &[(&str, &str, &str)] = &[
    // From PART5.md line 337: "LoadLibararyFail -- in both index loaders"
    (
        "LoadLibararyFail",
        "GlassWorm memexec crate typo fingerprint (Libarary instead of Library)",
        "PART5.md line 337, index_ia32.node, index_x64.node",
    ),
    // From PART5.md line 339: "Invlaid -- in data.dll at 0x1800084b8"
    (
        "Invlaid",
        "GlassWorm data.dll V10 path typo (Invlaid instead of Invalid)",
        "PART5.md line 339, data.dll V10 key extraction",
    ),
    // From PART5.md line 468, REPORT_index.md line 125
    (
        "NtAllocVmErr",
        "GlassWorm index loader NT status error typo",
        "PART5.md line 468, REPORT_index.md line 125",
    ),
];

/// Detector for typo attribution fingerprints
pub struct TypoAttributionDetector;

impl TypoAttributionDetector {
    /// Create a new typo attribution detector
    pub fn new() -> Self {
        Self
    }
}

impl Default for TypoAttributionDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for TypoAttributionDetector {
    fn name(&self) -> &str {
        "typo_attribution"
    }

    fn tier(&self) -> DetectorTier {
        DetectorTier::Tier3Behavioral
    }

    fn cost(&self) -> u8 {
        1  // Very low cost - simple string matching
    }

    fn signal_strength(&self) -> u8 {
        10  // Maximum signal - these are unique fingerprints
    }

    fn detect(&self, ir: &FileIR) -> Vec<Finding> {
        let mut findings = Vec::new();
        let content = ir.content();

        for (line_num, original_line) in content.lines().enumerate() {
            // Check for each verified typo fingerprint
            for (typo, description, source) in TYPO_FINGERPRINTS {
                if original_line.contains(typo) {
                    findings.push(self.create_finding(
                        Path::new(&ir.metadata.path),
                        line_num + 1,
                        original_line,
                        typo,
                        description,
                        source,
                    ));
                }
            }
        }

        findings
    }
}

impl TypoAttributionDetector {
    fn create_finding(
        &self,
        path: &Path,
        line: usize,
        original_line: &str,
        typo: &str,
        description: &str,
        source: &str,
    ) -> Finding {
        Finding::new(
            &path.to_string_lossy(),
            line,
            1,
            0,
            '\0',
            DetectionCategory::Unknown,  // Using Unknown for attribution
            Severity::High,  // HIGH confidence - unique fingerprints
            &format!("GlassWorm typo attribution fingerprint detected: '{}'", typo),
            &format!("This typo is a confirmed GlassWorm campaign fingerprint. {}. Source: {}", description, source),
        )
        .with_cwe_id("CWE-693")  // Protection Mechanism Failure
        .with_reference("https://github.com/samgrowls/glassworks/blob/main/glassworm-writeup/PART5.md")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::UnicodeConfig;

    #[test]
    fn test_detect_loadlibararyfail() {
        let detector = TypoAttributionDetector::new();
        let content = r#"
            // From index_ia32.node
            enum MemExecError {
                LoadLibararyFail,
                InvalidCString,
                GetProcAddressFail,
            }
        "#;

        let ir = FileIR::build(Path::new("index_ia32.node"), content);
        let findings = detector.detect(&ir);
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::High);
        assert!(findings[0].description.contains("LoadLibararyFail"));
    }

    #[test]
    fn test_detect_invlaid() {
        let detector = TypoAttributionDetector::new();
        let content = r#"
            // From data.dll V10 path
            const log_format = "[%02d:%02d:%02d.%03d-%s-%lu] [!] Decoded Key Is Invlaid!";
        "#;

        let ir = FileIR::build(Path::new("data.dll"), content);
        let findings = detector.detect(&ir);
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::High);
        assert!(findings[0].description.contains("Invlaid"));
    }

    #[test]
    fn test_detect_ntallocvmerr() {
        let detector = TypoAttributionDetector::new();
        let content = r#"
            // From index loaders
            const errors = [
                "NtAllocVmErr",
                "InvalidParameter",
            ];
        "#;

        let ir = FileIR::build(Path::new("index_x64.node"), content);
        let findings = detector.detect(&ir);
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::High);
        assert!(findings[0].description.contains("NtAllocVmErr"));
    }

    #[test]
    fn test_no_detect_correct_spellings() {
        let detector = TypoAttributionDetector::new();
        let content = r#"
            // Legitimate code with correct spellings
            const errors = [
                "LoadLibraryFail",  // Correct spelling
                "Invalid",          // Correct spelling
                "NtAllocVirtualMemoryError",  // Different error
            ];
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_detect_multiple_typos() {
        let detector = TypoAttributionDetector::new();
        let content = r#"
            // Multiple GlassWorm fingerprints
            enum Error {
                LoadLibararyFail,
            }
            const log = "Decoded Key Is Invlaid!";
            const nt_err = "NtAllocVmErr";
        "#;

        let ir = FileIR::build(Path::new("test.node"), content);
        let findings = detector.detect(&ir);
        assert_eq!(findings.len(), 3);
    }

    #[test]
    fn test_no_detect_legitimate_code() {
        let detector = TypoAttributionDetector::new();
        let content = r#"
            // Legitimate JavaScript code
            function loadLibrary(name) {
                try {
                    return require(name);
                } catch (e) {
                    console.log("Failed to load library:", e.message);
                }
            }
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);
        assert!(findings.is_empty());
    }
}
