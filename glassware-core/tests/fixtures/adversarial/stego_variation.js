// Adversarial Test Fixture: Steganographic Payload Evasion
// Tests dense VS codepoint run with variation in selector types.

// Original: Dense run of VS-16 (U+FE0F) encoding hidden data
// Mutated:  Mixed VS-16 and VS-17 to evade density detection
// Mutation: Alternating VS-16/VS-17 pattern
// Expected: Should still be detected by SteganoPayload detector

const hidden\u{FE0F}\u{FE0E}\u{FE0F}\u{FE0E}\u{FE0F}\u{FE0E} = 'data';
