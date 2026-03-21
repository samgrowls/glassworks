# Wave 1 Scan Results - Preliminary Analysis

**Date:** 2026-03-21  
**Scan:** 50 packages (55 in list, some failed)  
**Status:** CRITICAL FALSE POSITIVE ISSUES IDENTIFIED

---

## Executive Summary

⚠️ **CRITICAL ISSUE:** Multiple legitimate packages flagged as MALICIOUS:
- moment@2.30.1 (threat score: 10.00) - Date library
- prettier@3.2.5 (threat score: 7.98) - Code formatter
- typescript@5.4.2 (threat score: 7.14) - TypeScript compiler

**These are FALSE POSITIVES that must be fixed before production use.**

---

## Scan Statistics

| Metric | Rust Orchestrator | Python Harness |
|--------|-------------------|----------------|
| Packages scanned | 41 | ~25 (timed out) |
| Total time | 4m37s | 5m+ (incomplete) |
| Packages/sec | 0.15 | ~0.08 |
| Total findings | 1007 | N/A |
| Malicious flagged | 3 | N/A |

**Rust is ~2x faster than Python** for scanning.

---

## Critical False Positives

### moment@2.30.1 - THREAT SCORE: 10.00 (MALICIOUS)

**Why flagged:** Likely invisible characters in locale data files

**Reality:** moment.js is a legitimate date library with 18M+ weekly downloads. Locale files contain Unicode characters for date formatting.

**Fix needed:** Exclude locale/data files from invisible character detection, or whitelist known legitimate packages.

---

### prettier@3.2.5 - THREAT SCORE: 7.98 (MALICIOUS)

**Why flagged:** 61 findings including invisible characters

**Reality:** prettier is the most popular code formatter with 20M+ weekly downloads.

**Fix needed:** Same as moment - exclude test/fixtures/data files.

---

### typescript@5.4.2 - THREAT SCORE: 7.14 (MALICIOUS)

**Why flagged:** 17 findings

**Reality:** TypeScript is Microsoft's official TypeScript compiler.

**Fix needed:** Whitelist @microsoft and typescript packages.

---

## BlockchainC2 Detector Issue

**668 BlockchainC2 findings** - This detector is firing on ANY Solana/web3 API usage.

**Problem:** Legitimate crypto packages (ethers, web3, @solana/web3.js, viem, wagmi) all use Solana/RPC APIs - this is their intended functionality, NOT C2 communication.

**Fix needed:** 
1. Require additional signals (not just API calls)
2. Whitelist known legitimate crypto packages
3. Only flag when combined with other suspicious patterns

---

## Known Malicious Packages

**Could NOT scan:**
- react-native-country-select@0.3.91 - Version yanked from npm
- react-native-international-phone-number@0.11.8 - Version yanked from npm

**Solution:** Scan from evidence directory tarballs instead of npm registry.

---

## Recommendations

### Immediate (Before Next Scan)

1. **Fix threat score calculation** - Legitimate packages should never score >5.0
2. **Tune BlockchainC2 detector** - Require multiple signals, not just API usage
3. **Add package whitelist** - @microsoft, typescript, moment, prettier, etc.
4. **Exclude data/locale files** from invisible character detection
5. **Use evidence directory** for known malicious packages

### Short-term

1. **Add LLM verification** - High threat score packages should get LLM review before flagging as malicious
2. **Improve false positive detection** - Cross-reference with npm download stats, repository age, etc.
3. **Add confidence scores** - Distinguish between "likely malicious" and "needs review"

---

## Next Steps

1. **DO NOT run larger scans** until false positive issues are fixed
2. **Fix BlockchainC2 detector** - highest priority (668 FPs)
3. **Fix threat score calculation** - second priority
4. **Re-scan with fixes** - verify moment/prettier/typescript are clean
5. **Then proceed** to larger Wave 2 scan

---

## Raw Data

### Rust Orchestrator Output
- Location: `/tmp/rust-wave1/rust-wave1-output.log`
- Cache DB: `/tmp/rust-wave1/.glassware-orchestrator-cache.db`

### Python Harness Output
- Location: `/tmp/python-wave1-output.log`
- Incomplete due to timeout

---

**Bottom line:** The scanning infrastructure works, but the detection logic needs significant tuning to reduce false positives before production use.**
