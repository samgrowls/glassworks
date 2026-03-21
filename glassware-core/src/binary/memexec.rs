//! G9: memexec Detector
//!
//! Detects memexec crate usage for fileless PE loading.
//!
//! ## Intel Source
//!
//! From PART4.md and PART5.md, GlassWorm uses memexec 0.2.0 for in-memory
//! PE loading. Key indicators include:
//!
//! - Crate version string: `memexec 0.2.0`
//! - Error strings with typos: `LoadLibararyFail` (note: "Libarary" typo)
//! - Internal DLL name: `generate.dll`
//! - PDB string: `generate.pdb`
//! - Entry point: `generate::run`
//! - Native API imports: `NtAllocateVirtualMemory` (dynamically resolved)
//!
//! ## Detection Strategy
//!
//! String extraction from binary. The memexec crate leaves distinctive
//! fingerprints including the typo `LoadLibararyFail` which is unique
//! to the crate source.
//!
//! ## Severity
//!
//! **HIGH** - Fileless execution technique, confirmed GlassWorm TTP.

use crate::detector::Detector;
use crate::finding::{DetectionCategory, Finding, Severity};
use crate::ir::FileIR;

#[cfg(feature = "binary")]
use crate::binary::extract_features;

/// memexec crate version string
const MEMEXEC_VERSION: &str = "memexec 0.2.0";

/// memexec error strings (including the famous typo)
const MEMEXEC_ERRORS: &[&str] = &[
    "LoadLibararyFail",  // Typo preserved from crate source
    "NtAllocVmErr",      // NT allocation error
];

/// memexec internal names
const MEMEXEC_INTERNALS: &[&str] = &[
    "generate.dll",
    "generate.pdb",
    "generate::run",
    "memexec::",
];

/// Detector for memexec crate usage
pub struct MemexecDetector;

impl MemexecDetector {
    /// Create a new memexec detector
    pub fn new() -> Self {
        Self
    }
}

impl Default for MemexecDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "binary")]
impl MemexecDetector {
    /// Analyze binary features for memexec fingerprints
    fn analyze_binary(&self, features: &crate::binary::BinaryFeatures, path: &str) -> Vec<Finding> {
        let mut found_indicators = Vec::new();
        let mut has_typo = false;

        // Search for memexec indicators in strings
        for string in &features.strings {
            let content = &string.content;

            // Check version string
            if content.contains(MEMEXEC_VERSION) {
                found_indicators.push(("version", MEMEXEC_VERSION));
            }

            // Check error strings
            for &error in MEMEXEC_ERRORS {
                if content.contains(error) {
                    found_indicators.push(("error", error));
                    if error == "LoadLibararyFail" {
                        has_typo = true;
                    }
                }
            }

            // Check internal names
            for &internal in MEMEXEC_INTERNALS {
                if content.contains(internal) {
                    found_indicators.push(("internal", internal));
                }
            }

            // Check for memexec crate name alone
            if content == "memexec" || content.contains("memexec ") {
                found_indicators.push(("crate", "memexec"));
            }
        }

        // Also check debug info
        if let Some(debug) = &features.debug_info {
            if let Some(pdb_path) = &debug.pdb_path {
                if pdb_path.contains("generate.pdb") {
                    found_indicators.push(("debug", "generate.pdb"));
                }
            }
        }

        // Deduplicate
        found_indicators.sort();
        found_indicators.dedup();

        // Determine severity
        if has_typo || found_indicators.len() >= 2 {
            // HIGH: Typo fingerprint or multiple indicators
            return vec![self.create_finding(
                path,
                1,
                &found_indicators.iter().map(|(_, i)| *i).collect::<Vec<_>>(),
                found_indicators.len(),
                has_typo,
                Severity::High,
                "memexec fileless loader detected",
            )];
        } else if !found_indicators.is_empty() {
            // MEDIUM: Single indicator (could be legitimate Rust project)
            return vec![self.create_finding(
                path,
                1,
                &found_indicators.iter().map(|(_, i)| *i).collect::<Vec<_>>(),
                found_indicators.len(),
                has_typo,
                Severity::Medium,
                "memexec crate usage detected",
            )];
        }

        vec![]
    }

    fn create_finding(
        &self,
        path: &str,
        line: usize,
        indicators: &[&str],
        count: usize,
        has_typo: bool,
        severity: Severity,
        message: &str,
    ) -> Finding {
        let context = if has_typo {
            format!(
                "Indicators: {:?}, **TYPO FINGERPRINT: LoadLibararyFail**",
                indicators
            )
        } else {
            format!("Indicators: {:?}, Count: {}", indicators, count)
        };

        Finding::new(
            path,
            line,
            1,
            0,
            '\0',
            DetectionCategory::MemexecLoader,
            severity,
            message,
            "GlassWorm uses memexec crate for fileless PE loading. Review for embedded payloads.",
        )
        .with_cwe_id("CWE-506")
        .with_reference("https://codeberg.org/tip-o-deincognito/glassworm-writeup/raw/branch/main/PART4.md")
        .with_context(&context)
        .with_confidence(if has_typo { 0.98 } else { 0.7 })
    }
}

impl Detector for MemexecDetector {
    fn name(&self) -> &str {
        "memexec"
    }

    fn tier(&self) -> crate::detector::DetectorTier {
        crate::detector::DetectorTier::Tier2Secondary
    }

    fn cost(&self) -> u8 {
        5  // Medium cost - string search
    }

    fn signal_strength(&self) -> u8 {
        9  // Very high signal - especially with typo fingerprint
    }

    fn detect(&self, ir: &FileIR) -> Vec<Finding> {
        #[cfg(feature = "binary")]
        {
            // Only analyze .node files
            let path = ir.metadata.path.as_str();
            if !path.ends_with(".node") {
                return vec![];
            }

            // Try to extract binary features from the IR data
            let data = ir.data();
            if data.is_empty() {
                return vec![];
            }

            match extract_features(data) {
                Ok(features) => self.analyze_binary(&features, path),
                Err(_) => vec![],
            }
        }

        #[cfg(not(feature = "binary"))]
        {
            vec![]
        }
    }
}

#[cfg(all(test, feature = "binary"))]
mod tests {
    use super::*;
    use crate::binary::{BinaryFeatures, ExtractedString};
    use crate::ir::FileIR;
    use std::path::Path;

    #[test]
    fn test_detector_name() {
        let detector = MemexecDetector::new();
        assert_eq!(detector.name(), "memexec");
    }

    #[test]
    fn test_detector_tier() {
        let detector = MemexecDetector::new();
        assert_eq!(detector.tier(), crate::detector::DetectorTier::Tier2Secondary);
    }

    #[test]
    fn test_skip_non_node_files() {
        let detector = MemexecDetector::new();
        let content = "const x = 42;";
        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_detect_memexec_typo_fingerprint() {
        let features = BinaryFeatures {
            format: crate::binary::BinaryFormat::PE,
            imports: vec![],
            exports: vec![],
            strings: vec![ExtractedString {
                content: "LoadLibararyFail".to_string(),  // The famous typo
                offset: 0x1000,
                length: 16,
            }],
            sections: vec![],
            debug_info: None,
            data: vec![],
        };

        let detector = MemexecDetector::new();
        let findings = detector.analyze_binary(&features, "test.node");
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::High);
        assert!(findings[0].confidence.unwrap_or(0.0) >= 0.95);
    }

    #[test]
    fn test_detect_memexec_version() {
        let features = BinaryFeatures {
            format: crate::binary::BinaryFormat::PE,
            imports: vec![],
            exports: vec![],
            strings: vec![
                ExtractedString {
                    content: "memexec 0.2.0".to_string(),
                    offset: 0x1000,
                    length: 13,
                },
                ExtractedString {
                    content: "generate.dll".to_string(),
                    offset: 0x2000,
                    length: 12,
                },
            ],
            sections: vec![],
            debug_info: None,
            data: vec![],
        };

        let detector = MemexecDetector::new();
        let findings = detector.analyze_binary(&features, "test.node");
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::High);
    }

    #[test]
    fn test_detect_memexec_pdb() {
        let features = BinaryFeatures {
            format: crate::binary::BinaryFormat::PE,
            imports: vec![],
            exports: vec![],
            strings: vec![],
            sections: vec![],
            debug_info: Some(crate::binary::DebugInfo {
                pdb_path: Some("generate.pdb".to_string()),
                cargo_paths: vec![],
                build_timestamp: None,
            }),
            data: vec![],
        };

        let detector = MemexecDetector::new();
        let findings = detector.analyze_binary(&features, "test.node");
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_no_false_positive_clean_binary() {
        let features = BinaryFeatures {
            format: crate::binary::BinaryFormat::PE,
            imports: vec![],
            exports: vec![],
            strings: vec![
                ExtractedString {
                    content: "Hello World".to_string(),
                    offset: 0,
                    length: 11,
                },
            ],
            sections: vec![],
            debug_info: None,
            data: vec![],
        };

        let detector = MemexecDetector::new();
        let findings = detector.analyze_binary(&features, "test.node");
        assert!(findings.is_empty());
    }
}
