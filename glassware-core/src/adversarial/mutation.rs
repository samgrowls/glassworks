//! Mutation Engine for Adversarial Testing
//!
//! Systematically modifies known malicious patterns to test detector resilience.

use crate::finding::Finding;
use rand::thread_rng;
use rand::Rng;

/// A malicious payload for testing
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MaliciousPayload {
    pub content: String,
    pub file_path: String,
    pub findings: Vec<Finding>,
    pub attack_type: String,
}

impl MaliciousPayload {
    pub fn new(content: String, file_path: String, findings: Vec<Finding>, attack_type: String) -> Self {
        Self {
            content,
            file_path,
            findings,
            attack_type,
        }
    }
}

/// Mutation strategy trait
pub trait MutationStrategy: Send + Sync {
    /// Strategy name
    fn name(&self) -> &str;
    
    /// Strategy description
    fn description(&self) -> &str;
    
    /// Apply mutation to payload content
    fn mutate(&self, payload: &str, rate: f32) -> String;
}

/// Mutation engine orchestrator
pub struct MutationEngine {
    strategies: Vec<Box<dyn MutationStrategy>>,
}

impl MutationEngine {
    /// Create new mutation engine
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
        }
    }
    
    /// Add a mutation strategy
    pub fn add_strategy(&mut self, strategy: Box<dyn MutationStrategy>) {
        self.strategies.push(strategy);
    }
    
    /// Get all registered strategies
    pub fn strategies(&self) -> Vec<&str> {
        self.strategies.iter().map(|s| s.name()).collect()
    }
    
    /// Apply specific mutation strategy to payload
    pub fn mutate(
        &self,
        payload: &MaliciousPayload,
        strategy_name: &str,
        rate: f32,
    ) -> Vec<MaliciousPayload> {
        let mut mutated_payloads = Vec::new();
        
        // Find strategy by name
        let strategy = self.strategies.iter().find(|s| s.name() == strategy_name);
        
        if let Some(strategy) = strategy {
            let mutated_content = strategy.mutate(&payload.content, rate);
            
            mutated_payloads.push(MaliciousPayload::new(
                mutated_content,
                payload.file_path.clone(),
                payload.findings.clone(),
                payload.attack_type.clone(),
            ));
        }
        
        mutated_payloads
    }
    
    /// Apply all mutation strategies to payload
    pub fn mutate_all(&self, payload: &MaliciousPayload, rate: f32) -> Vec<MaliciousPayload> {
        let mut mutated_payloads = Vec::new();
        
        for strategy in &self.strategies {
            let mutated_content = strategy.mutate(&payload.content, rate);
            
            mutated_payloads.push(MaliciousPayload::new(
                mutated_content,
                payload.file_path.clone(),
                payload.findings.clone(),
                payload.attack_type.clone(),
            ));
        }
        
        mutated_payloads
    }
    
    /// Get mutation statistics
    pub fn get_stats(&self) -> MutationStats {
        MutationStats {
            total_strategies: self.strategies.len(),
            strategy_names: self.strategies.iter().map(|s| s.name().to_string()).collect(),
        }
    }
}

impl Default for MutationEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Mutation statistics
#[derive(Debug)]
pub struct MutationStats {
    pub total_strategies: usize,
    pub strategy_names: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adversarial::strategies::unicode::UnicodeSubstitutionStrategy;
    
    #[test]
    fn test_mutation_engine_creation() {
        let engine = MutationEngine::new();
        assert_eq!(engine.strategies().len(), 0);
    }
    
    #[test]
    fn test_add_strategy() {
        let mut engine = MutationEngine::new();
        engine.add_strategy(Box::new(UnicodeSubstitutionStrategy));
        assert_eq!(engine.strategies().len(), 1);
        assert!(engine.strategies().iter().any(|s| &s[..] == "unicode_substitution"));
    }
}
