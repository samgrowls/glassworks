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
//! - Multi-stage pipeline (Phase 5 - 2026-03-24)

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::error::{OrchestratorError, Result};

// Re-export core types for convenience
pub use glassware_core::Finding as CoreFinding;

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
    /// Model name (for backwards compatibility - use models for multiple).
    pub model: String,
    /// List of models to try in order (for fallback).
    pub models: Vec<String>,
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
            // Default to NVIDIA for deep analysis (orchestrator use case)
            base_url: "https://integrate.api.nvidia.com/v1".to_string(),
            api_key: String::new(),
            model: "meta/llama-3.3-70b-instruct".to_string(),
            // NVIDIA models with fallback for orchestrator deep analysis
            models: vec![
                "qwen/qwen3.5-397b-a17b".to_string(),  // Strongest - 397B
                "moonshotai/kimi-k2.5".to_string(),     // Kimi K2.5
                "z-ai/glm5".to_string(),                // GLM-5
                "meta/llama-3.3-70b-instruct".to_string(), // Fallback - 70B
            ],
            timeout_secs: 120,  // Longer timeout for deep analysis
            max_tokens: 2048,   // More tokens for detailed analysis
            temperature: 0.1,
            enable_cache: true,
            cache_dir: None,
        }
    }
}

impl LlmAnalyzerConfig {
    /// Create config from environment variables.
    /// Reads: GLASSWARE_LLM_BASE_URL, GLASSWARE_LLM_API_KEY, GLASSWARE_LLM_MODELS (or GLASSWARE_LLM_MODEL)
    /// 
    /// Default behavior (no env vars): NVIDIA models with fallback (orchestrator deep analysis)
    /// Override with GLASSWARE_LLM_BASE_URL for custom provider (e.g., Cerebras for CLI triage)
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var("GLASSWARE_LLM_BASE_URL").ok()?;
        let api_key = std::env::var("GLASSWARE_LLM_API_KEY").ok()?;
        
        // Try GLASSWARE_LLM_MODELS first (comma-separated), then fall back to GLASSWARE_LLM_MODEL
        let models = if let Ok(models_str) = std::env::var("GLASSWARE_LLM_MODELS") {
            models_str.split(',').map(|s| s.trim().to_string()).collect()
        } else if let Ok(model) = std::env::var("GLASSWARE_LLM_MODEL") {
            vec![model]
        } else {
            // Default to NVIDIA models with fallback for orchestrator
            vec![
                "qwen/qwen3.5-397b-a17b".to_string(),
                "moonshotai/kimi-k2.5".to_string(),
                "z-ai/glm5".to_string(),
                "meta/llama-3.3-70b-instruct".to_string(),
            ]
        };

        Some(Self {
            base_url,
            api_key,
            model: models.first().cloned().unwrap_or_else(|| "meta/llama-3.3-70b-instruct".to_string()),
            models,
            ..Default::default()
        })
    }

    /// Create config for Cerebras API (fast triage).
    pub fn cerebras(api_key: String) -> Self {
        Self {
            base_url: "https://api.cerebras.ai/v1".to_string(),
            api_key,
            model: "llama-3.3-70b".to_string(),
            // Single fast model for triage
            models: vec!["llama-3.3-70b".to_string()],
            timeout_secs: 30,  // Fast timeout for triage
            ..Default::default()
        }
    }

    /// Create config for NVIDIA API (deep analysis with model fallback).
    /// Uses NVIDIA_API_KEY environment variable.
    /// Models in fallback order: Qwen 3.5 (397B) → Kimi K2.5 → GLM-5 → Llama 3.3 (70B)
    pub fn nvidia_deep_analysis() -> Option<Self> {
        let api_key = std::env::var("NVIDIA_API_KEY").ok()?;
        
        Some(Self {
            base_url: "https://integrate.api.nvidia.com/v1".to_string(),
            api_key,
            model: "qwen/qwen3.5-397b-a17b".to_string(),
            // NVIDIA models in order of preference (strongest first)
            models: vec![
                "qwen/qwen3.5-397b-a17b".to_string(),  // Strongest - 397B
                "moonshotai/kimi-k2.5".to_string(),     // Kimi K2.5
                "z-ai/glm5".to_string(),                // GLM-5
                "meta/llama-3.3-70b-instruct".to_string(), // Fallback - 70B
            ],
            timeout_secs: 120,  // Longer timeout for deep analysis
            max_tokens: 2048,   // More tokens for detailed analysis
            temperature: 0.1,
            enable_cache: true,
            cache_dir: None,
        })
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

    /// Create config for NVIDIA NIM API with model fallback.
    pub fn nvidia_nim(api_key: String) -> Self {
        Self {
            base_url: "https://integrate.api.nvidia.com/v1".to_string(),
            api_key,
            model: "meta/llama-3.3-70b-instruct".to_string(),
            // NVIDIA models in order of preference (strongest first)
            models: vec![
                "qwen/qwen3.5-397b-a17b".to_string(),  // Strongest - 397B
                "moonshotai/kimi-k2.5".to_string(),     // Kimi K2.5
                "z-ai/glm5".to_string(),                // GLM-5
                "meta/llama-3.3-70b-instruct".to_string(), // Fallback - 70B
            ],
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
            .map_err(|e| OrchestratorError::http(e))?;

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

    /// Get the analyzer configuration.
    pub fn config(&self) -> &LlmAnalyzerConfig {
        &self.config
    }

    /// Analyze a single finding.
    pub async fn analyze_finding(&self, finding: &glassware_core::Finding) -> Result<LlmVerdict> {
        let findings = vec![LlmFinding::from(finding)];
        self.analyze(&findings).await
    }

    /// Analyze multiple findings with model fallback.
    /// Tries models in order until one succeeds.
    pub async fn analyze_with_fallback(&self, findings: &[LlmFinding]) -> Result<LlmVerdict> {
        if findings.is_empty() {
            return Err(OrchestratorError::config_error("No findings to analyze".to_string()));
        }

        let mut last_error: Option<OrchestratorError> = None;

        // Try each model in order
        for (i, model) in self.config.models.iter().enumerate() {
            debug!("Attempting LLM analysis with model {}/{}: {}", i + 1, self.config.models.len(), model);
            
            // Temporarily override the model for this attempt
            let mut config_with_model = self.config.clone();
            config_with_model.model = model.clone();
            
            match self.analyze_with_model(findings, &config_with_model).await {
                Ok(verdict) => {
                    info!("LLM analysis succeeded with model: {}", model);
                    return Ok(verdict);
                }
                Err(e) => {
                    warn!("LLM analysis failed with model {}: {}", model, e);
                    last_error = Some(e);
                    // Continue to next model
                }
            }
        }

        // All models failed
        Err(last_error.unwrap_or_else(|| {
            OrchestratorError::llm("All LLM models failed".to_string())
        }))
    }

    /// Analyze with a specific model (internal helper for fallback).
    async fn analyze_with_model(&self, findings: &[LlmFinding], config: &LlmAnalyzerConfig) -> Result<LlmVerdict> {
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

        // Create API request with specific model
        let request = self.create_request_with_model(&prompt, &config.model);

        // Make API call
        let response = self.call_api(&request).await?;

        // Parse response
        let verdict = self.parse_response(&response, findings)?;

        // Cache result
        if self.config.enable_cache {
            let mut cache = self.cache.lock().await;
            cache.insert(cache_key.clone(), LlmCacheEntry {
                input_hash: cache_key,
                verdict: verdict.clone(),
                timestamp: chrono::Utc::now(),
            });
        }

        Ok(verdict)
    }

    /// Analyze multiple findings.
    pub async fn analyze(&self, findings: &[LlmFinding]) -> Result<LlmVerdict> {
        // Use fallback method by default
        self.analyze_with_fallback(findings).await
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
            r#"You are a security expert specializing in detecting false positives in static analysis tools.
Your task is to review detected patterns and determine if they are REAL threats or FALSE POSITIVES.

## Critical Guidelines

**FALSE POSITIVE Indicators (likely NOT malicious):**
1. **Legitimate libraries**: i18n packages (moment, date-fns, intl), crypto libraries (ethers, web3), build tools (webpack, vite)
2. **Minified/bundled code**: Short variable names, no whitespace, build artifacts
3. **Framework code**: Angular, React, Vue core packages - these are legitimate frameworks
4. **Unicode in locale data**: U+FFFD, variation selectors in .json files for i18n
5. **Standard patterns**: codePointAt, fromCharCode in legitimate decoder libraries
6. **Blockchain APIs**: RPC calls in @ethersproject, web3, viem are legitimate crypto functionality
7. **Eval in bundlers**: webpack, esbuild, rollup use Function/eval for legitimate purposes

**REAL THREAT Indicators (likely malicious):**
1. **Steganographic payloads**: Hidden Unicode payloads that decode to executable code
2. **Decrypt → Execute chains**: Encrypted data that decrypts and immediately executes
3. **C2 communication**: Hidden communication to external servers (not legitimate APIs)
4. **Evasion techniques**: Time delays, sandbox detection, geofencing to avoid analysis
5. **Obfuscated malicious logic**: Code that hides its true intent through obfuscation

**Key Questions to Ask:**
1. Is this a well-known legitimate package? (check package name)
2. Is the pattern in minified/bundled code? (likely FP)
3. Is this standard functionality for this package type? (e.g., crypto lib doing crypto)
4. Does the code actually execute maliciously or just contain patterns?
5. Is there a clear attack chain or just isolated patterns?

**Confidence Guidelines:**
- **0.0-0.25**: Very likely FALSE POSITIVE (legitimate package, standard patterns)
- **0.25-0.50**: Likely FALSE POSITIVE (some concerning patterns but context suggests legitimate)
- **0.50-0.75**: Uncertain (mixed signals, needs human review)
- **0.75-1.0**: Likely MALICIOUS (clear attack chain, evasion, or steganographic payload)

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
        self.create_request_with_model(prompt, &self.config.model)
    }

    /// Create API request body with specific model.
    fn create_request_with_model(&self, prompt: &str, model: &str) -> serde_json::Value {
        serde_json::json!({
            "model": model,
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
            .map_err(|e| OrchestratorError::http(e))?;

        if response.status() == StatusCode::UNAUTHORIZED {
            return Err(OrchestratorError::config_error("Invalid API key".to_string()));
        }

        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            return Err(OrchestratorError::RateLimitExceeded { 
                retry_after: 60,
                context: crate::error::ErrorContext::new(),
            });
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(OrchestratorError::github(format!(
                "LLM API error {}: {}",
                status, body
            )));
        }

        let api_response: LlmApiResponse = response
            .json()
            .await
            .map_err(|e| OrchestratorError::http(e))?;

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
                OrchestratorError::io(e)
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

// ============================================================================
// Phase 5: Multi-Stage LLM Pipeline (2026-03-24)
// ============================================================================

/// Configuration for multi-stage LLM pipeline.
#[derive(Debug, Clone)]
pub struct MultiStagePipelineConfig {
    /// Stage 1: Triage config (Cerebras - fast, ~2s/pkg)
    pub triage_config: Option<LlmAnalyzerConfig>,
    /// Stage 2: Analysis config (NVIDIA - medium, ~15s/pkg)
    pub analysis_config: Option<LlmAnalyzerConfig>,
    /// Stage 3: Deep dive config (NVIDIA - slow, ~30s/pkg for borderline)
    pub deep_dive_config: Option<LlmAnalyzerConfig>,
    /// Score threshold for deep dive (e.g., 4.0-7.0 = borderline)
    pub deep_dive_score_threshold_min: f32,
    /// Score threshold for deep dive (e.g., 4.0-7.0 = borderline)
    pub deep_dive_score_threshold_max: f32,
    /// Enable triage stage
    pub triage_enabled: bool,
    /// Enable analysis stage
    pub analysis_enabled: bool,
    /// Enable deep dive stage
    pub deep_dive_enabled: bool,
    /// Cache results
    pub cache_enabled: bool,
}

impl Default for MultiStagePipelineConfig {
    fn default() -> Self {
        Self {
            // Stage 1: Cerebras for fast triage
            triage_config: LlmAnalyzerConfig::from_env().map(|mut cfg| {
                cfg.timeout_secs = 30;
                cfg.max_tokens = 512;
                cfg
            }),
            // Stage 2: NVIDIA for analysis
            analysis_config: LlmAnalyzerConfig::nvidia_deep_analysis(),
            // Stage 3: NVIDIA for deep dive (same config, different prompt)
            deep_dive_config: LlmAnalyzerConfig::nvidia_deep_analysis(),
            deep_dive_score_threshold_min: 4.0,
            deep_dive_score_threshold_max: 7.0,
            triage_enabled: true,
            analysis_enabled: true,
            deep_dive_enabled: true,
            cache_enabled: true,
        }
    }
}

impl MultiStagePipelineConfig {
    /// Create config from environment variables.
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Override with environment-based configs
        if let Ok(cerebras_key) = std::env::var("GLASSWARE_LLM_API_KEY") {
            config.triage_config = Some(LlmAnalyzerConfig::cerebras(cerebras_key));
        }

        config
    }

    /// Create config with only triage stage (fast scanning).
    pub fn triage_only() -> Self {
        Self {
            triage_config: LlmAnalyzerConfig::from_env(),
            analysis_config: None,
            deep_dive_config: None,
            triage_enabled: true,
            analysis_enabled: false,
            deep_dive_enabled: false,
            cache_enabled: true,
            deep_dive_score_threshold_min: 4.0,
            deep_dive_score_threshold_max: 7.0,
        }
    }

    /// Create config with triage + analysis (standard scanning).
    pub fn standard() -> Self {
        Self {
            triage_config: LlmAnalyzerConfig::from_env(),
            analysis_config: LlmAnalyzerConfig::nvidia_deep_analysis(),
            deep_dive_config: None,
            triage_enabled: true,
            analysis_enabled: true,
            deep_dive_enabled: false,
            cache_enabled: true,
            deep_dive_score_threshold_min: 4.0,
            deep_dive_score_threshold_max: 7.0,
        }
    }

    /// Create config with all stages (deep scanning for borderline cases).
    pub fn deep_scan() -> Self {
        Self {
            triage_config: LlmAnalyzerConfig::from_env(),
            analysis_config: LlmAnalyzerConfig::nvidia_deep_analysis(),
            deep_dive_config: LlmAnalyzerConfig::nvidia_deep_analysis(),
            triage_enabled: true,
            analysis_enabled: true,
            deep_dive_enabled: true,
            cache_enabled: true,
            deep_dive_score_threshold_min: 4.0,
            deep_dive_score_threshold_max: 7.0,
        }
    }
}

/// Result from multi-stage LLM pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    /// Final verdict.
    pub verdict: LlmVerdict,
    /// Stage that produced the verdict (1=triage, 2=analysis, 3=deep dive).
    pub stage: u8,
    /// Whether triage was run.
    pub triage_run: bool,
    /// Whether analysis was run.
    pub analysis_run: bool,
    /// Whether deep dive was run.
    pub deep_dive_run: bool,
    /// Triage verdict (if run).
    pub triage_verdict: Option<LlmVerdict>,
    /// Analysis verdict (if run).
    pub analysis_verdict: Option<LlmVerdict>,
    /// Deep dive verdict (if run).
    pub deep_dive_verdict: Option<LlmVerdict>,
    /// Total time spent in milliseconds.
    pub total_time_ms: u64,
    /// Models used at each stage.
    pub models_used: Vec<String>,
}

/// Multi-stage LLM pipeline for security analysis.
///
/// Implements the Phase 5 multi-stage pipeline from PROMPT.md:
/// - Stage 1: Triage (Cerebras - fast, ~2s/pkg) - Identify obvious FPs
/// - Stage 2: Analysis (NVIDIA - medium, ~15s/pkg) - Explain attack chain
/// - Stage 3: Deep Dive (NVIDIA - slow, ~30s/pkg) - Borderline cases
pub struct MultiStagePipeline {
    triage_analyzer: Option<LlmAnalyzer>,
    analysis_analyzer: Option<LlmAnalyzer>,
    deep_dive_analyzer: Option<LlmAnalyzer>,
    config: MultiStagePipelineConfig,
}

impl MultiStagePipeline {
    /// Create a new multi-stage pipeline with default config.
    pub fn new() -> Result<Self> {
        Self::with_config(MultiStagePipelineConfig::default())
    }

    /// Create a new multi-stage pipeline with custom config.
    pub fn with_config(config: MultiStagePipelineConfig) -> Result<Self> {
        let triage_analyzer = config.triage_config.as_ref()
            .filter(|_| config.triage_enabled)
            .map(|cfg| LlmAnalyzer::with_config(cfg.clone()))
            .transpose()?;

        let analysis_analyzer = config.analysis_config.as_ref()
            .filter(|_| config.analysis_enabled)
            .map(|cfg| LlmAnalyzer::with_config(cfg.clone()))
            .transpose()?;

        let deep_dive_analyzer = config.deep_dive_config.as_ref()
            .filter(|_| config.deep_dive_enabled)
            .map(|cfg| LlmAnalyzer::with_config(cfg.clone()))
            .transpose()?;

        Ok(Self {
            triage_analyzer,
            analysis_analyzer,
            deep_dive_analyzer,
            config,
        })
    }

    /// Run the multi-stage pipeline on findings.
    ///
    /// Returns a PipelineResult with the final verdict and stage information.
    pub async fn run(&self, findings: &[LlmFinding], threat_score: f32) -> Result<PipelineResult> {
        let start_time = std::time::Instant::now();
        let mut models_used = Vec::new();

        let mut triage_verdict: Option<LlmVerdict> = None;
        let mut analysis_verdict: Option<LlmVerdict> = None;
        let mut deep_dive_verdict: Option<LlmVerdict> = None;

        // Stage 1: Triage (fast FP identification)
        let triage_run = self.triage_analyzer.is_some();
        if let Some(ref triage) = self.triage_analyzer {
            info!("Stage 1: Running triage analysis (fast FP identification)");
            match triage.analyze(findings).await {
                Ok(verdict) => {
                    debug!("Triage verdict: is_malicious={}, confidence={:.2}", verdict.is_malicious, verdict.confidence);
                    
                    // Record model used
                    if let Some(model) = triage.config().models.first() {
                        models_used.push(format!("triage:{}", model));
                    }

                    // If triage says very likely FP (confidence < 0.25), skip further analysis
                    if verdict.confidence < 0.25 {
                        info!("Triage identified likely FP (confidence {:.2}), skipping further analysis", verdict.confidence);
                        triage_verdict = Some(verdict.clone());
                        
                        return Ok(PipelineResult {
                            verdict,
                            stage: 1,
                            triage_run,
                            analysis_run: false,
                            deep_dive_run: false,
                            triage_verdict,
                            analysis_verdict: None,
                            deep_dive_verdict: None,
                            total_time_ms: start_time.elapsed().as_millis() as u64,
                            models_used,
                        });
                    }

                    triage_verdict = Some(verdict);
                }
                Err(e) => {
                    warn!("Triage analysis failed: {}", e);
                    // Continue to Stage 2 even if triage fails
                }
            }
        }

        // Stage 2: Analysis (attack chain explanation)
        let analysis_run = self.analysis_analyzer.is_some();
        if let Some(ref analysis) = self.analysis_analyzer {
            info!("Stage 2: Running analysis (attack chain explanation)");
            match analysis.analyze(findings).await {
                Ok(verdict) => {
                    debug!("Analysis verdict: is_malicious={}, confidence={:.2}", verdict.is_malicious, verdict.confidence);
                    
                    // Record model used
                    if let Some(model) = analysis.config().models.first() {
                        models_used.push(format!("analysis:{}", model));
                    }

                    // If analysis is confident (>= 0.75), skip deep dive
                    if verdict.confidence >= 0.75 {
                        info!("Analysis confident (confidence {:.2}), skipping deep dive", verdict.confidence);
                        analysis_verdict = Some(verdict.clone());
                        
                        return Ok(PipelineResult {
                            verdict,
                            stage: 2,
                            triage_run,
                            analysis_run,
                            deep_dive_run: false,
                            triage_verdict,
                            analysis_verdict,
                            deep_dive_verdict: None,
                            total_time_ms: start_time.elapsed().as_millis() as u64,
                            models_used,
                        });
                    }

                    analysis_verdict = Some(verdict);
                }
                Err(e) => {
                    warn!("Analysis failed: {}", e);
                    // Continue to Stage 3 if deep dive is enabled
                }
            }
        }

        // Stage 3: Deep Dive (borderline cases only)
        let deep_dive_run = self.deep_dive_analyzer.is_some() 
            && threat_score >= self.config.deep_dive_score_threshold_min
            && threat_score <= self.config.deep_dive_score_threshold_max;

        if let Some(ref deep_dive) = self.deep_dive_analyzer {
            if deep_dive_run {
                info!("Stage 3: Running deep dive (borderline case: score {:.2})", threat_score);
                match deep_dive.analyze(findings).await {
                    Ok(verdict) => {
                        debug!("Deep dive verdict: is_malicious={}, confidence={:.2}", verdict.is_malicious, verdict.confidence);
                        
                        // Record model used
                        if let Some(model) = deep_dive.config().models.first() {
                            models_used.push(format!("deep_dive:{}", model));
                        }

                        deep_dive_verdict = Some(verdict.clone());
                        
                        return Ok(PipelineResult {
                            verdict,
                            stage: 3,
                            triage_run,
                            analysis_run,
                            deep_dive_run: true,
                            triage_verdict,
                            analysis_verdict,
                            deep_dive_verdict,
                            total_time_ms: start_time.elapsed().as_millis() as u64,
                            models_used,
                        });
                    }
                    Err(e) => {
                        warn!("Deep dive failed: {}", e);
                        // Fall back to analysis verdict if available
                    }
                }
            }
        }

        // Return best available verdict
        let verdict = deep_dive_verdict.clone()
            .or_else(|| analysis_verdict.clone())
            .or_else(|| triage_verdict.clone())
            .unwrap_or_else(|| LlmVerdict {
                is_malicious: false,
                confidence: 0.0,
                explanation: "All LLM stages failed".to_string(),
                recommendations: vec!["Manual review required".to_string()],
                false_positive_indicators: vec![],
            });

        let stage = if deep_dive_verdict.is_some() { 3 } 
            else if analysis_verdict.is_some() { 2 } 
            else if triage_verdict.is_some() { 1 } 
            else { 0 };

        Ok(PipelineResult {
            verdict,
            stage,
            triage_run,
            analysis_run,
            deep_dive_run,
            triage_verdict,
            analysis_verdict,
            deep_dive_verdict,
            total_time_ms: start_time.elapsed().as_millis() as u64,
            models_used,
        })
    }

    /// Run pipeline without triage (analysis + deep dive only).
    pub async fn run_analysis_only(&self, findings: &[LlmFinding], threat_score: f32) -> Result<PipelineResult> {
        // Just call run - config controls which stages are enabled
        // If triage is disabled in config, it will be skipped
        self.run(findings, threat_score).await
    }

    /// Get pipeline configuration.
    pub fn config(&self) -> &MultiStagePipelineConfig {
        &self.config
    }
}

/// Builder for multi-stage pipeline configuration.
pub struct PipelineBuilder {
    config: MultiStagePipelineConfig,
}

impl PipelineBuilder {
    /// Create a new pipeline builder.
    pub fn new() -> Self {
        Self {
            config: MultiStagePipelineConfig::default(),
        }
    }

    /// Enable/disable triage stage.
    pub fn with_triage(mut self, enabled: bool) -> Self {
        self.config.triage_enabled = enabled;
        self
    }

    /// Enable/disable analysis stage.
    pub fn with_analysis(mut self, enabled: bool) -> Self {
        self.config.analysis_enabled = enabled;
        self
    }

    /// Enable/disable deep dive stage.
    pub fn with_deep_dive(mut self, enabled: bool) -> Self {
        self.config.deep_dive_enabled = enabled;
        self
    }

    /// Set triage config.
    pub fn with_triage_config(mut self, config: LlmAnalyzerConfig) -> Self {
        self.config.triage_config = Some(config);
        self
    }

    /// Set analysis config.
    pub fn with_analysis_config(mut self, config: LlmAnalyzerConfig) -> Self {
        self.config.analysis_config = Some(config);
        self
    }

    /// Set deep dive config.
    pub fn with_deep_dive_config(mut self, config: LlmAnalyzerConfig) -> Self {
        self.config.deep_dive_config = Some(config);
        self
    }

    /// Set score thresholds for deep dive.
    pub fn with_score_thresholds(mut self, min: f32, max: f32) -> Self {
        self.config.deep_dive_score_threshold_min = min;
        self.config.deep_dive_score_threshold_max = max;
        self
    }

    /// Build the pipeline.
    pub fn build(self) -> Result<MultiStagePipeline> {
        MultiStagePipeline::with_config(self.config)
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod pipeline_tests {
    use super::*;

    #[test]
    fn test_pipeline_config_default() {
        let config = MultiStagePipelineConfig::default();
        assert!(config.triage_enabled);
        assert!(config.analysis_enabled);
        assert!(config.deep_dive_enabled);
        assert_eq!(config.deep_dive_score_threshold_min, 4.0);
        assert_eq!(config.deep_dive_score_threshold_max, 7.0);
    }

    #[test]
    fn test_pipeline_config_triage_only() {
        let config = MultiStagePipelineConfig::triage_only();
        assert!(config.triage_enabled);
        assert!(!config.analysis_enabled);
        assert!(!config.deep_dive_enabled);
    }

    #[test]
    fn test_pipeline_builder() {
        let pipeline = PipelineBuilder::new()
            .with_triage(true)
            .with_analysis(true)
            .with_deep_dive(false)
            .with_score_thresholds(3.0, 6.0)
            .build();

        assert!(pipeline.is_ok());
        let pipeline = pipeline.unwrap();
        assert!(pipeline.config().triage_enabled);
        assert!(pipeline.config().analysis_enabled);
        assert!(!pipeline.config().deep_dive_enabled);
        assert_eq!(pipeline.config().deep_dive_score_threshold_min, 3.0);
        assert_eq!(pipeline.config().deep_dive_score_threshold_max, 6.0);
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
    fn test_llm_config_models_default() {
        let config = LlmAnalyzerConfig::default();
        // Verify orchestrator defaults to NVIDIA for deep analysis
        assert_eq!(config.models.len(), 4);
        assert_eq!(config.models[0], "qwen/qwen3.5-397b-a17b");
        assert!(config.base_url.contains("nvidia"));
        assert_eq!(config.timeout_secs, 120);  // Longer timeout for deep analysis
    }

    #[test]
    fn test_llm_config_cerebras_triage() {
        // Test Cerebras config for CLI triage (fast)
        let config = LlmAnalyzerConfig::cerebras("test-key".to_string());
        assert_eq!(config.models.len(), 1);
        assert_eq!(config.models[0], "llama-3.3-70b");
        assert!(config.base_url.contains("cerebras"));
        assert_eq!(config.timeout_secs, 30);  // Fast timeout for triage
    }

    #[test]
    fn test_llm_config_nvidia_models() {
        // Test NVIDIA config with fallback
        let config = LlmAnalyzerConfig::nvidia_nim("test-key".to_string());
        assert_eq!(config.models.len(), 4);
        assert_eq!(config.models[0], "qwen/qwen3.5-397b-a17b");
        assert!(config.base_url.contains("nvidia"));
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
