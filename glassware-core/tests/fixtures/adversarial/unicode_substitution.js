// Adversarial Test Fixture: Unicode Substitution Evasion
// This fixture contains a payload with VS-16 that has been substituted to VS-17
// to test detector resilience against Unicode variation substitution.

// Original: const secret\u{FE0F}Key = 'malicious';
// Mutated:  const secret\u{FE0E}Key = 'malicious';
// Mutation: VS-16 (U+FE0F) → VS-17 (U+FE0E)
// Expected: Should still be detected by InvisibleCharDetector

const secretKey = 'malicious';
