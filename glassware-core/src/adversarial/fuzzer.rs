//! Fuzzer Engine for Adversarial Testing
//!
//! Generates random/semi-random inputs to find detector blind spots.

use rand::Rng;

/// Fuzz strategy trait
pub trait FuzzStrategy: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn fuzz(&self, input: &str, intensity: f32) -> String;
}

/// Fuzzer engine orchestrator
pub struct FuzzerEngine {
    strategies: Vec<Box<dyn FuzzStrategy>>,
}

impl FuzzerEngine {
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
        }
    }
    
    pub fn add_strategy(&mut self, strategy: Box<dyn FuzzStrategy>) {
        self.strategies.push(strategy);
    }
    
    pub fn fuzz(&self, input: &str, strategy_name: &str, intensity: f32) -> String {
        let strategy = self.strategies.iter().find(|s| s.name() == strategy_name);
        
        if let Some(strategy) = strategy {
            strategy.fuzz(input, intensity)
        } else {
            input.to_string()
        }
    }
    
    pub fn fuzz_all(&self, input: &str, intensity: f32) -> Vec<(String, String)> {
        self.strategies
            .iter()
            .map(|s| (s.name().to_string(), s.fuzz(input, intensity)))
            .collect()
    }
    
    pub fn strategies(&self) -> Vec<&str> {
        self.strategies.iter().map(|s| s.name()).collect()
    }
}

impl Default for FuzzerEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a fuzz test
#[derive(Debug)]
pub struct FuzzResult {
    pub input: String,
    pub output: String,
    pub strategy: String,
    pub intensity: f32,
    pub crash: bool,
    pub timeout: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fuzzer_engine_creation() {
        let engine = FuzzerEngine::new();
        assert_eq!(engine.strategies().len(), 0);
    }
}
