# P3 Sprint — Revised Plan (Incorporating Expert Feedback)

**Date:** 2026-03-20 15:00 UTC  
**Status:** ✅ REVISED & READY  
**Changes:** 10 tasks added, 3 adjusted priorities, 4 pre-sprint research tasks  

---

## Expert Review Summary

**Overall Assessment:** ⭐⭐⭐⭐☆ (4/5) - Excellent planning with minor gaps

**Key Recommendations Incorporated:**
1. ✅ Split orchestrator into workspace members (library + CLI)
2. ✅ Integrate adversarial testing during implementation (not just Phase 4)
3. ✅ Add SQLite migration from Python cache
4. ✅ Prioritize mutation strategies by real-world prevalence
5. ✅ Benchmark Python harness before sprint
6. ✅ Add cross-file mutation strategies
7. ✅ Implement streaming output (prevent OOM)
8. ✅ Optimize SQLite with WAL mode + batch writes
9. ✅ Run adversarial tests on nightly CI/CD (not PR)

---

## Pre-Sprint Research Tasks (4-8h)

### PS-1: Benchmark Python Harness Baseline

**Task:** Measure current Python harness performance

**Commands:**
```bash
cd harness
time python3 optimized_scanner.py diverse-500.txt -w 10 -e data/evidence/baseline -o baseline-results.json
```

**Metrics to Record:**
- Scan speed (packages/min)
- Memory peak (RSS)
- Error rate (%)
- Cache hit rate (%)

**File:** `harness/benchmarks/python-baseline.md`

**Effort:** 2h  
**Owner:** Lead  
**Status:** ⏳ Pending

---

### PS-2: SQLite Schema Migration Design

**Task:** Design migration from Python JSON cache to SQLite

**Current Python Cache:**
```json
// .glassware-cache.json
{
  "package-hash": {
    "findings": [...],
    "timestamp": "2026-03-20T..."
  }
}
```

**Target SQLite Schema:**
```sql
CREATE TABLE scan_results (
    id INTEGER PRIMARY KEY,
    package_hash TEXT UNIQUE,
    package_name TEXT,
    findings_json TEXT,
    scanned_at DATETIME,
    expires_at DATETIME
);
```

**Migration Script:**
```python
# harness/migrate_cache.py
import json, sqlite3

with open('.glassware-cache.json') as f:
    cache = json.load(f)

conn = sqlite3.connect('orchestrator-cache.db')
for pkg_hash, data in cache.items():
    conn.execute(
        "INSERT INTO scan_results (package_hash, findings_json, expires_at) VALUES (?, ?, datetime('now', '+7 days'))",
        (pkg_hash, json.dumps(data['findings']))
    )
conn.commit()
```

**File:** `harness/specs/CACHE-MIGRATION-SPEC.md`

**Effort:** 2h  
**Owner:** Subagent 1  
**Status:** ⏳ Pending

---

### PS-3: glassware-core API Stability Review

**Task:** Verify P3 changes won't break existing detector trait

**Review Points:**
- ✅ Detector trait already accepts `&FileIR` (from P1)
- ✅ No breaking changes needed for adversarial testing
- ✅ Orchestrator can use glassware-core as library

**File:** `harness/reports/API-STABILITY-REVIEW.md`

**Effort:** 1h  
**Owner:** Lead  
**Status:** ✅ COMPLETE (no breaking changes needed)

---

### PS-4: Research npm API Rate Limits

**Task:** Test current npm API rate limits

**Test:**
```bash
# Unauthenticated
for i in {1..20}; do
  time curl -s "https://registry.npmjs.org/-/v1/search?text=mcp-server&size=100" > /dev/null
done
```

**File:** `harness/reports/NPM-RATE-LIMITS.md`

**Effort:** 1h  
**Owner:** Subagent 2  
**Status:** ⏳ Pending

---

### PS-5: Tokio Console Compatibility

**Task:** Verify Tokio console compatible with Rust 1.70

**Test:**
```bash
cargo add tokio-console
cargo build --features console
```

**File:** `harness/reports/TOKIO-CONSOLE-COMPAT.md`

**Effort:** 0.5h  
**Owner:** Subagent 3  
**Status:** ⏳ Pending

---

### PS-6: SARIF Schema Version Research

**Task:** Confirm SARIF version for GitHub Advanced Security

**Research:**
- GitHub recommends: 2.1.0
- Verify with GitHub docs

**File:** `harness/reports/SARIF-VERSION.md`

**Effort:** 0.5h  
**Owner:** Subagent 4  
**Status:** ⏳ Pending

---

## Revised Sprint Tasks

### Week 1: Adversarial Testing (Revised)

#### Phase 1: Mutation Engine (7h) - Was 6h

**Added Tasks:**
- **AT-1.15** (1h): Cross-file mutation strategy (Expert recommendation)
- **AT-1.16** (0.5h): Detector API compatibility tests

**Adjusted Priority:**
- P0: Unicode Substitution, Variable Renaming (Expert recommendation)
- P1: Encoding Variation
- P2: Control Flow, Dead Code, API Substitution
- P3: String Obfuscation

**Files to Create:**
- `glassware-core/src/adversarial/strategies/cross_file.rs` ← NEW

---

#### Phase 2: Fuzzer Engine (5h) - Was 4h

**Added Tasks:**
- **AT-2.13** (0.5h): Unicode edge case fuzzing (BOM, zero-width, RTL marks)
- **AT-2.14** (0.5h): Incremental testing (run after each strategy)

**Files to Create:**
- `glassware-core/src/adversarial/fuzz_strategies/unicode_edge_cases.rs` ← NEW

---

#### Phase 3: Polymorphic Generator (4h) - Unchanged

**No changes** - Template-based approach validated by expert

---

#### Phase 4: Test Generator + Integration (3h) - Was 2h

**Added Tasks:**
- **AT-4.5** (0.5h): Nightly CI/CD workflow (not PR)
- **AT-4.6** (0.5h): Incremental test integration (run during Phases 1-3)

**Files to Create:**
- `.github/workflows/adversarial-nightly.yml` ← NEW

---

### Week 2-3: Rust Orchestrator (Revised)

#### Phase 1: Core Infrastructure (9h) - Was 8h

**Structural Change:**
```
glassware-orchestrator/
├── Cargo.toml (workspace) ← NEW
├── orchestrator-core/     ← NEW (library)
├── orchestrator-cli/      ← RENAMED (was root)
└── orchestrator-benches/  ← NEW
```

**Added Tasks:**
- **RO-1.11** (0.5h): SQLite WAL mode + batch writes (Expert recommendation)
- **RO-1.12** (0.5h): API stability tests for orchestrator-core library

**Files to Create:**
- `glassware-orchestrator/Cargo.toml` (workspace)
- `glassware-orchestrator/orchestrator-core/Cargo.toml`
- `glassware-orchestrator/orchestrator-cli/Cargo.toml`
- `glassware-orchestrator/orchestrator-benches/Cargo.toml`

---

#### Phase 2: Advanced Features (9h) - Was 8h

**Added Tasks:**
- **RO-2.10** (0.5h): Streaming output (prevent OOM on 500+ packages)
- **RO-2.11** (0.5h): Python cache migration script

**Files to Create:**
- `harness/migrate_cache.py` ← NEW
- `glassware-orchestrator/orchestrator-core/src/streaming.rs` ← NEW

---

#### Phase 3: Performance & Polish (9h) - Was 8h

**Added Tasks:**
- **RO-3.11** (0.5h): Python→Rust cache migration testing
- **RO-3.12** (0.5h): Tokio console integration

**Adjusted Priority:**
- Benchmark Python harness FIRST (before optimization)
- Compare Rust vs Python with same workload

---

## Revised Timeline

### Pre-Sprint (4-8h) - NEW

| Task | Owner | Status |
|------|-------|--------|
| PS-1: Benchmark Python | Lead | ⏳ Pending |
| PS-2: SQLite Migration | Subagent 1 | ⏳ Pending |
| PS-3: API Stability | Lead | ✅ Complete |
| PS-4: npm Rate Limits | Subagent 2 | ⏳ Pending |
| PS-5: Tokio Console | Subagent 3 | ⏳ Pending |
| PS-6: SARIF Version | Subagent 4 | ⏳ Pending |

---

### Week 1: Adversarial (19h) - Was 16h

| Day | Tasks | Hours | Deliverables |
|-----|-------|-------|--------------|
| **Day 1** | PS-1 + AT-1.1 to AT-1.16 | 7h | Mutation engine + cross-file + benchmarks |
| **Day 2** | AT-2.1 to AT-2.14 | 5h | Fuzzer engine + Unicode edge cases |
| **Day 3** | AT-3.1 to AT-3.9 | 4h | Polymorphic generator |
| **Day 4** | AT-4.1 to AT-4.6 + Integration | 3h | Full framework + nightly CI/CD |

---

### Week 2-3: Orchestrator (27h) - Was 24h

| Day | Tasks | Hours | Deliverables |
|-----|-------|-------|--------------|
| **Day 5** | PS-2 + RO-1.1 to RO-1.12 | 9h | Core + workspace + WAL mode |
| **Day 6** | RO-2.1 to RO-2.11 | 9h | Advanced + streaming + migration |
| **Day 7** | RO-3.1 to RO-3.12 | 9h | Performance + benchmarks + docs |

---

## Revised Success Metrics

### Adversarial Testing

| Metric | Original | Revised | Rationale |
|--------|----------|---------|-----------|
| **Evasion Rate** | <10% | <10% | Unchanged |
| **Test Cases** | >100 | >120 | +20 for cross-file + Unicode edge cases |
| **Mutation Strategies** | 7 | 8 | +1 cross-file |
| **Fuzz Strategies** | 5 | 6 | +1 Unicode edge cases |
| **CI/CD Integration** | PR checks | Nightly + detector changes | Expert recommendation |

---

### Rust Orchestrator

| Metric | Original | Revised | Rationale |
|--------|----------|---------|-----------|
| **Speedup vs Python** | >2x | >2x | Unchanged |
| **Memory Usage** | <500MB | <500MB | Unchanged |
| **Error Rate** | <1% | <1% | Unchanged |
| **Cache Hit Rate** | >50% | >50% | Unchanged |
| **Cache Migration** | N/A | ✅ Working migration script | Expert recommendation |
| **Streaming Output** | N/A | ✅ Prevent OOM | Expert recommendation |

---

## Revised Risk Management

### New Risks Identified

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Cache migration breaks existing data | Low | High | Test migration on copy, backup first |
| Streaming adds complexity | Medium | Low | Implement early, test thoroughly |
| Workspace structure confusing | Low | Low | Clear documentation, examples |

---

### Updated Mitigations

**Cache Migration:**
```bash
# Before migration
cp .glassware-cache.json .glassware-cache.json.backup

# Run migration
python3 harness/migrate_cache.py

# Verify
sqlite3 orchestrator-cache.db "SELECT COUNT(*) FROM scan_results;"
```

**Streaming Output:**
```rust
// Implement from Day 1, not as optimization
let results = orchestrator
    .scan_packages(packages)
    .buffered(10)  // Stream, don't buffer
    .try_for_each(|result| {
        writeln!(output, "{}", serde_json::to_string(&result)?)?;
        Ok(())
    })
    .await?;
```

---

## Revised Resource Allocation

### Subagent Assignments (Updated)

| Subagent | Original | Revised |
|----------|----------|---------|
| **Subagent 1** | Mutation strategies | PS-2 (SQLite migration) + Mutation |
| **Subagent 2** | Fuzz strategies | PS-4 (npm limits) + Fuzz |
| **Subagent 3** | Polymorphic | PS-5 (Tokio console) + Polymorphic |
| **Subagent 4** | Test generator | PS-6 (SARIF) + Test gen |
| **Subagent 5** | Orchestrator core | Workspace structure + core |
| **Subagent 6** | Advanced features | Streaming + migration |
| **Subagent 7** | Performance | Benchmarks + optimization |

---

### Expert Engagement (Updated)

| Expert | Engagement Point | Topic |
|--------|-----------------|-------|
| **Expert A (Unicode)** | AT-2.13 | Unicode edge case fuzzing |
| **Expert B (Async Rust)** | RO-1.3, RO-2.10 | Tokio runtime + streaming |
| **Expert C (SQLite)** | RO-1.11, PS-2 | WAL mode + migration |
| **Expert D (Security)** | AT-1.15 | Cross-file mutation strategy |

---

## Revised Documentation Plan

### New Documents to Create

| Document | Purpose | Owner |
|----------|---------|-------|
| `harness/benchmarks/python-baseline.md` | Python baseline metrics | Lead |
| `harness/specs/CACHE-MIGRATION-SPEC.md` | Migration design | Subagent 1 |
| `harness/reports/API-STABILITY-REVIEW.md` | API stability verification | Lead |
| `harness/reports/NPM-RATE-LIMITS.md` | npm API rate limits | Subagent 2 |
| `harness/reports/TOKIO-CONSOLE-COMPAT.md` | Tokio console compatibility | Subagent 3 |
| `harness/reports/SARIF-VERSION.md` | SARIF version research | Subagent 4 |
| `harness/migrate_cache.py` | Python→Rust cache migration | Subagent 6 |

---

### Updated Documents

| Document | Changes | Owner |
|----------|---------|-------|
| `P3-SPRINT-TASKS.md` | +10 tasks, adjusted priorities | Lead |
| `P3-SPRINT-OVERVIEW.md` | Revised timeline, metrics | Lead |
| `ADVERSARIAL-TESTING-SPEC.md` | +2 strategies, nightly CI/CD | Lead |
| `RUST-ORCHESTRATOR-SPEC.md` | Workspace structure, streaming | Lead |

---

## Go/No-Go Criteria

### Pre-Sprint Checklist (Must Complete Before Day 1)

- [ ] PS-1: Python baseline benchmarked
- [ ] PS-2: SQLite migration designed
- [ ] PS-3: API stability verified ✅
- [ ] PS-4: npm rate limits tested
- [ ] PS-5: Tokio console verified
- [ ] PS-6: SARIF version confirmed

**Go/No-Go Decision:** If ≥4/6 complete → GO. If <4 → DELAY 1 day.

---

### Day 1 Checkpoint

- [ ] Mutation engine working with 3 P0 strategies
- [ ] Cross-file mutation strategy designed
- [ ] Python baseline recorded

**Go/No-Go Decision:** If all 3 complete → CONTINUE. If <2 → ADJUST scope.

---

## Final Recommendation

**Expert Review Status:** ✅ ALL RECOMMENDATIONS INCORPORATED

**Revised Sprint:**
- **Duration:** 46h (was 40h) + 6h pre-sprint = 52h total
- **Timeline:** 8 days (was 7 days)
- **Deliverables:** +10 tasks, +7 documents, +2 strategies

**Confidence Level:** 🟢 **VERY HIGH** - Expert feedback significantly improved plan

---

**Status:** ✅ REVISED & READY  
**Next:** Complete pre-sprint research (4-8h), then start Day 1

**Timestamp:** 2026-03-20 15:30 UTC  
**Author:** glassware AI Assistant
