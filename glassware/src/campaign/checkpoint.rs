//! Campaign checkpoint persistence.
//!
//! Provides SQLite-based checkpoint storage for campaign resume functionality.

use rusqlite::{Connection, params};
use serde::{Serialize, Deserialize};
use std::path::Path;
use tracing::{debug, info, warn};

/// Campaign checkpoint data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignCheckpoint {
    pub case_id: String,
    pub campaign_name: String,
    pub status: String,
    pub config_json: String,
    pub completed_waves: Vec<String>,
    pub current_wave: Option<String>,
    pub wave_states: String, // JSON-encoded wave states
    pub created_at: String,
    pub updated_at: String,
}

/// Checkpoint manager for persistence.
pub struct CheckpointManager {
    conn: Connection,
}

impl CheckpointManager {
    /// Create or open checkpoint database.
    pub fn new(db_path: &Path) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(db_path)?;
        
        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS checkpoints (
                case_id TEXT PRIMARY KEY,
                campaign_name TEXT NOT NULL,
                status TEXT NOT NULL,
                config_json TEXT NOT NULL,
                completed_waves TEXT NOT NULL,
                current_wave TEXT,
                wave_states TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        
        Ok(Self { conn })
    }

    /// Save campaign checkpoint.
    pub fn save_checkpoint(&self, checkpoint: &CampaignCheckpoint) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "INSERT OR REPLACE INTO checkpoints 
             (case_id, campaign_name, status, config_json, completed_waves, 
              current_wave, wave_states, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                checkpoint.case_id,
                checkpoint.campaign_name,
                checkpoint.status,
                checkpoint.config_json,
                serde_json::to_string(&checkpoint.completed_waves).unwrap_or_default(),
                checkpoint.current_wave,
                checkpoint.wave_states,
                checkpoint.created_at,
                checkpoint.updated_at,
            ],
        )?;
        
        debug!("Saved checkpoint for campaign: {}", checkpoint.case_id);
        Ok(())
    }

    /// Load campaign checkpoint.
    pub fn load_checkpoint(&self, case_id: &str) -> Result<Option<CampaignCheckpoint>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT case_id, campaign_name, status, config_json, completed_waves,
                    current_wave, wave_states, created_at, updated_at
             FROM checkpoints
             WHERE case_id = ?1"
        )?;
        
        let checkpoint = stmt.query_row(params![case_id], |row| {
            Ok(CampaignCheckpoint {
                case_id: row.get(0)?,
                campaign_name: row.get(1)?,
                status: row.get(2)?,
                config_json: row.get(3)?,
                completed_waves: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                current_wave: row.get(5)?,
                wave_states: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        });
        
        match checkpoint {
            Ok(cp) => {
                debug!("Loaded checkpoint for campaign: {}", case_id);
                Ok(Some(cp))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// List all checkpoints.
    pub fn list_checkpoints(&self) -> Result<Vec<CampaignCheckpoint>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT case_id, campaign_name, status, config_json, completed_waves,
                    current_wave, wave_states, created_at, updated_at
             FROM checkpoints
             ORDER BY updated_at DESC"
        )?;
        
        let checkpoints = stmt.query_map([], |row| {
            Ok(CampaignCheckpoint {
                case_id: row.get(0)?,
                campaign_name: row.get(1)?,
                status: row.get(2)?,
                config_json: row.get(3)?,
                completed_waves: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                current_wave: row.get(5)?,
                wave_states: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;
        
        checkpoints.collect()
    }

    /// Delete a checkpoint.
    pub fn delete_checkpoint(&self, case_id: &str) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "DELETE FROM checkpoints WHERE case_id = ?1",
            params![case_id],
        )?;
        
        debug!("Deleted checkpoint: {}", case_id);
        Ok(())
    }

    /// Update campaign status.
    pub fn update_status(&self, case_id: &str, status: &str) -> Result<(), rusqlite::Error> {
        let updated_at = chrono::Utc::now().to_rfc3339();
        
        self.conn.execute(
            "UPDATE checkpoints SET status = ?1, updated_at = ?2 WHERE case_id = ?3",
            params![status, updated_at, case_id],
        )?;
        
        Ok(())
    }

    /// Add completed wave.
    pub fn add_completed_wave(&self, case_id: &str, wave_id: &str) -> Result<(), rusqlite::Error> {
        // Get current checkpoint
        if let Some(mut cp) = self.load_checkpoint(case_id)? {
            if !cp.completed_waves.contains(&wave_id.to_string()) {
                cp.completed_waves.push(wave_id.to_string());
                cp.updated_at = chrono::Utc::now().to_rfc3339();
                self.save_checkpoint(&cp)?;
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_checkpoint_save_load() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("checkpoints.db");
        
        let manager = CheckpointManager::new(&db_path).unwrap();
        
        let checkpoint = CampaignCheckpoint {
            case_id: "test-123".to_string(),
            campaign_name: "Test Campaign".to_string(),
            status: "running".to_string(),
            config_json: "{}".to_string(),
            completed_waves: vec!["wave1".to_string()],
            current_wave: Some("wave2".to_string()),
            wave_states: "{}".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        
        // Save
        manager.save_checkpoint(&checkpoint).unwrap();
        
        // Load
        let loaded = manager.load_checkpoint("test-123").unwrap().unwrap();
        
        assert_eq!(loaded.case_id, "test-123");
        assert_eq!(loaded.campaign_name, "Test Campaign");
        assert_eq!(loaded.completed_waves, vec!["wave1"]);
        assert_eq!(loaded.current_wave, Some("wave2".to_string()));
    }

    #[test]
    fn test_checkpoint_list() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("checkpoints.db");
        
        let manager = CheckpointManager::new(&db_path).unwrap();
        
        // Save multiple checkpoints
        for i in 0..3 {
            let checkpoint = CampaignCheckpoint {
                case_id: format!("test-{}", i),
                campaign_name: format!("Campaign {}", i),
                status: "completed".to_string(),
                config_json: "{}".to_string(),
                completed_waves: vec![],
                current_wave: None,
                wave_states: "{}".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            };
            manager.save_checkpoint(&checkpoint).unwrap();
        }
        
        // List
        let checkpoints = manager.list_checkpoints().unwrap();
        assert_eq!(checkpoints.len(), 3);
    }
}
