//! Per-Version Checkpoint Module
//!
//! Provides checkpoint/resume functionality at the version level for long-running scans.
//!
//! ## Features
//!
//! - Save progress after each version scan
//! - Resume from interrupted scans
//! - JSON-based checkpoint files
//! - Automatic cleanup on completion
//!
//! ## Usage
//!
//! ```rust
//! use orchestrator_core::checkpoint::VersionCheckpoint;
//!
//! let mut checkpoint = VersionCheckpoint::load_or_create("scan-checkpoint.json")?;
//!
//! for version in versions {
//!     if checkpoint.is_scanned(&version) {
//!         continue; // Skip already scanned versions
//!     }
//!
//!     // Scan version...
//!     let result = scan_version(version).await?;
//!
//!     // Mark as scanned
//!     checkpoint.mark_scanned(version, &result);
//!
//!     // Auto-save every 10 versions
//!     if checkpoint.should_save() {
//!         checkpoint.save()?;
//!     }
//! }
//!
//! checkpoint.save()?; // Final save
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{OrchestratorError, Result};

/// Checkpoint data for a single version scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionScanRecord {
    /// Package name
    pub package: String,
    /// Version string
    pub version: String,
    /// Scan timestamp
    pub scanned_at: DateTime<Utc>,
    /// Number of findings
    pub findings_count: usize,
    /// Threat score
    pub threat_score: f32,
    /// Is malicious
    pub is_malicious: bool,
    /// Error message if scan failed
    pub error: Option<String>,
}

/// Per-version checkpoint manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionCheckpoint {
    /// Checkpoint file path
    #[serde(skip)]
    pub checkpoint_file: PathBuf,

    /// Set of scanned version keys (package@version)
    pub scanned_versions: HashSet<String>,

    /// Scan records
    pub scan_records: Vec<VersionScanRecord>,

    /// Total versions to scan
    pub total_versions: usize,

    /// Scan start time
    pub started_at: DateTime<Utc>,

    /// Last save time
    pub last_saved_at: Option<DateTime<Utc>>,

    /// Auto-save interval (versions between saves)
    pub auto_save_interval: usize,

    /// Versions scanned since last save
    #[serde(skip)]
    pub versions_since_save: usize,
}

impl VersionCheckpoint {
    /// Create new checkpoint
    pub fn new(checkpoint_file: &Path, total_versions: usize) -> Self {
        Self {
            checkpoint_file: checkpoint_file.to_path_buf(),
            scanned_versions: HashSet::new(),
            scan_records: Vec::new(),
            total_versions,
            started_at: Utc::now(),
            last_saved_at: None,
            auto_save_interval: 10,
            versions_since_save: 0,
        }
    }

    /// Load existing checkpoint or create new one
    pub fn load_or_create(checkpoint_file: &Path, total_versions: usize) -> Result<Self> {
        if checkpoint_file.exists() {
            Self::load(checkpoint_file)
        } else {
            Ok(Self::new(checkpoint_file, total_versions))
        }
    }

    /// Load checkpoint from file
    pub fn load(checkpoint_file: &Path) -> Result<Self> {
        let content = fs::read_to_string(checkpoint_file).map_err(|e| {
            OrchestratorError::io_error(
                e,
                format!("Failed to read checkpoint file: {:?}", checkpoint_file),
            )
        })?;

        let mut checkpoint: Self = serde_json::from_str(&content).map_err(|e| {
            OrchestratorError::json_error(
                e,
                format!("Failed to parse checkpoint file: {:?}", checkpoint_file),
            )
        })?;

        // Ensure checkpoint file path is set
        checkpoint.checkpoint_file = checkpoint_file.to_path_buf();

        // Reset runtime counters
        checkpoint.versions_since_save = 0;

        Ok(checkpoint)
    }

    /// Save checkpoint to file
    pub fn save(&self) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.checkpoint_file.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                OrchestratorError::io_error(
                    e,
                    format!("Failed to create checkpoint directory: {:?}", parent),
                )
            })?;
        }

        let content = serde_json::to_string_pretty(self).map_err(|e| {
            OrchestratorError::json_error(e, "Failed to serialize checkpoint".to_string())
        })?;

        fs::write(&self.checkpoint_file, content).map_err(|e| {
            OrchestratorError::io_error(
                e,
                format!("Failed to write checkpoint file: {:?}", self.checkpoint_file),
            )
        })?;

        tracing::debug!("Checkpoint saved to {:?}", self.checkpoint_file);
        Ok(())
    }

    /// Check if version has been scanned
    pub fn is_scanned(&self, package: &str, version: &str) -> bool {
        let key = format!("{}@{}", package, version);
        self.scanned_versions.contains(&key)
    }

    /// Mark version as scanned
    pub fn mark_scanned(
        &mut self,
        package: &str,
        version: &str,
        findings_count: usize,
        threat_score: f32,
        is_malicious: bool,
        error: Option<String>,
    ) {
        let key = format!("{}@{}", package, version);

        if !self.scanned_versions.contains(&key) {
            self.scanned_versions.insert(key.clone());

            let record = VersionScanRecord {
                package: package.to_string(),
                version: version.to_string(),
                scanned_at: Utc::now(),
                findings_count,
                threat_score,
                is_malicious,
                error,
            };

            self.scan_records.push(record);
            self.versions_since_save += 1;
        }
    }

    /// Check if auto-save is needed
    pub fn should_save(&self) -> bool {
        self.versions_since_save >= self.auto_save_interval
    }

    /// Reset save counter after saving
    pub fn reset_save_counter(&mut self) {
        self.versions_since_save = 0;
        self.last_saved_at = Some(Utc::now());
    }

    /// Get progress statistics
    pub fn get_progress(&self) -> CheckpointProgress {
        let scanned = self.scanned_versions.len();
        let remaining = self.total_versions.saturating_sub(scanned);
        let percentage = if self.total_versions > 0 {
            (scanned as f64 / self.total_versions as f64) * 100.0
        } else {
            0.0
        };

        let elapsed = Utc::now().signed_duration_since(self.started_at);
        let elapsed_secs = elapsed.num_seconds() as f64;
        let rate = if elapsed_secs > 0.0 {
            scanned as f64 / elapsed_secs
        } else {
            0.0
        };

        CheckpointProgress {
            total: self.total_versions,
            scanned,
            remaining,
            percentage,
            rate,
            elapsed_secs,
        }
    }

    /// Check if scan is complete
    pub fn is_complete(&self) -> bool {
        self.scanned_versions.len() >= self.total_versions
    }

    /// Get failed scans
    pub fn get_failed_scans(&self) -> Vec<&VersionScanRecord> {
        self.scan_records.iter().filter(|r| r.error.is_some()).collect()
    }

    /// Get malicious scans
    pub fn get_malicious_scans(&self) -> Vec<&VersionScanRecord> {
        self.scan_records
            .iter()
            .filter(|r| r.is_malicious)
            .collect()
    }

    /// Set auto-save interval
    pub fn with_auto_save_interval(mut self, interval: usize) -> Self {
        self.auto_save_interval = interval;
        self
    }
}

/// Progress statistics
#[derive(Debug, Clone)]
pub struct CheckpointProgress {
    pub total: usize,
    pub scanned: usize,
    pub remaining: usize,
    pub percentage: f64,
    pub rate: f64, // versions per second
    pub elapsed_secs: f64,
}

impl std::fmt::Display for CheckpointProgress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{} ({:.1}%) - {:.1} ver/s - {:.0}s elapsed",
            self.scanned,
            self.total,
            self.percentage,
            self.rate,
            self.elapsed_secs
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_checkpoint_creation() {
        let temp_dir = TempDir::new().unwrap();
        let checkpoint_file = temp_dir.path().join("checkpoint.json");

        let checkpoint = VersionCheckpoint::new(&checkpoint_file, 100);
        assert_eq!(checkpoint.total_versions, 100);
        assert!(checkpoint.scanned_versions.is_empty());
    }

    #[test]
    fn test_checkpoint_mark_scanned() {
        let temp_dir = TempDir::new().unwrap();
        let checkpoint_file = temp_dir.path().join("checkpoint.json");

        let mut checkpoint = VersionCheckpoint::new(&checkpoint_file, 100);
        checkpoint.mark_scanned("pkg", "1.0.0", 5, 2.5, false, None);

        assert!(checkpoint.is_scanned("pkg", "1.0.0"));
        assert!(!checkpoint.is_scanned("pkg", "1.0.1"));
        assert_eq!(checkpoint.scan_records.len(), 1);
    }

    #[test]
    fn test_checkpoint_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let checkpoint_file = temp_dir.path().join("checkpoint.json");

        let mut checkpoint = VersionCheckpoint::new(&checkpoint_file, 100);
        checkpoint.mark_scanned("pkg", "1.0.0", 5, 2.5, false, None);
        checkpoint.save().unwrap();

        let loaded = VersionCheckpoint::load(&checkpoint_file).unwrap();
        assert!(loaded.is_scanned("pkg", "1.0.0"));
        assert_eq!(loaded.scan_records.len(), 1);
    }

    #[test]
    fn test_checkpoint_progress() {
        let temp_dir = TempDir::new().unwrap();
        let checkpoint_file = temp_dir.path().join("checkpoint.json");

        let mut checkpoint = VersionCheckpoint::new(&checkpoint_file, 100);

        for i in 0..50 {
            checkpoint.mark_scanned("pkg", &format!("1.0.{}", i), 0, 0.0, false, None);
        }

        let progress = checkpoint.get_progress();
        assert_eq!(progress.scanned, 50);
        assert_eq!(progress.remaining, 50);
        assert!((progress.percentage - 50.0).abs() < 0.1);
    }
}
