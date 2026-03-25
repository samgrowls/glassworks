# Glassworks Documentation Index

**Version:** v0.40.2-two-thresholds  
**Last Updated:** March 25, 2026

---

## 📚 Documentation Categories

### 1. Getting Started

| Document | Location | Purpose |
|----------|----------|---------|
| **README.md** | Root | Project overview, quick start |
| **QWEN.md** | Root | Project context, architecture |
| **HANDOFF-AGENT.md** | Root | **START HERE** - Agent handoff |

---

### 2. User Guides

| Document | Location | Purpose |
|----------|----------|---------|
| **CAMPAIGN-USER-GUIDE.md** | `docs/` | How to run campaigns |
| **DETECTION.md** | `docs/` | Detector reference (26 detectors) |
| **SCORING.md** | `docs/` | Scoring system specification |
| **LLM.md** | `docs/` | LLM integration guide |

---

### 3. Developer Handoff

| Document | Location | Purpose |
|----------|----------|---------|
| **HANDOFF-AGENT.md** | Root | **Primary handoff doc** |
| **HANDOFF/** | Directory | Legacy handoff docs |
| **REMEDIATION-FINAL-REPORT.md** | Root | Phase 1-7 summary |
| **GLASSWORM-INTEGRATION-SUMMARY.md** | Root | Phase 8-10 summary |

---

### 4. Audit & Analysis

| Document | Location | Purpose |
|----------|----------|---------|
| **PROMPT.md** | Root | Original remediation playbook |
| **PROMPT2.md** | Root | GlassWorm enhancement plan |
| **PROMPT3.md** | Root | Post-tuning audit |
| **PROMPT4.md** | Root | Pre-campaign briefing |
| **PROMPT5.md** | Root | Scoring redesign |
| **PROMPT6.md** | Root | Final FP reduction |
| **PROMPT7.md** | Root | LLM rate fix |

---

### 5. Campaign Results

| Document | Location | Purpose |
|----------|----------|---------|
| **wave11-live.log** | `output/` | Wave 11 live scan log |
| **wave11-results.md** | `output/` | Wave 11 results summary |
| **wave11-critical-analysis.md** | `output/` | FP investigation |
| **fp-investigation-prisma.md** | `output/` | @prisma deep dive |
| **FINAL-FP-FIXES-SUMMARY.md** | `output/phase-a-controlled/` | FP fixes summary |
| **LLM-RATE-LIMIT-FIX.md** | `output/phase-a-controlled/` | LLM performance fix |
| **TWO-THRESHOLD-CONFIG.md** | `output/phase-a-controlled/` | Two-threshold system |
| **SCORING-REDESIGN-RESULTS.md** | `output/phase-a-controlled/` | Scoring redesign results |

---

### 6. Evidence Library

| Category | Location | Packages |
|----------|----------|----------|
| **Original Evidence** | `evidence/*.tgz` | 4 packages |
| **Steganography** | `evidence/steganography/` | 4 packages |
| **Blockchain C2** | `evidence/blockchain_c2/` | 4 packages |
| **Time Delay** | `evidence/time_delay/` | 3 packages |
| **Exfiltration** | `evidence/exfiltration/` | 4 packages |
| **Combined** | `evidence/combined/` | 4 packages |
| **Total** | | **23 packages** |

---

### 7. Campaign Configurations

| Campaign | Location | Packages | Purpose |
|----------|----------|----------|---------|
| **Phase A** | `campaigns/phase-a-controlled/` | 200 | Pre-production validation |
| **Wave 6** | `campaigns/wave6.toml` | ~50 | Calibration |
| **Wave 7** | `campaigns/wave7-real-hunt.toml` | ~100 | Real hunt |
| **Wave 8** | `campaigns/wave8-expanded-hunt.toml` | ~200 | Expanded hunt |
| **Wave 9** | `campaigns/wave9-500plus.toml` | 500+ | Large hunt |
| **Wave 10** | `campaigns/wave10-1000plus.toml` | 1000+ | Production scale |
| **Wave 11** | `campaigns/wave11-evidence-validation.toml` | 54 | Evidence validation |
| **Wave 12** | `campaigns/wave12-5000pkg.toml` | 5000 | Large scale |

---

## 🚀 Quick Reference

### Build Commands

```bash
# Debug build
cargo build -p glassware

# Release build (optimized)
cargo build --release -p glassware

# Check only (fast)
cargo check -p glassware
```

### Run Commands

```bash
# Run campaign
./target/release/glassware campaign run campaigns/phase-a-controlled/config.toml --llm

# Scan single package
./target/release/glassware scan-npm @prisma/client@5.8.1

# Validate evidence
./tests/validate-evidence.sh evidence target/release/glassware
```

### Test Commands

```bash
# Run detector tests
cargo test -p glassware-core detector

# Run scoring tests
cargo test -p glassware scoring

# Run all tests
cargo test --release
```

---

## 📊 Current Status

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| **Detectors** | 26 | - | ✅ Complete |
| **Evidence Packages** | 23 | 20+ | ✅ Complete |
| **LLM Pipeline** | 3-stage | - | ✅ Complete |
| **Scan Time** | ~16s/pkg | <30s | ✅ Complete |
| **FP Rate** | 17% | ≤5% | ❌ **BLOCKED** |

---

## ⚠️ Current Blockers

1. **LLM Override Too Aggressive** - Confidence threshold 0.75 → 0.95
2. **Detectors Missing Context** - Telemetry, CI/CD, database patterns
3. **Scoring Exceptions** - @prisma scoring 10.00 (investigate)

**See:** `HANDOFF-AGENT.md` for detailed next steps.

---

## 📞 External Resources

| Organization | URL | Purpose |
|--------------|-----|---------|
| Koi Security | https://www.koisecurity.io/ | GlassWorm research |
| Aikido Security | https://www.aikido.dev/ | Supply chain analysis |
| Socket.dev | https://socket.dev/ | Real-time detection |
| npm Security | https://www.npmjs.com/security | Report malicious packages |

---

**Index Maintained By:** Glassworks Development Agent  
**Last Updated:** March 25, 2026  
**Version:** v0.40.2-two-thresholds
