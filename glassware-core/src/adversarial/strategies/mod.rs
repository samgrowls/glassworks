//! Mutation Strategies
//!
//! Various mutation strategies for adversarial testing.

pub mod unicode;
pub mod variable;
pub mod encoding;

pub use unicode::UnicodeSubstitutionStrategy;
pub use variable::VariableRenamingStrategy;
pub use encoding::EncodingVariationStrategy;
