//! ForceMemo Python Attack Detector
//!
//! Detects ForceMemo malware injected into Python repositories.
//! ForceMemo uses GitHub token theft to force-push malicious commits to Python repos.
//!
//! ## Detection Logic
//!
//! This detector scans Python files for:
//! 1. ForceMemo marker variables (lzcdrtfxyqiplpd, idzextbcjbgkdih, etc.)
//! 2. Three-layer obfuscation pattern (Base64 + Zlib + XOR with key 134)
//! 3. Suspicious imports (base64, zlib, os, subprocess together)
//!
//! ## Severity
//!
//! Critical - indicates ForceMemo attack injection

use crate::detector::{Detector, DetectorMetadata};
use crate::finding::{DetectionCategory, Finding, Severity};
use crate::ir::FileIR;
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

/// ForceMemo marker variables (consistent across all samples)
static FORCEMO_MARKERS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"lzcdrtfxyqiplpd").unwrap(),  // Base64 payload blob
        Regex::new(r"idzextbcjbgkdih").unwrap(),  // XOR key constant (134)
        Regex::new(r"aqgqzxkfjzbdnhz").unwrap(),  // Base64 module alias
        Regex::new(r"wogyjaaijwqbpxe").unwrap(),  // Zlib module alias
    ]
});

/// XOR key constant used by ForceMemo
const XOR_KEY: u8 = 134;

/// Detector for ForceMemo Python attacks
pub struct ForceMemoDetector;

impl ForceMemoDetector {
    /// Create a new ForceMemo detector
    pub fn new() -> Self {
        Self
    }
}

impl Default for ForceMemoDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for ForceMemoDetector {
    fn name(&self) -> &str {
        "forcememo_python"
    }

    fn detect(&self, ir: &FileIR) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Only scan Python files
        if !ir.metadata.extension.eq("py") {
            return findings;
        }

        // Count marker matches
        let mut marker_count = 0;
        let mut matched_markers: Vec<&str> = Vec::new();

        for (line_num, line) in ir.content().lines().enumerate() {
            for (marker_idx, marker_pattern) in FORCEMO_MARKERS.iter().enumerate() {
                if marker_pattern.is_match(line) {
                    marker_count += 1;
                    matched_markers.push(match marker_idx {
                        0 => "lzcdrtfxyqiplpd (payload blob)",
                        1 => "idzextbcjbgkdih (XOR key)",
                        2 => "aqgqzxkfjzbdnhz (base64 alias)",
                        3 => "wogyjaaijwqbpxe (zlib alias)",
                        _ => "unknown marker",
                    });

                    findings.push(
                        Finding::new(
                            &ir.metadata.path,
                            line_num + 1,
                            1,
                            0,
                            '\0',
                            DetectionCategory::ForceMemoPython,
                            Severity::Critical,
                            &format!("ForceMemo marker detected: {}", matched_markers.last().unwrap()),
                            "This marker is consistently used in ForceMemo malware injections. Immediate investigation required.",
                        )
                        .with_cwe_id("CWE-506")
                        .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode"),
                    );
                }
            }

            // Check for XOR key constant
            if line.contains(&format!("= {}", XOR_KEY)) || line.contains(&format!("== {}", XOR_KEY)) {
                if line.contains("idzextbcjbgkdih") || line.contains("xor") || line.contains("XOR") {
                    findings.push(
                        Finding::new(
                            &ir.metadata.path,
                            line_num + 1,
                            1,
                            0,
                            '\0',
                            DetectionCategory::ForceMemoPython,
                            Severity::Critical,
                            &format!("ForceMemo XOR key constant detected: {}", XOR_KEY),
                            "ForceMemo uses XOR key 134 for payload obfuscation. This is a strong indicator of ForceMemo infection.",
                        )
                        .with_cwe_id("CWE-506")
                        .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode"),
                    );
                }
            }
        }

        // Check for three-layer obfuscation pattern
        let has_base64 = ir.content().contains("base64");
        let has_zlib = ir.content().contains("zlib");
        let has_xor = ir.content().contains("xor") || ir.content().contains("XOR") || ir.content().contains("^");
        let has_obfuscation = has_base64 && has_zlib && has_xor;

        if has_obfuscation && marker_count > 0 {
            findings.push(
                Finding::new(
                    &ir.metadata.path,
                    1,
                    1,
                    0,
                    '\0',
                    DetectionCategory::ForceMemoPython,
                    Severity::Critical,
                    "Three-layer obfuscation pattern detected (Base64 + Zlib + XOR) with ForceMemo markers",
                    "ForceMemo uses three-layer obfuscation: Base64 encoding → Zlib compression → XOR with key 134. Combined with marker variables, this confirms ForceMemo infection.",
                )
                .with_cwe_id("CWE-506")
                .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode"),
            );
        }

        // Check for suspicious import pattern
        let has_suspicious_imports = (ir.content().contains("import base64") || ir.content().contains("from base64"))
            && (ir.content().contains("import zlib") || ir.content().contains("from zlib"))
            && (ir.content().contains("import os") || ir.content().contains("from os"))
            && (ir.content().contains("import subprocess") || ir.content().contains("from subprocess"));

        if has_suspicious_imports && marker_count > 0 {
            findings.push(
                Finding::new(
                    &ir.metadata.path,
                    1,
                    1,
                    0,
                    '\0',
                    DetectionCategory::ForceMemoPython,
                    Severity::Critical,
                    "Suspicious import pattern with ForceMemo markers detected",
                    "ForceMemo imports base64, zlib, os, and subprocess for payload decoding and execution. Combined with marker variables, this confirms infection.",
                )
                .with_cwe_id("CWE-506")
                .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode"),
            );
        }

        findings
    }

    fn metadata(&self) -> DetectorMetadata {
        DetectorMetadata {
            name: "forcememo_python".to_string(),
            version: "1.0.0".to_string(),
            description: "Detects ForceMemo malware markers and three-layer obfuscation patterns in Python files".to_string().to_string(),
        }
    }
}

impl ForceMemoDetector {
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
    fn test_detect_forcememo_markers() {
        let detector = ForceMemoDetector::new();
        let content = r#"
import base64, zlib, os, subprocess

# ForceMemo payload
lzcdrtfxyqiplpd = "eNpjYBBgAAEQAgw="
idzextbcjbgkdih = 134
aqgqzxkfjzbdnhz = base64
wogyjaaijwqbpxe = zlib

# Decode and execute
payload = zlib.decompress(base64.b64decode(lzcdrtfxyqiplpd))
exec(payload)
"#;

        let findings = detector.scan(Path::new("main.py"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.severity == Severity::Critical));
        assert!(findings.iter().any(|f| f.description.contains("lzcdrtfxyqiplpd")));
    }

    #[test]
    fn test_detect_xor_key() {
        let detector = ForceMemoDetector::new();
        let content = r#"
idzextbcjbgkdih = 134  # XOR key
payload = bytes([b ^ idzextbcjbgkdih for b in encrypted])
"#;

        let findings = detector.scan(Path::new("main.py"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.description.contains("XOR key")));
    }

    #[test]
    fn test_detect_three_layer_obfuscation() {
        let detector = ForceMemoDetector::new();
        let content = r#"
import base64, zlib

lzcdrtfxyqiplpd = "eNpjYBBgAAEQAgw="
data = zlib.decompress(base64.b64decode(lzcdrtfxyqiplpd))
xor_key = 134
decoded = bytes([b ^ xor_key for b in data])
"#;

        let findings = detector.scan(Path::new("main.py"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.description.contains("Three-layer obfuscation")));
    }

    #[test]
    fn test_no_detect_legitimate_python() {
        let detector = ForceMemoDetector::new();
        let content = r#"
import base64
import zlib

# Legitimate base64 encoding
def encode_data(data):
    return base64.b64encode(data.encode()).decode()

# Legitimate zlib compression
def compress_data(data):
    return zlib.compress(data.encode())
"#;

        let findings = detector.scan(Path::new("utils.py"), content, &UnicodeConfig::default());
        assert!(findings.is_empty());
    }

    #[test]
    fn test_no_detect_non_python_files() {
        let detector = ForceMemoDetector::new();
        let content = r#"
lzcdrtfxyqiplpd = "malicious"
idzextbcjbgkdih = 134
"#;

        // Should not detect in non-Python files
        let findings = detector.scan(Path::new("config.js"), content, &UnicodeConfig::default());
        assert!(findings.is_empty());
    }
}
