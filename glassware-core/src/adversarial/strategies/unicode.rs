//! Unicode Substitution Mutation Strategy
//!
//! Replaces Variation Selectors with equivalent Unicode characters.
//! Example: VS-16 (0xFE0F) → VS-17 (0xFE0E)

use super::super::mutation::MutationStrategy;
use rand::Rng;

/// Unicode substitution mutation strategy
pub struct UnicodeSubstitutionStrategy;

impl MutationStrategy for UnicodeSubstitutionStrategy {
    fn name(&self) -> &str {
        "unicode_substitution"
    }
    
    fn description(&self) -> &str {
        "Replace Variation Selectors with equivalent Unicode (VS-16 → VS-17, etc.)"
    }
    
    fn mutate(&self, payload: &str, rate: f32) -> String {
        use rand::thread_rng;
        let mut rng = thread_rng();
        payload
            .chars()
            .map(|c| {
                if rng.gen::<f32>() < rate {
                    // Substitute Unicode characters based on type
                    match c {
                        // VS-16 (U+FE0F) → VS-17 (U+FE0E)
                        '\u{FE0F}' => '\u{FE0E}',
                        // VS-17 (U+FE0E) → VS-16 (U+FE0F)
                        '\u{FE0E}' => '\u{FE0F}',
                        // Zero-width space (U+200B) → Zero-width non-joiner (U+200C)
                        '\u{200B}' => '\u{200C}',
                        // Zero-width non-joiner (U+200C) → Zero-width joiner (U+200D)
                        '\u{200C}' => '\u{200D}',
                        // Zero-width joiner (U+200D) → Zero-width space (U+200B)
                        '\u{200D}' => '\u{200B}',
                        // Left-to-right mark (U+200E) → Right-to-left mark (U+200F)
                        '\u{200E}' => '\u{200F}',
                        // Right-to-left mark (U+200F) → Left-to-right mark (U+200E)
                        '\u{200F}' => '\u{200E}',
                        // Keep other characters unchanged
                        _ => c,
                    }
                } else {
                    c
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;
    
    #[test]
    fn test_unicode_substitution_vs16() {
        let strategy = UnicodeSubstitutionStrategy;
        let input = "test\u{FE0F}string";  // Contains VS-16
        let mut rng = thread_rng();
        let mutated = strategy.mutate(&input, 1.0);  // 100% rate
        
        // Should contain VS-17 after mutation
        assert!(mutated.contains('\u{FE0E}'));
        // Should not contain original VS-16
        assert!(!mutated.contains('\u{FE0F}'));
    }
    
    #[test]
    fn test_unicode_substitution_zwj() {
        let strategy = UnicodeSubstitutionStrategy;
        let input = "test\u{200D}string";  // Contains ZWJ
        let mut rng = thread_rng();
        let mutated = strategy.mutate(&input, 1.0);  // 100% rate
        
        // Should contain ZWSP after mutation
        assert!(mutated.contains('\u{200B}'));
    }
    
    #[test]
    fn test_unicode_substitution_bidi() {
        let strategy = UnicodeSubstitutionStrategy;
        let input = "test\u{200E}\u{200F}string";  // Contains LTR and RTL marks
        let mut rng = thread_rng();
        let mutated = strategy.mutate(&input, 1.0);  // 100% rate
        
        // Should be swapped
        assert!(mutated.contains('\u{200F}'));
        assert!(mutated.contains('\u{200E}'));
    }
    
    #[test]
    fn test_unicode_substitution_zero_rate() {
        let strategy = UnicodeSubstitutionStrategy;
        let input = "test\u{FE0F}string";
        let mut rng = thread_rng();
        let mutated = strategy.mutate(&input, 0.0);  // 0% rate
        
        // Should be unchanged
        assert_eq!(mutated, input);
    }
}
