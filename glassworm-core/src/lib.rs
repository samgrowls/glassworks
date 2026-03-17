//! Glassworm Core - Unicode Attack Detection Library
//!
//! A zero-dependency library for detecting invisible Unicode characters and
//! trojan source attacks in source code.
//!
//! ## Features
//!
//! - **Invisible Character Detection**: Zero-width characters, variation selectors,
//!   bidi overrides used in Glassworm-style attacks
//! - **Homoglyph Detection**: Confusable characters from Cyrillic, Greek scripts
//! - **Bidirectional Override Detection**: Bidi control characters that reverse text
//! - **Glassworm Pattern Detection**: Decoder patterns, eval usage, encoding functions
//! - **Unicode Tag Detection**: Tag characters for metadata injection
//!
//! ## Example Usage
//!
//! ```rust
//! use glassworm_core::{scan, UnicodeFinding};
//!
//! // Scan content for Unicode attacks
//! let content = "const secret\u{FE00}Key = 'value';";
//! let findings = scan(content, "test.js");
//!
//! // Process findings
//! for finding in findings {
//!     println!("Found: {} at {}:{}", finding.description, finding.line, finding.column);
//! }
//! ```
//!
//! ## Performance
//!
//! - Time complexity: O(n) where n = number of characters
//! - Space complexity: O(1) beyond input storage
//! - Confusables lookup: O(1) using HashMap

pub mod classify;
pub mod config;
pub mod confusables;
pub mod decoder;
pub mod detectors;
pub mod finding;
pub mod ranges;
pub mod scanner;
pub mod script_detector;

// Re-export main types for convenience
pub use classify::{
    get_bidi_name, get_zero_width_name, is_in_critical_range, is_in_invisible_range,
    is_variation_selector, BidiChar, InvisibleRange, ZeroWidthChar,
};

pub use config::{DetectorConfig, SensitivityLevel, UnicodeConfig};

pub use confusables::data::{
    get_base_char, get_confusable_script, get_similarity, is_confusable, ConfusableEntry,
};

pub use decoder::{
    count_vs_codepoints, decode_vs_stego, find_vs_runs, is_vs_codepoint, shannon_entropy,
    DecodedPayload, PayloadClass,
};

pub use detectors::{
    BidiDetector, GlasswormDetector, HomoglyphDetector, InvisibleCharDetector, UnicodeTagDetector,
};

pub use finding::{Severity, SourceLocation, UnicodeCategory, UnicodeFinding};

pub use ranges::{CRITICAL_RANGES, INVISIBLE_RANGES};

pub use scanner::{scan, ScanSessionStats, UnicodeScanner};

pub use script_detector::{
    get_script, get_scripts_in_identifier, has_mixed_scripts, is_high_risk_script, is_pure_latin,
    is_pure_non_latin,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_variation_selector() {
        let content = "const secret\u{FE00}Key = 'value';";
        let findings = scan(content, "test.js");

        assert!(!findings.is_empty());
        assert!(findings
            .iter()
            .any(|f| f.category == UnicodeCategory::InvisibleCharacter));
    }

    #[test]
    fn test_scan_homoglyph() {
        let content = "const pаssword = 'secret';"; // Cyrillic 'а'
        let findings = scan(content, "test.js");

        assert!(!findings.is_empty());
        assert!(findings
            .iter()
            .any(|f| f.category == UnicodeCategory::Homoglyph));
    }

    #[test]
    fn test_scan_bidi() {
        let content = "const file = \"test\u{202E}exe\";";
        let findings = scan(content, "test.js");

        assert!(!findings.is_empty());
        assert!(findings
            .iter()
            .any(|f| f.category == UnicodeCategory::BidirectionalOverride));
    }

    #[test]
    fn test_clean_content() {
        let content = "const normal = 'hello world';";
        let findings = scan(content, "test.js");

        assert!(findings.is_empty());
    }
}
