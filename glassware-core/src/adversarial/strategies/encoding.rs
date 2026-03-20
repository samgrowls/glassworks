//! Encoding Variation Mutation Strategy
//!
//! Changes encoding schemes to evade detection.
//! Example: Base64 → Hex, UTF-8 → UTF-16

use super::super::mutation::MutationStrategy;
use rand::Rng;

/// Encoding variation mutation strategy
pub struct EncodingVariationStrategy;

impl MutationStrategy for EncodingVariationStrategy {
    fn name(&self) -> &str {
        "encoding_variation"
    }
    
    fn description(&self) -> &str {
        "Change encoding schemes (Base64 → Hex, UTF-8 → UTF-16, etc.)"
    }
    
    fn mutate(&self, payload: &str, rate: f32) -> String {
        use rand::thread_rng;
        let mut rng = thread_rng();
        // String-level mutations
        payload
            // Base64 → Hex-like pattern
            .replace("from('base64')", "from('hex')")
            .replace("from(\"base64\")", "from(\"hex\")")
            .replace("atob(", "hexDecode(")
            .replace("btoa(", "hexEncode(")
            .replace("Buffer.from", "Buffer.hexFrom")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;
    
    #[test]
    fn test_encoding_variation_base64() {
        let strategy = EncodingVariationStrategy;
        let input = "Buffer.from('base64')";
        let mut rng = thread_rng();
        let mutated = strategy.mutate(&input, 1.0);  // 100% rate
        
        // Should contain hex instead of base64
        assert!(mutated.contains("hex"));
    }
    
    #[test]
    fn test_encoding_variation_atob() {
        let strategy = EncodingVariationStrategy;
        let input = "atob(payload)";
        let mut rng = thread_rng();
        let mutated = strategy.mutate(&input, 1.0);  // 100% rate
        
        // Should contain hexDecode instead of atob
        assert!(mutated.contains("hexDecode"));
    }
}
