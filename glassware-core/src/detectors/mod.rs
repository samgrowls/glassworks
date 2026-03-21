//! Unicode Detector Modules
//!
//! This module contains individual detectors for different Unicode attack vectors.

pub mod bidi;
pub mod browser_kill;
pub mod exfil_schema;
pub mod glassware;
pub mod homoglyph;
pub mod invisible;
pub mod socketio_c2;
pub mod tags;
pub mod typo_attribution;

pub use bidi::BidiDetector;
pub use browser_kill::BrowserKillDetector;
pub use exfil_schema::ExfilSchemaDetector;
pub use glassware::GlasswareDetector;
pub use homoglyph::HomoglyphDetector;
pub use invisible::InvisibleCharDetector;
pub use socketio_c2::SocketIOC2Detector;
pub use tags::UnicodeTagDetector;
pub use typo_attribution::TypoAttributionDetector;
