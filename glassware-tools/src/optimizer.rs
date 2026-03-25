//! Optimizer Module
//! 
//! Proposes new configurations using systematic grid search with random sampling.

use crate::config::{AutoresearchConfig, ParameterRanges};
use rand::Rng;
use serde_json::json;

/// Configuration proposer
pub struct Optimizer {
    config: AutoresearchConfig,
    iteration: u32,
    best_score: f32,
    best_params: Option<serde_json::Value>,
}

impl Optimizer {
    /// Create a new optimizer
    pub fn new(config: AutoresearchConfig) -> Self {
        Self {
            config,
            iteration: 0,
            best_score: 0.0,
            best_params: None,
        }
    }
    
    /// Propose a new configuration
    pub fn propose(&mut self) -> (ScoringParams, serde_json::Value) {
        self.iteration += 1;
        
        let mut rng = rand::thread_rng();
        let ranges = &self.config.parameter_ranges;
        
        // Use grid search for first 20 iterations, then random sampling
        let params = if self.iteration <= 20 {
            // Grid search: systematic exploration
            self.grid_search_proposal(self.iteration, ranges)
        } else {
            // Random sampling around best known configuration
            self.random_proposal(&mut rng, ranges)
        };
        
        // Convert to JSON for logging
        let params_json = json!({
            "malicious_threshold": params.malicious_threshold,
            "suspicious_threshold": params.suspicious_threshold,
            "reputation_tier_1": params.reputation_tier_1,
            "reputation_tier_2": params.reputation_tier_2,
            "llm_override_threshold": params.llm_override_threshold,
            "llm_multiplier_min": params.llm_multiplier_min,
            "category_cap_1": params.category_cap_1,
            "category_cap_2": params.category_cap_2,
            "category_cap_3": params.category_cap_3,
            "dedup_similarity": params.dedup_similarity,
            "log_weight_base": params.log_weight_base,
        });
        
        (params, params_json)
    }
    
    /// Grid search proposal
    fn grid_search_proposal(&self, iteration: u32, ranges: &ParameterRanges) -> ScoringParams {
        // Cycle through parameters systematically
        let step_idx = (iteration - 1) as usize;
        
        ScoringParams {
            malicious_threshold: self.step_value(
                ranges.malicious_threshold_min,
                ranges.malicious_threshold_max,
                ranges.malicious_threshold_step,
                step_idx % 12,
            ),
            suspicious_threshold: self.step_value(
                ranges.suspicious_threshold_min,
                ranges.suspicious_threshold_max,
                ranges.suspicious_threshold_step,
                step_idx % 11,
            ),
            reputation_tier_1: self.step_value(
                ranges.reputation_tier_1_min,
                ranges.reputation_tier_1_max,
                ranges.reputation_tier_1_step,
                step_idx % 7,
            ),
            reputation_tier_2: self.step_value(
                ranges.reputation_tier_2_min,
                ranges.reputation_tier_2_max,
                ranges.reputation_tier_2_step,
                step_idx % 9,
            ),
            llm_override_threshold: self.step_value(
                ranges.llm_override_threshold_min,
                ranges.llm_override_threshold_max,
                ranges.llm_override_threshold_step,
                step_idx % 8,
            ),
            llm_multiplier_min: self.step_value(
                ranges.llm_multiplier_min_min,
                ranges.llm_multiplier_min_max,
                ranges.llm_multiplier_min_step,
                step_idx % 9,
            ),
            category_cap_1: self.step_value(
                ranges.category_cap_1_min,
                ranges.category_cap_1_max,
                ranges.category_cap_1_step,
                step_idx % 11,
            ),
            category_cap_2: self.step_value(
                ranges.category_cap_2_min,
                ranges.category_cap_2_max,
                ranges.category_cap_2_step,
                step_idx % 11,
            ),
            category_cap_3: self.step_value(
                ranges.category_cap_3_min,
                ranges.category_cap_3_max,
                ranges.category_cap_3_step,
                step_idx % 9,
            ),
            dedup_similarity: self.step_value(
                ranges.dedup_similarity_min,
                ranges.dedup_similarity_max,
                ranges.dedup_similarity_step,
                step_idx % 7,
            ),
            log_weight_base: self.step_value(
                ranges.log_weight_base_min,
                ranges.log_weight_base_max,
                ranges.log_weight_base_step,
                step_idx % 11,
            ),
        }
    }
    
    /// Random proposal around best known configuration
    fn random_proposal(&self, rng: &mut impl Rng, ranges: &ParameterRanges) -> ScoringParams {
        // If we have a best config, sample around it
        if let Some(best_json) = &self.best_params {
            if let Some(best) = ScoringParams::from_json(best_json) {
                let noise_scale = 0.1; // 10% noise
                
                return ScoringParams {
                    malicious_threshold: self.clamp_with_noise(
                        best.malicious_threshold,
                        ranges.malicious_threshold_min,
                        ranges.malicious_threshold_max,
                        noise_scale,
                        rng,
                    ),
                    suspicious_threshold: self.clamp_with_noise(
                        best.suspicious_threshold,
                        ranges.suspicious_threshold_min,
                        ranges.suspicious_threshold_max,
                        noise_scale,
                        rng,
                    ),
                    reputation_tier_1: self.clamp_with_noise(
                        best.reputation_tier_1,
                        ranges.reputation_tier_1_min,
                        ranges.reputation_tier_1_max,
                        noise_scale,
                        rng,
                    ),
                    reputation_tier_2: self.clamp_with_noise(
                        best.reputation_tier_2,
                        ranges.reputation_tier_2_min,
                        ranges.reputation_tier_2_max,
                        noise_scale,
                        rng,
                    ),
                    llm_override_threshold: self.clamp_with_noise(
                        best.llm_override_threshold,
                        ranges.llm_override_threshold_min,
                        ranges.llm_override_threshold_max,
                        noise_scale,
                        rng,
                    ),
                    llm_multiplier_min: self.clamp_with_noise(
                        best.llm_multiplier_min,
                        ranges.llm_multiplier_min_min,
                        ranges.llm_multiplier_min_max,
                        noise_scale,
                        rng,
                    ),
                    category_cap_1: self.clamp_with_noise(
                        best.category_cap_1,
                        ranges.category_cap_1_min,
                        ranges.category_cap_1_max,
                        noise_scale,
                        rng,
                    ),
                    category_cap_2: self.clamp_with_noise(
                        best.category_cap_2,
                        ranges.category_cap_2_min,
                        ranges.category_cap_2_max,
                        noise_scale,
                        rng,
                    ),
                    category_cap_3: self.clamp_with_noise(
                        best.category_cap_3,
                        ranges.category_cap_3_min,
                        ranges.category_cap_3_max,
                        noise_scale,
                        rng,
                    ),
                    dedup_similarity: self.clamp_with_noise(
                        best.dedup_similarity,
                        ranges.dedup_similarity_min,
                        ranges.dedup_similarity_max,
                        noise_scale,
                        rng,
                    ),
                    log_weight_base: self.clamp_with_noise(
                        best.log_weight_base,
                        ranges.log_weight_base_min,
                        ranges.log_weight_base_max,
                        noise_scale,
                        rng,
                    ),
                };
            }
        }
        
        // Otherwise, random sample from ranges
        ScoringParams {
            malicious_threshold: rng.gen_range(ranges.malicious_threshold_min..=ranges.malicious_threshold_max),
            suspicious_threshold: rng.gen_range(ranges.suspicious_threshold_min..=ranges.suspicious_threshold_max),
            reputation_tier_1: rng.gen_range(ranges.reputation_tier_1_min..=ranges.reputation_tier_1_max),
            reputation_tier_2: rng.gen_range(ranges.reputation_tier_2_min..=ranges.reputation_tier_2_max),
            llm_override_threshold: rng.gen_range(ranges.llm_override_threshold_min..=ranges.llm_override_threshold_max),
            llm_multiplier_min: rng.gen_range(ranges.llm_multiplier_min_min..=ranges.llm_multiplier_min_max),
            category_cap_1: rng.gen_range(ranges.category_cap_1_min..=ranges.category_cap_1_max),
            category_cap_2: rng.gen_range(ranges.category_cap_2_min..=ranges.category_cap_2_max),
            category_cap_3: rng.gen_range(ranges.category_cap_3_min..=ranges.category_cap_3_max),
            dedup_similarity: rng.gen_range(ranges.dedup_similarity_min..=ranges.dedup_similarity_max),
            log_weight_base: rng.gen_range(ranges.log_weight_base_min..=ranges.log_weight_base_max),
        }
    }
    
    /// Calculate stepped value
    fn step_value(&self, min: f32, max: f32, step: f32, step_idx: usize) -> f32 {
        let value = min + (step * step_idx as f32);
        value.min(max)
    }
    
    /// Add noise and clamp to range
    fn clamp_with_noise(
        &self,
        base: f32,
        min: f32,
        max: f32,
        noise_scale: f32,
        rng: &mut impl Rng,
    ) -> f32 {
        let noise = rng.gen_range(-noise_scale..=noise_scale) * base;
        let value = base + noise;
        value.clamp(min, max)
    }
    
    /// Update with result from iteration
    pub fn update(&mut self, score: f32, params: serde_json::Value) {
        if score > self.best_score {
            self.best_score = score;
            self.best_params = Some(params);
        }
    }
    
    /// Get best score so far
    pub fn best_score(&self) -> f32 {
        self.best_score
    }
}

/// Scoring parameters
#[derive(Debug, Clone)]
pub struct ScoringParams {
    pub malicious_threshold: f32,
    pub suspicious_threshold: f32,
    pub reputation_tier_1: f32,
    pub reputation_tier_2: f32,
    pub llm_override_threshold: f32,
    pub llm_multiplier_min: f32,
    pub category_cap_1: f32,
    pub category_cap_2: f32,
    pub category_cap_3: f32,
    pub dedup_similarity: f32,
    pub log_weight_base: f32,
}

impl ScoringParams {
    /// Parse from JSON
    pub fn from_json(json: &serde_json::Value) -> Option<Self> {
        Some(Self {
            malicious_threshold: json["malicious_threshold"].as_f64()? as f32,
            suspicious_threshold: json["suspicious_threshold"].as_f64()? as f32,
            reputation_tier_1: json["reputation_tier_1"].as_f64()? as f32,
            reputation_tier_2: json["reputation_tier_2"].as_f64()? as f32,
            llm_override_threshold: json["llm_override_threshold"].as_f64()? as f32,
            llm_multiplier_min: json["llm_multiplier_min"].as_f64()? as f32,
            category_cap_1: json["category_cap_1"].as_f64()? as f32,
            category_cap_2: json["category_cap_2"].as_f64()? as f32,
            category_cap_3: json["category_cap_3"].as_f64()? as f32,
            dedup_similarity: json["dedup_similarity"].as_f64()? as f32,
            log_weight_base: json["log_weight_base"].as_f64()? as f32,
        })
    }
}
