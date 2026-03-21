//! Scan Engine
//!
//! Orchestrates multiple detectors over files, merging and sorting findings.

use crate::attack_graph::AttackGraphEngine;
use crate::cache::{CacheStats, ScanCache};
use crate::config::{ScanConfig, UnicodeConfig};
use crate::detector::{Detector, ScanContext};
#[cfg(feature = "semantic")]
use crate::detector::SemanticDetector;
use crate::encrypted_payload_detector::EncryptedPayloadDetector;
use crate::finding::Finding;
use crate::forcememo_detector::ForceMemoDetector;
use crate::header_c2_detector::HeaderC2Detector;
use crate::ir::FileIR;
use crate::jpd_author_detector::JpdAuthorDetector;
use crate::rdd_detector::RddDetector;
use crate::unicode_detector::UnicodeDetector;
// NEW: Behavioral evasion detectors
use crate::locale_detector::LocaleGeofencingDetector;
use crate::time_delay_detector::TimeDelayDetector;
use crate::blockchain_c2_detector::BlockchainC2Detector;
// NEW: Campaign intelligence
use crate::campaign::{CampaignIntelligence, AnalyzedPackage};
// NEW: Cross-file taint tracking
#[cfg(feature = "semantic")]
use crate::module_graph::ModuleGraph;
#[cfg(feature = "semantic")]
use crate::cross_file_taint::CrossFileTaintTracker;
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
    /// Correlated attack chains (if attack graph is enabled)
    pub attack_chains: Vec<crate::attack_graph::AttackChain>,
    /// Overall threat score based on attack chains (0.0-10.0)
    pub threat_score: f32,
    /// Campaign intelligence (if campaign tracking is enabled)
    pub campaign_info: Option<CampaignInfo>,
    /// Cross-file taint flows (if cross-file analysis is enabled)
    #[cfg(feature = "semantic")]
    pub cross_file_flows: Vec<crate::taint::CrossFileTaintFlow>,
}

/// Campaign information in scan results
#[derive(Debug, Clone)]
pub struct CampaignInfo {
    /// Campaign ID if package belongs to a known campaign
    pub campaign_id: Option<String>,
    /// Related packages in the same campaign
    pub related_packages: Vec<String>,
    /// Shared infrastructure indicators
    pub shared_infrastructure: Vec<String>,
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

/// Detector DAG (Directed Acyclic Graph) for optimized execution ordering.
///
/// This struct manages detector execution order based on:
/// 1. Prerequisites (detectors that must run first)
/// 2. Cost (cheaper detectors run first)
/// 3. Signal strength (higher signal detectors run first)
///
/// The DAG enables short-circuit evaluation where expensive Tier 3 detectors
/// only run if Tier 1-2 detectors find something.
pub struct DetectorDAG {
    /// All detector nodes
    nodes: Vec<Box<dyn Detector>>,
    /// Adjacency list: detector name -> list of detector names that depend on it
    edges: std::collections::HashMap<String, Vec<String>>,
    /// Execution order (topologically sorted indices)
    execution_order: Vec<usize>,
}

impl DetectorDAG {
    /// Create a new DAG from a list of detectors.
    ///
    /// The execution order is determined by:
    /// 1. Prerequisites first (topological sort)
    /// 2. Within same dependency level: sort by cost (ascending)
    /// 3. Within same cost: sort by signal strength (descending)
    pub fn new(detectors: Vec<Box<dyn Detector>>) -> Self {
        let mut edges: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        let mut in_degree: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        
        // Build detector name -> index mapping
        let name_to_idx: std::collections::HashMap<&str, usize> = detectors
            .iter()
            .enumerate()
            .map(|(i, d)| (d.name(), i))
            .collect();
        
        // Initialize in-degree for all detectors
        for detector in &detectors {
            in_degree.insert(detector.name().to_string(), 0);
        }
        
        // Build edges from prerequisites
        for detector in &detectors {
            let name = detector.name().to_string();
            for prereq in detector.prerequisites() {
                if name_to_idx.contains_key(prereq) {
                    // prereq -> name (name depends on prereq)
                    edges.entry(prereq.to_string()).or_default().push(name.clone());
                    *in_degree.get_mut(&name).unwrap() += 1;
                }
            }
        }
        
        // Kahn's algorithm for topological sort with priority
        let mut execution_order = Vec::new();
        let mut available: Vec<usize> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(name, _)| *name_to_idx.get(name.as_str()).unwrap())
            .collect();
        
        // Sort available by (cost ASC, signal DESC)
        available.sort_by(|&a, &b| {
            let det_a = &detectors[a];
            let det_b = &detectors[b];
            det_a.cost()
                .cmp(&det_b.cost())
                .then_with(|| det_b.signal_strength().cmp(&det_a.signal_strength()))
        });
        
        while !available.is_empty() {
            // Take the highest priority detector
            let current_idx = available.remove(0);
            execution_order.push(current_idx);
            
            let current_name = detectors[current_idx].name().to_string();
            
            // Update in-degrees and add newly available detectors
            if let Some(dependents) = edges.get(&current_name) {
                for dependent_name in dependents {
                    let deg = in_degree.get_mut(dependent_name).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        let dep_idx = *name_to_idx.get(dependent_name.as_str()).unwrap();
                        available.push(dep_idx);
                        // Re-sort to maintain priority order
                        available.sort_by(|&a, &b| {
                            let det_a = &detectors[a];
                            let det_b = &detectors[b];
                            det_a.cost()
                                .cmp(&det_b.cost())
                                .then_with(|| det_b.signal_strength().cmp(&det_a.signal_strength()))
                        });
                    }
                }
            }
        }
        
        // Check for cycles (should not happen with proper prerequisites)
        if execution_order.len() != detectors.len() {
            // Cycle detected - add remaining detectors in arbitrary order
            let executed: std::collections::HashSet<usize> = execution_order.iter().copied().collect();
            for (i, _) in detectors.iter().enumerate() {
                if !executed.contains(&i) {
                    execution_order.push(i);
                }
            }
        }
        
        Self {
            nodes: detectors,
            edges,
            execution_order,
        }
    }
    
    /// Execute all detectors in the optimized order.
    ///
    /// Returns all findings collected from all detectors.
    /// May short-circuit if a detector returns true from should_short_circuit().
    ///
    /// # Arguments
    /// * `ir` - The unified intermediate representation of the file
    ///
    /// # Returns
    /// All findings detected in the file
    pub fn execute(&self, ir: &FileIR) -> Vec<Finding> {
        let mut all_findings = Vec::new();

        for &idx in &self.execution_order {
            let detector = &self.nodes[idx];

            // Check if detector should run based on current findings
            if !detector.should_run(&all_findings) {
                continue;
            }

            let findings = detector.detect(ir);
            all_findings.extend(findings);

            // Check for short-circuit
            if detector.should_short_circuit(&all_findings) {
                break;
            }
        }

        all_findings
    }
    
    /// Get the execution order for inspection/testing
    pub fn execution_order(&self) -> Vec<&str> {
        self.execution_order
            .iter()
            .map(|&i| self.nodes[i].name())
            .collect()
    }
    
    /// Get detector by name
    pub fn get_detector(&self, name: &str) -> Option<&dyn Detector> {
        self.nodes.iter()
            .find(|d| d.name() == name)
            .map(|d| d.as_ref())
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
    /// Attack graph engine for correlating findings into chains
    attack_graph: Option<AttackGraphEngine>,
    /// Campaign intelligence tracker
    campaign_intelligence: Option<CampaignIntelligence>,
    /// Module graph for cross-file analysis (JS/TS only)
    #[cfg(feature = "semantic")]
    module_graph: Option<ModuleGraph>,
    /// Cross-file taint tracker
    #[cfg(feature = "semantic")]
    cross_file_taint: Option<CrossFileTaintTracker>,
    /// Enable cross-file taint analysis
    #[cfg(feature = "semantic")]
    enable_cross_file_analysis: bool,
    /// DAG-based detector execution (if enabled)
    dag: Option<DetectorDAG>,
    /// Enable DAG-based execution (default: false for backward compatibility)
    enable_dag_execution: bool,
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
            attack_graph: None,
            campaign_intelligence: None,
            #[cfg(feature = "semantic")]
            module_graph: None,
            #[cfg(feature = "semantic")]
            cross_file_taint: None,
            #[cfg(feature = "semantic")]
            enable_cross_file_analysis: false,
            dag: None,
            enable_dag_execution: false,
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
            attack_graph: None,
            campaign_intelligence: None,
            #[cfg(feature = "semantic")]
            module_graph: None,
            #[cfg(feature = "semantic")]
            cross_file_taint: None,
            #[cfg(feature = "semantic")]
            enable_cross_file_analysis: false,
            dag: None,
            enable_dag_execution: false,
        }
    }

    /// Enable or disable LLM analysis
    #[cfg(feature = "llm")]
    pub fn with_llm(mut self, use_llm: bool) -> Self {
        self.use_llm = use_llm;
        self
    }

    /// Enable or disable DAG-based detector execution.
    ///
    /// When enabled, detectors are executed in an optimized order based on:
    /// 1. Prerequisites (detectors that must run first)
    /// 2. Cost (cheaper detectors run first)
    /// 3. Signal strength (higher signal detectors run first)
    ///
    /// This enables short-circuit evaluation where expensive Tier 3 detectors
    /// only run if Tier 1-2 detectors find something.
    ///
    /// # Arguments
    /// * `enabled` - Whether to enable DAG execution
    ///
    /// # Returns
    /// Self with DAG execution enabled/disabled
    pub fn with_dag_execution(mut self, enabled: bool) -> Self {
        self.enable_dag_execution = enabled;
        if enabled {
            // Build the DAG from current detectors
            let detectors: Vec<Box<dyn Detector>> = self.detectors
                .drain(..)
                .map(|d| -> Box<dyn Detector> { d })
                .collect();
            self.dag = Some(DetectorDAG::new(detectors));
        }
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

    /// Enable or disable attack graph correlation
    ///
    /// When enabled, the engine correlates individual findings into unified attack chains,
    /// providing a higher-level view of multi-stage attacks.
    ///
    /// # Arguments
    /// * `enabled` - Whether to enable attack graph correlation
    ///
    /// # Returns
    /// Self with attack graph enabled/disabled
    pub fn with_attack_graph(mut self, enabled: bool) -> Self {
        if enabled {
            self.attack_graph = Some(AttackGraphEngine::new());
        }
        self
    }

    /// Enable or disable campaign intelligence tracking
    ///
    /// When enabled, the engine tracks infrastructure reuse across packages,
    /// clusters packages by code similarity, and detects coordinated attack campaigns.
    ///
    /// # Arguments
    /// * `enabled` - Whether to enable campaign intelligence
    ///
    /// # Returns
    /// Self with campaign intelligence enabled/disabled
    pub fn with_campaign_intelligence(mut self, enabled: bool) -> Self {
        if enabled {
            self.campaign_intelligence = Some(CampaignIntelligence::new());
        }
        self
    }

    /// Enable or disable cross-file taint analysis
    ///
    /// When enabled, the engine builds a module graph of all JS/TS files in the package
    /// and tracks taint flows across file boundaries. This enables detection of split
    /// payloads where the decoder is in one file and the payload execution is in another.
    ///
    /// Note: This feature requires the `semantic` feature to be enabled and only works
    /// with JavaScript and TypeScript files.
    ///
    /// # Arguments
    /// * `enabled` - Whether to enable cross-file analysis
    ///
    /// # Returns
    /// Self with cross-file analysis enabled/disabled
    #[cfg(feature = "semantic")]
    pub fn with_cross_file_analysis(mut self, enabled: bool) -> Self {
        self.enable_cross_file_analysis = enabled;
        if enabled {
            self.module_graph = Some(ModuleGraph::new());
        }
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

        // E3: Browser-kill command detector (GlassWorm signature)
        engine.register(Box::new(crate::detectors::browser_kill::BrowserKillDetector::new()));

        // G3: Typo attribution detector (GlassWorm fingerprints)
        engine.register(Box::new(crate::detectors::typo_attribution::TypoAttributionDetector::new()));

        // G4: Exfil schema detector (GlassWorm JSON schema)
        engine.register(Box::new(crate::detectors::exfil_schema::ExfilSchemaDetector::new()));

        // G5: Socket.IO C2 detector (GlassWorm transport pattern)
        engine.register(Box::new(crate::detectors::socketio_c2::SocketIOC2Detector::new()));

        // Phase 2: Binary scanning detectors (.node files)
        #[cfg(feature = "binary")]
        {
            // G6: XorShift128 obfuscation
            engine.register(Box::new(crate::binary::XorShiftDetector::new()));
            // G7: IElevator COM CLSID
            engine.register(Box::new(crate::binary::IElevatorDetector::new()));
            // G8: APC injection
            engine.register(Box::new(crate::binary::ApcInjectionDetector::new()));
            // G9: memexec loader
            engine.register(Box::new(crate::binary::MemexecDetector::new()));
            // G11: .node metadata
            engine.register(Box::new(crate::binary::MetadataDetector::new()));
        }

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
            attack_graph: None,
            campaign_intelligence: None,
            #[cfg(feature = "semantic")]
            module_graph: None,
            #[cfg(feature = "semantic")]
            cross_file_taint: None,
            #[cfg(feature = "semantic")]
            enable_cross_file_analysis: false,
            dag: None,
            enable_dag_execution: false,
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
            attack_graph: None,
            campaign_intelligence: None,
            #[cfg(feature = "semantic")]
            module_graph: None,
            #[cfg(feature = "semantic")]
            cross_file_taint: None,
            #[cfg(feature = "semantic")]
            enable_cross_file_analysis: false,
            dag: None,
            enable_dag_execution: false,
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

        // E3: Browser-kill command detector (GlassWorm signature)
        engine.register(Box::new(crate::detectors::browser_kill::BrowserKillDetector::new()));

        // G3: Typo attribution detector (GlassWorm fingerprints)
        engine.register(Box::new(crate::detectors::typo_attribution::TypoAttributionDetector::new()));

        // G4: Exfil schema detector (GlassWorm JSON schema)
        engine.register(Box::new(crate::detectors::exfil_schema::ExfilSchemaDetector::new()));

        // G5: Socket.IO C2 detector (GlassWorm transport pattern)
        engine.register(Box::new(crate::detectors::socketio_c2::SocketIOC2Detector::new()));

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
                    attack_chains: Vec::new(),  // Attack chains not cached
                    threat_score: 0.0,  // Threat score not cached
                    campaign_info: None,  // Campaign info not cached
                    #[cfg(feature = "semantic")]
                    cross_file_flows: Vec::new(), // Cross-file flows not cached
                };
            }
        }

        // Cache miss - build unified IR once
        let ir = FileIR::build(path, content);

        // Run detectors using DAG execution if enabled
        let mut findings: Vec<Finding> = Vec::new();

        if self.enable_dag_execution {
            if let Some(ref dag) = self.dag {
                findings = dag.execute(&ir);
            } else {
                // Fallback to sequential if DAG not built
                for detector in &self.detectors {
                    findings.extend(detector.detect(&ir));
                }
            }
        } else {
            // Run detectors sequentially (all consume the same IR)
            for detector in &self.detectors {
                findings.extend(detector.detect(&ir));
            }
        }

        // Run semantic detectors on JS/TS files only
        #[cfg(feature = "semantic")]
        if !self.semantic_detectors.is_empty() {
            // Use pre-parsed AST from IR if available
            if let Some(ast) = ir.ast() {
                if ast.is_valid() {
                    // Build semantic analysis from AST
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
            // Clone findings for caching (we need to move sorted_findings into ScanResult)
            cache.set(path_str, content, sorted_findings.clone(), file_size);
        }

        // Run attack graph correlation if enabled
        let (attack_chains, threat_score) = if let Some(graph_engine) = &self.attack_graph {
            let mut engine = graph_engine.clone();
            engine.add_findings(sorted_findings.clone());
            let chains = engine.get_chains().to_vec();
            let score = engine.get_threat_score();
            (chains, score)
        } else {
            (Vec::new(), 0.0)
        };

        // Get campaign info if enabled
        let campaign_info = self.get_campaign_info(path, &sorted_findings);

        ScanResult {
            findings: sorted_findings,
            #[cfg(feature = "llm")]
            llm_verdicts,
            dedup_stats: Some(dedup_stats),
            cache_stats: self.cache.as_ref().map(|c| c.stats().clone()),
            attack_chains,
            threat_score,
            campaign_info,
            #[cfg(feature = "semantic")]
            cross_file_flows: Vec::new(), // Cross-file flows computed at package level
        }
    }

    /// Scan all files in a package with cross-file taint analysis
    ///
    /// This method builds a module graph of all JS/TS files in the package,
    /// runs detectors on each file, and then analyzes cross-file taint flows.
    ///
    /// # Arguments
    /// * `package_path` - Root directory of the package to scan
    ///
    /// # Returns
    /// ScanResult with findings and cross-file flows
    #[cfg(feature = "semantic")]
    pub fn scan_package(&mut self, package_path: &Path) -> std::io::Result<ScanResult> {
        use crate::cross_file_taint::{CrossFileTaintSource, CrossFileTaintSink};
        use std::collections::HashMap;
        
        if !self.enable_cross_file_analysis {
            // Fall back to regular scanning if cross-file analysis is disabled
            return Ok(self.scan_package_simple(package_path)?);
        }

        // Collect all JS/TS files in the package
        let files = self.collect_js_files(package_path)?;
        
        // Build module graph
        if let Some(ref mut graph) = self.module_graph {
            for file in &files {
                if let Ok(content) = fs::read_to_string(file) {
                    let path_str = file.to_string_lossy().to_string();
                    graph.add_file(&path_str, &content);
                }
            }
        }
        
        // Initialize cross-file taint tracker
        if let Some(graph) = self.module_graph.clone() {
            self.cross_file_taint = Some(CrossFileTaintTracker::new(graph));
        }
        
        // Scan each file and collect taint sources/sinks
        let mut all_findings: Vec<Finding> = Vec::new();
        let mut file_contents: HashMap<String, String> = HashMap::new();
        
        for file in &files {
            let content = fs::read_to_string(file)?;
            let path_str = file.to_string_lossy().to_string();
            file_contents.insert(path_str.clone(), content.clone());
            
            // Run regular detectors
            let ir = FileIR::build(file, &content);

            for detector in &self.detectors {
                all_findings.extend(detector.detect(&ir));
            }

            // Run semantic detectors and extract taint sources/sinks
            if !self.semantic_detectors.is_empty() {
                if let Some(analysis) = crate::semantic::build_semantic(&content, file) {
                    let sources = crate::taint::find_sources(&analysis);
                    let sinks = crate::taint::find_sinks(&analysis);
                    let flows = crate::taint::check_flows(&analysis, &sources, &sinks);

                    for detector in &self.semantic_detectors {
                        all_findings.extend(
                            detector.detect_semantic(&content, file, &flows, &sources, &sinks)
                        );
                    }

                    // Add sources/sinks to cross-file tracker
                    // First, extract all the data we need (to avoid borrow conflicts)
                    let mut source_data: Vec<CrossFileTaintSource> = Vec::new();
                    for source in &sources {
                        let symbol = Self::extract_symbol_from_source_static(&analysis, source)
                            .unwrap_or_else(|| "unknown".to_string());

                        let cross_source: CrossFileTaintSource = (
                            source,
                            path_str.as_str(),
                            byte_offset_to_line(&content, source.span().0),
                            symbol.as_str(),
                        ).into();
                        source_data.push(cross_source);
                    }

                    let mut sink_data: Vec<CrossFileTaintSink> = Vec::new();
                    for sink in &sinks {
                        let symbol = Self::extract_symbol_from_sink_static(&analysis, sink)
                            .unwrap_or_else(|| "unknown".to_string());

                        let cross_sink: CrossFileTaintSink = (
                            sink,
                            path_str.as_str(),
                            byte_offset_to_line(&content, sink.span().0),
                            symbol.as_str(),
                        ).into();
                        sink_data.push(cross_sink);
                    }

                    // Now add to tracker
                    if let Some(ref mut tracker) = self.cross_file_taint {
                        for source in source_data {
                            tracker.add_source(source);
                        }
                        for sink in sink_data {
                            tracker.add_sink(sink);
                        }
                    }
                }
            }
        }
        
        // Find cross-file flows
        let cross_file_flows = if let Some(ref tracker) = self.cross_file_taint {
            tracker.find_cross_file_flows().collect()
        } else {
            Vec::new()
        };
        
        // Deduplicate findings
        let (findings, dedup_stats) = self.deduplicate_findings(all_findings);
        
        // Sort by line, then column
        let mut sorted_findings = findings;
        sorted_findings.sort_by(|a, b| a.line.cmp(&b.line).then(a.column.cmp(&b.column)));
        
        // Run attack graph correlation if enabled
        let (attack_chains, threat_score) = if let Some(graph_engine) = &self.attack_graph {
            let mut engine = graph_engine.clone();
            engine.add_findings(sorted_findings.clone());
            let chains = engine.get_chains().to_vec();
            let score = engine.get_threat_score();
            (chains, score)
        } else {
            (Vec::new(), 0.0)
        };
        
        // Get campaign info (use first file for package name extraction)
        let campaign_info = if let Some(first_file) = files.first() {
            self.get_campaign_info(first_file, &sorted_findings)
        } else {
            None
        };
        
        Ok(ScanResult {
            findings: sorted_findings,
            #[cfg(feature = "llm")]
            llm_verdicts: Vec::new(), // LLM not run in package scan
            dedup_stats: Some(dedup_stats),
            cache_stats: None, // Cache not used in package scan
            attack_chains,
            threat_score,
            campaign_info,
            cross_file_flows,
        })
    }
    
    /// Simple package scan without cross-file analysis
    fn scan_package_simple(&self, package_path: &Path) -> std::io::Result<ScanResult> {
        let files = self.collect_js_files(package_path)?;
        let mut all_findings: Vec<Finding> = Vec::new();
        
        for file in &files {
            let content = fs::read_to_string(file)?;
            let result = self.scan_internal(file, &content);
            all_findings.extend(result.findings);
        }
        
        let (findings, dedup_stats) = self.deduplicate_findings(all_findings);
        
        Ok(ScanResult {
            findings,
            #[cfg(feature = "llm")]
            llm_verdicts: Vec::new(),
            dedup_stats: Some(dedup_stats),
            cache_stats: None,
            attack_chains: Vec::new(),
            threat_score: 0.0,
            campaign_info: None,
            #[cfg(feature = "semantic")]
            cross_file_flows: Vec::new(),
        })
    }
    
    /// Collect all JavaScript/TypeScript files in a directory
    #[cfg(feature = "semantic")]
    fn collect_js_files(&self, dir: &Path) -> std::io::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        if dir.is_file() {
            return Ok(vec![dir.to_path_buf()]);
        }
        
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                // Skip node_modules, .git, etc.
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if matches!(name, "node_modules" | ".git" | "target" | "dist" | "build") {
                        continue;
                    }
                }
                files.extend(self.collect_js_files(&path)?);
            } else if path.is_file() {
                // Check if it's a JS/TS file
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if matches!(ext, "js" | "mjs" | "cjs" | "ts" | "tsx" | "jsx") {
                        files.push(path);
                    }
                }
            }
        }
        
        Ok(files)
    }
    
    /// Extract symbol name from a taint source
    #[cfg(feature = "semantic")]
    fn extract_symbol_from_source_static(
        analysis: &crate::semantic::SemanticAnalysis,
        source: &crate::taint::TaintSource,
    ) -> Option<String> {
        // Try to find the declaration associated with this source
        if let Some(symbol_id) = source.assigned_to() {
            if let Some(decl) = analysis.declarations.iter().find(|d| d.symbol_id == symbol_id) {
                return Some(decl.name.clone());
            }
        }
        None
    }

    /// Extract symbol name from a taint sink
    #[cfg(feature = "semantic")]
    fn extract_symbol_from_sink_static(
        analysis: &crate::semantic::SemanticAnalysis,
        sink: &crate::taint::TaintSink,
    ) -> Option<String> {
        // For dynamic exec sinks, use the callee name
        // Note: Currently we only have DynamicExec sink type
        let crate::taint::TaintSink::DynamicExec { .. } = sink;
        
        // Find the call site
        if let Some(call) = analysis.call_sites.iter().find(|c| c.span == sink.span()) {
            return Some(call.callee.clone());
        }
        None
    }

    /// Get campaign information for a scanned package
    fn get_campaign_info(&self, path: &Path, _findings: &[Finding]) -> Option<CampaignInfo> {
        // Extract package name from path
        let package_name = Self::extract_package_name_from_path(path)?;

        // Check if campaign intelligence is enabled
        let intel = self.campaign_intelligence.as_ref()?;

        // Check if package belongs to a campaign
        let campaign = intel.get_package_campaign(&package_name)?;

        // Build campaign info
        Some(CampaignInfo {
            campaign_id: Some(campaign.id.clone()),
            related_packages: campaign.package_names().iter().map(|s| s.to_string()).collect(),
            shared_infrastructure: {
                let mut infra = Vec::new();
                infra.extend(campaign.infrastructure.domains.iter().map(|d| format!("domain:{}", d)));
                infra.extend(campaign.infrastructure.wallets.iter().map(|w| format!("wallet:{}", w)));
                infra.extend(campaign.infrastructure.authors.iter().map(|a| format!("author:{}", a)));
                infra
            },
        })
    }

    /// Extract package name from file path
    fn extract_package_name_from_path(path: &Path) -> Option<String> {
        // Try to extract npm package name (e.g., node_modules/@scope/pkg/file.js)
        let components: Vec<_> = path.components().collect();
        for (i, comp) in components.iter().enumerate() {
            if let Some(name) = comp.as_os_str().to_str() {
                if name == "node_modules" && i + 1 < components.len() {
                    let mut pkg_name = components[i + 1]
                        .as_os_str()
                        .to_string_lossy()
                        .to_string();

                    // Handle scoped packages (@scope/pkg)
                    if pkg_name.starts_with('@') && i + 2 < components.len() {
                        let scope = &pkg_name;
                        let pkg = components[i + 2]
                            .as_os_str()
                            .to_string_lossy()
                            .to_string();
                        pkg_name = format!("{}/{}", scope, pkg);
                    }

                    return Some(pkg_name);
                }
            }
        }

        // Fallback: use directory name or file stem
        path.parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .or_else(|| path.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string()))
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

    /// Add a package to campaign intelligence tracking
    ///
    /// This allows tracking packages across multiple scans for campaign detection.
    /// Call this method for each package you want to track in campaign analysis.
    ///
    /// # Arguments
    /// * `package` - The analyzed package to add
    ///
    /// # Returns
    /// true if the package was added, false if campaign intelligence is not enabled
    pub fn add_package_to_campaign(&mut self, package: AnalyzedPackage) -> bool {
        if let Some(ref mut intel) = self.campaign_intelligence {
            intel.add_package(package);
            true
        } else {
            false
        }
    }

    /// Get campaign intelligence statistics
    ///
    /// Returns infrastructure reuse statistics if campaign intelligence is enabled.
    pub fn get_campaign_stats(&self) -> Option<crate::campaign::InfrastructureStats> {
        self.campaign_intelligence.as_ref().map(|i| i.get_infrastructure_stats())
    }

    /// Get detected campaigns
    ///
    /// Returns a slice of detected campaigns if campaign intelligence is enabled.
    pub fn get_campaigns(&self) -> Option<&[crate::campaign::Campaign]> {
        self.campaign_intelligence.as_ref().map(|i| i.get_campaigns())
    }

    /// Get the number of registered detectors.
    pub fn detector_count(&self) -> usize {
        let mut count = self.detectors.len();
        #[cfg(feature = "semantic")]
        {
            count += self.semantic_detectors.len();
        }
        count
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
        // 13 base detectors (Unicode, EncryptedPayload, HeaderC2, RDD, JPD, ForceMemo,
        // Locale, TimeDelay, BlockchainC2, BrowserKill, TypoAttribution, ExfilSchema, SocketIOC2)
        // + 5 binary detectors (G6, G7, G8, G9, G11) when binary feature is enabled
        // + 4 semantic detectors (GW005-GW008) when semantic feature is enabled
        #[cfg(all(feature = "semantic", feature = "binary"))]
        assert_eq!(engine.detector_count(), 22);
        #[cfg(all(feature = "semantic", not(feature = "binary")))]
        assert_eq!(engine.detector_count(), 17);
        #[cfg(all(not(feature = "semantic"), feature = "binary"))]
        assert_eq!(engine.detector_count(), 18);
        #[cfg(all(not(feature = "semantic"), not(feature = "binary")))]
        assert_eq!(engine.detector_count(), 13);
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

    // DAG Execution Tests
    #[test]
    fn test_dag_construction() {
        use crate::detectors::invisible::InvisibleCharDetector;
        use crate::detectors::homoglyph::HomoglyphDetector;
        use crate::detectors::glassware::GlasswareDetector;

        let detectors: Vec<Box<dyn Detector>> = vec![
            Box::new(InvisibleCharDetector::with_default_config()),
            Box::new(HomoglyphDetector::with_default_config()),
            Box::new(GlasswareDetector::with_default_config()),
        ];

        let dag = DetectorDAG::new(detectors);
        let order = dag.execution_order();

        // invisible_char and homoglyph should run before glassware (prerequisites)
        let invisible_idx = order.iter().position(|&n| n == "invisible_char").unwrap();
        let homoglyph_idx = order.iter().position(|&n| n == "homoglyph").unwrap();
        let glassware_idx = order.iter().position(|&n| n == "glassware").unwrap();

        assert!(invisible_idx < glassware_idx);
        assert!(homoglyph_idx < glassware_idx);
    }

    #[test]
    fn test_dag_execution_order_by_cost_and_signal() {
        use crate::detectors::invisible::InvisibleCharDetector;
        use crate::detectors::bidi::BidiDetector;
        use crate::detectors::tags::UnicodeTagDetector;

        // All Tier 1, should be ordered by cost (ascending) then signal (descending)
        let detectors: Vec<Box<dyn Detector>> = vec![
            Box::new(UnicodeTagDetector::with_default_config()),  // cost=1, signal=7
            Box::new(InvisibleCharDetector::with_default_config()),  // cost=1, signal=9
            Box::new(BidiDetector::with_default_config()),  // cost=1, signal=9
        ];

        let dag = DetectorDAG::new(detectors);
        let order = dag.execution_order();

        // invisible_char and bidi should come before unicode_tag (higher signal)
        let invisible_idx = order.iter().position(|&n| n == "invisible_char").unwrap();
        let bidi_idx = order.iter().position(|&n| n == "bidi").unwrap();
        let tag_idx = order.iter().position(|&n| n == "unicode_tag").unwrap();

        assert!(invisible_idx < tag_idx);
        assert!(bidi_idx < tag_idx);
    }

    #[test]
    fn test_dag_execution_produces_findings() {
        use crate::detectors::invisible::InvisibleCharDetector;

        let detectors: Vec<Box<dyn Detector>> = vec![
            Box::new(InvisibleCharDetector::with_default_config()),
        ];

        let dag = DetectorDAG::new(detectors);
        let ir = FileIR::build(Path::new("test.js"), "const secret\u{FE00}Key = 'value';");

        let findings = dag.execute(&ir);
        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.category == DetectionCategory::InvisibleCharacter));
    }

    #[test]
    fn test_dag_short_circuit() {
        use crate::detector::DetectorTier;
        use crate::finding::{DetectionCategory, Finding, Severity};

        // Create a mock detector that short-circuits
        struct ShortCircuitDetector;
        impl Detector for ShortCircuitDetector {
            fn name(&self) -> &str { "short_circuit" }
            fn tier(&self) -> DetectorTier { DetectorTier::Tier1Primary }
            fn detect(&self, _ir: &FileIR) -> Vec<Finding> {
                vec![Finding::new(
                    "test", 1, 1, 0, '\0',
                    DetectionCategory::Unknown, Severity::Low,
                    "test", "test",
                )]
            }
            fn cost(&self) -> u8 { 1 }
            fn signal_strength(&self) -> u8 { 5 }
            fn should_short_circuit(&self, findings: &[Finding]) -> bool {
                !findings.is_empty()  // Short-circuit after first finding
            }
        }

        // Create a detector that should be short-circuited
        struct NeverRunDetector;
        impl Detector for NeverRunDetector {
            fn name(&self) -> &str { "never_run" }
            fn tier(&self) -> DetectorTier { DetectorTier::Tier3Behavioral }
            fn detect(&self, _ir: &FileIR) -> Vec<Finding> {
                panic!("This should not be called due to short-circuit");
            }
            fn cost(&self) -> u8 { 10 }
            fn signal_strength(&self) -> u8 { 5 }
            fn prerequisites(&self) -> Vec<&'static str> {
                vec!["short_circuit"]
            }
        }

        let detectors: Vec<Box<dyn Detector>> = vec![
            Box::new(ShortCircuitDetector),
            Box::new(NeverRunDetector),
        ];

        let dag = DetectorDAG::new(detectors);
        let ir = FileIR::build(Path::new("test.js"), "test content");

        // Should not panic - NeverRunDetector should be short-circuited
        let findings = dag.execute(&ir);
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn test_dag_vs_sequential_same_results() {
        use crate::detectors::invisible::InvisibleCharDetector;
        use crate::detectors::bidi::BidiDetector;

        let ir = FileIR::build(Path::new("test.js"), "const secret\u{FE00}Key = 'value';");

        // Create detectors for DAG
        let dag_detectors: Vec<Box<dyn Detector>> = vec![
            Box::new(InvisibleCharDetector::with_default_config()),
            Box::new(BidiDetector::with_default_config()),
        ];

        // Create detectors for sequential
        let seq_detectors: Vec<Box<dyn Detector>> = vec![
            Box::new(InvisibleCharDetector::with_default_config()),
            Box::new(BidiDetector::with_default_config()),
        ];

        let dag = DetectorDAG::new(dag_detectors);

        // DAG execution
        let dag_findings = dag.execute(&ir);

        // Sequential execution
        let mut sequential_findings = Vec::new();
        for detector in &seq_detectors {
            sequential_findings.extend(detector.detect(&ir));
        }

        // Same number of findings
        assert_eq!(dag_findings.len(), sequential_findings.len());

        // Same categories
        let dag_categories: Vec<_> = dag_findings.iter().map(|f| &f.category).collect();
        let seq_categories: Vec<_> = sequential_findings.iter().map(|f| &f.category).collect();
        assert_eq!(dag_categories, seq_categories);
    }

    #[test]
    fn test_engine_with_dag_execution() {
        let mut engine = ScanEngine::default_detectors();
        engine = engine.with_dag_execution(true);

        let content = "const secret\u{FE00}Key = 'value';";
        let findings = engine.scan(Path::new("test.js"), content);

        // Should still find invisible character with DAG execution
        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.category == DetectionCategory::InvisibleCharacter));
    }
}

/// Convert byte offset to line number (1-indexed)
#[cfg(feature = "semantic")]
fn byte_offset_to_line(source: &str, offset: u32) -> usize {
    source
        .char_indices()
        .enumerate()
        .find(|(_, (idx, _))| *idx >= offset as usize)
        .map(|(line, _)| line + 1)
        .unwrap_or(1)
}
