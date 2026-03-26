# GlassWorm Compilation Issue - ROOT CAUSE ANALYSIS

**Date:** 2026-03-26
**Status:** ✅ RESOLVED
**Root Cause:** Cached findings in `.glassware-orchestrator-cache.db`

---

## Problem Statement

After fixing the GlasswarePattern detector to require BOTH invisible characters AND VS-specific decoder patterns, the output still showed:
- `"GlassWare attack pattern detected: decoder_pattern (confidence: 95%)"`
- `"GlassWare attack pattern detected: encoding_pattern (confidence: 95%)"`

Despite:
- ✅ Source code updated correctly
- ✅ Binary rebuilt (different hash confirmed)
- ✅ Old strings NOT present in new binary (verified via `strings`)
- ✅ New strings present in binary ("GlassWorm steganography detected:")

---

## Investigation Process

### Phase 1: Binary Verification
```bash
# Verified new binary has correct strings
strings target/release/glassware | grep -i "GlassWorm"  # ✅ Found
strings target/release/glassware | grep -i "decoder_pattern"  # ❌ Not found
```

### Phase 2: Source Code Verification
```bash
# Verified source code is correct
grep -rn "decoder_pattern" glassware-core/src/detectors/glassware.rs
# Only found in variable names, NOT in output messages
```

### Phase 3: Cache Discovery
```bash
# Found cache database
ls -la .glassware-orchestrator-cache.db
# File existed with old findings

# Cleared cache
rm -f .glassware-orchestrator-cache.db

# Re-ran scan
./target/release/glassware scan-npm firebase@10.7.2
# ✅ No more "decoder_pattern" messages!
```

---

## Root Cause

The **cache database** (`.glassware-orchestrator-cache.db`) was storing findings from previous scans. When scanning the same package again, the scanner was returning **cached findings** instead of re-scanning.

**Cache Behavior:**
1. Package scanned → findings stored in SQLite cache
2. Same package scanned again → cached findings returned (faster)
3. Code fixed → cache NOT invalidated
4. Result: Old findings served from cache

---

## Resolution

### Immediate Fix
```bash
# Clear the cache database
rm -f .glassware-orchestrator-cache.db
rm -f .glassware-checkpoints.db  # Also clear checkpoint DB if needed
```

### Verification
```bash
# Before cache clear:
./target/release/glassware scan-npm firebase@10.7.2
# Output: "GlassWare attack pattern detected: decoder_pattern" ❌

# After cache clear:
./target/release/glassware scan-npm firebase@10.7.2
# Output: No decoder_pattern messages ✅
```

---

## Current State

### ✅ What's Working
1. **GlasswarePattern detector fixed** - Requires invisible chars + VS-specific decoder
2. **Binary rebuilt correctly** - New strings present, old strings absent
3. **Evidence detection works** - `glassworm-c2-001.tgz` still detected as malicious
4. **Cache cleared** - Fresh scans use new detector logic

### ⚠️ Remaining Issues
1. **Firebase still flagged** - But for DIFFERENT reasons:
   - BlockchainC2 (5-second polling interval)
   - HeaderC2 (HTTP header data extraction)
   - NOT GlasswarePattern anymore!
   
2. **Cache invalidation** - No automatic cache invalidation on code changes

---

## Recommendations

### 1. Add Cache Clear Command
```bash
# Add to CLI
glassware cache clear
glassware cache status
```

### 2. Add Cache Versioning
```rust
// Include detector version in cache key
// Invalidate cache when detector logic changes
struct CacheEntry {
    findings: Vec<Finding>,
    detector_version: String,  // Add version
    timestamp: u64,
}
```

### 3. Document Cache Location
Add to README:
```markdown
## Clearing Cache

If you suspect cached findings are affecting results:
```bash
rm -f .glassware-orchestrator-cache.db
```

Or use the `--no-cache` flag for one-time scans.
```

### 4. Add `--no-cache` Flag
```bash
# Scan without using cache
./target/release/glassware scan-npm firebase@10.7.2 --no-cache
```

---

## Testing Checklist

- [x] Binary rebuilt with correct code
- [x] Old strings not in binary
- [x] New strings in binary
- [x] Cache cleared
- [x] Evidence still detects (glassworm-c2-001.tgz)
- [x] Firebase no longer flagged for GlasswarePattern
- [ ] Test all evidence tarballs (4/4 should detect)
- [ ] Test clean baseline (firebase, web3, prisma should NOT flag for GlasswarePattern)

---

## Files Involved

| File | Purpose | Status |
|------|---------|--------|
| `.glassware-orchestrator-cache.db` | Findings cache | ⚠️ Root cause |
| `glassware-core/src/detectors/glassware.rs` | Detector logic | ✅ Fixed |
| `target/release/glassware` | Binary | ✅ Rebuilt |

---

## Lessons Learned

1. **Always clear cache after code changes** during development
2. **Add cache versioning** to auto-invalidate on detector changes
3. **Document cache location** in README
4. **Provide cache management commands** (clear, status, etc.)
5. **Test with `--no-cache`** when debugging detector issues

---

**Status:** ✅ RESOLVED
**Next Steps:** Run full evidence validation and clean baseline tests
