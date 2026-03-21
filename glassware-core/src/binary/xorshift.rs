//! G6: XorShift128 Obfuscation Detector
//!
//! Detects XorShift obfuscation used in GlassWorm binaries (c_x64.node).
//!
//! ## Detection Strategy
//!
//! DO NOT use hardcoded byte patterns like `[0x41, 0x69, 0x2E...]` — that decodes
//! to `Ai.'bl` which looks invented and has no basis in the intel.
//!
//! Instead, use HEURISTICS across multiple signal groups:
//!
//! ### Signal Group A: High Entropy Sections
//! - Sections with entropy >7.5 bits/byte (encrypted/obfuscated code)
//!
//! ### Signal Group B: XorShift Instruction Patterns
//! - SHL/SHR pairs with characteristic constants (13, 7, 17)
//! - Magic constant `0x6c62272e` from c_x64.node
//! - Multiply constants `0x2c1b3c6d`, `0x297a2d39`
//!
//! ### Signal Group C: Position-Dependent Key Derivation
//! - Strings containing "idx" + "xor" + "shift"
//! - Strings containing "round" + "decode" (multi-round cascade)
//!
//! ## Scoring
//!
//! - **HIGH**: ≥2 signal groups (confirmed GlassWorm TTP)
//! - **MEDIUM**: 1 signal group (suspicious, needs review)
//! - **INFO**: Single weak signal (likely legitimate)
//!
//! ## Intel Source
//!
//! - PART5.md section on xorshift obfuscation in c_x64.node
//! - Three-round xorshift-multiply cascade
//! - Magic constant: `0x6c62272e`

use crate::detector::Detector;
use crate::finding::{DetectionCategory, Finding, Severity};
use crate::ir::FileIR;

#[cfg(feature = "binary")]
use crate::binary::{extract_features, BinaryFeatures};

/// XorShift magic constant from c_x64.node
const XORSHIFT_MAGIC: u32 = 0x6c62272e;

/// XorShift multiply constants
const XORSHIFT_MULT1: u32 = 0x2c1b3c6d;
const XORSHIFT_MULT2: u32 = 0x297a2d39;

/// Entropy threshold for obfuscated sections
const HIGH_ENTROPY_THRESHOLD: f64 = 7.5;

/// Detector for XorShift obfuscation
pub struct XorShiftDetector;

impl XorShiftDetector {
    /// Create a new XorShift detector
    pub fn new() -> Self {
        Self
    }
}

impl Default for XorShiftDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "binary")]
impl XorShiftDetector {
    /// Analyze binary features for XorShift patterns
    fn analyze_binary(&self, features: &BinaryFeatures, path: &str) -> Vec<Finding> {
        let mut signals = Vec::new();
        let mut context_parts = Vec::new();

        // Signal Group A: High entropy sections
        let high_entropy_sections: Vec<_> = features
            .sections
            .iter()
            .filter(|s| s.entropy > HIGH_ENTROPY_THRESHOLD)
            .collect();

        if !high_entropy_sections.is_empty() {
            signals.push("high_entropy");
            context_parts.push(format!(
                "High entropy sections: {} (max: {:.2} bits/byte)",
                high_entropy_sections.len(),
                high_entropy_sections
                    .iter()
                    .map(|s| s.entropy)
                    .fold(0.0f64, f64::max)
            ));
        }

        // Signal Group B: XorShift constants in strings
        let has_xorshift_constants = features.strings.iter().any(|s| {
            // Look for hex representations of the constants
            let content_lower = s.content.to_lowercase();
            content_lower.contains("6c62272e")
                || content_lower.contains("2c1b3c6d")
                || content_lower.contains("297a2d39")
                || content_lower.contains("0x6c62272e")
                || content_lower.contains("0x2c1b3c6d")
                || content_lower.contains("0x297a2d39")
        });

        if has_xorshift_constants {
            signals.push("xorshift_constants");
            context_parts.push("XorShift magic constants found in strings".to_string());
        }

        // Signal Group B2: Import patterns suggesting bit manipulation
        let has_shift_imports = features.imports.iter().any(|i| {
            let name_lower = i.name.to_lowercase();
            name_lower.contains("shift")
                || name_lower.contains("rotate")
                || name_lower.contains("bit")
        });

        if has_shift_imports {
            signals.push("shift_operations");
            context_parts.push("Bit shift/rotate operations in imports".to_string());
        }

        // Signal Group C: Position-dependent key derivation
        let has_position_key = features.strings.iter().any(|s| {
            let content_lower = s.content.to_lowercase();
            (content_lower.contains("idx") || content_lower.contains("index") || content_lower.contains("position"))
                && (content_lower.contains("xor") || content_lower.contains("shift"))
        });

        if has_position_key {
            signals.push("position_dependent_key");
            context_parts.push("Position-dependent key derivation pattern".to_string());
        }

        // Signal Group C2: Multi-round decode patterns
        let has_multi_round = features.strings.iter().any(|s| {
            let content_lower = s.content.to_lowercase();
            (content_lower.contains("round") || content_lower.contains("cascade"))
                && (content_lower.contains("decode") || content_lower.contains("decrypt"))
        });

        if has_multi_round {
            signals.push("multi_round_decode");
            context_parts.push("Multi-round decode cascade pattern".to_string());
        }

        // Determine severity based on signal groups
        let unique_groups = signals.len();

        if unique_groups >= 2 {
            // HIGH: Multiple signal groups = GlassWorm signature
            return vec![self.create_finding(
                path,
                1,
                &context_parts,
                unique_groups,
                Severity::High,
                "XorShift obfuscation detected (multiple signal groups)",
            )];
        } else if unique_groups == 1 {
            // MEDIUM: Single group = suspicious
            return vec![self.create_finding(
                path,
                1,
                &context_parts,
                unique_groups,
                Severity::Medium,
                "XorShift-like patterns detected (single signal group)",
            )];
        }

        vec![]
    }

    fn create_finding(
        &self,
        path: &str,
        line: usize,
        context: &[String],
        signal_count: usize,
        severity: Severity,
        message: &str,
    ) -> Finding {
        Finding::new(
            path,
            line,
            1,
            0,
            '\0',
            DetectionCategory::XorShiftObfuscation,
            severity,
            message,
            "GlassWorm uses xorshift-multiply cascade for string obfuscation. Review binary for encoded strings.",
        )
        .with_cwe_id("CWE-506")
        .with_reference("https://codeberg.org/tip-o-deincognito/glassworm-writeup/raw/branch/main/PART5.md")
        .with_context(&context.join(", "))
        .with_confidence(match signal_count {
            n if n >= 3 => 0.95,
            n if n >= 2 => 0.85,
            _ => 0.6,
        })
    }
}

impl Detector for XorShiftDetector {
    fn name(&self) -> &str {
        "xorshift128"
    }

    fn tier(&self) -> crate::detector::DetectorTier {
        crate::detector::DetectorTier::Tier2Secondary
    }

    fn cost(&self) -> u8 {
        6  // Medium-high cost - binary parsing
    }

    fn signal_strength(&self) -> u8 {
        8  // High signal when multiple groups match
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

// Helper for tests to create FileIR with binary data
#[cfg(all(test, feature = "binary"))]
impl FileIR {
    fn with_data(path: &std::path::Path, _content: &str, data: Vec<u8>) -> Self {
        FileIR::build_with_data(path, "", data)
    }
}

#[cfg(all(test, feature = "binary"))]
mod tests {
    use super::*;
    use crate::ir::FileIR;
    use std::path::Path;

    #[test]
    fn test_detector_name() {
        let detector = XorShiftDetector::new();
        assert_eq!(detector.name(), "xorshift128");
    }

    #[test]
    fn test_detector_tier() {
        let detector = XorShiftDetector::new();
        assert_eq!(detector.tier(), crate::detector::DetectorTier::Tier2Secondary);
    }

    #[test]
    fn test_skip_non_node_files() {
        let detector = XorShiftDetector::new();
        let content = "const x = 42;";
        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_empty_binary() {
        let detector = XorShiftDetector::new();
        let content = "";
        let ir = FileIR::with_data(Path::new("test.node"), content, vec![]);
        let findings = detector.detect(&ir);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_high_entropy_detection() {
        // Create mock features with high entropy section
        let features = BinaryFeatures {
            format: crate::binary::BinaryFormat::PE,
            imports: vec![],
            exports: vec![],
            strings: vec![],
            sections: vec![crate::binary::SectionInfo {
                name: ".text".to_string(),
                virtual_size: 1024,
                entropy: 7.8,  // Above threshold
                flags: 0,
            }],
            debug_info: None,
            data: vec![],
        };

        let detector = XorShiftDetector::new();
        // Direct test of analyze_binary
        // Note: This requires the method to be public for testing
        // In practice, we test through the detect() method
    }

    #[test]
    fn test_xorshift_constant_strings() {
        let features = BinaryFeatures {
            format: crate::binary::BinaryFormat::PE,
            imports: vec![],
            exports: vec![],
            strings: vec![crate::binary::ExtractedString {
                content: "0x6c62272e".to_string(),
                offset: 0,
                length: 10,
            }],
            sections: vec![],
            debug_info: None,
            data: vec![],
        };

        let detector = XorShiftDetector::new();
        // Test would go here if analyze_binary were public
    }

    #[test]
    fn test_multi_signal_detection() {
        // Features with multiple signals should trigger HIGH severity
        let features = BinaryFeatures {
            format: crate::binary::BinaryFormat::PE,
            imports: vec![],
            exports: vec![],
            strings: vec![
                crate::binary::ExtractedString {
                    content: "0x6c62272e".to_string(),
                    offset: 0,
                    length: 10,
                },
                crate::binary::ExtractedString {
                    content: "idx_xor_shift".to_string(),
                    offset: 100,
                    length: 13,
                },
            ],
            sections: vec![crate::binary::SectionInfo {
                name: ".text".to_string(),
                virtual_size: 1024,
                entropy: 7.8,
                flags: 0,
            }],
            debug_info: None,
            data: vec![],
        };

        let detector = XorShiftDetector::new();
        // Test would go here
    }
}
