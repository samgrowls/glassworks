//! Encrypted Payload Detector (GW005)
//!
//! Detects files containing BOTH a high-entropy encoded blob AND dynamic code
//! execution — the signature pattern of an encrypted loader.
//!
//! ## Detection Logic
//!
//! This detector emits a finding ONLY when BOTH conditions are present in the same file:
//!
//! 1. **High-entropy blob**: String/template literals longer than 64 characters with
//!    Shannon entropy > 4.5 bits/byte, or continuous hex/base64 blocks.
//! 2. **Dynamic execution**: `eval(`, `new Function(`, `vm.runInNewContext`, etc.
//!
//! ## Severity
//!
//! High — indicates potential encrypted payload loader.

use crate::config::UnicodeConfig;
use crate::detector::{Detector, DetectorMetadata, ScanContext};
use crate::finding::{DetectionCategory, Finding, Severity};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

/// Minimum length for high-entropy blob detection
const MIN_BLOB_LENGTH: usize = 64;

/// Entropy threshold for detecting encrypted/encoded content
const ENTROPY_THRESHOLD: f64 = 4.5;

/// Lazy-compiled regex patterns for performance
static HEX_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"[0-9a-fA-F]{64,}").unwrap());
static BASE64_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"[A-Za-z0-9+/=]{64,}").unwrap());
static STRING_LITERAL_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#""([^"]{64,})"|'([^']{64,})'"#).unwrap());
static TEMPLATE_LITERAL_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"`([^`]{64,})`").unwrap());

// Dynamic execution patterns
static EVAL_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"\beval\s*\(").unwrap());
static FUNCTION_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\bnew\s+Function\s*\(").unwrap());
static VM_RUN_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\bvm\.runIn(NewContext|ThisContext)\s*\(").unwrap());
static EXEC_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b(exec|execSync)\s*\(").unwrap());
static CHILD_PROCESS_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\bchild_process\s*\+").unwrap());

// Decryption patterns - required for decrypt→exec flow
static DECRYPT_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(createDecipheriv|createDecipher|\.decrypt|\batob\s*\(|Buffer\.from\s*\([^)]*\)\s*\.toString)").unwrap()
});

/// Detector for encrypted payload patterns (GW005)
pub struct EncryptedPayloadDetector;

impl EncryptedPayloadDetector {
    /// Create a new encrypted payload detector
    pub fn new() -> Self {
        Self
    }

    /// Backward compatibility method for tests
    pub fn scan(&self, path: &Path, content: &str, _config: &UnicodeConfig) -> Vec<Finding> {
        let ctx = ScanContext::new(path.to_string_lossy().to_string(), content.to_string(), UnicodeConfig::default());
        self.detect(&ctx)
    }
}

impl Default for EncryptedPayloadDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for EncryptedPayloadDetector {
    fn name(&self) -> &str {
        "encrypted_payload"
    }

    fn detect(&self, ctx: &ScanContext) -> Vec<Finding> {
        let mut findings = Vec::new();
        let path = Path::new(&ctx.file_path);

        // Skip bundled/minified files (high FP rate)
        let path_str = path.to_string_lossy().to_lowercase();
        let is_bundled = path_str.contains("/dist/")
            || path_str.contains("/build/")
            || path_str.contains("/bin/")
            || path_str.ends_with(".mjs")
            || path_str.ends_with(".cjs")
            || path_str.contains(".min.")
            || path_str.contains(".bundle.")
            || path_str.contains(".umd.");

        if is_bundled {
            return findings;
        }

        // Check for high-entropy blobs
        let has_high_entropy_blob = self.detect_high_entropy_blob(&ctx.content);

        // Check for decrypt→exec flow (not just any exec)
        let has_decrypt_exec_flow = self.detect_decrypt_to_exec_flow(&ctx.content);

        // Only emit finding if BOTH conditions are present:
        // 1. High-entropy blob
        // 2. Decryption pattern followed by dynamic execution
        if has_high_entropy_blob && has_decrypt_exec_flow {
            // Find the line with the high-entropy blob for the finding location
            let blob_line = self.find_high_entropy_blob_line(&ctx.content).unwrap_or(1);

            let finding = Finding::new(
                &path.to_string_lossy(),
                blob_line,
                1,
                0,
                '\0',
                DetectionCategory::EncryptedPayload,
                Severity::High,
                "High-entropy blob combined with decrypt→exec flow — potential encrypted payload loader",
                "Review this file for encrypted payload patterns. The combination of high-entropy \
                 encoded data with decryption followed by dynamic code execution is characteristic \
                 of encrypted loaders used in supply chain attacks. Decode the blob to understand \
                 the hidden payload.",
            )
            .with_cwe_id("CWE-506")
            .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode");

            findings.push(finding);
        }

        findings
    }

    fn metadata(&self) -> DetectorMetadata {
        DetectorMetadata {
            name: "encrypted_payload".to_string(),
            version: "1.0.0".to_string(),
            description: "Detects encrypted payload patterns with high-entropy blobs and decrypt-to-exec flows".to_string().to_string(),
        }
    }
}

impl EncryptedPayloadDetector {
    /// Check if content contains a high-entropy blob
    fn detect_high_entropy_blob(&self, content: &str) -> bool {
        // Check for hex patterns: continuous hex chars >= 64
        for m in HEX_PATTERN.find_iter(content) {
            let entropy = self.calculate_entropy(m.as_str().as_bytes());
            if entropy > ENTROPY_THRESHOLD {
                return true;
            }
        }

        // Check for base64 patterns: continuous base64 chars >= 64
        for m in BASE64_PATTERN.find_iter(content) {
            let entropy = self.calculate_entropy(m.as_str().as_bytes());
            if entropy > ENTROPY_THRESHOLD {
                return true;
            }
        }

        // Check string literals for high entropy
        for line in content.lines() {
            if let Some(literal) = self.extract_string_literal(line) {
                if literal.len() >= MIN_BLOB_LENGTH {
                    let entropy = self.calculate_entropy(literal.as_bytes());
                    if entropy > ENTROPY_THRESHOLD {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Find the line number containing a high-entropy blob
    fn find_high_entropy_blob_line(&self, content: &str) -> Option<usize> {
        // Check for hex patterns
        for (line_num, line) in content.lines().enumerate() {
            if HEX_PATTERN.is_match(line) {
                return Some(line_num + 1);
            }
        }

        // Check for base64 patterns
        for (line_num, line) in content.lines().enumerate() {
            if BASE64_PATTERN.is_match(line) {
                return Some(line_num + 1);
            }
        }

        // Check string literals
        for (line_num, line) in content.lines().enumerate() {
            if let Some(literal) = self.extract_string_literal(line) {
                if literal.len() >= MIN_BLOB_LENGTH {
                    let entropy = self.calculate_entropy(literal.as_bytes());
                    if entropy > ENTROPY_THRESHOLD {
                        return Some(line_num + 1);
                    }
                }
            }
        }

        None
    }

    /// Detect decrypt→exec flow: decryption API usage followed by dynamic execution
    fn detect_decrypt_to_exec_flow(&self, content: &str) -> bool {
        // Check for decryption patterns
        let has_decrypt = DECRYPT_PATTERN.is_match(content);

        // Check for dynamic execution patterns
        let has_exec = EVAL_PATTERN.is_match(content)
            || FUNCTION_PATTERN.is_match(content)
            || VM_RUN_PATTERN.is_match(content)
            || EXEC_PATTERN.is_match(content)
            || CHILD_PROCESS_PATTERN.is_match(content);

        // Require BOTH decrypt AND exec
        has_decrypt && has_exec
    }

    /// Extract a string literal from a line of code
    fn extract_string_literal(&self, line: &str) -> Option<String> {
        // Match single-quoted, double-quoted, or template literals
        if let Some(caps) = STRING_LITERAL_PATTERN.captures(line) {
            if let Some(m) = caps.get(1).or_else(|| caps.get(2)) {
                return Some(m.as_str().to_string());
            }
        }

        if let Some(caps) = TEMPLATE_LITERAL_PATTERN.captures(line) {
            if let Some(m) = caps.get(1) {
                return Some(m.as_str().to_string());
            }
        }

        None
    }

    /// Calculate Shannon entropy of byte data
    fn calculate_entropy(&self, data: &[u8]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let mut counts = [0u64; 256];
        for &byte in data {
            counts[byte as usize] += 1;
        }

        let len = data.len() as f64;
        let mut entropy = 0.0;

        for &count in &counts {
            if count > 0 {
                let p = count as f64 / len;
                entropy -= p * p.log2();
            }
        }

        entropy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_base64_with_eval() {
        let detector = EncryptedPayloadDetector::new();
        let content = r#"
            const payload = "SGVsbG8gV29ybGQhIFRoaXMgaXMgYSB0ZXN0IHN0cmluZyB0aGF0IGlzIGxvbmcgZW5vdWdoIHRvIHRyaWdnZXIgZGV0ZWN0aW9uIGJhc2U2NCBlbmNvZGVkIGRhdGE=";
            eval(atob(payload));
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert_eq!(findings[0].category, DetectionCategory::EncryptedPayload);
        assert_eq!(findings[0].severity, Severity::High);
    }

    #[test]
    fn test_detect_hex_with_function() {
        let detector = EncryptedPayloadDetector::new();
        // High-entropy base64 string (encrypted-looking data) with decrypt→exec flow
        let content = r#"
            const data = "kJfXyZ2BvMnR0cHV3eIGIqaqrrK2ur7CxsrO0tba3uLm6u7y9vr/AwcLDxMXGx8jJysvMzc7P0NHS09TV1tfY2drb3N3e3+Dh4uPk5ebn6Onq6+zt7u/w8fLz9PX29/j5+vv8/f7/";
            new Function(atob(data))();
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert_eq!(findings[0].category, DetectionCategory::EncryptedPayload);
    }

    #[test]
    fn test_no_detect_eval_only() {
        let detector = EncryptedPayloadDetector::new();
        let content = r#"
            const x = "hello";
            eval(x);
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(findings.is_empty());
    }

    #[test]
    fn test_no_detect_blob_only() {
        let detector = EncryptedPayloadDetector::new();
        let content = r#"
            const data = "SGVsbG8gV29ybGQhIFRoaXMgaXMgYSB0ZXN0IHN0cmluZyB0aGF0IGlzIGxvbmcgZW5vdWdoIHRvIHRyaWdnZXIgZGV0ZWN0aW9uIGJhc2U2NCBlbmNvZGVkIGRhdGE=";
            console.log(data);
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(findings.is_empty());
    }

    #[test]
    fn test_no_detect_low_entropy() {
        let detector = EncryptedPayloadDetector::new();
        let content = r#"
            const repeated = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
            eval(repeated);
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(findings.is_empty());
    }

    #[test]
    fn test_no_detect_normal_code() {
        let detector = EncryptedPayloadDetector::new();
        let content = r#"
            const message = "Hello, World!";
            console.log(message);
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(findings.is_empty());
    }

    #[test]
    fn test_detect_vm_execution() {
        let detector = EncryptedPayloadDetector::new();
        let content = r#"
            const code = "dGhpcyBpcyBhIGJhc2U2NCBlbmNvZGVkIHN0cmluZyB0aGF0IGlzIGxvbmcgZW5vdWdoIHRvIHRyaWdnZXIgZGV0ZWN0aW9uIGFuZCBzaG91bGQgYmUgZmxhZ2dlZA==";
            vm.runInNewContext(atob(code));
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_detect_exec_sync() {
        let detector = EncryptedPayloadDetector::new();
        let content = r#"
            const cmd = "Y21kLmV4ZSAvYyBlY2hvIGhlbGxvIHdvcmxkIHRoaXMgaXMgYSBsb25nIGJhc2U2NCBlbmNvZGVkIHN0cmluZyBmb3IgdGVzdGluZyBwdXJwb3Nlcw==";
            execSync(atob(cmd));
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_detect_eval_atob_pattern() {
        let detector = EncryptedPayloadDetector::new();
        let content = r#"
            const data = "VGhpcyBpcyBhIGJhc2U2NCBlbmNvZGVkIHBheWxvYWQgdGhhdCBpcyBsb25nIGVub3VnaCB0byB0cmlnZ2VyIGRldGVjdGlvbi4=";
            eval(atob(data));
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert_eq!(findings[0].category, DetectionCategory::EncryptedPayload);
    }

    #[test]
    fn test_detect_buffer_from_tostring_eval() {
        let detector = EncryptedPayloadDetector::new();
        let content = r#"
            const payload = "VGhpcyBpcyBhIGJhc2U2NCBlbmNvZGVkIHBheWxvYWQgdGhhdCBpcyBsb25nIGVub3VnaCB0byB0cmlnZ2VyIGRldGVjdGlvbi4=";
            eval(Buffer.from(payload, 'base64').toString('utf-8'));
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_no_detect_low_entropy_blob_with_exec() {
        let detector = EncryptedPayloadDetector::new();
        let content = r#"
            const repeated = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
            eval(repeated);
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        // Should NOT detect - low entropy blob
        assert!(findings.is_empty());
    }

    #[test]
    fn test_no_detect_short_blob_with_exec() {
        let detector = EncryptedPayloadDetector::new();
        let content = r#"
            const short = "SGVsbG8gV29ybGQh";
            eval(atob(short));
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        // Should NOT detect - blob too short (< 64 chars)
        assert!(findings.is_empty());
    }

    #[test]
    fn test_no_detect_exec_without_decrypt() {
        let detector = EncryptedPayloadDetector::new();
        // Legitimate build script pattern: high-entropy URLs + execSync but NO decryption
        let content = r#"
            const https = require('https');
            const { execSync } = require('child_process');
            const IOS_URL = 'https://github.com/example/flir-sdk-binaries/releases/download/v1.0.1/ios.zip';
            const ANDROID_URL = 'https://github.com/example/flir-sdk-binaries/releases/download/v1.0.1/android.zip';
            execSync(`unzip -o "${zipPath}" -d "${TMP_DIR}"`);
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        // Should NOT detect - no decryption pattern (just download + exec)
        assert!(findings.is_empty());
    }

    #[test]
    fn test_entropy_calculation_uniform() {
        let detector = EncryptedPayloadDetector::new();
        let entropy = detector.calculate_entropy(&[0x41, 0x41, 0x41, 0x41]);
        assert!((entropy - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_entropy_calculation_high() {
        let detector = EncryptedPayloadDetector::new();
        let data: Vec<u8> = (0..=255).collect();
        let entropy = detector.calculate_entropy(&data);
        assert!(entropy > 7.9);
    }
}
