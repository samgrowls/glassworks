//! Adversarial Testing Framework
//!
//! This module provides adversarial testing capabilities for glassware detectors.

pub mod fuzz_strategies;
pub mod fuzzer;
pub mod mutation;
pub mod runner;
pub mod strategies;
// NEW: Polymorphic payload generator
pub mod polymorphic;
pub mod templates;
// NEW: Test generator for CI/CD integration
pub mod test_generator;

pub use fuzz_strategies::{
    BoundaryStrategy,
    HybridPatternsStrategy,
    MalformedInputStrategy,
    RandomUnicodeStrategy,
    SizeVariationStrategy,
};
pub use fuzzer::{FuzzResult, FuzzStrategy, FuzzerEngine};
pub use mutation::{MaliciousPayload, MutationEngine};
pub use runner::AdversarialRunner;
// NEW: Polymorphic generator re-exports
pub use polymorphic::{PolymorphicGenerator, GeneratorStats};
pub use templates::{
    PayloadTemplate,
    VariableSlot,
    SlotType,
    GlassWareTemplate,
    PhantomRavenTemplate,
    ForceMemoTemplate,
};
// NEW: Test generator re-exports
pub use test_generator::{EvasionSeverity, EvasionTestCase, TestGenerator, TestGeneratorStats};
