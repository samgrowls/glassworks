//! Malformed Input Fuzz Strategy
//!
//! Generates malformed syntax such as unclosed strings, invalid JSON,
//! mismatched brackets, and other syntax errors to test parser robustness.

use super::super::fuzzer::FuzzStrategy;
use rand::Rng;

/// Malformed input fuzz strategy
///
/// Generates syntactically invalid inputs to test if detectors
/// handle parsing errors gracefully:
/// - Unclosed strings/brackets
/// - Invalid JSON
/// - Mismatched quotes
/// - Truncated code
/// - Invalid escape sequences
pub struct MalformedInputStrategy;

impl MalformedInputStrategy {
    /// Generate unclosed string
    fn unclosed_string<R: Rng>(&self, rng: &mut R, input: &str) -> String {
        let quote = if rng.gen_bool(0.5) { '"' } else { '\'' };
        format!("{}{}{}", quote, input, "") // Missing closing quote
    }

    /// Generate unclosed bracket
    fn unclosed_bracket<R: Rng>(&self, rng: &mut R, input: &str) -> String {
        let brackets = ["(", "[", "{"];
        let bracket = brackets[rng.gen_range(0..brackets.len())];
        format!("{}{}", bracket, input) // Missing closing bracket
    }

    /// Generate mismatched brackets
    fn mismatched_brackets<R: Rng>(&self, rng: &mut R, input: &str) -> String {
        let open_brackets = ["(", "[", "{"];
        let close_brackets = [")", "]", "}"];
        let open = open_brackets[rng.gen_range(0..open_brackets.len())];
        let close = close_brackets[rng.gen_range(0..close_brackets.len())];
        format!("{}{}{}", open, input, close)
    }

    /// Generate invalid escape sequence
    fn invalid_escape<R: Rng>(&self, rng: &mut R, input: &str) -> String {
        let invalid_escapes = ["\\x", "\\u", "\\z", "\\g", "\\q"];
        let escape = invalid_escapes[rng.gen_range(0..invalid_escapes.len())];
        format!("{}{}{}", input, escape, rng.gen_range(0..100))
    }

    /// Generate truncated input
    fn truncated_input(&self, input: &str, intensity: f32) -> String {
        let len = input.len();
        if len == 0 {
            return input.to_string();
        }

        // Higher intensity = more truncation
        let keep_ratio = 1.0 - (intensity * 0.9);
        let keep_len = (len as f32 * keep_ratio) as usize;
        let keep_len = keep_len.max(1);

        // Truncate at random position
        input[..keep_len.min(len)].to_string()
    }

    /// Generate invalid JSON
    fn invalid_json<R: Rng>(&self, rng: &mut R) -> String {
        let invalid_jsons = [
            "{key: 'value'}",           // Missing quotes on key
            "{'key': 'value',}",        // Trailing comma
            "{\"key\": \"value\"}",     // Valid but wrapped
            "{\"key\": }",              // Missing value
            "{: \"value\"}",            // Missing key
            "{\"key\" \"value\"}",      // Missing colon
            "[1, 2, 3,]",              // Trailing comma in array
            "{\"a\": 1, \"a\": 2}",     // Duplicate keys
        ];
        invalid_jsons[rng.gen_range(0..invalid_jsons.len())].to_string()
    }

    /// Generate unbalanced quotes
    fn unbalanced_quotes<R: Rng>(&self, rng: &mut R, input: &str) -> String {
        let quote = if rng.gen_bool(0.5) { '"' } else { '\'' };
        let count = rng.gen_range(1..5);
        let quotes = quote.to_string().repeat(count);
        format!("{}{}{}", quotes, input, "") // Unbalanced
    }
}

impl FuzzStrategy for MalformedInputStrategy {
    fn name(&self) -> &str {
        "malformed_input"
    }

    fn description(&self) -> &str {
        "Generate malformed syntax (unclosed strings, invalid JSON, mismatched brackets, etc.)"
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

        // Choose malformed type
        let malformed_type = rng.gen_range(0..7);

        match malformed_type {
            0 => self.unclosed_string(&mut rng, input),
            1 => self.unclosed_bracket(&mut rng, input),
            2 => self.mismatched_brackets(&mut rng, input),
            3 => self.invalid_escape(&mut rng, input),
            4 => self.truncated_input(input, intensity),
            5 => self.invalid_json(&mut rng),
            _ => self.unbalanced_quotes(&mut rng, input),
        }
    }
}

impl Default for MalformedInputStrategy {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_malformed_zero_intensity() {
        let strategy = MalformedInputStrategy;
        let input = "test string";
        let fuzzed = strategy.fuzz(input, 0.0);

        // Should be unchanged at zero intensity
        assert_eq!(fuzzed, input);
    }

    #[test]
    fn test_malformed_unclosed_string() {
        let strategy = MalformedInputStrategy;
        let input = "test";
        
        // Run multiple times to hit unclosed string case
        let mut found_unclosed = false;
        for _ in 0..30 {
            let fuzzed = strategy.fuzz(input, 0.8);
            // Check for unclosed string pattern
            let quote_count = fuzzed.matches('"').count() + fuzzed.matches('\'').count();
            if quote_count % 2 != 0 {
                found_unclosed = true;
                break;
            }
        }
        
        assert!(found_unclosed, "Should generate unclosed string at some point");
    }

    #[test]
    fn test_malformed_truncation() {
        let strategy = MalformedInputStrategy;
        let input = "this is a long test string that should be truncated";
        let fuzzed = strategy.fuzz(input, 1.0);

        // Truncation should make it shorter (unless it hit another malformed type)
        // We just verify it's different from input
        assert_ne!(fuzzed, input, "High intensity should modify the input");
    }
}
