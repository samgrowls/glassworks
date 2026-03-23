# Wave 9 Planning - 500+ Package Scale

**Date:** March 23, 2026
**Status:** Planning - Ready to Execute After Wave 8 Validation

---

## Current Status

### Active Campaigns (Running in Parallel)

| Campaign | Packages | Status | Logging |
|----------|----------|--------|---------|
| **Wave 8** | 74 packages | 🟡 Running | `logs/wave8-debug.log` (55K) |
| **GitHub Scan** | 1 repo (glassworks) | 🟡 Running | `logs/github-scan-debug.log` (52K) |

### Wave 8 Categories

1. ✅ Known Malicious Baseline (4 packages)
2. ⏳ Clean Baseline (10 packages)
3. ⏳ Phone & SMS (9 packages)
4. ⏳ Auth & Biometrics (8 packages)
5. ⏳ Crypto & Blockchain (8 packages)
6. ⏳ Locale & Geofencing (6 packages)
7. ⏳ React Native UI (8 packages)
8. ⏳ Build & Dev Tools (6 packages)
9. ⏳ Utility Packages (8 packages)
10. ⏳ Network & HTTP (7 packages)

**Total:** 74 packages

---

## Wave 9 Design (500+ Packages)

### Goal

Scale to 500+ packages while maintaining:
- Zero false positives (whitelist working)
- High true positive rate (malicious detection)
- Reasonable completion time (< 2 hours)
- Full LLM analysis on flagged packages

### Package Sources

#### 1. React Native Ecosystem (150 packages)
- Top React Native libraries
- Native modules
- UI components
- Navigation libraries

#### 2. Vue.js Ecosystem (100 packages)
- Vue core and plugins
- Nuxt.js ecosystem
- Vue UI libraries
- State management

#### 3. Angular Ecosystem (100 packages)
- Angular core and CLI
- Material components
- RxJS and dependencies
- NgRx state management

#### 4. High-Risk Categories (100 packages)
- Phone/SMS handling
- Auth/biometrics
- Crypto/blockchain
- Locale/geofencing
- Network utilities

#### 5. Clean Baseline (50 packages)
- Top npm packages (known clean)
- Build tools
- Testing frameworks
- Documentation tools

---

## Configuration Strategy

### Whitelist (Critical for 500+ packages)

**Already configured in wave8:**
```toml
[settings.whitelist]
packages = [
    "moment", "lodash", "express",
    "i18next", "react-intl",
    # ... 18 total entries
]
```

**For Wave 9, expand to include:**
- All top 50 npm packages (review manually)
- All major framework packages
- All major build tools

### Scoring Thresholds

**Current:**
```toml
[settings.scoring]
malicious_threshold = 7.0
suspicious_threshold = 4.0
```

**Wave 9 tuning:**
- Start with current thresholds
- Adjust based on Wave 8 results
- Consider package-specific overrides for edge cases

### LLM Strategy

**Tier 1 (Cerebras):** Enable for all packages
- Fast triage (~2-5s per package)
- 500 packages × 5s = ~42 minutes total

**Tier 2 (NVIDIA):** Enable for flagged only
- Deep analysis (~15-30s per flagged)
- Expect 5-10% flagged = 25-50 packages
- 50 packages × 30s = ~25 minutes total

**Total LLM time:** ~67 minutes (acceptable)

---

## Execution Plan

### Phase 1: Wave 8 Validation (Today)

**Goals:**
- [ ] Wave 8 completes successfully
- [ ] No false positives (whitelist working)
- [ ] Malicious packages correctly flagged
- [ ] LLM analysis completes without errors
- [ ] Review logs for any issues

**Success Criteria:**
- Zero whitelisted packages flagged
- Known malicious packages detected
- Completion time < 30 minutes

### Phase 2: Wave 9 Preparation (Tomorrow)

**Tasks:**
1. Create `campaigns/wave9-500plus.toml`
2. Expand whitelist based on Wave 8 learnings
3. Prepare package lists by category
4. Set up evidence collection directories

### Phase 3: Wave 9 Execution (Day 2-3)

**Run with:**
```bash
glassware campaign run campaigns/wave9-500plus.toml \
  --llm --deep-llm \
  --log-level info \
  --log-file logs/wave9-run.log
```

**Expected Duration:** 1.5-2 hours

**Monitoring:**
- Check progress every 30 minutes
- Monitor log for errors
- Watch for memory/CPU issues

### Phase 4: Analysis & Tuning (Day 3-4)

**Tasks:**
1. Review all flagged packages
2. Use LLM queries to understand findings
3. Manually investigate suspicious packages
4. Tune thresholds if needed
5. Document false positives (update whitelist)

---

## Threshold Tuning Strategy

### When to Adjust

**Lower threshold (e.g., 7.0 → 6.0):**
- Known malicious packages not flagged
- LLM confirms malicious but score is low
- Multiple weak signals that should stack higher

**Raise threshold (e.g., 7.0 → 8.0):**
- Too many false positives despite whitelist
- Legitimate packages consistently flagged
- LLM confirms clean but score is high

### Package-Specific Overrides

**Consider adding:**
```toml
[settings.package_overrides]
# Lower threshold for high-risk packages
"react-native-*" = { threshold = 6.0 }
# Raise threshold for complex build tools
"webpack-*" = { threshold = 8.0 }
```

**Implementation:** Would require code change in `scanner.rs`

---

## Evidence Collection

### What to Collect

**For MALICIOUS packages only:**
- Full package source (tarball)
- Specific files with findings
- LLM analysis results
- Threat score breakdown
- Network indicators (if any)

**Storage:**
```
evidence/wave9/
├── <package-name>/
│   ├── source.tgz
│   ├── findings.json
│   ├── llm-analysis.json
│   └── manual-review.md
```

### Git Ignore

Already configured:
```
# .gitignore
evidence/
```

---

## Success Metrics

| Metric | Target | Wave 8 | Wave 9 Target |
|--------|--------|--------|---------------|
| Packages | 500+ | 74 | 500+ |
| False Positives | 0 | TBD | 0 |
| True Positives | >90% | TBD | >90% |
| Completion Time | <2h | ~10min | <2h |
| LLM Success Rate | >95% | TBD | >95% |
| Evidence Collected | Malicious only | TBD | Yes |

---

## Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Network failures | High | Checkpoint/resume, retry logic |
| Rate limiting | Medium | Adaptive rate limiter |
| Memory exhaustion | Medium | Monitor usage, reduce concurrency |
| False positives | High | Whitelist enhancement (done!) |
| Missing malicious | High | Lower threshold, LLM review |

---

## Next Steps

### Immediate (Today)
1. ✅ Wave 8 running with logging
2. ✅ GitHub scan running in parallel
3. ⏳ Monitor Wave 8 completion
4. ⏳ Review Wave 8 results

### Tomorrow
1. Create Wave 9 configuration
2. Expand whitelist based on Wave 8
3. Prepare 500+ package list
4. Run Wave 9

### Day 3-4
1. Analyze Wave 9 results
2. Tune thresholds
3. Manual investigation of flagged
4. Plan Wave 10 (1000+ packages)

---

## Logging & Debugging

### Current Log Files

| File | Purpose | Size |
|------|---------|------|
| `logs/wave8-debug.log` | Wave 8 debug output | 55K |
| `logs/github-scan-debug.log` | GitHub scan debug | 52K |

### Log Levels

- **DEBUG:** File-by-file scanning, network requests
- **INFO:** Package completion, wave progress
- **WARN:** Flagged packages, rate limit hits
- **ERROR:** Failures, network errors

### How to Check Progress

```bash
# Check if process is running
ps aux | grep glassware

# View last 50 lines of log
tail -50 logs/wave8-debug.log

# Count packages scanned
grep "Package.*scanned" logs/wave8-debug.log | wc -l

# Count flagged packages
grep "flagged as malicious" logs/wave8-debug.log
```

---

**Last Updated:** March 23, 2026
**Next Review:** After Wave 8 completion
