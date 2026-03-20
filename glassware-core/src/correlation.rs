//! Attack Correlation Module
//!
//! This module provides correlation logic for linking individual findings
//! into unified attack narratives (attack chains).
//!
//! ## Overview
//!
//! The correlation engine analyzes spatial, temporal, and logical relationships
//! between findings to identify multi-stage attacks such as:
//!
//! - **GlassWare Stego Chain**: Unicode stego → decoder function → eval/exec
//! - **Encrypted Exec Chain**: High-entropy blob → decrypt → dynamic execution
//! - **Header C2 Chain**: HTTP header extraction → decryption → execution
//!
//! ## Example
//!
//! ```rust
//! use glassware_core::correlation::{AttackGraphEngine, AttackType};
//! use glassware_core::Finding;
//!
//! let mut engine = AttackGraphEngine::new();
//! engine.add_findings(findings);
//!
//! for chain in engine.get_chains() {
//!     println!("Attack chain: {:?}", chain.classification);
//!     println!("Confidence: {:.2}", chain.confidence);
//! }
//! ```

use crate::finding::{DetectionCategory, Finding, Severity};
use std::collections::HashMap;
use std::path::Path;

/// Unique identifier generator for attack chains
fn generate_chain_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("chain_{:x}", timestamp)
}

/// An attack chain correlates multiple findings into a unified narrative
#[derive(Debug, Clone)]
pub struct AttackChain {
    /// Unique identifier for this chain
    pub id: String,

    /// Ordered steps in the attack (stego → decode → decrypt → exec)
    pub steps: Vec<Finding>,

    /// Overall confidence (0.0-1.0)
    pub confidence: f32,

    /// Classification of attack type
    pub classification: AttackType,

    /// Package/file where this was detected
    pub location: AttackLocation,
}

impl AttackChain {
    /// Create a new attack chain
    pub fn new(
        steps: Vec<Finding>,
        confidence: f32,
        classification: AttackType,
        location: AttackLocation,
    ) -> Self {
        Self {
            id: generate_chain_id(),
            steps,
            confidence,
            classification,
            location,
        }
    }

    /// Get the highest severity among all steps
    pub fn highest_severity(&self) -> Severity {
        self.steps.iter().map(|s| s.severity).max().unwrap_or(Severity::Info)
    }

    /// Get the number of steps in this chain
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Check if this chain contains a specific detection category
    pub fn contains_category(&self, category: &DetectionCategory) -> bool {
        self.steps.iter().any(|s| s.category == *category)
    }
}

/// Classification of attack type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum AttackType {
    /// Unicode steganography with decoder
    GlassWareStego,

    /// Encrypted payload with dynamic execution
    EncryptedExec,

    /// Header-based C2 with decryption
    HeaderC2Chain,

    /// Remote dependency delivery
    RemoteDependencyDelivery,

    /// Supply chain compromise (multiple packages)
    SupplyChainCompromise,

    /// Blockchain-based C2 (Solana, Google Calendar)
    BlockchainC2,

    /// Locale/timezone geofencing with delayed execution
    GeofencedExec,

    /// Unknown/uncategorized
    Unknown,
}

impl AttackType {
    /// Get a human-readable description of the attack type
    pub fn description(&self) -> &'static str {
        match self {
            AttackType::GlassWareStego => {
                "Unicode steganography with decoder function and dynamic execution"
            }
            AttackType::EncryptedExec => {
                "High-entropy encrypted payload with decryption and execution flow"
            }
            AttackType::HeaderC2Chain => {
                "HTTP header-based C2 channel with decryption and payload execution"
            }
            AttackType::RemoteDependencyDelivery => {
                "Remote URL-based dependency delivery (RDD attack)"
            }
            AttackType::SupplyChainCompromise => {
                "Multi-package supply chain compromise with coordinated attack"
            }
            AttackType::BlockchainC2 => {
                "Blockchain-based C2 communication (Solana RPC, Google Calendar)"
            }
            AttackType::GeofencedExec => {
                "Locale/timezone geofencing with time-delay sandbox evasion"
            }
            AttackType::Unknown => "Uncategorized attack pattern",
        }
    }

    /// Get the typical severity for this attack type
    pub fn typical_severity(&self) -> Severity {
        match self {
            AttackType::GlassWareStego => Severity::Critical,
            AttackType::EncryptedExec => Severity::High,
            AttackType::HeaderC2Chain => Severity::Critical,
            AttackType::RemoteDependencyDelivery => Severity::High,
            AttackType::SupplyChainCompromise => Severity::Critical,
            AttackType::BlockchainC2 => Severity::Critical,
            AttackType::GeofencedExec => Severity::High,
            AttackType::Unknown => Severity::Medium,
        }
    }
}

/// Location information for an attack chain
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttackLocation {
    /// Package name (if available)
    pub package: String,

    /// Files involved in the attack
    pub files: Vec<String>,

    /// Package version (if available)
    pub version: Option<String>,
}

impl AttackLocation {
    /// Create a new attack location
    pub fn new(package: &str, files: Vec<String>, version: Option<String>) -> Self {
        Self {
            package: package.to_string(),
            files,
            version,
        }
    }

    /// Create a location from a single file
    pub fn from_file(file: &str) -> Self {
        let package = Self::extract_package_name(file);
        Self {
            package,
            files: vec![file.to_string()],
            version: None,
        }
    }

    /// Extract package name from file path
    fn extract_package_name(file: &str) -> String {
        let path = Path::new(file);

        // Try to extract npm package name (e.g., node_modules/@scope/pkg/file.js)
        let components: Vec<_> = path.components().collect();
        for (i, comp) in components.iter().enumerate() {
            if let Some(name) = comp.as_os_str().to_str() {
                if name == "node_modules" && i + 1 < components.len() {
                    let mut pkg_name = components[i + 1]
                        .as_os_str()
                        .to_string_lossy()
                        .to_string();

                    // Handle scoped packages (@scope/pkg)
                    if pkg_name.starts_with('@') && i + 2 < components.len() {
                        let scope = &pkg_name;
                        let pkg = components[i + 2]
                            .as_os_str()
                            .to_string_lossy()
                            .to_string();
                        pkg_name = format!("{}/{}", scope, pkg);
                    }

                    return pkg_name;
                }
            }
        }

        // Fallback: use directory name
        path.parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    }
}

/// Correlation engine for building attack chains from findings
#[derive(Clone)]
pub struct AttackGraphEngine {
    findings: Vec<Finding>,
    chains: Vec<AttackChain>,
}

impl AttackGraphEngine {
    /// Create a new attack graph engine
    pub fn new() -> Self {
        Self {
            findings: Vec::new(),
            chains: Vec::new(),
        }
    }

    /// Add findings and attempt to correlate them into attack chains
    pub fn add_findings(&mut self, findings: Vec<Finding>) {
        self.findings = findings;
        self.correlate();
    }

    /// Get all detected attack chains
    pub fn get_chains(&self) -> &[AttackChain] {
        &self.chains
    }

    /// Get overall threat score for package (0.0-10.0)
    ///
    /// Score is based on:
    /// - Number of chains
    /// - Chain confidence levels
    /// - Chain severity
    /// - Attack type criticality
    pub fn get_threat_score(&self) -> f32 {
        if self.chains.is_empty() {
            return 0.0;
        }

        let mut score = 0.0;

        for chain in &self.chains {
            // Base score from confidence
            let base = chain.confidence * 2.0; // 0-2 points

            // Severity multiplier
            let severity_mult = match chain.highest_severity() {
                Severity::Critical => 2.0,
                Severity::High => 1.5,
                Severity::Medium => 1.0,
                Severity::Low => 0.5,
                Severity::Info => 0.25,
            };

            // Attack type criticality
            let type_mult = match &chain.classification {
                AttackType::SupplyChainCompromise => 2.0,
                AttackType::GlassWareStego => 1.8,
                AttackType::HeaderC2Chain => 1.8,
                AttackType::BlockchainC2 => 1.6,
                AttackType::EncryptedExec => 1.4,
                AttackType::RemoteDependencyDelivery => 1.3,
                AttackType::GeofencedExec => 1.2,
                AttackType::Unknown => 1.0,
            };

            // Step count bonus (more steps = more sophisticated attack)
            let step_bonus = (chain.step_count() as f32 * 0.2).min(1.0);

            score += (base * severity_mult * type_mult) + step_bonus;
        }

        // Cap at 10.0
        score.min(10.0)
    }

    /// Clear all findings and chains
    pub fn clear(&mut self) {
        self.findings.clear();
        self.chains.clear();
    }

    /// Correlate findings into attack chains
    fn correlate(&mut self) {
        self.chains.clear();

        // Group findings by file
        let findings_by_file = self.group_findings_by_file();

        // Collect new chains in a local vec to avoid borrow checker issues
        let mut new_chains: Vec<AttackChain> = Vec::new();

        // Try to detect each attack chain type
        for (file, file_findings) in &findings_by_file {
            // GlassWare stego chain
            if let Some(chain) = detect_glassware_chain(file_findings, file) {
                new_chains.push(chain);
                continue;  // Skip other chain types for this file
            }

            // Encrypted exec chain
            if let Some(chain) = detect_encrypted_exec_chain(file_findings, file) {
                new_chains.push(chain);
                continue;
            }

            // Header C2 chain
            if let Some(chain) = detect_header_c2_chain(file_findings, file) {
                new_chains.push(chain);
                continue;
            }

            // Blockchain C2 chain
            if let Some(chain) = detect_blockchain_c2_chain(file_findings, file) {
                new_chains.push(chain);
                continue;
            }

            // Geofenced exec chain
            if let Some(chain) = detect_geofenced_exec_chain(file_findings, file) {
                new_chains.push(chain);
            }
        }

        self.chains = new_chains;

        // Check for supply chain compromise (multiple packages with similar patterns)
        if let Some(chain) = detect_supply_chain_compromise(&self.findings) {
            self.chains.push(chain);
        }
    }

    /// Group findings by file path
    fn group_findings_by_file(&self) -> HashMap<String, Vec<&Finding>> {
        let mut groups: HashMap<String, Vec<&Finding>> = HashMap::new();

        for finding in &self.findings {
            groups
                .entry(finding.file.clone())
                .or_default()
                .push(finding);
        }

        groups
    }
}

impl Default for AttackGraphEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect GlassWare-style attack chain
///
/// Pattern: Unicode stego → decoder function → eval/exec
///
/// Required indicators:
/// - SteganoPayload OR InvisibleCharacter finding
/// - DecoderFunction OR GlasswarePattern finding
/// - Evidence of dynamic execution (eval, Function, etc.)
fn detect_glassware_chain(findings: &[&Finding], file: &str) -> Option<AttackChain> {
    let has_stego = findings.iter().any(|f| {
        f.category == DetectionCategory::SteganoPayload
            || f.category == DetectionCategory::InvisibleCharacter
            || f.category == DetectionCategory::PipeDelimiterStego
    });

    let has_decoder = findings.iter().any(|f| {
        f.category == DetectionCategory::DecoderFunction
            || f.category == DetectionCategory::GlasswarePattern
    });

    let has_exec = findings.iter().any(|f| {
        f.description.contains("eval")
            || f.description.contains("Function(")
            || f.description.contains("dynamic execution")
            || f.message_contains_exec()
    });

    if has_stego && has_decoder && has_exec {
        let mut chain_findings: Vec<Finding> = findings
            .iter()
            .filter(|f| {
                matches!(
                    f.category,
                    DetectionCategory::SteganoPayload
                        | DetectionCategory::InvisibleCharacter
                        | DetectionCategory::PipeDelimiterStego
                        | DetectionCategory::DecoderFunction
                        | DetectionCategory::GlasswarePattern
                )
            })
            .map(|f| (*f).clone())
            .collect();

        // Sort by line number for logical flow
        chain_findings.sort_by_key(|f| f.line);

        // Calculate confidence based on indicator count
        let indicator_count = [has_stego, has_decoder, has_exec]
            .iter()
            .filter(|&&x| x)
            .count();

        let confidence = match indicator_count {
            3 => 0.95,
            2 => 0.75,
            _ => 0.5,
        };

        Some(AttackChain::new(
            chain_findings,
            confidence,
            AttackType::GlassWareStego,
            AttackLocation::from_file(file),
        ))
    } else {
        None
    }
}

/// Detect encrypted execution chain
///
/// Pattern: High-entropy blob → decryption → dynamic execution
///
/// Required indicators:
/// - EncryptedPayload finding
/// - Evidence of decryption (crypto APIs, hardcoded keys)
/// - Evidence of dynamic execution
fn detect_encrypted_exec_chain(findings: &[&Finding], file: &str) -> Option<AttackChain> {
    let has_encrypted_payload = findings
        .iter()
        .any(|f| f.category == DetectionCategory::EncryptedPayload);

    let has_hardcoded_key = findings
        .iter()
        .any(|f| f.category == DetectionCategory::HardcodedKeyDecryption);

    let has_rc4 = findings
        .iter()
        .any(|f| f.category == DetectionCategory::Rc4Pattern);

    let has_decrypt = has_encrypted_payload || has_hardcoded_key || has_rc4;

    let has_exec = findings.iter().any(|f| {
        f.description.contains("eval")
            || f.description.contains("Function(")
            || f.description.contains("dynamic execution")
            || f.message_contains_exec()
    });

    if has_decrypt && has_exec {
        let mut chain_findings: Vec<Finding> = findings
            .iter()
            .filter(|f| {
                matches!(
                    f.category,
                    DetectionCategory::EncryptedPayload
                        | DetectionCategory::HardcodedKeyDecryption
                        | DetectionCategory::Rc4Pattern
                )
            })
            .map(|f| (*f).clone())
            .collect();

        chain_findings.sort_by_key(|f| f.line);

        // Higher confidence if we have multiple decryption indicators
        let indicator_count = [has_encrypted_payload, has_hardcoded_key, has_rc4]
            .iter()
            .filter(|&&x| x)
            .count();

        let confidence = if has_exec {
            match indicator_count {
                2..=3 => 0.9,
                1 => 0.75,
                _ => 0.6,
            }
        } else {
            0.5
        };

        Some(AttackChain::new(
            chain_findings,
            confidence,
            AttackType::EncryptedExec,
            AttackLocation::from_file(file),
        ))
    } else {
        None
    }
}

/// Detect HTTP header C2 chain
///
/// Pattern: HTTP header extraction → decryption → execution
///
/// Required indicators:
/// - HeaderC2 finding
/// - Evidence of HTTP client usage
/// - Decryption + execution flow
fn detect_header_c2_chain(findings: &[&Finding], file: &str) -> Option<AttackChain> {
    let has_header_c2 = findings
        .iter()
        .any(|f| f.category == DetectionCategory::HeaderC2);

    let has_exec = findings.iter().any(|f| {
        f.description.contains("eval")
            || f.description.contains("Function(")
            || f.description.contains("dynamic execution")
            || f.message_contains_exec()
    });

    if has_header_c2 && has_exec {
        let chain_findings: Vec<Finding> = findings
            .iter()
            .filter(|f| f.category == DetectionCategory::HeaderC2)
            .map(|f| (*f).clone())
            .collect();

        // Header C2 with exec is almost certainly malicious
        let confidence = 0.95;

        Some(AttackChain::new(
            chain_findings,
            confidence,
            AttackType::HeaderC2Chain,
            AttackLocation::from_file(file),
        ))
    } else {
        None
    }
}

/// Detect blockchain-based C2 chain
///
/// Pattern: Blockchain API calls → data extraction → decryption → execution
///
/// Required indicators:
/// - BlockchainC2 finding
/// - Evidence of dynamic execution
fn detect_blockchain_c2_chain(findings: &[&Finding], file: &str) -> Option<AttackChain> {
    let has_blockchain = findings
        .iter()
        .any(|f| f.category == DetectionCategory::BlockchainC2);

    let has_exec = findings.iter().any(|f| {
        f.description.contains("eval")
            || f.description.contains("Function(")
            || f.description.contains("dynamic execution")
            || f.message_contains_exec()
    });

    if has_blockchain && has_exec {
        let chain_findings: Vec<Finding> = findings
            .iter()
            .filter(|f| f.category == DetectionCategory::BlockchainC2)
            .map(|f| (*f).clone())
            .collect();

        let confidence = 0.9;

        Some(AttackChain::new(
            chain_findings,
            confidence,
            AttackType::BlockchainC2,
            AttackLocation::from_file(file),
        ))
    } else {
        None
    }
}

/// Detect geofenced execution chain
///
/// Pattern: Locale/timezone check → time delay → execution
///
/// Required indicators:
/// - LocaleGeofencing finding
/// - TimeDelaySandboxEvasion finding
/// - Evidence of conditional execution
fn detect_geofenced_exec_chain(findings: &[&Finding], file: &str) -> Option<AttackChain> {
    let has_geofencing = findings
        .iter()
        .any(|f| f.category == DetectionCategory::LocaleGeofencing);

    let has_time_delay = findings
        .iter()
        .any(|f| f.category == DetectionCategory::TimeDelaySandboxEvasion);

    let has_exec = findings.iter().any(|f| {
        f.description.contains("eval")
            || f.description.contains("Function(")
            || f.description.contains("dynamic execution")
            || f.message_contains_exec()
    });

    if (has_geofencing || has_time_delay) && has_exec {
        let mut chain_findings: Vec<Finding> = findings
            .iter()
            .filter(|f| {
                matches!(
                    f.category,
                    DetectionCategory::LocaleGeofencing
                        | DetectionCategory::TimeDelaySandboxEvasion
                )
            })
            .map(|f| (*f).clone())
            .collect();

        chain_findings.sort_by_key(|f| f.line);

        let confidence = if has_geofencing && has_time_delay {
            0.85
        } else if has_geofencing || has_time_delay {
            0.7
        } else {
            0.5
        };

        Some(AttackChain::new(
            chain_findings,
            confidence,
            AttackType::GeofencedExec,
            AttackLocation::from_file(file),
        ))
    } else {
        None
    }
}

/// Detect supply chain compromise across multiple packages
///
/// Pattern: Similar attack patterns in multiple packages
///
/// Required indicators:
/// - Multiple packages with similar findings
/// - Coordinated attack patterns
fn detect_supply_chain_compromise(findings: &[Finding]) -> Option<AttackChain> {
    // Group by package name
    let mut by_package: HashMap<String, Vec<&Finding>> = HashMap::new();

    for finding in findings {
        let package = AttackLocation::extract_package_name(&finding.file);
        by_package.entry(package).or_default().push(finding);
    }

    // Look for multiple packages with critical findings
    let packages_with_critical: Vec<_> = by_package
        .iter()
        .filter(|(_, pkg_findings)| {
            pkg_findings
                .iter()
                .any(|f| f.severity >= Severity::High)
        })
        .collect();

    if packages_with_critical.len() >= 2 {
        // Multiple packages under attack - supply chain compromise
        let all_files: Vec<String> = packages_with_critical
            .iter()
            .flat_map(|(_, pkg_findings)| {
                pkg_findings.iter().map(|f| f.file.clone())
            })
            .collect();

        let all_findings: Vec<Finding> = packages_with_critical
            .iter()
            .flat_map(|(_, pkg_findings)| pkg_findings.iter().map(|f| (*f).clone()))
            .collect();

        let package_name = "multi-package".to_string();

        Some(AttackChain::new(
            all_findings,
            0.9,
            AttackType::SupplyChainCompromise,
            AttackLocation::new(&package_name, all_files, None),
        ))
    } else {
        None
    }
}

/// Extension trait for Finding to check for exec patterns
trait FindingExecExt {
    fn message_contains_exec(&self) -> bool;
}

impl FindingExecExt for Finding {
    fn message_contains_exec(&self) -> bool {
        let msg_lower = self.description.to_lowercase();
        msg_lower.contains("eval")
            || msg_lower.contains("function(")
            || msg_lower.contains("dynamic execution")
            || msg_lower.contains("vm.run")
            || msg_lower.contains("exec")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_finding(
        file: &str,
        line: usize,
        category: DetectionCategory,
        description: &str,
        severity: Severity,
    ) -> Finding {
        Finding::new(
            file,
            line,
            1,
            0,
            '\0',
            category,
            severity,
            description,
            "Remediation",
        )
    }

    #[test]
    fn test_attack_chain_creation() {
        let findings = vec![
            create_finding(
                "test.js",
                10,
                DetectionCategory::SteganoPayload,
                "Steganographic payload detected",
                Severity::Critical,
            ),
            create_finding(
                "test.js",
                15,
                DetectionCategory::DecoderFunction,
                "Decoder function with codePointAt",
                Severity::High,
            ),
        ];

        let mut engine = AttackGraphEngine::new();
        engine.add_findings(findings);

        // Should not create a chain without exec
        assert!(engine.get_chains().is_empty());
    }

    #[test]
    fn test_glassware_chain_detection() {
        let findings = vec![
            create_finding(
                "test.js",
                10,
                DetectionCategory::SteganoPayload,
                "Steganographic payload detected",
                Severity::Critical,
            ),
            create_finding(
                "test.js",
                15,
                DetectionCategory::DecoderFunction,
                "Decoder function with codePointAt",
                Severity::High,
            ),
            create_finding(
                "test.js",
                20,
                DetectionCategory::GlasswarePattern,
                "GlassWare pattern: eval detected",
                Severity::Critical,
            ),
        ];

        let mut engine = AttackGraphEngine::new();
        engine.add_findings(findings);

        let chains = engine.get_chains();
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].classification, AttackType::GlassWareStego);
        assert!(chains[0].confidence >= 0.9);
    }

    #[test]
    fn test_encrypted_exec_chain_detection() {
        let findings = vec![
            create_finding(
                "test.js",
                5,
                DetectionCategory::EncryptedPayload,
                "High-entropy blob with decrypt→exec flow",
                Severity::High,
            ),
            create_finding(
                "test.js",
                10,
                DetectionCategory::HardcodedKeyDecryption,
                "Hardcoded key with dynamic execution",
                Severity::High,
            ),
        ];

        let mut engine = AttackGraphEngine::new();
        engine.add_findings(findings);

        let chains = engine.get_chains();
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].classification, AttackType::EncryptedExec);
    }

    #[test]
    fn test_header_c2_chain_detection() {
        let findings = vec![create_finding(
            "test.js",
            8,
            DetectionCategory::HeaderC2,
            "HTTP header C2 with eval execution",
            Severity::Critical,
        )];

        let mut engine = AttackGraphEngine::new();
        engine.add_findings(findings);

        let chains = engine.get_chains();
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].classification, AttackType::HeaderC2Chain);
    }

    #[test]
    fn test_threat_score_calculation() {
        let findings = vec![
            create_finding(
                "test.js",
                10,
                DetectionCategory::SteganoPayload,
                "Stegano payload",
                Severity::Critical,
            ),
            create_finding(
                "test.js",
                15,
                DetectionCategory::DecoderFunction,
                "Decoder with eval",
                Severity::High,
            ),
            create_finding(
                "test.js",
                20,
                DetectionCategory::GlasswarePattern,
                "GlassWare eval pattern",
                Severity::Critical,
            ),
        ];

        let mut engine = AttackGraphEngine::new();
        engine.add_findings(findings);

        let score = engine.get_threat_score();
        assert!(score > 0.0);
        assert!(score <= 10.0);
    }

    #[test]
    fn test_attack_location_extraction() {
        let location = AttackLocation::from_file("node_modules/@scope/pkg/dist/index.js");
        assert_eq!(location.package, "@scope/pkg");
        assert_eq!(location.files.len(), 1);
    }

    #[test]
    fn test_no_chain_for_clean_code() {
        let findings = vec![create_finding(
            "test.js",
            5,
            DetectionCategory::InvisibleCharacter,
            "Single invisible char",
            Severity::Low,
        )];

        let mut engine = AttackGraphEngine::new();
        engine.add_findings(findings);

        // Single low-severity finding should not form a chain
        assert!(engine.get_chains().is_empty());
    }
}
