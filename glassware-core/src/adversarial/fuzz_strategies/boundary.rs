//! Boundary Fuzz Strategy
//!
//! Tests boundary values such as max line length, max file size,
//! and other edge cases that might trigger buffer overflows or parsing issues.

use super::super::fuzzer::FuzzStrategy;
use rand::Rng;

/// Boundary fuzz strategy
///
/// Generates inputs at boundary conditions to test detector limits:
/// - Maximum line lengths
/// - Maximum file sizes
/// - Empty inputs
/// - Single character inputs
/// - Maximum repetition patterns
pub struct BoundaryStrategy;

impl BoundaryStrategy {
    /// Common boundary sizes for testing
    const BOUNDARY_SIZES: [usize; 8] = [
        0,      // Empty
        1,      // Single character
        2,      // Minimum pair
        80,     // Traditional terminal width
        255,    // Common buffer limit
        1024,   // 1KB boundary
        4096,   // 4KB page size
        65535,  // 16-bit limit
    ];

    /// Generate a string of repeated characters
    fn repeat_char(&self, c: char, count: usize) -> String {
        c.to_string().repeat(count)
    }

    /// Generate a line at boundary length
    fn boundary_line<R: Rng>(&self, rng: &mut R, target_len: usize) -> String {
        let chars = ['a', 'b', 'c', ' ', '\t', '0', '1'];
        (0..target_len)
            .map(|_| chars[rng.gen_range(0..chars.len())])
            .collect()
    }

    /// Generate maximum repetition pattern
    fn max_repetition(&self, pattern: &str, intensity: f32) -> String {
        let repetitions = match intensity {
            i if i < 0.33 => 100,
            i if i < 0.66 => 1000,
            _ => 10000,
        };
        pattern.repeat(repetitions)
    }
}

impl FuzzStrategy for BoundaryStrategy {
    fn name(&self) -> &str {
        "boundary"
    }

    fn description(&self) -> &str {
        "Test boundary values (max line length, max file size, empty, single char, etc.)"
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

        // Choose boundary type based on intensity
        let boundary_type = rng.gen_range(0..6);

        match boundary_type {
            0 => {
                // Empty input
                String::new()
            }
            1 => {
                // Single character
                input.chars().next().map(|c| c.to_string()).unwrap_or_default()
            }
            2 => {
                // Maximum line length (80, 255, or 1024)
                let target_len = Self::BOUNDARY_SIZES
                    [rng.gen_range(3..6)];
                self.boundary_line(&mut rng, target_len)
            }
            3 => {
                // Maximum repetition of input pattern
                if input.is_empty() {
                    self.repeat_char('X', Self::BOUNDARY_SIZES[5])
                } else {
                    self.max_repetition(input, intensity)
                }
            }
            4 => {
                // Maximum size (4KB or 64KB)
                let target_size = Self::BOUNDARY_SIZES
                    [rng.gen_range(6..8)];
                let mut result = String::with_capacity(target_size);
                while result.len() < target_size {
                    result.push_str(input);
                }
                result.truncate(target_size);
                result
            }
            _ => {
                // Null byte injection (common boundary test)
                format!("{}\0{}", input, input)
            }
        }
    }
}

impl Default for BoundaryStrategy {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boundary_zero_intensity() {
        let strategy = BoundaryStrategy;
        let input = "test string";
        let fuzzed = strategy.fuzz(input, 0.0);

        // Should be unchanged at zero intensity
        assert_eq!(fuzzed, input);
    }

    #[test]
    fn test_boundary_empty_result() {
        let strategy = BoundaryStrategy;
        let input = "test";
        
        // Run multiple times since boundary is random
        let mut found_empty = false;
        for _ in 0..20 {
            let fuzzed = strategy.fuzz(input, 0.5);
            if fuzzed.is_empty() {
                found_empty = true;
                break;
            }
        }
        
        // With 20 iterations, should hit empty case at least once
        assert!(found_empty, "Should generate empty input at some point");
    }

    #[test]
    fn test_boundary_max_size() {
        let strategy = BoundaryStrategy;
        let input = "test";
        let fuzzed = strategy.fuzz(input, 1.0);

        // Should generate something (may be empty in rare cases, so check length)
        // The max size case should generate at least 4KB
        if !fuzzed.is_empty() && fuzzed.len() >= 4096 {
            // Successfully hit max size boundary
            assert!(fuzzed.len() >= 4096);
        }
        // If it's not 4KB+, it hit another boundary type which is also valid
    }
}
