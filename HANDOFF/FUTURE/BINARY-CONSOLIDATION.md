# Binary Consolidation Analysis

**Date:** March 23, 2026
**Question:** How many Rust binaries do we produce? Can we consolidate into one?

---

## Current State

### Binaries Produced

| Binary | Location | Size | Purpose |
|--------|----------|------|---------|
| **glassware** | `glassware-cli/src/main.rs` | ~11MB | Simple directory/file scanner |
| **glassware-orchestrator** | `glassware-orchestrator/src/main.rs` | ~25MB | Full campaign orchestration |

**Total:** 2 binaries

### Workspace Structure

```
glassworks/
├── glassware-core/          # Library (no binary)
│   └── src/lib.rs           # Detection engine
│
├── glassware-cli/           # Binary 1: Simple scanner
│   └── src/main.rs          # ~11MB binary
│
└── glassware-orchestrator/  # Binary 2: Campaign orchestrator
    └── src/main.rs          # ~25MB binary
```

---

## Consolidation Analysis

### Recommendation: **YES - Consolidate into One Binary**

**Rationale:**
1. **Shared functionality:** Both use `glassware-core` detectors
2. **Overlapping dependencies:** Both use clap, tokio, serde, etc.
3. **User confusion:** Two binaries with similar purposes
4. **Maintenance burden:** Two binaries to build, test, document
5. **Feature parity:** Orchestrator has all CLI features + more

### Proposed Architecture

```
glassworks/
├── glassware-core/          # Library (unchanged)
│   └── src/lib.rs           # Detection engine
│
└── glassware/               # Single unified binary
    └── src/
        ├── main.rs          # CLI entry point
        ├── commands/
        │   ├── scan.rs      # Simple scan (old glassware-cli)
        │   ├── campaign.rs  # Campaign commands (old orchestrator)
        │   └── ...
        └── ...
```

### Migration Path

**Phase 1: Feature Parity Check (1 week)**
- Audit `glassware-cli` features
- Ensure `glassware-orchestrator` has all features
- Document any gaps

**Phase 2: Code Consolidation (1 week)**
- Move `glassware-orchestrator` to root `glassware/`
- Add `scan` subcommand for simple scanning
- Keep `campaign` subcommand for campaigns
- Update workspace structure

**Phase 3: Deprecation (1 week)**
- Mark `glassware-cli` as deprecated
- Update documentation
- Update CI/CD pipelines

**Phase 4: Removal (after 1 release cycle)**
- Remove `glassware-cli` crate
- Clean up workspace

---

## Optimization Opportunities

### Current Binary Size: ~25MB

**Breakdown (estimated):**
- `tokio` (async runtime): ~3MB
- `clap` (CLI parsing): ~1MB
- `sqlx` (SQLite): ~2MB
- `rusqlite`: ~2MB
- `ratatui` (TUI): ~2MB
- Detection engine + dependencies: ~10MB
- Debug symbols: ~5MB

### Optimization Strategies

#### 1. Release Profile Optimization

**Current:**
```toml
[profile.release]
# Default settings
```

**Optimized:**
```toml
[profile.release]
opt-level = 3           # Full optimization
lto = true              # Link-time optimization
codegen-units = 1       # Single codegen unit
strip = true            # Strip debug symbols
panic = "abort"         # Smaller panic handling
```

**Expected savings:** 30-40% size reduction (~15MB → ~9MB)

#### 2. Feature Flags

**Current:** All features always enabled

**Proposed:**
```toml
[features]
default = ["tui", "llm", "sqlite"]
tui = ["ratatui", "crossterm"]
llm = ["reqwest"]
sqlite = ["sqlx", "rusqlite"]
minimal = []  # No TUI, no LLM, no SQLite
```

**Expected savings:** 
- Minimal build: ~8MB
- Default build: ~15MB (with optimizations)

#### 3. Dependency Audit

**Heavy dependencies to review:**
- `sqlx` (2MB) - Could use `rusqlite` directly
- `ratatui` (2MB) - Only needed for TUI
- `reqwest` (1.5MB) - Only needed for LLM/GitHub

**Savings:** ~3-5MB with careful dependency management

#### 4. Memory Optimization

**Current memory usage:** ~50MB during scan

**Optimization opportunities:**
- Stream results instead of buffering
- Reduce concurrency if memory-bound
- Use `Box<dyn Trait>` for large structs
- Lazy loading for large data structures

**Expected savings:** 30-50% memory reduction

#### 5. Speed Optimization

**Current scan speed:** ~50k LOC/sec

**Optimization opportunities:**
- Parallel file reading (already using rayon)
- Better regex compilation caching
- Reduce allocations in hot paths
- Profile-guided optimization (PGO)

**Expected improvement:** 20-30% faster

---

## Implementation Plan

### Week 1: Binary Consolidation
- [ ] Audit features in both binaries
- [ ] Move orchestrator to root `glassware/`
- [ ] Add `scan` subcommand
- [ ] Update workspace structure
- [ ] Test all functionality

### Week 2: Size Optimization
- [ ] Enable LTO and strip symbols
- [ ] Add feature flags
- [ ] Audit dependencies
- [ ] Measure size reduction

### Week 3: Performance Optimization
- [ ] Profile memory usage
- [ ] Profile CPU hotspots
- [ ] Implement streaming where possible
- [ ] Enable PGO
- [ ] Benchmark improvements

---

## Expected Outcomes

### Before Consolidation
| Metric | Value |
|--------|-------|
| Binaries | 2 (glassware, glassware-orchestrator) |
| Total size | ~36MB (11MB + 25MB) |
| Memory usage | ~50MB |
| Scan speed | ~50k LOC/sec |

### After Consolidation + Optimization
| Metric | Target | Improvement |
|--------|--------|-------------|
| Binaries | 1 (glassware) | -50% |
| Total size | ~10-15MB | -60% |
| Memory usage | ~25-35MB | -40% |
| Scan speed | ~65k LOC/sec | +30% |

---

## Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking changes for users | Medium | Deprecation period, clear migration guide |
| Feature regression | Medium | Comprehensive testing, beta release |
| Build time increase | Low | LTO increases build time, but only affects releases |
| Dependency conflicts | Low | Careful dependency audit |

---

## Recommendation

**Proceed with consolidation and optimization.**

**Rationale:**
1. **User experience:** One binary is simpler
2. **Maintenance:** One codebase to maintain
3. **Performance:** Significant improvements possible
4. **Distribution:** Smaller download size
5. **Memory:** Lower resource usage for long runs

**Timeline:** 3 weeks total
**Resources:** 1 developer
**Risk:** Low (incremental migration)

---

## Next Steps

1. **Review this analysis** with team
2. **Create GitHub issue** for consolidation
3. **Set up tracking** for optimization metrics
4. **Begin Phase 1** (feature audit)

---

**This is a high-value initiative that will improve user experience, reduce maintenance burden, and improve performance across the board.**
