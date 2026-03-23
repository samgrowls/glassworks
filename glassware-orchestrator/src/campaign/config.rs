//! Campaign configuration (TOML parsing).
//!
//! Defines the schema for campaign configuration files and provides
//! loading/validation functionality.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info};

use crate::campaign::types::{Priority, WaveMode, SortOrder, GitHubSort};

/// Complete campaign configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignConfig {
    /// Campaign metadata.
    pub campaign: CampaignMetadata,
    /// Global campaign settings.
    #[serde(default)]
    pub settings: CampaignSettings,
    /// Wave definitions.
    #[serde(default)]
    pub waves: Vec<WaveConfig>,
}

impl Default for CampaignConfig {
    fn default() -> Self {
        Self {
            campaign: CampaignMetadata::default(),
            settings: CampaignSettings::default(),
            waves: Vec::new(),
        }
    }
}

impl CampaignConfig {
    /// Load campaign configuration from a TOML file.
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Loading campaign config from: {:?}", path);
        
        let content = std::fs::read_to_string(path)?;
        let config: CampaignConfig = toml::from_str(&content)?;
        
        config.validate()?;
        
        info!(
            "Loaded campaign '{}' with {} waves",
            config.campaign.name,
            config.waves.len()
        );
        
        Ok(config)
    }

    /// Validate the campaign configuration.
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate campaign metadata
        if self.campaign.name.trim().is_empty() {
            return Err(ConfigError::InvalidCampaign("Campaign name cannot be empty".to_string()));
        }

        // Validate waves
        if self.waves.is_empty() {
            return Err(ConfigError::InvalidCampaign("Campaign must have at least one wave".to_string()));
        }

        // Check for duplicate wave IDs
        let mut wave_ids = HashMap::new();
        for (index, wave) in self.waves.iter().enumerate() {
            if let Some(prev_index) = wave_ids.get(&wave.id) {
                return Err(ConfigError::DuplicateWaveId {
                    wave_id: wave.id.clone(),
                    first_index: *prev_index,
                    second_index: index,
                });
            }
            wave_ids.insert(wave.id.clone(), index);
        }

        // Validate wave dependencies
        for wave in &self.waves {
            for dep in &wave.depends_on {
                if !wave_ids.contains_key(dep) {
                    return Err(ConfigError::InvalidDependency {
                        wave_id: wave.id.clone(),
                        missing_dependency: dep.clone(),
                    });
                }
            }

            // Validate wave sources
            if wave.sources.is_empty() {
                return Err(ConfigError::InvalidWave {
                    wave_id: wave.id.clone(),
                    reason: "Wave must have at least one source".to_string(),
                });
            }

            // Validate source-specific requirements
            for source in &wave.sources {
                if let Err(reason) = source.validate() {
                    return Err(ConfigError::InvalidSource {
                        wave_id: wave.id.clone(),
                        source_type: source.source_type(),
                        reason,
                    });
                }
            }
        }

        // Check for circular dependencies
        if let Some(cycle) = self.detect_circular_dependencies() {
            return Err(ConfigError::CircularDependency { cycle });
        }

        Ok(())
    }

    /// Detect circular dependencies in wave graph.
    fn detect_circular_dependencies(&self) -> Option<Vec<String>> {
        let mut visited = HashMap::new();
        let mut rec_stack = HashMap::new();
        let mut path = Vec::new();

        for wave in &self.waves {
            if !visited.contains_key(&wave.id) {
                if let Some(cycle) = self.dfs_cycle_detect(&wave.id, &mut visited, &mut rec_stack, &mut path) {
                    return Some(cycle);
                }
            }
        }

        None
    }

    fn dfs_cycle_detect(
        &self,
        wave_id: &str,
        visited: &mut HashMap<String, bool>,
        rec_stack: &mut HashMap<String, bool>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(wave_id.to_string(), true);
        rec_stack.insert(wave_id.to_string(), true);
        path.push(wave_id.to_string());

        let wave = self.waves.iter().find(|w| w.id == wave_id)?;

        for dep in &wave.depends_on {
            if !visited.contains_key(dep) {
                if let Some(cycle) = self.dfs_cycle_detect(dep, visited, rec_stack, path) {
                    return Some(cycle);
                }
            } else if rec_stack.get(dep).copied().unwrap_or(false) {
                // Found cycle
                let cycle_start = path.iter().position(|id| id == dep).unwrap_or(0);
                let mut cycle = path[cycle_start..].to_vec();
                cycle.push(dep.clone());
                return Some(cycle);
            }
        }

        path.pop();
        rec_stack.insert(wave_id.to_string(), false);
        None
    }

    /// Get wave configuration by ID.
    pub fn get_wave(&self, wave_id: &str) -> Option<&WaveConfig> {
        self.waves.iter().find(|w| &w.id == wave_id)
    }

    /// Get waves that have no dependencies (can run first).
    pub fn get_root_waves(&self) -> Vec<&WaveConfig> {
        self.waves
            .iter()
            .filter(|w| w.depends_on.is_empty())
            .collect()
    }

    /// Get waves that depend on a specific wave.
    pub fn get_dependent_waves(&self, wave_id: &str) -> Vec<&WaveConfig> {
        self.waves
            .iter()
            .filter(|w| w.depends_on.iter().any(|dep| dep == wave_id))
            .collect()
    }

    /// Calculate total packages across all waves (estimate).
    pub fn estimate_total_packages(&self) -> usize {
        self.waves.iter().map(|w| w.estimated_packages()).sum()
    }
}

/// Campaign metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignMetadata {
    /// Human-readable campaign name.
    pub name: String,
    /// Campaign description.
    #[serde(default)]
    pub description: String,
    /// Who created this campaign.
    #[serde(default = "default_unknown")]
    pub created_by: String,
    /// Campaign priority level.
    #[serde(default)]
    pub priority: Priority,
    /// Tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Default for CampaignMetadata {
    fn default() -> Self {
        Self {
            name: "Unnamed Campaign".to_string(),
            description: String::new(),
            created_by: default_unknown(),
            priority: Priority::default(),
            tags: Vec::new(),
        }
    }
}

/// Global campaign settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignSettings {
    /// Maximum concurrent package scans.
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,
    /// npm API rate limit (requests per second).
    #[serde(default = "default_npm_rate_limit")]
    pub rate_limit_npm: f32,
    /// GitHub API rate limit (requests per second).
    #[serde(default = "default_github_rate_limit")]
    pub rate_limit_github: f32,
    /// Enable result caching.
    #[serde(default = "default_true")]
    pub cache_enabled: bool,
    /// Cache TTL in days.
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_days: u64,
    /// LLM configuration.
    #[serde(default)]
    pub llm: LlmSettings,
    /// Output configuration.
    #[serde(default)]
    pub output: OutputSettings,
}

impl Default for CampaignSettings {
    fn default() -> Self {
        Self {
            concurrency: default_concurrency(),
            rate_limit_npm: default_npm_rate_limit(),
            rate_limit_github: default_github_rate_limit(),
            cache_enabled: default_true(),
            cache_ttl_days: default_cache_ttl(),
            llm: LlmSettings::default(),
            output: OutputSettings::default(),
        }
    }
}

/// LLM configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmSettings {
    /// Enable Tier 1 LLM (Cerebras) during scan.
    #[serde(default)]
    pub tier1_enabled: bool,
    /// Tier 1 provider (cerebras).
    #[serde(default = "default_cerebras")]
    pub tier1_provider: String,
    /// Enable Tier 2 LLM (NVIDIA) for flagged packages.
    #[serde(default)]
    pub tier2_enabled: bool,
    /// Threat score threshold for Tier 2 analysis.
    #[serde(default = "default_tier2_threshold")]
    pub tier2_threshold: f32,
    /// NVIDIA model fallback chain.
    #[serde(default = "default_nvidia_models")]
    pub tier2_models: Vec<String>,
}

/// Output configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSettings {
    /// Output formats (json, markdown, sarif).
    #[serde(default = "default_formats")]
    pub formats: Vec<String>,
    /// Collect evidence for flagged packages.
    #[serde(default = "default_true")]
    pub evidence_collection: bool,
    /// Evidence output directory.
    #[serde(default = "default_evidence_dir")]
    pub evidence_dir: String,
    /// Report output directory.
    #[serde(default = "default_report_dir")]
    pub report_dir: String,
}

impl Default for OutputSettings {
    fn default() -> Self {
        Self {
            formats: default_formats(),
            evidence_collection: default_true(),
            evidence_dir: default_evidence_dir(),
            report_dir: default_report_dir(),
        }
    }
}

/// Wave configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveConfig {
    /// Unique wave identifier.
    pub id: String,
    /// Human-readable wave name.
    pub name: String,
    /// Wave description.
    #[serde(default)]
    pub description: String,
    /// Waves that must complete before this wave runs.
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Wave execution mode.
    #[serde(default = "default_wave_mode")]
    pub mode: WaveMode,
    /// Package sources for this wave.
    pub sources: Vec<WaveSource>,
    /// Package whitelist for this wave.
    #[serde(default)]
    pub whitelist: Vec<WhitelistEntry>,
    /// Validation expectations (for validate mode).
    #[serde(default)]
    pub expectations: Option<ValidationExpectations>,
    /// Reporting overrides.
    #[serde(default)]
    pub reporting: Option<ReportingOverrides>,
}

impl WaveConfig {
    /// Estimate total packages for this wave.
    pub fn estimated_packages(&self) -> usize {
        self.sources.iter().map(|s| s.estimated_count()).sum()
    }
}

/// Wave source definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WaveSource {
    /// Explicit package list.
    #[serde(rename = "packages")]
    Packages {
        /// Package specifications (name@version or just name).
        list: Vec<String>,
    },

    /// npm registry search.
    #[serde(rename = "npm_search")]
    NpmSearch {
        /// Search keywords.
        keywords: Vec<String>,
        /// Samples per keyword.
        #[serde(default = "default_samples_per_keyword")]
        samples_per_keyword: usize,
        /// Only packages published in last N days.
        #[serde(default)]
        days_recent: Option<u32>,
        /// Maximum download count filter.
        #[serde(default)]
        max_downloads: Option<u64>,
    },

    /// npm category sampling.
    #[serde(rename = "npm_category")]
    NpmCategory {
        /// Category name.
        category: String,
        /// Number of samples.
        samples: usize,
        /// Sort order.
        #[serde(default = "default_sort_recent")]
        sort_by: SortOrder,
    },

    /// GitHub repository search.
    #[serde(rename = "github_search")]
    GitHubSearch {
        /// Search query.
        query: String,
        /// Maximum results.
        max_results: usize,
        /// Sort order.
        #[serde(default = "default_sort_stars")]
        sort_by: GitHubSort,
    },
}

impl WaveSource {
    /// Get source type as string.
    pub fn source_type(&self) -> &'static str {
        match self {
            WaveSource::Packages { .. } => "packages",
            WaveSource::NpmSearch { .. } => "npm_search",
            WaveSource::NpmCategory { .. } => "npm_category",
            WaveSource::GitHubSearch { .. } => "github_search",
        }
    }

    /// Validate source-specific requirements.
    pub fn validate(&self) -> Result<(), String> {
        match self {
            WaveSource::Packages { list } => {
                if list.is_empty() {
                    return Err("Package list cannot be empty".to_string());
                }
            }
            WaveSource::NpmSearch { keywords, samples_per_keyword, .. } => {
                if keywords.is_empty() {
                    return Err("Keywords cannot be empty".to_string());
                }
                if *samples_per_keyword == 0 {
                    return Err("samples_per_keyword must be positive".to_string());
                }
            }
            WaveSource::NpmCategory { category, samples, .. } => {
                if category.trim().is_empty() {
                    return Err("Category cannot be empty".to_string());
                }
                if *samples == 0 {
                    return Err("samples must be positive".to_string());
                }
            }
            WaveSource::GitHubSearch { query, max_results, .. } => {
                if query.trim().is_empty() {
                    return Err("Query cannot be empty".to_string());
                }
                if *max_results == 0 {
                    return Err("max_results must be positive".to_string());
                }
            }
        }
        Ok(())
    }

    /// Estimate package count from this source.
    pub fn estimated_count(&self) -> usize {
        match self {
            WaveSource::Packages { list } => list.len(),
            WaveSource::NpmSearch { keywords, samples_per_keyword, .. } => {
                keywords.len() * samples_per_keyword
            }
            WaveSource::NpmCategory { samples, .. } => *samples,
            WaveSource::GitHubSearch { max_results, .. } => *max_results,
        }
    }
}

/// Whitelist entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistEntry {
    /// Package names or patterns to whitelist.
    pub packages: Vec<String>,
    /// Reason for whitelisting.
    #[serde(default)]
    pub reason: String,
}

/// Validation expectations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationExpectations {
    /// All malicious packages must be flagged.
    #[serde(default = "default_true")]
    pub must_flag_all: bool,
    /// Minimum threat score for flagged packages.
    #[serde(default = "default_min_threat_score")]
    pub min_threat_score: f32,
}

/// Reporting overrides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingOverrides {
    /// Include clean package summary.
    #[serde(default)]
    pub include_clean_summary: bool,
    /// Slack webhook URL.
    #[serde(default)]
    pub slack_webhook: Option<String>,
}

// Default value functions
fn default_unknown() -> String { "unknown".to_string() }
fn default_concurrency() -> usize { 10 }
fn default_npm_rate_limit() -> f32 { 10.0 }
fn default_github_rate_limit() -> f32 { 5.0 }
fn default_true() -> bool { true }
fn default_cache_ttl() -> u64 { 7 }
fn default_cerebras() -> String { "cerebras".to_string() }
fn default_tier2_threshold() -> f32 { 5.0 }
fn default_nvidia_models() -> Vec<String> {
    vec![
        "qwen/qwen3.5-397b-a17b".to_string(),
        "moonshotai/kimi-k2.5".to_string(),
        "z-ai/glm5".to_string(),
        "meta/llama-3.3-70b-instruct".to_string(),
    ]
}
fn default_wave_mode() -> WaveMode { WaveMode::Hunt }
fn default_samples_per_keyword() -> usize { 25 }
fn default_sort_recent() -> SortOrder { SortOrder::Recent }
fn default_sort_stars() -> GitHubSort { GitHubSort::Stars }
fn default_min_threat_score() -> f32 { 7.0 }
fn default_formats() -> Vec<String> {
    vec!["json".to_string(), "markdown".to_string()]
}
fn default_evidence_dir() -> String { "evidence".to_string() }
fn default_report_dir() -> String { "reports".to_string() }

/// Campaign configuration errors.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid campaign: {0}")]
    InvalidCampaign(String),

    #[error("Duplicate wave ID '{wave_id}' at indices {first_index} and {second_index}")]
    DuplicateWaveId {
        wave_id: String,
        first_index: usize,
        second_index: usize,
    },

    #[error("Wave '{wave_id}' has invalid dependency '{missing_dependency}'")]
    InvalidDependency {
        wave_id: String,
        missing_dependency: String,
    },

    #[error("Wave '{wave_id}' is invalid: {reason}")]
    InvalidWave {
        wave_id: String,
        reason: String,
    },

    #[error("Wave '{wave_id}' has invalid source '{source_type}': {reason}")]
    InvalidSource {
        wave_id: String,
        source_type: &'static str,
        reason: String,
    },

    #[error("Circular dependency detected: {cycle:?}")]
    CircularDependency { cycle: Vec<String> },

    #[error("TOML parse error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_campaign_config() {
        let config = CampaignConfig::default();
        assert_eq!(config.campaign.name, "Unnamed Campaign");
        assert_eq!(config.waves.len(), 0);
        assert_eq!(config.settings.concurrency, 10);
    }

    #[test]
    fn test_validate_empty_waves() {
        let config = CampaignConfig::default();
        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidCampaign(msg) => {
                assert!(msg.contains("at least one wave"));
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_validate_duplicate_wave_ids() {
        let config = CampaignConfig {
            campaign: CampaignMetadata {
                name: "Test".to_string(),
                ..Default::default()
            },
            waves: vec![
                WaveConfig {
                    id: "wave1".to_string(),
                    name: "Wave 1".to_string(),
                    description: String::new(),
                    depends_on: Vec::new(),
                    mode: WaveMode::Hunt,
                    sources: vec![WaveSource::Packages { list: vec!["express".to_string()] }],
                    whitelist: Vec::new(),
                    expectations: None,
                    reporting: None,
                },
                WaveConfig {
                    id: "wave1".to_string(), // Duplicate!
                    name: "Wave 1 Duplicate".to_string(),
                    description: String::new(),
                    depends_on: Vec::new(),
                    mode: WaveMode::Hunt,
                    sources: vec![WaveSource::Packages { list: vec!["lodash".to_string()] }],
                    whitelist: Vec::new(),
                    expectations: None,
                    reporting: None,
                },
            ],
            settings: CampaignSettings::default(),
        };

        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::DuplicateWaveId { wave_id, .. } => {
                assert_eq!(wave_id, "wave1");
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_validate_circular_dependencies() {
        let config = CampaignConfig {
            campaign: CampaignMetadata {
                name: "Test".to_string(),
                ..Default::default()
            },
            waves: vec![
                WaveConfig {
                    id: "wave1".to_string(),
                    name: "Wave 1".to_string(),
                    description: String::new(),
                    depends_on: vec!["wave2".to_string()],
                    mode: WaveMode::Hunt,
                    sources: vec![WaveSource::Packages { list: vec!["express".to_string()] }],
                    whitelist: Vec::new(),
                    expectations: None,
                    reporting: None,
                },
                WaveConfig {
                    id: "wave2".to_string(),
                    name: "Wave 2".to_string(),
                    description: String::new(),
                    depends_on: vec!["wave1".to_string()], // Circular!
                    mode: WaveMode::Hunt,
                    sources: vec![WaveSource::Packages { list: vec!["lodash".to_string()] }],
                    whitelist: Vec::new(),
                    expectations: None,
                    reporting: None,
                },
            ],
            settings: CampaignSettings::default(),
        };

        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::CircularDependency { cycle } => {
                assert!(cycle.contains(&"wave1".to_string()));
                assert!(cycle.contains(&"wave2".to_string()));
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_wave_source_validation() {
        // Valid package list
        let source = WaveSource::Packages { list: vec!["express".to_string()] };
        assert!(source.validate().is_ok());

        // Empty package list
        let source = WaveSource::Packages { list: vec![] };
        assert!(source.validate().is_err());

        // Valid npm search
        let source = WaveSource::NpmSearch {
            keywords: vec!["react".to_string()],
            samples_per_keyword: 10,
            days_recent: None,
            max_downloads: None,
        };
        assert!(source.validate().is_ok());

        // Empty keywords
        let source = WaveSource::NpmSearch {
            keywords: vec![],
            samples_per_keyword: 10,
            days_recent: None,
            max_downloads: None,
        };
        assert!(source.validate().is_err());
    }

    #[test]
    fn test_estimated_packages() {
        let config = CampaignConfig {
            campaign: CampaignMetadata {
                name: "Test".to_string(),
                ..Default::default()
            },
            waves: vec![
                WaveConfig {
                    id: "wave1".to_string(),
                    name: "Wave 1".to_string(),
                    description: String::new(),
                    depends_on: Vec::new(),
                    mode: WaveMode::Hunt,
                    sources: vec![
                        WaveSource::Packages { list: vec!["a".to_string(), "b".to_string()] },
                        WaveSource::NpmSearch {
                            keywords: vec!["x".to_string(), "y".to_string()],
                            samples_per_keyword: 5,
                            days_recent: None,
                            max_downloads: None,
                        },
                    ],
                    whitelist: Vec::new(),
                    expectations: None,
                    reporting: None,
                },
            ],
            settings: CampaignSettings::default(),
        };

        assert_eq!(config.estimate_total_packages(), 12); // 2 + (2 * 5)
    }
}
