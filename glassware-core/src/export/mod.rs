//! Export Module
//!
//! This module provides export capabilities for detection patterns,
//! specifically YARA rule generation.
//!
//! ## Components
//!
//! - **E5**: YARA rule export

pub mod yara;

pub use yara::{export_yara_rules, YaraRule, YaraExporter};
