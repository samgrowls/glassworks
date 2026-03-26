# Pushed to GitHub - Ready for Expert Review

**Date:** 2026-03-25 21:15 UTC  
**Tag:** `v0.57.0-glassworm-fix-attempt`  
**Branch:** `v0.57.0-glassworm-fix-attempt`

---

## What Was Pushed

### GitHub Repository
- **URL:** https://github.com/samgrowls/glassworks
- **Tag:** `v0.57.0-glassworm-fix-attempt`
- **Branch:** `v0.57.0-glassworm-fix-attempt`

### Commit Contents

**Code Changes:**
1. `glassware-core/src/detectors/glassware.rs`
   - `detect_glassware_patterns()`: Now requires invisible chars + decoder
   - `detect_decoder_functions()`: Now requires invisible chars
   - Removed generic patterns (atob, fromCharCode, Buffer.from)

2. `glassware-core/src/detectors/blockchain_polling.rs`
   - Limited memo findings to 3 per file
   - Skip .d.ts files

3. `glassware/src/scanner.rs`
   - Skip files >1MB

4. `campaigns/wave10-1000plus.toml`
   - Disabled Tier 2 LLM (causing timeouts)

**Documentation:**
- `GLASSWORM-FIX-STATUS.md` - Detailed status and investigation
- `HONEST-ASSESSMENT.md` - Root cause analysis
- `CRITICAL-FIXES-APPLIED.md` - What was fixed

---

## Known Issue (For Expert Review)

**Problem:** Code changes in place but output not reflecting fixes

**Expected Behavior:**
```
[critical] file.js:78 - GlassWorm steganography detected: 15 invisible chars + decoder (confidence: 90%)
```

**Actual Output:**
```
[critical] file.js:78 - GlassWare attack pattern detected: decoder_pattern (confidence: 95%)
```

**Test Case:**
```bash
./target/release/glassware scan-npm firebase@10.7.2
# Should NOT flag firebase (no invisible chars)
# Currently: Flags with 13 critical, 7 high findings
```

**Investigation Results:**
- ✅ Code changes verified in source
- ✅ Binary rebuilt (different hash)
- ✅ Old string NOT in binary (`strings` verified)
- ✅ New string IS in binary
- ❌ Output still shows old format

**Mystery:** Where is "decoder_pattern (confidence: 95%)" coming from?

---

## Questions for Expert

1. **Finding Description Generation**
   - Where is the finding description formatted?
   - Is there post-processing that modifies descriptions?
   - Could there be cached findings?

2. **Code Path Investigation**
   - How to trace which code path actually runs?
   - Are there multiple detectors with similar names?
   - Could there be feature flag issues?

3. **Build System**
   - How to ensure cargo uses fresh .o files?
   - Could there be incremental compilation caching the old code?
   - Should we try `cargo clean` full rebuild?

---

## Commands for Expert

```bash
# Clone the fix attempt branch
git clone -b v0.57.0-glassworm-fix-attempt https://github.com/samgrowls/glassworks.git
cd glassworks

# Build
cargo build --release -p glassware

# Test on firebase (should NOT flag)
./target/release/glassware scan-npm firebase@10.7.2

# Test on evidence (should detect)
./target/release/glassware scan-tarball evidence/react-native-country-select-0.3.91.tgz

# Debug: Add println! to detect_glassware_patterns()
# Line ~441 in glassware-core/src/detectors/glassware.rs
println!("DEBUG: invisible_chars={}", invisible_chars.len());
```

---

## Files to Review

**Primary:**
- `glassware-core/src/detectors/glassware.rs` (lines 421-520)
- `GLASSWORM-FIX-STATUS.md` (detailed investigation)

**Supporting:**
- `HONEST-ASSESSMENT.md` (root cause analysis)
- `CRITICAL-FIXES-APPLIED.md` (what was fixed)

---

## DO NOT MERGE TO MAIN

**Current FP Rate:** ~10% (unacceptable)  
**Target FP Rate:** <1%  
**Status:** BLOCKED pending expert review

---

**Contact:** Session logs available in glassworks-v0.57.0-longwave directory  
**Time Invested:** 4+ hours on this specific issue
