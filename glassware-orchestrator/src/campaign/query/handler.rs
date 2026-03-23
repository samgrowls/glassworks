//! Campaign Query Handler
//!
//! Provides LLM-based querying of campaign data for interactive analysis.
//!
//! This module allows users to ask natural language questions about campaign
//! findings, packages, and results using the NVIDIA LLM API.

use std::path::Path;
use tracing::info;

use crate::campaign::{CheckpointManager, CampaignCheckpoint};
use crate::llm::{LlmAnalyzer, LlmAnalyzerConfig};
use crate::error::{OrchestratorError, Result};

/// Query a campaign with a natural language question.
///
/// # Arguments
/// * `case_id` - Campaign case ID to query
/// * `question` - Natural language question about the campaign
///
/// # Returns
/// * `Ok(String)` - LLM response to the question
/// * `Err(OrchestratorError)` - Query failed
///
/// # Example
/// ```rust,no_run
/// # async fn example() -> anyhow::Result<()> {
/// let response = query_campaign("case-123", "What packages were flagged as malicious?").await?;
/// println!("{}", response);
/// # Ok(())
/// # }
/// ```
pub async fn query_campaign(case_id: &str, question: &str) -> Result<String> {
    info!("Querying campaign '{}' with question: {}", case_id, question);

    // Load campaign checkpoint
    let checkpoint_db = Path::new(".glassware-checkpoints.db");
    let checkpoint_mgr = CheckpointManager::new(checkpoint_db)
        .map_err(|e| OrchestratorError::internal_error(format!("Failed to open checkpoint database: {}", e)))?;

    let checkpoint = checkpoint_mgr.load_checkpoint(case_id)
        .map_err(|e| OrchestratorError::internal_error(format!("Failed to load checkpoint: {}", e)))?
        .ok_or_else(|| OrchestratorError::not_found(case_id.to_string()))?;

    info!("Loaded checkpoint for campaign: {} (status: {})", checkpoint.case_id, checkpoint.status);

    // Build context from checkpoint data
    let context = build_campaign_context(&checkpoint)?;

    // Create LLM analyzer with NVIDIA deep analysis config
    let analyzer = LlmAnalyzer::with_config(
        LlmAnalyzerConfig::nvidia_deep_analysis()
            .ok_or_else(|| OrchestratorError::config_error("NVIDIA_API_KEY environment variable not set".to_string()))?
    )?;

    // Build the prompt with context and question
    let prompt = build_query_prompt(&context, question);

    // Send to LLM for analysis
    let response = analyzer.query(&prompt).await
        .map_err(|e| OrchestratorError::llm(format!("LLM query failed: {}", e)))?;

    Ok(response)
}

/// Build campaign context from checkpoint data.
fn build_campaign_context(checkpoint: &CampaignCheckpoint) -> Result<String> {
    use serde_json::Value;

    // Parse campaign config
    let config: Value = serde_json::from_str(&checkpoint.config_json)
        .map_err(|e| OrchestratorError::json_error(e, "Failed to parse campaign config"))?;

    // Build context string
    let mut context = String::new();

    context.push_str(&format!("Campaign: {}\n", checkpoint.campaign_name));
    context.push_str(&format!("Case ID: {}\n", checkpoint.case_id));
    context.push_str(&format!("Status: {}\n", checkpoint.status));
    context.push_str(&format!("Created: {}\n", checkpoint.created_at));
    context.push_str(&format!("Updated: {}\n", checkpoint.updated_at));
    context.push_str(&format!("Completed Waves: {:?}\n", checkpoint.completed_waves));

    if let Some(current_wave) = &checkpoint.current_wave {
        context.push_str(&format!("Current Wave: {}\n", current_wave));
    }

    // Add config summary if available
    if let Some(waves) = config.get("waves").and_then(|v| v.as_array()) {
        context.push_str(&format!("\nTotal Waves: {}\n", waves.len()));
        for (i, wave) in waves.iter().enumerate() {
            if let Some(name) = wave.get("name").and_then(|v| v.as_str()) {
                context.push_str(&format!("  Wave {}: {}\n", i + 1, name));
            }
        }
    }

    // Add wave states if available
    if !checkpoint.wave_states.is_empty() && checkpoint.wave_states != "{}" {
        if let Ok(states) = serde_json::from_str::<Value>(&checkpoint.wave_states) {
            context.push_str(&format!("\nWave States: {}\n", states));
        }
    }

    Ok(context)
}

/// Build the query prompt for the LLM.
fn build_query_prompt(context: &str, question: &str) -> String {
    format!(
        r#"You are a security analysis assistant helping to query campaign data.

## Campaign Context
{}

## User Question
{}

## Instructions
Answer the user's question based on the campaign context provided above.
If the context doesn't contain enough information to answer the question,
say so clearly and suggest what additional information would be needed.

Be concise but thorough. Use bullet points or tables when appropriate for clarity.

## Answer
"#,
        context,
        question
    )
}

/// Extension trait for LLM analyzer to support campaign queries.
impl LlmAnalyzer {
    /// Query the LLM with a custom prompt.
    ///
    /// # Arguments
    /// * `prompt` - The prompt to send to the LLM
    ///
    /// # Returns
    /// * `Ok(String)` - LLM response text
    /// * `Err(OrchestratorError)` - Query failed
    pub async fn query(&self, prompt: &str) -> Result<String> {
        use crate::llm::LlmFinding;

        // Create a synthetic finding to use the existing analyze API
        // This is a workaround - ideally we'd have a direct query method
        let findings = vec![LlmFinding {
            file: "campaign_context".to_string(),
            line: 0,
            severity: "Info".to_string(),
            category: "Query".to_string(),
            description: prompt.to_string(),
            context: Some(prompt.to_string()),
            decoded_payload: None,
        }];

        // Use the existing analyze method
        let verdict = self.analyze(&findings).await
            .map_err(|e| OrchestratorError::llm(format!("LLM analysis failed: {}", e)))?;

        // The verdict explanation contains the response
        Ok(verdict.explanation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_campaign_context() {
        let checkpoint = CampaignCheckpoint {
            case_id: "test-123".to_string(),
            campaign_name: "Test Campaign".to_string(),
            status: "completed".to_string(),
            config_json: r#"{"waves": [{"name": "Wave 1"}, {"name": "Wave 2"}]}"#.to_string(),
            completed_waves: vec!["wave1".to_string(), "wave2".to_string()],
            current_wave: None,
            wave_states: "{}".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T01:00:00Z".to_string(),
        };

        let context = build_campaign_context(&checkpoint).unwrap();

        assert!(context.contains("Test Campaign"));
        assert!(context.contains("test-123"));
        assert!(context.contains("completed"));
        assert!(context.contains("Total Waves: 2"));
    }

    #[test]
    fn test_build_query_prompt() {
        let context = "Campaign: Test\nCase ID: 123";
        let question = "What packages were flagged?";

        let prompt = build_query_prompt(context, question);

        assert!(prompt.contains("Campaign Context"));
        assert!(prompt.contains("Campaign: Test"));
        assert!(prompt.contains("User Question"));
        assert!(prompt.contains("What packages were flagged?"));
        assert!(prompt.contains("Instructions"));
    }
}
