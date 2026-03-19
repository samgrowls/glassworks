//! Scan Engine
//!
//! Orchestrates multiple detectors over files, merging and sorting findings.

use crate::cache::{CacheStats, ScanCache};
use crate::config::{ScanConfig, UnicodeConfig};
use crate::detector::{Detector, ScanContext};
#[cfg(feature = "semantic")]
use crate::detector::SemanticDetector;
use crate::encrypted_payload_detector::EncryptedPayloadDetector;
use crate::finding::Finding;
use crate::forcememo_detector::ForceMemoDetector;
use crate::header_c2_detector::HeaderC2Detector;
use crate::jpd_author_detector::JpdAuthorDetector;
use crate::rdd_detector::RddDetector;
use crate::unicode_detector::UnicodeDetector;
// NEW: Behavioral evasion detectors
use crate::locale_detector::LocaleGeofencingDetector;
use crate::time_delay_detector::TimeDelayDetector;
use crate::blockchain_c2_detector::BlockchainC2Detector;
#[cfg(feature = "llm")]
use std::collections::HashMap;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::fs;

/// Result of a scan operation, including findings and optional LLM verdicts.
pub struct ScanResult {
    pub findings: Vec<Finding>,
    #[cfg(feature = "llm")]
    pub llm_verdicts: Vec<crate::llm::LlmFileResult>,
    /// Statistics about deduplication (if enabled)
    pub dedup_stats: Option<DedupStats>,
    /// Statistics about cache performance (if caching is enabled)
    pub cache_stats: Option<CacheStats>,
}

/// Statistics about deduplication performed during scanning
#[derive(Debug, Clone, Default)]
pub struct DedupStats {
    /// Total findings before deduplication
    pub total_before: usize,
    /// Total findings after deduplication
    pub total_after: usize,
    /// Number of duplicates removed
    pub duplicates_removed: usize,
}

impl DedupStats {
    /// Create new stats with the given counts
    pub fn new(total_before: usize, total_after: usize) -> Self {
        Self {
            total_before,
            total_after,
            duplicates_removed: total_before.saturating_sub(total_after),
        }
    }
}

/// Orchestrates multiple detectors over files.
///
/// The ScanEngine registers detectors and runs them against file content,
/// collecting and sorting all findings.
pub struct ScanEngine {
    detectors: Vec<Box<dyn Detector>>,
    #[cfg(feature = "semantic")]
    semantic_detectors: Vec<Box<dyn SemanticDetector>>,
    config: UnicodeConfig,
    scan_config: ScanConfig,
    #[cfg(feature = "llm")]
    use_llm: bool,
    /// Enable deduplication of findings
    enable_dedup: bool,
    /// Optional cache for incremental scanning
    cache: Option<ScanCache>,
}

impl ScanEngine {
    /// Create a new engine with no detectors.
    pub fn new() -> Self {
        Self {
            detectors: Vec::new(),
            #[cfg(feature = "semantic")]
            semantic_detectors: Vec::new(),
            config: UnicodeConfig::default(),
            scan_config: ScanConfig::default(),
            #[cfg(feature = "llm")]
            use_llm: false,
            enable_dedup: true, // Enable deduplication by default
            cache: None,
        }
    }

    /// Create a new engine with a scan configuration.
    pub fn with_config(config: ScanConfig) -> Self {
        let enable_dedup = config.enable_dedup;
        #[cfg(feature = "llm")]
        let use_llm = config.enable_llm;

        Self {
            detectors: Vec::new(),
            #[cfg(feature = "semantic")]
            semantic_detectors: Vec::new(),
            config: UnicodeConfig::default(),
            scan_config: config,
            #[cfg(feature = "llm")]
            use_llm,
            enable_dedup,
            cache: None,
        }
    }

    /// Enable or disable LLM analysis
    #[cfg(feature = "llm")]
    pub fn with_llm(mut self, use_llm: bool) -> Self {
        self.use_llm = use_llm;
        self
    }

    /// Enable or disable findings deduplication
    pub fn with_deduplication(mut self, enable: bool) -> Self {
        self.enable_dedup = enable;
        self
    }

    /// Enable caching for incremental scanning
    ///
    /// # Arguments
    /// * `cache_file` - Path to the cache file for persistence
    /// * `ttl_days` - Number of days before cache entries expire
    ///
    /// # Returns
    /// Self with caching enabled
    pub fn with_cache(mut self, cache_file: PathBuf, ttl_days: u64) -> Self {
        self.cache = Some(ScanCache::new(cache_file, ttl_days));
        self
    }

    /// Disable caching
    pub fn without_cache(mut self) -> Self {
        self.cache = None;
        self
    }

    /// Get cache statistics if caching is enabled
    pub fn cache_stats(&self) -> Option<CacheStats> {
        self.cache.as_ref().map(|c| c.stats())
    }

    /// Save cache to disk
    ///
    /// This should be called after scanning is complete to persist the cache.
    /// Returns Ok(()) on success, or an io::Error on failure.
    pub fn save_cache(&self) -> std::io::Result<()> {
        if let Some(cache) = &self.cache {
            cache.save()
        } else {
            Ok(())
        }
    }

    /// Create an engine pre-loaded with all built-in detectors.
    pub fn default_detectors() -> Self {
        let mut engine = Self::new();
        engine.register(Box::new(UnicodeDetector::new()));
        engine.register(Box::new(EncryptedPayloadDetector::new()));
        engine.register(Box::new(HeaderC2Detector::new()));
        
        // NEW: RDD detector for PhantomRaven detection
        engine.register(Box::new(RddDetector::new()));
        
        // NEW: JPD author detector (PhantomRaven signature)
        engine.register(Box::new(JpdAuthorDetector::new()));
        
        // NEW: ForceMemo Python detector
        engine.register(Box::new(ForceMemoDetector::new()));
        
        // NEW: Behavioral evasion detectors (GW009-GW011)
        engine.register(Box::new(LocaleGeofencingDetector::new()));
        engine.register(Box::new(TimeDelayDetector::new()));
        engine.register(Box::new(BlockchainC2Detector::new()));

        #[cfg(feature = "semantic")]
        {
            engine.register_semantic(Box::new(crate::gw005_semantic::Gw005SemanticDetector::new()));
            engine.register_semantic(Box::new(crate::gw006_semantic::Gw006SemanticDetector::new()));
            engine.register_semantic(Box::new(crate::gw007_semantic::Gw007SemanticDetector::new()));
            engine.register_semantic(Box::new(crate::gw008_semantic::Gw008SemanticDetector::new()));
        }

        engine
    }

    /// Create an engine with a custom Unicode configuration.
    pub fn with_unicode_config(config: UnicodeConfig) -> Self {
        Self {
            detectors: Vec::new(),
            #[cfg(feature = "semantic")]
            semantic_detectors: Vec::new(),
            config,
            scan_config: ScanConfig::default(),
            #[cfg(feature = "llm")]
            use_llm: false,
            enable_dedup: true,
            cache: None,
        }
    }

    /// Create an engine pre-loaded with all built-in detectors and a custom scan configuration.
    pub fn default_detectors_with_config(config: ScanConfig) -> Self {
        let enable_dedup = config.enable_dedup;
        #[cfg(feature = "llm")]
        let use_llm = config.enable_llm;

        let mut engine = Self {
            detectors: Vec::new(),
            #[cfg(feature = "semantic")]
            semantic_detectors: Vec::new(),
            config: UnicodeConfig::default(),
            scan_config: config,
            #[cfg(feature = "llm")]
            use_llm,
            enable_dedup,
            cache: None,
        };

        engine.register(Box::new(UnicodeDetector::new()));
        engine.register(Box::new(EncryptedPayloadDetector::new()));
        engine.register(Box::new(HeaderC2Detector::new()));

        // NEW: RDD detector for PhantomRaven detection
        engine.register(Box::new(RddDetector::new()));

        // NEW: JPD author detector (PhantomRaven signature)
        engine.register(Box::new(JpdAuthorDetector::new()));

        // NEW: ForceMemo Python detector
        engine.register(Box::new(ForceMemoDetector::new()));

        // NEW: Behavioral evasion detectors (GW009-GW011)
        engine.register(Box::new(LocaleGeofencingDetector::new()));
        engine.register(Box::new(TimeDelayDetector::new()));
        engine.register(Box::new(BlockchainC2Detector::new()));

        #[cfg(feature = "semantic")]
        {
            engine.register_semantic(Box::new(crate::gw005_semantic::Gw005SemanticDetector::new()));
            engine.register_semantic(Box::new(crate::gw006_semantic::Gw006SemanticDetector::new()));
            engine.register_semantic(Box::new(crate::gw007_semantic::Gw007SemanticDetector::new()));
            engine.register_semantic(Box::new(crate::gw008_semantic::Gw008SemanticDetector::new()));
        }

        engine
    }

    /// Register a detector.
    pub fn register(&mut self, detector: Box<dyn Detector>) {
        self.detectors.push(detector);
    }

    /// Register a semantic detector.
    #[cfg(feature = "semantic")]
    pub fn register_semantic(&mut self, detector: Box<dyn SemanticDetector>) {
        self.semantic_detectors.push(detector);
    }

    /// Deduplicate findings by (file, line, column, category).
    ///
    /// When multiple detectors flag the same location with the same category,
    /// keeps only the finding with the highest severity.
    ///
    /// Returns the deduplicated findings and statistics.
    fn deduplicate_findings(&self, findings: Vec<Finding>) -> (Vec<Finding>, DedupStats) {
        if !self.enable_dedup {
            let len = findings.len();
            return (findings, DedupStats::new(len, len));
        }

        let total_before = findings.len();

        // Key = (file, line, column, category)
        // Use BTreeMap for deterministic ordering
        let mut deduped: BTreeMap<(String, usize, usize, String), Finding> = BTreeMap::new();

        for finding in findings {
            let key = (
                finding.file.clone(),
                finding.line,
                finding.column,
                finding.category.as_str().to_string(),
            );

            // Keep highest severity if duplicate
            deduped
                .entry(key)
                .and_modify(|existing| {
                    if finding.severity > existing.severity {
                        *existing = finding.clone();
                    }
                })
                .or_insert(finding.clone());
        }

        let mut result: Vec<Finding> = deduped.into_values().collect();

        // Sort by line, then column (maintain original sort order)
        result.sort_by(|a, b| a.line.cmp(&b.line).then(a.column.cmp(&b.column)));

        let total_after = result.len();
        let stats = DedupStats::new(total_before, total_after);

        (result, stats)
    }

    /// Scan file content with all registered detectors.
    /// Returns findings sorted by location.
    ///
    /// For LLM analysis, use `scan_with_llm()` instead.
    pub fn scan(&self, path: &Path, content: &str) -> Vec<Finding> {
        self.scan_internal(path, content).findings
    }

    /// Scan file content with all registered detectors and optional LLM analysis.
    /// Returns ScanResult with findings and deduplication statistics.
    #[cfg(feature = "llm")]
    pub fn scan_with_llm(&self, path: &Path, content: &str) -> ScanResult {
        self.scan_internal(path, content)
    }

    /// Scan file content and return findings with deduplication statistics.
    /// This is the full version that includes dedup stats.
    pub fn scan_with_stats(&self, path: &Path, content: &str) -> ScanResult {
        self.scan_internal(path, content)
    }

    /// Internal scan method that returns ScanResult
    fn scan_internal(&self, path: &Path, content: &str) -> ScanResult {
        // Get file size for cache validation
        let file_size = fs::metadata(path)
            .map(|m| m.len())
            .unwrap_or(content.len() as u64);

        // Check cache first (if enabled)
        if let Some(cache) = &self.cache {
            let path_str = path.to_string_lossy().to_string();
            if let Some(cached_findings) = cache.get(&path_str, content, file_size) {
                // Cache hit! Return cached findings
                return ScanResult {
                    findings: cached_findings.clone(),
                    #[cfg(feature = "llm")]
                    llm_verdicts: Vec::new(),
                    dedup_stats: None,
                    cache_stats: Some(cache.stats().clone()),
                };
            }
        }

        // Cache miss - run detectors
        let mut findings: Vec<Finding> = Vec::new();

        // Create scan context for unified trait interface
        let ctx = ScanContext::from_path(
            path,
            content.to_string(),
            self.config.clone(),
        );

        // Run regex-based detectors on all files
        for detector in &self.detectors {
            findings.extend(detector.detect(&ctx));
        }

        // Run semantic detectors on JS/TS files only
        #[cfg(feature = "semantic")]
        if !self.semantic_detectors.is_empty() {
            if let Some(analysis) = crate::semantic::build_semantic(content, path) {
                let sources = crate::taint::find_sources(&analysis);
                let sinks = crate::taint::find_sinks(&analysis);
                let flows = crate::taint::check_flows(&analysis, &sources, &sinks);

                for detector in &self.semantic_detectors {
                    findings
                        .extend(detector.detect_semantic(content, path, &flows, &sources, &sinks));
                }
            }
        }

        // Deduplicate findings
        let (findings, dedup_stats) = self.deduplicate_findings(findings);

        // Sort by line, then column
        // (already sorted in deduplicate_findings, but ensure consistency)
        let mut sorted_findings = findings;
        sorted_findings.sort_by(|a, b| a.line.cmp(&b.line).then(a.column.cmp(&b.column)));

        // Run LLM analysis if enabled and there are findings
        #[cfg(feature = "llm")]
        let llm_verdicts = if self.use_llm && !sorted_findings.is_empty() {
            self.run_llm_analysis(&sorted_findings, path, content)
        } else {
            Vec::new()
        };

        // Store in cache (if enabled)
        if let Some(cache) = &self.cache {
            let path_str = path.to_string_lossy().to_string();
            // Clone findings for caching (we need to return them too)
            let findings_clone = sorted_findings.clone();
            cache.set(path_str, content, findings_clone, file_size);
        }

        ScanResult {
            findings: sorted_findings,
            #[cfg(feature = "llm")]
            llm_verdicts,
            dedup_stats: Some(dedup_stats),
            cache_stats: self.cache.as_ref().map(|c| c.stats().clone()),
        }
    }

    /// Run LLM analysis on flagged files
    #[cfg(feature = "llm")]
    fn run_llm_analysis(
        &self,
        findings: &[Finding],
        _path: &Path,
        content: &str,
    ) -> Vec<crate::llm::LlmFileResult> {
        use crate::llm::{LlmConfig, OpenAiCompatibleAnalyzer};

        // Try to load config - if it fails, return empty vec (don't fail the scan)
        let config = match LlmConfig::from_env() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: LLM analysis skipped - {}", e);
                return Vec::new();
            }
        };

        let analyzer = OpenAiCompatibleAnalyzer::new(config);

        // Group findings by file
        let mut findings_by_file: HashMap<std::path::PathBuf, Vec<&Finding>> = HashMap::new();
        for finding in findings {
            findings_by_file
                .entry(std::path::PathBuf::from(&finding.file))
                .or_default()
                .push(finding);
        }

        let mut results = Vec::new();

        // Analyze each file
        for (file_path, file_findings) in findings_by_file {
            // Convert Vec<&Finding> to Vec<Finding> for analyze_file
            let findings_vec: Vec<Finding> = file_findings.iter().map(|f| (*f).clone()).collect();
            match analyzer.analyze_file(content, &file_path, &findings_vec) {
                Ok(verdict) => {
                    results.push(crate::llm::LlmFileResult { file_path, verdict });
                }
                Err(e) => {
                    eprintln!("Warning: LLM analysis failed for {:?}: {}", file_path, e);
                }
            }
        }

        results
    }

    /// Get the number of registered detectors.
    pub fn detector_count(&self) -> usize {
        self.detectors.len()
    }
}

impl Default for ScanEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::DetectionCategory;
    use crate::scanner::UnicodeScanner;

    #[test]
    fn test_engine_default_detectors() {
        let engine = ScanEngine::default_detectors();
        // 3 original detectors + 3 new behavioral evasion detectors (GW009-GW011)
        // + 3 semantic detectors (GW005-GW007) when semantic feature is enabled
        #[cfg(feature = "semantic")]
        assert_eq!(engine.detector_count(), 9);
        #[cfg(not(feature = "semantic"))]
        assert_eq!(engine.detector_count(), 6);
    }

    #[test]
    fn test_engine_scan_variation_selector() {
        let engine = ScanEngine::default_detectors();
        let content = "const secret\u{FE00}Key = 'value';";
        let findings = engine.scan(Path::new("test.js"), content);

        assert!(!findings.is_empty());
        assert!(findings
            .iter()
            .any(|f| f.category == DetectionCategory::InvisibleCharacter));
    }

    #[test]
    fn test_engine_matches_unicode_scanner() {
        // Verify that ScanEngine produces the same results as UnicodeScanner
        let engine = ScanEngine::default_detectors();
        let scanner = UnicodeScanner::with_default_config();

        let content = "const secret\u{FE00}Key = 'value';";
        let engine_findings = engine.scan(Path::new("test.js"), content);
        let scanner_findings = scanner.scan(content, "test.js");

        // Both should find the same number of findings
        assert_eq!(engine_findings.len(), scanner_findings.len());

        // Both should find an InvisibleCharacter
        assert!(engine_findings
            .iter()
            .any(|f| f.category == DetectionCategory::InvisibleCharacter));
        assert!(scanner_findings
            .iter()
            .any(|f| f.category == DetectionCategory::InvisibleCharacter));
    }

    #[test]
    fn test_deduplication_same_location_same_category() {
        // Test that duplicates at same location with same category are deduplicated
        let engine = ScanEngine::default_detectors();
        // Content with multiple invisible characters at different locations
        let content = "const\u{FE00} secret\u{FE00}Key = 'value';";
        let result = engine.scan_with_stats(Path::new("test.js"), content);

        // Should have dedup stats
        assert!(result.dedup_stats.is_some());
        let stats = result.dedup_stats.unwrap();

        // Verify deduplication occurred
        println!("Before: {}, After: {}, Removed: {}", 
                 stats.total_before, stats.total_after, stats.duplicates_removed);
        
        // All findings should be unique by (file, line, column, category)
        let findings = &result.findings;
        for i in 0..findings.len() {
            for j in (i + 1)..findings.len() {
                let same_location = findings[i].file == findings[j].file
                    && findings[i].line == findings[j].line
                    && findings[i].column == findings[j].column;
                let same_category = findings[i].category == findings[j].category;
                
                // Should not have duplicates with same location AND category
                if same_location && same_category {
                    panic!("Found duplicate findings at same location with same category");
                }
            }
        }
    }

    #[test]
    fn test_deduplication_preserves_highest_severity() {
        // Create mock findings with same location/category but different severities
        use crate::finding::Severity;
        
        let findings = vec![
            Finding::new(
                "test.js", 1, 5, 0xFE00, '\u{FE00}',
                DetectionCategory::InvisibleCharacter,
                Severity::Low,
                "Low severity finding",
                "Remove it"
            ),
            Finding::new(
                "test.js", 1, 5, 0xFE00, '\u{FE00}',
                DetectionCategory::InvisibleCharacter,
                Severity::Critical,
                "Critical severity finding",
                "Remove it immediately"
            ),
            Finding::new(
                "test.js", 1, 5, 0xFE00, '\u{FE00}',
                DetectionCategory::InvisibleCharacter,
                Severity::Medium,
                "Medium severity finding",
                "Remove it"
            ),
        ];

        let engine = ScanEngine::default_detectors();
        let (deduped, stats) = engine.deduplicate_findings(findings);

        // Should have deduplicated to 1 finding
        assert_eq!(stats.total_before, 3);
        assert_eq!(stats.total_after, 1);
        assert_eq!(stats.duplicates_removed, 2);

        // Should keep the highest severity (Critical)
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].severity, Severity::Critical);
        assert!(deduped[0].description.contains("Critical"));
    }

    #[test]
    fn test_deduplication_different_categories_preserved() {
        // Test that findings at same location but different categories are preserved
        use crate::finding::Severity;
        
        let findings = vec![
            Finding::new(
                "test.js", 1, 5, 0xFE00, '\u{FE00}',
                DetectionCategory::InvisibleCharacter,
                Severity::High,
                "Invisible character found",
                "Remove it"
            ),
            Finding::new(
                "test.js", 1, 5, 0x202C, '\u{202C}',
                DetectionCategory::BidirectionalOverride,
                Severity::Critical,
                "Bidi override found",
                "Remove it"
            ),
        ];

        let engine = ScanEngine::default_detectors();
        let (deduped, stats) = engine.deduplicate_findings(findings);

        // Both findings should be preserved (different categories)
        assert_eq!(stats.total_before, 2);
        assert_eq!(stats.total_after, 2);
        assert_eq!(stats.duplicates_removed, 0);
        assert_eq!(deduped.len(), 2);
    }

    #[test]
    fn test_deduplication_disabled() {
        // Test that deduplication can be disabled
        let mut engine = ScanEngine::default_detectors();
        engine.enable_dedup = false;
        
        let content = "const\u{FE00} secret\u{FE00}Key = 'value';";
        let result = engine.scan_with_stats(Path::new("test.js"), content);

        // Stats should show no deduplication
        let stats = result.dedup_stats.unwrap();
        assert_eq!(stats.total_before, stats.total_after);
        assert_eq!(stats.duplicates_removed, 0);
    }

    #[test]
    fn test_deduplication_with_builder_pattern() {
        // Test enabling/disabling dedup via builder pattern
        let engine_with_dedup = ScanEngine::default_detectors()
            .with_deduplication(true);
        assert!(engine_with_dedup.enable_dedup);

        let engine_without_dedup = ScanEngine::default_detectors()
            .with_deduplication(false);
        assert!(!engine_without_dedup.enable_dedup);
    }

    #[test]
    fn test_default_detectors_with_config() {
        use crate::config::ScanConfig;
        use crate::Severity;

        let config = ScanConfig::default()
            .with_min_severity(Severity::High)
            .with_deduplication(false);

        let engine = ScanEngine::default_detectors_with_config(config.clone());

        // Verify engine has detectors registered
        assert!(engine.detector_count() > 0);

        // Verify dedup is set from config
        assert!(!engine.enable_dedup);

        // Verify engine can scan
        let content = "const secret\u{FE00}Key = 'value';";
        let findings = engine.scan(Path::new("test.js"), content);

        // Should find invisible character
        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.severity >= Severity::High));
    }
}
