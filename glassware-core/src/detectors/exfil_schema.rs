//! Exfil Schema Detector (G4)
//!
//! Detects the JSON key vocabulary used in GlassWorm's data exfiltration payloads.
//! The schema is reconstructed from Part 5 disassembly of the JSON output writer.
//!
//! ## Detection Logic
//!
//! This detector emits findings when:
//! 1. ≥3 exfil keys co-occur in the same file
//! 2. At least one key is "high-signal" (e.g., sync_oauth_token, walletCount)
//!
//! ## Complete Key List (from PART5.md exfil JSON schema)
//!
//! ### High-Signal Keys (require ≥2 additional keys to fire)
//! - `sync_oauth_token` - Chrome Sync OAuth token
//! - `send_tab_private_key` - Chrome/Firefox Send Tab private key
//! - `close_tab_private_key` - Chrome/Firefox Close Tab private key
//! - `walletCount` - Cryptocurrency wallet count
//! - `creditCardCount` - Credit card count
//! - `cookieCount` - Cookie count
//! - `loginCount` - Login credential count
//!
//! ### Medium-Signal Keys (require ≥3 additional keys to fire)
//! - `master_key` - Browser master encryption key
//! - `app_bound_key` - V20 App-Bound encryption key
//! - `dpapi_key` - V10 DPAPI encryption key
//! - `session_token` - Active session token
//! - `profile_oauth_token` - Profile OAuth token
//!
//! ### Low-Signal Keys (context only, don't count toward threshold)
//! - `user_agent`, `email`, `uid`, `verified`
//! - Array keys: `cookies`, `logins`, `credit_cards`, `autofill`, `history`, `bookmarks`, `tokens`
//!
//! ## Severity
//!
//! - HIGH: ≥3 keys including ≥1 high-signal key
//! - MEDIUM: ≥4 keys without high-signal keys
//! - INFO: 1-2 keys alone (could be legitimate)
//!
//! ## Configuration
//!
//! Threshold is configurable via environment variable:
//! - `GLASSWARE_EXFIL_THRESHOLD` (default: 3)
//!
//! ## False Positive Mitigation
//!
//! Single keys or common JSON keys (user_agent, email) alone don't trigger.
//! Requires co-occurrence of multiple exfil-specific keys.

use crate::detector::{Detector, DetectorTier};
use crate::finding::{DetectionCategory, Finding, Severity};
use crate::ir::FileIR;
use std::path::Path;

/// High-signal exfil keys - require ≥2 additional keys to fire HIGH
const HIGH_SIGNAL_KEYS: &[&str] = &[
    "sync_oauth_token",
    "send_tab_private_key",
    "close_tab_private_key",
    "walletCount",
    "creditCardCount",
    "cookieCount",
    "loginCount",
];

/// Medium-signal exfil keys - require ≥3 additional keys to fire MEDIUM
const MEDIUM_SIGNAL_KEYS: &[&str] = &[
    "master_key",
    "app_bound_key",
    "dpapi_key",
    "session_token",
    "profile_oauth_token",
];

/// Low-signal keys - context only, don't count toward threshold
const LOW_SIGNAL_KEYS: &[&str] = &[
    "user_agent",
    "email",
    "uid",
    "verified",
    // Array container keys
    "cookies",
    "logins",
    "credit_cards",
    "autofill",
    "history",
    "bookmarks",
    "tokens",
];

/// All exfil keys for matching
const ALL_EXFIL_KEYS: &[&str] = &[
    "sync_oauth_token",
    "send_tab_private_key",
    "close_tab_private_key",
    "walletCount",
    "creditCardCount",
    "cookieCount",
    "loginCount",
    "master_key",
    "app_bound_key",
    "dpapi_key",
    "session_token",
    "profile_oauth_token",
    "user_agent",
    "email",
    "uid",
    "verified",
    "cookies",
    "logins",
    "credit_cards",
    "autofill",
    "history",
    "bookmarks",
    "tokens",
];

/// Detector for GlassWorm exfil JSON schema
pub struct ExfilSchemaDetector {
    threshold: usize,
}

impl ExfilSchemaDetector {
    /// Create a new exfil schema detector with default threshold
    pub fn new() -> Self {
        Self {
            threshold: std::env::var("GLASSWARE_EXFIL_THRESHOLD")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3),
        }
    }

    /// Create with custom threshold
    pub fn with_threshold(threshold: usize) -> Self {
        Self { threshold }
    }
}

impl Default for ExfilSchemaDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for ExfilSchemaDetector {
    fn name(&self) -> &str {
        "exfil_schema"
    }

    fn tier(&self) -> DetectorTier {
        DetectorTier::Tier2Secondary
    }

    fn cost(&self) -> u8 {
        3  // Medium cost - string matching with scoring
    }

    fn signal_strength(&self) -> u8 {
        9  // High signal when threshold met
    }

    fn detect(&self, ir: &FileIR) -> Vec<Finding> {
        let mut findings = Vec::new();
        let content = ir.content();

        // Track found keys and their line numbers
        let mut found_keys: Vec<(&str, usize)> = Vec::new();
        let mut high_signal_count = 0;
        let mut medium_signal_count = 0;

        for (line_num, line) in content.lines().enumerate() {
            // Check for each exfil key
            for key in ALL_EXFIL_KEYS {
                // Match JSON key pattern: "key": or 'key':
                let patterns = [
                    format!("\"{}\"", key),
                    format!("'{}'", key),
                ];

                for pattern in &patterns {
                    if line.contains(pattern) {
                        found_keys.push((key, line_num + 1));

                        // Categorize signal strength
                        if HIGH_SIGNAL_KEYS.contains(key) {
                            high_signal_count += 1;
                        } else if MEDIUM_SIGNAL_KEYS.contains(key) {
                            medium_signal_count += 1;
                        }
                        // Low-signal keys don't count toward threshold
                    }
                }
            }
        }

        // Calculate total score (only high + medium signal keys count)
        let total_signal_keys = high_signal_count + medium_signal_count;

        // Determine if threshold is met
        if total_signal_keys >= self.threshold {
            // Determine severity
            let (severity, message) = if high_signal_count > 0 {
                (
                    Severity::High,
                    format!(
                        "GlassWorm exfil JSON schema detected ({} keys including {} high-signal)",
                        total_signal_keys, high_signal_count
                    ),
                )
            } else {
                (
                    Severity::Medium,
                    format!(
                        "GlassWorm exfil JSON schema detected ({} keys, no high-signal)",
                        total_signal_keys
                    ),
                )
            };

            // Build key list for description
            let key_list: Vec<&str> = found_keys.iter().map(|(k, _)| *k).collect();
            let unique_keys: Vec<&str> = key_list.iter()
                .fold(Vec::new(), |mut acc, &k| {
                    if !acc.contains(&k) { acc.push(k); }
                    acc
                });

            findings.push(self.create_finding(
                Path::new(&ir.metadata.path),
                found_keys.first().map(|(_, l)| *l).unwrap_or(1),
                &unique_keys,
                total_signal_keys,
                high_signal_count,
                severity,
                &message,
            ));
        }

        findings
    }
}

impl ExfilSchemaDetector {
    fn create_finding(
        &self,
        path: &Path,
        line: usize,
        keys: &[&str],
        total_keys: usize,
        high_signal_keys: usize,
        severity: Severity,
        message: &str,
    ) -> Finding {
        Finding::new(
            &path.to_string_lossy(),
            line,
            1,
            0,
            '\0',
            DetectionCategory::Unknown,  // Using Unknown for exfil schema
            severity,
            message,
            "GlassWorm uses a specific JSON schema for data exfiltration. Review for credential theft and data collection.",
        )
        .with_cwe_id("CWE-359")  // Exposure of Private Information
        .with_reference("https://github.com/samgrowls/glassworks/blob/main/glassworm-writeup/PART5.md")
        .with_context(&format!(
            "Detected keys: {:?} ({} total, {} high-signal)",
            keys, total_keys, high_signal_keys
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::UnicodeConfig;

    #[test]
    fn test_detect_high_signal_schema() {
        let detector = ExfilSchemaDetector::new();
        let content = r#"
            const exfilData = {
                "sync_oauth_token": "<token>",
                "send_tab_private_key": "<key>",
                "cookieCount": 150,
                "loginCount": 45,
            };
        "#;

        let ir = FileIR::build(Path::new("exfil.js"), content);
        let findings = detector.detect(&ir);
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::High);
        assert!(findings[0].description.contains("high-signal"));
    }

    #[test]
    fn test_detect_medium_schema() {
        let detector = ExfilSchemaDetector::new();
        let content = r#"
            const data = {
                "master_key": "<key>",
                "app_bound_key": "<key>",
                "dpapi_key": "<key>",
                "session_token": "<token>",
            };
        "#;

        let ir = FileIR::build(Path::new("exfil.js"), content);
        let findings = detector.detect(&ir);
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::Medium);
    }

    #[test]
    fn test_no_detect_single_key() {
        let detector = ExfilSchemaDetector::new();
        let content = r#"
            // Single key alone (could be legitimate)
            const config = {
                "user_agent": "Mozilla/5.0",
            };
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_no_detect_two_keys() {
        let detector = ExfilSchemaDetector::new();
        let content = r#"
            // Two keys below threshold
            const data = {
                "email": "user@example.com",
                "uid": "12345",
            };
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_detect_threshold_met() {
        let detector = ExfilSchemaDetector::new();
        let content = r#"
            const exfil = {
                "cookies": [],
                "logins": [],
                "master_key": "<key>",
                "app_bound_key": "<key>",
                "dpapi_key": "<key>",
            };
        "#;

        let ir = FileIR::build(Path::new("exfil.js"), content);
        let findings = detector.detect(&ir);
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_custom_threshold() {
        let detector = ExfilSchemaDetector::with_threshold(5);
        let content = r#"
            const data = {
                "cookieCount": 100,
                "loginCount": 50,
                "master_key": "<key>",
            };
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);
        // Only 3 signal keys, threshold is 5, should NOT fire
        assert!(findings.is_empty());
    }

    #[test]
    fn test_detect_full_schema() {
        let detector = ExfilSchemaDetector::new();
        let content = r#"
            // Full GlassWorm exfil schema from PART5
            const payload = {
                "user_agent": "Mozilla/5.0",
                "master_key": "<base64>",
                "email": "user@gmail.com",
                "uid": "12345",
                "verified": true,
                "session_token": "<token>",
                "sync_oauth_token": "<oauth>",
                "send_tab_private_key": "<key>",
                "app_bound_key": "<key>",
                "dpapi_key": "<key>",
                "tokens": [],
                "cookies": [],
                "logins": [],
                "credit_cards": [],
                "autofill": [],
                "history": [],
                "bookmarks": [],
            };
        "#;

        let ir = FileIR::build(Path::new("full_exfil.js"), content);
        let findings = detector.detect(&ir);
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::High);
    }
}
