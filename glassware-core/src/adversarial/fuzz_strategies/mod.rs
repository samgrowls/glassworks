//! Fuzz Strategies
//!
//! Various fuzz strategies for adversarial testing of glassware detectors.
//!
//! Each strategy implements the `FuzzStrategy` trait and provides
//! different approaches to generating test inputs.

pub mod random_unicode;
pub mod boundary;
pub mod malformed;
pub mod hybrid;
pub mod size_variation;

pub use random_unicode::RandomUnicodeStrategy;
pub use boundary::BoundaryStrategy;
pub use malformed::MalformedInputStrategy;
pub use hybrid::HybridPatternsStrategy;
pub use size_variation::SizeVariationStrategy;
