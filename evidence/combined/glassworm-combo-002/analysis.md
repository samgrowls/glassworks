# Evidence Package: glassworm-combo-002

## Attack Pattern

This package demonstrates **combined blockchain C2 + sandbox evasion**. The C2 functionality only activates in real environments, evading automated analysis.

### Attack Chain

**Phase 1: Sandbox Detection**
- CPU count check (< 2 cores = sandbox)
- Memory check (< 2GB = sandbox)
- CI environment variable check
- NODE_ENV check for test/development

**Phase 2: Conditional C2 Activation**
- **In sandbox**: Silent exit after 5 seconds
- **In real environment**: Activate blockchain C2

**Phase 3: Blockchain C2**
- Connect to Solana mainnet
- Monitor wallet for commands
- Execute received commands

### Why Combine These Techniques

- **Evades analysis** - Won't activate in sandboxes
- **Longer dwell time** - Survives initial detection
- **Resilient C2** - Blockchain-based command channel
- **Clean exit** - No errors in sandbox (avoids alerts)

## GlassWorm Indicators

- **os.cpus().length < 2** - CPU-based sandbox detection
- **os.totalmem() < 2GB** - Memory-based sandbox detection
- **process.env.CI** - CI environment check
- **process.exit(0)** - Silent exit
- **api.mainnet-beta.solana.com** - Solana RPC
- **getSignaturesForAddress** - Wallet monitoring
- **setInterval with 300000ms** - 5-minute polling

## Expected Detection

- **Detector:** Time Delay Sandbox Evasion (L3)
- **Severity:** Critical
- **Confidence:** 0.94
- **Category:** behavioral/sandbox_evasion

- **Detector:** Blockchain C2 Detection (L3)
- **Severity:** Critical
- **Confidence:** 0.90
- **Category:** blockchain_c2/solana

- **Detector:** GlassWare Pattern Detection (L2)
- **Severity:** Critical
- **Confidence:** 0.88
- **Category:** combined_attack

## Source

Synthetic evidence created for Glassworks testing
Based on: https://codeberg.org/tip-o-deincognito/glassworm-writeup
