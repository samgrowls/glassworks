//! Hybrid Patterns Fuzz Strategy
//!
//! Mixes benign and malicious patterns to test signal dilution detection.
//! This strategy attempts to hide malicious payloads within legitimate-looking code.

use super::super::fuzzer::FuzzStrategy;
use rand::Rng;
use rand::prelude::SliceRandom;

/// Hybrid patterns fuzz strategy
///
/// Combines legitimate code patterns with hidden malicious content
/// to test if detectors can identify threats amid noise:
/// - Benign comments with hidden payloads
/// - Legitimate function names with malicious internals
/// - Normal imports with embedded steganography
pub struct HybridPatternsStrategy;

/// Benign-looking function prefixes
const BENIGN_PREFIX: [&'static str; 10] = [
    "// Utility function for string processing\nfunction processString(s) { ",
    "// Helper method for data validation\nfunction validateData(d) { ",
    "// Common utility for formatting\nfunction formatOutput(o) { ",
    "// Standard logging helper\nfunction logMessage(m) { ",
    "// Configuration loader\nfunction loadConfig(c) { ",
    "/* Initialize application */\nfunction init() { ",
    "// Event handler for user input\nfunction handleInput(e) { ",
    "// Data transformation utility\nfunction transform(t) { ",
    "// Cache management function\nfunction cacheGet(k) { ",
    "// Error handling wrapper\nfunction handleError(err) { ",
];

/// Benign-looking suffixes
const BENIGN_SUFFIX: [&'static str; 10] = [
    " }",
    " return true; }",
    " return false; }",
    " return null; }",
    " return undefined; }",
    " console.log('done'); }",
    " return result; }",
    " throw new Error('failed'); }",
    " return Promise.resolve(); }",
    " export default this; }",
];

/// Invisible characters for embedding
const INVISIBLE_CHARS: [char; 5] = [
    '\u{200B}', // Zero-width space
    '\u{200C}', // Zero-width non-joiner
    '\u{200D}', // Zero-width joiner
    '\u{FEFF}', // BOM / Zero-width no-break space
    '\u{00AD}', // Soft hyphen
];

impl HybridPatternsStrategy {
    /// Embed hidden payload using invisible characters
    fn embed_payload<R: Rng>(&self, rng: &mut R, payload: &str, benign: &str) -> String {
        let mut result = String::new();
        let invisible_seq: String = payload
            .chars()
            .flat_map(|c| {
                // Encode each character as sequence of invisible chars
                let byte = c as u32;
                (0..8).map(move |i| {
                    let bit = (byte >> i) & 1;
                    if bit == 0 {
                        INVISIBLE_CHARS[0] // ZWSP for 0
                    } else {
                        INVISIBLE_CHARS[1] // ZWNJ for 1
                    }
                })
            })
            .collect();

        // Insert invisible payload at random position in benign code
        let insert_pos = rng.gen_range(0..=benign.len());
        result.push_str(&benign[..insert_pos]);
        result.push_str(&invisible_seq);
        result.push_str(&benign[insert_pos..]);
        result
    }

    /// Mix benign and malicious patterns
    fn dilute_signal<R: Rng>(&self, rng: &mut R, input: &str, intensity: f32) -> String {
        let prefix = BENIGN_PREFIX.choose(rng).unwrap();
        let suffix = BENIGN_SUFFIX.choose(rng).unwrap();

        // Higher intensity = more dilution (more benign code)
        let noise_factor = (intensity * 5.0) as usize;
        let mut result = String::from(*prefix);

        // Add input (potentially malicious) wrapped in benign code
        result.push_str(input);

        // Add noise lines
        for _ in 0..noise_factor {
            let noise = match rng.gen_range(0..5) {
                0 => format!("\n// Comment {}", rng.gen_range(0..1000)),
                1 => format!("\nconst _{} = {};", rng.gen_range(0..100), rng.gen_range(0..100)),
                2 => format!("\nvar temp{} = null;", rng.gen_range(0..100)),
                3 => format!("\nlet unused{} = undefined;", rng.gen_range(0..100)),
                _ => format!("\n/* block comment {} */", rng.gen_range(0..100)),
            };
            result.push_str(&noise);
        }

        result.push_str(suffix);
        result
    }

    /// Create camelCase homoglyph attack
    fn homoglyph_mix<R: Rng>(&self, rng: &mut R, input: &str) -> String {
        // Replace some Latin characters with Cyrillic lookalikes
        input
            .chars()
            .map(|c| {
                if rng.gen_bool(0.3) {
                    // Replace with homoglyph
                    match c {
                        'a' => '\u{0430}', // Cyrillic а
                        'e' => '\u{0435}', // Cyrillic е
                        'o' => '\u{043E}', // Cyrillic о
                        'p' => '\u{0440}', // Cyrillic р
                        'c' => '\u{0441}', // Cyrillic с
                        'x' => '\u{0445}', // Cyrillic х
                        'y' => '\u{0443}', // Cyrillic у
                        _ => c,
                    }
                } else {
                    c
                }
            })
            .collect()
    }
}

impl FuzzStrategy for HybridPatternsStrategy {
    fn name(&self) -> &str {
        "hybrid_patterns"
    }

    fn description(&self) -> &str {
        "Mix benign + malicious patterns (signal dilution, homoglyph attacks, embedded payloads)"
    }

    fn fuzz(&self, input: &str, intensity: f32) -> String {
        use rand::thread_rng;
        let mut rng = thread_rng();

        // Clamp intensity to [0.0, 1.0]
        let intensity = intensity.clamp(0.0, 1.0);

        // At zero intensity, return input unchanged
        if intensity == 0.0 {
            return input.to_string();
        }

        // Choose hybrid type
        let hybrid_type = rng.gen_range(0..4);

        match hybrid_type {
            0 => self.embed_payload(&mut rng, input, input),
            1 => self.dilute_signal(&mut rng, input, intensity),
            2 => self.homoglyph_mix(&mut rng, input),
            _ => {
                // Combine multiple techniques
                let diluted = self.dilute_signal(&mut rng, input, intensity);
                self.homoglyph_mix(&mut rng, &diluted)
            }
        }
    }
}

impl Default for HybridPatternsStrategy {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_zero_intensity() {
        let strategy = HybridPatternsStrategy;
        let input = "test string";
        let fuzzed = strategy.fuzz(input, 0.0);

        // Should be unchanged at zero intensity
        assert_eq!(fuzzed, input);
    }

    #[test]
    fn test_hybrid_dilution() {
        let strategy = HybridPatternsStrategy;
        let input = "malicious_code()";
        
        // Run multiple times since hybrid type is random
        let mut found_dilution = false;
        for _ in 0..30 {
            let fuzzed = strategy.fuzz(input, 1.0);
            
            // Should be longer due to added benign code
            if fuzzed.len() > input.len() {
                // Check for benign patterns
                if fuzzed.contains("function") || fuzzed.contains("//") || fuzzed.contains("/*") {
                    found_dilution = true;
                    break;
                }
            }
        }
        
        assert!(found_dilution, "Should generate diluted patterns at some point");
    }

    #[test]
    fn test_hybrid_homoglyph() {
        let strategy = HybridPatternsStrategy;
        let input = "password";
        
        // Run multiple times to hit homoglyph case
        let mut found_homoglyph = false;
        for _ in 0..30 {
            let fuzzed = strategy.fuzz(input, 0.8);
            // Check for Cyrillic lookalikes
            if fuzzed.contains('\u{0430}') || 
               fuzzed.contains('\u{0435}') || 
               fuzzed.contains('\u{043E}') ||
               fuzzed.contains('\u{0440}') {
                found_homoglyph = true;
                break;
            }
        }
        
        assert!(found_homoglyph, "Should generate homoglyph characters at some point");
    }
}
