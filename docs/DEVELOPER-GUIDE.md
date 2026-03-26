# GlassWorm Detection System - Developer Guide

**Version:** v0.66.0-evidence-curated
**Date:** 2026-03-26
**Status:** Production Ready for GlassWorm Detection

---

## Overview

This tool detects **GlassWorm steganographic attacks** in npm packages and GitHub repositories. GlassWorm attacks hide malicious payloads using invisible Unicode characters (ZWSP, ZWNJ, Variation Selectors) combined with decoder functions.

**Key Design Principle:** GlassWorm attacks MUST have invisible Unicode characters. This is the defining characteristic that distinguishes GlassWorm from other supply chain attacks.

---

## Architecture

### Detectors (All Enabled)

| Tier | Detector | Purpose | Signal |
|------|----------|---------|--------|
| L1 | InvisibleCharacter | Detects ZWSP, ZWNJ, VS chars | High |
| L1 | Homoglyph | Detects confusable characters | Medium |
| L1 | BidirectionalOverride | Detects RTL override attacks | Medium |
| L2 | GlasswarePattern | Detects decoder patterns + stego | High |
| L2 | EncryptedPayload | Detects high-entropy blobs | Medium |
| L2 | RDD | Detects URL dependencies | Low |
| L2 | JpdAuthor | Detects PhantomRaven signature | High |
| L3 | LocaleGeofencing | Detects geographic targeting | High |
| L3 | TimeDelaySandboxEvasion | Detects CI/CD bypass | High |
| L3 | BlockchainC2 | Detects known C2 wallets/IPs | Critical |
| L3 | BlockchainPolling | Detects 5-min polling pattern | High |
| L3 | ExfilSchema | Detects exfiltration patterns | High |
| L3 | SocketIOC2 | Detects Socket.IO C2 | Medium |

### Scoring System

**Tiered Scoring (Configurable via TOML):**

```toml
[settings.scoring.tier_config]
mode = "tiered"

# Tier 1: Always runs
[[settings.scoring.tiers]]
tier = 1
detectors = ["invisible_char", "glassware_pattern", "obfuscation", "blockchain_c2"]
threshold = 0.0
weight_multiplier = 1.0

# Tier 2: Only runs if Tier 1 score >= threshold
[[settings.scoring.tiers]]
tier = 2
detectors = ["header_c2", "exfil_schema"]
threshold = 2.0
weight_multiplier = 0.8

# Tier 3: Only runs if Tier 1+2 score >= threshold
[[settings.scoring.tiers]]
tier = 3
detectors = ["locale_geofencing", "time_delay_sandbox_evasion"]
threshold = 10.0
weight_multiplier = 0.8
```

**Category Caps (Prevent False Positives):**
- 1 category: capped at 5.0 (suspicious, not malicious)
- 2 categories: capped at 7.0 (borderline malicious)
- 3 categories: capped at 8.5 (likely malicious)
- 4+ categories: no cap (very likely malicious)

**Malicious Threshold:** 7.0 (configurable)

---

## Evidence Set

**Curated Evidence:** 4 packages (100% detection rate)

| Package | Type | Score | Characteristics |
|---------|------|-------|-----------------|
| iflow-mcp-watercrawl-mcp-1.3.4 | Real Attack | 8.50 | Invisible chars + C2 + obfuscation |
| glassworm-combo-002 | Synthetic | 7.00 | Invisible chars + C2 + evasion |
| glassworm-combo-003 | Synthetic | 7.00 | Invisible chars + C2 |
| glassworm-combo-004 | Synthetic | 7.00 | Invisible chars + C2 + persistence |

**Archived (Non-GlassWorm):** 23 packages in `evidence-archived/`
- react-native-* packages: Obfuscation-only, NO invisible chars
- aifabrix-miso-client: Encrypted payload only, NO invisible chars
- glassworm-c2-*, steg-*, evasion-*, exfil-*: Weak/broken synthetics

---

## Usage

### Quick Start

```bash
# Scan single package
./target/release/glassware scan-npm <package>@<version>

# Scan tarball
./target/release/glassware scan-tarball evidence/<package>.tgz

# Run campaign
./target/release/glassware campaign run campaigns/wave15.toml --llm

# Clear cache (important after code changes!)
./target/release/glassware cache-clear
```

### Campaign Configuration

See `campaigns/wave15.toml` for template. Key sections:

```toml
[settings]
concurrency = 20
rate_limit_npm = 10.0
cache_enabled = true
cache_ttl_days = 7

[settings.scoring]
malicious_threshold = 7.0
suspicious_threshold = 4.0

[settings.scoring.tier_config]
mode = "tiered"
# ... tier definitions ...

[settings.llm]
tier1_enabled = true   # Cerebras for fast triage
tier1_threshold = 6.0  # Only analyze packages scoring >= 6.0
tier2_enabled = true   # NVIDIA for deep analysis
tier2_threshold = 8.0  # Only analyze packages scoring >= 8.0
```

---

## LLM Integration

### Tier 1 (Cerebras - Fast Triage)
- **Purpose:** Quick malicious/benign classification
- **Provider:** Cerebras (llama-3.3-70b)
- **Threshold:** Only analyze packages scoring >= 6.0
- **Rate Limit:** ~100 requests/minute
- **Status:** Enable if API key available, disable if rate limiting

### Tier 2 (NVIDIA - Deep Analysis)
- **Purpose:** Detailed analysis with model fallback
- **Provider:** NVIDIA API
- **Threshold:** Only analyze packages scoring >= 8.0
- **Rate Limit:** ~20 requests/minute
- **Status:** Enable for high-confidence packages only

### Environment Variables

```bash
# Tier 1 LLM (Cerebras)
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="csk-..."
export GLASSWARE_LLM_MODEL="llama-3.3-70b"

# Tier 2 LLM (NVIDIA)
export NVIDIA_API_KEY="nvapi-..."

# GitHub (private repos)
export GITHUB_TOKEN="ghp_..."
```

---

## Cache Management

**IMPORTANT:** Cache can serve stale findings after code changes!

```bash
# Clear all cache
./target/release/glassware cache-clear

# Or manually
rm -f .glassware-orchestrator-cache.db
rm -f .glassware-checkpoints.db
```

**Cache Files:**
- `.glassware-orchestrator-cache.db` - Scan results cache
- `.glassware-checkpoints.db` - Campaign resume data
- `.glassware-llm-cache.json` - LLM response cache

---

## Testing Workflow

### 1. Evidence Validation (Always First)

```bash
# Test on curated evidence
for tgz in evidence/*.tgz; do
    ./target/release/glassware scan-tarball "$tgz"
done

# Expected: 100% detection, all scores >= 7.0
```

### 2. Clean Baseline (FP Check)

```bash
# Test on known clean packages
./target/release/glassware scan-npm express@4.19.2
./target/release/glassware scan-npm lodash@4.17.21
./target/release/glassware scan-npm axios@1.6.7

# Expected: All scores < 5.0, none flagged as malicious
```

### 3. Wave Campaign (Scale Testing)

```bash
# Run wave campaign
./target/release/glassware campaign run campaigns/wave15.toml --llm

# Monitor progress
tail -f logs/wave15-*.log

# Check results
grep "flagged as malicious" logs/wave15-*.log | wc -l
```

### 4. FP Analysis

```bash
# Review flagged packages
grep "flagged as malicious" logs/wave15-*.log

# For each flagged package, verify:
# 1. Has invisible chars? (required for GlassWorm)
# 2. Has decoder patterns? (required for GlassWorm)
# 3. Multiple signal categories? (increases confidence)
```

---

## Known Issues & Limitations

### 1. BlockchainC2 False Positives

**Issue:** Legitimate blockchain SDKs (@solana/web3.js, ethers) trigger BlockchainC2 detector.

**Mitigation:**
- Tiered scoring gates BlockchainC2 behind Tier 1 signals
- Only packages with invisible chars + BlockchainC2 score high
- Known wallet/IP detection still works (critical)

### 2. Obfuscation-Only Attacks

**Issue:** Packages with heavy obfuscation but NO invisible chars score 5.0 (below threshold).

**Design Decision:** This is CORRECT behavior. Obfuscation-only is NOT GlassWorm.

**Examples:**
- react-native-country-select (archived)
- react-native-intl-phone-number (archived)
- aifabrix-miso-client (archived)

### 3. LLM Rate Limiting

**Issue:** Tier 1 LLM (Cerebras) rate limits at ~100 req/min.

**Mitigation:**
- Set `tier1_threshold = 6.0` to only analyze suspicious packages
- Disable Tier 1 if not needed: `tier1_enabled = false`
- Use `--no-llm` flag for initial scans

---

## Development Guidelines

### Adding New Detectors

1. Create detector in `glassware-core/src/detectors/<name>.rs`
2. Implement `Detector` trait
3. Register in `glassware-core/src/engine.rs`
4. Add to tier config in campaign TOML
5. Test on evidence set (must maintain 100% detection)
6. Test on clean baseline (must maintain < 1% FP rate)

### Modifying Scoring

1. **NEVER** create exceptions that bypass category caps
2. **ALWAYS** test on evidence set first
3. **ALWAYS** test on clean baseline
4. Category caps exist to prevent FPs - respect them

### Evidence Management

1. **Only include packages with invisible Unicode characters**
2. Verify each evidence package manually:
   ```bash
   python3 -c "
   content = open('package.js', 'rb').read()
   invisible = [b for i, b in enumerate(content) if b == 0xE2]
   print(f'Found {len(invisible)} potential invisible char sequences')
   "
   ```
3. Archive weak/broken synthetics to `evidence-archived/`
4. Maintain 100% detection rate on evidence set

---

## Troubleshooting

### Package Not Detected (But Should Be)

1. Check for invisible chars:
   ```bash
   python3 -c "
   content = open('file.js', 'rb').read()
   for i, b in enumerate(content):
       if b == 0xE2 and i+2 < len(content):
           cp = int.from_bytes(content[i:i+3], 'big')
           if cp in [0xE2808B, 0xE2808C, 0xE2808D]:
               print(f'Found invisible char at {i}')
   "
   ```

2. If no invisible chars → NOT GlassWorm (by design)
3. If has invisible chars → check decoder patterns

### Too Many False Positives

1. Clear cache: `./target/release/glassware cache-clear`
2. Check tier thresholds (may be too low)
3. Check category caps (should be active)
4. Review flagged packages manually

### LLM Not Working

1. Check environment variables
2. Test API key:
   ```bash
   curl -H "Authorization: Bearer $GLASSWARE_LLM_API_KEY" \
        https://api.cerebras.ai/v1/models
   ```
3. Disable LLM if needed: `--no-llm` or `tier1_enabled = false`

---

## Next Steps / Roadmap

### Immediate
- [ ] Run wave15 (500 packages) validation
- [ ] Measure FP rate on clean baseline (target: < 1%)
- [ ] Maintain 100% evidence detection

### Short-Term
- [ ] Expand evidence set to 10+ confirmed GlassWorm attacks
- [ ] Improve obfuscation detection (without increasing FPs)
- [ ] Add more conditional rules for high-confidence patterns

### Long-Term
- [ ] Support additional attack types (beyond GlassWorm)
- [ ] Add SARIF output for GitHub Advanced Security
- [ ] Integrate with CI/CD pipelines

---

## References

- [GlassWorm Writeup](https://codeberg.org/tip-o-deincognito/glassworm-writeup)
- [Aikido Security Blog](https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode)
- [Sonatype Research](https://www.sonatype.com/blog/glassworm-supply-chain-attack)

---

**Last Updated:** 2026-03-26
**Maintained By:** GlassWorm Detection Team
