# Evidence Scan Results & Long-Running Campaign Design

**Date:** March 23, 2026
**Status:** Active - Wave 8 Running (74 packages)

---

## Evidence Archive Scan Results

### Known Malicious Packages Tested

| Package | Version | Findings | Threat Score | Flagged | Notes |
|---------|---------|----------|--------------|---------|-------|
| react-native-country-select | 0.3.91 | 11 | 4.00 | ❌ No | Below threshold (7.0) |
| react-native-international-phone-number | 0.11.8 | 10 | 10.00 | ✅ Yes | Correctly detected |

### Analysis

**react-native-country-select (0.3.91):**
- 11 findings detected
- Threat score: 4.00 (below malicious threshold of 7.0)
- **QA Finding:** This package is confirmed malicious per Koi Security research
- **Issue:** Our scoring may be too conservative for this variant
- **Action:** Consider lowering threshold or adjusting detector weights

**react-native-international-phone-number (0.11.8):**
- 10 findings detected
- Threat score: 10.00 (maximum)
- **Correctly flagged as malicious** ✅
- Detection working as expected

### Comparison with Live Scans

**Wave 6/7 Results (live npm downloads):**
- react-native-country-select@0.3.1: 1 finding, score 3.50, NOT flagged
- react-native-international-phone-number@0.10.7: 0 findings, score 0.00, NOT flagged

**Evidence archive versions:**
- react-native-country-select@0.3.91: 11 findings, score 4.00, NOT flagged
- react-native-international-phone-number@0.11.8: 10 findings, score 10.00, FLAGGED ✅

**Conclusion:** Newer versions in evidence archive have more detectable patterns. The original malicious versions (0.3.1, 0.10.7) may have been cleaned or our detection needs tuning for those specific versions.

---

## Long-Running Campaign Design (from HANDOFF/FUTURE/)

### Key Features for 100k+ Package Campaigns

#### 1. Reliability Enhancements

**Adaptive Checkpointing:**
- Current: Fixed interval (every N packages)
- Proposed: Adaptive based on error rate, time, and wave boundaries
- Checkpoint every 50 packages OR 30 seconds (whichever is longer)
- Force checkpoint on wave completion

**Database Compaction:**
- SQLite checkpoint DB grows unbounded
- Periodic VACUUM and old data archival
- Retention: Keep detailed data for 3 recent waves, archive older

**Automatic Retry with Backoff:**
- Context-aware retry for different error types
- Circuit breaker pattern for persistent failures
- Retry budget per hour to prevent runaway

#### 2. Monitoring Enhancements

**Progress Trends:**
- Track packages/hour over time
- Detect slowdowns early
- Trend-adjusted ETA calculation

**Anomaly Detection:**
- Sudden slowdown alerts (>50% drop)
- High flag rate alerts (>20%)
- Error spike detection (>10%)
- Memory leak detection

**Resource Monitoring:**
- Memory, CPU, disk usage tracking
- Threshold-based alerts
- Open file descriptor monitoring

#### 3. Management Features

**Campaign Queue:**
- Priority-based queuing (critical, high, medium, low)
- Multiple campaigns waiting to run
- Reorder queue capability

**Multi-Campaign Orchestration:**
- Run multiple campaigns concurrently
- Resource allocation per campaign
- Fair share distribution

**Scheduled Pause/Resume:**
- Pause during business hours
- Resume overnight/weekends
- Cron-based scheduling

---

## Current Campaign Status

### Wave 8 Progress (Running in Background)

**Configuration:**
- 74 packages across 10 categories
- LLM Tier 1 + Tier 2 enabled
- Concurrency: 20
- Whitelist: 18 packages (i18n, build tools, crypto)

**Categories:**
1. Known Malicious Baseline (4 packages) - ✅ Complete
2. Clean Baseline (10 packages) - Running
3. Phone & SMS (9 packages) - Running
4. Auth & Biometrics (8 packages) - Running
5. Crypto & Blockchain (8 packages) - Running
6. Locale & Geofencing (6 packages) - Running
7. React Native UI (8 packages) - Running
8. Build & Dev Tools (6 packages) - Running
9. Utility Packages (8 packages) - Running
10. Network & HTTP (7 packages) - Running

**Early Findings:**
- crypto-js@4.2.0: FLAGGED as malicious (score 9.50, 5 findings)
- node-forge@1.3.1: FLAGGED as malicious (score 10.00, 61 findings)
- Several packages with findings but below threshold

---

## Whitelist Enhancement Proposal

### Current Behavior

Whitelist currently filters which packages to scan (package selection phase).

**Problem:** moment@2.30.1 was still flagged because:
1. It was in the campaign package list
2. Whitelist only filters at selection phase
3. Scoring doesn't consult whitelist

### Proposed Enhancement

Apply whitelist at **scoring phase** in addition to selection phase:

```rust
// In scanner.rs or scoring module
fn should_whitelist_package(package_name: &str, config: &WhitelistConfig) -> bool {
    // Check packages list
    if config.packages.iter().any(|p| package_name.starts_with(p)) {
        return true;
    }
    // Check crypto_packages
    if config.crypto_packages.iter().any(|p| package_name.starts_with(p)) {
        return true;
    }
    // Check build_tools
    if config.build_tools.iter().any(|p| package_name.starts_with(p)) {
        return true;
    }
    false
}

// In calculate_threat_score or is_malicious determination
if should_whitelist_package(&package_name, &whitelist_config) {
    // Override: not malicious regardless of findings
    return PackageVerdict::Clean("Whitelisted package".to_string());
}
```

### Benefits

1. **Defense in depth:** Whitelist at multiple phases
2. **False positive prevention:** Even if scanned, won't be flagged
3. **Operator confidence:** Known safe packages never flagged

### Implementation Priority

**Priority:** High (prevents false positives in large campaigns)
**Estimated Effort:** 2-3 hours
**Risk:** Low (whitelist is already configured, just applying it differently)

---

## Recommendations

### Immediate Actions

1. **Monitor Wave 8 completion** - Review findings, adjust thresholds if needed
2. **Implement whitelist enhancement** - Apply at scoring phase
3. **Review crypto-js and node-forge findings** - Confirm if true positives or need whitelist

### Short-Term (This Week)

1. **Run Wave 9 (500+ packages)** - Scale up gradually
2. **Implement adaptive checkpointing** - From long-running design
3. **Add progress trend tracking** - Better ETA estimation

### Long-Term (Next Month)

1. **Campaign queue system** - Multiple campaigns waiting
2. **Anomaly detection** - Automatic slowdown alerts
3. **Scheduled pause/resume** - Business hours awareness

---

## Files Referenced

- `HANDOFF/FUTURE/LONG-RUNNING-CAMPAIGNS.md` - Full design document (1688 lines)
- `evidence-archive/evidence/` - Known malicious package tarballs
- `campaigns/wave8-expanded-hunt.toml` - Current running campaign
- `glassware/src/config.rs` - Whitelist configuration

---

**Next Review:** After Wave 8 completion
**Contact:** Previous developer has full context on long-running design
