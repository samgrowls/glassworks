# Evidence Package: glassworm-steg-001

## Attack Pattern

This package demonstrates **binary steganography using zero-width Unicode characters**. The attack encodes sensitive data (a cryptocurrency wallet address) using:

- **ZWSP (U+200B)** - Zero Width Space representing binary `0`
- **ZWNJ (U+200C)** - Zero Width Non-Joiner representing binary `1`

The encoded payload is embedded within the JavaScript source code as the `hiddenPayload` variable. When executed, the package decodes the hidden wallet address and logs it, simulating the retrieval of exfiltration targets or C2 addresses.

### Encoding Process

1. Convert secret string to binary (8 bits per character)
2. Replace each `0` with ZWSP, each `1` with ZWNJ
3. Store the invisible character sequence in code
4. Decoder reverses the process at runtime

## GlassWorm Indicators

- **ZWSP characters (U+200B)** - Multiple instances in source
- **ZWNJ characters (U+200C)** - Multiple instances in source
- **Binary encoding pattern** - Bit manipulation with zero-width chars
- **Hidden payload variable** - `hiddenPayload` contains encoded data
- **Self-executing decoder** - IIFE that decodes hidden data

## Expected Detection

- **Detector:** Invisible Character Detection (L1)
- **Severity:** High
- **Confidence:** 0.90
- **Category:** steganography/invisible_chars

- **Detector:** GlassWare Pattern Detection (L2)
- **Severity:** Critical
- **Confidence:** 0.85
- **Category:** steganography/binary_encoding

## Source

Synthetic evidence created for Glassworks testing
Based on: https://codeberg.org/tip-o-deincognito/glassworm-writeup
