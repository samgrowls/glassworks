# Questions for Previous Developer

**Document Version:** 1.0
**Date:** March 23, 2026
**Purpose:** Clarifications needed for binary consolidation and optimization

---

## Executive Summary

This document contains architectural decisions and implementation details that require context from the original developer. These questions arose during the consolidation planning phase.

**Priority Levels:**
- 🔴 **High** - Blocks consolidation, need answer before proceeding
- 🟡 **Medium** - Important for optimization, can proceed with assumptions
- 🟢 **Low** - Nice to know, can be decided during implementation

---

## Architecture Decisions

### 1. Caching Strategy Divergence 🔴

**Context:** glassware-cli uses JSON-based caching (`.glassware-cache.json`), while glassware-orchestrator uses SQLite (`.glassware-orchestrator-cache.db`).

**Questions:**
1. Why were two different caching implementations chosen?
2. Was SQLite chosen specifically for campaign checkpointing, or also for simple scan caching?
3. Should the unified `scan` command use SQLite or JSON caching?

**Current Understanding:**
- JSON cache: Simple key-value, file-based, easy to inspect
- SQLite cache: Better for concurrent access, supports complex queries, used for checkpoints

**Recommendation:** Use SQLite for unified caching (single implementation, better performance), but provide JSON export for inspection.

---

### 2. Parallelism Model Difference 🔴

**Context:** glassware-cli uses `rayon` (synchronous parallel iterators), while glassware-orchestrator uses `tokio` (async runtime).

**Questions:**
1. Was this divergence intentional or accidental?
2. Are there technical reasons why `scan` should use rayon vs tokio?
3. Should we standardize on one model for the unified binary?

**Current Understanding:**
- Rayon: Simpler for CPU-bound parallel file scanning
- Tokio: Necessary for async I/O (npm/GitHub APIs, LLM calls)

**Recommendation:** Keep rayon for `scan` command (CPU-bound file scanning), use tokio for `campaign` command (I/O-bound API calls).

---

### 3. TUI Feature Gating 🟡

**Context:** TUI is currently always compiled into glassware-orchestrator.

**Questions:**
1. Was there a reason TUI wasn't feature-gated?
2. Should TUI be optional in the unified binary?
3. Are there users who specifically want a TUI-less build?

**Current Understanding:**
- TUI adds ~2MB to binary size
- TUI dependencies (ratatui, crossterm) are stable
- TUI is only used for `campaign demo` and `campaign monitor`

**Recommendation:** Feature-gate TUI (`--features tui`) to allow smaller builds for CI/CD environments.

---

### 4. LLM Tier Architecture 🟡

**Context:** glassware-orchestrator has `--llm` (Tier 1) and `--deep-llm` (Tier 2), but glassware-cli only has `--llm`.

**Questions:**
1. Was Tier 2 intentionally excluded from CLI, or just not implemented yet?
2. Should `scan` command support both tiers?
3. Are there different environment variables for each tier?

**Current Understanding:**
- Tier 1: Cerebras (fast, ~2-5s/pkg)
- Tier 2: NVIDIA (deep analysis, ~15-30s/pkg)
- Environment: `GLASSWARE_LLM_BASE_URL` + `GLASSWARE_LLM_API_KEY` for Tier 1, `NVIDIA_API_KEY` for Tier 2

**Recommendation:** Add `--deep-llm` to scan command for consistency.

---

### 5. glassware-core Feature Usage 🔴

**Context:** glassware-cli uses `glassware-core` with `features = ["full", "binary"]`, while glassware-orchestrator uses `features = ["binary"]`.

**Questions:**
1. Why doesn't orchestrator use "full" features?
2. What features are in "full" that orchestrator doesn't need?
3. Should both use the same feature set?

**Current Understanding:**
- "full" includes: regex, serde, serde_json, lazy_static, once_cell, semantic, cache, binary
- Orchestrator may not need semantic analysis (oxc parser) for npm/GitHub scanning

**Recommendation:** Audit glassware-core features and use minimal required set for each command.

---

### 6. rusqlite + sqlx Duplication 🟡

**Context:** glassware-orchestrator uses both `sqlx` (async SQLite) and `rusqlite` (sync SQLite).

**Questions:**
1. Why are both SQLite libraries used?
2. Which components use sqlx vs rusqlite?
3. Can we consolidate to a single SQLite implementation?

**Current Understanding:**
- sqlx: Used for async checkpoint operations
- rusqlite: Used for simple cache operations
- Duplication adds ~2MB to binary

**Recommendation:** Evaluate using only rusqlite with async wrapper, or only sqlx throughout.

---

### 7. tokio "full" Features 🟡

**Context:** glassware-orchestrator uses `tokio = { version = "1.35", features = ["full"] }`.

**Questions:**
1. Were all tokio features actually needed?
2. Was there a conscious decision to use "full" vs selective features?
3. Which tokio features are actually used?

**Current Understanding:**
- "full" enables: rt, rt-multi-thread, macros, fs, io-util, net, signal, sync, time, process, parker, blocking
- Likely unused: process, parker, blocking

**Recommendation:** Use selective features to save ~0.5-1MB.

---

## Implementation Details

### 8. Scan Registry Purpose 🟢

**Context:** glassware-orchestrator has `scan_registry.rs` module.

**Questions:**
1. What is the purpose of the scan registry?
2. Is it used for campaign management or simple scanning?
3. Should it be preserved in the unified binary?

**Current Understanding:**
- Tracks scan history (scan-list, scan-show, scan-cancel commands)
- Stored in `.glassware-scan-registry.json`
- Only relevant for campaign-style scanning

**Recommendation:** Keep for campaign commands, not needed for simple scan.

---

### 9. Adversarial Testing Integration 🟢

**Context:** glassware-orchestrator has `adversarial.rs` module and `--adversarial` flag.

**Questions:**
1. How is adversarial testing used in practice?
2. Should it be available for simple `scan` command?
3. Is it production-ready or experimental?

**Current Understanding:**
- Mutation and fuzzing engines for testing detector resilience
- Used for detector development, not typical scanning
- Adds complexity and dependencies

**Recommendation:** Keep as campaign-only feature, feature-gate as `adversarial`.

---

### 10. Version Scanner Usage 🟢

**Context:** glassware-orchestrator has `version_scanner.rs` for scanning multiple versions of npm packages.

**Questions:**
1. Is version scanning used in production campaigns?
2. Should it be available as a standalone command?
3. How does it integrate with campaigns?

**Current Understanding:**
- Scans multiple versions of a package (e.g., last 10 versions)
- Used via `scan-npm --versions <policy>`
- Policy options: last-10, last-180d, all, comma-separated

**Recommendation:** Keep as campaign feature, consider adding to scan command.

---

### 11. Sampler Module Purpose 🟢

**Context:** glassware-orchestrator has `sampler.rs` for sampling npm packages by category.

**Questions:**
1. Is the sampler used in production?
2. Should it be a standalone command or campaign-only?
3. How are categories defined?

**Current Understanding:**
- `sample-packages` command for generating package lists
- Categories: ai-ml, native-build, install-scripts, etc.
- Used for campaign preparation

**Recommendation:** Keep as campaign utility command.

---

### 12. Config System Details 🟡

**Context:** glassware-orchestrator has `config.rs` for TOML-based configuration.

**Questions:**
1. Where is the config file stored?
2. What settings are configurable?
3. Should config apply to `scan` command as well?

**Current Understanding:**
- Config locations: `~/.config/glassware/config.toml`, `./glassware.toml`
- Settings: scoring thresholds, performance, output format
- Used primarily for campaigns

**Recommendation:** Support config for both scan and campaign commands.

---

## Optimization Questions

### 13. Current Performance Bottlenecks 🟡

**Questions:**
1. What are the known performance bottlenecks?
2. Have there been any profiling results?
3. Are there any "known slow" code paths?

**Current Understanding:**
- Scan speed: ~50k LOC/sec
- Memory: ~50MB during scan
- No PGO currently enabled

**Recommendation:** Profile before and after optimization to measure impact.

---

### 14. Memory Usage Patterns 🟡

**Questions:**
1. What are the largest memory consumers during scanning?
2. Is memory usage proportional to codebase size?
3. Are there any memory leaks observed in long runs?

**Current Understanding:**
- Cache loaded into memory
- Results buffered before output
- TUI state in memory

**Recommendation:** Use memory profiling tools (valgrind, heaptrack) to identify hotspots.

---

### 15. Build Time Concerns 🟢

**Questions:**
1. What is the current release build time?
2. Is build time a concern for CI/CD?
3. Are there any slow-to-compile dependencies?

**Current Understanding:**
- LTO will increase build time by 50-100%
- Acceptable for release builds (debug remains fast)
- CI/CD may need adjustment

**Recommendation:** Consider separate CI jobs for release builds.

---

## Testing Questions

### 16. Test Coverage Gaps 🟡

**Questions:**
1. Are there any untested code paths?
2. What is the overall test coverage percentage?
3. Are there integration tests for campaigns?

**Current Understanding:**
- ~40+ unit tests in orchestrator
- Integration tests in glassware-core
- No end-to-end campaign tests identified

**Recommendation:** Add integration tests for consolidated binary.

---

### 17. Test Data Management 🟢

**Questions:**
1. Where is test data stored?
2. Are there fixtures for malicious packages?
3. How are false positives tracked?

**Current Understanding:**
- Test fixtures in glassware-core/tests/
- Wave 6 campaign used for calibration
- Evidence stored in `evidence/` directory

**Recommendation:** Preserve test fixtures during consolidation.

---

## Migration Questions

### 18. User Base Impact 🔴

**Questions:**
1. How many users currently use glassware-cli?
2. How many use glassware-orchestrator?
3. Are there any scripts/CI pipelines that depend on specific binary names?

**Current Understanding:**
- glassware-cli: Simple scanning use case
- glassware-orchestrator: Campaign scanning, security research
- Unknown: External dependencies on binary names

**Recommendation:** Provide compatibility wrappers or aliases during deprecation period.

---

### 19. Documentation Dependencies 🟡

**Questions:**
1. Which documentation files reference binary names?
2. Are there external docs (website, README on other repos)?
3. Should we update all docs before or after release?

**Current Understanding:**
- docs/ directory has user guides
- HANDOFF/ has developer docs
- README.md has command examples

**Recommendation:** Update all internal docs before release, provide migration guide.

---

### 20. Release Strategy 🟡

**Questions:**
1. Should consolidation be a major version bump (v1.0)?
2. Should there be a deprecation release first (v0.9 with warnings)?
3. How long should the deprecation period be?

**Current Understanding:**
- Current version: v0.8.0 (workspace)
- Consolidation is significant but not breaking (commands preserved)
- Deprecation period: 1 release cycle recommended

**Recommendation:** v0.9.0 with deprecation, v1.0.0 with removal.

---

## Summary of Critical Questions

**Must Answer Before Proceeding (🔴 High Priority):**

1. **Caching Strategy** - JSON vs SQLite for unified caching
2. **Parallelism Model** - Rayon vs tokio for scan command
3. **glassware-core Features** - Why different feature sets?
4. **User Base Impact** - External dependencies on binary names

**Can Proceed with Assumptions (🟡 Medium Priority):**

5. TUI feature gating
6. LLM tier architecture
7. rusqlite + sqlx duplication
8. tokio "full" features
9. Config system details
10. Performance bottlenecks
11. Memory usage patterns
12. Test coverage gaps
13. Documentation dependencies
14. Release strategy

**Nice to Know (🟢 Low Priority):**

15. Scan registry purpose
16. Adversarial testing integration
17. Version scanner usage
18. Sampler module purpose
19. Build time concerns
20. Test data management

---

## Assumptions for Implementation

If answers are not available, the following assumptions will be used:

| Question | Assumption |
|----------|------------|
| Caching | Use SQLite for unified caching |
| Parallelism | Keep rayon for scan, tokio for campaign |
| TUI | Feature-gate TUI |
| LLM tiers | Add --deep-llm to scan command |
| glassware-core | Audit and use minimal features |
| SQLite libs | Evaluate consolidation to single lib |
| tokio | Use selective features |
| Config | Support for both scan and campaign |
| Version | v0.9.0 deprecation, v1.0.0 removal |

---

**Please review these questions and provide context where possible. For urgent questions (🔴), a quick response is appreciated to avoid blocking progress.**
