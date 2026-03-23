# Binary Consolidation Plan - REVISED

**Version:** 2.0 (Corrected)
**Date:** March 23, 2026
**Status:** ✅ Ready to Implement (Simplified Plan)
**Previous Version:** 1.0 (Had misconceptions - corrected)

---

## Executive Summary

**CRITICAL CORRECTION:** The previous analysis (v1.0) contained **significant misconceptions**. This revised plan reflects the **actual state** of the codebase.

### What We Got Wrong Before ❌

| Previous Claim | Reality ✅ |
|----------------|-----------|
| "CLI missing JSON Lines" | Already in orchestrator: `--format jsonl` |
| "CLI missing Tier 2 LLM" | Already in orchestrator: `--deep-llm` |
| "CLI missing concurrency" | Already in orchestrator: `--concurrency N` |
| "Complex migration needed" | Just rename orchestrator to glassware |
| "3 weeks timeline" | 1-2 weeks is sufficient |

### The Truth ✅

**glassware-orchestrator already has ALL features:**
- ✅ Campaign orchestration with checkpoint/resume
- ✅ TUI with live monitoring and drill-down
- ✅ LLM Tier 1 (Cerebras) and Tier 2 (NVIDIA)
- ✅ All output formats (JSON, JSONL, SARIF)
- ✅ Concurrency control, rate limiting
- ✅ SQLite caching, adversarial testing

**glassware-cli is a SIMPLE scanner:**
- ✅ Single-file directory scanner
- ✅ Uses rayon for parallel file scanning
- ✅ JSON-based caching (simple key-value)
- ❌ No campaigns, no TUI, no LLM Tier 2

---

## Revised Strategy

### Goal

**Rename `glassware-orchestrator` to `glassware`** and deprecate `glassware-cli`.

### Why This is Simpler

1. **No code migration needed** - orchestrator already has everything
2. **No feature gaps to fill** - all features exist
3. **No complex restructuring** - just rename and update workspace

### Timeline

| Phase | Duration | Tasks |
|-------|----------|-------|
| **Week 1** | 3-4 days | Rename, deprecate, test |
| **Week 2** | 5 days | Optimization (LTO, features, deps) |

**Total:** 1.5-2 weeks (not 3 weeks)

---

## Current State (Corrected)

### Binary Comparison

| Feature | glassware-cli | glassware-orchestrator |
|---------|---------------|------------------------|
| **Purpose** | Simple file/directory scanner | Full campaign orchestration |
| **Binary Name** | `glassware` | `glassware-orchestrator` |
| **Size** | ~11MB | ~25MB |
| **Lines of Code** | 1,019 (single file) | 1,889 + 26 modules |
| **Parallelism** | rayon (sync) | tokio (async) |
| **Caching** | JSON file | SQLite database |
| **TUI** | ❌ | ✅ Full TUI |
| **Campaigns** | ❌ | ✅ Full DAG execution |
| **LLM Tier 1** | ✅ `--llm` | ✅ `--llm` |
| **LLM Tier 2** | ❌ | ✅ `--deep-llm` |
| **Output Formats** | pretty, json, sarif | pretty, json, **jsonl**, sarif |
| **Concurrency** | ❌ (fixed) | ✅ `--concurrency N` |
| **Rate Limiting** | ❌ | ✅ Built-in |
| **Checkpoint/Resume** | ❌ | ✅ SQLite checkpoints |

### Command Comparison

#### glassware-cli (Current)

```bash
glassware [PATHS...] [OPTIONS]

# Options
--format <FORMAT>         # pretty, json, sarif
--severity <LEVEL>        # info, low, medium, high, critical
--quiet, --no-color
--extensions <EXT>        # File extensions
--exclude <DIR>           # Directories to exclude
--llm                     # Tier 1 LLM
--cache-file <PATH>       # JSON cache
--cache-ttl <DAYS>        # Cache TTL
--no-cache
```

#### glassware-orchestrator (Current)

```bash
glassware-orchestrator <COMMAND>

# Campaign Commands
campaign run <CONFIG> [--llm] [--llm-deep] [--concurrency N] [--format jsonl]
campaign resume <CASE-ID>
campaign status <CASE-ID>
campaign list
campaign monitor <CASE-ID>
campaign demo
campaign report <CASE-ID>
campaign query <CASE-ID> "question"
campaign command <CASE-ID> <CMD>

# Scan Commands
scan-npm <PACKAGES...> [--versions POLICY]
scan-github <REPOS...> [--ref REF]
scan-file <FILE>
scan-tarball <TARBALL...>
search-github <QUERY> [--max-results N]

# Other Commands
resume <SOURCE>
sample-packages <CATEGORY>
cache-stats, cache-cleanup
scan-list, scan-show, scan-cancel
config init|show|edit|validate|reset

# Global Options
--format <FORMAT>         # pretty, json, jsonl, sarif
--severity <LEVEL>
--concurrency <N>         # Max concurrent operations
--npm-rate-limit <N>      # npm rate limit
--github-rate-limit <N>   # GitHub rate limit
--llm                     # Tier 1 LLM (Cerebras)
--deep-llm                # Tier 2 LLM (NVIDIA)
--streaming               # JSON Lines output
--adversarial             # Adversarial testing
--log-level <LEVEL>
--verbose, --quiet, --no-color
--checkpoint-dir <DIR>
--cache-db <PATH>         # SQLite cache
--cache-ttl <DAYS>
--no-cache
```

**Conclusion:** glassware-orchestrator has ALL features. It should become the unified `glassware` binary.

---

## Implementation Plan (Revised)

### Week 1: Rename & Deprecate (3-4 days)

#### Step 1: Rename glassware-orchestrator to glassware

```bash
# Rename directory
mv glassware-orchestrator glassware

# Update glassware/Cargo.toml
# Change: name = "glassware-orchestrator"
# To:     name = "glassware"
```

**glassware/Cargo.toml:**
```toml
[package]
name = "glassware"                    # Changed from glassware-orchestrator
version.workspace = true
edition.workspace = true
description = "Unified GlassWare detection and campaign orchestration"
```

---

#### Step 2: Update Workspace

**Root Cargo.toml:**
```toml
[workspace]
members = [
    "glassware-core",
    "glassware",                       # Changed from glassware-orchestrator
    # "glassware-cli",                 # Comment out (deprecation)
]
resolver = "2"
```

---

#### Step 3: Add Deprecation Warning to glassware-cli

**glassware-cli/src/main.rs:** Add at the very beginning of main():

```rust
fn main() -> Result<()> {
    eprintln!("⚠️  DEPRECATION WARNING");
    eprintln!("The 'glassware' binary is deprecated and will be removed in v1.0.0.");
    eprintln!("Please use 'glassware scan' instead:");
    eprintln!("  glassware scan /path/to/code --format json");
    eprintln!();

    // ... rest of existing main()
}
```

---

#### Step 4: Update CLI Help Text

**glassware/src/cli.rs:** Update the CLI metadata:

```rust
#[derive(Parser, Debug)]
#[command(name = "glassware")]                    # Changed
#[command(author = "glassware contributors")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "GlassWare attack detection and campaign orchestration", long_about = None)]
// ... rest unchanged
```

---

#### Step 5: Add `scan` Subcommand (Optional UX Improvement)

To make the transition smoother for glassware-cli users, add a simple `scan` subcommand that wraps the existing scan logic.

**Create `glassware/src/commands/scan.rs`:**

```rust
//! Simple file/directory scanning (for glassware-cli users)
//!
//! This is a thin wrapper around glassware-core's scanner
//! to provide a simple UX for users migrating from glassware-cli.

use clap::Args;
use glassware_core::{scanner, Severity};
use std::path::PathBuf;

/// Scan files and directories for GlassWare attacks
#[derive(Debug, Clone, Args)]
pub struct CmdScan {
    /// Files or directories to scan
    #[arg(required = true)]
    pub paths: Vec<PathBuf>,

    /// Output format: pretty, json, sarif
    #[arg(short, long, value_enum, default_value = "pretty")]
    pub format: OutputFormat,

    /// Minimum severity to report
    #[arg(short, long, value_enum, default_value = "low")]
    pub severity: SeverityLevel,

    // ... (other options from glassware-cli)
}

impl CmdScan {
    pub fn run(&self) -> Result<()> {
        // Simple wrapper around glassware_core::scanner
        // Implementation can reuse orchestrator's scan logic
        todo!("Implement simple scan wrapper")
    }
}
```

**Note:** This is **optional**. Users can also use:
```bash
glassware scan-file /path/to/code    # Already exists
```

---

#### Step 6: Build & Test

```bash
# Build unified binary
cargo build --release -p glassware

# Test basic commands
./target/release/glassware --help
./target/release/glassware campaign --help
./target/release/glassware campaign demo

# Test scan commands
./target/release/glassware scan-file /path/to/code
./target/release/glassware scan-npm express@4.19.2

# Run tests
cargo test -p glassware
```

---

#### Step 7: Create Deprecation Notice

**Create `DEPRECATION-NOTICE.md`:**

```markdown
# Deprecation Notice

**Effective:** Version 0.9.0 (March 2026)
**Removal:** Version 1.0.0 (April 2026)

## Deprecated Binaries

The following binaries are deprecated:

- `glassware` (from `glassware-cli` crate) - simple scanner
- `glassware-orchestrator` - renamed to `glassware`

## Migration Guide

### Simple File Scanning

**Old (v0.8.0):**
```bash
glassware /path/to/code --format json
```

**New (v0.9.0+):**
```bash
glassware scan-file /path/to/code --format json
# Or (if scan subcommand added):
glassware scan /path/to/code --format json
```

### Campaign Scanning

**Old (v0.8.0):**
```bash
glassware-orchestrator campaign run wave6.toml
```

**New (v0.9.0+):**
```bash
glassware campaign run wave6.toml
```

### All Commands

| Old Command | New Command |
|-------------|-------------|
| `glassware-orchestrator campaign run` | `glassware campaign run` |
| `glassware-orchestrator campaign demo` | `glassware campaign demo` |
| `glassware-orchestrator scan-npm` | `glassware scan-npm` |
| `glassware-orchestrator scan-github` | `glassware scan-github` |
| `glassware-orchestrator scan-file` | `glassware scan-file` |
| `glassware /path` | `glassware scan-file /path` or `glassware scan /path` |

## Timeline

- **v0.9.0 (March 2026):** Deprecation warnings added
- **v1.0.0 (April 2026):** `glassware-cli` removed, `glassware-orchestrator` renamed

## Questions?

See `docs/binaryconsolidation/CONSOLIDATION-PLAN-REVISED.md` for details.
```

---

#### Step 8: Update Documentation

**Update README.md:**

```markdown
### Commands

```bash
# Scan files/directories
glassware scan-file /path/to/code
glassware scan-npm express@4.19.2
glassware scan-github org/repo

# Run campaign
glassware campaign run campaigns/wave6.toml

# TUI demo
glassware campaign demo

# Generate report
glassware campaign report <case-id>

# Query with LLM
glassware campaign query <case-id> "Why was express flagged?"
```
```

**Update QWEN.md:** Update the project structure and commands.

---

### Week 2: Optimization (5 days)

#### Step 1: Add Release Profile

**Add to root Cargo.toml:**

```toml
[profile.release]
opt-level = 3           # Full optimization
lto = true              # Link-time optimization
codegen-units = 1       # Single codegen unit
strip = true            # Strip debug symbols
panic = "abort"         # Smaller panic handling

# Debug builds remain fast
[profile.dev]
opt-level = 0
debug = true
```

**Expected:** 30-40% size reduction (~25MB → ~15MB)

---

#### Step 2: Feature-Gate TUI

**Update glassware/Cargo.toml:**

```toml
[dependencies]
# ... (required dependencies)

# Optional: TUI
ratatui = { version = "0.24", optional = true }
crossterm = { version = "0.27", optional = true }

[features]
default = ["tui", "llm"]
tui = ["ratatui", "crossterm"]
llm = ["reqwest"]
minimal = []
```

**Update glassware/src/main.rs:**

```rust
#[cfg(feature = "tui")]
mod tui;

// In command handling:
#[cfg(feature = "tui")]
Commands::Campaign(CampaignCommands::Demo) => {
    cmd_campaign_demo(&cli).await?;
}
#[cfg(not(feature = "tui"))]
Commands::Campaign(CampaignCommands::Demo) => {
    eprintln!("Error: TUI feature not enabled. Build with --features tui");
    std::process::exit(1);
}
```

---

#### Step 3: Selective Tokio Features

**Update workspace Cargo.toml:**

```toml
[workspace.dependencies]
# Current (wasteful):
# tokio = { version = "1.35", features = ["full"] }

# Better (selective):
tokio = { version = "1.35", features = [
    "rt-multi-thread",
    "macros",
    "fs",
    "io-util",
    "net",
    "signal",
    "sync",
    "time"
] }
```

**Expected:** ~0.5-1MB savings

---

#### Step 4: Evaluate rusqlite/sqlx Consolidation

**Current:** Both `sqlx` (async) and `rusqlite` (sync) are used.

**Option A: Use only rusqlite with async wrapper**
```rust
use tokio::task::spawn_blocking;

async fn query_async() -> Result<()> {
    spawn_blocking(|| {
        // Use rusqlite here
    }).await?
}
```

**Option B: Use only sqlx**
- Replace rusqlite calls with sqlx
- Keep async throughout

**Expected:** ~2MB savings

**Recommendation:** Evaluate effort vs. savings. May defer to v1.1.0.

---

#### Step 5: Measure & Benchmark

```bash
# Build with optimizations
cargo build --release -p glassware

# Check size
ls -lh target/release/glassware

# Benchmark scan speed
time ./target/release/glassware scan-file /path/to/large/codebase

# Benchmark memory
/usr/bin/time -v ./target/release/glassware campaign demo

# Compare with v0.8.0
# - Binary size
# - Memory usage
# - Scan speed
```

---

## Success Metrics

### Before (v0.8.0)

| Metric | glassware-cli | glassware-orchestrator | Total |
|--------|---------------|------------------------|-------|
| Binary Size | ~11MB | ~25MB | ~36MB |
| Memory | ~30MB | ~50MB | - |
| Scan Speed | ~50k LOC/s | ~50k LOC/s | - |

### After (v0.9.0)

| Metric | Target | Notes |
|--------|--------|-------|
| Binaries | 1 (`glassware`) | -50% |
| Binary Size | ~15-18MB | LTO + strip only |
| Binary Size (minimal) | ~10-12MB | Without TUI, LLM |
| Memory | ~35-40MB | Slight improvement |
| Scan Speed | ~50-55k LOC/s | Similar (no regression) |

### After (v1.0.0 with full optimization)

| Metric | Target | Notes |
|--------|--------|-------|
| Binary Size | ~10-15MB | With all optimizations |
| Memory | ~25-35MB | With streaming improvements |
| Scan Speed | ~60-65k LOC/s | With PGO and optimizations |

---

## Risks & Mitigation

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Breaking changes for CLI users | Medium | Medium | Deprecation warning, migration guide |
| TUI feature-gate breaks demo | Low | Low | Test thoroughly, provide clear error |
| rusqlite/sqlx consolidation breaks things | Medium | Medium | Defer to v1.1.0 if risky |
| Build time increases with LTO | Low | High | Acceptable (release builds only) |

**Overall Risk:** 🟢 Low - Simple rename, minimal code changes

---

## Testing Checklist

Before releasing v0.9.0:

- [ ] `glassware --help` works
- [ ] `glassware campaign --help` works
- [ ] `glassware campaign demo` works (TUI)
- [ ] `glassware campaign run wave6.toml` works
- [ ] `glassware campaign run wave6.toml --llm` works
- [ ] `glassware campaign run wave6.toml --deep-llm` works
- [ ] `glassware campaign run wave6.toml --format jsonl` works
- [ ] `glassware scan-file /path` works
- [ ] `glassware scan-npm express@4.19.2` works
- [ ] `glassware scan-github org/repo` works
- [ ] `glassware campaign query <id> "question"` works
- [ ] All tests pass: `cargo test -p glassware`
- [ ] Binary size is acceptable
- [ ] Deprecation warning shows for glassware-cli

---

## Release Plan

### v0.9.0 (Deprecation Release)

**Timeline:** March 27-29, 2026

**Changes:**
- Rename glassware-orchestrator → glassware
- Add deprecation warning to glassware-cli
- Update documentation
- Create migration guide

**Git Tag:** `v0.9.0-consolidated`

---

### v1.0.0 (Removal Release)

**Timeline:** April 10-13, 2026

**Changes:**
- Remove glassware-cli from workspace
- Remove glassware-orchestrator references
- Full optimization enabled (LTO, features, etc.)

**Git Tag:** `v1.0.0-unified`

---

## Next Steps

1. **Review this revised plan** - Ensure understanding
2. **Start with Step 1** - Rename glassware-orchestrator to glassware
3. **Test thoroughly** - All commands must work
4. **Create v0.9.0 tag** - Push to remote
5. **Proceed to Week 2** - Optimization phase

---

## Questions?

If anything is unclear, see:
- `RESPONSE-TO-AGENT.md` - Previous developer's clarifications
- `QUESTIONS.md` - Original questions (some now answered)
- `HANDOFF/README.md` - General handoff

**Key Insight:** glassware-orchestrator already has everything. Just rename it and optimize!

---

**Last Updated:** March 23, 2026
**Status:** Ready to implement
