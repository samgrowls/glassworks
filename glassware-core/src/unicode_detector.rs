//! Unicode Detector
//!
//! Wraps the existing Unicode scanning logic to implement the Detector trait.

use crate::detector::{Detector, DetectorMetadata};
use crate::finding::Finding;
use crate::ir::FileIR;
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

    fn detect(&self, ir: &FileIR) -> Vec<Finding> {
        // Use the existing UnicodeScanner to perform the scan
        // Note: UnicodeScanner still uses its own internal logic
        // The IR's unicode analysis is available via ir.unicode() if needed
        let scanner = UnicodeScanner::new(crate::config::UnicodeConfig::default());
        scanner.scan(ir.content(), &ir.metadata.path)
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
    pub fn scan(&self, path: &Path, content: &str, config: &crate::config::UnicodeConfig) -> Vec<Finding> {
        // Build IR and call detect (for backward compatibility)
        let ir = FileIR::build(path, content);
        // Use scanner with provided config
        let scanner = UnicodeScanner::new(config.clone());
        scanner.scan(content, &path.to_string_lossy())
    }
}
