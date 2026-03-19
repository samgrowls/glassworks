//! Incremental Scanning Cache
//!
//! This module provides hash-based caching to only scan changed files,
//! achieving significant speedup on re-scans.

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::finding::Finding;

/// A single cache entry for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCacheEntry {
    /// SHA-256 hash of the file content
    pub hash: String,
    /// Cached findings from the last scan
    pub findings: Vec<Finding>,
    /// Timestamp when the entry was created (Unix epoch seconds)
    pub timestamp: u64,
    /// File size in bytes (for additional validation)
    pub file_size: u64,
}

/// Statistics about cache performance
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: usize,
    /// Number of cache misses
    pub misses: usize,
    /// Number of expired entries
    pub expired: usize,
    /// Number of entries loaded from disk
    pub loaded: usize,
    /// Number of entries saved to disk
    pub saved: usize,
}

impl CacheStats {
    /// Create new stats with the given counts
    pub fn new(hits: usize, misses: usize, expired: usize, loaded: usize) -> Self {
        Self {
            hits,
            misses,
            expired,
            loaded,
            saved: 0,
        }
    }
}

impl CacheStats {
    /// Calculate the hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }

    /// Get the total number of lookups
    pub fn total_lookups(&self) -> usize {
        self.hits + self.misses
    }
}

/// Cache file format for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheFile {
    /// Cache format version for future compatibility
    version: u32,
    /// Cache entries mapped by file path
    entries: HashMap<String, FileCacheEntry>,
    /// Timestamp when the cache was created/updated
    created_at: u64,
}

impl CacheFile {
    fn new() -> Self {
        Self {
            version: 1,
            entries: HashMap::new(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

/// Hash-based cache for incremental scanning
///
/// The cache stores findings keyed by file path and content hash.
/// When a file is scanned:
/// 1. Check if cache entry exists for the path
/// 2. If exists, verify hash matches current content
/// 3. If hash matches and not expired, return cached findings (cache hit)
/// 4. Otherwise, scan and update cache (cache miss)
pub struct ScanCache {
    /// Path to the cache file on disk
    cache_file: PathBuf,
    /// In-memory cache entries (using Mutex for thread-safe interior mutability)
    entries: Mutex<HashMap<String, FileCacheEntry>>,
    /// Time-to-live for cache entries in days
    ttl_days: u64,
    /// Cache statistics (using Mutex for thread-safe interior mutability)
    stats: Mutex<CacheStats>,
    /// Whether the cache is enabled
    enabled: bool,
}

impl ScanCache {
    /// Create a new cache with the specified cache file and TTL
    ///
    /// # Arguments
    /// * `cache_file` - Path to the cache file for persistence
    /// * `ttl_days` - Number of days before cache entries expire
    ///
    /// # Returns
    /// A new ScanCache instance with existing entries loaded from disk
    pub fn new(cache_file: PathBuf, ttl_days: u64) -> Self {
        let cache = Self {
            cache_file,
            entries: Mutex::new(HashMap::new()),
            ttl_days,
            stats: Mutex::new(CacheStats::default()),
            enabled: true,
        };

        // Load existing cache from disk
        cache.load_from_disk();

        cache
    }

    /// Create a disabled cache (no-op operations)
    pub fn disabled() -> Self {
        Self {
            cache_file: PathBuf::new(),
            entries: Mutex::new(HashMap::new()),
            ttl_days: 0,
            stats: Mutex::new(CacheStats::default()),
            enabled: false,
        }
    }

    /// Check if the cache is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.stats.lock().unwrap().clone()
    }

    /// Get mutable reference to cache statistics
    pub fn stats_mut(&mut self) -> &mut CacheStats {
        self.stats.get_mut().unwrap()
    }

    /// Try to get cached findings for a file
    ///
    /// # Arguments
    /// * `path` - File path to look up
    /// * `content` - Current file content for hash verification
    /// * `file_size` - Current file size in bytes
    ///
    /// # Returns
    /// `Some(Vec<Finding>)` if cache hit, `None` if cache miss
    pub fn get(&self, path: &str, content: &str, file_size: u64) -> Option<Vec<Finding>> {
        if !self.enabled {
            return None;
        }

        let entries = self.entries.lock().unwrap();
        let entry = match entries.get(path) {
            Some(entry) => entry,
            None => {
                drop(entries);
                let mut stats = self.stats.lock().unwrap();
                stats.misses += 1;
                return None;
            }
        };

        // Check if entry has expired
        if self.is_expired(entry.timestamp) {
            drop(entries);
            let mut stats = self.stats.lock().unwrap();
            stats.expired += 1;
            stats.misses += 1;
            return None;
        }

        // Verify content hash matches
        let current_hash = Self::hash_content(content);
        if entry.hash != current_hash {
            drop(entries);
            let mut stats = self.stats.lock().unwrap();
            stats.misses += 1;
            return None;
        }

        // Verify file size matches (additional validation)
        if entry.file_size != file_size {
            drop(entries);
            let mut stats = self.stats.lock().unwrap();
            stats.misses += 1;
            return None;
        }

        // Cache hit! Clone the findings and return
        let mut stats = self.stats.lock().unwrap();
        stats.hits += 1;
        Some(entry.findings.clone())
    }

    /// Store findings in the cache
    ///
    /// # Arguments
    /// * `path` - File path
    /// * `content` - File content (for hashing)
    /// * `findings` - Scan results to cache
    /// * `file_size` - File size in bytes
    pub fn set(&self, path: String, content: &str, findings: Vec<Finding>, file_size: u64) {
        if !self.enabled {
            return;
        }

        let hash = Self::hash_content(content);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let entry = FileCacheEntry {
            hash,
            findings,
            timestamp,
            file_size,
        };

        self.entries.lock().unwrap().insert(path, entry);
    }

    /// Remove a specific entry from the cache
    pub fn remove(&self, path: &str) {
        self.entries.lock().unwrap().remove(path);
    }

    /// Clear all cache entries
    pub fn clear(&self) {
        self.entries.lock().unwrap().clear();
    }

    /// Save cache to disk
    ///
    /// # Returns
    /// `Ok(())` on success, `Err(std::io::Error)` on failure
    pub fn save(&self) -> std::io::Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // Ensure parent directory exists
        if let Some(parent) = self.cache_file.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let cache_file = CacheFile {
            version: 1,
            entries: self.entries.lock().unwrap().clone(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let file = File::create(&self.cache_file)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &cache_file)?;

        Ok(())
    }

    /// Load cache from disk
    ///
    /// Silently ignores errors (cache will start fresh)
    fn load_from_disk(&self) {
        if !self.cache_file.exists() {
            return;
        }

        match File::open(&self.cache_file) {
            Ok(file) => {
                let reader = BufReader::new(file);
                match serde_json::from_reader::<_, CacheFile>(reader) {
                    Ok(cache_file) => {
                        // Filter out expired entries
                        let mut valid_entries = HashMap::new();
                        let mut stats = self.stats.lock().unwrap();
                        for (path, entry) in cache_file.entries {
                            if !self.is_expired(entry.timestamp) {
                                valid_entries.insert(path, entry);
                                stats.loaded += 1;
                            } else {
                                stats.expired += 1;
                            }
                        }
                        *self.entries.lock().unwrap() = valid_entries;
                    }
                    Err(e) => {
                        // Invalid cache file, start fresh
                        eprintln!(
                            "Warning: Failed to parse cache file: {}. Starting with fresh cache.",
                            e
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to open cache file: {}. Starting with fresh cache.", e);
            }
        }
    }

    /// Check if a timestamp has expired based on TTL
    fn is_expired(&self, timestamp: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let ttl_seconds = self.ttl_days * 24 * 60 * 60;
        now.saturating_sub(timestamp) > ttl_seconds
    }

    /// Calculate SHA-256 hash of content
    fn hash_content(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Get the number of cache entries
    pub fn len(&self) -> usize {
        self.entries.lock().unwrap().len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.lock().unwrap().is_empty()
    }

    /// Get the cache file path
    pub fn cache_file_path(&self) -> &Path {
        &self.cache_file
    }

    /// Remove stale entries (expired or invalid)
    ///
    /// This can be called periodically to clean up the cache
    pub fn cleanup(&self) -> usize {
        let mut entries = self.entries.lock().unwrap();
        let before = entries.len();
        entries.retain(|_, entry| !self.is_expired(entry.timestamp));
        before - entries.len()
    }
}

impl Default for ScanCache {
    fn default() -> Self {
        Self::new(PathBuf::from(".glassware-cache.json"), 7)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::{DetectionCategory, Severity};
    use tempfile::TempDir;

    fn create_test_finding() -> Finding {
        Finding::new(
            "test.js",
            1,
            5,
            0xFE00,
            '\u{FE00}',
            DetectionCategory::InvisibleCharacter,
            Severity::High,
            "Test finding",
            "Remove it",
        )
    }

    #[test]
    fn test_cache_hash_content() {
        let content = "hello world";
        let hash1 = ScanCache::hash_content(content);
        let hash2 = ScanCache::hash_content(content);
        let hash3 = ScanCache::hash_content("hello world!");

        assert_eq!(hash1, hash2); // Same content = same hash
        assert_ne!(hash1, hash3); // Different content = different hash
    }

    #[test]
    fn test_cache_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let cache_file = temp_dir.path().join("cache.json");

        let cache = ScanCache::new(cache_file, 7);

        let path = "test.js";
        let content = "const x = 1;";
        let findings = vec![create_test_finding()];
        let file_size = content.len() as u64;

        // Cache miss on first lookup
        assert!(cache.get(path, content, file_size).is_none());
        assert_eq!(cache.stats().misses, 1);

        // Store in cache
        cache.set(path.to_string(), content, findings.clone(), file_size);

        // Cache hit on second lookup
        let cached = cache.get(path, content, file_size).unwrap();
        assert_eq!(cached.len(), 1);
        assert_eq!(cache.stats().hits, 1);
    }

    #[test]
    fn test_cache_hash_invalidation() {
        let temp_dir = TempDir::new().unwrap();
        let cache_file = temp_dir.path().join("cache.json");

        let cache = ScanCache::new(cache_file, 7);

        let path = "test.js";
        let content1 = "const x = 1;";
        let content2 = "const x = 2;";
        let findings = vec![create_test_finding()];

        // Store with content1
        cache.set(path.to_string(), content1, findings.clone(), content1.len() as u64);

        // Lookup with content2 should miss (hash mismatch)
        assert!(cache.get(path, content2, content2.len() as u64).is_none());

        // Lookup with content1 should hit
        assert!(cache.get(path, content1, content1.len() as u64).is_some());
    }

    #[test]
    fn test_cache_file_size_validation() {
        let temp_dir = TempDir::new().unwrap();
        let cache_file = temp_dir.path().join("cache.json");

        let cache = ScanCache::new(cache_file, 7);

        let path = "test.js";
        let content = "const x = 1;";
        let findings = vec![create_test_finding()];

        // Store with correct file size
        cache.set(path.to_string(), content, findings.clone(), content.len() as u64);

        // Lookup with wrong file size should miss
        assert!(cache.get(path, content, 999).is_none());

        // Lookup with correct file size should hit
        assert!(cache.get(path, content, content.len() as u64).is_some());
    }

    #[test]
    fn test_cache_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let cache_file = temp_dir.path().join("cache.json");

        // Create cache and add entry
        {
            let cache = ScanCache::new(cache_file.clone(), 7);
            let path = "test.js";
            let content = "const x = 1;";
            let findings = vec![create_test_finding()];
            cache.set(path.to_string(), content, findings.clone(), content.len() as u64);
            cache.save().unwrap();
        }

        // Load cache from disk
        {
            let cache = ScanCache::new(cache_file, 7);
            assert_eq!(cache.len(), 1);
            assert_eq!(cache.stats().loaded, 1);
        }
    }

    #[test]
    fn test_cache_disabled() {
        let cache = ScanCache::disabled();

        assert!(!cache.is_enabled());

        let path = "test.js";
        let content = "const x = 1;";
        let findings = vec![create_test_finding()];

        // Operations should be no-ops
        cache.set(path.to_string(), content, findings.clone(), content.len() as u64);
        assert!(cache.get(path, content, content.len() as u64).is_none());
        assert_eq!(cache.stats().hits, 0);
        assert_eq!(cache.stats().misses, 0);
    }

    #[test]
    fn test_cache_clear_and_remove() {
        let temp_dir = TempDir::new().unwrap();
        let cache_file = temp_dir.path().join("cache.json");

        let cache = ScanCache::new(cache_file, 7);

        // Add multiple entries
        cache.set("test1.js".to_string(), "content1", vec![], 8);
        cache.set("test2.js".to_string(), "content2", vec![], 8);
        cache.set("test3.js".to_string(), "content3", vec![], 8);

        assert_eq!(cache.len(), 3);

        // Remove one entry
        cache.remove("test2.js");
        assert_eq!(cache.len(), 2);

        // Clear all
        cache.clear();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_stats() {
        let temp_dir = TempDir::new().unwrap();
        let cache_file = temp_dir.path().join("cache.json");

        let cache = ScanCache::new(cache_file, 7);

        let path = "test.js";
        let content = "const x = 1;";

        // Multiple lookups
        cache.get(path, content, content.len() as u64); // miss
        cache.get(path, content, content.len() as u64); // miss
        cache.set(path.to_string(), content, vec![], content.len() as u64);
        cache.get(path, content, content.len() as u64); // hit
        cache.get(path, content, content.len() as u64); // hit

        let stats = cache.stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 2);
        assert_eq!(stats.total_lookups(), 4);
        assert!((stats.hit_rate() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_cache_default() {
        let cache = ScanCache::default();
        assert!(!cache.cache_file_path().as_os_str().is_empty());
        assert!(cache.is_enabled());
    }
}
