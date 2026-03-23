# Campaign Ramp-Up Plan

**Version:** 1.0
**Date:** March 23, 2026
**Status:** Active - Wave 6/7 QA, Wave 8 Ready

---

## Objective

Scale GlassWare detection from calibration (11 packages) to production-scale (1000+ packages) while:
1. Finding real malicious packages in the wild
2. Minimizing false positives through proper whitelisting
3. Using both LLM tiers for accurate classification
4. Collecting evidence for confirmed malicious code only

---

## Campaign Progression

| Wave | Packages | Purpose | Status | LLM Tiers |
|------|----------|---------|--------|-----------|
| **Wave 6** | 11 | Calibration & validation | ✅ Ready | Tier 1 + Tier 2 |
| **Wave 7** | 24 | Real-world hunt (small) | ✅ Ready | Tier 1 + Tier 2 |
| **Wave 8** | 100+ | Expanded hunt | ✅ Ready | Tier 1 + Tier 2 |
| **Wave 9** | 500+ | Large-scale hunt | ⚪ Planned | Tier 1 + Tier 2 |
| **Wave 10** | 1000+ | Production-scale | ⚪ Planned | Tier 1 + Tier 2 |

---

## Current Phase: Wave 6/7 QA

### Goals
- Validate detection accuracy with known malicious packages
- Confirm whitelist prevents false positives
- Test LLM integration (both tiers)
- Verify evidence collection works

### Commands

```bash
# Run Wave 6 (calibration)
glassware campaign run campaigns/wave6.toml --llm --deep-llm

# Run Wave 7 (real-world hunt)
glassware campaign run campaigns/wave7-real-hunt.toml --llm --deep-llm

# Monitor progress
glassware campaign monitor <case-id>

# Generate report
glassware campaign report <case-id> --format markdown

# Query findings with LLM
glassware campaign query <case-id> "Why was this package flagged?"
```

### Success Criteria
- [ ] Known malicious packages flagged (react-native-country-select, etc.)
- [ ] Whitelisted packages NOT flagged (moment, lodash, express)
- [ ] LLM analysis completes without errors
- [ ] Evidence collected for flagged packages
- [ ] Reports generated successfully

---

## Next Phase: Wave 8 (Expanded Hunt)

### Goals
- Scale to 100+ packages
- Cover 10 high-risk categories
- Use both LLM tiers for all flagged packages
- Collect evidence for malicious findings only

### Categories Covered
1. **Known Malicious** (validation) - 4 packages
2. **Clean Baseline** (FP validation) - 10 packages
3. **Phone/SMS** (high risk) - 9 packages
4. **Auth/Biometrics** (high risk) - 8 packages
5. **Crypto/Blockchain** (medium risk) - 8 packages
6. **Locale/Geofencing** (medium risk) - 6 packages
7. **React Native UI** (medium risk) - 8 packages
8. **Build/Dev Tools** (low risk) - 6 packages
9. **Utility Packages** (medium risk) - 8 packages
10. **Network/HTTP** (medium risk) - 7 packages

**Total:** ~74 packages (expandable)

### Commands

```bash
# Run Wave 8 with full LLM analysis
glassware campaign run campaigns/wave8-expanded-hunt.toml --llm --deep-llm

# Monitor with TUI
glassware campaign monitor <case-id>

# Generate comprehensive report
glassware campaign report <case-id> --format markdown --llm
```

---

## Future Phases

### Wave 9: Large-Scale Hunt (500+ packages)

**Focus:** Ecosystem-wide scanning
**Categories:**
- Top 100 npm packages (validation)
- React Native ecosystem (200 packages)
- Vue/Angular ecosystem (200 packages)
- High-risk categories (50 packages)

**Timeline:** After Wave 8 validation

### Wave 10: Production-Scale (1000+ packages)

**Focus:** Production-ready detection
**Categories:**
- Top 500 npm packages
- Multiple ecosystems (npm, PyPI, RubyGems)
- GitHub repositories with suspicious patterns

**Timeline:** After Wave 9 validation

---

## Evidence Collection

### What to Collect

**For MALICIOUS packages only:**
- Full package source code
- Specific files with findings
- LLM analysis results
- Threat score breakdown
- Network indicators (if any)

**What NOT to collect:**
- Clean package source (waste of space)
- False positive evidence (whitelisted)
- Temporary scan files

### Evidence Directory Structure

```
evidence/
├── wave6/           # Calibration evidence
│   └── <package-name>/
│       ├── source/  # Package source
│       ├── findings.json
│       └── llm-analysis.json
├── wave7/           # Real-world hunt evidence
└── wave8/           # Expanded hunt evidence
```

### Git Ignore

Evidence directories are gitignored:
```
# .gitignore
evidence/
data/evidence/
```

---

## LLM Strategy

### Tier 1 (Cerebras) - Fast Triage
- **Speed:** ~2-5 seconds per package
- **Purpose:** Initial classification during scan
- **Enabled:** `--llm` flag or `tier1_enabled = true`
- **Use:** All packages in scan

### Tier 2 (NVIDIA) - Deep Analysis
- **Speed:** ~15-30 seconds per package
- **Purpose:** Detailed analysis of flagged packages
- **Enabled:** `--deep-llm` flag or `tier2_enabled = true`
- **Use:** Packages with threat_score >= threshold

### Configuration

```toml
[settings.llm]
tier1_enabled = true   # Fast triage for all
tier1_provider = "cerebras"
tier2_enabled = true   # Deep analysis for flagged
tier2_threshold = 7.0  # Only high-risk packages
```

---

## QA Process

### For Each Wave

1. **Run Campaign**
   ```bash
   glassware campaign run campaigns/waveX.toml --llm --deep-llm
   ```

2. **Review Results**
   - Check flagged packages
   - Verify whitelist working (no FPs for known packages)
   - Review LLM analysis quality

3. **Generate Report**
   ```bash
   glassware campaign report <case-id> --format markdown
   ```

4. **Document Findings**
   - True positives: Malicious packages confirmed
   - False positives: Update whitelist
   - False negatives: Adjust detection rules

5. **Decide Next Steps**
   - Wave successful → Proceed to next wave
   - Issues found → Fix and re-run

### Metrics to Track

| Metric | Target | Wave 6 | Wave 7 | Wave 8 |
|--------|--------|--------|--------|--------|
| Packages scanned | - | 11 | 24 | 100+ |
| True positives | 100% of known | - | - | - |
| False positives | < 5% | - | - | - |
| LLM success rate | > 95% | - | - | - |
| Evidence collected | Malicious only | - | - | - |

---

## Known Malicious Packages (Reference)

These packages are confirmed malicious and MUST be flagged:

| Package | Version | Threat | Source |
|---------|---------|--------|--------|
| react-native-country-select | 0.3.1 | GlassWare steganography | Koi Security |
| react-native-international-phone-number | 0.10.7 | GlassWare steganography | Koi Security |
| react-native-phone-input | 1.3.7 | GlassWare patterns | Community reports |
| react-native-otp-inputs | 0.3.1 | GlassWare patterns | Community reports |

---

## Known False Positives (Whitelisted)

These packages naturally contain Unicode and should NOT be flagged:

| Package | Reason |
|---------|--------|
| moment, moment-timezone | i18n data |
| lodash | Complex utility patterns |
| express | Middleware patterns, i18n |
| react-intl, i18next | Internationalization |
| date-fns, dayjs | Date formatting Unicode |
| globalize | Globalization library |
| prettier, typescript, eslint | Build tools |

---

## Troubleshooting

### High False Positive Rate

**Symptom:** Many clean packages flagged

**Fix:**
1. Add to whitelist in campaign config
2. Adjust scoring thresholds
3. Re-run campaign

### Missing Malicious Packages

**Symptom:** Known malicious packages not flagged

**Fix:**
1. Lower malicious_threshold
2. Check detector enablement
3. Review LLM analysis

### LLM Errors

**Symptom:** LLM analysis fails

**Fix:**
1. Check API keys in ~/.env
2. Verify network connectivity
3. Try without --deep-llm first

---

## Contact & Support

- **Documentation:** `docs/CAMPAIGN-USER-GUIDE.md`
- **Agent Guidance:** `HANDOFF/AGENT-GUIDANCE.md`
- **Corrections:** `docs/binaryconsolidation/CORRECTIONS.md`

---

**Last Updated:** March 23, 2026
**Next Review:** After Wave 8 completion
