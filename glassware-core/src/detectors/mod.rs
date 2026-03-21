//! Unicode Detector Modules
//!
//! This module contains individual detectors for different Unicode attack vectors.

pub mod bidi;
pub mod browser_kill;
pub mod glassware;
pub mod homoglyph;
pub mod invisible;
pub mod tags;

pub use bidi::BidiDetector;
pub use browser_kill::BrowserKillDetector;
pub use glassware::GlasswareDetector;
pub use homoglyph::HomoglyphDetector;
pub use invisible::InvisibleCharDetector;
pub use tags::UnicodeTagDetector;
