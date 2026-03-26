//! Scoring Configuration
//!
//! This module provides configuration for the scoring system with:
//! - Tiered scoring configuration
//! - Per-detector weights
//! - Conditional rules for dynamic scoring adjustments
//! - Backward compatible with existing configurations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Scoring configuration
///
/// Controls how threat scores are calculated from findings, including
/// tiered execution, detector weights, and conditional rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    // === Thresholds ===
    /// Threat score threshold for "malicious" classification (default: 8.0)
    #[serde(default = "default_malicious_threshold")]
    pub malicious_threshold: f32,

    /// Threat score threshold for "suspicious" classification (default: 4.0)
    #[serde(default = "default_suspicious_threshold")]
    pub suspicious_threshold: f32,

    // === Tier Configuration (New) ===
    /// Tier configuration for tiered scoring execution
    #[serde(default)]
    pub tier_config: TierConfig,

    // === Detector Weights (New) ===
    /// Per-detector weight configuration
    #[serde(default)]
    pub weights: DetectorWeights,

    // === Conditional Rules (New) ===
    /// Conditional rules for dynamic scoring adjustments
    #[serde(default)]
    pub conditional_rules: Vec<ConditionalRule>,

    // === Legacy Weights (Deprecated but kept for backward compatibility) ===
    /// Base weights per severity level (legacy, kept for backward compatibility)
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

// ============================================================================
// Tier Configuration
// ============================================================================

/// Tier execution mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TierMode {
    /// All detectors run, scores summed (current behavior)
    Independent,
    /// Tiered execution with thresholds
    Tiered,
}

/// Tier configuration for controlling detector execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierConfig {
    /// Execution mode (default: Independent)
    #[serde(default = "default_tier_mode")]
    pub mode: TierMode,

    /// Tier definitions
    #[serde(default)]
    pub tiers: Vec<TierDefinition>,
}

/// Definition of a single tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierDefinition {
    /// Tier level (1 = highest priority)
    pub tier: u8,

    /// Detector names to run in this tier
    pub detectors: Vec<String>,

    /// Score threshold to proceed to next tier
    pub threshold: f32,

    /// Weight multiplier for this tier (default: 1.0)
    #[serde(default = "default_multiplier")]
    pub weight_multiplier: f32,
}

// ============================================================================
// Detector Weights
// ============================================================================

/// Per-detector weight configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectorWeights {
    /// Weight for invisible character detection (default: 5.0)
    #[serde(default = "default_invisible")]
    pub invisible_char: f32,

    /// Weight for homoglyph detection (default: 3.0)
    #[serde(default = "default_homoglyph")]
    pub homoglyph: f32,

    /// Weight for GlassWare pattern detection (default: 8.0)
    #[serde(default = "default_glassware")]
    pub glassware_pattern: f32,

    /// Weight for blockchain C2 detection (default: 10.0)
    #[serde(default = "default_blockchain")]
    pub blockchain_c2: f32,

    /// Weight for header C2 detection (default: 8.0)
    #[serde(default = "default_header")]
    pub header_c2: f32,

    /// Weight for exfiltration schema detection (default: 6.0)
    #[serde(default = "default_exfil")]
    pub exfil_schema: f32,

    /// Weight for locale geofencing detection (default: 7.0)
    #[serde(default = "default_locale")]
    pub locale_geofencing: f32,

    /// Weight for time delay sandbox evasion detection (default: 7.0)
    #[serde(default = "default_time_delay")]
    pub time_delay_sandbox_evasion: f32,

    /// Weight for blockchain polling detection (default: 10.0)
    #[serde(default = "default_blockchain_polling")]
    pub blockchain_polling: f32,

    /// Weight for obfuscation detection (default: 8.0)
    #[serde(default = "default_obfuscation")]
    pub obfuscation: f32,
}

impl DetectorWeights {
    /// Get weight for a detector by category name
    pub fn get_detector_weight(&self, category: &glassware_core::DetectionCategory) -> f32 {
        match category {
            glassware_core::DetectionCategory::InvisibleCharacter => self.invisible_char,
            glassware_core::DetectionCategory::Homoglyph => self.homoglyph,
            glassware_core::DetectionCategory::GlasswarePattern => self.glassware_pattern,
            glassware_core::DetectionCategory::BlockchainC2 => self.blockchain_c2,
            glassware_core::DetectionCategory::HeaderC2 => self.header_c2,
            glassware_core::DetectionCategory::ExfilSchema => self.exfil_schema,
            glassware_core::DetectionCategory::LocaleGeofencing => self.locale_geofencing,
            glassware_core::DetectionCategory::TimeDelaySandboxEvasion => self.time_delay_sandbox_evasion,
            glassware_core::DetectionCategory::SteganoPayload => self.obfuscation,
            glassware_core::DetectionCategory::DecoderFunction => self.obfuscation,
            glassware_core::DetectionCategory::EncryptedPayload => self.obfuscation,
            glassware_core::DetectionCategory::SocketIOC2 => self.blockchain_c2,
            _ => 5.0,  // Default weight for unknown categories
        }
    }
}

// ============================================================================
// Conditional Rules
// ============================================================================

/// Conditional rule for dynamic scoring adjustments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalRule {
    /// Rule name
    pub name: String,

    /// Rule description
    pub description: String,

    /// Condition expression (e.g., "detector == 'blockchain_c2' && score > 5.0")
    pub condition: String,

    /// Action to take (e.g., "multiply_score(1.5)", "set_minimum(9.0)")
    pub action: String,
}

// ============================================================================
// Default Value Functions
// ============================================================================

fn default_malicious_threshold() -> f32 {
    8.0
}

fn default_suspicious_threshold() -> f32 {
    4.0
}

fn default_tier_mode() -> TierMode {
    TierMode::Independent
}

fn default_multiplier() -> f32 {
    1.0
}

fn default_invisible() -> f32 {
    5.0
}

fn default_homoglyph() -> f32 {
    3.0
}

fn default_glassware() -> f32 {
    8.0
}

fn default_blockchain() -> f32 {
    10.0
}

fn default_header() -> f32 {
    8.0
}

fn default_exfil() -> f32 {
    6.0
}

fn default_locale() -> f32 {
    7.0
}

fn default_time_delay() -> f32 {
    7.0
}

fn default_blockchain_polling() -> f32 {
    10.0
}

fn default_obfuscation() -> f32 {
    8.0
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

fn default_finding_base_weights() -> HashMap<String, f32> {
    let mut weights = HashMap::new();
    weights.insert("Critical".to_string(), 3.0);
    weights.insert("High".to_string(), 2.0);
    weights.insert("Medium".to_string(), 1.0);
    weights.insert("Low".to_string(), 0.5);
    weights
}

// ============================================================================
// Default Trait Implementations
// ============================================================================

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            malicious_threshold: default_malicious_threshold(),
            suspicious_threshold: default_suspicious_threshold(),
            tier_config: TierConfig::default(),
            weights: DetectorWeights::default(),
            conditional_rules: Vec::default(),
            finding_base_weights: default_finding_base_weights(),
            pattern_dedup_enabled: default_true(),
            pattern_similarity_threshold: default_similarity_threshold(),
            known_c2_min_score: default_known_c2_min_score(),
            steganography_min_score: default_steganography_min_score(),
            glassworm_c2_min_score: default_glassworm_c2_min_score(),
        }
    }
}

impl Default for TierConfig {
    fn default() -> Self {
        Self {
            mode: default_tier_mode(),
            tiers: Vec::default(),
        }
    }
}

impl Default for TierDefinition {
    fn default() -> Self {
        Self {
            tier: 1,
            detectors: Vec::default(),
            threshold: 0.0,
            weight_multiplier: default_multiplier(),
        }
    }
}

impl Default for DetectorWeights {
    fn default() -> Self {
        Self {
            invisible_char: default_invisible(),
            homoglyph: default_homoglyph(),
            glassware_pattern: default_glassware(),
            blockchain_c2: default_blockchain(),
            header_c2: default_header(),
            exfil_schema: default_exfil(),
            locale_geofencing: default_locale(),
            time_delay_sandbox_evasion: default_time_delay(),
            blockchain_polling: default_blockchain_polling(),
            obfuscation: default_obfuscation(),
        }
    }
}

impl Default for ConditionalRule {
    fn default() -> Self {
        Self {
            name: String::default(),
            description: String::default(),
            condition: String::default(),
            action: String::default(),
        }
    }
}

// ============================================================================
// Helper Methods
// ============================================================================

impl ScoringConfig {
    /// Get the base weight for a severity level (legacy method)
    pub fn get_severity_weight(&self, severity: &str) -> f32 {
        self.finding_base_weights
            .get(severity)
            .copied()
            .unwrap_or(0.5)
    }

    /// Get the weight for a specific detector
    pub fn get_detector_weight(&self, detector: &str) -> f32 {
        match detector {
            "invisible_char" => self.weights.invisible_char,
            "homoglyph" => self.weights.homoglyph,
            "glassware_pattern" => self.weights.glassware_pattern,
            "blockchain_c2" => self.weights.blockchain_c2,
            "header_c2" => self.weights.header_c2,
            "exfil_schema" => self.weights.exfil_schema,
            "locale_geofencing" => self.weights.locale_geofencing,
            "time_delay_sandbox_evasion" => self.weights.time_delay_sandbox_evasion,
            "blockchain_polling" => self.weights.blockchain_polling,
            "obfuscation" => self.weights.obfuscation,
            _ => 1.0, // Default weight for unknown detectors
        }
    }

    /// Check if tiered mode is enabled
    pub fn is_tiered_mode(&self) -> bool {
        self.tier_config.mode == TierMode::Tiered
    }

    /// Get detectors for a specific tier
    pub fn get_tier_detectors(&self, tier_level: u8) -> Option<&Vec<String>> {
        self.tier_config
            .tiers
            .iter()
            .find(|t| t.tier == tier_level)
            .map(|t| &t.detectors)
    }

    /// Get the threshold for a specific tier
    pub fn get_tier_threshold(&self, tier_level: u8) -> Option<f32> {
        self.tier_config
            .tiers
            .iter()
            .find(|t| t.tier == tier_level)
            .map(|t| t.threshold)
    }

    /// Get the weight multiplier for a specific tier
    pub fn get_tier_multiplier(&self, tier_level: u8) -> Option<f32> {
        self.tier_config
            .tiers
            .iter()
            .find(|t| t.tier == tier_level)
            .map(|t| t.weight_multiplier)
    }
}

// ============================================================================
// Tests
// ============================================================================

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

        // New fields
        assert_eq!(config.tier_config.mode, TierMode::Independent);
        assert!(config.tier_config.tiers.is_empty());
        assert_eq!(config.weights.invisible_char, 5.0);
        assert_eq!(config.weights.homoglyph, 3.0);
        assert_eq!(config.weights.glassware_pattern, 8.0);
        assert_eq!(config.weights.blockchain_c2, 10.0);
        assert!(config.conditional_rules.is_empty());
    }

    #[test]
    fn test_detector_weights_default() {
        let weights = DetectorWeights::default();
        assert_eq!(weights.invisible_char, 5.0);
        assert_eq!(weights.homoglyph, 3.0);
        assert_eq!(weights.glassware_pattern, 8.0);
        assert_eq!(weights.blockchain_c2, 10.0);
        assert_eq!(weights.header_c2, 8.0);
        assert_eq!(weights.exfil_schema, 6.0);
        assert_eq!(weights.locale_geofencing, 7.0);
        assert_eq!(weights.time_delay_sandbox_evasion, 7.0);
        assert_eq!(weights.blockchain_polling, 10.0);
        assert_eq!(weights.obfuscation, 8.0);
    }

    #[test]
    fn test_tier_config_default() {
        let tier_config = TierConfig::default();
        assert_eq!(tier_config.mode, TierMode::Independent);
        assert!(tier_config.tiers.is_empty());
    }

    #[test]
    fn test_tier_definition_default() {
        let tier = TierDefinition::default();
        assert_eq!(tier.tier, 1);
        assert!(tier.detectors.is_empty());
        assert_eq!(tier.threshold, 0.0);
        assert_eq!(tier.weight_multiplier, 1.0);
    }

    #[test]
    fn test_get_detector_weight() {
        let config = ScoringConfig::default();
        assert_eq!(config.get_detector_weight("invisible_char"), 5.0);
        assert_eq!(config.get_detector_weight("homoglyph"), 3.0);
        assert_eq!(config.get_detector_weight("blockchain_c2"), 10.0);
        assert_eq!(config.get_detector_weight("unknown_detector"), 1.0);
    }

    #[test]
    fn test_is_tiered_mode() {
        let mut config = ScoringConfig::default();
        assert!(!config.is_tiered_mode());

        config.tier_config.mode = TierMode::Tiered;
        assert!(config.is_tiered_mode());
    }

    #[test]
    fn test_tiered_config_from_toml() {
        let toml_str = r#"
            malicious_threshold = 7.5
            suspicious_threshold = 3.5

            [tier_config]
            mode = "tiered"

            [[tier_config.tiers]]
            tier = 1
            detectors = ["invisible_char", "homoglyph", "bidi"]
            threshold = 5.0
            weight_multiplier = 1.0

            [[tier_config.tiers]]
            tier = 2
            detectors = ["glassware_pattern", "blockchain_c2"]
            threshold = 8.0
            weight_multiplier = 1.5
        "#;

        let config: ScoringConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.malicious_threshold, 7.5);
        assert_eq!(config.suspicious_threshold, 3.5);
        assert_eq!(config.tier_config.mode, TierMode::Tiered);
        assert_eq!(config.tier_config.tiers.len(), 2);

        let tier1 = &config.tier_config.tiers[0];
        assert_eq!(tier1.tier, 1);
        assert_eq!(tier1.detectors, vec!["invisible_char", "homoglyph", "bidi"]);
        assert_eq!(tier1.threshold, 5.0);
        assert_eq!(tier1.weight_multiplier, 1.0);

        let tier2 = &config.tier_config.tiers[1];
        assert_eq!(tier2.tier, 2);
        assert_eq!(tier2.detectors, vec!["glassware_pattern", "blockchain_c2"]);
        assert_eq!(tier2.threshold, 8.0);
        assert_eq!(tier2.weight_multiplier, 1.5);
    }

    #[test]
    fn test_detector_weights_from_toml() {
        let toml_str = r#"
            [weights]
            invisible_char = 6.0
            homoglyph = 4.0
            glassware_pattern = 9.0
            blockchain_c2 = 12.0
            header_c2 = 10.0
            exfil_schema = 8.0
            locale_geofencing = 9.0
            time_delay_sandbox_evasion = 9.0
            blockchain_polling = 12.0
            obfuscation = 10.0
        "#;

        let config: ScoringConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.weights.invisible_char, 6.0);
        assert_eq!(config.weights.homoglyph, 4.0);
        assert_eq!(config.weights.glassware_pattern, 9.0);
        assert_eq!(config.weights.blockchain_c2, 12.0);
        assert_eq!(config.weights.header_c2, 10.0);
        assert_eq!(config.weights.exfil_schema, 8.0);
        assert_eq!(config.weights.locale_geofencing, 9.0);
        assert_eq!(config.weights.time_delay_sandbox_evasion, 9.0);
        assert_eq!(config.weights.blockchain_polling, 12.0);
        assert_eq!(config.weights.obfuscation, 10.0);
    }

    #[test]
    fn test_conditional_rules_from_toml() {
        let toml_str = r#"
            [[conditional_rules]]
            name = "blockchain_amplifier"
            description = "Amplify score for blockchain C2 with polling"
            condition = "detector == 'blockchain_c2' && polling_detected"
            action = "multiply_score(1.5)"

            [[conditional_rules]]
            name = "steganography_minimum"
            description = "Set minimum score for steganography with decoder"
            condition = "detector == 'glassware_pattern' && has_decoder"
            action = "set_minimum(9.0)"
        "#;

        let config: ScoringConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.conditional_rules.len(), 2);

        let rule1 = &config.conditional_rules[0];
        assert_eq!(rule1.name, "blockchain_amplifier");
        assert_eq!(
            rule1.description,
            "Amplify score for blockchain C2 with polling"
        );
        assert_eq!(
            rule1.condition,
            "detector == 'blockchain_c2' && polling_detected"
        );
        assert_eq!(rule1.action, "multiply_score(1.5)");

        let rule2 = &config.conditional_rules[1];
        assert_eq!(rule2.name, "steganography_minimum");
        assert_eq!(
            rule2.description,
            "Set minimum score for steganography with decoder"
        );
        assert_eq!(rule2.condition, "detector == 'glassware_pattern' && has_decoder");
        assert_eq!(rule2.action, "set_minimum(9.0)");
    }

    #[test]
    fn test_backward_compatibility() {
        // Test that old config format still works
        let toml_str = r#"
            malicious_threshold = 8.0
            suspicious_threshold = 4.0
            pattern_dedup_enabled = true
            pattern_similarity_threshold = 0.8
            known_c2_min_score = 9.0
            steganography_min_score = 8.5
            glassworm_c2_min_score = 9.0

            [finding_base_weights]
            Critical = 3.0
            High = 2.0
            Medium = 1.0
            Low = 0.5
        "#;

        // Should deserialize without error
        let config: ScoringConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.malicious_threshold, 8.0);
        assert_eq!(config.suspicious_threshold, 4.0);
        assert!(config.pattern_dedup_enabled);
        assert_eq!(config.tier_config.mode, TierMode::Independent);
        assert_eq!(config.weights.invisible_char, 5.0); // Default value
        assert!(config.conditional_rules.is_empty());
    }

    #[test]
    fn test_full_config_from_toml() {
        let toml_str = r#"
            malicious_threshold = 7.0
            suspicious_threshold = 3.0

            [tier_config]
            mode = "tiered"

            [[tier_config.tiers]]
            tier = 1
            detectors = ["invisible_char", "homoglyph"]
            threshold = 4.0

            [[tier_config.tiers]]
            tier = 2
            detectors = ["glassware_pattern", "blockchain_c2"]
            threshold = 7.0
            weight_multiplier = 1.2

            [weights]
            invisible_char = 6.0
            homoglyph = 4.0
            glassware_pattern = 10.0
            blockchain_c2 = 12.0

            [[conditional_rules]]
            name = "test_rule"
            description = "Test conditional rule"
            condition = "score > 5.0"
            action = "multiply_score(1.5)"
        "#;

        let config: ScoringConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.malicious_threshold, 7.0);
        assert_eq!(config.suspicious_threshold, 3.0);
        assert_eq!(config.tier_config.mode, TierMode::Tiered);
        assert_eq!(config.tier_config.tiers.len(), 2);
        assert_eq!(config.weights.invisible_char, 6.0);
        assert_eq!(config.weights.homoglyph, 4.0);
        assert_eq!(config.weights.glassware_pattern, 10.0);
        assert_eq!(config.weights.blockchain_c2, 12.0);
        assert_eq!(config.conditional_rules.len(), 1);
        assert_eq!(config.conditional_rules[0].name, "test_rule");
    }

    #[test]
    fn test_config_serialization() {
        let config = ScoringConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: ScoringConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config.malicious_threshold, deserialized.malicious_threshold);
        assert_eq!(config.suspicious_threshold, deserialized.suspicious_threshold);
        assert_eq!(
            config.tier_config.mode,
            deserialized.tier_config.mode
        );
        assert_eq!(config.weights.invisible_char, deserialized.weights.invisible_char);
    }

    #[test]
    fn test_tier_helper_methods() {
        let mut config = ScoringConfig::default();
        config.tier_config.mode = TierMode::Tiered;
        config.tier_config.tiers = vec![
            TierDefinition {
                tier: 1,
                detectors: vec!["invisible_char".to_string(), "homoglyph".to_string()],
                threshold: 5.0,
                weight_multiplier: 1.0,
            },
            TierDefinition {
                tier: 2,
                detectors: vec!["glassware_pattern".to_string()],
                threshold: 8.0,
                weight_multiplier: 1.5,
            },
        ];

        assert!(config.is_tiered_mode());

        let tier1_detectors = config.get_tier_detectors(1);
        assert!(tier1_detectors.is_some());
        assert_eq!(tier1_detectors.unwrap().len(), 2);

        let tier2_threshold = config.get_tier_threshold(2);
        assert_eq!(tier2_threshold, Some(8.0));

        let tier2_multiplier = config.get_tier_multiplier(2);
        assert_eq!(tier2_multiplier, Some(1.5));

        let tier3 = config.get_tier_detectors(3);
        assert!(tier3.is_none());
    }
}
