# GlassWorm Detector Fix - Status Report

**Date:** 2026-03-25 21:10 UTC  
**Status:** PARTIAL - Code fixed but not working

---

## What Was Fixed

### 1. `detect_glassware_patterns()` - ✅ Code Updated

**File:** `glassware-core/src/detectors/glassware.rs`

**Changes:**
- Now requires BOTH invisible chars AND decoder patterns (line 441-443)
- Removed generic patterns (atob, btoa, Buffer.from)
- Only flags VS-specific decoders (0xFE00, 0xE0100)
- Confidence calculation based on combination

**Code:**
```rust
// STEP 1: Check for invisible Unicode characters (REQUIRED)
let invisible_chars = Self::find_invisible_unicode(content);
if invisible_chars.is_empty() {
    return findings;  // No invisible chars = NOT GlassWorm
}

// STEP 2: Check for GlassWorm-specific decoder patterns
// Only flags VS-specific patterns, not generic fromCharCode
```

### 2. `detect_decoder_functions()` - ✅ Code Updated

**File:** `glassware-core/src/detectors/glassware.rs` (line 354-400)

**Changes:**
- Now requires invisible chars OR VS-specific decoder
- Won't flag legitimate Unicode handling code

**Code:**
```rust
// First check if file has invisible Unicode chars (required for GlassWorm)
let has_invisible = Self::find_invisible_unicode(content).len() > 0;
if !has_invisible {
    return findings;  // No invisible chars = NOT GlassWorm
}
```

### 3. Decoder Patterns - ✅ Updated

**File:** `glassware-core/src/detectors/glassware.rs` (line 41-52)

**Changes:**
- Removed: `String.fromCharCode`, `String.fromCodePoint` (generic)
- Removed: `.filter(c => c !== null)` (generic)
- Kept: VS-specific patterns only (0xFE00, 0xE0100)

---

## What's NOT Working

### Output Still Shows Old Format

**Expected:**
```
[critical] file.js:78 - GlassWorm steganography detected: 15 invisible chars + decoder (confidence: 90%)
```

**Actual:**
```
[critical] file.js:78 - GlassWare attack pattern detected: decoder_pattern (confidence: 95%)
```

### Firebase Still Flagged

**Expected:** Firebase should NOT be flagged (no invisible chars)  
**Actual:** Firebase flagged with 13 critical, 7 high findings

---

## Investigation Results

### Binary Verification

✅ New binary built (different hash: `a795199a2ed84cd0294c9c525ac9cfb8`)  
✅ New string in binary: "GlassWorm steganography detected:"  
✅ Old string NOT in binary: "GlassWare attack pattern detected: decoder_pattern"

### Source Verification

✅ Changes in source file (line 421-520)  
✅ `detect_decoder_functions` updated (line 354-400)  
✅ Patterns updated (line 41-52)

### Build Verification

✅ `cargo clean -p glassware-core` run  
✅ File touched to force recompile  
✅ Binary rebuilt (1m 41s compile time)

---

## Mystery

**The output shows a string that:**
1. Does NOT exist in source code
2. Does NOT exist in binary (verified with `strings`)
3. Does NOT exist in library .rlib files

**But the output STILL shows it.**

---

## Possible Causes

1. **Cached findings** - Maybe findings are cached somewhere and being reused?
2. **Dynamic output formatting** - Maybe the description is being generated from category + pattern name?
3. **Multiple code paths** - Maybe there's another detector outputting this?
4. **Build system issue** - Maybe cargo is using a cached .o file?

---

## Next Steps (For Fresh Eyes)

### 1. Verify Which Code Is Running

```bash
# Add debug output to detect_glassware_patterns
println!("DEBUG: detect_glassware_patterns called, invisible_chars={}", invisible_chars.len());

# Rebuild and check if debug output appears
```

### 2. Check All Detectors

Search for ALL places that output "decoder_pattern":
```bash
grep -r "decoder_pattern" glassware-core/src/
grep -r "encoding_pattern" glassware-core/src/
```

### 3. Check Finding Post-Processing

Maybe findings are being modified after detection:
```bash
grep -r "finding.description" glassware/src/
grep -r "Finding {" glassware-core/src/ | grep -v "test"
```

### 4. Try Different Approach

Instead of modifying existing detector, create a NEW detector `glassworm_stego.rs` with the correct logic and disable the old one.

---

## Current Recommendation

**DO NOT RELEASE** until this is fixed. The detector is still flagging legitimate packages (firebase, web3, prisma) at an unacceptable rate.

**Immediate Actions:**
1. Debug which code path is actually running
2. Find where "decoder_pattern" output is generated
3. Fix or disable that code path
4. Test on firebase, web3, prisma - should NOT flag
5. Test on evidence - should still detect

**Alternative:** Disable `GlasswarePattern` detector entirely until fixed, rely on other detectors (InvisibleCharacter, BlockchainC2 with wallet specificity).

---

**Status:** BLOCKED - Need fresh investigation  
**Priority:** CRITICAL - Blocks release  
**Time Spent:** 4+ hours on this specific issue
