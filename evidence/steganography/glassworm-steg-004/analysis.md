# Evidence Package: glassworm-steg-004

## Attack Pattern

This package demonstrates **combined steganographic techniques** - using multiple hiding methods simultaneously to increase detection evasion and payload redundancy.

### Combined Techniques

1. **Package.json zero-width embedding**
   - ZWSP/ZWNJ in description field
   - ZWSP/ZWNJ in keywords array
   - ZWSP/ZWNJ in author field

2. **Hidden payload field**
   - `_steg_payload` contains base64-encoded wallet address
   - Underscore prefix makes it appear as internal metadata

3. **Comment-based base64 with zero-width**
   - Base64 payload in JSDoc comment
   - Zero-width characters interspersed in base64

4. **Binary encoding functions**
   - Full encode/decode pipeline in source
   - Multiple extraction methods

### Attack Flow

1. Package imported
2. `extractAllPayloads()` called automatically
3. Tries 3 extraction methods:
   - Hidden field (base64 decode)
   - Zero-width in package.json (binary decode)
   - Comments (base64 + zero-width strip)
4. All methods return same wallet address
5. Redundancy ensures payload survives partial cleaning

## GlassWorm Indicators

- **Multiple ZWSP instances** - Throughout package.json and source
- **Multiple ZWNJ instances** - Throughout package.json and source
- **Underscore-prefixed field** - `_steg_payload`
- **Base64 in comments** - Long base64 string
- **Binary encoding functions** - encodeToBinary/decodeFromBinary
- **Multiple extraction methods** - Redundant payload retrieval

## Expected Detection

- **Detector:** Invisible Character Detection (L1)
- **Severity:** Critical
- **Confidence:** 0.95
- **Category:** steganography/multiple_vectors

- **Detector:** GlassWare Pattern Detection (L2)
- **Severity:** Critical
- **Confidence:** 0.92
- **Category:** steganography/combined

- **Detector:** Encrypted Payload Detection (L2)
- **Severity:** High
- **Confidence:** 0.85
- **Category:** encoded_payload

## Source

Synthetic evidence created for Glassworks testing
Based on: https://codeberg.org/tip-o-deincognito/glassworm-writeup
