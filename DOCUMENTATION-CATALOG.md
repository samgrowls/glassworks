# Documentation Catalog — glassware

**Last updated:** 2026-03-19  
**Purpose:** Distinguish current/operational docs from historical/design docs  

---

## 🟢 CURRENT & OPERATIONAL (Use These)

### For Developers/Agents Continuing Work

| Document | Purpose | Status |
|----------|---------|--------|
| **`HANDOFF-WORKFLOW.md`** | **Production workflow guide** | ✅ **START HERE** |
| `HANDOFF.md` | Technical handoff with findings | ✅ Current |
| `SESSION-FINAL-2026-03-19.md` | Latest session summary | ✅ Current |
| `TODO.md` | Prioritized task list | ✅ Current |
| `OPTIMIZATION-ROADMAP.md` | Optimization plan | ✅ Current |

### For Running Scans

| Document | Purpose | Location |
|----------|---------|----------|
| `diverse_sampling.py --help` | Sampling parameters | Run `python3 diverse_sampling.py --help` |
| `optimized_scanner.py --help` | Scan parameters | Run `python3 optimized_scanner.py --help` |
| `batch_llm_analyzer.py --help` | LLM parameters | Run `python3 batch_llm_analyzer.py --help` |

### For Understanding Detections

| Document | Purpose | Status |
|----------|---------|--------|
| `glassware-core/src/detectors/` | Detector implementations | ✅ Source of truth |
| `glassware-core/src/scanner.rs` | Main scanner logic | ✅ Source of truth |
| `harness/reports/MANUAL-REVIEW-HIGH-URGENCY.md` | FP pattern examples | ✅ Current |

---

## 🟡 DESIGN & PLANNING (Reference Only)

### Future Implementations

| Document | Purpose | Status |
|----------|---------|--------|
| `LLM-ORCHESTRATION-DESIGN.md` | Multi-provider LLM design | 🟡 Design (not implemented) |
| `GITHUB-REPO-SCANNING-PLAN.md` | GitHub scanning plan | 🟡 Design (deferred) |
| `HIGH-ALTITUDE-REVIEW.md` | System optimization review | 🟡 Strategic planning |

### Implementation Guides

| Document | Purpose | Status |
|----------|---------|--------|
| `IMPLEMENTATION-SUMMARY-LOW-HANGING-FRUIT.md` | Recent improvements | 🟡 Historical (implemented) |
| `LLM-PROMPT-IMPROVEMENT-RESULTS.md` | LLM prompt tuning | 🟡 Historical (implemented) |
| `BUNDLED-MJS-FIX.md` | .mjs/.cjs filter fix | 🟡 Historical (implemented) |

---

## 🔴 HISTORICAL & SESSION LOGS (Archive)

### Session Summaries

| Document | Date | Purpose |
|----------|------|---------|
| `SESSION-SUMMARY-2026-03-18.md` | Mar 18 | Initial MCP discoveries |
| `SESSION-SUMMARY-CHECKPOINT.md` | Mar 18 | Checkpoint summary |
| `SESSION-SUMMARY-RATE-LIMITING.md` | Mar 19 | Rate limiting implementation |
| `SESSION-SUMMARY-FINAL-2026-03-18.md` | Mar 18 | Day 1 final summary |
| `SESSION-SUMMARY-FINAL-2026-03-19.md` | Mar 19 | Day 2 final summary |

### Analysis Reports

| Document | Date | Purpose |
|----------|------|---------|
| `ANALYSIS-IFLOW-MCP-TRIO.md` | Mar 19 | 3 @iflow-mcp packages |
| `ANALYSIS-REF-TOOLS-MCP.md` | Mar 19 | ref-tools-mcp analysis |
| `LLM-ANALYSIS-HIGH-PRIORITY.md` | Mar 19 | High-priority LLM results |
| `SAMPLING-SCAN-RESULTS.md` | Mar 19 | 110-package scan results |
| `SCAN-SUMMARY-COMPLETE.md` | Mar 19 | 506-package scan results |

### Discovery Reports

| Document | Date | Purpose |
|----------|---------|--------|
| `CRITICAL-THREAT-DISCOVERY.md` | Mar 18 | Initial threat discovery |
| `CRITICAL-MCP-FINDING.md` | Mar 18 | MCP package findings |
| `NEW-DISCOVERY-IFLOW-MCP-PHASE1.md` | Mar 18 | Phase 1 discoveries |
| `DETECTION-VALIDATION.md` | Mar 18 | Detection accuracy validation |

---

## 📊 Quick Reference

### What to Read First

**New developer/agent starting:**
1. `HANDOFF-WORKFLOW.md` ← **Start here**
2. `SESSION-FINAL-2026-03-19.md` ← Latest status
3. `TODO.md` ← What to do next

**Understanding a specific finding:**
1. Check `harness/reports/` for analysis reports
2. Review detector source code in `glassware-core/src/detectors/`
3. Check `MANUAL-REVIEW-HIGH-URGENCY.md` for FP patterns

**Implementing new features:**
1. Check `OPTIMIZATION-ROADMAP.md` for priorities
2. Review design docs (`LLM-ORCHESTRATION-DESIGN.md`, etc.)
3. Implement in `glassware-core/src/` or `harness/`

---

## 🗂️ File Organization

```
glassworks/
├── HANDOFF-WORKFLOW.md          ← 🟢 START HERE
├── HANDOFF.md                    ← 🟢 Current technical handoff
├── TODO.md                       ← 🟢 Current priorities
├── OPTIMIZATION-ROADMAP.md       ← 🟡 Design/plan
├── OPTI.md                       ← 🟡 Original optimization spec
│
├── harness/
│   ├── reports/
│   │   ├── SESSION-FINAL-2026-03-19.md  ← 🟢 Latest summary
│   │   ├── MANUAL-REVIEW-HIGH-URGENCY.md ← 🟢 FP patterns
│   │   ├── LLM-ORCHESTRATION-DESIGN.md   ← 🟡 Design
│   │   ├── GITHUB-REPO-SCANNING-PLAN.md  ← 🟡 Design
│   │   └── *.md                          ← 🔴 Historical session logs
│   └── data/
│       └── evidence/             ← Scan results & LLM analyses
│
├── glassware-core/
│   └── src/
│       ├── scanner.rs            ← 🟢 Main scanner
│       ├── detectors/            ← 🟢 Detection logic
│       └── ...
│
└── llm-analyzer/
    └── analyzer.py               ← 🟢 LLM analysis
```

---

## 📋 Maintenance

**When creating new docs:**
- Use clear naming: `SESSION-YYYY-MM-DD.md`, `ANALYSIS-PACKAGE-NAME.md`
- Add to this catalog within 24 hours
- Mark old docs as 🔴 Historical when superseded

**When implementing features:**
- Move design docs from 🟡 to 🔴 after implementation
- Update `HANDOFF-WORKFLOW.md` with new workflows
- Update `TODO.md` with new priorities

---

**Last cataloged:** 2026-03-19  
**Next review:** After 30k scan completes  
