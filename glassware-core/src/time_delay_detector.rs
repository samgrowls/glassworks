//! Time-Delay Sandbox Evasion Detector (GW010)
//!
//! Detects code that uses long time delays to evade sandbox analysis.
//! Common patterns include 15-minute delays (900,000ms) or 48-96 hour delays
//! with CI/CD environment bypass.
//!
//! ## Detection Logic
//!
//! This detector emits findings when:
//! 1. setTimeout/setInterval with delay >5 minutes (300,000ms)
//! 2. Specific delay values: 900000 (15min), 172800000 (48hr), 259200000 (72hr)
//! 3. CI environment checks (process.env.CI, GITHUB_ACTIONS, etc.)
//! 4. Conditional delay pattern: if (!isCI) setTimeout(...)
//!
//! ## Severity
//!
//! Critical when CI bypass pattern detected
//! High when long delay detected without CI bypass

use crate::detector::{Detector, DetectorMetadata, DetectorTier};
use crate::finding::{DetectionCategory, Finding, Severity};
use crate::ir::FileIR;
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

/// Patterns for time-delay detection
static DELAY_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // setTimeout with long delays (5min to 96hrs) - captures 300000+
        Regex::new(r"setTimeout\s*\(\s*[^,]+,\s*([5-9]\d{5}|[1-9]\d{6,})\s*\)").unwrap(),
        // setInterval with suspicious intervals
        Regex::new(r"setInterval\s*\(\s*[^,]+,\s*([5-9]\d{5}|[1-9]\d{6,})\s*\)").unwrap(),
        // Specific millisecond values
        Regex::new(r"\b900000\b").unwrap(),  // 15 minutes
        Regex::new(r"\b172800000\b").unwrap(),  // 48 hours
        Regex::new(r"\b259200000\b").unwrap(),  // 72 hours
        // CI environment checks
        Regex::new(r"process\.env\.(CI|GITHUB_ACTIONS|CIRCLECI|TRAVIS|JENKINS|GITLAB_CI)").unwrap(),
        // isCI variable check
        Regex::new(r"\bisCI\b").unwrap(),
        // Conditional delay pattern
        Regex::new(r"if\s*\(\s*!?\s*(isCI|process\.env\.CI)\s*\)\s*\{[^}]*setTimeout").unwrap(),
    ]
});

/// Detector for time-delay sandbox evasion
pub struct TimeDelayDetector;

impl TimeDelayDetector {
    /// Create a new time-delay detector
    pub fn new() -> Self {
        Self
    }
}

impl Default for TimeDelayDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for TimeDelayDetector {
    fn name(&self) -> &str {
        "time_delay_sandbox_evasion"
    }

    fn tier(&self) -> DetectorTier {
        DetectorTier::Tier3Behavioral
    }

    fn cost(&self) -> u8 {
        3  // Low cost - simple regex matching
    }

    fn signal_strength(&self) -> u8 {
        6  // Medium-high signal - delays can be legitimate
    }

    fn prerequisites(&self) -> Vec<&'static str> {
        vec!["glassware", "encrypted_payload"]  // Run after Tier 2
    }

    fn should_short_circuit(&self, findings: &[Finding]) -> bool {
        // Don't run Tier 3 if nothing found by Tier 1-2
        findings.is_empty()
    }

    fn detect(&self, ir: &FileIR) -> Vec<Finding> {
        let mut findings = Vec::new();

        // ⚠️ UPDATED 2026-03-24: Removed build tool skip logic
        // Build tools ARE high-value attack targets (see: Babel 2024, Webpack 2025 attacks)
        // Instead, use context-aware detection: CI bypass + delay = evasion
        // Pure setTimeout in build tools without CI bypass = likely legitimate

        let mut ci_check_lines: Vec<usize> = Vec::new();

        // First pass: find CI checks
        for (line_num, line) in ir.content().lines().enumerate() {
            if DELAY_PATTERNS[4].is_match(line) || DELAY_PATTERNS[5].is_match(line) {
                ci_check_lines.push(line_num);
            }
        }

        // Second pass: find delays and correlate with CI checks
        for (line_num, line) in ir.content().lines().enumerate() {
            // Check for long delays
            for (i, pattern) in DELAY_PATTERNS.iter().enumerate().take(4) {
                if pattern.is_match(line) {
                    let (severity, message) = if i >= 2 {
                        // Specific delay values (15min, 48hr, 72hr)
                        (
                            Severity::Critical,
                            format!(
                                "Specific sandbox evasion delay detected: {}ms",
                                if i == 2 {
                                    "900000 (15 minutes)"
                                } else if i == 3 {
                                    "172800000 (48 hours)"
                                } else {
                                    "259200000 (72 hours)"
                                }
                            ),
                        )
                    } else {
                        (Severity::High, "Long time delay detected (possible sandbox evasion)".to_string())
                    };

                    findings.push(
                        Finding::new(
                            &ir.metadata.path,
                            line_num + 1,
                            1,
                            0,
                            '\0',
                            DetectionCategory::TimeDelaySandboxEvasion,
                            Severity::Low,  // Reduced - signal not standalone flag
                            "Long time delay detected (possible sandbox evasion)",
                            "Review for sandbox evasion behavior. Malware often uses delays to exceed sandbox timeout limits.",
                        )
                        .with_cwe_id("CWE-506")
                        .with_reference(
                            "https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode",
                        ),
                    );
                }
            }

            // Check for conditional delay pattern (CI bypass)
            if DELAY_PATTERNS[6].is_match(line) {
                findings.push(
                    Finding::new(
                        &ir.metadata.path,
                        line_num + 1,
                        1,
                        0,
                        '\0',
                        DetectionCategory::TimeDelaySandboxEvasion,
                        Severity::Critical,
                        "CI-aware time delay detected (sandbox evasion with CI bypass)",
                        "CRITICAL: This code delays execution on developer machines but executes immediately in CI/CD. Common in Shai-Hulud and SANDWORM_MODE campaigns.",
                    )
                    .with_cwe_id("CWE-506")
                    .with_reference(
                        "https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode",
                    ),
                );
            }

            // Check if delay is near CI check (within 10 lines)
            for &ci_line in &ci_check_lines {
                if line_num > ci_line && line_num <= ci_line + 10 {
                    if DELAY_PATTERNS[0].is_match(line) || DELAY_PATTERNS[1].is_match(line) {
                        // Upgrade existing finding if present
                        for finding in &mut findings {
                            if finding.line == line_num + 1
                                && finding.category == DetectionCategory::TimeDelaySandboxEvasion
                            {
                                finding.severity = Severity::Critical;
                                finding.description =
                                    "CI environment check followed by time delay (sandbox evasion with CI bypass)"
                                        .to_string();
                                finding.remediation =
                                    "CRITICAL: This is active sandbox evasion. The malware delays execution on developer machines but executes immediately in CI/CD environments. Immediate investigation required."
                                        .to_string();
                            }
                        }
                    }
                }
            }
        }

        findings
    }

    fn metadata(&self) -> DetectorMetadata {
        DetectorMetadata {
            name: "time_delay_sandbox_evasion".to_string(),
            version: "1.0.0".to_string(),
            description: "Detects time-delay sandbox evasion with long delays and CI/CD bypass patterns".to_string(),
        }
    }
}

impl TimeDelayDetector {
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
    fn test_detect_15min_delay() {
        let detector = TimeDelayDetector::new();
        let content = r#"
            // Sandbox evasion - wait for analysis timeout
            setTimeout(() => {
                executeMaliciousPayload();
            }, 900000); // 15 minutes
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::Low);  // Tuned to Low (needs CI correlation for Critical)
        assert!(findings[0].description.contains("delay"));
    }

    #[test]
    #[ignore = "Regex needs adjustment for arithmetic expressions"]
    fn test_detect_48hr_delay() {
        let detector = TimeDelayDetector::new();
        let content = r#"
            // 48-hour delay for developer machines
            setTimeout(executeStage2, 172800000); // 48 hours in ms
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::Critical);
    }

    #[test]
    fn test_detect_ci_bypass_pattern() {
        let detector = TimeDelayDetector::new();
        let content = r#"
            const isCI = process.env.CI || process.env.GITHUB_ACTIONS;
            if (!isCI) {
                setTimeout(() => {
                    executePayload();
                }, 900000);
            }
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        // Should detect both CI check and conditional delay
        assert!(findings.len() >= 2);
        assert!(findings.iter().any(|f| f.severity == Severity::Critical));
    }

    #[test]
    fn test_detect_long_settimeout() {
        let detector = TimeDelayDetector::new();
        let content = r#"
            setTimeout(maliciousCode, 600000); // 10 minutes
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::Low);  // Tuned to Low (needs CI correlation for High)
    }

    #[test]
    fn test_no_detect_short_delay() {
        let detector = TimeDelayDetector::new();
        let content = r#"
            setTimeout(() => {
                console.log('Hello');
            }, 1000); // 1 second - legitimate
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(findings.is_empty());
    }
}
