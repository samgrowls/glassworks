//! LLM analysis integration for OpenAI-compatible APIs.
//!
//! This module provides LLM-based analysis of security findings using
//! OpenAI-compatible APIs (Cerebras, Groq, NVIDIA NIM, Ollama, etc.).
//!
//! Features:
//! - OpenAI-compatible API
//! - NVIDIA NIM, Cerebras, Groq support
//! - Batch analysis
//! - Caching of LLM results

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::error::{OrchestratorError, Result};

/// LLM verdict from analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmVerdict {
    /// Whether the finding is likely a true positive.
    pub is_malicious: bool,
    /// Confidence score (0.0-1.0).
    pub confidence: f32,
    /// Explanation of the verdict.
    pub explanation: String,
    /// Recommended actions.
    pub recommendations: Vec<String>,
    /// False positive indicators.
    pub false_positive_indicators: Vec<String>,
}

/// LLM analysis request.
#[derive(Debug, Clone, Serialize)]
pub struct LlmAnalysisRequest {
    /// Findings to analyze.
    pub findings: Vec<LlmFinding>,
    /// Analysis prompt.
    pub prompt: String,
}

/// Finding formatted for LLM analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmFinding {
    /// File path.
    pub file: String,
    /// Line number.
    pub line: usize,
    /// Severity.
    pub severity: String,
    /// Category.
    pub category: String,
    /// Description.
    pub description: String,
    /// Code context.
    pub context: Option<String>,
    /// Decoded payload.
    pub decoded_payload: Option<String>,
}

/// LLM API response.
#[derive(Debug, Clone, Deserialize)]
pub struct LlmApiResponse {
    /// API choices.
    pub choices: Vec<LlmChoice>,
    /// Usage information.
    #[serde(default)]
    pub usage: Option<LlmUsage>,
}

/// LLM choice.
#[derive(Debug, Clone, Deserialize)]
pub struct LlmChoice {
    /// Message content.
    pub message: LlmMessage,
    /// Finish reason.
    #[serde(default)]
    pub finish_reason: Option<String>,
}

/// LLM message.
#[derive(Debug, Clone, Deserialize)]
pub struct LlmMessage {
    /// Message role.
    pub role: String,
    /// Message content.
    pub content: String,
}

/// LLM usage information.
#[derive(Debug, Clone, Deserialize)]
pub struct LlmUsage {
    /// Prompt tokens.
    #[serde(default)]
    pub prompt_tokens: u32,
    /// Completion tokens.
    #[serde(default)]
    pub completion_tokens: u32,
    /// Total tokens.
    #[serde(default)]
    pub total_tokens: u32,
}

/// LLM cache entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCacheEntry {
    /// Input hash for caching.
    pub input_hash: String,
    /// Cached verdict.
    pub verdict: LlmVerdict,
    /// Cache timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Configuration for LLM analyzer.
#[derive(Debug, Clone)]
pub struct LlmAnalyzerConfig {
    /// Base URL for the LLM API.
    pub base_url: String,
    /// API key.
    pub api_key: String,
    /// Model name.
    pub model: String,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
    /// Maximum tokens for response.
    pub max_tokens: u32,
    /// Temperature for generation.
    pub temperature: f32,
    /// Enable caching.
    pub enable_cache: bool,
    /// Cache directory path.
    pub cache_dir: Option<String>,
}

impl Default for LlmAnalyzerConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.cerebras.ai/v1".to_string(),
            api_key: String::new(),
            model: "llama-3.3-70b".to_string(),
            timeout_secs: 30,
            max_tokens: 1024,
            temperature: 0.1, // Low temperature for consistent analysis
            enable_cache: true,
            cache_dir: None,
        }
    }
}

impl LlmAnalyzerConfig {
    /// Create config for Cerebras API.
    pub fn cerebras(api_key: String) -> Self {
        Self {
            base_url: "https://api.cerebras.ai/v1".to_string(),
            api_key,
            model: "llama-3.3-70b".to_string(),
            ..Default::default()
        }
    }

    /// Create config for Groq API.
    pub fn groq(api_key: String) -> Self {
        Self {
            base_url: "https://api.groq.com/openai/v1".to_string(),
            api_key,
            model: "llama-3.3-70b-versatile".to_string(),
            ..Default::default()
        }
    }

    /// Create config for NVIDIA NIM API.
    pub fn nvidia_nim(api_key: String) -> Self {
        Self {
            base_url: "https://integrate.api.nvidia.com/v1".to_string(),
            api_key,
            model: "meta/llama-3.3-70b-instruct".to_string(),
            ..Default::default()
        }
    }

    /// Create config for OpenAI API.
    pub fn openai(api_key: String) -> Self {
        Self {
            base_url: "https://api.openai.com/v1".to_string(),
            api_key,
            model: "gpt-4o".to_string(),
            ..Default::default()
        }
    }

    /// Create config for local Ollama.
    pub fn ollama(model: String) -> Self {
        Self {
            base_url: "http://localhost:11434/v1".to_string(),
            api_key: String::new(), // Ollama doesn't require API key by default
            model,
            ..Default::default()
        }
    }
}

/// LLM analyzer for security findings.
pub struct LlmAnalyzer {
    client: Client,
    config: LlmAnalyzerConfig,
    cache: Arc<Mutex<HashMap<String, LlmCacheEntry>>>,
}

impl LlmAnalyzer {
    /// Create a new LLM analyzer with default configuration.
    pub fn new() -> Result<Self> {
        Self::with_config(LlmAnalyzerConfig::default())
    }

    /// Create a new LLM analyzer with custom configuration.
    pub fn with_config(config: LlmAnalyzerConfig) -> Result<Self> {
        if config.api_key.is_empty() && !config.base_url.contains("localhost") {
            warn!("LLM API key is empty. API calls will fail.");
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| OrchestratorError::http_error(e))?;

        let cache = Arc::new(Mutex::new(HashMap::new()));
        
        let analyzer = Self {
            client,
            config,
            cache,
        };

        // Note: Cache loading would need to be async or done separately
        // For now, we skip disk cache loading in the constructor

        Ok(analyzer)
    }

    /// Analyze a single finding.
    pub async fn analyze_finding(&self, finding: &glassware_core::Finding) -> Result<LlmVerdict> {
        let findings = vec![LlmFinding::from(finding)];
        self.analyze(&findings).await
    }

    /// Analyze multiple findings.
    pub async fn analyze(&self, findings: &[LlmFinding]) -> Result<LlmVerdict> {
        if findings.is_empty() {
            return Err(OrchestratorError::config_error("No findings to analyze".to_string()));
        }

        // Check cache
        let cache_key = self.compute_cache_key(findings);
        {
            let cache = self.cache.lock().await;
            if let Some(entry) = cache.get(&cache_key) {
                debug!("LLM cache hit");
                return Ok(entry.verdict.clone());
            }
        }

        // Build prompt
        let prompt = self.build_prompt(findings);

        // Create API request
        let request = self.create_request(&prompt);

        // Make API call
        let response = self.call_api(&request).await?;

        // Parse response
        let verdict = self.parse_response(&response, findings)?;

        // Cache result
        if self.config.enable_cache {
            let mut cache = self.cache.lock().await;
            let cache_key_clone = cache_key.clone();
            cache.insert(cache_key_clone, LlmCacheEntry {
                input_hash: cache_key,
                verdict: verdict.clone(),
                timestamp: chrono::Utc::now(),
            });
        }

        Ok(verdict)
    }

    /// Analyze findings in batch.
    pub async fn analyze_batch(&self, batches: &[Vec<LlmFinding>]) -> Result<Vec<LlmVerdict>> {
        let mut results = Vec::new();

        for (i, batch) in batches.iter().enumerate() {
            debug!("Analyzing batch {}/{}", i + 1, batches.len());
            match self.analyze(batch).await {
                Ok(verdict) => results.push(verdict),
                Err(e) => {
                    error!("Failed to analyze batch {}: {}", i + 1, e);
                    results.push(LlmVerdict {
                        is_malicious: false,
                        confidence: 0.0,
                        explanation: format!("Analysis failed: {}", e),
                        recommendations: vec![],
                        false_positive_indicators: vec![],
                    });
                }
            }
        }

        Ok(results)
    }

    /// Build analysis prompt.
    fn build_prompt(&self, findings: &[LlmFinding]) -> String {
        let mut prompt = String::from(
            r#"You are a security expert analyzing code for malicious patterns.
Your task is to determine if the detected patterns are intentional attacks or false positives.

Consider:
1. Context and intent of the code
2. Common false positive patterns
3. Legitimate use cases for the detected patterns
4. Overall security implications

"#,
        );

        prompt.push_str("## Findings to Analyze:\n\n");

        for (i, finding) in findings.iter().enumerate() {
            prompt.push_str(&format!("### Finding {}\n", i + 1));
            prompt.push_str(&format!("- **File**: {}\n", finding.file));
            prompt.push_str(&format!("- **Line**: {}\n", finding.line));
            prompt.push_str(&format!("- **Severity**: {}\n", finding.severity));
            prompt.push_str(&format!("- **Category**: {}\n", finding.category));
            prompt.push_str(&format!("- **Description**: {}\n", finding.description));

            if let Some(ref context) = finding.context {
                prompt.push_str(&format!("- **Code Context**:\n```\n{}\n```\n", context));
            }

            if let Some(ref payload) = finding.decoded_payload {
                prompt.push_str(&format!("- **Decoded Payload**: {}\n", payload));
            }

            prompt.push('\n');
        }

        prompt.push_str(
            r#"## Response Format

Provide your analysis in the following JSON format:

```json
{
  "is_malicious": true/false,
  "confidence": 0.0-1.0,
  "explanation": "Detailed explanation of your verdict",
  "recommendations": ["list", "of", "recommended", "actions"],
  "false_positive_indicators": ["list", "of", "fp", "indicators"]
}
```

Be concise but thorough. Focus on actionable insights."#,
        );

        prompt
    }

    /// Create API request body.
    fn create_request(&self, prompt: &str) -> serde_json::Value {
        serde_json::json!({
            "model": self.config.model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a security expert analyzing code for malicious patterns. Respond with JSON only."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "response_format": { "type": "json_object" }
        })
    }

    /// Call LLM API.
    async fn call_api(&self, request: &serde_json::Value) -> Result<LlmApiResponse> {
        let url = format!("{}/chat/completions", self.config.base_url);

        let mut req_builder = self.client
            .post(&url)
            .json(request)
            .header("Content-Type", "application/json");

        // Add API key if provided
        if !self.config.api_key.is_empty() {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", self.config.api_key));
        }

        let response = req_builder
            .send()
            .await
            .map_err(|e| OrchestratorError::http_error(e))?;

        if response.status() == StatusCode::UNAUTHORIZED {
            return Err(OrchestratorError::config_error("Invalid API key".to_string()));
        }

        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            return Err(OrchestratorError::RateLimitExceeded { retry_after: 60 });
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(OrchestratorError::GitHub(format!(
                "LLM API error {}: {}",
                status, body
            )));
        }

        let api_response: LlmApiResponse = response
            .json()
            .await
            .map_err(|e| OrchestratorError::http_error(e))?;

        if api_response.choices.is_empty() {
            return Err(OrchestratorError::config_error("LLM returned no choices".to_string()));
        }

        Ok(api_response)
    }

    /// Parse LLM response into verdict.
    fn parse_response(&self, response: &LlmApiResponse, _findings: &[LlmFinding]) -> Result<LlmVerdict> {
        let content = &response.choices[0].message.content;

        // Try to parse JSON from response
        let verdict: LlmVerdict = serde_json::from_str(content).unwrap_or_else(|e| {
            warn!("Failed to parse LLM JSON response: {}", e);
            // Fallback: create a basic verdict from the raw response
            LlmVerdict {
                is_malicious: content.contains("malicious") || content.contains("true"),
                confidence: 0.5,
                explanation: content.clone(),
                recommendations: vec!["Manual review recommended".to_string()],
                false_positive_indicators: vec![],
            }
        });

        info!(
            "LLM verdict: malicious={}, confidence={:.2}",
            verdict.is_malicious, verdict.confidence
        );

        Ok(verdict)
    }

    /// Compute cache key for findings.
    fn compute_cache_key(&self, findings: &[LlmFinding]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for finding in findings {
            finding.file.hash(&mut hasher);
            finding.line.hash(&mut hasher);
            finding.category.hash(&mut hasher);
            finding.description.hash(&mut hasher);
        }

        format!("{:x}", hasher.finish())
    }

    /// Get cache file path.
    fn get_cache_path(&self) -> std::path::PathBuf {
        if let Some(ref dir) = self.config.cache_dir {
            std::path::Path::new(dir).join("llm_cache.json")
        } else {
            // Default to current directory
            std::path::Path::new(".glassware-llm-cache.json").to_path_buf()
        }
    }

    /// Clear the cache.
    pub async fn clear_cache(&self) -> Result<()> {
        let mut cache = self.cache.lock().await;
        cache.clear();

        let cache_path = self.get_cache_path();
        if cache_path.exists() {
            std::fs::remove_file(&cache_path).map_err(|e| {
                OrchestratorError::io_error(e)
            })?;
        }

        info!("Cleared LLM cache");
        Ok(())
    }

    /// Get cache size.
    pub async fn cache_size(&self) -> usize {
        let cache = self.cache.lock().await;
        cache.len()
    }
}

impl Default for LlmAnalyzer {
    fn default() -> Self {
        Self::new().expect("Failed to create default LLM analyzer")
    }
}

impl From<&glassware_core::Finding> for LlmFinding {
    fn from(finding: &glassware_core::Finding) -> Self {
        Self {
            file: finding.file.clone(),
            line: finding.line,
            severity: format!("{:?}", finding.severity),
            category: format!("{:?}", finding.category),
            description: finding.description.clone(),
            context: finding.context.clone(),
            decoded_payload: finding.decoded_payload.as_ref().map(|p| {
                // Convert DecodedPayload to string representation
                if let Some(ref text) = p.decoded_text {
                    text.clone()
                } else {
                    format!("[Binary payload, entropy: {:.2}]", p.entropy)
                }
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glassware_core::{Finding, Severity, DetectionCategory};

    fn create_test_finding() -> Finding {
        Finding {
            file: "test.js".to_string(),
            line: 1,
            column: 5,
            code_point: 0xFE00,
            character: "\u{FE00}".to_string(),
            raw_bytes: None,
            severity: Severity::High,
            category: DetectionCategory::InvisibleCharacter,
            description: "Invisible character detected".to_string(),
            remediation: "Remove it".to_string(),
            cwe_id: None,
            references: vec![],
            context: Some("const x = 1;".to_string()),
            decoded_payload: None,
            confidence: None,
        }
    }

    #[test]
    fn test_llm_analyzer_config_default() {
        let config = LlmAnalyzerConfig::default();
        assert_eq!(config.base_url, "https://api.cerebras.ai/v1");
        assert_eq!(config.model, "llama-3.3-70b");
        assert!(config.api_key.is_empty());
    }

    #[test]
    fn test_llm_analyzer_config_presets() {
        let cerebras = LlmAnalyzerConfig::cerebras("key".to_string());
        assert!(cerebras.base_url.contains("cerebras"));

        let groq = LlmAnalyzerConfig::groq("key".to_string());
        assert!(groq.base_url.contains("groq"));

        let nvidia = LlmAnalyzerConfig::nvidia_nim("key".to_string());
        assert!(nvidia.base_url.contains("nvidia"));

        let openai = LlmAnalyzerConfig::openai("key".to_string());
        assert!(openai.base_url.contains("openai"));

        let ollama = LlmAnalyzerConfig::ollama("llama3".to_string());
        assert!(ollama.base_url.contains("localhost"));
        assert_eq!(ollama.model, "llama3");
    }

    #[test]
    fn test_llm_finding_from_finding() {
        let finding = create_test_finding();
        let llm_finding = LlmFinding::from(&finding);

        assert_eq!(llm_finding.file, "test.js");
        assert_eq!(llm_finding.severity, "High");
        assert_eq!(llm_finding.category, "InvisibleCharacter");
    }

    #[test]
    fn test_llm_analyzer_creation() {
        // Test with empty API key (should warn but succeed)
        let config = LlmAnalyzerConfig::default();
        let analyzer = LlmAnalyzer::with_config(config);
        assert!(analyzer.is_ok());
    }

    #[test]
    fn test_llm_analyzer_cache_key() {
        let analyzer = LlmAnalyzer::new().unwrap();
        let findings = vec![LlmFinding {
            file: "test.js".to_string(),
            line: 1,
            severity: "High".to_string(),
            category: "Test".to_string(),
            description: "Test".to_string(),
            context: None,
            decoded_payload: None,
        }];

        let key1 = analyzer.compute_cache_key(&findings);
        let key2 = analyzer.compute_cache_key(&findings);

        assert_eq!(key1, key2);
    }

    #[test]
    fn test_llm_verdict_serialization() {
        let verdict = LlmVerdict {
            is_malicious: true,
            confidence: 0.95,
            explanation: "Test explanation".to_string(),
            recommendations: vec!["Action 1".to_string()],
            false_positive_indicators: vec!["FP indicator".to_string()],
        };

        let json = serde_json::to_string(&verdict).unwrap();
        assert!(json.contains("is_malicious"));
        assert!(json.contains("true"));

        let parsed: LlmVerdict = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_malicious);
        assert_eq!(parsed.confidence, 0.95);
    }

    #[test]
    fn test_llm_analyzer_prompt_building() {
        let analyzer = LlmAnalyzer::new().unwrap();
        let findings = vec![LlmFinding {
            file: "test.js".to_string(),
            line: 1,
            severity: "High".to_string(),
            category: "Test".to_string(),
            description: "Test description".to_string(),
            context: Some("code context".to_string()),
            decoded_payload: None,
        }];

        let prompt = analyzer.build_prompt(&findings);
        assert!(prompt.contains("test.js"));
        assert!(prompt.contains("Test description"));
        assert!(prompt.contains("code context"));
    }
}
