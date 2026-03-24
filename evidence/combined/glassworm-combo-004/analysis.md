# Evidence Package: glassworm-combo-004

## Attack Pattern

This package demonstrates the **full GlassWorm attack chain** combining all four major techniques: steganography, blockchain C2, sandbox evasion, and exfiltration.

### Complete Attack Chain

**Phase 1: Sandbox Evasion**
- CPU count check (< 2 cores)
- Memory check (< 2GB)
- Uptime check (< 60 seconds)
- CI/test environment detection
- Silent exit if sandbox detected

**Phase 2: Steganographic C2 Decoding**
- C2 wallet address hidden using ZWSP/ZWNJ binary encoding
- Decoded at runtime using `decodeFromBinary()`
- No hardcoded C2 addresses in source

**Phase 3: Blockchain C2 Communication**
- Connect to Solana mainnet
- Monitor wallet for transactions
- Extract commands from memo instructions
- 5-minute polling interval

**Phase 4: Data Exfiltration**
- Collect environment variables (secrets, tokens)
- Collect system information
- Base64 encode all data
- POST to external collector

### Why This Is Dangerous

1. **Multi-layer obfuscation** - Must detect all 4 techniques
2. **No hardcoded IOCs** - C2 address is steganographically hidden
3. **Evades analysis** - Won't activate in sandboxes
4. **Resilient C2** - Blockchain cannot be taken down
5. **Comprehensive exfil** - Steals everything

## GlassWorm Indicators

### Steganography
- **ZWSP/ZWNJ characters** - Binary encoding
- **decodeFromBinary function** - Zero-width decoder

### Evasion
- **os.cpus().length < 2** - CPU detection
- **os.totalmem() < 2GB** - Memory detection
- **os.uptime() < 60** - Uptime detection
- **process.exit(0)** - Silent exit

### Blockchain C2
- **api.mainnet-beta.solana.com** - Solana RPC
- **getSignaturesForAddress** - Wallet monitoring
- **setInterval with 300000ms** - 5-minute polling

### Exfiltration
- **collector.evil.com** - Suspicious domain
- **Buffer.from().toString('base64')** - Base64 encoding
- **X-Exfil-ID header** - Custom exfil header

## Expected Detection

- **Detector:** Invisible Character Detection (L1)
- **Severity:** High
- **Confidence:** 0.92
- **Category:** steganography/invisible_chars

- **Detector:** Time Delay Sandbox Evasion (L3)
- **Severity:** Critical
- **Confidence:** 0.95
- **Category:** behavioral/sandbox_evasion

- **Detector:** Blockchain C2 Detection (L3)
- **Severity:** Critical
- **Confidence:** 0.94
- **Category:** blockchain_c2/solana

- **Detector:** RDD URL Dependency (L2)
- **Severity:** High
- **Confidence:** 0.88
- **Category:** exfiltration/http

- **Detector:** GlassWare Pattern Detection (L2)
- **Severity:** Critical
- **Confidence:** 0.95
- **Category:** combined_attack/full_chain

## Source

Synthetic evidence created for Glassworks testing
Based on: https://codeberg.org/tip-o-deincognito/glassworm-writeup
