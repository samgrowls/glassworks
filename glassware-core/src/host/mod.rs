//! Host Forensics Module
//!
//! This module provides host-level forensics capabilities for detecting
//! GlassWorm persistence artifacts on the local filesystem.
//!
//! ## Components
//!
//! - **G1**: Filesystem persistence scanner (dropped payloads, temp files, registry)
//! - **G2**: Chrome preference tampering detector (sideloaded extensions)
//! - **E2**: Host indicator enrichment (cross-correlation with JS/binary findings)

pub mod filesystem;
pub mod chrome;
pub mod enrichment;

pub use filesystem::{scan_filesystem, FilesystemScanner};
pub use chrome::{scan_chrome_profile, ChromePrefsScanner};
pub use enrichment::{enrich_findings, EnrichmentContext};
