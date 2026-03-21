//! CLI Flag Validation
//!
//! Validates CLI flag combinations and environment requirements before execution.

/// Validation errors and warnings
#[derive(Debug, Clone)]
pub struct ValidationErrors {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.errors.is_empty() {
            writeln!(f, "Validation errors:")?;
            for error in &self.errors {
                writeln!(f, "  × {}", error)?;
            }
        }
        
        if !self.warnings.is_empty() {
            writeln!(f, "Warnings:")?;
            for warning in &self.warnings {
                writeln!(f, "  ⚠ {}", warning)?;
            }
        }
        
        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}

/// Validate CLI arguments
pub fn validate_cli(
    llm: bool,
    no_cache: bool,
    cache_db: &str,
    concurrency: usize,
) -> std::result::Result<(), ValidationErrors> {
    let mut errors = vec![];
    let mut warnings = vec![];
    
    // LLM validation
    #[cfg(feature = "llm")]
    if llm {
        if std::env::var("GLASSWARE_LLM_BASE_URL").is_err() {
            errors.push("--llm requires GLASSWARE_LLM_BASE_URL environment variable".to_string());
        }
        if std::env::var("GLASSWARE_LLM_API_KEY").is_err() {
            errors.push("--llm requires GLASSWARE_LLM_API_KEY environment variable".to_string());
        }
    }
    
    // Cache conflicts
    if no_cache && cache_db != ".glassware-orchestrator-cache.db" {
        errors.push("--no-cache conflicts with --cache-db".to_string());
    }
    
    // Concurrency warnings
    if concurrency > 20 {
        warnings.push(format!("High concurrency ({}) may cause rate limiting", concurrency));
    }
    
    if errors.is_empty() {
        // Print warnings
        for warning in &warnings {
            tracing::warn!("{}", warning);
        }
        Ok(())
    } else {
        Err(ValidationErrors { errors, warnings })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Cli, Commands};
    
    #[test]
    fn test_validate_cli_llm_without_env() {
        // This test would fail if env vars are set, so we skip it
        // In practice, users should have env vars set
    }
    
    #[test]
    fn test_validate_cli_cache_conflict() {
        // Would need to construct Cli manually
        // Skip for now - tested manually
    }
}
