# Binary Consolidation and Optimization Plan

**Document Version:** 1.0
**Date:** March 23, 2026
**Status:** Planning Phase - Ready for Review
**Author:** Research Analysis

---

## Executive Summary

This document provides a comprehensive plan for consolidating two Rust binaries (`glassware-cli` and `glassware-orchestrator`) into a single unified binary, along with optimization strategies to achieve significant improvements in binary size, memory usage, and scan speed.

### Current State

| Metric | Value |
|--------|-------|
| Binaries | 2 (`glassware`, `glassware-orchestrator`) |
| Total Size | ~36MB (11MB + 25MB) |
| Memory Usage | ~50MB during scan |
| Scan Speed | ~50k LOC/sec |

### Target State

| Metric | Target | Improvement |
|--------|--------|-------------|
| Binaries | 1 (`glassware`) | -50% |
| Total Size | 10-15MB | -60% |
| Memory Usage | 25-35MB | -40% |
| Scan Speed | ~65k LOC/sec | +30% |

### Timeline

**Total Duration:** 3 weeks
**Resources:** 1 developer
**Risk Level:** Low (incremental migration)

---

## 1. Current State Analysis

### 1.1 Feature Comparison Table

| Feature | glassware-cli | glassware-orchestrator | Gap Analysis |
|---------|---------------|------------------------|--------------|
| **Core Scanning** | | | |
| Directory/file scanning | ✅ Full support | ✅ Via campaign | CLI has simpler UX |
| File extension filtering | ✅ `--extensions` | ✅ Config-based | Parity |
| Exclusion patterns | ✅ `--exclude` | ✅ Config-based | Parity |
| Parallel scanning (rayon) | ✅ | ✅ (tokio) | CLI uses rayon, orchestrator uses tokio |
| **Output Formats** | | | |
| Pretty print | ✅ | ✅ | Parity |
| JSON | ✅ | ✅ | Parity |
| SARIF 2.1.0 | ✅ | ✅ | Parity |
| JSON Lines | ❌ | ✅ | **CLI missing** |
| **Detection Features** | | | |
| L1 detectors (Unicode, homoglyphs, bidi) | ✅ | ✅ | Parity |
| L2 detectors (GlassWare patterns) | ✅ | ✅ | Parity |
| L3 detectors (behavioral) | ✅ (via core) | ✅ | Parity |
| LLM analysis (Tier 1) | ✅ `--llm` | ✅ `--llm` | Parity |
| LLM analysis (Tier 2) | ❌ | ✅ `--deep-llm` | **CLI missing** |
| **Caching** | | | |
| Incremental scanning | ✅ JSON cache | ✅ SQLite cache | Different implementations |
| Cache TTL | ✅ `--cache-ttl` | ✅ Config-based | Parity |
| Cache stats | ✅ | ✅ | Parity |
| **Campaign Features** | | | |
| Campaign TOML config | ❌ | ✅ | **Orchestrator only** |
| Wave execution (DAG) | ❌ | ✅ | **Orchestrator only** |
| Checkpoint/resume | ❌ | ✅ | **Orchestrator only** |
| Progress tracking | ❌ | ✅ | **Orchestrator only** |
| **TUI** | | | |
| Live monitoring | ❌ | ✅ (`campaign demo`) | **Orchestrator only** |
| Command palette | ❌ | ✅ | **Orchestrator only** |
| Package drill-down | ❌ | ✅ | **Orchestrator only** |
| **Advanced Features** | | | |
| npm package scanning | ❌ | ✅ | **Orchestrator only** |
| GitHub repo scanning | ❌ | ✅ | **Orchestrator only** |
| Tarball scanning | ❌ | ✅ | **Orchestrator only** |
| Adversarial testing | ❌ | ✅ | **Orchestrator only** |
| Version scanning | ❌ | ✅ | **Orchestrator only** |
| Scan registry | ❌ | ✅ | **Orchestrator only** |
| **CLI Options** | | | |
| Severity filtering | ✅ `--severity` | ✅ `--severity` | Parity |
| Quiet mode | ✅ `--quiet` | ✅ `--quiet` | Parity |
| No color | ✅ `--no-color` | ✅ `--no-color` | Parity |
| Concurrency control | ❌ (fixed) | ✅ `--concurrency` | **CLI missing** |
| Rate limiting | ❌ | ✅ | **CLI missing** |
| Logging config | ❌ | ✅ `--log-level` | **CLI missing** |
| Verbose mode | ❌ | ✅ `--verbose` | **CLI missing** |

### 1.2 CLI Subcommands Inventory

#### glassware-cli (Current)

```
glassware [PATHS...] [OPTIONS]

Arguments:
  paths                  Files or directories to scan (required, multiple)

Options:
  -f, --format <FORMAT>  Output format: pretty, json, sarif [default: pretty]
  -s, --severity <LEVEL> Minimum severity: info, low, medium, high, critical [default: low]
  -q, --quiet            Suppress output, only set exit code
      --no-color         Disable colored output
      --extensions <EXT> File extensions (comma-separated) [default: js,mjs,cjs,ts,tsx,...]
      --exclude <DIR>    Directories to exclude (comma-separated) [default: .git,node_modules,...]
      --llm              Run LLM analysis (feature: llm)
      --cache-file <PATH> Cache file path [default: .glassware-cache.json]
      --cache-ttl <DAYS> Cache TTL [default: 7]
      --no-cache         Disable caching
```

#### glassware-orchestrator (Current)

```
glassware-orchestrator <COMMAND>

Commands:
  campaign        Campaign management
    run           Run campaign from config
    resume        Resume interrupted campaign
    status        Show campaign status
    list          List recent campaigns
    monitor       Live TUI monitoring
    demo          TUI demo with sample data
    report        Generate report
    query         Ask LLM questions
    command       Send steering commands

  scan-npm        Scan npm packages
  scan-github     Scan GitHub repositories
  search-github   Search GitHub repositories
  scan-file       Scan packages from file
  scan-tarball    Scan tarball files directly
  resume          Resume interrupted scan
  sample-packages Sample packages from npm
  cache-stats     Show cache statistics
  cache-cleanup   Clean up expired cache
  scan-list       List scan history
  scan-show       Show scan details
  scan-cancel     Cancel running scan
  config          Configuration management

Global Options:
  -f, --format <FORMAT>     Output format: pretty, json, jsonl, sarif
  -s, --severity <LEVEL>    Minimum severity
  -c, --concurrency <NUM>   Maximum concurrent operations [default: 10]
      --max-retries <NUM>   Maximum retries [default: 3]
      --npm-rate-limit <N>  npm rate limit [default: 10]
      --github-rate-limit   GitHub rate limit [default: 5]
      --github-token <TOK>  GitHub token
      --streaming           Enable streaming output
      --adversarial         Enable adversarial testing
      --llm                 Enable Tier 1 LLM
      --deep-llm            Enable Tier 2 LLM
      --log-level <LEVEL>   Log level [default: info]
      --log-file <PATH>     Log output file
  -o, --output <PATH>       Output file path
  -q, --quiet               Quiet mode
  -v, --verbose             Verbose mode
      --no-color            Disable colored output
      --checkpoint-dir <D>  Checkpoint directory
      --threat-threshold <T> Threat score threshold
      --cache-db <PATH>     Cache database path
      --cache-ttl <DAYS>    Cache TTL
      --no-cache            Disable caching
```

### 1.3 Dependency Analysis

#### Overlapping Dependencies

| Dependency | glassware-cli | glassware-orchestrator | Version | Notes |
|------------|---------------|------------------------|---------|-------|
| `glassware-core` | ✅ (full, binary) | ✅ (binary) | workspace | Core detection engine |
| `clap` | ✅ (derive) | ✅ (workspace) | 4.4 | CLI parsing |
| `serde` | ✅ (derive) | ✅ (workspace) | 1.0 | Serialization |
| `serde_json` | ✅ | ✅ (workspace) | 1.0 | JSON handling |
| `tokio` | ❌ | ✅ (full) | 1.35 | Async runtime (orchestrator only) |
| `rayon` | ✅ | ❌ | 1.10 | Parallel iterators (CLI only) |
| `ignore` | ✅ | ❌ | 0.4 | File walking (CLI only) |
| `colored` | ✅ | ❌ | 2 | Terminal colors (CLI only) |
| `walkdir` | ❌ | ✅ (workspace) | 2.4 | Directory walking (orchestrator only) |
| `tracing` | ❌ | ✅ (workspace) | 0.1 | Logging (orchestrator only) |
| `tracing-subscriber` | ❌ | ✅ (workspace) | 0.3 | Logging (orchestrator only) |
| `reqwest` | ❌ (via core) | ✅ (workspace) | 0.11 | HTTP client |
| `sqlx` | ❌ | ✅ (workspace) | 0.7 | SQLite (orchestrator only) |
| `rusqlite` | ❌ | ✅ (bundled) | 0.29 | SQLite (orchestrator only) |
| `ratatui` | ❌ | ✅ | 0.24 | TUI framework (orchestrator only) |
| `crossterm` | ❌ | ✅ | 0.27 | Terminal control (orchestrator only) |
| `governor` | ❌ | ✅ (workspace) | 0.6 | Rate limiting (orchestrator only) |
| ` anyhow` | ❌ | ✅ (workspace) | 1.0 | Error handling (orchestrator only) |
| `thiserror` | ❌ | ✅ (workspace) | 1.0 | Error types (orchestrator only) |
| `futures` | ❌ | ✅ (workspace) | 0.3 | Async utilities (orchestrator only) |
| `uuid` | ❌ | ✅ (workspace) | 1.6 | UUIDs (orchestrator only) |
| `chrono` | ❌ | ✅ (workspace) | 0.4 | Date/time (orchestrator only) |
| `parking_lot` | ❌ | ✅ (workspace) | 0.12 | Synchronization (orchestrator only) |
| `toml` | ❌ | ✅ | 0.8 | TOML parsing (orchestrator only) |
| `tera` | ❌ | ✅ | 1.19 | Templates (orchestrator only) |
| `tar` | ❌ | ✅ | 0.4 | Tarball handling (orchestrator only) |
| `flate2` | ❌ | ✅ | 1.0 | Compression (orchestrator only) |

#### Heavy Dependencies (Estimated Size Impact)

| Dependency | Estimated Size | Can Be Optional? | Notes |
|------------|---------------|------------------|-------|
| `tokio` (full) | ~3MB | ❌ (needed for async) | Consider `tokio-lite` |
| `sqlx` + `rusqlite` | ~4MB | ⚠️ (checkpoint needs SQLite) | Could use single SQLite lib |
| `ratatui` + `crossterm` | ~2MB | ✅ (TUI feature) | Already optional via subcommand |
| `reqwest` | ~1.5MB | ⚠️ (LLM/GitHub needs HTTP) | Feature-gate LLM |
| `tera` | ~1MB | ✅ (report feature) | Only for markdown reports |
| `governor` | ~0.5MB | ⚠️ (rate limiting) | Important for API calls |

#### CLI-Only Dependencies (to be merged)

| Dependency | Purpose | Migration Strategy |
|------------|---------|-------------------|
| `rayon` | Parallel file scanning | Keep for `scan` subcommand |
| `ignore` | File walking with gitignore | Keep for `scan` subcommand |
| `colored` | Terminal colors | Merge with orchestrator's color handling |

### 1.4 Code Structure Analysis

#### glassware-cli Structure

```
glassware-cli/
└── src/
    └── main.rs (1019 lines)
        ├── Args parsing (clap derive)
        ├── collect_files() - WalkBuilder + OverrideBuilder
        ├── should_scan_file() - Extension checking
        ├── Parallel scanning (rayon par_iter)
        ├── Thread-safe collections (Arc<Mutex<>>)
        ├── output_pretty() / output_pretty_with_llm()
        ├── output_json()
        └── output_sarif()
```

**Key Characteristics:**
- Single-file implementation
- Synchronous with rayon parallelism
- Simple file/directory focus
- JSON-based caching (via glassware-core)

#### glassware-orchestrator Structure

```
glassware-orchestrator/
└── src/
    ├── main.rs (1889 lines)
    │   ├── CLI command routing
    │   ├── cmd_campaign_*() handlers
    │   ├── cmd_scan_*() handlers
    │   ├── cmd_config_*() handlers
    │   ├── create_orchestrator()
    │   └── print_results()
    │
    ├── cli.rs (350+ lines)
    │   ├── Cli struct (clap derive)
    │   ├── Commands enum (subcommands)
    │   ├── CampaignCommands enum
    │   ├── ConfigCommands enum
    │   └── OutputFormat, SeverityLevel enums
    │
    ├── campaign/ (11 modules)
    │   ├── types.rs - Core types
    │   ├── event_bus.rs - Pub/sub
    │   ├── state_manager.rs - Queryable state
    │   ├── command_channel.rs - Steering
    │   ├── config.rs - TOML parsing
    │   ├── wave.rs - Wave execution
    │   ├── executor.rs - DAG scheduler
    │   ├── checkpoint.rs - Persistence
    │   ├── report.rs - Report generation
    │   └── query/ - LLM queries
    │
    ├── tui/
    │   ├── app.rs - TUI application
    │   └── ui.rs - Rendering
    │
    └── Library modules (26 files)
        ├── orchestrator.rs - Core orchestration
        ├── scanner.rs - Package scanning
        ├── downloader.rs - npm/GitHub download
        ├── cacher.rs - SQLite caching
        ├── llm.rs - LLM analysis
        └── ... (21 more)
```

**Key Characteristics:**
- Modular library + binary structure
- Async tokio runtime
- Campaign orchestration with DAG
- TUI support
- SQLite-based caching

### 1.5 Build Configuration Review

#### Current Workspace Settings (`/home/shva/samgrowls/glassworks/Cargo.toml`)

```toml
[workspace]
members = [
    "glassware-core",
    "glassware-cli",
    "glassware-orchestrator",
]
resolver = "2"

[workspace.package]
version = "0.8.0"
edition = "2021"
license = "MIT"
rust-version = "1.70"

[workspace.dependencies]
# All shared dependencies defined here
```

#### glassware-cli Cargo.toml

```toml
[package]
name = "glassware-cli"
version = "0.5.0"

[dependencies]
glassware-core = { path = "../glassware-core", features = ["full", "binary"] }
clap = { version = "4", features = ["derive"] }
colored = "2"
serde_json = "1"
serde = { version = "1", features = ["derive"] }
ignore = "0.4"
globset = "0.4"
rayon = "1.10"

[features]
default = ["llm", "binary"]
llm = ["glassware-core/llm"]
binary = ["glassware-core/binary"]
```

**Missing:** No `[profile.release]` section - uses Rust defaults

#### glassware-orchestrator Cargo.toml

```toml
[package]
name = "glassware-orchestrator"
version.workspace = true

[dependencies]
# Uses workspace dependencies
glassware-core = { workspace = true, features = ["binary"] }
tokio = { workspace = true }
# ... (many more workspace deps)
ratatui = "0.24"
crossterm = "0.27"
tar = "0.4"
flate2 = "1.0"
toml = "0.8"
dirs = "5.0"
md5 = "0.7"
rusqlite = { version = "0.29", features = ["bundled"] }
tera = "1.19"

[features]
default = ["llm", "binary"]
llm = []
rate-limit = []
retry = []
binary = ["glassware-core/binary"]
```

**Missing:** No `[profile.release]` section - uses Rust defaults

#### glassware-core Cargo.toml

```toml
[package]
name = "glassware-core"
version = "0.5.0"

[dependencies]
unicode-script = "0.5"
num_cpus = "1.16"
rand = "0.8"
base64 = "0.21"

# Optional
regex = { version = "1.10", optional = true }
serde = { version = "1.0", features = ["derive"], optional = true }
serde_json = { version = "1.0", optional = true }
# ... (many more optional deps)

[features]
default = ["full"]
full = ["regex", "serde", "serde_json", "lazy_static", "once_cell", "semantic", "cache", "binary"]
minimal = []
```

**Missing:** No `[profile.release]` section - uses Rust defaults

### 1.6 Existing Documentation Review

#### BINARY-CONSOLIDATION.md Analysis

The existing document at `/home/shva/samgrowls/glassworks/HANDOFF/FUTURE/BINARY-CONSOLIDATION.md` provides:

- ✅ Current state analysis (accurate)
- ✅ Consolidation recommendation (YES)
- ✅ Proposed architecture diagram
- ✅ Migration path (4 phases)
- ✅ Optimization strategies (LTO, features, dependencies)
- ✅ Expected outcomes table
- ✅ Week-by-week implementation plan

**Gaps Identified:**
- ❌ No detailed feature audit (completed in this document)
- ❌ No dependency overlap analysis (completed in this document)
- ❌ No test coverage analysis
- ❌ No specific workspace restructuring details
- ❌ No risk mitigation strategies

#### Related Documentation

| Document | Relevance |
|----------|-----------|
| `HANDOFF/ARCHITECTURE-OVERVIEW.md` | Campaign architecture context |
| `HANDOFF/ARCHITECTURAL-CONSIDERATIONS.md` | TUI design, plugin architecture |
| `HANDOFF/PRODUCTION-ROADMAP.md` | Production requirements |
| `design/CAMPAIGN-ARCHITECTURE.md` | Campaign system design |
| `design/RFC-001-TUI-ARCHITECTURE.md` | TUI architecture |
| `design/WAVE-CAMPAIGN-DESIGN.md` | Wave campaign design |

### 1.7 Test Coverage Analysis

#### glassware-core Tests

| Test File | Purpose | Features Required |
|-----------|---------|-------------------|
| `integration_campaign_fixtures.rs` | Campaign fixture testing | full |
| `integration_false_positives.rs` | False positive detection | full |
| `integration_edge_cases.rs` | Edge case handling | full |
| `integration_scan_directory.rs` | Directory scanning | full |
| `integration_cross_file.rs` | Cross-file analysis | full |
| `integration_campaign_intelligence.rs` | Campaign intelligence | full |
| `integration_attack_graph.rs` | Attack graph analysis | full |

#### glassware-orchestrator Tests

Located in-module (`#[cfg(test)]`):

| Module | Test Count | Coverage |
|--------|-----------|----------|
| `orchestrator.rs` | 5+ | Creation, config |
| `sampler.rs` | 1+ | Package sampling |
| `adversarial.rs` | 7+ | Mutation, fuzzing |
| `retry.rs` | 6+ | Retry logic |
| `llm.rs` | 9+ | LLM analysis |
| `downloader.rs` | 1+ | Download logic |
| `lib.rs` | 12+ | Library smoke tests |

**Total:** ~40+ unit tests in orchestrator

#### Test Preservation Requirements

During consolidation:
1. All glassware-core integration tests must pass
2. All orchestrator unit tests must pass
3. CLI-specific tests needed for new `scan` subcommand
4. End-to-end campaign tests must continue working

---

## 2. Consolidation Strategy

### 2.1 Recommended Workspace Restructuring

#### Target Structure

```
glassworks/
├── Cargo.toml                    # Workspace root
├── glassware-core/               # Library (unchanged)
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
│
├── glassware/                    # NEW: Unified binary (was glassware-orchestrator)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs               # Entry point
│       ├── cli.rs                # CLI definitions (expanded)
│       ├── commands/             # NEW: Command modules
│       │   ├── mod.rs
│       │   ├── scan.rs           # NEW: Simple scanning (from glassware-cli)
│       │   ├── campaign.rs       # Campaign commands (existing)
│       │   ├── cache.rs          # Cache commands (existing)
│       │   └── config.rs         # Config commands (existing)
│       ├── tui/                  # TUI (existing)
│       └── ...                   # Library modules (existing)
│
└── glassware-cli/                # DEPRECATED (removed after migration)
```

#### Workspace Cargo.toml Updates

```toml
[workspace]
members = [
    "glassware-core",
    "glassware",  # Changed from glassware-orchestrator
]
# Removed: glassware-cli (deprecated)
resolver = "2"

[workspace.package]
version = "0.9.0"  # Bump for consolidation release
edition = "2021"
license = "MIT"
rust-version = "1.70"

[workspace.dependencies]
glassware-core = { path = "./glassware-core", version = "0.6.0" }

# Existing workspace deps...
tokio = { version = "1.35", features = ["full"] }
clap = { version = "4.4", features = ["derive"] }
# ...

# NEW: Add CLI-specific deps
rayon = "1.10"
ignore = "0.4"
colored = "2"
```

#### New glassware/Cargo.toml

```toml
[package]
name = "glassware"  # Changed from glassware-orchestrator
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Unified GlassWare detection system - scanning and campaign orchestration"
authors = ["glassware team"]

[[bin]]
name = "glassware"
path = "src/main.rs"

[dependencies]
# Core
glassware-core = { workspace = true, features = ["full", "binary"] }

# Async runtime
tokio = { workspace = true }

# CLI
clap = { workspace = true }

# HTTP
reqwest = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Database
sqlx = { workspace = true }

# Error handling
thiserror = { workspace = true }
anyhow = { workspace = true }

# Utilities
futures = { workspace = true }
walkdir = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }

# Logging
tracing = { workspace = true }
tracing-subscriber = { workspace = true }

# Concurrency
parking_lot = { workspace = true }
urlencoding = { workspace = true }
rand = { workspace = true }

# Rate limiting
governor = { workspace = true }
nonzero_ext = { workspace = true }

# Utilities
tempfile = { workspace = true }
tar = "0.4"
flate2 = "1.0"
toml = "0.8"
dirs = "5.0"
md5 = "0.7"
rusqlite = { version = "0.29", features = ["bundled"] }
tera = "1.19"

# TUI
ratatui = "0.24"
crossterm = "0.27"

# NEW: CLI scanning features (from glassware-cli)
rayon = "1.10"
ignore = "0.4"
colored = "2"
globset = "0.4"

[features]
default = ["tui", "llm", "sqlite", "binary"]
tui = ["ratatui", "crossterm"]
llm = ["glassware-core/llm"]
sqlite = ["sqlx", "rusqlite"]
binary = ["glassware-core/binary"]
minimal = []  # No TUI, no LLM, no SQLite

[dev-dependencies]
tokio-test = { workspace = true }
```

### 2.2 CLI Subcommand Design

#### Unified Command Structure

```
glassware <COMMAND>

Commands:
  scan                    Scan files/directories (old glassware-cli)
  campaign                Campaign management
  cache                   Cache management
  config                  Configuration
  completion              Generate shell completions

Global Options:
  -v, --verbose           Verbose output
  -q, --quiet             Quiet mode
      --no-color          Disable colors
      --log-level <LVL>   Log level [default: info]
  -h, --help              Print help
  -V, --version           Print version
```

#### `scan` Subcommand (from glassware-cli)

```
glassware scan [PATHS...] [OPTIONS]

Arguments:
  paths...                 Files or directories to scan

Options:
  -f, --format <FORMAT>    Output format [default: pretty] [possible: pretty, json, sarif]
  -s, --severity <LEVEL>   Minimum severity [default: low] [possible: info, low, medium, high, critical]
  -q, --quiet              Suppress output, only set exit code
      --no-color           Disable colored output
      --extensions <EXT>   File extensions [default: js,mjs,cjs,ts,tsx,jsx,py,rs,go,java,rb,php,sh,bash,zsh,yml,yaml,toml,json,xml,md,txt]
      --exclude <DIR>      Directories to exclude [default: .git,node_modules,target,__pycache__,.venv,vendor]
      --llm                Run LLM analysis (requires GLASSWARE_LLM_BASE_URL and GLASSWARE_LLM_API_KEY)
      --cache-file <PATH>  Cache file path [default: .glassware-cache.json]
      --cache-ttl <DAYS>   Cache TTL [default: 7]
      --no-cache           Disable caching
  -j, --jobs <NUM>         Number of parallel jobs [default: auto]
```

#### `campaign` Subcommand (existing, unchanged)

```
glassware campaign <COMMAND>

Commands:
  run           Run campaign from config
  resume        Resume interrupted campaign
  status        Show campaign status
  list          List recent campaigns
  monitor       Live TUI monitoring
  demo          TUI demo with sample data
  report        Generate report
  query         Ask LLM questions
  command       Send steering commands
```

### 2.3 Migration Path

#### Phase 1: Feature Parity Check (Week 1, Days 1-2)

**Goal:** Ensure no functionality is lost

**Tasks:**
1. [ ] Audit all glassware-cli CLI options
2. [ ] Verify glassware-orchestrator has equivalent features
3. [ ] Document any gaps (see Section 4: Questions)
4. [ ] Create test cases for CLI-specific functionality

**Gap Analysis Results:**

| Feature | CLI | Orchestrator | Action |
|---------|-----|--------------|--------|
| JSON Lines output | ❌ | ✅ | Add to scan command |
| Tier 2 LLM | ❌ | ✅ | Add `--deep-llm` to scan |
| Concurrency control | ❌ | ✅ | Add `-j/--jobs` to scan |
| Streaming output | ❌ | ✅ | Add `--streaming` to scan |

#### Phase 2: Code Consolidation (Week 1, Days 3-5 + Week 2, Days 1-2)

**Goal:** Merge codebases with minimal breaking changes

**Tasks:**
1. [ ] Rename `glassware-orchestrator/` to `glassware/`
2. [ ] Update workspace `Cargo.toml`
3. [ ] Create `src/commands/scan.rs` from glassware-cli logic
4. [ ] Update `src/cli.rs` with new `scan` subcommand
5. [ ] Update `src/main.rs` command routing
6. [ ] Merge caching implementations (JSON vs SQLite)
7. [ ] Update all path references
8. [ ] Build and test

**Code Migration - scan.rs Structure:**

```rust
// src/commands/scan.rs

use clap::Parser;
use glassware_core::{ScanConfig, ScanEngine, Finding, Severity};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use crate::cli::OutputFormat;
use crate::cli::SeverityLevel;

/// Scan files and directories for GlassWare attacks
#[derive(Parser, Debug)]
pub struct ScanCommand {
    /// Files or directories to scan
    #[arg(required = true)]
    pub paths: Vec<PathBuf>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "pretty")]
    pub format: OutputFormat,

    /// Minimum severity to report
    #[arg(short, long, value_enum, default_value = "low")]
    pub severity: SeverityLevel,

    /// Suppress output, only set exit code
    #[arg(short, long, default_value = "false")]
    pub quiet: bool,

    // ... (rest of CLI options)
}

impl ScanCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        // Implementation from glassware-cli main.rs
        // Adapted for library structure
    }
}
```

#### Phase 3: Deprecation (Week 2, Days 3-5)

**Goal:** Mark old binaries as deprecated

**Tasks:**
1. [ ] Update glassware-cli `Cargo.toml` with deprecation notice
2. [ ] Add runtime warning to glassware-cli
3. [ ] Update README.md with migration guide
4. [ ] Update documentation
5. [ ] Update CI/CD pipelines

**Deprecation Notice for glassware-cli/Cargo.toml:**

```toml
[package]
name = "glassware-cli"
version = "0.5.1"  # Patch bump
deprecated = true
description = "DEPRECATED: Use 'glassware' package instead. Simple directory scanner - now part of unified glassware binary."
```

**Runtime Warning:**

```rust
fn main() {
    eprintln!("⚠️  WARNING: glassware-cli is deprecated. Use 'glassware scan' instead.");
    eprintln!("   See: https://github.com/samgrowls/glassworks/blob/main/docs/MIGRATION.md");
    eprintln!();
    // ... continue with existing functionality
}
```

#### Phase 4: Removal (After 1 Release Cycle)

**Goal:** Clean up deprecated code

**Tasks:**
1. [ ] Wait for one release cycle (v0.9.x)
2. [ ] Remove glassware-cli from workspace
3. [ ] Delete glassware-cli directory
4. [ ] Clean up any remaining references
5. [ ] Update changelog

### 2.4 Breaking Changes Assessment

| Change | Impact | Mitigation |
|--------|--------|------------|
| Binary name change (`glassware-orchestrator` → `glassware`) | Medium | Update scripts, docs, CI/CD |
| glassware-cli removal | Medium | Deprecation period, migration guide |
| Cache format change (JSON → SQLite for scan) | Low | Auto-migration on first run |
| CLI option renames | Low | Aliases in clap, deprecation warnings |

---

## 3. Optimization Plan

### 3.1 Release Profile Optimization

#### Current State

**All crates use Rust default release profile:**
```toml
# No [profile.release] section = defaults
```

**Default Rust release settings:**
- `opt-level = 3`
- `lto = false`
- `codegen-units = 16`
- `strip = false`
- `panic = "unwind"`

#### Optimized Profile Configuration

Add to workspace root `Cargo.toml`:

```toml
[profile.release]
# Full optimization
opt-level = 3

# Link-time optimization (significant size reduction)
lto = true

# Single codegen unit (better optimization, slower build)
codegen-units = 1

# Strip debug symbols and unused code
strip = true

# Abort on panic (smaller than unwind)
panic = "abort"

# Optimize for size (alternative: keep opt-level=3 for speed)
# Uncomment for even smaller binary:
# opt-level = "s"  # or "z" for absolute smallest
```

#### Expected Size Reduction

| Optimization | Estimated Savings |
|--------------|-------------------|
| `lto = true` | 10-15% |
| `codegen-units = 1` | 5-10% |
| `strip = true` | 15-20% (debug symbols) |
| `panic = "abort"` | 2-5% |
| **Total Expected** | **30-40%** (~25MB → ~15MB) |

#### Build Time Impact

| Setting | Build Time Impact |
|---------|-------------------|
| `lto = true` | +50-100% |
| `codegen-units = 1` | +20-30% |
| **Total** | **+70-130%** (acceptable for release builds) |

**Note:** Debug builds remain fast; only release builds affected.

### 3.2 Feature Flag Design

#### Current Features

```toml
# glassware-orchestrator
[features]
default = ["llm", "binary"]
llm = []
rate-limit = []
retry = []
binary = ["glassware-core/binary"]
```

#### Proposed Feature Structure

```toml
[features]
# Default: Full-featured binary
default = ["tui", "llm", "sqlite", "binary"]

# TUI support (ratatui, crossterm)
tui = ["dep:ratatui", "dep:crossterm"]

# LLM analysis (reqwest, dotenvy)
llm = ["glassware-core/llm", "dep:reqwest"]

# SQLite caching (sqlx, rusqlite)
sqlite = ["dep:sqlx", "dep:rusqlite"]

# Binary analysis (.node files)
binary = ["glassware-core/binary"]

# Minimal build (no TUI, no LLM, no SQLite)
minimal = []

# Development features
dev = ["sqlite", "binary"]  # For testing
```

#### Feature-Gated Dependencies

```toml
[dependencies]
# ... (required deps)

# Optional deps
ratatui = { version = "0.24", optional = true }
crossterm = { version = "0.27", optional = true }
reqwest = { workspace = true, optional = true }
sqlx = { workspace = true, optional = true }
rusqlite = { version = "0.29", features = ["bundled"], optional = true }
```

#### Conditional Compilation

```rust
// In main.rs or cli.rs

#[cfg(feature = "tui")]
mod tui;

#[cfg(feature = "llm")]
use glassware_core::llm::LlmAnalyzer;

// In command handlers
#[cfg(feature = "tui")]
Commands::Campaign(CampaignCommands::Demo) => {
    tui::run_demo()?;
}

#[cfg(not(feature = "tui"))]
Commands::Campaign(CampaignCommands::Demo) => {
    return Err(anyhow::anyhow!(
        "TUI not available. Build with --features tui"
    ));
}
```

#### Expected Size Savings by Feature

| Feature Combination | Estimated Size |
|--------------------|---------------|
| Default (all features) | ~15MB (with LTO) |
| Without TUI | ~13MB |
| Without LLM | ~13.5MB |
| Without SQLite | ~11MB |
| Minimal (no features) | ~8-10MB |

### 3.3 Dependency Optimization

#### Heavy Dependency Review

**1. tokio (Full Features)**

Current: `tokio = { version = "1.35", features = ["full"] }`

**Issue:** `full` enables all features, many unused.

**Recommendation:** Use only needed features:

```toml
tokio = { version = "1.35", features = [
    "rt-multi-thread",
    "macros",
    "fs",
    "io-util",
    "net",
    "signal",
    "sync",
    "time",
    "process",
] }
```

**Savings:** ~0.5-1MB

**2. sqlx + rusqlite Duplication**

Current: Both `sqlx` (async) and `rusqlite` (sync) for SQLite.

**Issue:** Two SQLite implementations.

**Recommendation:** Evaluate using only `rusqlite` with async wrapper:

```toml
# Option A: Keep both (simpler migration)
sqlx = { workspace = true }  # For async checkpoint ops
rusqlite = { version = "0.29", features = ["bundled"] }  # For simple cache

# Option B: Use only rusqlite (more work, smaller binary)
rusqlite = { version = "0.29", features = ["bundled"] }
# Add async wrapper if needed
```

**Savings (Option B):** ~2MB

**3. reqwest (HTTP Client)**

Current: `reqwest = { version = "0.11", features = ["json", "rustls-tls"] }`

**Issue:** Large dependency (~1.5MB).

**Recommendation:** Keep for LLM/GitHub, but feature-gate:

```toml
reqwest = { workspace = true, optional = true }
```

**4. tera (Template Engine)**

Current: `tera = "1.19"`

**Issue:** Only used for markdown reports.

**Recommendation:** Feature-gate or simplify:

```toml
tera = { version = "1.19", optional = true }
```

Or use simpler string formatting for reports.

**Savings:** ~1MB if removed

#### Dependency Audit Summary

| Dependency | Current | Optimized | Savings |
|------------|---------|-----------|---------|
| tokio (full) | ~3MB | ~2MB | ~1MB |
| sqlx + rusqlite | ~4MB | ~2MB | ~2MB |
| reqwest (feature) | ~1.5MB | ~1.5MB | 0 (but optional) |
| tera (optional) | ~1MB | ~0MB | ~1MB |
| **Total** | ~9.5MB | ~5.5MB | **~4MB** |

### 3.4 Memory Optimization

#### Current Memory Usage Patterns

**Identified from code analysis:**

1. **Buffering:** Results collected in `Vec` before output
2. **Parallel scanning:** All files scanned concurrently
3. **Cache:** Full cache loaded into memory
4. **TUI state:** Full campaign state in memory

#### Optimization Opportunities

**1. Streaming Results**

Current (from orchestrator.rs):
```rust
let results = orchestrator.scan_npm_packages(&packages).await;
// All results buffered
print_results(cli, &results)?;
```

Optimized:
```rust
// Stream results as they complete
let mut stream = orchestrator.scan_npm_packages_streaming(&packages).await;
while let Some(result) = stream.next().await {
    print_result(cli, &result)?;  // Immediate output
}
```

**Savings:** 30-50% for large scans

**2. Bounded Concurrency**

Current: Fixed concurrency (default 10)

Optimized: Dynamic based on memory pressure
```rust
let concurrency = std::cmp::min(
    config.concurrency,
    available_memory_mb / 50  // 50MB per scan task
);
```

**3. Cache Streaming**

Current: Full cache loaded
```rust
let cache = Cacher::load(&path).await?;
```

Optimized: Lazy loading
```rust
let cache = Cacher::lazy_load(&path).await?;
// Only loads entries as needed
```

**4. TUI State Optimization**

Current: Full event history in memory
```rust
recent_events: VecDeque<CampaignEvent>,  // Last 100 events
```

Optimized: Ring buffer with size limit
```rust
recent_events: BoundedEventBuffer::new(50),  // Limit to 50
```

#### Expected Memory Savings

| Optimization | Current | Target | Savings |
|--------------|---------|--------|---------|
| Streaming results | ~50MB | ~30MB | ~40% |
| Bounded concurrency | ~50MB | ~35MB | ~30% |
| Lazy cache loading | ~50MB | ~40MB | ~20% |
| TUI buffer limit | ~50MB | ~45MB | ~10% |

### 3.5 CPU/Speed Optimization

#### Current Scan Speed: ~50k LOC/sec

#### Optimization Opportunities

**1. Regex Compilation Caching**

Current: Regex patterns may be recompiled.

Optimized: Use `once_cell` or `lazy_static`:
```rust
use once_cell::sync::Lazy;

static PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"...").unwrap());
```

**Improvement:** 5-10%

**2. Rayon Parallelism Tuning**

Current: Default rayon thread pool.

Optimized: Tune for workload:
```rust
rayon::ThreadPoolBuilder::new()
    .num_threads(config.jobs)
    .build_global()
    .unwrap();
```

**Improvement:** 10-15%

**3. File Reading Optimization**

Current: `fs::read_to_string()` for each file.

Optimized: Memory-mapped files for large files:
```rust
use memmap2::Mmap;

let file = File::open(path)?;
let mmap = unsafe { Mmap::map(&file)? };
let content = std::str::from_utf8(&mmap)?;
```

**Improvement:** 10-20% for large files

**4. Profile-Guided Optimization (PGO)**

Enable PGO for release builds:
```bash
# Instrument
RUSTFLAGS="-Cprofile-generate=/tmp/pgo" cargo build --release

# Run representative workload
./target/release/glassware scan /path/to/large/codebase

# Optimize with profile
RUSTFLAGS="-Cprofile-use=/tmp/pgo" cargo build --release
```

**Improvement:** 10-20%

#### Expected Speed Improvements

| Optimization | Current | Target | Improvement |
|--------------|---------|--------|-------------|
| Regex caching | 50k LOC/s | 53k LOC/s | +6% |
| Rayon tuning | 50k LOC/s | 55k LOC/s | +10% |
| Memory mapping | 50k LOC/s | 58k LOC/s | +16% |
| PGO | 50k LOC/s | 65k LOC/s | +30% |

---

## 4. Implementation Roadmap

### Week 1: Foundation

#### Day 1-2: Feature Audit

**Tasks:**
- [ ] Complete feature comparison table (Section 1.1)
- [ ] Identify all CLI options for both binaries
- [ ] Document feature gaps
- [ ] Create test checklist

**Deliverables:**
- Feature comparison spreadsheet
- Test case list

#### Day 3-5: Code Consolidation Start

**Tasks:**
- [ ] Rename `glassware-orchestrator/` to `glassware/`
- [ ] Update workspace `Cargo.toml`
- [ ] Create `src/commands/scan.rs` skeleton
- [ ] Migrate glassware-cli scanning logic
- [ ] Update CLI definitions in `src/cli.rs`

**Deliverables:**
- Working unified binary (basic scan + campaign)
- Build passes

### Week 2: Integration

#### Day 1-2: Complete Code Merge

**Tasks:**
- [ ] Merge caching implementations
- [ ] Integrate LLM support in scan command
- [ ] Add JSON Lines output to scan
- [ ] Update command routing in main.rs
- [ ] Test all subcommands

**Deliverables:**
- All features working
- Integration tests pass

#### Day 3-5: Deprecation

**Tasks:**
- [ ] Add deprecation warning to glassware-cli
- [ ] Update README.md
- [ ] Create migration guide
- [ ] Update CI/CD for new structure
- [ ] Create release notes

**Deliverables:**
- Deprecation notice in place
- Documentation updated

### Week 3: Optimization

#### Day 1-2: Build Optimization

**Tasks:**
- [ ] Add `[profile.release]` settings
- [ ] Configure feature flags
- [ ] Audit and optimize dependencies
- [ ] Measure binary size reduction
- [ ] Update build documentation

**Deliverables:**
- Optimized release build
- Size metrics documented

#### Day 3-4: Performance Tuning

**Tasks:**
- [ ] Profile memory usage
- [ ] Implement streaming where beneficial
- [ ] Tune rayon parallelism
- [ ] Enable PGO
- [ ] Benchmark scan speed

**Deliverables:**
- Performance metrics
- Tuning documentation

#### Day 5: Testing & Polish

**Tasks:**
- [ ] Run full test suite
- [ ] Fix any regressions
- [ ] Update all documentation
- [ ] Create release candidate
- [ ] Final review

**Deliverables:**
- Release candidate ready
- All tests passing

### Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Feature regression | Medium | High | Comprehensive testing, beta release |
| Breaking changes | Medium | Medium | Deprecation period, migration guide |
| Build time increase | High | Low | Only affects release builds |
| Dependency conflicts | Low | Medium | Careful dependency audit |
| Performance regression | Low | High | Benchmarking, profiling |

### Testing Strategy

#### Unit Tests

- [ ] All existing unit tests pass
- [ ] Add tests for new `scan` command
- [ ] Test feature flag combinations

#### Integration Tests

- [ ] End-to-end scan test
- [ ] Campaign execution test
- [ ] TUI demo test
- [ ] LLM integration test (mocked)

#### Performance Tests

- [ ] Binary size measurement
- [ ] Memory usage profiling
- [ ] Scan speed benchmark
- [ ] Build time measurement

#### Manual Testing

- [ ] All CLI subcommands
- [ ] All CLI options
- [ ] TUI interaction
- [ ] Campaign workflow
- [ ] Migration from old binaries

---

## 5. Success Metrics

### Binary Consolidation

| Metric | Before | After | Status |
|--------|--------|-------|--------|
| Number of binaries | 2 | 1 | ✅ |
| Total binary size | ~36MB | 10-15MB | 🎯 Target |
| Build complexity | 2 crates | 1 crate | ✅ |

### Optimization Goals

| Metric | Baseline | Target | Measurement |
|--------|----------|--------|-------------|
| Binary size (release) | ~25MB | 10-15MB | `ls -lh target/release/glassware` |
| Memory usage | ~50MB | 25-35MB | `/usr/bin/time -v` |
| Scan speed | ~50k LOC/s | ~65k LOC/s | Benchmark with test corpus |
| Build time (release) | ~2 min | ~4 min | `cargo build --release` |

### User Experience

| Metric | Target | Measurement |
|--------|--------|-------------|
| CLI compatibility | 100% | All old commands work |
| Documentation | Complete | All docs updated |
| Migration friction | Low | Clear migration guide |

---

## 6. Appendix

### A. File Reference List

**Key Files Modified:**

| File | Purpose | Lines Changed (Est.) |
|------|---------|---------------------|
| `Cargo.toml` (workspace) | Workspace config | +20 |
| `glassware/Cargo.toml` | Unified binary config | +50 |
| `glassware/src/cli.rs` | CLI definitions | +100 |
| `glassware/src/commands/scan.rs` | New scan command | ~400 (new) |
| `glassware/src/main.rs` | Command routing | +50 |
| `glassware-cli/Cargo.toml` | Deprecation notice | +5 |

**Files Removed:**

| File | Reason |
|------|--------|
| `glassware-cli/src/main.rs` | Merged into glassware |
| `glassware-cli/` (entire dir) | Deprecated |

### B. Command Mapping

| Old Command | New Command |
|-------------|-------------|
| `glassware /path/to/scan` | `glassware scan /path/to/scan` |
| `glassware-orchestrator campaign run` | `glassware campaign run` |
| `glassware-orchestrator campaign demo` | `glassware campaign demo` |
| `glassware-orchestrator scan-npm` | `glassware campaign run` (with npm sources) |

### C. Environment Variables

| Variable | Purpose | Required For |
|----------|---------|--------------|
| `GLASSWARE_LLM_BASE_URL` | LLM API endpoint | `--llm` flag |
| `GLASSWARE_LLM_API_KEY` | LLM API key | `--llm` flag |
| `GITHUB_TOKEN` | GitHub API access | GitHub scanning |
| `NVIDIA_API_KEY` | NVIDIA LLM access | `--deep-llm` flag |

### D. Build Commands

```bash
# Debug build (fast)
cargo build -p glassware

# Release build (optimized)
cargo build -p glassware --release

# Minimal build (smallest)
cargo build -p glassware --release --no-default-features --features minimal

# Without TUI
cargo build -p glassware --release --no-default-features --features llm,sqlite,binary

# With all features
cargo build -p glassware --release --all-features
```

---

**Document End**

*This consolidation plan provides a comprehensive roadmap for merging the two binaries while achieving significant optimization goals. The phased approach minimizes risk and ensures no functionality is lost during the migration.*
