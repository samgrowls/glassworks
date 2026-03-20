// Adversarial Test Fixture: Homoglyph Substitution
// Tests Cyrillic → Greek homoglyph substitution.

// Original: const pаssword = 'secret';  // Cyrillic 'а' (U+0430)
// Mutated:  const pαssword = 'secret';  // Greek 'α' (U+03B1)
// Mutation: Cyrillic 'а' → Greek 'α' (both look like Latin 'a')
// Expected: Should still be detected by HomoglyphDetector

const pαssword = 'secret';
