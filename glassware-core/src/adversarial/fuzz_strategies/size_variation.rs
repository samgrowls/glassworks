//! Size Variation Fuzz Strategy
//!
//! Varies payload size from tiny to huge to test detector performance
//! across different input sizes.

use super::super::fuzzer::FuzzStrategy;
use rand::Rng;

/// Size variation fuzz strategy
///
/// Tests detector behavior with inputs of varying sizes:
/// - Tiny (1-10 bytes)
/// - Small (11-100 bytes)
/// - Medium (101-1000 bytes)
/// - Large (1KB-10KB)
/// - Huge (10KB-100KB+)
pub struct SizeVariationStrategy;

impl SizeVariationStrategy {
    /// Size categories in bytes
    const SIZE_TINY: (usize, usize) = (1, 10);
    const SIZE_SMALL: (usize, usize) = (11, 100);
    const SIZE_MEDIUM: (usize, usize) = (101, 1000);
    const SIZE_LARGE: (usize, usize) = (1024, 10240);
    const SIZE_HUGE: (usize, usize) = (10240, 102400);

    /// Generate random string of specific length
    fn random_string<R: Rng>(&self, rng: &mut R, length: usize) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                abcdefghijklmnopqrstuvwxyz\
                                0123456789\n ";
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// Repeat input to reach target size
    fn expand_to_size(&self, input: &str, target_size: usize) -> String {
        if input.is_empty() {
            return self.random_string(&mut rand::thread_rng(), target_size);
        }

        let mut result = String::with_capacity(target_size);
        while result.len() < target_size {
            result.push_str(input);
        }
        result.truncate(target_size);
        result
    }

    /// Get target size based on intensity
    fn get_target_size<R: Rng>(&self, rng: &mut R, intensity: f32) -> usize {
        // Intensity maps to size categories:
        // 0.0-0.2: Tiny
        // 0.2-0.4: Small
        // 0.4-0.6: Medium
        // 0.6-0.8: Large
        // 0.8-1.0: Huge

        let (min, max) = match intensity {
            i if i < 0.2 => Self::SIZE_TINY,
            i if i < 0.4 => Self::SIZE_SMALL,
            i if i < 0.6 => Self::SIZE_MEDIUM,
            i if i < 0.8 => Self::SIZE_LARGE,
            _ => Self::SIZE_HUGE,
        };

        rng.gen_range(min..=max)
    }

    /// Truncate input to target size
    fn truncate_to_size(&self, input: &str, target_size: usize) -> String {
        if input.len() <= target_size {
            return input.to_string();
        }
        input[..target_size].to_string()
    }
}

impl FuzzStrategy for SizeVariationStrategy {
    fn name(&self) -> &str {
        "size_variation"
    }

    fn description(&self) -> &str {
        "Vary payload size from tiny (1-10 bytes) to huge (10KB-100KB+)"
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

        // Get target size based on intensity
        let target_size = self.get_target_size(&mut rng, intensity);

        // Decide whether to expand or truncate based on current size
        if input.len() < target_size {
            // Need to expand
            self.expand_to_size(input, target_size)
        } else if input.len() > target_size {
            // Need to truncate
            self.truncate_to_size(input, target_size)
        } else {
            // Already at target size (rare)
            input.to_string()
        }
    }
}

impl Default for SizeVariationStrategy {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_variation_zero_intensity() {
        let strategy = SizeVariationStrategy;
        let input = "test string";
        let fuzzed = strategy.fuzz(input, 0.0);

        // Should be unchanged at zero intensity
        assert_eq!(fuzzed, input);
    }

    #[test]
    fn test_size_variation_tiny() {
        let strategy = SizeVariationStrategy;
        let input = "test";
        let fuzzed = strategy.fuzz(input, 0.1);

        // Should be in tiny range (1-10 bytes) or expanded from input
        assert!(fuzzed.len() >= 1);
        // May be larger if input was already larger than tiny range
    }

    #[test]
    fn test_size_variation_huge() {
        let strategy = SizeVariationStrategy;
        let input = "test";
        let fuzzed = strategy.fuzz(input, 0.9);

        // Should be in huge range (10KB+)
        assert!(fuzzed.len() >= 10240);
    }

    #[test]
    fn test_size_variation_expansion() {
        let strategy = SizeVariationStrategy;
        let input = "x";
        let fuzzed = strategy.fuzz(input, 0.7);

        // Should be expanded significantly
        assert!(fuzzed.len() > input.len());
        // Should contain only the original character (repeated)
        assert!(fuzzed.chars().all(|c| c == 'x'));
    }
}
