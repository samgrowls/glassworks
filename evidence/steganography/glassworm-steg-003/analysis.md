# Evidence Package: glassworm-steg-003

## Attack Pattern

This package demonstrates **base64-encoded payloads hidden in source code comments** with zero-width Unicode characters interspersed.

### Technique

1. **Base64 encoding** - The wallet address is encoded as base64: `MHg3NDJkMzVDYzY2MzRDMDUzMjkyNWEzYjg0NEJjOWU3NTk1ZjBiRWIx`
2. **Zero-width character insertion** - ZWSP and ZWNJ characters are inserted between base64 characters
3. **Comment hiding** - The encoded payload is placed in JSDoc-style comments
4. **Decoder function** - `extractPayloadFromComments()` scans comments, removes invisible chars, decodes base64

### Why This Works

- Comments are ignored by JavaScript parsers
- Zero-width characters are invisible in most editors
- Base64 looks like random configuration data
- Pattern detection must look inside comments

## GlassWorm Indicators

- **ZWSP in comments** - Zero-width chars inside comment blocks
- **ZWNJ in comments** - Alternating invisible characters
- **Base64 pattern** - Long base64 string in comment
- **Comment scanner** - Function that parses comments for data
- **Hex address pattern** - Decoded value starts with `0x`

## Expected Detection

- **Detector:** Invisible Character Detection (L1)
- **Severity:** High
- **Confidence:** 0.88
- **Category:** steganography/comments

- **Detector:** GlassWare Pattern Detection (L2)
- **Severity:** Critical
- **Confidence:** 0.85
- **Category:** steganography/base64_encoded

- **Detector:** Encrypted Payload Detection (L2)
- **Severity:** Medium
- **Confidence:** 0.70
- **Category:** encoded_data

## Source

Synthetic evidence created for Glassworks testing
Based on: https://codeberg.org/tip-o-deincognito/glassworm-writeup
