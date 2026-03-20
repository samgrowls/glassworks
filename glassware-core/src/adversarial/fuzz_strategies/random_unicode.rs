//! Random Unicode Fuzz Strategy
//!
//! Inserts random Unicode characters from BMP (Basic Multilingual Plane)
//! and SMP (Supplementary Multilingual Plane) to test detector robustness.

use super::super::fuzzer::FuzzStrategy;
use rand::Rng;
use rand::prelude::SliceRandom;

/// Random Unicode fuzz strategy
///
/// Inserts invisible and special Unicode characters at random positions
/// to test if detectors can handle various Unicode edge cases.
pub struct RandomUnicodeStrategy;

/// BMP invisible characters (U+200B to U+200F)
const BMP_INVISIBLE: [char; 7] = [
    '\u{200B}', // Zero-width space
    '\u{200C}', // Zero-width non-joiner
    '\u{200D}', // Zero-width joiner
    '\u{200E}', // Left-to-right mark
    '\u{200F}', // Right-to-left mark
    '\u{FEFF}', // Byte order mark / Zero-width no-break space
    '\u{00AD}', // Soft hyphen
];

/// Variation selectors (U+FE00 to U+FE0F)
const VARIATION_SELECTORS: [char; 16] = [
    '\u{FE00}', '\u{FE01}', '\u{FE02}', '\u{FE03}',
    '\u{FE04}', '\u{FE05}', '\u{FE06}', '\u{FE07}',
    '\u{FE08}', '\u{FE09}', '\u{FE0A}', '\u{FE0B}',
    '\u{FE0C}', '\u{FE0D}', '\u{FE0E}', '\u{FE0F}',
];

/// SMP invisible characters (U+1D173 to U+1D17A)
const SMP_INVISIBLE: [char; 8] = [
    '\u{1D173}', // Musical combining mark
    '\u{1D174}', '\u{1D175}', '\u{1D176}',
    '\u{1D177}', '\u{1D178}', '\u{1D179}', '\u{1D17A}',
];

impl RandomUnicodeStrategy {
    /// Get a random BMP invisible character
    fn random_bmp_invisible<R: Rng>(&self, rng: &mut R) -> char {
        *BMP_INVISIBLE.choose(rng).unwrap()
    }

    /// Get a random variation selector
    fn random_variation_selector<R: Rng>(&self, rng: &mut R) -> char {
        *VARIATION_SELECTORS.choose(rng).unwrap()
    }

    /// Get a random SMP character
    fn random_smp_invisible<R: Rng>(&self, rng: &mut R) -> char {
        *SMP_INVISIBLE.choose(rng).unwrap()
    }
}

impl FuzzStrategy for RandomUnicodeStrategy {
    fn name(&self) -> &str {
        "random_unicode"
    }

    fn description(&self) -> &str {
        "Insert random Unicode characters from BMP and SMP (U+200B-U+200F, U+FE00-U+FE0F, etc.)"
    }

    fn fuzz(&self, input: &str, intensity: f32) -> String {
        use rand::thread_rng;
        let mut rng = thread_rng();
        let mut result = String::new();

        // Clamp intensity to [0.0, 1.0]
        let intensity = intensity.clamp(0.0, 1.0);

        // Calculate number of insertions based on intensity and input length
        let num_insertions = ((input.len() as f32) * intensity) as usize;

        // Collect insertion positions
        let mut positions: Vec<usize> = (0..=input.len()).collect();
        positions.shuffle(&mut rng);
        positions.truncate(num_insertions);
        positions.sort();

        let mut last_pos = 0;
        for &pos in &positions {
            // Add original content up to this position
            if pos > last_pos {
                result.push_str(&input[last_pos..pos]);
            }

            // Insert random Unicode character
            let unicode_char = match rng.gen_range(0..3) {
                0 => self.random_bmp_invisible(&mut rng),
                1 => self.random_variation_selector(&mut rng),
                _ => self.random_smp_invisible(&mut rng),
            };
            result.push(unicode_char);

            last_pos = pos;
        }

        // Add remaining content
        if last_pos < input.len() {
            result.push_str(&input[last_pos..]);
        }

        result
    }
}

impl Default for RandomUnicodeStrategy {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_unicode_zero_intensity() {
        let strategy = RandomUnicodeStrategy;
        let input = "test string";
        let fuzzed = strategy.fuzz(input, 0.0);

        // Should be unchanged at zero intensity
        assert_eq!(fuzzed, input);
    }

    #[test]
    fn test_random_unicode_high_intensity() {
        let strategy = RandomUnicodeStrategy;
        let input = "test";
        let fuzzed = strategy.fuzz(input, 1.0);

        // Should be longer than input (contains inserted characters)
        assert!(fuzzed.len() > input.len());

        // Should still contain original characters
        for c in input.chars() {
            assert!(fuzzed.contains(c));
        }
    }

    #[test]
    fn test_random_unicode_contains_invisible_chars() {
        let strategy = RandomUnicodeStrategy;
        let input = "test";
        let fuzzed = strategy.fuzz(input, 0.8);

        // Should contain at least one invisible character
        let has_invisible = fuzzed.chars().any(|c| {
            matches!(c,
                '\u{200B}'..='\u{200F}' |
                '\u{FE00}'..='\u{FE0F}' |
                '\u{1D173}'..='\u{1D17A}' |
                '\u{FEFF}' | '\u{00AD}'
            )
        });
        assert!(has_invisible);
    }
}
