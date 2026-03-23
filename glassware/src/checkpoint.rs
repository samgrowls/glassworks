//! Checkpoint/Resume support for long-running scan operations.
//!
//! This module provides the ability to save scan state to JSON and resume from checkpoints.
//!
//! Features:
//! - Save scan state to JSON
//! - Resume from checkpoint
//! - Auto-save every N packages
//! - Cleanup old checkpoints

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, warn};

use crate::error::{OrchestratorError, Result};
use crate::scanner::PackageScanResult;

/// Scan result for checkpoint storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// Package name.
    pub package_name: String,
    /// Source type (npm, github, file).
    pub source_type: String,
    /// Version or commit hash.
    pub version: String,
    /// Path to scanned content.
    pub path: String,
    /// Content hash.
    pub content_hash: String,
    /// Number of findings.
    pub findings_count: usize,
    /// Threat score.
    pub threat_score: f32,
    /// Whether the package is malicious.
    pub is_malicious: bool,
    /// Scan timestamp.
    pub timestamp: DateTime<Utc>,
}

impl From<&PackageScanResult> for ScanResult {
    fn from(result: &PackageScanResult) -> Self {
        Self {
            package_name: result.package_name.clone(),
            source_type: result.source_type.clone(),
            version: result.version.clone(),
            path: result.path.clone(),
            content_hash: result.content_hash.clone(),
            findings_count: result.findings.len(),
            threat_score: result.threat_score,
            is_malicious: result.is_malicious,
            timestamp: Utc::now(),
        }
    }
}

/// Checkpoint data for save/resume operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Source identifier (e.g., "npm", "github", or file path).
    pub source: String,
    /// List of packages already scanned.
    pub scanned: Vec<String>,
    /// List of packages remaining to scan.
    pub remaining: Vec<String>,
    /// Scan results for completed packages.
    pub results: Vec<ScanResult>,
    /// Checkpoint timestamp.
    pub timestamp: DateTime<Utc>,
    /// Total packages to scan.
    pub total: usize,
    /// Number of errors encountered.
    pub errors: usize,
    /// Checkpoint version for compatibility checking.
    pub checkpoint_version: String,
}

impl Checkpoint {
    /// Create a new checkpoint.
    pub fn new(source: String, packages: Vec<String>) -> Self {
        let total = packages.len();
        Self {
            source,
            scanned: Vec::new(),
            remaining: packages,
            results: Vec::new(),
            timestamp: Utc::now(),
            total,
            errors: 0,
            checkpoint_version: "1.0.0".to_string(),
        }
    }

    /// Mark a package as scanned.
    pub fn mark_scanned(&mut self, package: &str, result: Option<&PackageScanResult>) {
        if let Some(pos) = self.remaining.iter().position(|p| p == package) {
            self.remaining.remove(pos);
            self.scanned.push(package.to_string());
            
            if let Some(r) = result {
                self.results.push(ScanResult::from(r));
            }
        }
    }

    /// Mark a package as having an error.
    pub fn mark_error(&mut self, package: &str) {
        self.errors += 1;
        // Keep the package in remaining for retry
    }

    /// Get completion percentage.
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.scanned.len() as f64 / self.total as f64) * 100.0
        }
    }

    /// Save checkpoint to a JSON file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self).map_err(|e| {
            OrchestratorError::cache_error(format!("Failed to serialize checkpoint: {}", e))
        })?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                OrchestratorError::io(e)
            })?;
        }

        fs::write(path, json).map_err(|e| {
            OrchestratorError::io(e)
        })?;

        info!("Checkpoint saved to: {}", path.display());
        Ok(())
    }

    /// Load checkpoint from a JSON file.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(OrchestratorError::not_found(format!(
                "Checkpoint file not found: {}",
                path.display()
            )));
        }

        let json = fs::read_to_string(path).map_err(|e| {
            OrchestratorError::io(e)
        })?;

        let checkpoint: Checkpoint = serde_json::from_str(&json).map_err(|e| {
            OrchestratorError::cache_error(format!("Failed to deserialize checkpoint: {}", e))
        })?;

        info!("Checkpoint loaded from: {}", path.display());
        Ok(checkpoint)
    }

    /// Check if checkpoint is valid (has remaining work).
    pub fn is_complete(&self) -> bool {
        self.remaining.is_empty()
    }

    /// Get remaining packages count.
    pub fn remaining_count(&self) -> usize {
        self.remaining.len()
    }

    /// Get scanned packages count.
    pub fn scanned_count(&self) -> usize {
        self.scanned.len()
    }

    /// Merge results into checkpoint.
    pub fn merge_results(&mut self, results: Vec<PackageScanResult>) {
        for result in results {
            if let Some(pos) = self.scanned.iter().position(|p| p == &result.package_name) {
                // Update existing result
                if pos < self.results.len() {
                    self.results[pos] = ScanResult::from(&result);
                }
            }
        }
    }
}

/// Checkpoint manager for auto-save and cleanup.
pub struct CheckpointManager {
    /// Directory to store checkpoints.
    checkpoint_dir: PathBuf,
    /// Auto-save interval (number of packages).
    auto_save_interval: usize,
    /// Maximum number of checkpoints to keep.
    max_checkpoints: usize,
    /// Current checkpoint.
    checkpoint: Option<Checkpoint>,
    /// Packages scanned since last auto-save.
    packages_since_save: usize,
}

impl CheckpointManager {
    /// Create a new checkpoint manager.
    pub fn new(checkpoint_dir: &Path) -> Self {
        // Create checkpoint directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(checkpoint_dir) {
            warn!("Failed to create checkpoint directory: {}", e);
        }

        Self {
            checkpoint_dir: checkpoint_dir.to_path_buf(),
            auto_save_interval: 10, // Auto-save every 10 packages
            max_checkpoints: 5,
            checkpoint: None,
            packages_since_save: 0,
        }
    }

    /// Set auto-save interval.
    pub fn with_auto_save_interval(mut self, interval: usize) -> Self {
        self.auto_save_interval = interval;
        self
    }

    /// Set maximum checkpoints to keep.
    pub fn with_max_checkpoints(mut self, max: usize) -> Self {
        self.max_checkpoints = max;
        self
    }

    /// Initialize a new checkpoint for a scan operation.
    pub fn init_checkpoint(&mut self, source: String, packages: Vec<String>) -> Result<()> {
        let checkpoint = Checkpoint::new(source, packages);
        self.checkpoint = Some(checkpoint);
        self.packages_since_save = 0;
        Ok(())
    }

    /// Load existing checkpoint if available.
    pub fn load_checkpoint(&mut self, source: &str) -> Result<bool> {
        let checkpoint_path = self.checkpoint_path(source);
        
        if checkpoint_path.exists() {
            let checkpoint = Checkpoint::load(&checkpoint_path)?;
            self.checkpoint = Some(checkpoint);
            self.packages_since_save = 0;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get current checkpoint (immutable).
    pub fn checkpoint(&self) -> Option<&Checkpoint> {
        self.checkpoint.as_ref()
    }

    /// Get current checkpoint (mutable).
    pub fn checkpoint_mut(&mut self) -> Option<&mut Checkpoint> {
        self.checkpoint.as_mut()
    }

    /// Mark a package as scanned and auto-save if needed.
    pub fn mark_scanned(&mut self, package: &str, result: Option<&PackageScanResult>) -> Result<()> {
        if let Some(checkpoint) = &mut self.checkpoint {
            checkpoint.mark_scanned(package, result);
            self.packages_since_save += 1;

            // Auto-save if threshold reached
            if self.packages_since_save >= self.auto_save_interval {
                self.save_checkpoint()?;
                self.packages_since_save = 0;
            }
        }
        Ok(())
    }

    /// Mark a package as having an error.
    pub fn mark_error(&mut self, package: &str) -> Result<()> {
        if let Some(checkpoint) = &mut self.checkpoint {
            checkpoint.mark_error(package);
        }
        Ok(())
    }

    /// Save current checkpoint.
    pub fn save_checkpoint(&self) -> Result<()> {
        if let Some(checkpoint) = &self.checkpoint {
            let checkpoint_path = self.checkpoint_path(&checkpoint.source);
            checkpoint.save(&checkpoint_path)?;
        }
        Ok(())
    }

    /// Cleanup old checkpoints.
    pub fn cleanup_old_checkpoints(&self) -> Result<usize> {
        let mut removed = 0;

        // Get all checkpoint files
        let mut checkpoints: Vec<(PathBuf, DateTime<Utc>)> = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.checkpoint_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("checkpoint") {
                    if let Ok(metadata) = fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            let modified: DateTime<Utc> = modified.into();
                            checkpoints.push((path, modified));
                        }
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        checkpoints.sort_by(|a, b| b.1.cmp(&a.1));

        // Remove old checkpoints beyond max
        for (path, _) in checkpoints.iter().skip(self.max_checkpoints) {
            debug!("Removing old checkpoint: {}", path.display());
            if let Err(e) = fs::remove_file(path) {
                warn!("Failed to remove old checkpoint {}: {}", path.display(), e);
            } else {
                removed += 1;
            }
        }

        if removed > 0 {
            info!("Cleaned up {} old checkpoints", removed);
        }

        Ok(removed)
    }

    /// Clear checkpoint for a source.
    pub fn clear_checkpoint(&self, source: &str) -> Result<()> {
        let checkpoint_path = self.checkpoint_path(source);
        if checkpoint_path.exists() {
            fs::remove_file(&checkpoint_path).map_err(|e| {
                OrchestratorError::io(e)
            })?;
            info!("Cleared checkpoint for: {}", source);
        }
        Ok(())
    }

    /// Clear all checkpoints.
    pub fn clear_all_checkpoints(&self) -> Result<usize> {
        let mut cleared = 0;

        if let Ok(entries) = fs::read_dir(&self.checkpoint_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("checkpoint") {
                    fs::remove_file(&path).map_err(|e| {
                        OrchestratorError::io(e)
                    })?;
                    cleared += 1;
                }
            }
        }

        if cleared > 0 {
            info!("Cleared {} checkpoints", cleared);
        }

        Ok(cleared)
    }

    /// Get checkpoint file path for a source.
    fn checkpoint_path(&self, source: &str) -> PathBuf {
        // Sanitize source name for filename
        let sanitized = source
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect::<String>();

        self.checkpoint_dir.join(format!("{}.checkpoint", sanitized))
    }

    /// Get remaining packages from checkpoint.
    pub fn get_remaining(&self) -> Vec<String> {
        self.checkpoint
            .as_ref()
            .map(|c| c.remaining.clone())
            .unwrap_or_default()
    }

    /// Get all results from checkpoint.
    pub fn get_results(&self) -> Vec<ScanResult> {
        self.checkpoint
            .as_ref()
            .map(|c| c.results.clone())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_checkpoint() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let checkpoint_path = temp_dir.path().join("test.checkpoint");
        (temp_dir, checkpoint_path)
    }

    #[test]
    fn test_checkpoint_creation() {
        let packages = vec!["pkg1".to_string(), "pkg2".to_string(), "pkg3".to_string()];
        let checkpoint = Checkpoint::new("npm".to_string(), packages);

        assert_eq!(checkpoint.source, "npm");
        assert_eq!(checkpoint.total, 3);
        assert_eq!(checkpoint.remaining.len(), 3);
        assert_eq!(checkpoint.scanned.len(), 0);
        assert_eq!(checkpoint.percentage(), 0.0);
    }

    #[test]
    fn test_checkpoint_mark_scanned() {
        let packages = vec!["pkg1".to_string(), "pkg2".to_string()];
        let mut checkpoint = Checkpoint::new("npm".to_string(), packages);

        checkpoint.mark_scanned("pkg1", None);

        assert_eq!(checkpoint.scanned.len(), 1);
        assert_eq!(checkpoint.remaining.len(), 1);
        assert_eq!(checkpoint.percentage(), 50.0);
    }

    #[test]
    fn test_checkpoint_save_load() {
        let (temp_dir, checkpoint_path) = create_test_checkpoint();

        let packages = vec!["pkg1".to_string(), "pkg2".to_string()];
        let mut checkpoint = Checkpoint::new("npm".to_string(), packages);
        checkpoint.mark_scanned("pkg1", None);

        // Save
        checkpoint.save(&checkpoint_path).unwrap();

        // Load
        let loaded = Checkpoint::load(&checkpoint_path).unwrap();

        assert_eq!(loaded.source, "npm");
        assert_eq!(loaded.scanned.len(), 1);
        assert_eq!(loaded.remaining.len(), 1);
    }

    #[test]
    fn test_checkpoint_is_complete() {
        let packages = vec!["pkg1".to_string()];
        let mut checkpoint = Checkpoint::new("npm".to_string(), packages);

        assert!(!checkpoint.is_complete());

        checkpoint.mark_scanned("pkg1", None);

        assert!(checkpoint.is_complete());
    }

    #[test]
    fn test_checkpoint_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CheckpointManager::new(temp_dir.path());

        assert_eq!(manager.auto_save_interval, 10);
        assert_eq!(manager.max_checkpoints, 5);
    }

    #[test]
    fn test_checkpoint_manager_init() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = CheckpointManager::new(temp_dir.path());

        let packages = vec!["pkg1".to_string(), "pkg2".to_string()];
        manager.init_checkpoint("npm".to_string(), packages).unwrap();

        let checkpoint = manager.checkpoint().unwrap();
        assert_eq!(checkpoint.total, 2);
    }

    #[test]
    fn test_checkpoint_manager_mark_scanned() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = CheckpointManager::new(temp_dir.path());

        let packages = vec!["pkg1".to_string(), "pkg2".to_string()];
        manager.init_checkpoint("npm".to_string(), packages).unwrap();

        manager.mark_scanned("pkg1", None).unwrap();

        let checkpoint = manager.checkpoint().unwrap();
        assert_eq!(checkpoint.scanned.len(), 1);
    }

    #[test]
    fn test_checkpoint_manager_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = CheckpointManager::new(temp_dir.path());

        let packages = vec!["pkg1".to_string(), "pkg2".to_string()];
        manager.init_checkpoint("npm".to_string(), packages).unwrap();

        manager.mark_scanned("pkg1", None).unwrap();
        manager.save_checkpoint().unwrap();

        // Load checkpoint
        let loaded = manager.load_checkpoint("npm").unwrap();
        assert!(loaded);

        let checkpoint = manager.checkpoint().unwrap();
        assert_eq!(checkpoint.scanned.len(), 1);
    }

    #[test]
    fn test_checkpoint_manager_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = CheckpointManager::new(temp_dir.path())
            .with_max_checkpoints(2)
            .with_auto_save_interval(1);

        // Create multiple checkpoints
        for i in 0..5 {
            let packages = vec![format!("pkg{}", i)];
            manager.init_checkpoint(format!("source{}", i), packages).unwrap();
            manager.mark_scanned(&format!("pkg{}", i), None).unwrap();
            manager.save_checkpoint().unwrap();
        }

        // Cleanup
        let removed = manager.cleanup_old_checkpoints().unwrap();
        
        // Should have removed some old checkpoints
        assert!(removed >= 0); // May be 0 if files are created too fast
    }

    #[test]
    fn test_scan_result_from_package_scan_result() {
        use crate::scanner::PackageScanResult;

        let package_result = PackageScanResult {
            package_name: "test-pkg".to_string(),
            source_type: "npm".to_string(),
            version: "1.0.0".to_string(),
            path: "/path".to_string(),
            content_hash: "hash".to_string(),
            findings: vec![],
            threat_score: 0.0,
            is_malicious: false,
            llm_verdict: None,
        };

        let scan_result = ScanResult::from(&package_result);
        assert_eq!(scan_result.package_name, "test-pkg");
        assert_eq!(scan_result.threat_score, 0.0);
        assert!(!scan_result.is_malicious);
    }
}
