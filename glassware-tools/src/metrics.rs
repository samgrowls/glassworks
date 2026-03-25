//! Metrics Calculation Module
//! 
//! Calculates F1 score, FP rate, and detection rate from benchmark results.

use serde::{Deserialize, Serialize};

/// Benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub evidence_total: usize,
    pub evidence_detected: usize,
    pub clean_total: usize,
    pub clean_flagged: usize,
    pub scan_time_seconds: f64,
    pub scan_speed_loc_per_sec: f64,
}

impl BenchmarkResult {
    /// Calculate false positive rate
    pub fn fp_rate(&self) -> f32 {
        if self.clean_total == 0 {
            0.0
        } else {
            self.clean_flagged as f32 / self.clean_total as f32
        }
    }
    
    /// Calculate detection rate (recall)
    pub fn detection_rate(&self) -> f32 {
        if self.evidence_total == 0 {
            0.0
        } else {
            self.evidence_detected as f32 / self.evidence_total as f32
        }
    }
    
    /// Calculate precision
    pub fn precision(&self) -> f32 {
        let total_flagged = self.clean_flagged + self.evidence_detected;
        if total_flagged == 0 {
            0.0
        } else {
            self.evidence_detected as f32 / total_flagged as f32
        }
    }
    
    /// Calculate F1 score
    pub fn f1_score(&self) -> f32 {
        let precision = self.precision();
        let recall = self.detection_rate();
        
        if precision + recall == 0.0 {
            0.0
        } else {
            2.0 * (precision * recall) / (precision + recall)
        }
    }
    
    /// Calculate objective score (used for optimization)
    /// Returns 0.0 if hard constraints are violated
    pub fn objective_score(&self, config: &crate::config::AutoresearchConfig) -> f32 {
        // Hard constraint: Evidence detection must be ≥90%
        if self.detection_rate() < config.optimization.target_detection_rate {
            return 0.0;
        }
        
        // Calculate F1 score
        let f1 = self.f1_score();
        
        // Penalty for FP rate above target
        let fp_penalty = if self.fp_rate() > config.optimization.target_fp_rate {
            let excess = self.fp_rate() - config.optimization.target_fp_rate;
            (1.0 - (excess * 10.0)).max(0.0)
        } else {
            1.0
        };
        
        // Penalty for slow scan speed
        let speed_penalty = if self.scan_speed_loc_per_sec < 20000.0 {
            0.5
        } else if self.scan_speed_loc_per_sec < 30000.0 {
            0.8
        } else {
            1.0
        };
        
        f1 * fp_penalty * speed_penalty
    }
}

/// Iteration record for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationRecord {
    pub iteration: u32,
    pub timestamp: String,
    pub fp_rate: f32,
    pub detection_rate: f32,
    pub f1_score: f32,
    pub objective_score: f32,
    pub parameters: serde_json::Value,
    pub accepted: bool,
}

impl IterationRecord {
    pub fn new(
        iteration: u32,
        result: &BenchmarkResult,
        config: &crate::config::AutoresearchConfig,
        parameters: serde_json::Value,
    ) -> Self {
        let obj_score = result.objective_score(config);
        Self {
            iteration,
            timestamp: chrono::Utc::now().to_rfc3339(),
            fp_rate: result.fp_rate(),
            detection_rate: result.detection_rate(),
            f1_score: result.f1_score(),
            objective_score: obj_score,
            parameters,
            accepted: obj_score > 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_f1_score_calculation() {
        let result = BenchmarkResult {
            evidence_total: 10,
            evidence_detected: 9,
            clean_total: 100,
            clean_flagged: 5,
            scan_time_seconds: 100.0,
            scan_speed_loc_per_sec: 50000.0,
        };
        
        assert!((result.detection_rate() - 0.9).abs() < 0.001);
        assert!((result.fp_rate() - 0.05).abs() < 0.001);
        assert!(result.f1_score() > 0.0);
    }
}
