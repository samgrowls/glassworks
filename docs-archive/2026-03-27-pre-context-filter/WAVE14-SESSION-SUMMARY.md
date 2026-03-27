# Wave 14 Long-Horizon Testing - Session Summary

**Date:** 2026-03-26
**Status:** IN PROGRESS
**Checkpoint:** v0.61.0-longwave-testing-started

---

## Session Goals

Start long-horizon testing with iterative improvements to detectors and scanning infrastructure.

## Accomplishments

### 1. Cache Issue Resolution ✅
- **Root Cause:** `.glassware-orchestrator-cache.db` was serving stale findings
- **Fix:** Added `glassware cache-clear` command
- **Documentation:** Added cache management section to README.md
- **Tag:** v0.60.0-cache-fix

### 2. GlasswarePattern Detector Improvements ✅
- **Added:** Obfuscation pattern detection (string arrays, XOR, bracket notation)
- **Kept:** Invisible char + decoder detection (GlassWorm steganography)
- **Result:** Catches both steganography AND obfuscation variants

### 3. Campaign Infrastructure ✅
- Created wave13-iterative-test.toml (comprehensive testing framework)
- Created wave13a-quick-test.toml (quick validation)
- Created wave14-long-horizon-starter.toml (123 packages, ~2 min)

### 4. Long-Horizon Testing Started ✅
- Wave14 completed: 123 packages scanned in 140 seconds
- Campaign infrastructure working reliably
- LLM integration functional (Tier 1 - Cerebras)

---

## Current Detection Status

### Evidence Detection
| Package | Status | Notes |
|---------|--------|-------|
| iflow-mcp-watercrawl-mcp | ✅ DETECTED (8.50) | Invisible chars + decoder |
| react-native-country-select | ⚠️ PARTIAL (5.00 avg) | Obfuscation detected, below threshold |
| glassworm-c2-001 to 004 | ✅ DETECTED (9.00) | Real GlassWorm attacks |
| glassworm-combo-001 to 004 | ✅ DETECTED (8.50-9.00) | Real GlassWorm attacks |

### False Positive Analysis
| Package | Score | Cause | Status |
|---------|-------|-------|--------|
| @solana/web3.js | 10.00 | BlockchainC2 (245 findings) | KNOWN ISSUE |
| firebase | 10.00 | BlockchainC2 + HeaderC2 | KNOWN ISSUE |
| prettier | 6.32 | Various | KNOWN ISSUE |
| webpack | 5.62 | Various | KNOWN ISSUE |
| playwright | 7.10 | Various | KNOWN ISSUE |
| pm2 | 9.23 | Various | KNOWN ISSUE |

---

## Key Findings

### 1. BlockchainC2 Detector Too Aggressive
- **Issue:** Legitimate blockchain SDKs flagged (Solana, Firebase)
- **Cause:** Detector flags ANY usage of blockchain APIs
- **Impact:** High FP rate on web3/crypto packages
- **Fix Needed:** Only flag known malicious wallets/IPs, not legitimate SDK usage

### 2. Obfuscation Detection Working But Under-Threshold
- **Issue:** react-native-country-select detected but score too low (5.00 vs 7.0 threshold)
- **Cause:** Obfuscation patterns detected but not weighted heavily enough
- **Impact:** Some real attacks may be missed
- **Fix:** Adjust scoring or lower threshold for obfuscation-only attacks

### 3. Whitelist Not Applied at Scanner Level
- **Issue:** Campaign whitelist only filters which packages to scan, not findings
- **Cause:** Intentional design - all packages evaluated equally
- **Impact:** Can't use whitelist to suppress FPs
- **Fix:** Need to fix detectors, not whitelist

---

## Next Iteration Priorities

### Priority 1: Fix BlockchainC2 Detector
```rust
// Current: Flags any Solana/Firebase usage
// Fix: Only flag known malicious wallets/IPs + suspicious patterns

// Known malicious wallets (CRITICAL - always flag)
const KNOWN_C2_WALLETS = [...];

// Known C2 IPs (CRITICAL - always flag)
const KNOWN_C2_IPS = [...];

// Suspicious patterns (MEDIUM - flag with other indicators)
// - getSignaturesForAddress + setInterval (GlassWorm signature)
// - 5-second polling interval (GlassWorm signature)
```

### Priority 2: Tune Obfuscation Detection
- Lower threshold from 3+ indicators to 2+ indicators ✅ (already done)
- Increase weight for bracket notation + dynamic exec combination
- Add more obfuscation patterns (control flow flattening, etc.)

### Priority 3: Scale Up Testing
- Wave 15: 500 packages (~10 min)
- Wave 16: 1000 packages (~20 min)
- Wave 17: 2000+ packages (~40 min)

---

## Campaign Performance

| Wave | Packages | Duration | Malicious | FPs | Notes |
|------|----------|----------|-----------|-----|-------|
| 13A | 10 | ~5 sec | 0 | 0 | Quick validation |
| 14 | 123 | 140 sec | 9 | ~9 | Long-horizon starter |

**Scan Rate:** ~0.88 packages/second (including download)
**LLM Overhead:** Tier 1 LLM adds ~1-2 sec per flagged package

---

## Files Changed This Session

- `glassware-core/src/detectors/glassware.rs` - Obfuscation detection
- `glassware/src/cli.rs` - cache-clear command
- `glassware/src/main.rs` - cache-clear handler
- `README.md` - Cache management documentation
- `ROOT-CAUSE-ANALYSIS.md` - Cache issue analysis
- `campaigns/wave13*.toml` - Testing campaigns
- `campaigns/wave14*.toml` - Long-horizon starter

---

## Recommendations

### Immediate (Next Session)
1. Fix BlockchainC2 detector to only flag known malicious indicators
2. Test on wave15 (500 packages) with fixed detector
3. Measure FP rate reduction

### Short-Term (This Week)
1. Complete wave16 (1000 packages) testing
2. Validate evidence detection rate (target: 100%)
3. Document detector behavior and tuning guide

### Long-Term (This Month)
1. Scale to 10,000+ package scans
2. Integrate LLM Tier 2 for borderline cases
3. Build automated FP regression testing

---

**Status:** LONG-HORIZON TESTING STARTED
**Next:** Fix BlockchainC2 detector, run wave15 (500 pkg)
**Checkpoint Tag:** v0.61.0-longwave-testing-started
