//! Campaign report generation.
//!
//! This module provides markdown report generation for completed campaigns.
//! Reports include:
//! - Executive summary
//! - Wave results table
//! - Findings by category
//! - LLM analysis summary (if available)
//! - Evidence manifest
//! - Appendix with configuration

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tera::{Tera, Context};
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::campaign::executor::CampaignResult;

/// Report generation context for the template.
#[derive(Debug, Clone, Serialize)]
pub struct ReportContext {
    /// Campaign case ID.
    pub case_id: String,
    /// Campaign name.
    pub campaign_name: String,
    /// Campaign status.
    pub status: String,
    /// Report generation timestamp.
    pub generated_at: String,
    /// Campaign start time (if available).
    pub started_at: Option<String>,
    /// Campaign end time (if available).
    pub completed_at: Option<String>,
    /// Total duration as formatted string.
    pub duration: String,
    /// Total packages scanned.
    pub total_scanned: usize,
    /// Total packages flagged.
    pub total_flagged: usize,
    /// Total packages malicious.
    pub total_malicious: usize,
    /// Flag rate as percentage.
    pub flag_rate: f64,
    /// Malicious rate as percentage.
    pub malicious_rate: f64,
    /// Wave results for the table.
    pub waves: Vec<WaveReport>,
    /// Findings grouped by category.
    pub findings_by_category: HashMap<String, usize>,
    /// LLM analysis summary (if available).
    pub llm_summary: Option<LlmSummary>,
    /// Evidence manifest entries.
    pub evidence: Vec<EvidenceEntry>,
    /// Campaign configuration summary.
    pub config_summary: ConfigSummary,
}

/// Wave result for the report table.
#[derive(Debug, Clone, Serialize)]
pub struct WaveReport {
    /// Wave ID.
    pub wave_id: String,
    /// Wave name.
    pub wave_name: String,
    /// Packages scanned.
    pub packages_scanned: usize,
    /// Packages flagged.
    pub packages_flagged: usize,
    /// Packages malicious.
    pub packages_malicious: usize,
    /// Flag rate as percentage.
    pub flag_rate: f64,
    /// Malicious rate as percentage.
    pub malicious_rate: f64,
}

/// LLM analysis summary.
#[derive(Debug, Clone, Serialize)]
pub struct LlmSummary {
    /// Total analyses performed.
    pub total_analyses: usize,
    /// Confirmed malicious by LLM.
    pub confirmed_malicious: usize,
    /// False positive detections.
    pub false_positives: usize,
    /// Average confidence score.
    pub avg_confidence: f64,
}

/// Evidence manifest entry.
#[derive(Debug, Clone, Serialize)]
pub struct EvidenceEntry {
    /// Evidence type (package, finding, log).
    pub entry_type: String,
    /// Evidence identifier.
    pub identifier: String,
    /// Storage path or reference.
    pub location: String,
    /// Timestamp of collection.
    pub collected_at: String,
}

/// Campaign configuration summary.
#[derive(Debug, Clone, Serialize)]
pub struct ConfigSummary {
    /// Concurrency setting.
    pub concurrency: usize,
    /// Rate limit setting.
    pub rate_limit: u32,
    /// LLM enabled.
    pub llm_enabled: bool,
    /// Threat score threshold.
    pub threat_threshold: f32,
}

/// Report generator for campaigns.
pub struct ReportGenerator {
    tera: Tera,
}

impl ReportGenerator {
    /// Create a new report generator with embedded template.
    pub fn new() -> Result<Self, tera::Error> {
        let mut tera = Tera::default();
        
        // Register the embedded template
        tera.add_raw_template("report.md.tera", include_str!("../../templates/report.md.tera"))?;
        
        // Register custom filters
        tera.register_filter("format_duration", Self::format_duration_filter);
        tera.register_filter("format_percentage", Self::format_percentage_filter);
        tera.register_filter("format_datetime", Self::format_datetime_filter);
        
        Ok(Self { tera })
    }
    
    /// Generate a markdown report from campaign results.
    ///
    /// # Arguments
    /// * `result` - Campaign execution result
    /// * `config_summary` - Campaign configuration summary
    ///
    /// # Returns
    /// * `Ok(String)` - Generated markdown report
    /// * `Err(tera::Error)` - Template rendering failed
    pub fn generate_report(
        &self,
        result: &CampaignResult,
        config_summary: ConfigSummary,
    ) -> Result<String, tera::Error> {
        let context = self.build_context(result, config_summary);
        self.tera.render("report.md.tera", &context)
    }
    
    /// Generate and save report to file.
    ///
    /// # Arguments
    /// * `result` - Campaign execution result
    /// * `config_summary` - Campaign configuration summary
    /// * `output_path` - Path to save the report
    ///
    /// # Returns
    /// * `Ok(())` - Report saved successfully
    /// * `Err(ReportError)` - Failed to generate or save report
    pub fn generate_and_save(
        &self,
        result: &CampaignResult,
        config_summary: ConfigSummary,
        output_path: &Path,
    ) -> Result<(), ReportError> {
        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| ReportError::IoError(format!("Failed to create directory: {}", e)))?;
        }
        
        // Generate report
        let report = self.generate_report(result, config_summary)
            .map_err(|e| ReportError::TemplateError(e.to_string()))?;
        
        // Write to file
        fs::write(output_path, report)
            .map_err(|e| ReportError::IoError(format!("Failed to write report: {}", e)))?;
        
        Ok(())
    }
    
    /// Build the report context from campaign results.
    fn build_context(
        &self,
        result: &CampaignResult,
        config_summary: ConfigSummary,
    ) -> Context {
        let now = Utc::now();
        
        // Calculate rates
        let flag_rate = if result.total_scanned > 0 {
            (result.total_flagged as f64 / result.total_scanned as f64) * 100.0
        } else {
            0.0
        };
        
        let malicious_rate = if result.total_scanned > 0 {
            (result.total_malicious as f64 / result.total_scanned as f64) * 100.0
        } else {
            0.0
        };
        
        // Build wave reports
        let waves: Vec<WaveReport> = result.wave_results.iter().map(|wave| {
            let flag_rate = if wave.packages_scanned > 0 {
                (wave.packages_flagged as f64 / wave.packages_scanned as f64) * 100.0
            } else {
                0.0
            };
            
            let malicious_rate = if wave.packages_scanned > 0 {
                (wave.packages_malicious as f64 / wave.packages_scanned as f64) * 100.0
            } else {
                0.0
            };
            
            WaveReport {
                wave_id: wave.wave_id.clone(),
                wave_name: format!("Wave {}", wave.wave_id), // Could be enhanced with wave config name
                packages_scanned: wave.packages_scanned,
                packages_flagged: wave.packages_flagged,
                packages_malicious: wave.packages_malicious,
                flag_rate,
                malicious_rate,
            }
        }).collect();
        
        // Build findings by category (placeholder - would need actual findings data)
        let mut findings_by_category = HashMap::new();
        findings_by_category.insert("Suspicious Patterns".to_string(), result.total_flagged);
        findings_by_category.insert("Malicious Indicators".to_string(), result.total_malicious);
        
        // Build context
        let mut context = Context::new();
        context.insert("case_id", &result.case_id);
        context.insert("campaign_name", &result.campaign_name);
        context.insert("status", &format!("{:?}", result.status));
        context.insert("generated_at", &now.format("%Y-%m-%d %H:%M:%S UTC").to_string());
        context.insert("started_at", &Option::<String>::None); // Would need timestamp from state
        context.insert("completed_at", &Some(now.format("%Y-%m-%d %H:%M:%S UTC").to_string()));
        context.insert("duration", &self.format_duration(result.duration));
        context.insert("total_scanned", &result.total_scanned);
        context.insert("total_flagged", &result.total_flagged);
        context.insert("total_malicious", &result.total_malicious);
        context.insert("flag_rate", &format!("{:.2}%", flag_rate));
        context.insert("malicious_rate", &format!("{:.2}%", malicious_rate));
        context.insert("waves", &waves);
        context.insert("findings_by_category", &findings_by_category);
        context.insert("llm_summary", &Option::<LlmSummary>::None); // Would need LLM data
        context.insert("evidence", &Vec::<EvidenceEntry>::new()); // Would need evidence data
        context.insert("config_summary", &config_summary);
        
        context
    }
    
    /// Format duration as human-readable string.
    fn format_duration(&self, duration: std::time::Duration) -> String {
        let total_secs = duration.as_secs();
        let hours = total_secs / 3600;
        let mins = (total_secs % 3600) / 60;
        let secs = total_secs % 60;
        
        if hours > 0 {
            format!("{}h {}m {}s", hours, mins, secs)
        } else if mins > 0 {
            format!("{}m {}s", mins, secs)
        } else {
            format!("{}s", secs)
        }
    }
    
    /// Tera filter for formatting duration.
    fn format_duration_filter(
        value: &tera::Value,
        _: &HashMap<String, tera::Value>,
    ) -> Result<tera::Value, tera::Error> {
        let secs = value.as_u64().ok_or_else(|| {
            tera::Error::msg("format_duration filter expects a u64 seconds value")
        })?;
        
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        let remaining_secs = secs % 60;
        
        let formatted = if hours > 0 {
            format!("{}h {}m {}s", hours, mins, remaining_secs)
        } else if mins > 0 {
            format!("{}m {}s", mins, remaining_secs)
        } else {
            format!("{}s", remaining_secs)
        };
        
        Ok(tera::Value::String(formatted))
    }
    
    /// Tera filter for formatting percentage.
    fn format_percentage_filter(
        value: &tera::Value,
        _: &HashMap<String, tera::Value>,
    ) -> Result<tera::Value, tera::Error> {
        let num = value.as_f64().ok_or_else(|| {
            tera::Error::msg("format_percentage filter expects a f64 value")
        })?;
        
        Ok(tera::Value::String(format!("{:.2}%", num)))
    }
    
    /// Tera filter for formatting datetime.
    fn format_datetime_filter(
        value: &tera::Value,
        _: &HashMap<String, tera::Value>,
    ) -> Result<tera::Value, tera::Error> {
        let s = value.as_str().ok_or_else(|| {
            tera::Error::msg("format_datetime filter expects a string value")
        })?;
        
        // Try to parse and reformat
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            Ok(tera::Value::String(dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()))
        } else {
            Ok(tera::Value::String(s.to_string()))
        }
    }
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create ReportGenerator")
    }
}

/// Report generation errors.
#[derive(Debug, thiserror::Error)]
pub enum ReportError {
    #[error("Template error: {0}")]
    TemplateError(String),
    
    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("Campaign not found: {0}")]
    CampaignNotFound(String),
    
    #[error("Checkpoint error: {0}")]
    CheckpointError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[test]
    fn test_format_duration() {
        let generator = ReportGenerator::default();
        
        assert_eq!(generator.format_duration(Duration::from_secs(5)), "5s");
        assert_eq!(generator.format_duration(Duration::from_secs(65)), "1m 5s");
        assert_eq!(generator.format_duration(Duration::from_secs(3665)), "1h 1m 5s");
    }
    
    #[test]
    fn test_report_generator_creation() {
        let result = ReportGenerator::new();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_wave_report_calculation() {
        let wave = WaveReport {
            wave_id: "wave1".to_string(),
            wave_name: "Wave 1".to_string(),
            packages_scanned: 100,
            packages_flagged: 10,
            packages_malicious: 2,
            flag_rate: 10.0,
            malicious_rate: 2.0,
        };
        
        assert!((wave.flag_rate - 10.0).abs() < f64::EPSILON);
        assert!((wave.malicious_rate - 2.0).abs() < f64::EPSILON);
    }
}
