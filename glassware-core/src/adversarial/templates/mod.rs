//! Payload Templates for Polymorphic Generation
//!
//! This module defines payload templates for generating polymorphic malicious payloads.
//! Each template represents a different attack pattern with variable slots for polymorphism.

pub mod glassware;
pub mod phantom_raven;
pub mod forcememo;

pub use glassware::GlassWareTemplate;
pub use forcememo::ForceMemoTemplate;
pub use phantom_raven::PhantomRavenTemplate;

use std::collections::HashMap;

/// A payload template with variable slots for polymorphic generation
pub trait PayloadTemplate: Send + Sync {
    /// Template name (e.g., "glassware", "phantom_raven", "forcememo")
    fn name(&self) -> &str;

    /// Template description
    fn description(&self) -> &str;

    /// Get the base payload structure with variable slots
    fn base_payload(&self) -> String;

    /// Get available variable slots in the template
    fn get_slots(&self) -> Vec<VariableSlot>;

    /// Generate a payload instance by filling slots with values
    fn generate(&self, slot_values: &HashMap<String, String>) -> String;

    /// Get the decoder/encoder logic for this template
    fn decoder_logic(&self) -> String;

    /// Get the execution pattern for this template
    fn exec_pattern(&self) -> String;
}

/// A variable slot that can be polymorphically varied
#[derive(Debug, Clone)]
pub struct VariableSlot {
    /// Slot identifier (used in templates as {{slot_id}})
    pub id: String,
    /// Slot type (variable_name, encoding, string_literal, etc.)
    pub slot_type: SlotType,
    /// Default value for the slot
    pub default_value: String,
    /// Description of what this slot controls
    pub description: String,
}

/// Types of variable slots
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotType {
    /// Variable/function name
    VariableName,
    /// Encoding method (base64, hex, custom)
    Encoding,
    /// String literal (can be obfuscated)
    StringLiteral,
    /// Control flow construct (if/switch/loop)
    ControlFlow,
    /// Unicode variation (VS-16 vs VS-17)
    UnicodeVariant,
    /// Numeric constant
    NumericConstant,
    /// Module import style
    ImportStyle,
}

impl VariableSlot {
    /// Create a new variable slot
    pub fn new(id: &str, slot_type: SlotType, default_value: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            slot_type,
            default_value: default_value.to_string(),
            description: description.to_string(),
        }
    }
}

/// Helper function to replace slots in a template string
pub fn replace_slots(template: &str, slot_values: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (slot_id, value) in slot_values {
        let placeholder = format!("{{{}}}", slot_id);
        result = result.replace(&placeholder, value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_slots() {
        let mut slots = HashMap::new();
        slots.insert("var1".to_string(), "myVar".to_string());
        slots.insert("var2".to_string(), "anotherVar".to_string());

        let template = "const {var1} = decode({var2});";
        let result = replace_slots(template, &slots);

        assert_eq!(result, "const myVar = decode(anotherVar);");
    }

    #[test]
    fn test_replace_slots_partial() {
        let mut slots = HashMap::new();
        slots.insert("var1".to_string(), "myVar".to_string());
        // var2 not provided - should remain as placeholder

        let template = "const {var1} = decode({var2});";
        let result = replace_slots(template, &slots);

        assert_eq!(result, "const myVar = decode({var2});");
    }
}
