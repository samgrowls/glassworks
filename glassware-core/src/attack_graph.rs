//! Attack Graph Engine
//!
//! This module provides the main attack graph engine that correlates individual findings
//! into unified attack narratives (attack chains).
//!
//! ## Overview
//!
//! The attack graph engine analyzes findings from multiple detectors and identifies
//! multi-stage attack patterns by correlating:
//! - **Spatial proximity**: Findings in the same file or nearby locations
//! - **Logical flow**: Sequential patterns like stego → decode → exec
//! - **Attack signatures**: Known multi-stage attack patterns (GlassWare, encrypted loaders, C2)
//!
//! ## Attack Chain Types
//!
//! The engine detects the following attack chain types:
//!
//! 1. **GlassWareStego**: Unicode steganography with decoder function and dynamic execution
//! 2. **EncryptedExec**: High-entropy encrypted payload with decryption and execution
//! 3. **HeaderC2Chain**: HTTP header-based C2 with decryption and payload execution
//! 4. **BlockchainC2**: Blockchain-based C2 (Solana RPC, Google Calendar)
//! 5. **GeofencedExec**: Locale/timezone geofencing with time-delay sandbox evasion
//! 6. **SupplyChainCompromise**: Multi-package coordinated attack
//!
//! ## Integration with ScanEngine
//!
//! ```rust,no_run
//! use glassware_core::{ScanEngine, ScanResult};
//! use std::path::Path;
//!
//! let engine = ScanEngine::default_detectors()
//!     .with_attack_graph(true);  // Enable attack graph correlation
//!
//! let result = engine.scan(Path::new("src/index.js"), content);
//!
//! // Access attack chains
//! for chain in &result.attack_chains {
//!     println!("Attack chain: {:?}", chain.classification);
//!     println!("Confidence: {:.2}", chain.confidence);
//!     println!("Threat score: {:.1}", result.threat_score);
//! }
//! ```
//!
//! ## Threat Scoring
//!
//! The engine calculates an overall threat score (0.0-10.0) based on:
//! - Number of detected chains
//! - Chain confidence levels
//! - Chain severity (Critical > High > Medium > Low)
//! - Attack type criticality (SupplyChain > GlassWare > HeaderC2 > etc.)
//! - Chain sophistication (more steps = higher score)

pub use crate::correlation::{
    AttackChain, AttackGraphEngine, AttackLocation, AttackType,
};

use crate::finding::Finding;

/// Extended scan result that includes attack chains
pub struct AttackGraphResult {
    /// Individual findings from detectors
    pub findings: Vec<Finding>,

    /// Correlated attack chains
    pub attack_chains: Vec<AttackChain>,

    /// Overall threat score (0.0-10.0)
    pub threat_score: f32,
}

impl AttackGraphResult {
    /// Create a new attack graph result
    pub fn new(findings: Vec<Finding>, attack_chains: Vec<AttackChain>, threat_score: f32) -> Self {
        Self {
            findings,
            attack_chains,
            threat_score,
        }
    }

    /// Check if any attack chains were detected
    pub fn has_attack_chains(&self) -> bool {
        !self.attack_chains.is_empty()
    }

    /// Get the highest severity among all chains
    pub fn highest_chain_severity(&self) -> Option<crate::finding::Severity> {
        self.attack_chains
            .iter()
            .map(|c| c.highest_severity())
            .max()
    }

    /// Get chains filtered by attack type
    pub fn chains_by_type(&self, attack_type: &AttackType) -> Vec<&AttackChain> {
        self.attack_chains
            .iter()
            .filter(|c| c.classification == *attack_type)
            .collect()
    }
}

/// Extension trait for ScanEngine to add attack graph capabilities
pub trait ScanEngineAttackGraphExt {
    /// Enable or disable attack graph correlation
    fn with_attack_graph(self, enabled: bool) -> Self;

    /// Scan with attack graph correlation enabled
    fn scan_with_attack_graph(&self, path: &std::path::Path, content: &str) -> AttackGraphResult;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::correlation::AttackGraphEngine;
    use crate::finding::{DetectionCategory, Severity};

    #[test]
    fn test_attack_graph_engine_creation() {
        let engine = AttackGraphEngine::new();
        assert!(engine.get_chains().is_empty());
        assert_eq!(engine.get_threat_score(), 0.0);
    }

    #[test]
    fn test_attack_graph_result() {
        let findings = vec![];
        let chains = vec![];
        let result = AttackGraphResult::new(findings, chains, 0.0);

        assert!(!result.has_attack_chains());
        assert_eq!(result.highest_chain_severity(), None);
    }

    #[test]
    fn test_attack_type_description() {
        assert!(AttackType::GlassWareStego.description().len() > 0);
        assert!(AttackType::EncryptedExec.description().len() > 0);
        assert!(AttackType::HeaderC2Chain.description().len() > 0);
    }

    #[test]
    fn test_attack_type_severity() {
        assert_eq!(
            AttackType::GlassWareStego.typical_severity(),
            Severity::Critical
        );
        assert_eq!(AttackType::EncryptedExec.typical_severity(), Severity::High);
        assert_eq!(
            AttackType::HeaderC2Chain.typical_severity(),
            Severity::Critical
        );
    }
}
