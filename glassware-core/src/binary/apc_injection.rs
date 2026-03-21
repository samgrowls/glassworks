//! G8: APC Injection Detector
//!
//! Detects APC (Asynchronous Procedure Call) injection imports used in GlassWorm binaries.
//!
//! ## Intel Source
//!
//! From PART4.md and PART5.md, c_x64.node uses the following APC injection sequence:
//!
//! 1. `FindResourceW` / `LoadResource` / `LockResource` - Extract embedded DLL
//! 2. `CreateProcessW` - Create suspended browser process (flags: 0x800000a)
//! 3. `VirtualAllocEx` - Allocate memory in target
//! 4. `WriteProcessMemory` - Write DLL path to target
//! 5. `QueueUserAPC` - Queue APC with LoadLibraryW
//! 6. `DebugActiveProcessStop` - Detach debugger, fire APC
//! 7. `CreateNamedPipeA` - IPC for key exfiltration
//!
//! ## Detection Strategy
//!
//! Import table analysis. We look for the combination of:
//! - Process manipulation APIs (CreateProcessW, VirtualAllocEx, WriteProcessMemory)
//! - APC-related APIs (QueueUserAPC)
//! - Debug control APIs (DebugActiveProcessStop)
//!
//! ## Severity
//!
//! **HIGH** - Process injection technique, confirmed GlassWorm TTP.

use crate::detector::Detector;
use crate::finding::{DetectionCategory, Finding, Severity};
use crate::ir::FileIR;

#[cfg(feature = "binary")]
use crate::binary::extract_features;

/// APC injection related native API imports
const APC_IMPORTS: &[&str] = &[
    // Core APC injection APIs
    "QueueUserAPC",
    // Process manipulation
    "VirtualAllocEx",
    "WriteProcessMemory",
    "CreateProcessW",
    // Debug control (for firing APC)
    "DebugActiveProcessStop",
    // Resource extraction (for embedded DLL)
    "FindResourceW",
    "LoadResource",
    "LockResource",
    // IPC for exfiltration
    "CreateNamedPipeA",
    // Native APIs (dynamically resolved in GlassWorm)
    "NtAllocateVirtualMemory",
    "NtProtectVirtualMemory",
    "NtQuerySystemInformation",
];

/// Strong indicators - combination suggests APC injection
const STRONG_APC_INDICATORS: &[&str] = &[
    "QueueUserAPC",
    "VirtualAllocEx",
    "WriteProcessMemory",
    "DebugActiveProcessStop",
];

/// Detector for APC injection signatures
pub struct ApcInjectionDetector;

impl ApcInjectionDetector {
    /// Create a new APC injection detector
    pub fn new() -> Self {
        Self
    }
}

impl Default for ApcInjectionDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "binary")]
impl ApcInjectionDetector {
    /// Analyze binary features for APC injection imports
    fn analyze_binary(&self, features: &crate::binary::BinaryFeatures, path: &str) -> Vec<Finding> {
        let mut found_imports = Vec::new();
        let mut strong_indicators = 0;

        // Search for APC-related imports
        for import in &features.imports {
            let import_name = &import.name;

            // Check if this import matches any APC pattern
            for &apc_import in APC_IMPORTS {
                if import_name.contains(apc_import) {
                    found_imports.push((import_name.as_str(), apc_import));

                    // Check if this is a strong indicator
                    if STRONG_APC_INDICATORS.contains(&apc_import) {
                        strong_indicators += 1;
                    }
                }
            }
        }

        // Also check strings for dynamically resolved APIs
        for string in &features.strings {
            for &apc_import in APC_IMPORTS {
                if string.content.contains(apc_import) {
                    found_imports.push((string.content.as_str(), apc_import));
                    if STRONG_APC_INDICATORS.contains(&apc_import) {
                        strong_indicators += 1;
                    }
                }
            }
        }

        // Deduplicate
        found_imports.sort();
        found_imports.dedup();

        // Determine severity based on indicators
        if strong_indicators >= 3 {
            // HIGH: Multiple strong indicators = confirmed APC injection pattern
            return vec![self.create_finding(
                path,
                1,
                &found_imports.iter().map(|(_, i)| *i).collect::<Vec<_>>(),
                strong_indicators,
                Severity::High,
                "APC injection pattern detected (multiple strong indicators)",
            )];
        } else if strong_indicators >= 1 || found_imports.len() >= 3 {
            // MEDIUM: Some indicators present
            return vec![self.create_finding(
                path,
                1,
                &found_imports.iter().map(|(_, i)| *i).collect::<Vec<_>>(),
                strong_indicators,
                Severity::Medium,
                "Process injection APIs detected",
            )];
        }

        vec![]
    }

    fn create_finding(
        &self,
        path: &str,
        line: usize,
        imports: &[&str],
        strong_count: usize,
        severity: Severity,
        message: &str,
    ) -> Finding {
        Finding::new(
            path,
            line,
            1,
            0,
            '\0',
            DetectionCategory::ApcInjection,
            severity,
            message,
            "GlassWorm uses APC injection to load malicious DLLs into browser processes.",
        )
        .with_cwe_id("CWE-506")
        .with_reference("https://codeberg.org/tip-o-deincognito/glassworm-writeup/raw/branch/main/PART4.md")
        .with_context(&format!(
            "APIs: {:?}, Strong indicators: {}",
            imports, strong_count
        ))
        .with_confidence(match strong_count {
            n if n >= 3 => 0.95,
            n if n >= 1 => 0.75,
            _ => 0.5,
        })
    }
}

impl Detector for ApcInjectionDetector {
    fn name(&self) -> &str {
        "apc_injection"
    }

    fn tier(&self) -> crate::detector::DetectorTier {
        crate::detector::DetectorTier::Tier2Secondary
    }

    fn cost(&self) -> u8 {
        5  // Medium cost - import table analysis
    }

    fn signal_strength(&self) -> u8 {
        9  // Very high signal - process injection is rare in legitimate code
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
    use crate::binary::{BinaryFeatures, ImportEntry};
    use crate::ir::FileIR;
    use std::path::Path;

    #[test]
    fn test_detector_name() {
        let detector = ApcInjectionDetector::new();
        assert_eq!(detector.name(), "apc_injection");
    }

    #[test]
    fn test_detector_tier() {
        let detector = ApcInjectionDetector::new();
        assert_eq!(detector.tier(), crate::detector::DetectorTier::Tier2Secondary);
    }

    #[test]
    fn test_skip_non_node_files() {
        let detector = ApcInjectionDetector::new();
        let content = "const x = 42;";
        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_detect_apc_injection_pattern() {
        let features = BinaryFeatures {
            format: crate::binary::BinaryFormat::PE,
            imports: vec![
                ImportEntry {
                    name: "QueueUserAPC".to_string(),
                    library: Some("kernel32.dll".to_string()),
                },
                ImportEntry {
                    name: "VirtualAllocEx".to_string(),
                    library: Some("kernel32.dll".to_string()),
                },
                ImportEntry {
                    name: "WriteProcessMemory".to_string(),
                    library: Some("kernel32.dll".to_string()),
                },
                ImportEntry {
                    name: "DebugActiveProcessStop".to_string(),
                    library: Some("kernel32.dll".to_string()),
                },
            ],
            exports: vec![],
            strings: vec![],
            sections: vec![],
            debug_info: None,
            data: vec![],
        };

        let detector = ApcInjectionDetector::new();
        let findings = detector.analyze_binary(&features, "test.node");
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::High);
        assert!(findings[0].description.contains("multiple strong indicators"));
    }

    #[test]
    fn test_detect_partial_apc_pattern() {
        let features = BinaryFeatures {
            format: crate::binary::BinaryFormat::PE,
            imports: vec![
                ImportEntry {
                    name: "CreateProcessW".to_string(),
                    library: Some("kernel32.dll".to_string()),
                },
                ImportEntry {
                    name: "VirtualAllocEx".to_string(),
                    library: Some("kernel32.dll".to_string()),
                },
            ],
            exports: vec![],
            strings: vec![],
            sections: vec![],
            debug_info: None,
            data: vec![],
        };

        let detector = ApcInjectionDetector::new();
        let findings = detector.analyze_binary(&features, "test.node");
        // Should be MEDIUM (only 2 imports, 1 strong indicator)
        assert!(!findings.is_empty());
        assert!(findings[0].severity >= Severity::Medium);
    }

    #[test]
    fn test_no_false_positive_clean_binary() {
        let features = BinaryFeatures {
            format: crate::binary::BinaryFormat::PE,
            imports: vec![
                ImportEntry {
                    name: "printf".to_string(),
                    library: Some("msvcrt.dll".to_string()),
                },
                ImportEntry {
                    name: "malloc".to_string(),
                    library: Some("msvcrt.dll".to_string()),
                },
            ],
            exports: vec![],
            strings: vec![],
            sections: vec![],
            debug_info: None,
            data: vec![],
        };

        let detector = ApcInjectionDetector::new();
        let findings = detector.analyze_binary(&features, "test.node");
        assert!(findings.is_empty());
    }
}
