# GlassWorm v0.57.0 - Honest Assessment

**Date:** 2026-03-25  
**Status:** CRITICAL DESIGN FLAW IDENTIFIED

---

## The Truth (No Cover Up)

### Wave 10 Results - REAL FP Rate

**Clean Baseline (112 packages):**
- **11 flagged as malicious** (10% FP rate) ❌

**Flagged packages include:**
- firebase@10.7.2 (score 10.00) - Google's official SDK
- @solana/web3.js@1.87.6 (score 10.00) - Official Solana SDK
- @prisma/client@5.8.1 (score 10.00) - Popular ORM
- mongodb, mysql2, typeorm (score 9.00-10.00) - Database drivers
- webpack@5.89.0 (score 5.62) - Standard build tool
- prettier@3.1.1 (score 6.32) - Code formatter

**This is UNACCEPTABLE for a security tool.**

---

## Root Cause: Broken Detector Design

### GlasswarePattern Detector - Fundamentally Flawed

**Current Logic:**
```rust
// Flags these patterns as "GlassWare attack":
decoder_patterns = [
    r"String\.fromCharCode\s*\(",      // Used in 90% of JS code
    r"String\.fromCodePoint\s*\(",     // Standard Unicode API
    r"\.filter\s*\(\s*c\s*=>\s*c\s*!==\s*null\s*\)",  // Common array op
]

encoding_patterns = [
    r"\batob\s*\(",                     // Standard base64 decode
    r"\bbtoa\s*\(",                     // Standard base64 encode
    r"Buffer\.from\s*\([^,]+,\s*base64", // Standard Node.js encoding
]
```

**Problem:** These are STANDARD JavaScript/Node.js APIs used by:
- Firebase SDK ✅
- Solana SDK ✅
- Prisma ORM ✅
- Webpack ✅
- Basically every major library ✅

**Current Detection Logic:**
```rust
// Flag as "GlassWare attack" if 2+ patterns found
if indicators.len() >= 2 {
    findings.push(Critical("GlassWare attack pattern detected"));
}
```

**Result:** Every library using base64 + fromCharCode gets flagged as malicious.

---

## What GlassWorm ACTUALLY Looks Like

Based on real evidence (react-native-country-select, iflow-mcp-watercrawl):

**Real GlassWorm Patterns:**
1. **Invisible Unicode characters** (ZWNJ, ZWJ, VS) embedded in code
2. **Decoder that specifically handles Unicode steganography**:
   - `codePointAt(0xFE00)` - Specific to variation selectors
   - Filtering OUT invisible chars (not just null)
   - Reconstructing hidden payload from VS sequences
3. **Execution of decoded payload** (eval, Function, dynamic require)

**NOT:**
- `atob()` for base64 decoding
- `String.fromCharCode()` for Unicode
- `Buffer.from(x, 'base64')` for encoding

---

## The Fix (Not a Cover Up)

### Option 1: Require Invisible + Decoder Combination

```rust
// Only flag if BOTH present:
let has_invisible = content.chars().any(|c| is_variation_selector(c as u32));
let has_decoder = DECODER_PATTERNS.iter().any(|p| p.is_match(content));

if has_invisible && has_decoder {
    // This is actual GlassWare steganography
    findings.push(Critical("Unicode steganography with decoder"));
}
// Otherwise skip - legitimate encoding usage
```

### Option 2: Much More Specific Patterns

```rust
// Instead of generic fromCharCode, look for:
decoder_patterns = [
    // VS-specific decoding
    r"codePointAt\s*\(\s*0x[Ff][Ee]00\s*\)",  // Specific to FE00
    r"fromCodePoint\s*\([^)]*0x[Ff][Ee]0",    // VS range
    // Filtering invisible chars specifically
    r"\.filter\s*\([^)]*0x200[BC]",  // ZWSP/ZWNJ filtering
    // Payload reconstruction
    r"\.join\s*\(\s*['\"]''['\"]\s*\)",  // Reconstructing hidden string
];
```

### Option 3: Require 3+ Signals Including Invisible

```rust
// Require:
// 1. Invisible characters (VS, ZWNJ, etc.)
// 2. Decoder pattern
// 3. Execution pattern (eval, Function, dynamic require)
// ALL THREE = GlassWare
// Only 1-2 = Skip (legitimate usage)
```

---

## Why This Happened

### Historical Context

1. **Early detection** - Started with specific GlassWorm patterns
2. **Pattern creep** - Added more patterns to catch variants
3. **No validation** - Never tested against major SDKs (firebase, web3, etc.)
4. **Confidence inflation** - 95% confidence on `atob()` is absurd

### What Went Wrong

1. **Tested only on evidence** - 4 malicious tarballs, all detected ✅
2. **Never tested on real SDKs** - firebase, prisma, web3 never scanned
3. **Pattern matching ≠ detection** - Just because pattern exists doesn't mean attack
4. **No negative test set** - No "known clean" packages in testing

---

## Path Forward (Honest)

### Immediate (Today)

1. **DO NOT TAG** - Current state is broken
2. **Disable GlasswarePattern detector** - It's causing more harm than good
3. **Focus on detectors that work:**
   - InvisibleCharacter ✅ (when not over-triggering)
   - BlockchainC2 ✅ (with wallet/IP specificity)
   - TimeDelay ✅ (with CI bypass requirement)

### Short-Term (This Week)

1. **Redesign GlasswarePattern detector:**
   - Require invisible + decoder combination
   - OR use much more specific patterns
   - Test against firebase, web3, prisma BEFORE deploying

2. **Build negative test set:**
   - Top 100 npm packages
   - All must score < 4.0 (not malicious)
   - Run after every detector change

3. **Validate on evidence:**
   - 4 original tarballs must still detect
   - If not, detector is too strict

### Long-Term (This Month)

1. **Tier 2 LLM integration:**
   - Use LLM to review borderline cases (score 5-8)
   - Human review for high-profile packages (firebase, webpack, etc.)

2. **Evidence library expansion:**
   - Need 20+ confirmed malicious packages
   - Variety of attack types
   - Both tarballs and GitHub repos

3. **Continuous testing:**
   - Wave 10 (1000+ packages) after every change
   - FP rate must stay < 1%
   - Evidence detection must stay 100%

---

## Current State (Honest Assessment)

### What Works ✅
- Scanner no longer hangs (large file skip works)
- Evidence detection: 4/4 original tarballs detected
- InvisibleCharacter detector (when not combined with broken GlasswarePattern)
- BlockchainC2 detector (with wallet specificity)

### What's Broken ❌
- GlasswarePattern detector (flagging standard JS APIs)
- FP rate ~10% on clean baseline (unacceptable)
- No validation against major SDKs before deployment
- Overconfidence in pattern matching (95% on `atob()` is absurd)

### What Needs Work ⚠️
- Scoring system (category diversity caps working, but garbage in = garbage out)
- Evidence library (only 4 confirmed malicious)
- Test coverage (no negative test set)

---

## Recommendation

**DO NOT RELEASE v0.57.0 in current state.**

**Reason:** 10% FP rate on legitimate packages makes tool unusable for production.

**Path to Release:**
1. Disable or fundamentally redesign GlasswarePattern detector
2. Build negative test set (Top 100 npm)
3. Validate: FP < 1%, Evidence = 100%
4. THEN consider tag (v0.57.0 or v0.58.0)

**Timeline:** 2-3 days for proper fix and validation.

---

**Status:** CRITICAL DESIGN FLAW IDENTIFIED  
**Honesty Level:** 100% - No cover up, no whitelisting shortcuts  
**Next:** Redesign GlasswarePattern detector or disable it
