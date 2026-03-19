//! Unicode Detector
//!
//! Wraps the existing Unicode scanning logic to implement the Detector trait.

use crate::config::UnicodeConfig;
use crate::detector::{Detector, DetectorMetadata, ScanContext};
use crate::finding::Finding;
use crate::scanner::UnicodeScanner;
use std::path::Path;

/// Unicode attack detector implementing the Detector trait.
///
/// This detector wraps the existing Unicode scanning logic including:
/// - Invisible character detection (zero-width, variation selectors)
/// - Homoglyph/confusable character detection
/// - Bidirectional override detection
/// - Glassware pattern detection
/// - Unicode tag detection
pub struct UnicodeDetector;

impl Detector for UnicodeDetector {
    fn name(&self) -> &str {
        "unicode"
    }

    fn detect(&self, ctx: &ScanContext) -> Vec<Finding> {
        // Use the existing UnicodeScanner to perform the scan
        let scanner = UnicodeScanner::new(ctx.config.clone());
        scanner.scan(&ctx.content, &ctx.file_path)
    }

    fn metadata(&self) -> DetectorMetadata {
        DetectorMetadata {
            name: "unicode".to_string(),
            version: "1.0.0".to_string(),
            description: "Detects Unicode-based attacks including invisible characters, homoglyphs, and bidirectional overrides".to_string().to_string(),
        }
    }
}

impl Default for UnicodeDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl UnicodeDetector {
    /// Create a new Unicode detector
    pub fn new() -> Self {
        Self
    }

    /// Backward compatibility method for tests
    pub fn scan(&self, path: &Path, content: &str, config: &UnicodeConfig) -> Vec<Finding> {
        let ctx = ScanContext::new(path.to_string_lossy().to_string(), content.to_string(), config.clone());
        self.detect(&ctx)
    }
}
