# Binary Consolidation - Getting Started

**Start Here!** 🚀

**Date:** March 23, 2026
**Status:** ✅ Ready to Implement

---

## Quick Summary

### What Happened

1. **Research Phase:** A subagent conducted analysis of the codebase
2. **Misconceptions:** The analysis contained several errors about "missing features"
3. **Correction:** Previous developer provided clarifications in `RESPONSE-TO-AGENT.md`
4. **Revised Plan:** Simple rename of `glassware-orchestrator` to `glassware`

### The Truth

**`glassware-orchestrator` already has ALL features:**
- ✅ Campaign orchestration with checkpoint/resume
- ✅ TUI with live monitoring and drill-down  
- ✅ LLM Tier 1 (Cerebras) and Tier 2 (NVIDIA)
- ✅ All output formats (JSON, JSONL, SARIF)
- ✅ Concurrency control, rate limiting
- ✅ SQLite caching, adversarial testing

**`glassware-cli` is just a simple scanner:**
- Single-file implementation (~1,000 lines)
- Uses rayon for parallel file scanning
- No campaigns, no TUI, no advanced features

### The Plan (Simplified)

```
Week 1: Rename + Deprecate (3-4 days)
  ├─ Rename glassware-orchestrator → glassware
  ├─ Add deprecation warning to glassware-cli
  ├─ Update workspace configuration
  └─ Test all commands

Week 2: Optimization (5 days)
  ├─ Enable LTO + strip symbols
  ├─ Feature-gate TUI
  ├─ Selective tokio features
  └─ Measure & benchmark
```

**Timeline:** 1.5-2 weeks (not 3 weeks as originally estimated)

---

## Reading Order

### Step 1: Read the Correction (10 min)

**File:** [`RESPONSE-TO-AGENT.md`](./RESPONSE-TO-AGENT.md)

This is the previous developer's clarification. It corrects all the misconceptions from the initial analysis.

**Key points:**
- glassware-orchestrator has ALL features
- Architecture divergence (rayon vs tokio, JSON vs SQLite) is intentional
- Consolidation is just a rename operation

---

### Step 2: Read the Revised Plan (20 min)

**File:** [`CONSOLIDATION-PLAN-REVISED.md`](./CONSOLIDATION-PLAN-REVISED.md)

This is the corrected implementation plan. It's much simpler than the original because we're just renaming, not migrating code.

**Sections to focus on:**
- Section 2: Implementation Plan (Week 1: Rename & Deprecate)
- Section 3: Optimization Plan (Week 2)
- Section 4: Testing Checklist

---

### Step 3: Review Executive Summary (5 min)

**File:** [`EXECUTIVE-SUMMARY.md`](./EXECUTIVE-SUMMARY.md)

High-level overview with metrics and timeline.

---

### Step 4: Start Implementation

**File:** [`WORKSPACE-RESTRUCTURING.md`](./WORKSPACE-RESTRUCTURING.md)

Follow the step-by-step instructions. Most steps are simple renames and config updates.

---

## Implementation Checklist

### Week 1: Rename & Deprecate

- [ ] **Step 1:** Rename `glassware-orchestrator/` → `glassware/`
- [ ] **Step 2:** Update `glassware/Cargo.toml` (package name)
- [ ] **Step 3:** Update root `Cargo.toml` (workspace members)
- [ ] **Step 4:** Add deprecation warning to `glassware-cli/src/main.rs`
- [ ] **Step 5:** Update CLI help text in `glassware/src/cli.rs`
- [ ] **Step 6:** Optional - Add `scan` subcommand for CLI migration
- [ ] **Step 7:** Build and test all commands
- [ ] **Step 8:** Create `DEPRECATION-NOTICE.md`
- [ ] **Step 9:** Update README.md and QWEN.md
- [ ] **Step 10:** Tag v0.9.0 and push to remote

### Week 2: Optimization

- [ ] **Step 1:** Add `[profile.release]` to root Cargo.toml
- [ ] **Step 2:** Feature-gate TUI in `glassware/Cargo.toml`
- [ ] **Step 3:** Use selective tokio features
- [ ] **Step 4:** Evaluate rusqlite/sqlx consolidation (optional)
- [ ] **Step 5:** Measure binary size, memory, speed
- [ ] **Step 6:** Tag v1.0.0 and push to remote

---

## Commands to Test

After renaming, test these commands:

```bash
# Build
cargo build --release -p glassware

# Help
./target/release/glassware --help
./target/release/glassware campaign --help

# Campaign
./target/release/glassware campaign run campaigns/wave6.toml
./target/release/glassware campaign demo
./target/release/glassware campaign run wave6.toml --llm
./target/release/glassware campaign run wave6.toml --deep-llm
./target/release/glassware campaign run wave6.toml --format jsonl
./target/release/glassware campaign run wave6.toml --concurrency 20

# Scan
./target/release/glassware scan-file /path/to/code
./target/release/glassware scan-npm express@4.19.2
./target/release/glassware scan-github org/repo

# Query
./target/release/glassware campaign query <case-id> "question"

# Tests
cargo test -p glassware
```

---

## Expected Outcomes

### After v0.9.0 (Rename Only)

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Binaries | 2 | 1 | -50% |
| Binary Size | ~25MB (orchestrator) | ~25MB (glassware) | No change |
| Memory | ~50MB | ~50MB | No change |
| Scan Speed | ~50k LOC/s | ~50k LOC/s | No change |

**Goal:** Functional parity, no breaking changes

### After v1.0.0 (With Optimization)

| Metric | Before | Target | Change |
|--------|--------|--------|--------|
| Binary Size | ~25MB | ~10-15MB | -40-60% |
| Memory | ~50MB | ~25-35MB | -30-50% |
| Scan Speed | ~50k LOC/s | ~60-65k LOC/s | +20-30% |

**Goal:** Optimized build with LTO, feature-gating, selective deps

---

## Git Strategy

### Tagging

```bash
# Before starting
git checkout -b feature/binary-consolidation

# After Week 1 (rename complete, tested)
git tag v0.9.0-consolidated
git push origin v0.9.0-consolidated

# After Week 2 (optimizations complete)
git tag v1.0.0-unified
git push origin v1.0.0-unified
```

### Commit Messages

Use clear, descriptive commit messages:

```bash
# Week 1
git commit -m "Rename glassware-orchestrator to glassware"
git commit -m "Add deprecation warning to glassware-cli"
git commit -m "Update workspace configuration for unified binary"
git commit -m "Update CLI help text and documentation"

# Week 2
git commit -m "Add release profile with LTO and strip"
git commit -m "Feature-gate TUI support"
git commit -m "Use selective tokio features to reduce size"
```

---

## Questions?

### If You're Unsure About Something

1. **Check `RESPONSE-TO-AGENT.md`** - Previous developer's clarifications
2. **Check `CONSOLIDATION-PLAN-REVISED.md`** - Detailed implementation guide
3. **Check `QUESTIONS.md`** - Original questions (now answered)

### Critical Questions (Already Answered)

| Question | Answer |
|----------|--------|
| Caching strategy (JSON vs SQLite)? | Intentional - keep both |
| Parallelism (rayon vs tokio)? | Intentional - keep both |
| glassware-core features? | Use minimal (`binary` only) |
| User base impact? | Provide deprecation period |

---

## Success Criteria

You're done when:

- [ ] `glassware` binary works for all use cases
- [ ] `glassware-cli` shows deprecation warning
- [ ] All tests pass
- [ ] Binary size is acceptable
- [ ] Documentation is updated
- [ ] v0.9.0 tag is pushed
- [ ] v1.0.0 tag is pushed (after optimization)

---

## Next Action

**Right now, do this:**

1. Read `RESPONSE-TO-AGENT.md` (10 min)
2. Read `CONSOLIDATION-PLAN-REVISED.md` Section 1-2 (15 min)
3. Start Week 1, Step 1: Rename the directory

```bash
mv glassware-orchestrator glassware
```

Good luck! The plan is solid, the documentation is comprehensive, and the previous developer is available for questions if needed. 🚀
