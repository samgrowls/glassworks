# Binary Consolidation Implementation Tracker

**Status:** 🟡 Planning Complete - Ready to Start
**Start Date:** March 23, 2026
**Target Completion:** April 13, 2026 (3 weeks)
**Current Week:** Week 1 (Planning)

---

## Progress Summary

| Week | Focus | Status | Completion |
|------|-------|--------|------------|
| **Week 1** | Feature audit, code consolidation | 🟡 In Progress | 80% |
| **Week 2** | Size optimization | ⚪ Pending | 0% |
| **Week 3** | Performance optimization | ⚪ Pending | 0% |

---

## Week 1: Feature Audit & Code Consolidation

### Phase 1.1: Feature Audit ✅ COMPLETE

**Goal:** Understand all features in both binaries

- [x] Read glassware-cli/src/main.rs (1,019 lines)
- [x] Inventory all CLI commands and flags
- [x] Read glassware-orchestrator/src/main.rs (1,889 lines)
- [x] Inventory all orchestrator commands
- [x] Create feature comparison table
- [x] Identify gaps (CLI features missing in orchestrator)

**Findings:**
- CLI has simpler UX for file scanning
- CLI missing: JSON Lines output, Tier 2 LLM, concurrency control
- Orchestrator has all advanced features (campaigns, TUI, npm/GitHub)

**Documentation:** See `CONSOLIDATION-PLAN.md` Section 1.1

---

### Phase 1.2: Dependency Analysis ✅ COMPLETE

**Goal:** Understand overlapping and heavy dependencies

- [x] Analyze glassware-cli/Cargo.toml
- [x] Analyze glassware-orchestrator/Cargo.toml
- [x] Identify overlapping dependencies (8 shared)
- [x] Identify CLI-only dependencies (rayon, ignore, colored)
- [x] Identify orchestrator-only dependencies (tokio, sqlx, ratatui, etc.)
- [x] Estimate size impact of heavy dependencies

**Key Findings:**
- tokio (full): ~3MB → can reduce with selective features
- sqlx + rusqlite: ~4MB → potential consolidation
- ratatui + crossterm: ~2MB → can be feature-gated
- No `[profile.release]` optimization in any crate

**Documentation:** See `CONSOLIDATION-PLAN.md` Section 1.3

---

### Phase 1.3: Code Structure Analysis ✅ COMPLETE

**Goal:** Understand code organization for migration

- [x] Map glassware-cli structure (single file)
- [x] Map glassware-orchestrator structure (modular)
- [x] Review campaign module structure (11 modules)
- [x] Review TUI structure (app.rs, ui.rs)
- [x] Identify shared vs unique code paths

**Key Findings:**
- CLI: Simple rayon-based parallel scanning
- Orchestrator: Async tokio-based with DAG scheduling
- Different caching: JSON (CLI) vs SQLite (orchestrator)

**Documentation:** See `CONSOLIDATION-PLAN.md` Section 1.4

---

### Phase 1.4: Build Configuration Review ✅ COMPLETE

**Goal:** Identify optimization opportunities

- [x] Review workspace Cargo.toml
- [x] Review all crate Cargo.toml files
- [x] Check for `[profile.release]` settings
- [x] Review feature flag definitions
- [x] Identify optimization gaps

**Key Findings:**
- NO `[profile.release]` in any Cargo.toml
- Using Rust defaults (missed optimization opportunity)
- Feature flags exist but not fully utilized

**Recommendations:** See `CONSOLIDATION-PLAN.md` Section 3.1

---

### Phase 1.5: Questions for Previous Developer ✅ COMPLETE

**Goal:** Clarify architectural decisions

- [x] Create QUESTIONS.md document
- [x] List 20 questions organized by priority
- [x] Document assumptions for unanswered questions
- [x] Identify 4 critical blocking questions

**Critical Questions (🔴):**
1. Caching strategy divergence (JSON vs SQLite)
2. Parallelism model difference (rayon vs tokio)
3. glassware-core feature usage
4. User base impact on binary names

**Documentation:** See `QUESTIONS.md`

---

### Phase 1.6: Workspace Restructuring Plan 🟡 IN PROGRESS

**Goal:** Define target structure

- [x] Create target directory structure
- [ ] Decide on binary naming
- [ ] Plan module migration order
- [ ] Create migration checklist
- [ ] Set up tracking for breaking changes

**Target Structure:**
```
glassworks/
├── glassware-core/          # Library (unchanged)
└── glassware/               # Unified binary (renamed)
    ├── src/commands/scan.rs    # From glassware-cli
    ├── src/commands/campaign.rs # Existing
    ├── src/tui/               # Existing
    └── ...                    # Existing modules
```

---

### Phase 1.7: Code Migration ⚪ PENDING

**Goal:** Move glassware-cli code into unified binary

- [ ] Create `src/commands/` directory
- [ ] Create `src/commands/mod.rs`
- [ ] Migrate `scan` command from glassware-cli
- [ ] Integrate file walking (ignore crate)
- [ ] Integrate parallel scanning (rayon)
- [ ] Integrate output formatters (pretty, json, sarif)
- [ ] Add JSON Lines output (gap fill)
- [ ] Add `--deep-llm` flag (gap fill)
- [ ] Add concurrency control (gap fill)
- [ ] Test `glassware scan` functionality

**Estimated Effort:** 3-4 days

---

### Phase 1.8: Deprecation Strategy ⚪ PENDING

**Goal:** Plan deprecation of old binaries

- [ ] Create deprecation timeline
- [ ] Write deprecation notices for glassware-cli
- [ ] Write deprecation notices for glassware-orchestrator
- [ ] Update README with new commands
- [ ] Create migration guide for users
- [ ] Update CI/CD pipelines

**Timeline:**
- v0.9.0: Deprecation warnings
- v1.0.0: Remove old binaries

---

### Phase 1.9: Testing Strategy ⚪ PENDING

**Goal:** Ensure all tests pass after consolidation

- [ ] Run existing glassware-core tests
- [ ] Run existing orchestrator tests
- [ ] Create tests for new `scan` command
- [ ] Create integration tests for unified binary
- [ ] Test all CLI subcommands
- [ ] Test TUI functionality
- [ ] Test campaign functionality
- [ ] Performance benchmarking (before/after)

---

### Phase 1.10: Documentation Updates ⚪ PENDING

**Goal:** Update all documentation for new structure

- [ ] Update README.md with new commands
- [ ] Update QWEN.md
- [ ] Update docs/CAMPAIGN-USER-GUIDE.md
- [ ] Update HANDOFF/README.md
- [ ] Create migration guide
- [ ] Update examples/
- [ ] Update website/ (if exists)

---

## Week 2: Size Optimization

### Phase 2.1: Release Profile Optimization ⚪ PENDING

**Goal:** Configure optimal release build

- [ ] Add `[profile.release]` to workspace Cargo.toml
- [ ] Enable LTO (link-time optimization)
- [ ] Set codegen-units = 1
- [ ] Enable strip (remove debug symbols)
- [ ] Set panic = "abort"
- [ ] Measure size reduction
- [ ] Test functionality with optimized build

**Target Configuration:**
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

**Expected Savings:** 30-40% (~25MB → ~15MB)

---

### Phase 2.2: Feature Flag Design ⚪ PENDING

**Goal:** Make optional features truly optional

- [ ] Design feature flag hierarchy
- [ ] Feature-gate TUI (`--features tui`)
- [ ] Feature-gate LLM (`--features llm`)
- [ ] Feature-gate reports (`--features reports`)
- [ ] Create `minimal` feature set
- [ ] Test builds with different feature combinations
- [ ] Document feature flags for users

**Proposed Features:**
```toml
[features]
default = ["tui", "llm", "reports"]
tui = ["ratatui", "crossterm"]
llm = ["reqwest"]
reports = ["tera"]
minimal = []
```

**Expected Savings:** Minimal build ~8MB

---

### Phase 2.3: Dependency Optimization ⚪ PENDING

**Goal:** Reduce dependency bloat

- [ ] Audit tokio features (use selective instead of "full")
- [ ] Evaluate sqlx vs rusqlite consolidation
- [ ] Remove unused dependencies
- [ ] Update to latest versions (may have size improvements)
- [ ] Consider lighter alternatives where possible
- [ ] Measure size impact of each change

**Targets:**
- tokio: Selective features (-0.5-1MB)
- SQLite: Single implementation (-2MB)
- Other: Cleanup (-1-2MB)

---

### Phase 2.4: Code Size Analysis ⚪ PENDING

**Goal:** Identify code bloat

- [ ] Use `cargo-bloat` to analyze binary
- [ ] Identify largest functions
- [ ] Identify dead code elimination opportunities
- [ ] Review monomorphization issues (generics)
- [ ] Optimize or remove bloated code
- [ ] Measure impact

**Tools:**
```bash
cargo install cargo-bloat
cargo bloat --release --crates
cargo bloat --release --filter-function
```

---

### Phase 2.5: Build Measurement ⚪ PENDING

**Goal:** Track size metrics

- [ ] Create size tracking spreadsheet
- [ ] Measure binary size after each optimization
- [ ] Document what worked/didn't work
- [ ] Create before/after comparison table

**Metrics to Track:**
- Binary size (release)
- Build time
- Dependency count
- Crate sizes

---

## Week 3: Performance Optimization

### Phase 3.1: Memory Profiling ⚪ PENDING

**Goal:** Identify memory hotspots

- [ ] Profile memory usage during scan
- [ ] Profile memory usage during campaign
- [ ] Identify largest allocations
- [ ] Identify memory leaks (if any)
- [ ] Create memory optimization plan

**Tools:**
- valgrind (massif)
- heaptrack
- `/usr/bin/time -v`

---

### Phase 3.2: CPU Profiling ⚪ PENDING

**Goal:** Identify CPU bottlenecks

- [ ] Profile CPU usage during scan
- [ ] Profile CPU usage during campaign
- [ ] Identify hot functions
- [ ] Measure time in detectors
- [ ] Create CPU optimization plan

**Tools:**
- perf (Linux)
- Instruments (macOS)
- samply

---

### Phase 3.3: Streaming Implementation ⚪ PENDING

**Goal:** Reduce memory buffering

- [ ] Audit current buffering behavior
- [ ] Implement streaming for file scanning
- [ ] Implement streaming for output
- [ ] Implement streaming for LLM responses
- [ ] Test memory reduction

**Expected Savings:** 20-30% memory reduction

---

### Phase 3.4: Parallelism Optimization ⚪ PENDING

**Goal:** Improve scan speed

- [ ] Profile rayon usage in scan command
- [ ] Profile tokio usage in campaign command
- [ ] Optimize worker pool sizes
- [ ] Reduce contention in parallel code
- [ ] Test speed improvements

**Target:** 65k LOC/sec (+30%)

---

### Phase 3.5: PGO (Profile-Guided Optimization) ⚪ PENDING

**Goal:** Enable PGO for release builds

- [ ] Set up PGO build pipeline
- [ ] Generate profiling data
- [ ] Build with PGO
- [ ] Measure performance improvement
- [ ] Document PGO setup

**Expected Improvement:** 10-15% speedup

---

### Phase 3.6: Final Benchmarking ⚪ PENDING

**Goal:** Measure all improvements

- [ ] Binary size benchmark
- [ ] Memory usage benchmark
- [ ] Scan speed benchmark
- [ ] Campaign execution benchmark
- [ ] Create before/after comparison

**Success Criteria:**
- [ ] Binary size: 10-15MB (was ~36MB)
- [ ] Memory: 25-35MB (was ~50MB)
- [ ] Scan speed: ~65k LOC/sec (was ~50k LOC/sec)

---

## Risks & Mitigation

| Risk | Impact | Probability | Mitigation | Status |
|------|--------|-------------|------------|--------|
| Breaking changes for users | High | Medium | Deprecation period, migration guide | 🟡 Monitoring |
| Feature regression | Medium | Low | Comprehensive testing | 🟢 On track |
| Build time increase | Low | High | Acceptable for release builds | 🟢 Accepted |
| Dependency conflicts | Medium | Low | Careful audit | 🟢 On track |
| Performance regression | High | Low | Benchmarking at each step | 🟢 On track |

---

## Metrics Dashboard

### Current State (Before Consolidation)

| Metric | glassware-cli | glassware-orchestrator | Total |
|--------|---------------|------------------------|-------|
| Binary Size | ~11MB | ~25MB | ~36MB |
| Memory Usage | ~30MB | ~50MB | - |
| Scan Speed | ~50k LOC/s | ~50k LOC/s | - |
| Build Time (release) | ~90s | ~120s | - |

### Target State (After Consolidation)

| Metric | Target | Improvement |
|--------|--------|-------------|
| Binary Size | 10-15MB | -60% |
| Memory Usage | 25-35MB | -40% |
| Scan Speed | ~65k LOC/s | +30% |
| Build Time (release) | ~180s (with LTO) | +50% (acceptable) |

---

## Decision Log

### 2026-03-23: Planning Phase Complete

**Decisions Made:**
1. ✅ Proceed with consolidation (confirmed from BINARY-CONSOLIDATION.md)
2. ✅ Target structure: Rename glassware-orchestrator to glassware
3. ✅ Add `scan` subcommand for simple file scanning
4. ✅ Keep rayon for scan command (CPU-bound)
5. ✅ Keep tokio for campaign command (I/O-bound)
6. ✅ Use SQLite for unified caching (deprecate JSON cache)
7. ✅ Feature-gate TUI, LLM, reports
8. ✅ Enable LTO, strip, codegen-units=1, panic=abort

**Pending Decisions (awaiting answers):**
1. ⏳ glassware-core feature usage (question sent to previous developer)
2. ⏳ rusqlite vs sqlx consolidation (needs investigation)
3. ⏳ tokio selective features (needs audit)

---

## Next Actions

### Immediate (This Week)

1. [ ] Receive answers to critical questions
2. [ ] Finalize workspace restructuring plan
3. [ ] Begin code migration (Phase 1.7)
4. [ ] Create `src/commands/` directory structure

### This Sprint (Week 1)

1. [ ] Complete code migration
2. [ ] Test all functionality
3. [ ] Create deprecation notices
4. [ ] Update documentation

---

## Notes

- **Context Conservation:** Using subagents for research was effective
- **Documentation:** Comprehensive planning prevents rework
- **Questions:** 20 questions created, 4 critical - awaiting answers
- **Risk Level:** Low - incremental migration with rollback capability

---

**Last Updated:** March 23, 2026
**Next Review:** March 25, 2026
