//! Configuration module for GlassWorm orchestrator.
//!
//! Provides a hierarchical configuration system with support for:
//! - User-level config (~/.config/glassware/config.toml)
//! - Project-level config (.glassware.toml)
//! - Environment variable overrides
//! - CLI argument overrides
//! - Sensible defaults

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlasswareConfig {
    pub scoring: ScoringConfig,
    pub detectors: DetectorConfig,
    pub whitelist: WhitelistConfig,
    pub performance: PerformanceConfig,
    pub llm: LlmConfig,
    pub output: OutputConfig,
}

/// Scoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    /// Threat score threshold for "malicious" classification
    #[serde(default = "default_malicious_threshold")]
    pub malicious_threshold: f32,

    /// Threat score threshold for "suspicious" classification
    #[serde(default = "default_suspicious_threshold")]
    pub suspicious_threshold: f32,

    /// Weight per category present in signal stacking
    #[serde(default = "default_category_weight")]
    pub category_weight: f32,

    /// Weight per critical finding
    #[serde(default = "default_critical_weight")]
    pub critical_weight: f32,

    /// Weight per high severity finding
    #[serde(default = "default_high_weight")]
    pub high_weight: f32,
}

/// Detector-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DetectorConfig {
    #[serde(default)]
    pub invisible_char: DetectorWeightConfig,

    #[serde(default)]
    pub homoglyph: DetectorWeightConfig,

    #[serde(default)]
    pub bidi: DetectorWeightConfig,

    #[serde(default)]
    pub blockchain_c2: DetectorWeightConfig,

    #[serde(default)]
    pub glassware_pattern: DetectorWeightConfig,

    #[serde(default)]
    pub locale_geofencing: LocaleGeofencingConfig,
}

/// Per-detector weight configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectorWeightConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_weight")]
    pub weight: f32,
}

impl Default for DetectorWeightConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            weight: default_weight(),
        }
    }
}

/// Locale geofencing detector specific config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleGeofencingConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub skip_for_packages: Vec<String>,
}

impl Default for LocaleGeofencingConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            skip_for_packages: Vec::new(),
        }
    }
}

/// Package whitelist configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WhitelistConfig {
    /// Packages to never flag (locale libraries, etc.)
    #[serde(default)]
    pub packages: Vec<String>,

    /// Crypto libraries (blockchain API calls are legitimate)
    #[serde(default)]
    pub crypto_packages: Vec<String>,

    /// Build tools (time delays are legitimate)
    #[serde(default)]
    pub build_tools: Vec<String>,

    /// State management libraries (complex patterns are legitimate)
    #[serde(default)]
    pub state_management: Vec<String>,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,

    #[serde(default = "default_npm_rate_limit")]
    pub npm_rate_limit: f32,

    #[serde(default = "default_github_rate_limit")]
    pub github_rate_limit: f32,

    #[serde(default = "default_true")]
    pub cache_enabled: bool,

    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_days: u32,
}

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmConfig {
    #[serde(default)]
    pub provider: String,

    #[serde(default)]
    pub cerebras: LlmProviderConfig,

    #[serde(default)]
    pub nvidia: LlmProviderConfig,
}

/// LLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmProviderConfig {
    #[serde(default)]
    pub base_url: String,

    #[serde(default)]
    pub model: String,

    #[serde(default)]
    pub models: Vec<String>,
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_format")]
    pub format: String,

    #[serde(default = "default_min_severity")]
    pub min_severity: String,

    #[serde(default = "default_true")]
    pub color: bool,
}

// Default value functions
fn default_malicious_threshold() -> f32 {
    8.0 // Raised from 7.0 for Phase A.5 scoring redesign
}
fn default_suspicious_threshold() -> f32 {
    4.0 // Raised from 3.0 for Phase A.5 scoring redesign
}
fn default_category_weight() -> f32 {
    2.0
}
fn default_critical_weight() -> f32 {
    3.0
}
fn default_high_weight() -> f32 {
    1.5
}
fn default_true() -> bool {
    true
}
fn default_weight() -> f32 {
    1.0
}
fn default_concurrency() -> usize {
    10
}
fn default_npm_rate_limit() -> f32 {
    10.0
}
fn default_github_rate_limit() -> f32 {
    5.0
}
fn default_cache_ttl() -> u32 {
    7
}
fn default_format() -> String {
    "pretty".to_string()
}
fn default_min_severity() -> String {
    "low".to_string()
}

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            malicious_threshold: default_malicious_threshold(),
            suspicious_threshold: default_suspicious_threshold(),
            category_weight: default_category_weight(),
            critical_weight: default_critical_weight(),
            high_weight: default_high_weight(),
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            concurrency: default_concurrency(),
            npm_rate_limit: default_npm_rate_limit(),
            github_rate_limit: default_github_rate_limit(),
            cache_enabled: default_true(),
            cache_ttl_days: default_cache_ttl(),
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: default_format(),
            min_severity: default_min_severity(),
            color: default_true(),
        }
    }
}

impl Default for GlasswareConfig {
    fn default() -> Self {
        Self {
            scoring: ScoringConfig::default(),
            detectors: DetectorConfig::default(),
            whitelist: WhitelistConfig::default(),
            performance: PerformanceConfig::default(),
            llm: LlmConfig::default(),
            output: OutputConfig::default(),
        }
    }
}

impl GlasswareConfig {
    /// Get the default user config directory path
    pub fn user_config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("glassware"))
    }

    /// Get the default user config file path
    pub fn user_config_file() -> Option<PathBuf> {
        Self::user_config_dir().map(|dir| dir.join("config.toml"))
    }

    /// Get the project config file path (current directory)
    pub fn project_config_file() -> Option<PathBuf> {
        std::env::current_dir()
            .ok()
            .map(|dir| dir.join(".glassware.toml"))
    }

    /// Load configuration with hierarchy:
    /// CLI overrides > Project config > User config > Env vars > Defaults
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // Start with defaults
        let mut config = GlasswareConfig::default();

        // Load user config if exists
        if let Some(user_path) = Self::user_config_file() {
            if user_path.exists() {
                let user_config = Self::from_file(&user_path)?;
                config = merge_configs(config, user_config);
            }
        }

        // Load project config if exists
        if let Some(project_path) = Self::project_config_file() {
            if project_path.exists() {
                let project_config = Self::from_file(&project_path)?;
                config = merge_configs(config, project_config);
            }
        }

        // TODO: Apply environment variable overrides
        // TODO: Apply CLI overrides

        Ok(config)
    }

    /// Load configuration from a TOML file
    pub fn from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to user config file
    pub fn save_user_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(config_dir) = Self::user_config_dir() {
            std::fs::create_dir_all(&config_dir)?;

            if let Some(config_path) = Self::user_config_file() {
                let content = toml::to_string_pretty(self)?;
                std::fs::write(config_path, content)?;
            }
        }
        Ok(())
    }

    /// Validate configuration (strict mode)
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Validate thresholds
        if self.scoring.malicious_threshold < self.scoring.suspicious_threshold {
            return Err(
                "malicious_threshold must be >= suspicious_threshold".into()
            );
        }

        if self.scoring.malicious_threshold > 10.0
            || self.scoring.malicious_threshold < 0.0
        {
            return Err(
                "malicious_threshold must be between 0.0 and 10.0".into()
            );
        }

        // Validate weights
        if self.scoring.category_weight < 0.0 {
            return Err("category_weight must be non-negative".into());
        }

        // Validate performance settings
        if self.performance.concurrency == 0 {
            return Err("concurrency must be at least 1".into());
        }

        Ok(())
    }
}

/// Merge two configs, with `override_config` taking precedence
fn merge_configs(
    base: GlasswareConfig,
    override_config: GlasswareConfig,
) -> GlasswareConfig {
    GlasswareConfig {
        scoring: merge_scoring(base.scoring, override_config.scoring),
        detectors: merge_detectors(base.detectors, override_config.detectors),
        whitelist: merge_whitelist(base.whitelist, override_config.whitelist),
        performance: merge_performance(
            base.performance,
            override_config.performance,
        ),
        llm: merge_llm(base.llm, override_config.llm),
        output: merge_output(base.output, override_config.output),
    }
}

/// Merge scoring configs, with override taking precedence for non-default values
fn merge_scoring(base: ScoringConfig, override_config: ScoringConfig) -> ScoringConfig {
    ScoringConfig {
        malicious_threshold: if override_config.malicious_threshold
            != default_malicious_threshold()
        {
            override_config.malicious_threshold
        } else {
            base.malicious_threshold
        },
        suspicious_threshold: if override_config.suspicious_threshold
            != default_suspicious_threshold()
        {
            override_config.suspicious_threshold
        } else {
            base.suspicious_threshold
        },
        category_weight: override_config.category_weight,
        critical_weight: override_config.critical_weight,
        high_weight: override_config.high_weight,
    }
}

/// Merge detector configs, with override taking precedence
fn merge_detectors(
    base: DetectorConfig,
    override_config: DetectorConfig,
) -> DetectorConfig {
    DetectorConfig {
        invisible_char: merge_detector_weight_config(
            base.invisible_char,
            override_config.invisible_char,
        ),
        homoglyph: merge_detector_weight_config(
            base.homoglyph,
            override_config.homoglyph,
        ),
        bidi: merge_detector_weight_config(base.bidi, override_config.bidi),
        blockchain_c2: merge_detector_weight_config(
            base.blockchain_c2,
            override_config.blockchain_c2,
        ),
        glassware_pattern: merge_detector_weight_config(
            base.glassware_pattern,
            override_config.glassware_pattern,
        ),
        locale_geofencing: merge_locale_geofencing_config(
            base.locale_geofencing,
            override_config.locale_geofencing,
        ),
    }
}

/// Merge per-detector weight configs
fn merge_detector_weight_config(
    base: DetectorWeightConfig,
    override_config: DetectorWeightConfig,
) -> DetectorWeightConfig {
    DetectorWeightConfig {
        enabled: if override_config.enabled != default_true() {
            override_config.enabled
        } else {
            base.enabled
        },
        weight: if override_config.weight != default_weight() {
            override_config.weight
        } else {
            base.weight
        },
    }
}

/// Merge locale geofencing configs
fn merge_locale_geofencing_config(
    base: LocaleGeofencingConfig,
    override_config: LocaleGeofencingConfig,
) -> LocaleGeofencingConfig {
    LocaleGeofencingConfig {
        enabled: if override_config.enabled != default_true() {
            override_config.enabled
        } else {
            base.enabled
        },
        skip_for_packages: [
            base.skip_for_packages,
            override_config.skip_for_packages,
        ]
        .concat(),
    }
}

/// Merge whitelist configs by concatenating lists
fn merge_whitelist(
    base: WhitelistConfig,
    override_config: WhitelistConfig,
) -> WhitelistConfig {
    WhitelistConfig {
        packages: [base.packages, override_config.packages].concat(),
        crypto_packages: [base.crypto_packages, override_config.crypto_packages]
            .concat(),
        build_tools: [base.build_tools, override_config.build_tools].concat(),
        state_management: [base.state_management, override_config.state_management].concat(),
    }
}

/// Merge performance configs, with override taking precedence
fn merge_performance(
    base: PerformanceConfig,
    override_config: PerformanceConfig,
) -> PerformanceConfig {
    PerformanceConfig {
        concurrency: if override_config.concurrency != default_concurrency() {
            override_config.concurrency
        } else {
            base.concurrency
        },
        npm_rate_limit: if override_config.npm_rate_limit
            != default_npm_rate_limit()
        {
            override_config.npm_rate_limit
        } else {
            base.npm_rate_limit
        },
        github_rate_limit: if override_config.github_rate_limit
            != default_github_rate_limit()
        {
            override_config.github_rate_limit
        } else {
            base.github_rate_limit
        },
        cache_enabled: if override_config.cache_enabled != default_true() {
            override_config.cache_enabled
        } else {
            base.cache_enabled
        },
        cache_ttl_days: if override_config.cache_ttl_days != default_cache_ttl() {
            override_config.cache_ttl_days
        } else {
            base.cache_ttl_days
        },
    }
}

/// Merge LLM configs, with override taking precedence
fn merge_llm(base: LlmConfig, override_config: LlmConfig) -> LlmConfig {
    LlmConfig {
        provider: if override_config.provider.is_empty() {
            base.provider
        } else {
            override_config.provider
        },
        cerebras: merge_llm_provider_config(
            base.cerebras,
            override_config.cerebras,
        ),
        nvidia: merge_llm_provider_config(base.nvidia, override_config.nvidia),
    }
}

/// Merge LLM provider configs
fn merge_llm_provider_config(
    base: LlmProviderConfig,
    override_config: LlmProviderConfig,
) -> LlmProviderConfig {
    LlmProviderConfig {
        base_url: if override_config.base_url.is_empty() {
            base.base_url
        } else {
            override_config.base_url
        },
        model: if override_config.model.is_empty() {
            base.model
        } else {
            override_config.model
        },
        models: [base.models, override_config.models].concat(),
    }
}

/// Merge output configs, with override taking precedence
fn merge_output(
    base: OutputConfig,
    override_config: OutputConfig,
) -> OutputConfig {
    OutputConfig {
        format: if override_config.format != default_format() {
            override_config.format
        } else {
            base.format
        },
        min_severity: if override_config.min_severity != default_min_severity() {
            override_config.min_severity
        } else {
            base.min_severity
        },
        color: if override_config.color != default_true() {
            override_config.color
        } else {
            base.color
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = GlasswareConfig::default();
        assert_eq!(config.scoring.malicious_threshold, 7.0);
        assert_eq!(config.scoring.suspicious_threshold, 3.0);
        assert_eq!(config.performance.concurrency, 10);
        assert!(config.performance.cache_enabled);
        assert!(config.output.color);
    }

    #[test]
    fn test_config_from_toml() {
        let toml_content = r#"
[scoring]
malicious_threshold = 8.0
suspicious_threshold = 4.0

[performance]
concurrency = 20
npm_rate_limit = 15.0
"#;

        let config: GlasswareConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(config.scoring.malicious_threshold, 8.0);
        assert_eq!(config.scoring.suspicious_threshold, 4.0);
        assert_eq!(config.performance.concurrency, 20);
        assert_eq!(config.performance.npm_rate_limit, 15.0);
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config = GlasswareConfig::default();
        let content = toml::to_string_pretty(&config).unwrap();
        std::fs::write(&config_path, content).unwrap();

        let loaded_config = GlasswareConfig::from_file(&config_path).unwrap();
        assert_eq!(
            config.scoring.malicious_threshold,
            loaded_config.scoring.malicious_threshold
        );
    }

    #[test]
    fn test_config_validation_valid() {
        let config = GlasswareConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_invalid_threshold() {
        let config = GlasswareConfig {
            scoring: ScoringConfig {
                malicious_threshold: 2.0,
                suspicious_threshold: 5.0,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_invalid_concurrency() {
        let config = GlasswareConfig {
            performance: PerformanceConfig {
                concurrency: 0,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_merge_scoring() {
        let base = ScoringConfig::default();
        let override_config = ScoringConfig {
            malicious_threshold: 9.0,
            ..Default::default()
        };
        let merged = merge_scoring(base, override_config);
        assert_eq!(merged.malicious_threshold, 9.0);
        assert_eq!(merged.suspicious_threshold, 3.0); // from base
    }

    #[test]
    fn test_merge_whitelist() {
        let base = WhitelistConfig {
            packages: vec!["package1".to_string()],
            crypto_packages: vec!["crypto1".to_string()],
            build_tools: vec!["build1".to_string()],
            state_management: vec!["state1".to_string()],
        };
        let override_config = WhitelistConfig {
            packages: vec!["package2".to_string()],
            crypto_packages: vec!["crypto2".to_string()],
            build_tools: vec!["build2".to_string()],
            state_management: vec!["state2".to_string()],
        };
        let merged = merge_whitelist(base, override_config);
        assert_eq!(merged.packages.len(), 2);
        assert_eq!(merged.crypto_packages.len(), 2);
        assert_eq!(merged.build_tools.len(), 2);
        assert_eq!(merged.state_management.len(), 2);
    }

    #[test]
    fn test_merge_detectors() {
        let base = DetectorConfig::default();
        let override_config = DetectorConfig {
            invisible_char: DetectorWeightConfig {
                enabled: false,
                weight: 2.0,
            },
            ..Default::default()
        };
        let merged = merge_detectors(base, override_config);
        assert!(!merged.invisible_char.enabled);
        assert_eq!(merged.invisible_char.weight, 2.0);
        assert!(merged.homoglyph.enabled); // from base
    }

    #[test]
    fn test_user_config_paths() {
        // Test that config paths can be generated
        let user_dir = GlasswareConfig::user_config_dir();
        let user_file = GlasswareConfig::user_config_file();
        let project_file = GlasswareConfig::project_config_file();

        assert!(user_dir.is_some());
        assert!(user_file.is_some());
        assert!(project_file.is_some());
    }
}
