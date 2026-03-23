//! Core types for the campaign system.
//!
//! This module defines the fundamental types used throughout
//! the campaign execution engine.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;
use uuid::Uuid;

/// Unique identifier for a campaign run.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CaseId(String);

impl CaseId {
    /// Generate a new unique case ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create a case ID from a string.
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Get the case ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Generate a case ID with timestamp prefix for readability.
    pub fn with_timestamp(prefix: &str) -> Self {
        let timestamp = Utc::now().format("%Y%m%d-%H%M%S");
        Self(format!("{}-{}-{}", prefix, timestamp, Uuid::new_v4().as_simple()))
    }
}

impl Default for CaseId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CaseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Campaign priority level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Priority::Low => write!(f, "low"),
            Priority::Medium => write!(f, "medium"),
            Priority::High => write!(f, "high"),
            Priority::Critical => write!(f, "critical"),
        }
    }
}

/// Campaign execution status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CampaignStatus {
    #[default]
    Initializing,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl fmt::Display for CampaignStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CampaignStatus::Initializing => write!(f, "initializing"),
            CampaignStatus::Running => write!(f, "running"),
            CampaignStatus::Paused => write!(f, "paused"),
            CampaignStatus::Completed => write!(f, "completed"),
            CampaignStatus::Failed => write!(f, "failed"),
            CampaignStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl CampaignStatus {
    /// Check if the campaign is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            CampaignStatus::Completed | CampaignStatus::Failed | CampaignStatus::Cancelled
        )
    }

    /// Check if the campaign can accept commands.
    pub fn accepts_commands(&self) -> bool {
        matches!(self, CampaignStatus::Running | CampaignStatus::Paused)
    }
}

/// Wave execution mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaveMode {
    /// Validate detection with known malicious/clean packages.
    Validate,
    /// Hunt for malicious packages in a category.
    #[default]
    Hunt,
    /// Monitor packages for changes (future feature).
    Monitor,
}

impl fmt::Display for WaveMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WaveMode::Validate => write!(f, "validate"),
            WaveMode::Hunt => write!(f, "hunt"),
            WaveMode::Monitor => write!(f, "monitor"),
        }
    }
}

/// Wave execution status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaveStatus {
    #[default]
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

impl fmt::Display for WaveStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WaveStatus::Pending => write!(f, "pending"),
            WaveStatus::Running => write!(f, "running"),
            WaveStatus::Completed => write!(f, "completed"),
            WaveStatus::Failed => write!(f, "failed"),
            WaveStatus::Skipped => write!(f, "skipped"),
        }
    }
}

/// Package scanning stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageStage {
    #[default]
    Downloading,
    Scanning,
    LlmAnalysis,
    Complete,
}

impl fmt::Display for PackageStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageStage::Downloading => write!(f, "downloading"),
            PackageStage::Scanning => write!(f, "scanning"),
            PackageStage::LlmAnalysis => write!(f, "llm_analysis"),
            PackageStage::Complete => write!(f, "complete"),
        }
    }
}

/// Sort order for npm package sampling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    #[default]
    Recent,
    Popular,
    Random,
}

/// Sort order for GitHub repository search.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GitHubSort {
    #[default]
    Stars,
    Forks,
    Updated,
}

/// Wave state tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveState {
    /// Unique wave identifier.
    pub id: String,
    /// Human-readable wave name.
    pub name: String,
    /// Current execution status.
    pub status: WaveStatus,
    /// Execution mode (validate, hunt, monitor).
    pub mode: WaveMode,
    /// Total packages to scan.
    pub packages_total: usize,
    /// Packages successfully scanned.
    pub packages_scanned: usize,
    /// Packages with any findings.
    pub packages_flagged: usize,
    /// Packages marked as malicious.
    pub packages_malicious: usize,
    /// When the wave started.
    pub started_at: Option<DateTime<Utc>>,
    /// When the wave completed.
    pub completed_at: Option<DateTime<Utc>>,
    /// Error message if the wave failed.
    pub error_message: Option<String>,
}

impl WaveState {
    /// Create a new wave state in pending status.
    pub fn new(id: impl Into<String>, name: impl Into<String>, mode: WaveMode) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            status: WaveStatus::Pending,
            mode,
            packages_total: 0,
            packages_scanned: 0,
            packages_flagged: 0,
            packages_malicious: 0,
            started_at: None,
            completed_at: None,
            error_message: None,
        }
    }

    /// Get progress as a percentage (0.0 - 100.0).
    pub fn progress_percentage(&self) -> f64 {
        if self.packages_total == 0 {
            0.0
        } else {
            (self.packages_scanned as f64 / self.packages_total as f64) * 100.0
        }
    }

    /// Get estimated time remaining based on current scan rate.
    pub fn eta(&self) -> Option<Duration> {
        if self.started_at.is_none() || self.packages_scanned == 0 {
            return None;
        }

        let elapsed = Utc::now().signed_duration_since(self.started_at.unwrap());
        let elapsed_secs = elapsed.num_seconds() as f64;
        if elapsed_secs <= 0.0 || self.packages_scanned == 0 {
            return None;
        }

        let secs_per_package = elapsed_secs / self.packages_scanned as f64;
        let remaining_packages = self.packages_total.saturating_sub(self.packages_scanned);
        
        Some(Duration::from_secs((secs_per_package * remaining_packages as f64) as u64))
    }

    /// Mark the wave as started.
    pub fn start(&mut self) {
        self.status = WaveStatus::Running;
        self.started_at = Some(Utc::now());
    }

    /// Mark the wave as completed.
    pub fn complete(&mut self) {
        self.status = WaveStatus::Completed;
        self.completed_at = Some(Utc::now());
    }

    /// Mark the wave as failed.
    pub fn fail(&mut self, error: impl Into<String>) {
        self.status = WaveStatus::Failed;
        self.error_message = Some(error.into());
        self.completed_at = Some(Utc::now());
    }
}

/// Campaign-level statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CampaignStats {
    /// Total packages to scan across all waves.
    pub packages_total: usize,
    /// Packages successfully scanned.
    pub packages_scanned: usize,
    /// Packages with any findings.
    pub packages_flagged: usize,
    /// Packages marked as malicious.
    pub packages_malicious: usize,
    /// Total findings across all packages.
    pub findings_total: usize,
    /// Number of LLM analyses performed.
    pub llm_analyses_run: usize,
    /// Number of evidence packages collected.
    pub evidence_collected: usize,
    /// Estimated time remaining (seconds).
    pub eta_seconds: Option<u64>,
    /// Scan rate (packages per minute).
    pub scan_rate_per_minute: f32,
}

impl CampaignStats {
    /// Calculate scan rate based on elapsed time.
    pub fn update_scan_rate(&mut self, started_at: DateTime<Utc>) {
        let elapsed = Utc::now().signed_duration_since(started_at);
        let elapsed_minutes = elapsed.num_seconds() as f32 / 60.0;
        
        if elapsed_minutes > 0.0 {
            self.scan_rate_per_minute = self.packages_scanned as f32 / elapsed_minutes;
        }
    }

    /// Calculate ETA based on remaining packages and current rate.
    pub fn update_eta(&mut self, started_at: DateTime<Utc>) {
        let remaining = self.packages_total.saturating_sub(self.packages_scanned);
        
        if self.scan_rate_per_minute > 0.0 && remaining > 0 {
            let eta_minutes = remaining as f32 / self.scan_rate_per_minute;
            self.eta_seconds = Some((eta_minutes * 60.0) as u64);
        } else {
            self.eta_seconds = None;
        }
    }
}

/// Active package being processed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivePackage {
    /// Package name (e.g., "express").
    pub name: String,
    /// Package version (e.g., "4.19.2").
    pub version: String,
    /// Wave ID this package belongs to.
    pub wave_id: String,
    /// Current processing stage.
    pub stage: PackageStage,
    /// When processing started.
    pub started_at: DateTime<Utc>,
}

impl ActivePackage {
    /// Create a new active package entry.
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        wave_id: impl Into<String>,
        stage: PackageStage,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            wave_id: wave_id.into(),
            stage,
            started_at: Utc::now(),
        }
    }

    /// Get elapsed time since processing started.
    pub fn elapsed(&self) -> Duration {
        Utc::now().signed_duration_since(self.started_at)
            .to_std()
            .unwrap_or(Duration::ZERO)
    }
}

/// In-memory campaign state (queryable by TUI/CLI).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignState {
    /// Unique case identifier.
    pub case_id: String,
    /// Campaign name from config.
    pub campaign_name: String,
    /// Current execution status.
    pub status: CampaignStatus,
    /// When the campaign started.
    pub started_at: DateTime<Utc>,
    /// When the campaign completed (if terminal).
    pub completed_at: Option<DateTime<Utc>>,
    /// Currently executing wave ID.
    pub current_wave: Option<String>,
    /// State of all waves in the campaign.
    pub waves: HashMap<String, WaveState>,
    /// Campaign-level statistics.
    pub stats: CampaignStats,
    /// Currently processing package (if any).
    pub active_package: Option<ActivePackage>,
    /// Hash of the campaign config file.
    pub config_hash: String,
}

impl CampaignState {
    /// Create a new campaign state.
    pub fn new(case_id: impl Into<String>, campaign_name: impl Into<String>) -> Self {
        Self {
            case_id: case_id.into(),
            campaign_name: campaign_name.into(),
            status: CampaignStatus::Initializing,
            started_at: Utc::now(),
            completed_at: None,
            current_wave: None,
            waves: HashMap::new(),
            stats: CampaignStats::default(),
            active_package: None,
            config_hash: String::new(),
        }
    }

    /// Get overall progress as a percentage.
    pub fn progress_percentage(&self) -> f64 {
        if self.stats.packages_total == 0 {
            0.0
        } else {
            (self.stats.packages_scanned as f64 / self.stats.packages_total as f64) * 100.0
        }
    }

    /// Get a wave state by ID.
    pub fn get_wave(&self, wave_id: &str) -> Option<&WaveState> {
        self.waves.get(wave_id)
    }

    /// Get a mutable wave state by ID.
    pub fn get_wave_mut(&mut self, wave_id: &str) -> Option<&mut WaveState> {
        self.waves.get_mut(wave_id)
    }

    /// Calculate aggregate statistics from all waves.
    pub fn recalculate_stats(&mut self) {
        let mut total = 0;
        let mut scanned = 0;
        let mut flagged = 0;
        let mut malicious = 0;

        for wave in self.waves.values() {
            total += wave.packages_total;
            scanned += wave.packages_scanned;
            flagged += wave.packages_flagged;
            malicious += wave.packages_malicious;
        }

        self.stats.packages_total = total;
        self.stats.packages_scanned = scanned;
        self.stats.packages_flagged = flagged;
        self.stats.packages_malicious = malicious;
        
        // Update derived metrics
        self.stats.update_scan_rate(self.started_at);
        self.stats.update_eta(self.started_at);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_id_generation() {
        let id1 = CaseId::new();
        let id2 = CaseId::new();
        assert_ne!(id1, id2);
        assert_eq!(id1.as_str().len(), 36); // UUID length
    }

    #[test]
    fn test_case_id_with_timestamp() {
        let id = CaseId::with_timestamp("wave6");
        assert!(id.as_str().starts_with("wave6-"));
    }

    #[test]
    fn test_campaign_status_is_terminal() {
        assert!(!CampaignStatus::Running.is_terminal());
        assert!(!CampaignStatus::Paused.is_terminal());
        assert!(CampaignStatus::Completed.is_terminal());
        assert!(CampaignStatus::Failed.is_terminal());
        assert!(CampaignStatus::Cancelled.is_terminal());
    }

    #[test]
    fn test_wave_state_progress() {
        let mut wave = WaveState::new("w1", "Test Wave", WaveMode::Hunt);
        wave.packages_total = 100;
        wave.packages_scanned = 50;
        
        assert!((wave.progress_percentage() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_campaign_state_aggregate_stats() {
        let mut state = CampaignState::new("test", "Test Campaign");
        
        let mut wave1 = WaveState::new("w1", "Wave 1", WaveMode::Hunt);
        wave1.packages_total = 100;
        wave1.packages_scanned = 50;
        wave1.packages_flagged = 5;
        wave1.packages_malicious = 2;
        
        let mut wave2 = WaveState::new("w2", "Wave 2", WaveMode::Hunt);
        wave2.packages_total = 200;
        wave2.packages_scanned = 100;
        wave2.packages_flagged = 10;
        wave2.packages_malicious = 3;
        
        state.waves.insert("w1".to_string(), wave1);
        state.waves.insert("w2".to_string(), wave2);
        
        state.recalculate_stats();
        
        assert_eq!(state.stats.packages_total, 300);
        assert_eq!(state.stats.packages_scanned, 150);
        assert_eq!(state.stats.packages_flagged, 15);
        assert_eq!(state.stats.packages_malicious, 5);
    }
}
