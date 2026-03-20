# Adversarial Test Fixtures

This directory contains test fixtures for adversarial testing of glassware detectors.

## Fixture Categories

### Unicode Substitution
- `unicode_substitution.js` - VS-16 → VS-17 substitution
- `zw_substitution.js` - Zero-width character substitution (ZWSP → ZWNJ)
- `bidi_substitution.js` - Bidi mark substitution (LTR → RTL)

### Obfuscation Variation
- `variable_rename.js` - Decoder variable renaming (decoder → d3c0d3r)
- `encoding_variation.js` - Encoding scheme variation (Base64 → Hex)
- `homoglyph_substitution.js` - Homoglyph substitution (Cyrillic → Greek)

### Combined Evasions
- `combined_evasion.js` - Multiple evasion techniques combined
- `stego_variation.js` - Steganographic payload with mixed VS types
- `pipe_delimiter_evasion.js` - Pipe delimiter npm variant with VS-17

## Usage

These fixtures are used by:
1. Unit tests in `test_generator.rs`
2. Integration tests for detector resilience
3. CI/CD adversarial testing workflow

## Adding New Fixtures

When adding a new evasion fixture:
1. Name the file descriptively (e.g., `evasion_type.js`)
2. Include comments documenting:
   - Original payload pattern
   - Mutation applied
   - Expected detection behavior
3. Add to this README
4. Create corresponding test in `test_generator.rs`

## Evasion Severity Levels

Fixtures are categorized by evasion severity:
- **Critical**: Evades all detectors
- **High**: Evades Tier 1-2 detectors (regex + semantic)
- **Medium**: Evades some Tier 3 detectors
- **Low**: Evades single detector

Current fixtures target **Low** to **Medium** severity for regression testing.
