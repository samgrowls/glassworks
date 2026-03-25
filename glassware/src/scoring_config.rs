//! Scoring Configuration
//!
//! This module provides configuration for the new scoring system with:
//! - Adjustable thresholds for malicious/suspicious classification
//! - Per-severity finding weights
//! - Deduplication settings
//! - Exception rules for known malicious patterns

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

/// Scoring configuration
///
/// Controls how threat scores are calculated from findings, including
/// deduplication, weighting, and exception rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    // === Thresholds ===
    /// Threat score threshold for "malicious" classification (default: 8.0)
    #[serde(default = "default_malicious_threshold")]
    pub malicious_threshold: f32,

    /// Threat score threshold for "suspicious" classification (default: 4.0)
    #[serde(default = "default_suspicious_threshold")]
    pub suspicious_threshold: f32,

    // === Weights ===
    /// Base weights per severity level
    #[serde(default = "default_finding_base_weights")]
    pub finding_base_weights: HashMap<String, f32>,

    // === Deduplication ===
    /// Enable pattern deduplication (default: true)
    #[serde(default = "default_true")]
    pub pattern_dedup_enabled: bool,

    /// Similarity threshold for pattern matching (reserved for future fuzzy matching)
    #[serde(default = "default_similarity_threshold")]
    pub pattern_similarity_threshold: f32,

    // === Exceptions ===
    /// Minimum score for known C2 wallets/IPs (default: 9.0)
    #[serde(default = "default_known_c2_min_score")]
    pub known_c2_min_score: f32,

    /// Minimum score for steganography with decoder (default: 8.5)
    #[serde(default = "default_steganography_min_score")]
    pub steganography_min_score: f32,

    /// Minimum score for GlassWorm C2 polling (default: 9.0)
    #[serde(default = "default_glassworm_c2_min_score")]
    pub glassworm_c2_min_score: f32,
}

/// Default value functions
fn default_malicious_threshold() -> f32 {
    8.0 // Raised from 7.0 for Phase A.5
}

fn default_suspicious_threshold() -> f32 {
    4.0
}

fn default_finding_base_weights() -> HashMap<String, f32> {
    let mut weights = HashMap::new();
    weights.insert("Critical".to_string(), 3.0);
    weights.insert("High".to_string(), 2.0);
    weights.insert("Medium".to_string(), 1.0);
    weights.insert("Low".to_string(), 0.5);
    weights
}

fn default_true() -> bool {
    true
}

fn default_similarity_threshold() -> f32 {
    0.8
}

fn default_known_c2_min_score() -> f32 {
    9.0
}

fn default_steganography_min_score() -> f32 {
    8.5
}

fn default_glassworm_c2_min_score() -> f32 {
    9.0
}

impl Default for ScoringConfig {
    fn default() -> Self {
        // Check for autoresearch runtime config override
        let malicious_threshold = env::var("GLASSWARE_MALICIOUS_THRESHOLD")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(default_malicious_threshold);

        let suspicious_threshold = env::var("GLASSWARE_SUSPICIOUS_THRESHOLD")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(default_suspicious_threshold);

        Self {
            malicious_threshold,
            suspicious_threshold,
            finding_base_weights: default_finding_base_weights(),
            pattern_dedup_enabled: default_true(),
            pattern_similarity_threshold: default_similarity_threshold(),
            known_c2_min_score: default_known_c2_min_score(),
            steganography_min_score: default_steganography_min_score(),
            glassworm_c2_min_score: default_glassworm_c2_min_score(),
        }
    }
}

impl ScoringConfig {
    /// Get the base weight for a severity level
    pub fn get_severity_weight(&self, severity: &str) -> f32 {
        self.finding_base_weights
            .get(severity)
            .copied()
            .unwrap_or(0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ScoringConfig::default();
        assert_eq!(config.malicious_threshold, 8.0);
        assert_eq!(config.suspicious_threshold, 4.0);
        assert!(config.pattern_dedup_enabled);
        assert_eq!(config.known_c2_min_score, 9.0);
        assert_eq!(config.steganography_min_score, 8.5);
        assert_eq!(config.glassworm_c2_min_score, 9.0);
    }

    #[test]
    fn test_severity_weights() {
        let config = ScoringConfig::default();
        assert_eq!(config.get_severity_weight("Critical"), 3.0);
        assert_eq!(config.get_severity_weight("High"), 2.0);
        assert_eq!(config.get_severity_weight("Medium"), 1.0);
        assert_eq!(config.get_severity_weight("Low"), 0.5);
        assert_eq!(config.get_severity_weight("Unknown"), 0.5);
    }

    #[test]
    fn test_config_serialization() {
        let config = ScoringConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: ScoringConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config.malicious_threshold, deserialized.malicious_threshold);
    }
}
