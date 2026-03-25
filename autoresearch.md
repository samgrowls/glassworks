# Autoresearch: GlassWorm False Positive Rate Tuning

## Objective

Reduce false positive rate in GlassWorm steganography detector from 10% to <5% while maintaining 100% evidence detection rate. The detector incorrectly flags legitimate Unicode characters in i18n/locale data as malicious steganographic payloads.

## Metrics

- **Primary**: `combined_score` (0-1 scale, higher is better) — optimization target
  - Formula: `combined_score = (evidence_rate × 0.6) + ((100 - fp_rate) × 0.4)`
- **Secondary**: 
  - `evidence_detection_rate` (0-1 scale, higher is better) — must maintain ≥1.0
  - `fp_rate` (0-1 scale, lower is better) — target <0.05

## How to Run

```bash
./benchmarks/fp-success-benchmark.sh --quick
```

The script outputs JSON with metrics:
```json
{
  "evidence_detection_rate": 1.00,
  "fp_rate": 0.10,
  "combined_score": 0.96
}
```

## Current Baseline

| Metric | Value | Target |
|--------|-------|--------|
| Evidence Detection Rate | 100% (23/23) | ≥100% |
| False Positive Rate | 10% (1/10) | <5% |
| Combined Score | 0.96 | >0.98 |

## Root Cause Analysis

**FP Source:** `moment@2.30.1` flagged as malicious (threat score: 7.00)

**Why:** InvisibleCharacter detector finding zero-width characters in locale data:
- ZWNJ (U+200C) - Zero Width Non-Joiner
- ZWJ (U+200D) - Zero Width Joiner

**Context:** These are LEGITIMATE Unicode characters used for Persian/Arabic script rendering in moment.js locale files. They are NOT steganographic payloads.

**Detection Pattern:**
- moment.js contains extensive locale data for internationalization
- Persian/Arabic locales use ZWNJ/ZWJ for proper script rendering
- Current detector flags ALL zero-width characters without context awareness

## Files in Scope

| File | Purpose | What to Modify |
|------|---------|----------------|
| `glassware-core/src/invisible.rs` | InvisibleCharacter detector | Add file type/path awareness to skip i18n data files |
| `glassware-core/src/scanner.rs` | Scoring engine | Adjust category diversity caps if needed |
| `glassware/src/scanner.rs` | Threat score calculation | Tune scoring thresholds |

## Off Limits

- `benchmarks/fp-success-benchmark.sh` - Do not modify the benchmark
- Evidence packages in `evidence/` directory
- Clean package list in benchmark script
- Other detectors (glassware_pattern, blockchain_c2, time_delay, etc.) unless FP persists after InvisibleCharacter fixes

## Constraints

1. **Must maintain 100% evidence detection** - All 23 malicious packages must remain detected
2. **Benchmark must complete in <3 minutes** - Keep changes lightweight
3. **No new dependencies** - Use only existing Rust standard library and project dependencies
4. **Minimal changes** - Focus on targeted fixes, not rewrites
5. **All tests must pass** - Run `cargo test` after changes

## Evidence Packages (Must All Remain Detected)

### Real Malicious Tarballs (4)
- `react-native-country-select-0.3.91.tgz` - GlassWare steganography attack
- `react-native-international-phone-number-0.11.8.tgz` - GlassWare steganography attack
- `iflow-mcp-watercrawl-watercrawl-mcp-1.3.4.tgz` - Bundled steganographic payload
- `aifabrix-miso-client-4.7.2.tgz` - Bundled steganographic payload

### Synthetic Evidence (19)
- `glassworm-c2-001.tgz` through `glassworm-c2-004.tgz` - Blockchain C2 patterns
- `glassworm-combo-001.tgz` through `glassworm-combo-004.tgz` - Combined attack patterns
- `glassworm-exfil-001.tgz` through `glassworm-exfil-004.tgz` - Data exfiltration patterns
- `glassworm-steg-001.tgz` through `glassworm-steg-004.tgz` - Steganography patterns
- `glassworm-evasion-001.tgz` through `glassworm-evasion-003.tgz` - Time delay evasion

## Preferred Fix Approaches (Try in Order)

### Approach 1: File Extension Filtering
Skip `.json` files in InvisibleCharacter detector - i18n data is typically in JSON format.

```rust
// In invisible.rs detect() function
if file_path.ends_with(".json") {
    return Vec::new(); // Skip JSON files
}
```

### Approach 2: Path-Based Filtering
Skip files in i18n/locale directories:
- `/locale/`, `/locales/`, `/i18n/`, `/lang/`, `/languages/`
- Files containing `moment` in path (moment.js locale data)

```rust
let path_lower = file_path.to_lowercase();
if path_lower.contains("/locale/") || path_lower.contains("/i18n/") {
    return Vec::new();
}
```

### Approach 3: Unicode Density Threshold
Calculate ratio of invisible chars to total chars. Skip files with high density (likely i18n data) vs concentrated patterns (likely steganography).

```rust
let density = invisible_char_count as f64 / total_char_count as f64;
if density > 0.01 { // More than 1% invisible chars = likely i18n
    return Vec::new();
}
```

### Approach 4: Context-Aware Detection
Check if invisible chars appear in string literals vs code. I18n data uses them in strings; steganography hides them in code.

### Approach 5: Score Adjustment
Require multiple attack categories before flagging as malicious. Single-category detections (only InvisibleCharacter) should cap at "suspicious" not "malicious".

## What's Been Tried

Experiment 1: Fix file path propagation (SUCCESS) - FP 10%->0%, Score 0.96->1.00. Root cause: scanner passed directory path instead of file path to engine.scan(). Fixed glassware/src/scanner.rs to pass &entry_path. Added i18n skip in homoglyph.rs.

## Session Controls

- **Interrupt:** Press Escape to stop loop and request summary
- **Stop:** `/autoresearch off` - Stop auto-resume, keep logs
- **Clear:** `/autoresearch clear` - Delete logs and reset
- **Resume:** `/autoresearch continue` - Resume session
- **Dashboard:** Ctrl+X - Toggle inline results table
- **Fullscreen:** Ctrl+Shift+X - Fullscreen overlay with spinner

## Configuration

```json
{
  "workingDir": "/home/shva/samgrowls/glassworks-v0.41",
  "maxIterations": 30,
  "checks_timeout_seconds": 600,
  "benchmark_script": "./benchmarks/fp-success-benchmark.sh --quick",
  "optimization_goal": "maximize",
  "metric": "combined_score"
}
```
