//! Detector Trait
//!
//! Defines the interface for detection modules that scan file content
//! for suspicious patterns.

use crate::config::UnicodeConfig;
use crate::finding::Finding;
use std::path::Path;

/// Context provided to detectors for scanning
///
/// This struct contains all information needed by a detector to perform its analysis.
pub struct ScanContext {
    /// Path to the file being scanned
    pub file_path: String,
    /// Content of the file being scanned
    pub content: String,
    /// Configuration for the scan
    pub config: UnicodeConfig,
}

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

/// A detection module that scans file content for suspicious patterns.
///
/// Each detector targets a specific class of attack technique.
/// The engine runs all registered detectors against each file.
pub trait Detector: Send + Sync {
    /// Get detector name
    fn name(&self) -> &str;

    /// Run detection on the provided context
    fn detect(&self, ctx: &ScanContext) -> Vec<Finding>;

    /// Get detector metadata (optional)
    fn metadata(&self) -> DetectorMetadata {
        DetectorMetadata {
            name: self.name().to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
        }
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
