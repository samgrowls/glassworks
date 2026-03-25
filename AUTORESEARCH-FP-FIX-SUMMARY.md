# Autoresearch Session: FP Rate Tuning - Final Summary

## Objective
Reduce false positive rate in GlassWorm steganography detector from 10% to <5% while maintaining 100% evidence detection rate.

## Results Achieved
| Metric | Before | After | Target |
|--------|--------|-------|--------|
| False Positive Rate | 10% (1/10) | 0% (0/10) | <5% ✅ |
| Evidence Detection | 100% (23/23) | 100% (23/23) | ≥100% ✅ |
| Combined Score | 0.96 | 1.00 | >0.98 ✅ |

## Root Cause Analysis

### The Problem
`moment@2.30.1` was being flagged as malicious (threat score: 7.00) due to:
1. **InvisibleCharacter detector** finding zero-width characters (ZWNJ/ZWJ) in Persian/Arabic locale files
2. **Homoglyph detector** finding Cyrillic characters in Macedonian locale files

These are LEGITIMATE Unicode characters used for internationalization, NOT steganographic payloads.

### The Bug
In `glassware/src/scanner.rs`, the scan loop was passing the **directory path** instead of the **file path** to detectors:

```rust
// BEFORE (broken):
let findings = engine.scan(path, &content);  // path = /tmp/.tmpXYZ (directory)

// AFTER (fixed):
let findings = engine.scan(&entry_path, &content);  // entry_path = /tmp/.tmpXYZ/locale/si.js (file)
```

This broke path-based heuristics in detectors that skip i18n files (e.g., `/locale/`, `/i18n/` patterns).

## Changes Made

### 1. `glassware/src/scanner.rs`
- Line 268: Changed `engine.scan(path, &content)` → `engine.scan(&entry_path, &content)`
- Line 326: Same change for `scan_directory_for_tarball()`

### 2. `glassware-core/src/detectors/homoglyph.rs`
- Added i18n path skip at the start of `detect_impl()`:
```rust
let i18n_paths = ["/locale/", "/locales/", "/i18n/", "/lang/", "/languages/", "/nls/", "/translation/", "/translations/"];
if i18n_paths.iter().any(|dir| path_lower.contains(dir)) {
    return findings; // Skip i18n files entirely
}
```

## Verification

### Clean Packages (All Safe)
| Package | Threat Score |
|---------|-------------|
| express@4.19.2 | 3.39 |
| lodash@4.17.21 | 1.00 |
| axios@1.6.7 | 0.33 |
| chalk@5.3.0 | 0.00 |
| debug@4.3.4 | 0.00 |
| moment@2.30.1 | 3.32 (was 7.00) |
| uuid@9.0.1 | 0.00 |
| async@3.2.5 | 0.00 |
| glob@10.3.10 | 0.00 |
| ws@8.16.0 | 0.00 |

### Evidence Packages (All Detected)
All 23 evidence packages remain correctly flagged with threat scores ≥7.0.

## Session Statistics
- Total experiments: 3
- Baseline score: 0.96
- Final score: 1.00
- Improvement: +4.2%

## Files Modified
1. `glassware/src/scanner.rs` - Fixed file path propagation
2. `glassware-core/src/detectors/homoglyph.rs` - Added i18n path skip

## Tag
`v0.41.1-fp-fix`
