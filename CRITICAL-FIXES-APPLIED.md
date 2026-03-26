# GlassWorm v0.57.0 - Critical Bug Fixes

**Date:** 2026-03-25  
**Status:** CRITICAL FIXES APPLIED - Wave 10 Running

---

## Problems Found & Fixed

### 1. Blockchain Polling Detector - Finding Flooding ❌→✅

**Problem:** `Memo(?:Instruction|Program)?` pattern matched THOUSANDS of times in .d.ts files

**Symptoms:**
- Scanner hung on large packages (typescript, etc.)
- Thousands of "Memo instruction usage" findings
- Wave 10 flagged 10%+ of packages as malicious

**Fix 1:** Limit memo findings to 3 per file
**File:** `glassware-core/src/detectors/blockchain_polling.rs`
```rust
// Track memo findings to avoid flooding (limit to first 3 per file)
let mut memo_findings = 0;
const MAX_MEMO_FINDINGS: usize = 3;

if POLLING_PATTERNS[5].is_match(line) && memo_findings < MAX_MEMO_FINDINGS {
    // ... add finding
    memo_findings += 1;
}
```

**Fix 2:** Skip .d.ts files entirely
```rust
// Skip TypeScript definition files (.d.ts) - they contain type definitions, not C2 logic
if path.ends_with(".d.ts") {
    return findings;
}
```

---

### 2. Large File Scanning - Hanging on Massive Libraries ❌→✅

**Problem:** Scanning 5-9MB library files (typescript.js, tsserver.js, etc.)

**Symptoms:**
- Scanner hung indefinitely on packages with large library files
- TypeScript package took forever (32MB total, 9MB main file)

**Fix:** Skip files >1MB in collector
**File:** `glassware/src/scanner.rs`
```rust
// Skip large files (>1MB) to avoid hanging on massive library files
if let Ok(metadata) = std::fs::metadata(&path) {
    if metadata.len() > 1024 * 1024 { // 1MB
        debug!("Skipping large file: {} ({:.2}MB)", 
            path.display(), 
            metadata.len() as f64 / (1024.0 * 1024.0));
        continue;
    }
}
```

**Result:** TypeScript now scans in 2 seconds (99 files instead of hanging on 3 massive files)

---

### 3. Tier 2 LLM Causing Timeouts ❌→✅

**Problem:** Tier 2 LLM enabled, causing timeouts on flagged packages

**Fix:** Disable Tier 2 LLM
**File:** `campaigns/wave10-1000plus.toml`
```toml
[settings.llm]
tier1_enabled = false  # Rate limiting
tier2_enabled = false  # Causing timeouts - investigate detectors instead
```

---

## Test Results

### Before Fixes
- typescript@5.3.3: **HUNG FOREVER** ❌
- Wave 10: **10%+ FP rate** (typescript, prettier, firebase flagged) ❌
- Scan time: **Infinite** ❌

### After Fixes
- typescript@5.3.3: **2 seconds, 99 files, 0 malicious** ✅
- @types/express: **<1 second, 2 files, 0 malicious** ✅
- chalk: **<1 second, 0 findings** ✅
- Wave 10: **RUNNING** (still some FPs but completing) ✅

---

## Remaining Issues

### False Positives Still Occurring

Some packages still flagged:
- @vueuse/core (7.09) - Web3 utility patterns
- @azure/msal-browser (9.00) - Auth library
- @prisma/client (10.00) - Database ORM
- firebase (10.00) - Firebase SDK
- ant-design-vue (6.48) - UI framework

**These need detector tuning, NOT whitelisting:**
1. BlockchainC2 detector too sensitive to generic Web3 patterns
2. Need to distinguish legitimate SDK usage from actual C2
3. Require multiple signals (wallet + polling + exfil) not just one

---

## Next Steps

1. **Wait for Wave 10 completion** - Verify scan completes without hanging
2. **Analyze FP patterns** - Which detectors are still too sensitive?
3. **Tune detectors** - Not whitelist, but improve detection logic
4. **Create Wave 11** - Evidence validation with tarballs
5. **Test evidence detection** - Verify 4/4 original tarballs detected
6. **Consider tag** - Only after FP rate <1%

---

## Files Modified

1. `glassware-core/src/detectors/blockchain_polling.rs`
   - Added memo finding limit (3 per file)
   - Added .d.ts file skip

2. `glassware/src/scanner.rs`
   - Added large file skip (>1MB)

3. `campaigns/wave10-1000plus.toml`
   - Disabled Tier 2 LLM

---

## Key Learnings

1. **Pattern matching on common terms floods findings** - Always limit per-file
2. **Large library files should be skipped** - They're not where malware hides
3. **TypeScript definitions (.d.ts) are safe** - Skip for blockchain detectors
4. **Test on large packages early** - typescript, @angular/core, etc.
5. **LLM timeouts mask detector bugs** - Fix detectors first, add LLM later

---

**Status:** CRITICAL FIXES COMPLETE - Wave 10 Running  
**Confidence:** HIGH - Scanner no longer hangs  
**Next:** Wait for Wave 10 completion, analyze remaining FPs
