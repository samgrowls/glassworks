# Autoresearch Loop Setup Plan

**Version:** 1.0
**Date:** March 25, 2026
**Status:** 🟡 Planning Phase
**Author:** Glassworks Development Agent

---

## Executive Summary

This document outlines the plan to implement an **autoresearch optimization loop** for Glassworks to solve the false positive (FP) problem that manual tuning has failed to resolve after 6+ iterations.

### The Problem

- **Current FP Rate:** 17% (9 false positives out of 54 packages in Wave 11)
- **Target FP Rate:** ≤5%
- **Evidence Detection:** 100% (23/23 packages) ✅
- **Manual Tuning Attempts:** 6 iterations with diminishing returns

### The Solution

Implement an automated parameter optimization loop using **pi-autoresearch** that:
1. Tests 50-100+ configurations systematically
2. Uses F1 score as objective metric
3. Explores multi-parameter interactions
4. Converges to local optimum automatically

### Expected Timeline

| Phase | Duration | Outcome |
|-------|----------|---------|
| **Setup** | 1-2 days | Infrastructure ready |
| **Optimization** | 2-3 days | Optimal config found |
| **Validation** | 1 day | Phase A re-run successful |
| **Total** | **4-6 days** | FP rate ≤5%, ready for Phase B |

---

## Part 1: Current State Analysis

### 1.1 The 9 False Positives (Wave 11)

| Package | Score | Why Flagged | Root Cause |
|---------|-------|-------------|------------|
| @prisma/client | 10.00 | HeaderC2, TimeDelay | Telemetry + CI scripts |
| prisma | 10.00 | Same as client | Telemetry + CI scripts |
| @solana/web3.js | 10.00 | BlockchainC2 | Blockchain SDK patterns |
| typescript | 9.00 | LLM override (0.95) | Code generation patterns |
| firebase | 9.00 | LLM override (0.95) | Cloud SDK patterns |
| ethers | 6.70 | LLM override (0.90) | Blockchain SDK patterns |
| prettier | 6.37 | LLM override (0.90) | Code formatter patterns |
| viem | 5.27 | Score > 5.0 | Blockchain SDK patterns |
| webpack | 5.83 | Score > 5.0 | Build tool patterns |

### 1.2 Parameters to Optimize

Based on `glassware/src/scoring_config.rs` and `campaigns/phase-a-controlled/config.toml`:

| Parameter | Current | Range to Explore | Impact |
|-----------|---------|------------------|--------|
| `malicious_threshold` | 7.0-8.0 | 6.0-9.0 | High - controls what gets flagged |
| `suspicious_threshold` | 3.5-4.0 | 3.0-5.0 | Medium - affects reporting |
| `llm.tier1_threshold` | 6.0 | 5.0-7.5 | Medium - when to run fast LLM |
| `llm.tier2_threshold` | 7.0 | 6.0-8.5 | High - when to run deep LLM |
| `llm_override_confidence` | 0.95 | 0.80-0.99 | Critical - LLM override aggressiveness |
| `finding_base_weights.Critical` | 3.0 | 2.0-4.0 | High - critical finding impact |
| `finding_base_weights.High` | 2.0 | 1.5-3.0 | Medium - high finding impact |
| `known_c2_min_score` | 9.0 | 8.0-10.0 | Medium - C2 floor |
| `steganography_min_score` | 8.5 | 7.5-9.5 | Medium - steg floor |
| `pattern_dedup_enabled` | true | true/false | Low - dedup on/off |

**Total combinations:** ~70,000 (impossible to test manually)

### 1.3 Why Manual Tuning Failed

```
Manual Tuning Cycle (Repeated 6 times):
1. Discover FP category (e.g., i18n, telemetry, blockchain)
2. Implement fix for that category
3. FP rate drops temporarily
4. NEW FP category emerges (whack-a-mole)
5. Repeat

Root Causes:
• Infinite legitimate pattern space (can't enumerate all)
• LLM is inconsistent signal (helps sometimes, hurts others)
• Too many interdependent parameters for human optimization
• No convergence guarantee
```

---

## Part 2: Autoresearch Infrastructure

### 2.1 What is pi-autoresearch?

**pi-autoresearch** is an autonomous experiment loop extension that:
- Proposes configuration changes
- Runs benchmarks automatically
- Measures results (F1 score)
- Keeps what works, discards what doesn't
- Iterates 50-100+ times per day

**Key Features:**
- Domain-agnostic (works for any optimization problem)
- Confidence scoring using Median Absolute Deviation (MAD)
- Automatic commit/revert based on results
- Live dashboard for monitoring

### 2.2 Installation

```bash
# Install pi-autoresearch extension
pi install https://github.com/davebcn87/pi-autoresearch

# Or manual install
cp -r extensions/pi-autoresearch ~/.pi/agent/extensions/
cp -r skills/autoresearch-create ~/.pi/agent/skills/
pi /reload
```

### 2.3 Session Files Created

The autoresearch loop will create:

| File | Purpose |
|------|---------|
| `autoresearch.md` | Session document (objective, metrics, scope, history) |
| `autoresearch.sh` | Benchmark script (runs workload, outputs metric) |
| `autoresearch.config.json` | Configuration (working dir, max iterations) |
| `autoresearch.checks.sh` | Optional correctness checks (tests, lint) |
| `autoresearch.jsonl` | Append-only log of every run |

### 2.4 Benchmark Script Design

The benchmark script (`autoresearch.sh`) must:

1. **Pre-checks:** Ensure binary is built, evidence exists
2. **Run workload:** Scan all benchmark packages
3. **Output metric:** Print `METRIC name=F1_score value=0.XX`

**Example Structure:**

```bash
#!/bin/bash
# autoresearch.sh - Benchmark script for autoresearch loop

set -e

# === Pre-checks ===
if [ ! -f "target/release/glassware" ]; then
    echo "Building glassware..."
    cargo build --release -p glassware
fi

# === Scan evidence packages (should be flagged) ===
EVIDENCE_DETECTED=0
EVIDENCE_TOTAL=0
for pkg in evidence/*.tgz evidence/*/*.tgz; do
    [ -f "$pkg" ] || continue
    EVIDENCE_TOTAL=$((EVIDENCE_TOTAL + 1))
    OUTPUT=$(./target/release/glassware scan-tarball "$pkg" 2>&1 || true)
    if echo "$OUTPUT" | grep -q "malicious"; then
        EVIDENCE_DETECTED=$((EVIDENCE_DETECTED + 1))
    fi
done
DETECTION_RATE=$(echo "scale=4; $EVIDENCE_DETECTED / $EVIDENCE_TOTAL" | bc)

# === Scan clean packages (should NOT be flagged) ===
FP_COUNT=0
CLEAN_TOTAL=0
for pkg in benchmarks/clean-packages/*.tgz; do
    [ -f "$pkg" ] || continue
    CLEAN_TOTAL=$((CLEAN_TOTAL + 1))
    OUTPUT=$(./target/release/glassware scan-tarball "$pkg" 2>&1 || true)
    if echo "$OUTPUT" | grep -q "malicious"; then
        FP_COUNT=$((FP_COUNT + 1))
    fi
done
FP_RATE=$(echo "scale=4; $FP_COUNT / $CLEAN_TOTAL" | bc)

# === Calculate F1 score ===
PRECISION=$(echo "scale=4; 1.0 - $FP_RATE" | bc)
RECALL=$DETECTION_RATE
if (( $(echo "$PRECISION + $RECALL == 0" | bc -l) )); then
    F1=0
else
    F1=$(echo "scale=4; 2 * ($PRECISION * $RECALL) / ($PRECISION + $RECALL)" | bc)
fi

# === Output metric (autoresearch reads this) ===
echo "METRIC name=F1_score value=$F1"
echo "METRIC name=FP_rate value=$FP_RATE"
echo "METRIC name=Detection_rate value=$DETECTION_RATE"

# === Exit with status based on F1 ===
if (( $(echo "$F1 >= 0.90" | bc -l) )); then
    exit 0  # Success
else
    exit 1  # Needs improvement
fi
```

### 2.5 Configuration File

```json
{
  "workingDir": "/home/shva/samgrowls/glassworks",
  "maxIterations": 100,
  "targetMetric": "F1_score",
  "optimizationDirection": "maximize",
  "successThreshold": 0.90
}
```

---

## Part 3: Benchmark Dataset

### 3.1 Evidence Packages (Positive Class)

**Current Status:** 23 packages ✅

| Category | Count | Location |
|----------|-------|----------|
| Real malicious packages | 4 | `evidence/*.tgz` |
| GlassWorm steganography | 4 | `evidence/steganography/` |
| GlassWorm blockchain C2 | 4 | `evidence/blockchain_c2/` |
| GlassWorm time delay | 3 | `evidence/time_delay/` |
| GlassWorm exfiltration | 4 | `evidence/exfiltration/` |
| GlassWorm combined | 4 | `evidence/combined/` |

**Target:** ≥90% detection rate

### 3.2 Clean Packages (Negative Class) - NEEDS CREATION

**Current Status:** ❌ NOT CREATED

**Required:** 50-100 known-clean packages for FP measurement

**Proposed Benchmark Set:**

```bash
# benchmarks/clean-packages/packages.txt
# High-popularity packages that should be clean

# Core frameworks (10)
react
lodash
express
axios
moment
webpack
babel-core
typescript
eslint
prettier

# Database (5)
mongoose
sequelize
redis
mongodb
mysql2

# Testing (5)
jest
mocha
chai
sinon
supertest

# Build tools (5)
vite
rollup
esbuild
parcel
gulp

# Utilities (10)
async
chalk
debug
uuid
commander
dotenv
underscore
glob
fs-extra
bluebird

# HTTP/Networking (5)
node-fetch
got
request
superagent
follow-redirects

# UI Frameworks (10)
vue
@angular/core
jquery
bootstrap
tailwindcss
styled-components
antd
material-ui
@mui/material
emotion

# State Management (5)
redux
react-redux
mobx
vuex
pinia

# Date/Time (4)
dayjs
date-fns
luxon
moment-timezone

# Blockchain (4) - CRITICAL: These are currently FP
ethers
web3
viem
@solana/web3.js

# ORM/Database (3) - CRITICAL: These are currently FP
prisma
@prisma/client
typeorm

# Cloud SDKs (5)
firebase
@aws-sdk/client-s3
@google-cloud/storage
azure-storage
@azure/cosmos

# Monitoring/Observability (5)
winston
pino
@sentry/node
newrelic
datadog-metrics

# Security (5)
jsonwebtoken
bcrypt
bcryptjs
passport
crypto-js

# TOTAL: ~85 packages
```

### 3.3 Download Script

```bash
#!/bin/bash
# scripts/download-benchmark-packages.sh

set -e

BENCHMARK_DIR="benchmarks/clean-packages"
mkdir -p "$BENCHMARK_DIR"

echo "Downloading benchmark packages..."

while read -r package; do
    # Skip comments and empty lines
    [[ "$package" =~ ^#.*$ ]] && continue
    [[ -z "$package" ]] && continue
    
    echo "Downloading: $package"
    npm pack "$package" --pack-destination "$BENCHMARK_DIR/" 2>/dev/null || {
        echo "  WARNING: Failed to download $package"
    }
done < "$BENCHMARK_DIR/packages.txt"

echo ""
echo "Downloaded $(ls "$BENCHMARK_DIR"/*.tgz 2>/dev/null | wc -l) packages"
```

---

## Part 4: Implementation Plan

### Phase 1: Infrastructure Setup (Days 1-2)

#### Task 1.1: Install pi-autoresearch

```bash
cd /home/shva/samgrowls/glassworks
pi install https://github.com/davebcn87/pi-autoresearch
```

**Deliverable:** Autoresearch extension installed and working

#### Task 1.2: Create Clean Package Benchmark Set

```bash
# Create benchmark directory structure
mkdir -p benchmarks/clean-packages

# Create package list
cat > benchmarks/clean-packages/packages.txt << 'EOF'
# Core frameworks
react
lodash
express
# ... (full list from Section 3.2)
EOF

# Download packages
bash scripts/download-benchmark-packages.sh
```

**Deliverable:** 50-100 clean packages in `benchmarks/clean-packages/`

#### Task 1.3: Create Benchmark Script

Create `autoresearch.sh` (see Section 2.4 for template)

**Deliverable:** Working benchmark script that outputs F1 score

#### Task 1.4: Create Configuration

Create `autoresearch.config.json`:

```json
{
  "workingDir": "/home/shva/samgrowls/glassworks",
  "maxIterations": 100,
  "targetMetric": "F1_score",
  "optimizationDirection": "maximize",
  "successThreshold": 0.90,
  "parameterRanges": {
    "malicious_threshold": [6.0, 9.0],
    "suspicious_threshold": [3.0, 5.0],
    "llm_tier1_threshold": [5.0, 7.5],
    "llm_tier2_threshold": [6.0, 8.5],
    "llm_override_confidence": [0.80, 0.99],
    "finding_base_weights_critical": [2.0, 4.0],
    "finding_base_weights_high": [1.5, 3.0]
  }
}
```

**Deliverable:** Configuration file ready

### Phase 2: Optimization Run (Days 3-5)

#### Task 2.1: Start Autoresearch Loop

```bash
# Start autoresearch session
/skill:autoresearch-create

# Agent will ask:
# - Goal: "Minimize false positive rate while maintaining evidence detection"
# - Command: "./autoresearch.sh"
# - Metric: "F1_score (maximize)"
# - Files in scope: "glassware/src/scoring_config.rs, campaigns/phase-a-controlled/config.toml"
```

#### Task 2.2: Monitor Progress

**Dashboard Commands:**
- `Ctrl+X` - Expand status widget
- `Ctrl+Shift+X` - Fullscreen overlay
- `Escape` - Stop and ask for summary

**What to Watch:**
- Iteration count (target: 50-100)
- Best F1 score so far (target: ≥0.90)
- Confidence score (target: ≥2.0)
- FP rate trend (target: ≤5%)
- Detection rate trend (target: ≥90%)

#### Task 2.3: Mid-Run Adjustments

If after 25 iterations:
- **F1 < 0.80:** Expand parameter ranges
- **FP rate > 10%:** Add more clean packages to benchmark
- **Detection rate < 90%:** Review evidence packages

### Phase 3: Validation (Day 6)

#### Task 3.1: Apply Best Configuration

The autoresearch loop will output the best configuration found:

```toml
# campaigns/phase-a-controlled/config.toml (optimized)
[settings.scoring]
malicious_threshold = X.X  # Optimized value
suspicious_threshold = X.X

[settings.llm]
tier1_threshold = X.X
tier2_threshold = X.X

# glassware/src/scoring_config.rs (optimized)
fn default_finding_base_weights() -> HashMap<String, f32> {
    let mut weights = HashMap::new();
    weights.insert("Critical".to_string(), X.X);  // Optimized
    weights.insert("High".to_string(), X.X);
    // ...
}
```

#### Task 3.2: Run Phase A Campaign

```bash
./target/release/glassware campaign run campaigns/phase-a-controlled/config.toml --llm
```

**Expected Results:**
- 200 packages scanned in ~2-3 hours
- FP rate ≤5%
- Evidence detection ≥90%

#### Task 3.3: Create Final Report

Document:
- Best configuration found
- F1 score trajectory over iterations
- Final FP rate and detection rate
- Comparison to manual tuning attempts
- Recommendation for Phase B

---

## Part 5: Questions & Help Needed

### 5.1 Questions for Previous Agent / User

1. **LLM API Rate Limits:**
   - What are the current rate limits for Cerebras and NVIDIA APIs?
   - Will 50-100 iterations × ~200 packages cause rate limiting issues?
   - Should we implement a "dry run" mode without LLM for initial iterations?

2. **Clean Package Selection:**
   - Are there specific packages we should AVOID in the clean benchmark set?
   - Should we include the 9 FP packages from Wave 11 as a "known FP" test set?

3. **Configuration Constraints:**
   - Are there any parameters that should NOT be modified by autoresearch?
   - Any hard constraints (e.g., malicious_threshold must be ≥ 6.0)?

4. **Validation Criteria:**
   - Is F1 score ≥0.90 the right target?
   - Should we prioritize FP rate over detection rate (or vice versa)?

### 5.2 Help Needed from User

1. **API Key Verification:**
   ```bash
   # Verify NVIDIA API key is working
   curl -X POST https://integrate.api.nvidia.com/v1/chat/completions \
     -H "Authorization: Bearer $NVIDIA_API_KEY" \
     -H "Content-Type: application/json" \
     -d '{"model":"meta/llama3-70b-instruct","messages":[{"role":"user","content":"Hello"}]}'
   ```

2. **npm Registry Access:**
   - Confirm npm registry access is working for downloading 85+ benchmark packages
   - Any private packages that need authentication?

3. **Disk Space:**
   - 85 packages × ~500KB average = ~43MB for benchmark set
   - Confirm sufficient disk space in `/home/shva/samgrowls/glassworks/benchmarks/`

4. **Time Commitment:**
   - Estimated 4-6 days for full optimization
   - Daily check-ins recommended to monitor progress

---

## Part 6: Risk Mitigation

### 6.1 Potential Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **LLM rate limiting** | High | Medium | Implement caching, dry-run mode |
| **npm download failures** | Medium | Low | Retry logic, skip failed packages |
| **Autoresearch converges to local optimum** | Medium | Medium | Run multiple times with different seeds |
| **Optimized config breaks existing detection** | Low | High | Validate on evidence packages every iteration |
| **Build times too slow** | Medium | Medium | Pre-build binary, incremental compilation |

### 6.2 Fallback Plan

If autoresearch fails to find optimal configuration after 100 iterations:

1. **Manual Review:** Analyze top 10 configurations, identify patterns
2. **Hybrid Approach:** Manual tuning based on autoresearch insights
3. **Expand Benchmark:** Add more clean packages to better represent FP space
4. **Detector Fixes:** Implement specific fixes for top 3 FP categories (telemetry, CI/CD, blockchain)

---

## Part 7: Success Criteria

### 7.1 Primary Metrics

| Metric | Current | Target | Stretch Goal |
|--------|---------|--------|---------------|
| **F1 Score** | ~0.83 | ≥0.90 | ≥0.95 |
| **FP Rate** | 17% | ≤5% | ≤3% |
| **Detection Rate** | 100% | ≥90% | ≥95% |
| **Iterations Run** | 0 | 50-100 | 100+ |

### 7.2 Secondary Metrics

| Metric | Target |
|--------|--------|
| **Scan Time (per pkg)** | <30s |
| **LLM API Calls** | <500 total |
| **Confidence Score** | ≥2.0 |

### 7.3 Definition of Done

- [ ] Autoresearch infrastructure set up and working
- [ ] Clean package benchmark set created (50+ packages)
- [ ] Benchmark script outputs F1 score correctly
- [ ] 50-100 optimization iterations completed
- [ ] Best configuration identified with confidence ≥2.0
- [ ] Phase A campaign re-run with FP rate ≤5%
- [ ] Evidence detection rate ≥90%
- [ ] Final report created
- [ ] Ready to proceed to Phase B

---

## Part 8: Next Steps

### Immediate (Today)

1. **User Decision:** Confirm you want to proceed with autoresearch approach
2. **Install pi-autoresearch:** `pi install https://github.com/davebcn87/pi-autoresearch`
3. **Answer Questions:** Provide answers to Section 5.1 questions
4. **Verify API Keys:** Test NVIDIA and Cerebras API connectivity

### Day 1-2 (Setup)

1. Create clean package benchmark set
2. Create benchmark script (`autoresearch.sh`)
3. Create configuration file (`autoresearch.config.json`)
4. Test benchmark script manually
5. Start autoresearch session

### Day 3-5 (Optimization)

1. Monitor autoresearch loop progress
2. Make mid-run adjustments if needed
3. Track best configuration found

### Day 6 (Validation)

1. Apply best configuration
2. Run Phase A campaign
3. Create final report
4. Proceed to Phase B

---

## Appendix A: Command Reference

### Build Commands

```bash
# Debug build
cargo build -p glassware

# Release build (optimized)
cargo build --release -p glassware

# Check only (fast)
cargo check -p glassware
```

### Scan Commands

```bash
# Scan single package
./target/release/glassware scan-npm @prisma/client@5.8.1

# Scan tarball
./target/release/glassware scan-tarball evidence/package.tgz

# Scan directory
./target/release/glassware scan /path/to/code
```

### Campaign Commands

```bash
# Run campaign
./target/release/glassware campaign run campaigns/phase-a-controlled/config.toml --llm

# Monitor campaign
./target/release/glassware campaign monitor <case-id>

# Generate report
./target/release/glassware campaign report <case-id>
```

### Test Commands

```bash
# Validate evidence
./tests/validate-evidence.sh evidence target/release/glassware

# Run scoring tests
cargo test -p glassware scoring

# Run all tests
cargo test --release
```

---

## Appendix B: File Reference

| File | Purpose | Location |
|------|---------|----------|
| **Scoring Config** | Scoring parameters | `glassware/src/scoring_config.rs` |
| **Scoring Engine** | Score calculation | `glassware/src/scoring.rs` |
| **Phase A Config** | Campaign configuration | `campaigns/phase-a-controlled/config.toml` |
| **Evidence** | Malicious packages | `evidence/` |
| **Benchmarks** | Clean packages | `benchmarks/clean-packages/` (to create) |
| **Autoresearch** | Benchmark script | `autoresearch.sh` (to create) |
| **Autoresearch** | Configuration | `autoresearch.config.json` (to create) |
| **Validation** | Evidence test script | `tests/validate-evidence.sh` |

---

## Appendix C: Glossary

| Term | Definition |
|------|------------|
| **FP Rate** | False Positive Rate - % of clean packages incorrectly flagged |
| **Detection Rate** | % of malicious packages correctly identified |
| **F1 Score** | Harmonic mean of precision and recall |
| **Precision** | 1 - FP Rate (how many flagged are actually malicious) |
| **Recall** | Detection Rate (how many malicious are caught) |
| **Autoresearch** | Automated parameter optimization loop |
| **Phase A** | Pre-production validation on 200 curated packages |
| **Phase B** | Wild scanning on 500+ packages |

---

**Document Created By:** Glassworks Development Agent
**Date:** March 25, 2026
**Status:** 🟡 **Awaiting User Approval to Proceed**
