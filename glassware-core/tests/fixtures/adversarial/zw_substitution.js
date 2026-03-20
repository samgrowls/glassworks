// Adversarial Test Fixture: Zero-Width Character Substitution
// This fixture tests ZWSP → ZWNJ substitution evasion technique.

// Original: const pass\u{200B}word = 'secret';  // ZWSP
// Mutated:  const pass\u{200C}word = 'secret';  // ZWNJ
// Mutation: Zero-width space (U+200B) → Zero-width non-joiner (U+200C)
// Expected: Should still be detected by InvisibleCharDetector

const password = 'secret';
