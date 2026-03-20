//! Variable Renaming Mutation Strategy
//!
//! Renames decoder variables to evade pattern matching.
//! Example: `decoder` → `d3c0d3r`, `decode_func` → `_decode`

use super::super::mutation::MutationStrategy;
use rand::Rng;
use regex::Regex;
use std::sync::LazyLock;

/// Variable renaming mutation strategy
pub struct VariableRenamingStrategy;

// Common decoder variable patterns
static DECODER_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"\bdecoder\b").unwrap(),
        Regex::new(r"\bdecode_func\b").unwrap(),
        Regex::new(r"\bdecodeFn\b").unwrap(),
        Regex::new(r"\bdecoder_func\b").unwrap(),
        Regex::new(r"\bhidden_decoder\b").unwrap(),
    ]
});

// Renaming strategies
static RENAMINGS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "d3c0d3r",
        "_decoder",
        "__decoder__",
        "decoder_",
        "decode_util",
        "str_decoder",
        "decoder_v2",
    ]
});

impl MutationStrategy for VariableRenamingStrategy {
    fn name(&self) -> &str {
        "variable_renaming"
    }
    
    fn description(&self) -> &str {
        "Rename decoder variables to evade pattern matching (decoder → d3c0d3r, etc.)"
    }
    
    fn mutate(&self, payload: &str, rate: f32) -> String {
        use rand::thread_rng;
        let mut rng = thread_rng();
        let mut mutated = payload.to_string();
        
        for pattern in DECODER_PATTERNS.iter() {
            if rng.gen::<f32>() < rate {
                // Choose random renaming
                let new_name = RENAMINGS[rng.gen_range(0..RENAMINGS.len())];
                mutated = pattern.replace_all(&mutated, new_name).to_string();
            }
        }
        
        mutated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;
    
    #[test]
    fn test_variable_renaming_decoder() {
        let strategy = VariableRenamingStrategy;
        let input = "const decoder = (s) => s;";
        let mut rng = thread_rng();
        let mutated = strategy.mutate(&input, 1.0);  // 100% rate
        
        // Should not contain original "decoder"
        assert!(!mutated.contains("decoder = "));
        // Should contain one of the renamings
        assert!(mutated.contains("d3c0d3r") || 
                mutated.contains("_decoder") || 
                mutated.contains("__decoder__"));
    }
    
    #[test]
    fn test_variable_renaming_zero_rate() {
        let strategy = VariableRenamingStrategy;
        let input = "const decoder = (s) => s;";
        let mut rng = thread_rng();
        let mutated = strategy.mutate(&input, 0.0);  // 0% rate
        
        // Should be unchanged
        assert_eq!(mutated, input);
    }
}
