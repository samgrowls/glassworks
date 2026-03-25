//! Autoresearch Configuration Module
//! 
//! Loads and validates autoresearch configuration from TOML file.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoresearchConfig {
    pub optimization: OptimizationConfig,
    pub parameter_ranges: ParameterRanges,
    pub paths: PathsConfig,
}

/// Optimization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    pub max_iterations: u32,
    pub target_fp_rate: f32,
    pub target_detection_rate: f32,
    pub target_f1_score: f32,
    pub use_subset_for_iteration: bool,
    pub subset_size: usize,
    pub full_validation_size: usize,
    pub use_llm_in_phase1: bool,
    pub use_llm_in_phase2: bool,
    pub llm_cache_enabled: bool,
    pub scan_concurrency: usize,
}

/// Parameter ranges for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterRanges {
    // Malicious threshold
    pub malicious_threshold_min: f32,
    pub malicious_threshold_max: f32,
    pub malicious_threshold_step: f32,
    
    // Suspicious threshold
    pub suspicious_threshold_min: f32,
    pub suspicious_threshold_max: f32,
    pub suspicious_threshold_step: f32,
    
    // Reputation multipliers
    pub reputation_tier_1_min: f32,
    pub reputation_tier_1_max: f32,
    pub reputation_tier_1_step: f32,
    
    pub reputation_tier_2_min: f32,
    pub reputation_tier_2_max: f32,
    pub reputation_tier_2_step: f32,
    
    // LLM settings
    pub llm_override_threshold_min: f32,
    pub llm_override_threshold_max: f32,
    pub llm_override_threshold_step: f32,
    
    pub llm_multiplier_min_min: f32,
    pub llm_multiplier_min_max: f32,
    pub llm_multiplier_min_step: f32,
    
    // Category caps
    pub category_cap_1_min: f32,
    pub category_cap_1_max: f32,
    pub category_cap_1_step: f32,
    
    pub category_cap_2_min: f32,
    pub category_cap_2_max: f32,
    pub category_cap_2_step: f32,
    
    pub category_cap_3_min: f32,
    pub category_cap_3_max: f32,
    pub category_cap_3_step: f32,
    
    // Deduplication
    pub dedup_similarity_min: f32,
    pub dedup_similarity_max: f32,
    pub dedup_similarity_step: f32,
    
    // Log weight
    pub log_weight_base_min: f32,
    pub log_weight_base_max: f32,
    pub log_weight_base_step: f32,
}

/// Path configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    pub evidence_dir: String,
    pub clean_packages_dir: String,
    pub output_dir: String,
    pub cache_dir: String,
}

impl AutoresearchConfig {
    /// Load configuration from file
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }
    
    /// Validate configuration
    fn validate(&self) -> anyhow::Result<()> {
        // Validate ranges
        if self.parameter_ranges.malicious_threshold_min >= self.parameter_ranges.malicious_threshold_max {
            anyhow::bail!("malicious_threshold_min must be < malicious_threshold_max");
        }
        
        if self.optimization.target_fp_rate < 0.0 || self.optimization.target_fp_rate > 1.0 {
            anyhow::bail!("target_fp_rate must be between 0.0 and 1.0");
        }
        
        if self.optimization.target_detection_rate < 0.0 || self.optimization.target_detection_rate > 1.0 {
            anyhow::bail!("target_detection_rate must be between 0.0 and 1.0");
        }
        
        Ok(())
    }
    
    /// Get output directory path
    pub fn output_dir(&self) -> PathBuf {
        PathBuf::from(&self.paths.output_dir)
    }
    
    /// Get evidence directory path
    pub fn evidence_dir(&self) -> PathBuf {
        PathBuf::from(&self.paths.evidence_dir)
    }
    
    /// Get clean packages directory path
    pub fn clean_packages_dir(&self) -> PathBuf {
        PathBuf::from(&self.paths.clean_packages_dir)
    }
}
