# Phase A: Controlled Campaign - Pre-Production Validation

## Overview

**Campaign Name:** Pre-Production Validation  
**Version:** 0.35.0  
**Phase:** A (Controlled)  
**Duration:** 3 days  
**Status:** Ready for execution

---

## Purpose

This campaign validates Glassworks detection capabilities on **known-clean, high-popularity npm packages** before proceeding to wild scans. The goal is to establish a baseline false positive rate and ensure detector tuning is appropriate for production use.

### Primary Objectives

1. **Validate Low False Positive Rate** - Confirm ≤5% FP rate on curated clean packages
2. **Test LLM Pipeline** - Verify LLM triage and analysis work correctly under load
3. **Benchmark Performance** - Measure scan speed, memory usage, and resource consumption
4. **Manual Review Workflow** - Test the process for reviewing flagged packages

---

## Campaign Configuration

| Parameter | Value |
|-----------|-------|
| **Total Packages** | 200 |
| **Max Concurrency** | 4 |
| **Timeout per Package** | 300 seconds |
| **Malicious Threshold** | 7.0 |
| **Suspicious Threshold** | 3.5 |
| **LLM Triage** | Enabled (Cerebras) |
| **LLM Deep Dive** | Enabled (threshold: 4.0) |
| **Output Formats** | JSON, Markdown |

---

## Package Selection

### Tier 1: Ultra-Popular (50 packages)
The most downloaded npm packages with massive adoption:
- Core frameworks: React, Vue, Angular
- Build tools: Webpack, Vite, Babel, TypeScript
- Utilities: Lodash, Axios, Moment, Chalk
- Testing: Jest, Mocha, Chai, Sinon
- Database: Mongoose, Sequelize, Redis, MongoDB
- Security: JsonWebToken, Bcrypt, Passport

### Tier 2: Popular - 1M+ downloads/month (75 packages)
Widely adopted packages across categories:
- UI libraries: Material-UI, Ant Design, TailwindCSS
- State management: Redux, MobX, Pinia, Zustand
- Routing: React Router, Vue Router
- Forms: Formik, React Hook Form, VeeValidate
- GraphQL: Apollo, GraphQL.js
- Blockchain: Ethers, Web3, Viem

### Tier 3: Medium - 100K+ downloads/month (75 packages)
Specialized packages with significant adoption:
- UI components: React Select, Framer Motion, Headless UI
- Charts: Recharts, Chart.js, Victory
- Editors: Monaco, CodeMirror, Slate, TipTap
- Cloud: AWS SDK, Google Cloud Storage
- Monitoring: Sentry, Prometheus, DataDog
- ML: TensorFlow.js, Natural

---

## Success Criteria

| Metric | Target | Action if Failed |
|--------|--------|------------------|
| **False Positive Rate** | ≤5% | Tune detector thresholds, review scoring |
| **Scan Completion Rate** | ≥95% | Fix timeout/network issues |
| **LLM Response Rate** | ≥90% | Check API keys, rate limits |
| **Average Scan Time** | <1s/package | Optimize if slower |
| **Manual Review Time** | <5min/package | Improve LLM explanations |

---

## Execution Instructions

### 1. Pre-Campaign Checklist

```bash
# Verify release binary is built
cargo build --release -p glassware

# Verify LLM API keys are configured
echo $GLASSWARE_LLM_BASE_URL
echo $GLASSWARE_LLM_API_KEY

# Run pre-flight check
./target/release/glassware scan-npm react@18.2.0 --llm

# Clear any existing cache
rm -f .glassware-orchestrator-cache.db
```

### 2. Run Campaign

```bash
# Start the campaign
./target/release/glassware campaign run campaigns/phase-a-controlled/config.toml --llm

# Or use TUI for live monitoring
./target/release/glassware campaign demo
```

### 3. Monitor Progress

```bash
# Watch logs in real-time
tail -f logs/pre-production-validation-*.log

# Check completion status
grep "Wave.*completed" logs/pre-production-validation-*.log

# Monitor flagged packages
grep "flagged as" logs/pre-production-validation-*.log
```

### 4. Review Results

```bash
# View all packages with score ≥7.0 (malicious)
cat output/pre-production-validation/results.json | \
  jq '.[] | select(.score >= 7.0) | {package, version, score, findings}'

# View packages with score 3.5-6.9 (suspicious)
cat output/pre-production-validation/results.json | \
  jq '.[] | select(.score >= 3.5 and .score < 7.0) | {package, version, score}'

# Calculate false positive rate
cat output/pre-production-validation/results.json | \
  jq '[.[] | select(.score >= 7.0)] | length'

# Generate markdown report
./target/release/glassware campaign report <case-id>
```

### 5. Manual Review Process

For each package flagged as malicious (score ≥7.0):

1. **Review Findings**
   ```bash
   ./target/release/glassware scan-npm <package>@<version> --llm
   ```

2. **Check Code Snippets**
   - Open results.json
   - Find the package entry
   - Review `findings[].code_snippet` field

3. **LLM Analysis**
   ```bash
   ./target/release/glassware campaign query <case-id> \
     "Why was <package> flagged as malicious?"
   ```

4. **Manual Inspection** (if needed)
   ```bash
   # Download and extract package
   npm view <package>@<version> dist.tarball
   curl <tarball_url> | tar -xzf -
   
   # Search for suspicious patterns
   grep -rn "eval\|Function(" package/
   grep -rn "setTimeout\|setInterval" package/
   ```

5. **Document Verdict**
   - True Positive: Actual malicious code detected
   - False Positive: Legitimate code incorrectly flagged
   - Suspicious: Needs further investigation

---

## Expected Outcomes

### Best Case (Go to Phase B)
- ✅ False positive rate ≤5%
- ✅ All evidence packages detected (if included)
- ✅ LLM analysis provides useful explanations
- ✅ Scan speed ≥30k LOC/sec

### Acceptable (Minor Tuning Required)
- ⚠️ False positive rate 5-10%
- ⚠️ Some LLM responses unclear
- ⚠️ Scan speed 20-30k LOC/sec

**Action:** Tune thresholds, improve LLM prompts, re-run Phase A

### Unacceptable (Major Issues)
- ❌ False positive rate >10%
- ❌ Evidence packages not detected
- ❌ LLM pipeline failures
- ❌ Scan speed <20k LOC/sec

**Action:** Pause campaign, fix root causes, re-validate evidence

---

## Output Files

After campaign completion:

```
output/phase-a-controlled/
├── results.json           # Full scan results
├── results.md             # Markdown summary report
├── progress.log           # Execution log
├── checkpoint.json        # Resume checkpoint
├── evidence/              # Detected findings with code snippets
│   ├── <package>-<version>/
│   └── ...
└── reports/
    └── final-report.md    # Stakeholder report
```

---

## Troubleshooting

### High False Positive Rate

1. Identify common pattern:
   ```bash
   cat output/*/results.json | \
     jq '.[] | select(.score >= 7.0) | .findings[].category' | \
     sort | uniq -c | sort -rn
   ```

2. Review detector thresholds in `glassware-core/src/scoring/`

3. Adjust `suspicious_threshold` in config (try 4.0-5.0)

4. Re-run on subset of packages

### LLM API Failures

1. Verify API keys:
   ```bash
   curl -H "Authorization: Bearer $GLASSWARE_LLM_API_KEY" \
     https://api.cerebras.ai/v1/models
   ```

2. Check rate limits in logs

3. Disable LLM temporarily:
   ```bash
   ./target/release/glassware campaign run campaigns/phase-a-controlled/config.toml
   ```

### Package Download Failures

1. Check npm registry status: https://status.npmjs.org/

2. Increase timeout in config:
   ```toml
   timeout_seconds = 600
   ```

3. Reduce concurrency:
   ```toml
   max_concurrent = 2
   ```

---

## Next Steps

After Phase A completion:

1. **Review Results** - Manual review of all flagged packages
2. **Calculate Metrics** - FP rate, detection rate, scan speed
3. **Decision Point:**
   - **Go:** FP rate ≤5% → Proceed to Phase B (Wild Scan - Small)
   - **No-Go:** FP rate >10% → Tune detectors, re-run Phase A
   - **Conditional:** FP rate 5-10% → Minor tuning, partial proceed

---

## Contacts

| Role | Responsibility |
|------|----------------|
| Security Team | Campaign execution, manual review |
| DevOps | Infrastructure, monitoring |
| Project Lead | Go/No-Go decisions |

---

## Campaign History

| Date | Action | Notes |
|------|--------|-------|
| 2026-03-24 | Campaign created | Configuration files generated |
| TBD | Campaign started | Awaiting execution |
| TBD | Campaign completed | Results pending |

---

**Document Version:** 1.0  
**Last Updated:** 2026-03-24  
**Classification:** Internal Use Only
