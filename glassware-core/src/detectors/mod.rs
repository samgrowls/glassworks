//! Unicode Detector Modules
//!
//! This module contains individual detectors for different Unicode attack vectors.

pub mod bidi;
pub mod blockchain_polling;
pub mod browser_kill;
pub mod exfil_schema;
pub mod exfiltration;
pub mod glassware;
pub mod homoglyph;
pub mod invisible;
pub mod sandbox_evasion;
pub mod socketio_c2;
pub mod tags;
pub mod typo_attribution;
pub mod unicode_steganography_v2;

pub use bidi::BidiDetector;
pub use blockchain_polling::BlockchainPollingDetector;
pub use browser_kill::BrowserKillDetector;
pub use exfil_schema::ExfilSchemaDetector;
pub use exfiltration::ExfiltrationDetector;
pub use glassware::GlasswareDetector;
pub use homoglyph::HomoglyphDetector;
pub use invisible::InvisibleCharDetector;
pub use sandbox_evasion::SandboxEvasionDetector;
pub use socketio_c2::SocketIOC2Detector;
pub use tags::UnicodeTagDetector;
pub use typo_attribution::TypoAttributionDetector;
pub use unicode_steganography_v2::UnicodeSteganographyV2Detector;
