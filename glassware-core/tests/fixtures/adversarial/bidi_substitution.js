// Adversarial Test Fixture: Bidi Mark Substitution
// Tests LTR → RTL mark substitution evasion.

// Original: const file = "test\u{200E}exe";  // LTR mark
// Mutated:  const file = "test\u{200F}exe";  // RTL mark
// Mutation: Left-to-right mark (U+200E) → Right-to-left mark (U+200F)
// Expected: Should still be detected by BidiDetector

const file = "testexe";
