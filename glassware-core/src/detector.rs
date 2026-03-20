//! Detector Trait
//!
//! Defines the interface for detection modules that scan file content
//! for suspicious patterns.

use crate::config::UnicodeConfig;
use crate::finding::Finding;
use crate::ir::FileIR;
use std::path::Path;

/// Context provided to detectors for scanning
///
/// This struct contains all information needed by a detector to perform its analysis.
/// 
/// # Deprecated
/// 
/// This struct is deprecated in favor of [`FileIR`]. Detectors should now accept
/// `&FileIR` directly instead of `&ScanContext`.
/// 
/// ## Migration Guide
/// 
/// **Before:**
/// ```rust
/// fn detect(&self, ctx: &ScanContext) -> Vec<Finding> {
///     let json: Value = serde_json::from_str(&ctx.content)?;
///     // ... detect
/// }
/// ```
/// 
/// **After:**
/// ```rust
/// fn detect(&self, ir: &FileIR) -> Vec<Finding> {
///     let json = ir.json().as_ref()?;  // Use pre-parsed JSON
///     // ... detect
/// }
/// ```
#[deprecated(since = "0.5.1", note = "Use FileIR instead for unified parsing")]
pub struct ScanContext {
    /// Path to the file being scanned
    pub file_path: String,
    /// Content of the file being scanned
    pub content: String,
    /// Configuration for the scan
    pub config: UnicodeConfig,
}

#[allow(deprecated)]
impl ScanContext {
    /// Create a new scan context
    pub fn new(file_path: String, content: String, config: UnicodeConfig) -> Self {
        Self {
            file_path,
            content,
            config,
        }
    }

    /// Create a scan context from a path and content
    pub fn from_path<P: AsRef<Path>>(
        path: P,
        content: String,
        config: UnicodeConfig,
    ) -> Self {
        Self {
            file_path: path.as_ref().to_string_lossy().to_string(),
            content,
            config,
        }
    }
}

/// Metadata about a detector
#[derive(Debug, Clone)]
pub struct DetectorMetadata {
    /// Human-readable name of the detector
    pub name: String,
    /// Version of the detector
    pub version: String,
    /// Description of what the detector does
    pub description: String,
}

/// Detector priority tier for phased scanning
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DetectorTier {
    /// Primary detectors - always run, low FP rate (invisible chars, homoglyphs, bidi)
    Tier1Primary = 1,
    /// Secondary detectors - run if Tier 1 finds something or file heuristics pass (glassware patterns, encrypted payload)
    Tier2Secondary = 2,
    /// Behavioral detectors - only run if Tier 1+2 find something (locale, time delay, blockchain C2)
    Tier3Behavioral = 3,
}

/// A detection module that scans file content for suspicious patterns.
///
/// Each detector targets a specific class of attack technique.
/// The engine runs all registered detectors against each file.
///
/// ## Detector Tiers
///
/// Detectors are organized into three tiers for optimal performance and false positive reduction:
///
/// - **Tier 1 (Primary)**: Always run, very low FP rate. Includes invisible character, homoglyph, and bidi detection.
/// - **Tier 2 (Secondary)**: Run if Tier 1 finds something OR file passes heuristics (not minified/bundled).
/// - **Tier 3 (Behavioral)**: Only run if Tier 1+2 find something. Includes locale geofencing, time delays, blockchain C2.
///
/// This tiered approach dramatically reduces false positives on legitimate codebases while maintaining
/// high detection accuracy for real attacks.
///
/// ## Unified IR Layer
///
/// As of version 0.5.1, detectors receive a [`FileIR`](crate::ir::FileIR) instance instead of raw content.
/// The IR is built once by the engine and contains:
/// - Pre-parsed JSON (for package.json files)
/// - Pre-parsed AST (for JS/TS files, when semantic feature is enabled)
/// - Unicode analysis results
/// - File metadata (minification, bundling detection)
///
/// This eliminates redundant parsing across detectors, improving performance by 20-30%.
///
/// ## Example
///
/// ```rust
/// use glassware_core::{Detector, FileIR, Finding};
///
/// struct MyDetector;
///
/// impl Detector for MyDetector {
///     fn name(&self) -> &str {
///         "my_detector"
///     }
///
///     fn detect(&self, ir: &FileIR) -> Vec<Finding> {
///         // Access pre-parsed JSON
///         if let Some(json) = ir.json() {
///             // Check dependencies...
///         }
///         
///         // Access pre-parsed AST (JS/TS only)
///         #[cfg(feature = "semantic")]
///         if let Some(ast) = ir.ast() {
///             // Analyze AST...
///         }
///         
///         // Access Unicode analysis
///         if ir.unicode().has_invisible {
///             // Report invisible characters...
///         }
///         
///         vec![]
///     }
/// }
/// ```
pub trait Detector: Send + Sync {
    /// Get detector name
    fn name(&self) -> &str;

    /// Get detector tier (default: Tier1)
    fn tier(&self) -> DetectorTier {
        DetectorTier::Tier1Primary
    }

    /// Run detection on the provided IR
    ///
    /// # Arguments
    /// * `ir` - The unified intermediate representation of the file
    ///
    /// # Returns
    /// A vector of findings detected in the file
    fn detect(&self, ir: &FileIR) -> Vec<Finding>;

    /// Get detector metadata (optional)
    fn metadata(&self) -> DetectorMetadata {
        DetectorMetadata {
            name: self.name().to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
        }
    }

    /// Get the computational cost of this detector (1-10, where 1=cheapest, 10=most expensive)
    ///
    /// This is used for execution ordering - cheaper detectors run first.
    fn cost(&self) -> u8 {
        5  // Default: medium cost
    }

    /// Get the signal strength of this detector (1-10, where 10=highest signal/lowest FP rate)
    ///
    /// This is used for execution ordering - higher signal detectors run first.
    fn signal_strength(&self) -> u8 {
        5  // Default: medium signal
    }

    /// Get the list of detector names that must run before this detector
    ///
    /// This creates execution dependencies in the DAG.
    fn prerequisites(&self) -> Vec<&'static str> {
        vec![]  // Default: no prerequisites
    }

    /// Check if this detector should short-circuit remaining execution based on current findings
    ///
    /// Returns true if remaining detectors should be skipped.
    fn should_short_circuit(&self, _findings: &[Finding]) -> bool {
        false  // Default: don't short-circuit
    }

    /// Check if this detector should run based on other findings
    ///
    /// Override for tiered detection logic:
    /// - Tier 1 detectors: Always return true
    /// - Tier 2 detectors: Return true if Tier 1 found something OR file passes heuristics
    /// - Tier 3 detectors: Return true only if Tier 1+2 found something
    fn should_run(&self, _other_findings: &[Finding]) -> bool {
        true  // Default: always run (Tier 1 behavior)
    }
}

/// Detector that operates on parsed semantic information (JS/TS only).
///
/// This trait is for detectors that use OXC semantic analysis
/// for more accurate flow-based detection.
#[cfg(feature = "semantic")]
pub trait SemanticDetector: Send + Sync {
    /// Unique identifier matching a GW rule (e.g., "GW005")
    fn id(&self) -> &str;

    /// Run detection using semantic analysis + taint flows.
    /// `sources` and `sinks` are pre-computed taint sources and sinks.
    fn detect_semantic(
        &self,
        source_code: &str,
        path: &Path,
        flows: &[crate::taint::TaintFlow],
        sources: &[crate::taint::TaintSource],
        sinks: &[crate::taint::TaintSink],
    ) -> Vec<Finding>;
}
