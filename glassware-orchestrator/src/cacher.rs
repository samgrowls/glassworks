//! SQLite caching module with 7-day TTL.
//!
//! This module provides persistent caching for scan results using SQLite,
//! with automatic cleanup of expired entries.

use chrono::{DateTime, Duration, Utc};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool, FromRow};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tracing::{debug, error, info, warn};

use crate::error::{OrchestratorError, Result};

/// Default cache TTL (time-to-live) in days.
const DEFAULT_TTL_DAYS: i64 = 7;

/// Cache entry for a scanned package or repository.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, FromRow)]
pub struct CacheEntry {
    /// Unique identifier for the cached item.
    pub key: String,
    /// Source type (npm, github, file).
    pub source_type: String,
    /// Scan result as JSON.
    pub result: String,
    /// Timestamp when the entry was created.
    #[sqlx(rename = "created_at")]
    pub created_at: DateTime<Utc>,
    /// Timestamp when the entry expires.
    #[sqlx(rename = "expires_at")]
    pub expires_at: DateTime<Utc>,
    /// Hash of the scanned content for validation.
    pub content_hash: Option<String>,
}

/// SQLite cache manager.
#[derive(Clone)]
pub struct Cacher {
    pool: Arc<SqlitePool>,
    ttl_days: i64,
}

impl Cacher {
    /// Create a new cacher with the default database path.
    pub async fn new() -> Result<Self> {
        Self::with_path(".glassware-orchestrator-cache.db").await
    }

    /// Create a new cacher with a custom database path.
    pub async fn with_path<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        Self::with_path_and_ttl(db_path, DEFAULT_TTL_DAYS).await
    }

    /// Create a new cacher with custom path and TTL.
    pub async fn with_path_and_ttl<P: AsRef<Path>>(
        db_path: P,
        ttl_days: i64,
    ) -> Result<Self> {
        let mut db_path = db_path.as_ref().to_path_buf();

        // Convert relative path to absolute path
        if db_path.is_relative() {
            if let Ok(current_dir) = std::env::current_dir() {
                db_path = current_dir.join(db_path);
            }
        }

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                OrchestratorError::cache_error(format!("Failed to create cache directory: {}", e))
            })?;
        }

        // Create the database file if it doesn't exist (sqlx requires the file to exist)
        if !db_path.exists() {
            tokio::fs::File::create(&db_path).await.map_err(|e| {
                OrchestratorError::cache_error(format!("Failed to create cache database file: {}", e))
            })?;
        }

        // Build SQLite connection URL with sqlite: prefix (required by sqlx)
        // For absolute paths: sqlite:/tmp/file.db (one slash after sqlite:)
        // For relative paths: sqlite:file.db (no slash after sqlite:)
        let db_url = format!(
            "sqlite:{}",
            db_path.to_str().ok_or_else(|| {
                OrchestratorError::cache_error("Invalid database path".to_string())
            })?
        );

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .min_connections(1)
            .acquire_timeout(StdDuration::from_secs(30))
            .connect(&db_url)
            .await
            .map_err(|e| OrchestratorError::database_error(e, format!(
                "Failed to connect to cache database '{}'",
                db_url
            )))?;

        let cacher = Self {
            pool: Arc::new(pool),
            ttl_days,
        };

        // Initialize database schema
        cacher.init_db().await?;

        Ok(cacher)
    }

    /// Initialize the database schema.
    async fn init_db(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS cache_entries (
                key TEXT PRIMARY KEY,
                source_type TEXT NOT NULL,
                result TEXT NOT NULL,
                created_at DATETIME NOT NULL,
                expires_at DATETIME NOT NULL,
                content_hash TEXT
            )
            "#,
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| OrchestratorError::database(e))?;

        // Create index on expires_at for efficient cleanup
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_expires_at ON cache_entries(expires_at)
            "#,
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| OrchestratorError::database(e))?;

        // Create index on source_type for filtering
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_source_type ON cache_entries(source_type)
            "#,
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| OrchestratorError::database(e))?;

        // Enable WAL mode for better concurrency
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&*self.pool)
            .await
            .map_err(|e| OrchestratorError::database_error(e, "Failed to set WAL mode"))?;

        // Set synchronous to NORMAL for better performance (safe with WAL)
        sqlx::query("PRAGMA synchronous = NORMAL")
            .execute(&*self.pool)
            .await
            .map_err(|e| OrchestratorError::database_error(e, "Failed to set synchronous"))?;

        // Set cache size (64MB)
        sqlx::query("PRAGMA cache_size = -64000")
            .execute(&*self.pool)
            .await
            .map_err(|e| OrchestratorError::database_error(e, "Failed to set cache_size"))?;

        info!("Cache database initialized with WAL mode");
        Ok(())
    }

    /// Get a cached entry by key.
    pub async fn get(&self, key: &str) -> Result<Option<CacheEntry>> {
        // Query with string timestamps
        let row: Option<(String, String, String, String, String, Option<String>)> = sqlx::query_as(
            r#"
            SELECT key, source_type, result, created_at, expires_at, content_hash
            FROM cache_entries
            WHERE key = ? AND expires_at > datetime('now')
            "#,
        )
        .bind(key)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| OrchestratorError::database(e))?;

        if let Some((key, source_type, result, created_at_str, expires_at_str, content_hash)) = row {
            // Parse DateTime from strings
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| OrchestratorError::cache_error(format!("Invalid created_at timestamp: {}", e)))?
                .with_timezone(&Utc);
            let expires_at = DateTime::parse_from_rfc3339(&expires_at_str)
                .map_err(|e| OrchestratorError::cache_error(format!("Invalid expires_at timestamp: {}", e)))?
                .with_timezone(&Utc);

            debug!("Cache hit for key: {}", key);
            Ok(Some(CacheEntry {
                key,
                source_type,
                result,
                created_at,
                expires_at,
                content_hash,
            }))
        } else {
            debug!("Cache miss for key: {}", key);
            Ok(None)
        }
    }

    /// Store a cache entry.
    pub async fn set(&self, entry: CacheEntry) -> Result<()> {
        // Convert DateTime to SQLite format string
        let created_at_str = entry.created_at.to_rfc3339();
        let expires_at_str = entry.expires_at.to_rfc3339();

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO cache_entries (key, source_type, result, created_at, expires_at, content_hash)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&entry.key)
        .bind(&entry.source_type)
        .bind(&entry.result)
        .bind(&created_at_str)
        .bind(&expires_at_str)
        .bind(&entry.content_hash)
        .execute(&*self.pool)
        .await
        .map_err(|e| OrchestratorError::database(e))?;

        debug!("Cached entry for key: {}", entry.key);
        Ok(())
    }

    /// Create a cache entry from scan result.
    pub fn create_entry(
        &self,
        key: String,
        source_type: String,
        result: String,
        content_hash: Option<String>,
    ) -> CacheEntry {
        let now = Utc::now();
        let expires_at = now + Duration::days(self.ttl_days);

        CacheEntry {
            key,
            source_type,
            result,
            created_at: now,
            expires_at,
            content_hash,
        }
    }

    /// Check if a key exists in cache and is not expired.
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM cache_entries
            WHERE key = ? AND expires_at > datetime('now')
            "#,
        )
        .bind(key)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| OrchestratorError::database(e))?;

        Ok(count.0 > 0)
    }

    /// Remove a cached entry.
    pub async fn remove(&self, key: &str) -> Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM cache_entries WHERE key = ?
            "#,
        )
        .bind(key)
        .execute(&*self.pool)
        .await
        .map_err(|e| OrchestratorError::database(e))?;

        Ok(result.rows_affected() > 0)
    }

    /// Clean up expired entries.
    pub async fn cleanup_expired(&self) -> Result<usize> {
        let result = sqlx::query(
            r#"
            DELETE FROM cache_entries WHERE expires_at <= datetime('now')
            "#,
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| OrchestratorError::database(e))?;

        let removed = result.rows_affected() as usize;
        if removed > 0 {
            info!("Cleaned up {} expired cache entries", removed);
        }
        Ok(removed)
    }

    /// Get cache statistics.
    pub async fn stats(&self) -> Result<CacheStats> {
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM cache_entries")
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| OrchestratorError::database(e))?;

        let expired: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM cache_entries WHERE expires_at <= datetime('now')",
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| OrchestratorError::database(e))?;

        let npm: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM cache_entries WHERE source_type = 'npm'",
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| OrchestratorError::database(e))?;

        let github: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM cache_entries WHERE source_type = 'github'",
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| OrchestratorError::database(e))?;

        let file: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM cache_entries WHERE source_type = 'file'",
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| OrchestratorError::database(e))?;

        Ok(CacheStats {
            total_entries: total.0 as usize,
            expired_entries: expired.0 as usize,
            npm_entries: npm.0 as usize,
            github_entries: github.0 as usize,
            file_entries: file.0 as usize,
        })
    }

    /// Clear all cache entries.
    pub async fn clear(&self) -> Result<()> {
        sqlx::query("DELETE FROM cache_entries")
            .execute(&*self.pool)
            .await
            .map_err(|e| OrchestratorError::database(e))?;

        info!("Cache cleared");
        Ok(())
    }

    /// Get the TTL in days.
    pub fn ttl_days(&self) -> i64 {
        self.ttl_days
    }
}

/// Cache statistics.
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Total number of entries.
    pub total_entries: usize,
    /// Number of expired entries.
    pub expired_entries: usize,
    /// Number of npm entries.
    pub npm_entries: usize,
    /// Number of GitHub entries.
    pub github_entries: usize,
    /// Number of file entries.
    pub file_entries: usize,
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Cache Statistics:")?;
        writeln!(f, "  Total entries: {}", self.total_entries)?;
        writeln!(f, "  Expired entries: {}", self.expired_entries)?;
        writeln!(f, "  npm entries: {}", self.npm_entries)?;
        writeln!(f, "  GitHub entries: {}", self.github_entries)?;
        writeln!(f, "  File entries: {}", self.file_entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_cacher() -> (Cacher, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_cache.db");
        let cacher = Cacher::with_path_and_ttl(&db_path, 7).await.unwrap();
        (cacher, temp_dir)
    }

    #[tokio::test]
    async fn test_cache_set_get() {
        let (cacher, _temp_dir) = create_test_cacher().await;

        let entry = cacher.create_entry(
            "test-key".to_string(),
            "npm".to_string(),
            r#"{"findings": []}"#.to_string(),
            Some("abc123".to_string()),
        );

        cacher.set(entry).await.unwrap();

        let retrieved = cacher.get("test-key").await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.key, "test-key");
        assert_eq!(retrieved.source_type, "npm");
        assert_eq!(retrieved.result, r#"{"findings": []}"#);
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let (cacher, _temp_dir) = create_test_cacher().await;

        let retrieved = cacher.get("nonexistent-key").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_cache_exists() {
        let (cacher, _temp_dir) = create_test_cacher().await;

        assert!(!cacher.exists("test-key").await.unwrap());

        let entry = cacher.create_entry(
            "test-key".to_string(),
            "npm".to_string(),
            r#"{"findings": []}"#.to_string(),
            None,
        );
        cacher.set(entry).await.unwrap();

        assert!(cacher.exists("test-key").await.unwrap());
    }

    #[tokio::test]
    async fn test_cache_remove() {
        let (cacher, _temp_dir) = create_test_cacher().await;

        let entry = cacher.create_entry(
            "test-key".to_string(),
            "npm".to_string(),
            r#"{"findings": []}"#.to_string(),
            None,
        );
        cacher.set(entry).await.unwrap();

        assert!(cacher.remove("test-key").await.unwrap());
        assert!(!cacher.exists("test-key").await.unwrap());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let (cacher, _temp_dir) = create_test_cacher().await;

        // Add some entries
        for i in 0..3 {
            let entry = cacher.create_entry(
                format!("npm-{}", i),
                "npm".to_string(),
                r#"{"findings": []}"#.to_string(),
                None,
            );
            cacher.set(entry).await.unwrap();
        }

        for i in 0..2 {
            let entry = cacher.create_entry(
                format!("github-{}", i),
                "github".to_string(),
                r#"{"findings": []}"#.to_string(),
                None,
            );
            cacher.set(entry).await.unwrap();
        }

        let stats = cacher.stats().await.unwrap();
        assert_eq!(stats.total_entries, 5);
        assert_eq!(stats.npm_entries, 3);
        assert_eq!(stats.github_entries, 2);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let (cacher, _temp_dir) = create_test_cacher().await;

        let entry = cacher.create_entry(
            "test-key".to_string(),
            "npm".to_string(),
            r#"{"findings": []}"#.to_string(),
            None,
        );
        cacher.set(entry).await.unwrap();

        cacher.clear().await.unwrap();

        let stats = cacher.stats().await.unwrap();
        assert_eq!(stats.total_entries, 0);
    }
}
