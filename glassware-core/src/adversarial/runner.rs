//! Adversarial Test Runner
//!
//! Orchestrates adversarial testing and collects evasion statistics.

use super::mutation::{MaliciousPayload, MutationEngine};
use crate::finding::Finding;

/// Result of a mutation test
#[derive(Debug)]
pub struct MutationTestResult {
    pub original_detected: bool,
    pub mutated_detected: bool,
    pub mutation_type: String,
    pub mutation_rate: f32,
    pub evasion_successful: bool,
}

/// Adversarial test runner
pub struct AdversarialRunner {
    engine: MutationEngine,
}

impl AdversarialRunner {
    /// Create new adversarial runner
    pub fn new(engine: MutationEngine) -> Self {
        Self { engine }
    }
    
    /// Run mutation test on single payload
    pub fn test_mutation(
        &self,
        payload: &MaliciousPayload,
        strategy_name: &str,
        rate: f32,
    ) -> Vec<MutationTestResult> {
        let mutated_payloads = self.engine.mutate(payload, strategy_name, rate);
        
        mutated_payloads
            .iter()
            .map(|mutated| MutationTestResult {
                original_detected: !payload.findings.is_empty(),
                mutated_detected: !mutated.findings.is_empty(),
                mutation_type: strategy_name.to_string(),
                mutation_rate: rate,
                evasion_successful: payload.findings.is_empty() != mutated.findings.is_empty(),
            })
            .collect()
    }
    
    /// Run all mutation strategies on payload
    pub fn test_all(&self, payload: &MaliciousPayload, rate: f32) -> Vec<MutationTestResult> {
        let mutated_payloads = self.engine.mutate_all(payload, rate);
        let strategies = self.engine.strategies();
        
        mutated_payloads
            .iter()
            .zip(strategies.iter())
            .map(|(mutated, &strategy_name)| MutationTestResult {
                original_detected: !payload.findings.is_empty(),
                mutated_detected: !mutated.findings.is_empty(),
                mutation_type: strategy_name.to_string(),
                mutation_rate: rate,
                evasion_successful: payload.findings.is_empty() != mutated.findings.is_empty(),
            })
            .collect()
    }
    
    /// Calculate evasion rate from test results
    pub fn calculate_evasion_rate(results: &[MutationTestResult]) -> f32 {
        if results.is_empty() {
            return 0.0;
        }
        
        let evasions = results.iter().filter(|r| r.evasion_successful).count() as f32;
        evasions / results.len() as f32
    }
    
    /// Generate test report
    pub fn generate_report(&self, results: &[MutationTestResult]) -> String {
        let total = results.len();
        let evasions = results.iter().filter(|r| r.evasion_successful).count();
        let evasion_rate = Self::calculate_evasion_rate(results);
        
        format!(
            "Adversarial Test Report\n\
             =======================\n\
             Total tests: {}\n\
             Successful evasions: {} ({:.1}%)\n\
             Evasion rate: {:.1}%\n",
            total, evasions, (evasions as f32 / total as f32) * 100.0, evasion_rate * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adversarial::strategies::unicode::UnicodeSubstitutionStrategy;
    
    #[test]
    fn test_adversarial_runner_creation() {
        let mut engine = MutationEngine::new();
        engine.add_strategy(Box::new(UnicodeSubstitutionStrategy));
        let runner = AdversarialRunner::new(engine);
        
        assert_eq!(runner.engine.strategies().len(), 1);
    }
    
    #[test]
    fn test_evasion_rate_calculation() {
        let results = vec![
            MutationTestResult {
                original_detected: true,
                mutated_detected: false,
                mutation_type: "test".to_string(),
                mutation_rate: 0.1,
                evasion_successful: true,
            },
            MutationTestResult {
                original_detected: true,
                mutated_detected: true,
                mutation_type: "test".to_string(),
                mutation_rate: 0.1,
                evasion_successful: false,
            },
        ];
        
        let rate = AdversarialRunner::calculate_evasion_rate(&results);
        assert_eq!(rate, 0.5);  // 50% evasion rate
    }
}
