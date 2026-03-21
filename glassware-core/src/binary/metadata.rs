//! G11: .node Metadata Detector
//!
//! Detects build metadata, PDB paths, and developer forensics in .node files.
//!
//! ## Intel Source
//!
//! From PART5.md section on developer forensics:
//!
//! ### PDB Paths
//! - `N:\work\chrome_current\DumpBrowserSecrets\...` (GlassWorm build path)
//! - `C:\Users\Administrator\.cargo\registry\src\...\neon-1.1.1\...` (Windows)
//! - `/Users/davidioasd/.cargo/registry/src/...` (macOS)
//!
//! ### Build Environment
//! - Windows username: `Administrator`
//! - macOS username: `davidioasd`
//! - Toolchain: VS2022 17.12 Rust compiler (build 35207+)
//!
//! ### Attribution Intelligence
//! - PDB paths reveal build machine structure
//! - Cargo registry paths indicate Rust-based development
//! - Build timestamps show development timeline
//!
//! ## Detection Strategy
//!
//! Extract PDB paths from PE debug info and search for distinctive strings:
//! - Known GlassWorm build paths
//! - Developer usernames
//! - Cargo registry paths with neon framework
//!
//! ## Severity
//!
//! - **INFO**: General metadata (attribution intelligence)
//! - **MEDIUM**: Known GlassWorm build paths or usernames
//! - **HIGH**: Combination of multiple GlassWorm indicators

use crate::detector::Detector;
use crate::finding::{DetectionCategory, Finding, Severity};
use crate::ir::FileIR;

#[cfg(feature = "binary")]
use crate::binary::extract_features;

/// Known GlassWorm build paths
const GLASSWORM_PATHS: &[&str] = &[
    "N:\\work\\chrome_current",
    "DumpBrowserSecrets",
    "ext_sideloader",
    "vscode_env_history",
];

/// Developer usernames from GlassWorm binaries
const GLASSWORM_USERNAMES: &[&str] = &["Administrator", "davidioasd"];

/// Suspicious PDB path patterns
const SUSPICIOUS_PDB_PATTERNS: &[&str] = &[
    "jucku",
    "myextension",
    "DllExtractChromiumSecrets",
    "generate.pdb",
];

/// Detector for .node metadata
pub struct MetadataDetector;

impl MetadataDetector {
    /// Create a new metadata detector
    pub fn new() -> Self {
        Self
    }
}

impl Default for MetadataDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "binary")]
impl MetadataDetector {
    /// Analyze binary features for metadata indicators
    fn analyze_binary(&self, features: &crate::binary::BinaryFeatures, path: &str) -> Vec<Finding> {
        let mut found_indicators = Vec::new();
        let mut severity = Severity::Info;

        // Check PDB paths from debug info
        if let Some(debug) = &features.debug_info {
            if let Some(pdb_path) = &debug.pdb_path {
                // Check for GlassWorm build paths
                for &glassworm_path in GLASSWORM_PATHS {
                    if pdb_path.contains(glassworm_path) {
                        found_indicators.push(("pdb_path", glassworm_path));
                        severity = Severity::High;
                    }
                }

                // Check for suspicious patterns
                for &pattern in SUSPICIOUS_PDB_PATTERNS {
                    if pdb_path.contains(pattern) {
                        found_indicators.push(("pdb_path", pattern));
                        if severity < Severity::Medium {
                            severity = Severity::Medium;
                        }
                    }
                }

                // Check for GlassWorm usernames
                for &username in GLASSWORM_USERNAMES {
                    if pdb_path.contains(username) {
                        found_indicators.push(("username", username));
                        severity = Severity::High;
                    }
                }
            }

            // Check Cargo registry paths
            for cargo_path in &debug.cargo_paths {
                if cargo_path.contains("neon-") {
                    found_indicators.push(("cargo", "neon framework"));
                    if severity < Severity::Info {
                        severity = Severity::Info;
                    }
                }
            }
        }

        // Check strings for additional metadata
        for string in &features.strings {
            let content = &string.content;

            // Check for GlassWorm paths
            for &glassworm_path in GLASSWORM_PATHS {
                if content.contains(glassworm_path) {
                    found_indicators.push(("string", glassworm_path));
                    severity = Severity::High;
                }
            }

            // Check for usernames
            for &username in GLASSWORM_USERNAMES {
                if content.contains(username) {
                    found_indicators.push(("username", username));
                    if severity < Severity::Medium {
                        severity = Severity::Medium;
                    }
                }
            }

            // Check for suspicious patterns
            for &pattern in SUSPICIOUS_PDB_PATTERNS {
                if content.contains(pattern) {
                    found_indicators.push(("string", pattern));
                    if severity < Severity::Medium {
                        severity = Severity::Medium;
                    }
                }
            }
        }

        // Deduplicate
        found_indicators.sort();
        found_indicators.dedup();

        if !found_indicators.is_empty() {
            return vec![self.create_finding(
                path,
                1,
                &found_indicators.iter().map(|(_, i)| *i).collect::<Vec<_>>(),
                severity,
                "Build metadata detected",
            )];
        }

        vec![]
    }

    fn create_finding(
        &self,
        path: &str,
        line: usize,
        indicators: &[&str],
        severity: Severity,
        message: &str,
    ) -> Finding {
        Finding::new(
            path,
            line,
            1,
            0,
            '\0',
            DetectionCategory::Unknown,  // Metadata is attribution, not direct threat
            severity,
            message,
            "Review build metadata for attribution intelligence. PDB paths and usernames can identify threat actors.",
        )
        .with_cwe_id("CWE-506")
        .with_reference("https://codeberg.org/tip-o-deincognito/glassworm-writeup/raw/branch/main/PART5.md")
        .with_context(&format!("Indicators: {:?}", indicators))
        .with_confidence(match severity {
            Severity::High => 0.9,
            Severity::Medium => 0.7,
            _ => 0.5,
        })
    }
}

impl Detector for MetadataDetector {
    fn name(&self) -> &str {
        "node_metadata"
    }

    fn tier(&self) -> crate::detector::DetectorTier {
        crate::detector::DetectorTier::Tier1Primary
    }

    fn cost(&self) -> u8 {
        3  // Low cost - simple string matching
    }

    fn signal_strength(&self) -> u8 {
        5  // Medium signal - attribution intelligence
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
    use crate::binary::{BinaryFeatures, DebugInfo, ExtractedString};
    use crate::ir::FileIR;
    use std::path::Path;

    #[test]
    fn test_detector_name() {
        let detector = MetadataDetector::new();
        assert_eq!(detector.name(), "node_metadata");
    }

    #[test]
    fn test_detector_tier() {
        let detector = MetadataDetector::new();
        assert_eq!(detector.tier(), crate::detector::DetectorTier::Tier1Primary);
    }

    #[test]
    fn test_skip_non_node_files() {
        let detector = MetadataDetector::new();
        let content = "const x = 42;";
        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_detect_glassworm_build_path() {
        let features = BinaryFeatures {
            format: crate::binary::BinaryFormat::PE,
            imports: vec![],
            exports: vec![],
            strings: vec![],
            sections: vec![],
            debug_info: Some(DebugInfo {
                pdb_path: Some("N:\\work\\chrome_current\\DumpBrowserSecrets\\build\\Release\\dump_browser_secrets.pdb".to_string()),
                cargo_paths: vec![],
                build_timestamp: None,
            }),
            data: vec![],
        };

        let detector = MetadataDetector::new();
        let findings = detector.analyze_binary(&features, "test.node");
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::High);
    }

    #[test]
    fn test_detect_glassworm_username() {
        let features = BinaryFeatures {
            format: crate::binary::BinaryFormat::PE,
            imports: vec![],
            exports: vec![],
            strings: vec![ExtractedString {
                content: "C:\\Users\\Administrator\\.cargo\\registry".to_string(),
                offset: 0x1000,
                length: 40,
            }],
            sections: vec![],
            debug_info: None,
            data: vec![],
        };

        let detector = MetadataDetector::new();
        let findings = detector.analyze_binary(&features, "test.node");
        assert!(!findings.is_empty());
        assert!(findings[0].severity >= Severity::Medium);
    }

    #[test]
    fn test_detect_suspicious_pdb() {
        let features = BinaryFeatures {
            format: crate::binary::BinaryFormat::PE,
            imports: vec![],
            exports: vec![],
            strings: vec![],
            sections: vec![],
            debug_info: Some(DebugInfo {
                pdb_path: Some("generate.pdb".to_string()),
                cargo_paths: vec![],
                build_timestamp: None,
            }),
            data: vec![],
        };

        let detector = MetadataDetector::new();
        let findings = detector.analyze_binary(&features, "test.node");
        assert!(!findings.is_empty());
        assert!(findings[0].severity >= Severity::Medium);
    }

    #[test]
    fn test_no_false_positive_clean_pdb() {
        let features = BinaryFeatures {
            format: crate::binary::BinaryFormat::PE,
            imports: vec![],
            exports: vec![],
            strings: vec![],
            sections: vec![],
            debug_info: Some(DebugInfo {
                pdb_path: Some("C:\\Users\\Developer\\project\\target\\debug\\mylib.pdb".to_string()),
                cargo_paths: vec![],
                build_timestamp: None,
            }),
            data: vec![],
        };

        let detector = MetadataDetector::new();
        let findings = detector.analyze_binary(&features, "test.node");
        // Clean PDB paths should not trigger
        assert!(findings.is_empty());
    }
}
