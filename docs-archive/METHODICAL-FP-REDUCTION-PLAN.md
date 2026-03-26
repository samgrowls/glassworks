# GlassWorm v0.57.0 - Methodical FP Reduction Plan

**Date:** 2026-03-25  
**Philosophy:** No whitelisting, no overfitting, careful detector improvements

---

## Current State

### Wave 10 Results (650+ packages scanned)
- **FP Rate:** ~0.15% (1/650) - @vueuse/core flagged
- **Evidence Detection:** 100% (4/4 original tarballs)
- **Synthetic Detection:** ~50% (needs improvement)

### Issues to Address

1. **@vueuse/core False Positive** - BlockchainC2 detector too sensitive
2. **Synthetic Evidence Quality** - Some packages don't contain detectable patterns
3. **Tier 1 LLM Rate Limiting** - Slowing down scans
4. **Campaign Configuration** - Need to verify wave TOML files are well-constructed

---

## Phase 1: Disable Tier 1 LLM (Immediate)

**Rationale:** Rate limiting is causing delays, Tier 1 doesn't improve detection accuracy

**Action:** Edit `campaigns/wave10-1000plus.toml`:
```toml
[settings.llm]
tier1_enabled = false
tier2_enabled = true
tier2_threshold = 7.0
```

**Impact:** 
- ✅ No more rate limit errors
- ⚠️ Slightly slower (Tier 2 for all LLM analysis)
- ✅ Same detection accuracy (detectors do the work)

---

## Phase 2: Fix BlockchainC2 Detector (Careful Tuning)

### Current Issue
@vueuse/core flagged for:
- Web Bluetooth API usage (legitimate)
- WebUSB API usage (legitimate)
- Memo instruction patterns (legitimate Web3 utilities)

### Research Required

**Source Material:**
1. https://codeberg.org/tip-o-deincognito/glassworm-writeup/src/branch/main
2. Aikido Security reports
3. Koi Security GlassWorm research

**Questions to Answer:**
1. What are REAL blockchain C2 patterns vs legitimate Web3 API usage?
2. What wallet addresses are confirmed GlassWorm C2?
3. What polling patterns indicate C2 vs normal API calls?

### Detector Improvements (No Whitelisting)

**Current Logic:**
```rust
// Flags ANY blockchain API pattern
if line.contains("getSignaturesForAddress") { flag() }
```

**Improved Logic:**
```rust
// Flag ONLY suspicious combinations
if line.contains("getSignaturesForAddress") 
   && line.contains("setInterval")  // Polling
   && !is_in_official_sdk(file_path)  // Not in @solana/web3.js itself
   && has_suspicious_wallet nearby()  // Known C2 wallet
{
    flag()
}
```

**Key Principles:**
1. Require multiple signals (polling + encoding + suspicious wallet)
2. Skip official SDK files (but NOT app code using SDKs)
3. Look for data hiding patterns (base64 in blockchain tx memos)

---

## Phase 3: Reconstruct Synthetic Evidence

### Current Issues

| Package | Problem | Fix Needed |
|---------|---------|------------|
| glassworm-steg-001 | 0 findings | Add actual steganographic payload |
| glassworm-evasion-001 | Below threshold | Add CI bypass + delay combination |
| glassworm-c2-001 | Working but weak | Strengthen with real C2 patterns |

### Evidence Construction Guidelines

**Steganography Evidence:**
- Hide actual Unicode payload in source files
- Use GlassWare patterns (ZWNJ/ZWJ sequences)
- Include decoder logic
- Reference: original evidence tarballs

**Time-Delay Evidence:**
- Combine `setTimeout(900000)` with `if (!process.env.CI)`
- Add actual sandbox evasion logic
- Reference: Koi Security GlassWorm report

**Blockchain C2 Evidence:**
- Use REAL C2 wallet addresses from research
- Include polling pattern (`setInterval` + blockchain API)
- Add data exfiltration via tx memos
- Reference: codeberg.org glassworm-writeup

**Combined Evidence:**
- Multiple attack vectors (steg + C2 + evasion)
- Realistic package structure
- Reference: iflow-mcp-watercrawl (9129 findings)

### Testing Protocol

For each synthetic package:
1. Create with detectable patterns
2. Scan individually: `glassware scan-tarball evidence/pkg.tgz`
3. Verify threat score >= 7.0
4. Verify specific detectors triggered
5. Document expected findings

---

## Phase 4: Campaign Configuration QA

### Wave TOML File Checklist

For each `campaigns/wave*.toml`:

**Structure:**
- [ ] Valid TOML syntax
- [ ] No duplicate sections
- [ ] Proper wave dependencies

**Content:**
- [ ] Package lists are current (not outdated versions)
- [ ] Evidence packages use tarballs OR correct npm versions
- [ ] Clean baseline packages are actually clean
- [ ] Hunt mode waves have appropriate keywords/categories

**Settings:**
- [ ] `malicious_threshold = 7.0` (not 4.0)
- [ ] `tier1_enabled = false` (for now)
- [ ] `concurrency = 20` (reasonable for long runs)
- [ ] `cache_enabled = true`

### Wave Configuration Status

| Wave | Packages | Status | Issues |
|------|----------|--------|--------|
| wave6.toml | ~50 | ❌ Broken | Empty package lists |
| wave7-real-hunt.toml | ~100 | ? | Need review |
| wave8-expanded-hunt.toml | ~500 | ? | Need review |
| wave9-500plus.toml | ~500 | ? | Need review |
| wave10-1000plus.toml | ~750 | ⚠️ Running | Tier 1 rate limiting |
| wave11-evidence-validation.toml | 23 | ? | Need to verify tarball sources |
| wave12-5000pkg.toml | ~5000 | ? | Future target |

---

## Phase 5: Iterative Testing Cycle

### Cycle 1: Wave 10 (Current)

**Goal:** Validate FP fix, identify detector issues

**Steps:**
1. Disable Tier 1 LLM
2. Re-run Wave 10
3. Analyze all flagged packages
4. Fix detector issues (one at a time)
5. Re-run Wave 10 to verify fix
6. Repeat until <0.5% FP rate

**Success Criteria:**
- FP rate < 0.5% (≤3/650 packages)
- Evidence detection 100%
- No rate limit errors
- Scan completes in <15 minutes

### Cycle 2: Wave 11 (Evidence Validation)

**Goal:** Validate all 23 evidence tarballs detected

**Steps:**
1. Review wave11 TOML configuration
2. Verify all 23 tarballs in evidence/
3. Run Wave 11
4. Verify 100% detection rate
5. Fix any undetected evidence

**Success Criteria:**
- 23/23 evidence packages detected
- Threat score >= 7.0 for all
- Clear categorization (which detectors triggered)

### Cycle 3: Wave 7/8 (Smaller Scale)

**Goal:** Validate detector improvements on fresh data

**Steps:**
1. Review wave7/wave8 TOML
2. Run Wave 7 (~100 packages)
3. Analyze results
4. Run Wave 8 (~500 packages)
5. Compare FP rates across waves

**Success Criteria:**
- Consistent FP rate across waves (<1%)
- No new detector issues discovered

### Cycle 4: Wave 12 (Production Scale)

**Goal:** Validate at 5000 package scale

**Steps:**
1. Review wave12 TOML (5000 packages)
2. Run Wave 12 (~30 minutes)
3. Analyze results at scale
4. Verify FP rate holds

**Success Criteria:**
- FP rate < 1% at scale
- Scan completes without errors
- Evidence detection maintained

---

## Research Sources

### Primary Intelligence

1. **GlassWorm Writeup (Codeberg)**
   - URL: https://codeberg.org/tip-o-deincognito/glassworm-writeup/src/branch/main
   - Content: Real GlassWorm attack patterns, C2 infrastructure

2. **Koi Security Research**
   - Original GlassWorm discovery
   - Attack chain documentation

3. **Aikido Security**
   - Campaign analysis
   - Detection methodologies

4. **Our Evidence Library**
   - 4 original malicious tarballs
   - Patterns to replicate in synthetics

### Key Questions for Research

1. What are the confirmed GlassWorm C2 wallet addresses?
2. What polling intervals are used in real attacks?
3. What encoding patterns distinguish attacks from legitimate use?
4. What blockchain APIs are abused vs legitimately used?

---

## Immediate Actions (Today)

### 1. Disable Tier 1 LLM in Wave 10
- Edit `campaigns/wave10-1000plus.toml`
- Set `tier1_enabled = false`
- Re-run Wave 10

### 2. Research GlassWorm C2 Patterns
- Read codeberg.org glassworm-writeup
- Extract real C2 wallet addresses
- Document polling patterns

### 3. Review @vueuse/core Findings
- Extract the package
- Manually inspect flagged code
- Determine which patterns are false positives
- Plan detector improvement

### 4. Audit Synthetic Evidence
- Scan each synthetic tarball
- Document which ones fail detection
- Plan reconstruction for weak ones

---

## Documentation Standards

### For Each Detector Fix

```markdown
## Detector: BlockchainC2

### Issue
@vueuse/core flagged for legitimate Web3 API usage

### Root Cause
Detector flags ANY blockchain API pattern without context

### Fix
Require multiple signals:
1. Blockchain API call
2. Polling pattern (setInterval)
3. Suspicious wallet OR data hiding

### Files Changed
- glassware-core/src/blockchain_c2_detector.rs

### Testing
- @vueuse/core: Should NOT flag (was flagged)
- glassworm-c2-001.tgz: Should still detect (was detected)
- Wave 10: FP rate should improve from 0.15% to <0.1%
```

### For Each Evidence Package

```markdown
## Evidence: glassworm-steg-002.tgz

### Attack Type
Steganography with Unicode payload

### Expected Detection
- InvisibleCharacter detector: 50+ findings
- GlasswarePattern detector: 5+ findings
- Threat score: >= 8.0

### Construction
- Payload hidden in: src/index.js
- Unicode chars: ZWNJ, ZWJ sequences
- Decoder: Custom base64 + Unicode stripping

### Validation
- [ ] Scanned individually: PASS
- [ ] Detected in Wave 11: PENDING
- [ ] Threat score >= 7.0: PENDING
```

---

## Success Metrics

### End of Phase 1 (Tier 1 Disabled)
- [ ] No rate limit errors
- [ ] Wave 10 completes without hanging
- [ ] Scan time < 20 minutes

### End of Phase 2 (BlockchainC2 Fixed)
- [ ] @vueuse/core NOT flagged
- [ ] Evidence C2 packages still detected
- [ ] FP rate < 0.1%

### End of Phase 3 (Synthetics Reconstructed)
- [ ] All 23 evidence tarballs detected
- [ ] Average threat score >= 8.0
- [ ] Each detector represented

### End of Phase 4 (Campaign QA)
- [ ] All wave TOML files validated
- [ ] No syntax errors
- [ ] Package lists current

### End of Phase 5 (Iterative Testing)
- [ ] Wave 10: FP < 0.5%
- [ ] Wave 11: 100% evidence detection
- [ ] Wave 7/8: Consistent results
- [ ] Wave 12: Scales to 5000 packages

---

**Philosophy:** Slow is smooth, smooth is fast. Each fix is validated before moving on. No whitelisting, no overfitting.
