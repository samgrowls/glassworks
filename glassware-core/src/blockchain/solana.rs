//! G10: Solana Memo C2 Parser
//!
//! Detects C2 commands encoded in Solana blockchain transaction memo fields.
//!
//! ## Intel Source
//!
//! From PART3.md, the GlassWorm campaign uses Solana wallets as C2 channels:
//!
//! ### Known C2 Wallets
//! - `28PKnu7RzizxBzFPoLp69HLXp9bJL3JFtT2s5QzHsEA2` — RAT C2 config (Oct 2025 – Present)
//! - `BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC` — RAT C2 (Nov 2025 – Present)
//! - `DSRUBTziADDHSik7WQvSMjvwCHFsbsThrbbjWMoJPUiW` — Chrome extension C2
//! - `6YGcuyFRJKZtcaYCCFba9fScNUvPkGXodXE1mJiSzqDJ` — Payload delivery URLs
//!
//! ### Memo Formats
//!
//! **Config memos (28PKnu, BjVeAj, DSRUBTz wallets):**
//! ```json
//! {"c2server":"http://[IP]:5000","checkIp":"http://[IP]"}
//! ```
//!
//! **Payload delivery (6YGcuy wallet):**
//! ```json
//! {"link":"http://[IP]/[base64-path]"}
//! ```
//! - Path is base64-encoded 16-byte AES block identifier
//! - Path rotates per memo (per-victim/per-campaign)
//!
//! ## Detection Strategy
//!
//! Parse memo JSON and look for:
//! - `c2server` field with HTTP URL
//! - `checkIp` field
//! - `link` field with base64-encoded path
//! - Known GlassWorm wallet addresses
//!
//! ## Severity
//!
//! - **HIGH**: Memo with c2server + known GlassWorm wallet
//! - **MEDIUM**: Memo with c2server field (unverified wallet)
//! - **INFO**: Memo with link field only (structural detection)

use crate::finding::{DetectionCategory, Finding, Severity};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Known GlassWorm Solana C2 wallets
const GLASSWORM_WALLETS: &[&str] = &[
    "28PKnu7RzizxBzFPoLp69HLXp9bJL3JFtT2s5QzHsEA2",
    "BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC",
    "DSRUBTziADDHSik7WQvSMjvwCHFsbsThrbbjWMoJPUiW",
    "6YGcuyFRJKZtcaYCCFba9fScNUvPkGXodXE1mJiSzqDJ",
];

/// Parsed Solana memo command
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MemoCommand {
    /// The wallet address that posted the memo
    pub wallet: String,
    /// C2 server URL (if present)
    pub c2_server: Option<String>,
    /// Check IP URL (if present)
    pub check_ip: Option<String>,
    /// Payload link (if present)
    pub link: Option<String>,
    /// Raw memo content
    pub raw_memo: String,
    /// Whether this wallet is a known GlassWorm C2
    pub is_known_glassworm: bool,
}

/// Solana memo parser
pub struct SolanaMemoParser {
    /// Findings collected during parsing
    findings: Vec<Finding>,
}

impl SolanaMemoParser {
    /// Create a new Solana memo parser
    pub fn new() -> Self {
        Self {
            findings: Vec::new(),
        }
    }

    /// Parse a Solana memo and detect C2 patterns
    #[cfg(feature = "serde")]
    pub fn parse(&mut self, memo_content: &str, wallet: &str) -> Option<MemoCommand> {
        self.findings.clear();

        // Try to parse as JSON
        let memo_json: serde_json::Value = match serde_json::from_str(memo_content) {
            Ok(v) => v,
            Err(_) => {
                // Not JSON - could be base64 or other encoding
                // Flag for manual review at INFO level
                self.add_finding(
                    wallet,
                    Severity::Info,
                    "Non-JSON memo content",
                    memo_content,
                );
                return None;
            }
        };

        // Extract fields
        let c2_server = memo_json.get("c2server").and_then(|v| v.as_str()).map(String::from);
        let check_ip = memo_json.get("checkIp").and_then(|v| v.as_str()).map(String::from);
        let link = memo_json.get("link").and_then(|v| v.as_str()).map(String::from);

        // Check if wallet is known GlassWorm
        let is_known_glassworm = GLASSWORM_WALLETS.iter().any(|w| *w == wallet);

        // Determine severity and create finding
        let severity = if is_known_glassworm && (c2_server.is_some() || link.is_some()) {
            // Known GlassWorm wallet with C2 or payload link = HIGH
            Severity::High
        } else if c2_server.is_some() {
            // Unknown wallet with c2server = MEDIUM
            Severity::Medium
        } else if link.is_some() {
            // Link only (structural detection) = INFO
            Severity::Info
        } else {
            // No C2 indicators found
            return Some(MemoCommand {
                wallet: wallet.to_string(),
                c2_server: None,
                check_ip: None,
                link: None,
                raw_memo: memo_content.to_string(),
                is_known_glassworm,
            });
        };

        // Build context
        let mut context_parts = Vec::new();
        if is_known_glassworm {
            context_parts.push("known GlassWorm wallet");
        }
        if c2_server.is_some() {
            context_parts.push("c2server field");
        }
        if check_ip.is_some() {
            context_parts.push("checkIp field");
        }
        if link.is_some() {
            context_parts.push("link field");
        }

        self.add_finding(
            wallet,
            severity,
            &format!("Solana memo C2 ({})", context_parts.join(", ")),
            memo_content,
        );

        Some(MemoCommand {
            wallet: wallet.to_string(),
            c2_server,
            check_ip,
            link,
            raw_memo: memo_content.to_string(),
            is_known_glassworm,
        })
    }

    /// Parse memo without wallet context (structural detection only)
    #[cfg(feature = "serde")]
    pub fn parse_structural(&mut self, memo_content: &str) -> Option<MemoCommand> {
        self.parse(memo_content, "unknown")
    }

    /// Analyze a memo for C2 patterns without full parsing
    /// Returns severity level based on structural analysis
    pub fn analyze_structure(memo_content: &str) -> Severity {
        // Check for JSON structure
        if !memo_content.starts_with('{') {
            return Severity::Info;
        }

        // Check for key fields
        let has_c2server = memo_content.contains("\"c2server\"");
        let has_check_ip = memo_content.contains("\"checkIp\"");
        let has_link = memo_content.contains("\"link\"");

        if has_c2server {
            Severity::Medium
        } else if has_link {
            Severity::Info
        } else if has_check_ip {
            Severity::Info
        } else {
            Severity::Info
        }
    }

    /// Get findings from the last parse operation
    pub fn get_findings(&self) -> Vec<Finding> {
        self.findings.clone()
    }

    /// Add a finding
    fn add_finding(&mut self, wallet: &str, severity: Severity, category: &str, memo: &str) {
        self.findings.push(
            Finding::new(
                wallet,
                1,
                1,
                0,
                '\0',
                DetectionCategory::BlockchainC2,
                severity,
                "Solana memo C2 pattern detected",
                "Review the memo content for C2 server configuration or payload links.",
            )
            .with_cwe_id("CWE-506")
            .with_reference("https://codeberg.org/tip-o-deincognito/glassworm-writeup/raw/branch/main/PART3.md")
            .with_context(category)
            .with_confidence(if severity == Severity::High { 0.95 } else { 0.7 }),
        );
    }
}

impl Default for SolanaMemoParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a Solana memo and detect C2 patterns
#[cfg(feature = "serde")]
pub fn parse_memo(memo_content: &str, wallet: &str) -> Option<MemoCommand> {
    let mut parser = SolanaMemoParser::new();
    parser.parse(memo_content, wallet)
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = SolanaMemoParser::new();
        assert!(parser.findings.is_empty());
    }

    #[test]
    fn test_parse_config_memo_known_wallet() {
        let memo = r#"{"c2server":"http://217.69.3.51:5000","checkIp":"http://217.69.3.51"}"#;
        let wallet = "28PKnu7RzizxBzFPoLp69HLXp9bJL3JFtT2s5QzHsEA2";

        let mut parser = SolanaMemoParser::new();
        let result = parser.parse(memo, wallet);

        assert!(result.is_some());
        let cmd = result.unwrap();
        assert!(cmd.is_known_glassworm);
        assert_eq!(cmd.c2_server, Some("http://217.69.3.51:5000".to_string()));
        assert_eq!(cmd.check_ip, Some("http://217.69.3.51".to_string()));

        // Should generate HIGH severity finding
        let findings = parser.get_findings();
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::High);
    }

    #[test]
    fn test_parse_payload_link_memo() {
        let memo = r#"{"link":"http://217.69.3.51/Aq9UfpDha27tnnODBaw7OA%3D%3D"}"#;
        let wallet = "6YGcuyFRJKZtcaYCCFba9fScNUvPkGXodXE1mJiSzqDJ";

        let mut parser = SolanaMemoParser::new();
        let result = parser.parse(memo, wallet);

        assert!(result.is_some());
        let cmd = result.unwrap();
        assert!(cmd.is_known_glassworm);
        assert!(cmd.link.is_some());

        // Should generate HIGH severity finding (known wallet + link)
        let findings = parser.get_findings();
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::High);
    }

    #[test]
    fn test_parse_unknown_wallet_with_c2server() {
        let memo = r#"{"c2server":"http://example.com:5000"}"#;
        let wallet = "UnknownWallet123456789";

        let mut parser = SolanaMemoParser::new();
        let result = parser.parse(memo, wallet);

        assert!(result.is_some());
        let cmd = result.unwrap();
        assert!(!cmd.is_known_glassworm);
        assert!(cmd.c2_server.is_some());

        // Should generate MEDIUM severity (c2server but unknown wallet)
        let findings = parser.get_findings();
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::Medium);
    }

    #[test]
    fn test_parse_non_json_memo() {
        let memo = "just plain text memo";
        let wallet = "SomeWallet123";

        let mut parser = SolanaMemoParser::new();
        let result = parser.parse(memo, wallet);

        // Non-JSON memos return None but generate INFO finding
        assert!(result.is_none());
        let findings = parser.get_findings();
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::Info);
    }

    #[test]
    fn test_analyze_structure_c2server() {
        let memo = r#"{"c2server":"http://evil.com:5000","other":"data"}"#;
        let severity = SolanaMemoParser::analyze_structure(memo);
        assert_eq!(severity, Severity::Medium);
    }

    #[test]
    fn test_analyze_structure_link_only() {
        let memo = r#"{"link":"http://evil.com/payload"}"#;
        let severity = SolanaMemoParser::analyze_structure(memo);
        assert_eq!(severity, Severity::Info);
    }

    #[test]
    fn test_analyze_structure_no_indicators() {
        let memo = r#"{"note":"just a regular memo"}"#;
        let severity = SolanaMemoParser::analyze_structure(memo);
        assert_eq!(severity, Severity::Info);
    }

    #[test]
    fn test_all_known_wallets_recognized() {
        for wallet in GLASSWORM_WALLETS {
            let memo = r#"{"c2server":"http://test.com"}"#;
            let mut parser = SolanaMemoParser::new();
            let result = parser.parse(memo, wallet);
            assert!(result.is_some());
            assert!(result.unwrap().is_known_glassworm);
        }
    }
}
