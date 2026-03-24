# Evidence Package: glassworm-combo-001

## Attack Pattern

This package demonstrates **combined steganography + blockchain C2**. The C2 wallet address is hidden using steganography, and commands are received via blockchain transactions.

### Attack Chain

**Phase 1: Steganographic C2 Address Retrieval**
- C2 wallet address encoded using ZWSP/ZWNJ binary
- Hidden in source code as `HIDDEN_WALLET`
- Decoded at runtime using `decodeFromBinary()`

**Phase 2: Blockchain C2 Communication**
- Connects to Solana mainnet
- Monitors decoded wallet for transactions
- Extracts commands from memo instructions
- Executes commands: EXFIL, EXEC

### Why Combine These Techniques

- **C2 address hidden** - No hardcoded URLs or addresses
- **Decentralized C2** - No server to take down
- **Two-layer obfuscation** - Must detect both steg and blockchain
- **Resilient** - Survives partial analysis

## GlassWorm Indicators

- **ZWSP/ZWNJ characters** - Steganographic encoding
- **decodeFromBinary function** - Zero-width decoder
- **api.mainnet-beta.solana.com** - Solana RPC
- **getSignaturesForAddress** - Wallet monitoring
- **setInterval with 300000ms** - 5-minute polling
- **innerInstructions parsing** - Command extraction

## Expected Detection

- **Detector:** Invisible Character Detection (L1)
- **Severity:** High
- **Confidence:** 0.90
- **Category:** steganography/invisible_chars

- **Detector:** Blockchain C2 Detection (L3)
- **Severity:** Critical
- **Confidence:** 0.92
- **Category:** blockchain_c2/combined

- **Detector:** GlassWare Pattern Detection (L2)
- **Severity:** Critical
- **Confidence:** 0.88
- **Category:** combined_attack

## Source

Synthetic evidence created for Glassworks testing
Based on: https://codeberg.org/tip-o-deincognito/glassworm-writeup
