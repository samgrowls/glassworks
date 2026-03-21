//! Binary Scanning Module
//!
//! This module provides binary analysis capabilities for `.node` files
//! (renamed native shared libraries: PE/ELF/Mach-O).
//!
//! ## Architecture
//!
//! The binary scanner extracts features using `goblin` (zero-copy parser) and
//! runs detectors against the extracted features. Output is the same `Finding`
//! type as the JS/TS detectors, ensuring unified reporting.
//!
//! ## Detectors
//!
//! - **G6**: XorShift128 obfuscation (heuristic-based)
//! - **G7**: IElevator COM CLSID detection
//! - **G8**: APC injection signatures
//! - **G9**: memexec crate usage
//! - **G11**: .node metadata (PDB paths, build info)

pub mod extractor;

#[cfg(feature = "binary")]
pub mod xorshift;
#[cfg(feature = "binary")]
pub mod ielevator;
#[cfg(feature = "binary")]
pub mod apc_injection;
#[cfg(feature = "binary")]
pub mod memexec;
#[cfg(feature = "binary")]
pub mod metadata;

#[cfg(feature = "binary")]
pub use extractor::{
    extract_features, BinaryFeatures, BinaryFormat, DebugInfo, ExportEntry,
    ExtractedString, ImportEntry, SectionInfo,
};

#[cfg(feature = "binary")]
pub use xorshift::XorShiftDetector;
#[cfg(feature = "binary")]
pub use ielevator::IElevatorDetector;
#[cfg(feature = "binary")]
pub use apc_injection::ApcInjectionDetector;
#[cfg(feature = "binary")]
pub use memexec::MemexecDetector;
#[cfg(feature = "binary")]
pub use metadata::MetadataDetector;

/// Binary scanner trait for orchestrating binary detectors
#[cfg(feature = "binary")]
pub trait BinaryScanner: Send + Sync {
    /// Get scanner name
    fn name(&self) -> &str;

    /// Run detection on binary features
    fn scan(&self, features: &BinaryFeatures, path: &str) -> Vec<crate::finding::Finding>;
}
