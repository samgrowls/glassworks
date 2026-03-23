# Wave 6 Execution Log

**Started:** March 22, 2026 20:16:10 UTC
**Case ID:** wave-6-calibration-20260322-201610
**Status:** ⏳ Running

---

## Campaign Configuration

**File:** `campaigns/wave6.toml`

**Waves:**
- **Wave 6A:** 2 known malicious packages (validate mode)
- **Wave 6B:** 5 known clean packages (validate mode)  
- **Wave 6C:** 4 React Native packages (hunt mode)

**Total:** 11 packages

---

## Execution Log

```
2026-03-22T20:16:10Z INFO Loading campaign configuration: campaigns/wave6.toml
2026-03-22T20:16:10Z INFO Loaded campaign 'Wave 6 Calibration' with 3 waves
2026-03-22T20:16:10Z INFO 🚀 Starting campaign execution...
2026-03-22T20:16:10Z INFO 🚀 Starting campaign 'Wave 6 Calibration' (case: wave-6-calibration-20260322-201610)
2026-03-22T20:16:10Z INFO Campaign has 2 execution stages
```

**Stage 1:** Wave 6A (no dependencies)
**Stage 2:** Wave 6B, Wave 6C (both depend on 6A, can run parallel)

---

## Expected Results

### Wave 6A: Known Malicious Baseline

**Expected:** Both packages flagged as malicious

| Package | Expected Threat Score | Expected LLM Verdict |
|---------|----------------------|---------------------|
| react-native-country-select@0.3.91 | ≥7.0 | Malicious |
| react-native-international-phone-number@0.11.8 | ≥7.0 | Malicious |

### Wave 6B: Clean Baseline

**Expected:** No packages flagged

| Package | Expected Threat Score |
|---------|----------------------|
| express@4.19.2 | <3.0 |
| lodash@4.17.21 | <3.0 |
| axios@1.6.7 | <3.0 |
| chalk@5.3.0 | <3.0 |
| debug@4.3.4 | <3.0 |

### Wave 6C: React Native Ecosystem

**Expected:** Variable (hunt mode)

| Package | Notes |
|---------|-------|
| react-native-phone-input@1.3.7 | Unknown |
| react-native-otp-inputs@0.3.1 | Unknown |
| react-native-locale@0.0.15 | Unknown |
| react-native-localize@3.0.6 | Unknown |

---

## Success Criteria

- ✅ Campaign completes without errors
- ✅ Wave 6A: 2/2 malicious packages detected (100% detection rate)
- ✅ Wave 6B: 0/5 clean packages flagged (0% false positive rate)
- ✅ Wave 6C: Results documented (hunt mode, no expectations)

---

## Issues & Observations

### (To be filled in during/after execution)

---

## Next Steps After Completion

1. **Review results** - Check detection rates
2. **Document findings** - Update this log
3. **Tune configuration** - Adjust thresholds if needed
4. **Proceed to Phase 2** - Implement resume, commands, reports

---

**Note:** Campaign is running in background. Check back in 10-15 minutes for completion.
