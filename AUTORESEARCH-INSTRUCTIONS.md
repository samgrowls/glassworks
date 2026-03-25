# Autoresearch Instructions for LLM Agent

**Version:** 1.0
**Date:** March 25, 2026
**Goal:** Minimize false positive rate while maintaining evidence detection

---

## Objective

You are an AI research agent optimizing the Glassworks malware detection scoring system.

**Target Metrics:**
- **False Positive (FP) Rate:** ≤5% (clean packages incorrectly flagged)
- **Detection Rate:** ≥90% (malicious packages correctly caught)
- **F1 Score:** Maximize (balance of precision and recall)

**Current Problem:**
- FP rate is ~100% (ALL clean packages flagged)
- Detection rate is 100% (all evidence caught)
- F1 score is ~0.48 (needs to be ≥0.90)

---

## What You Can Modify

**File:** `glassware/src/scoring.rs`

**Functions you can edit:**
1. `calculate_score()` - Main scoring logic
2. `apply_category_caps()` - Category diversity capping
3. `apply_exceptions()` - Special case handling
4. `calculate_pattern_score()` - Individual pattern scoring

**You can also modify:**
- `glassware/src/scoring_config.rs` - Configuration defaults
- `glassware/src/package_context.rs` - Package reputation logic

---

## What Is FIXED (Do NOT Modify)

**Detectors:** `glassware-core/src/detectors/`
- Invisible character detection
- Homoglyph detection
- Bidirectional text detection
- Blockchain C2 detection
- Time delay detection
- All other detectors

**Why:** Detectors find patterns. The scoring system decides if those patterns indicate malware.

**Evidence packages:** `evidence/`
- Must maintain ≥90% detection on these 23 packages

**Clean packages:** `benchmarks/clean-packages/`
- Must achieve ≤5% FP rate on these 76 packages

---

## Current Scoring Logic (Simplified)

```rust
pub fn calculate_score(&self, findings: &[Finding]) -> f32 {
    // 1. Calculate base score from findings
    let mut base_score = 0.0;
    for finding in findings {
        base_score += severity_weight(finding.severity);  // Critical=3, High=2, Medium=1
    }
    
    // 2. Apply category diversity cap
    base_score = base_score.min(category_cap);  // Currently 5.0-8.5
    
    // 3. Apply reputation multiplier (popular packages get benefit of doubt)
    base_score *= reputation_multiplier;  // Currently 0.3-0.7 for popular
    
    // 4. Apply exceptions (known C2, steganography = high score)
    base_score = apply_exceptions(base_score, findings);
    
    // 5. Clamp to 0-10
    base_score.min(10.0).max(0.0)
}
```

**Current thresholds:**
- `malicious_threshold = 8.0` (packages scoring ≥8 flagged)
- `suspicious_threshold = 4.0` (packages scoring ≥4 suspicious)

---

## Known False Positive Categories

**Popular packages flagged incorrectly:**
1. **UI Frameworks:** react, vue, angular (have Unicode for i18n)
2. **Build Tools:** webpack, vite, rollup (have build scripts)
3. **Blockchain SDKs:** @solana/web3.js, ethers, viem (have blockchain patterns)
4. **Database ORMs:** prisma, mongoose (have telemetry, encryption)
5. **Cloud SDKs:** firebase, aws-sdk (have API calls, encryption)
6. **Monitoring:** newrelic, sentry (have telemetry, HTTP calls)
7. **Utilities:** lodash, moment, async (widely used, should be clean)

**Why they're flagged:**
- Unicode characters in i18n strings → invisible char detector
- Telemetry calls to known endpoints → exfiltration detector
- CI/CD scripts → sandbox evasion detector
- Blockchain SDK patterns → blockchain C2 detector
- Encryption for configs → encrypted payload detector

---

## Strategy Suggestions

**You might try:**

1. **Package reputation:**
   - Popular packages (100K+ downloads/week) get score reduction
   - Verified maintainers get benefit of doubt
   - Old packages (>1 year) less likely to be malicious

2. **Context-aware scoring:**
   - Telemetry to known companies (prisma.io, sentry.io) = legitimate
   - CI detection in /scripts/ or /build/ = legitimate
   - Blockchain in SDK packages = legitimate
   - Encryption for configs = legitimate

3. **Finding count thresholds:**
   - Popular packages with <10 findings = likely FP
   - Packages with 100+ findings = more likely malicious

4. **Category combinations:**
   - Single category (just Unicode) = lower score
   - Multiple categories (Unicode + encryption + HTTP) = higher score

5. **Threshold adjustments:**
   - Lower malicious_threshold (currently 8.0)
   - Adjust reputation multipliers (currently 0.3-0.7)
   - Tune category caps (currently 5.0-8.5)

---

## Experiment Process

**For each iteration:**

1. **Read current state:**
   - Current scoring.rs code
   - Last 5-10 benchmark results
   - Identify patterns (what's working, what's not)

2. **Propose change:**
   - Modify ONE function or logic block
   - Keep changes focused and testable
   - Add comments explaining your reasoning

3. **Build and test:**
   - `cargo build --release`
   - Run benchmark on 50 clean + 23 evidence packages
   - Extract: FP rate, detection rate, F1 score

4. **Evaluate:**
   - If F1 improved AND FP ≤5% → KEEP
   - If F1 worse OR FP >5% → REVERT

5. **Log result:**
   - Record change, metrics, decision
   - Use for next iteration's context

---

## Output Format

**For each code change, provide:**

```rust
// CHANGE: [Brief description, e.g., "Add popularity-based score reduction"]
// HYPOTHESIS: [Why this should help, e.g., "Popular packages are vetted by community"]

// CODE:
[Your modified function(s)]

// EXPECTED IMPACT:
// - FP rate: X% → Y%
// - Detection: X% → Y%
// - F1: X.XX → Y.YY
```

---

## Success Criteria

**Iteration is SUCCESSFUL if:**
- ✅ F1 score improved by ≥0.05
- ✅ FP rate ≤5% (hard constraint)
- ✅ Detection rate ≥90% (hard constraint)
- ✅ Code compiles without errors
- ✅ Code is readable and maintainable

**Stop conditions:**
- 🎯 F1 ≥ 0.90 with FP ≤5% → Goal achieved!
- 📉 No improvement in 20 iterations → Need new strategy
- ⏰ 100 iterations reached → Use best configuration

---

## Current Best Configuration

**As of March 25, 2026:**
- `malicious_threshold = 8.0`
- `suspicious_threshold = 4.0`
- `category_cap_1 = 5.0`
- `reputation_multiplier = 0.3-0.7`

**Results:**
- FP rate: ~100% (ALL clean packages flagged)
- Detection: 100% (all evidence caught)
- F1: 0.48

**Problem:** Thresholds too low OR reputation not strong enough

---

## Example Changes

### Example 1: Stronger Reputation

```rust
// CHANGE: Increase reputation bonus for popular packages
// HYPOTHESIS: Popular packages are community-vetted, less likely malicious

// In package_context.rs:
pub fn reputation_multiplier(&self) -> f32 {
    if self.weekly_downloads > 1_000_000 {
        return 0.1;  // 90% score reduction for ultra-popular
    } else if self.weekly_downloads > 100_000 {
        return 0.3;  // 70% reduction for popular
    } else if self.weekly_downloads > 10_000 {
        return 0.5;  // 50% reduction
    }
    1.0  // No reduction for niche packages
}
```

### Example 2: Context-Aware Telemetry

```rust
// CHANGE: Whitelist known telemetry endpoints
// HYPOTHESIS: Telemetry to known companies is legitimate

// In exfiltration detector or scoring:
const LEGITIMATE_TELEMETRY: &[&str] = &[
    "prisma.io", "sentry.io", "newrelic.com", 
    "datadoghq.com", "segment.com"
];

if findings.iter().any(|f| {
    f.category == DetectionCategory::Exfiltration &&
    LEGITIMATE_TELEMETRY.iter().any(|t| f.description.contains(t))
}) {
    // Reduce severity or skip these findings
    base_score *= 0.5;
}
```

### Example 3: Finding Count Threshold

```rust
// CHANGE: Popular packages with few findings = likely FP
// HYPOTHESIS: Real malware has multiple indicators

if self.package_context.weekly_downloads > 100_000 
   && findings.len() < 5 {
    // Popular package with few findings = likely false positive
    return base_score * 0.3;  // 70% reduction
}
```

---

## Notes

**Be creative!** The examples above are starting points. You might discover:
- Better reputation signals (GitHub stars, maintainer verification)
- Better context detection (file paths, dependency patterns)
- Better scoring logic (non-linear combinations, decision trees)

**Think like a security researcher:** What patterns distinguish real malware from legitimate packages?

**Good luck!** 🚀

---

**Instructions By:** Glassworks Development Team
**Based On:** karpathy/autoresearch approach
**License:** MIT
