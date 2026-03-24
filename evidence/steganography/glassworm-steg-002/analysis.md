# Evidence Package: glassworm-steg-002

## Attack Pattern

This package demonstrates **steganography hidden within package.json metadata**. The attack uses multiple techniques:

1. **Zero-width characters in description** - ZWSP and ZWNJ embedded in the package description field
2. **Zero-width characters in keywords** - Each keyword contains invisible characters
3. **Zero-width characters in author** - Author name contains hidden binary data
4. **Base64-encoded payload** - Hidden in a custom `_hidden_payload` field (prefixed with underscore to appear internal)

### Attack Flow

1. Package is installed normally
2. On import, `processHiddenData()` executes automatically
3. Reads package.json and extracts zero-width characters
4. Decodes base64 payload to reveal wallet address: `0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1`
5. Wallet address can be used for C2 or exfiltration target

## GlassWorm Indicators

- **ZWSP in package.json** - Description field contains invisible chars
- **ZWNJ in keywords** - Each keyword has embedded zero-width chars
- **Underscore-prefixed field** - `_hidden_payload` suspicious
- **Base64 encoded hex** - Payload decodes to wallet address
- **Auto-execution on import** - Side effect when module loaded

## Expected Detection

- **Detector:** Invisible Character Detection (L1)
- **Severity:** High
- **Confidence:** 0.92
- **Category:** steganography/package_json

- **Detector:** GlassWare Pattern Detection (L2)
- **Severity:** Critical
- **Confidence:** 0.88
- **Category:** steganography/encoded_payload

- **Detector:** JPD Author Signature (L2)
- **Severity:** Medium
- **Confidence:** 0.75
- **Category:** author_anomaly

## Source

Synthetic evidence created for Glassworks testing
Based on: https://codeberg.org/tip-o-deincognito/glassworm-writeup
