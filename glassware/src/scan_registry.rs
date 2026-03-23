//! Scan Registry - Track scan history and current state
//!
//! Provides visibility into:
//! - Currently running scans
//! - Completed scans
//! - Failed scans
//! - Scan history for auditing

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::error::{OrchestratorError, Result};

/// Unique scan identifier
pub type ScanId = String;

/// Scan status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScanStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for ScanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanStatus::Running => write!(f, "running"),
            ScanStatus::Completed => write!(f, "completed"),
            ScanStatus::Failed => write!(f, "failed"),
            ScanStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Scan record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanRecord {
    pub id: ScanId,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: ScanStatus,
    pub command: String,
    pub packages: Vec<String>,
    pub version_policy: Option<String>,
    pub findings_count: usize,
    pub malicious_count: usize,
    pub error: Option<String>,
}

impl ScanRecord {
    /// Create a new scan record
    pub fn new(command: &str, packages: &[String], version_policy: Option<&str>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            started_at: Utc::now(),
            completed_at: None,
            status: ScanStatus::Running,
            command: command.to_string(),
            packages: packages.to_vec(),
            version_policy: version_policy.map(String::from),
            findings_count: 0,
            malicious_count: 0,
            error: None,
        }
    }
    
    /// Mark scan as completed
    pub fn complete(&mut self, findings: usize, malicious: usize) {
        self.status = ScanStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.findings_count = findings;
        self.malicious_count = malicious;
    }
    
    /// Mark scan as failed
    pub fn fail(&mut self, error: String) {
        self.status = ScanStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.error = Some(error);
    }
    
    /// Mark scan as cancelled
    pub fn cancel(&mut self) {
        self.status = ScanStatus::Cancelled;
        self.completed_at = Some(Utc::now());
    }
}

/// Scan registry
pub struct ScanRegistry {
    state_file: PathBuf,
    scans: Vec<ScanRecord>,
}

impl ScanRegistry {
    /// Create or load scan registry
    pub fn new(state_file: Option<&Path>) -> Result<Self> {
        let state_file = state_file
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".glassware-scan-registry.json"));
        
        let scans = if state_file.exists() {
            Self::load_from_file(&state_file)?
        } else {
            vec![]
        };
        
        Ok(Self { state_file, scans })
    }
    
    /// Load scans from file
    fn load_from_file(path: &Path) -> Result<Vec<ScanRecord>> {
        let content = fs::read_to_string(path).map_err(|e| {
            OrchestratorError::io_error(e, format!("Failed to read scan registry: {}", path.display()))
        })?;
        
        #[derive(Deserialize)]
        struct RegistryFile {
            scans: Vec<ScanRecord>,
        }
        
        let registry: RegistryFile = serde_json::from_str(&content).map_err(|e| {
            OrchestratorError::json_error(e, format!("Failed to parse scan registry: {}", path.display()))
        })?;
        
        Ok(registry.scans)
    }
    
    /// Save scans to file
    fn save_to_file(&self) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.state_file.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                OrchestratorError::io_error(e, "Failed to create scan registry directory")
            })?;
        }
        
        #[derive(Serialize)]
        struct RegistryFile {
            scans: Vec<ScanRecord>,
        }
        
        let registry = RegistryFile {
            scans: self.scans.clone(),
        };
        
        let content = serde_json::to_string_pretty(&registry).map_err(|e| {
            OrchestratorError::json_error(e, "Failed to serialize scan registry")
        })?;
        
        fs::write(&self.state_file, content).map_err(|e| {
            OrchestratorError::io_error(e, format!("Failed to write scan registry: {}", self.state_file.display()))
        })?;
        
        Ok(())
    }
    
    /// Start a new scan
    pub fn start_scan(&mut self, command: &str, packages: &[String], version_policy: Option<&str>) -> ScanId {
        let record = ScanRecord::new(command, packages, version_policy);
        let id = record.id.clone();
        self.scans.push(record);
        
        // Save immediately
        if let Err(e) = self.save_to_file() {
            tracing::warn!("Failed to save scan registry: {}", e);
        }
        
        id
    }
    
    /// Complete a scan
    pub fn complete_scan(&mut self, id: &str, findings: usize, malicious: usize) -> Result<()> {
        if let Some(record) = self.scans.iter_mut().find(|r| r.id == id) {
            record.complete(findings, malicious);
            self.save_to_file()?;
        } else {
            return Err(OrchestratorError::validation_error(
                format!("Scan not found: {}", id),
                Some("scan_id"),
            ));
        }
        Ok(())
    }
    
    /// Fail a scan
    pub fn fail_scan(&mut self, id: &str, error: String) -> Result<()> {
        if let Some(record) = self.scans.iter_mut().find(|r| r.id == id) {
            record.fail(error);
            self.save_to_file()?;
        } else {
            return Err(OrchestratorError::validation_error(
                format!("Scan not found: {}", id),
                Some("scan_id"),
            ));
        }
        Ok(())
    }
    
    /// Cancel a scan
    pub fn cancel_scan(&mut self, id: &str) -> Result<()> {
        if let Some(record) = self.scans.iter_mut().find(|r| r.id == id) {
            record.cancel();
            self.save_to_file()?;
        } else {
            return Err(OrchestratorError::validation_error(
                format!("Scan not found: {}", id),
                Some("scan_id"),
            ));
        }
        Ok(())
    }
    
    /// List scans by status
    pub fn list_scans(&self, status: Option<ScanStatus>) -> Vec<&ScanRecord> {
        self.scans
            .iter()
            .filter(|s| status.as_ref().map_or(true, |expected_status| s.status == *expected_status))
            .collect()
    }
    
    /// Get running scans
    pub fn get_running_scans(&self) -> Vec<&ScanRecord> {
        self.list_scans(Some(ScanStatus::Running))
    }
    
    /// Get scan by ID
    pub fn get_scan(&self, id: &str) -> Option<&ScanRecord> {
        self.scans.iter().find(|r| r.id == id)
    }
    
    /// Get scan history (completed scans)
    pub fn get_history(&self, limit: usize) -> Vec<&ScanRecord> {
        self.scans
            .iter()
            .filter(|s| s.status == ScanStatus::Completed)
            .take(limit)
            .collect()
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> ScanStats {
        let mut stats = ScanStats::default();
        
        for scan in &self.scans {
            match scan.status {
                ScanStatus::Running => stats.running += 1,
                ScanStatus::Completed => {
                    stats.completed += 1;
                    stats.total_findings += scan.findings_count;
                    stats.total_malicious += scan.malicious_count;
                }
                ScanStatus::Failed => stats.failed += 1,
                ScanStatus::Cancelled => stats.cancelled += 1,
            }
        }
        
        stats
    }
}

/// Scan statistics
#[derive(Debug, Clone, Default)]
pub struct ScanStats {
    pub running: usize,
    pub completed: usize,
    pub failed: usize,
    pub cancelled: usize,
    pub total_findings: usize,
    pub total_malicious: usize,
}

impl std::fmt::Display for ScanStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Scan Statistics:")?;
        writeln!(f, "  Running: {}", self.running)?;
        writeln!(f, "  Completed: {}", self.completed)?;
        writeln!(f, "  Failed: {}", self.failed)?;
        writeln!(f, "  Cancelled: {}", self.cancelled)?;
        writeln!(f, "  Total findings: {}", self.total_findings)?;
        writeln!(f, "  Total malicious: {}", self.total_malicious)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_scan_registry_creation() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("scan-registry.json");
        
        let registry = ScanRegistry::new(Some(&state_file)).unwrap();
        assert!(registry.scans.is_empty());
    }
    
    #[test]
    fn test_scan_registry_start_complete() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("scan-registry.json");
        
        let mut registry = ScanRegistry::new(Some(&state_file)).unwrap();
        
        let id = registry.start_scan("scan-npm test-pkg", &["test-pkg".to_string()], None);
        assert!(!id.is_empty());
        
        registry.complete_scan(&id, 5, 1).unwrap();
        
        let scan = registry.get_scan(&id).unwrap();
        assert_eq!(scan.status, ScanStatus::Completed);
        assert_eq!(scan.findings_count, 5);
        assert_eq!(scan.malicious_count, 1);
    }
    
    #[test]
    fn test_scan_registry_fail() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("scan-registry.json");
        
        let mut registry = ScanRegistry::new(Some(&state_file)).unwrap();
        
        let id = registry.start_scan("scan-npm test-pkg", &["test-pkg".to_string()], None);
        
        registry.fail_scan(&id, "Download failed".to_string()).unwrap();
        
        let scan = registry.get_scan(&id).unwrap();
        assert_eq!(scan.status, ScanStatus::Failed);
        assert_eq!(scan.error, Some("Download failed".to_string()));
    }
    
    #[test]
    fn test_scan_registry_list() {
        let temp_dir = TempDir::new().unwrap();
        let state_file = temp_dir.path().join("scan-registry.json");
        
        let mut registry = ScanRegistry::new(Some(&state_file)).unwrap();
        
        // Start and complete some scans
        let id1 = registry.start_scan("scan-npm pkg1", &["pkg1".to_string()], None);
        registry.complete_scan(&id1, 0, 0).unwrap();
        
        let id2 = registry.start_scan("scan-npm pkg2", &["pkg2".to_string()], None);
        registry.complete_scan(&id2, 5, 1).unwrap();
        
        // Start a running scan
        let _id3 = registry.start_scan("scan-npm pkg3", &["pkg3".to_string()], None);
        
        // List by status
        let completed = registry.list_scans(Some(ScanStatus::Completed));
        assert_eq!(completed.len(), 2);
        
        let running = registry.get_running_scans();
        assert_eq!(running.len(), 1);
        
        // Get stats
        let stats = registry.get_stats();
        assert_eq!(stats.completed, 2);
        assert_eq!(stats.running, 1);
        assert_eq!(stats.total_findings, 5);
        assert_eq!(stats.total_malicious, 1);
    }
}
