# Workspace Restructuring Plan

**Version:** 1.0
**Date:** March 23, 2026
**Status:** Ready for Implementation

---

## Overview

This document provides step-by-step instructions for restructuring the workspace from two binaries to a single unified binary.

---

## Current Structure

```
glassworks/
├── Cargo.toml                        # Workspace root
├── Cargo.lock
├── .gitignore
├── README.md
│
├── glassware-core/                   # Library crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── ... (modules)
│
├── glassware-cli/                    # Binary 1 (TO BE DEPRECATED)
│   ├── Cargo.toml
│   └── src/
│       └── main.rs                   # 1,019 lines - simple scanner
│
└── glassware-orchestrator/           # Binary 2 (TO BE RENAMED)
    ├── Cargo.toml
    └── src/
        ├── main.rs                   # 1,889 lines
        ├── cli.rs                    # 350+ lines
        ├── campaign/                 # 11 modules
        ├── tui/                      # 2 modules
        └── ... (26 files total)
```

---

## Target Structure

```
glassworks/
├── Cargo.toml                        # Workspace root (updated)
├── Cargo.lock
├── .gitignore
├── README.md                         # Updated with new commands
├── QWEN.md                           # Updated
│
├── glassware-core/                   # Library crate (unchanged)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── ... (modules)
│
├── glassware/                        # NEW: Unified binary
│   ├── Cargo.toml                    # Merged from orchestrator + cli
│   └── src/
│       ├── main.rs                   # Updated entry point
│       ├── cli.rs                    # Expanded with scan command
│       ├── commands/                 # NEW: Command modules
│       │   ├── mod.rs                # NEW
│       │   ├── scan.rs               # NEW: From glassware-cli
│       │   └── campaign.rs           # Existing (refactored)
│       ├── campaign/                 # Existing (11 modules)
│       ├── tui/                      # Existing (2 modules)
│       └── ... (26 existing files)
│
└── DEPRECATION-NOTICE.md             # NEW: For old binary users
```

---

## Migration Steps

### Step 1: Backup Current State

```bash
# Ensure everything is committed
git add .
git commit -m "Pre-consolidation backup"

# Create backup branch
git branch backup/pre-consolidation
```

---

### Step 2: Rename glassware-orchestrator to glassware

```bash
# Rename directory
mv glassware-orchestrator glassware

# Update package name in glassware/Cargo.toml
# Change: name = "glassware-orchestrator"
# To:     name = "glassware"
```

**glassware/Cargo.toml changes:**
```toml
[package]
name = "glassware"                    # Changed from glassware-orchestrator
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Unified GlassWare detection tool"  # Updated
authors = ["glassware team"]

[dependencies]
# Keep all existing dependencies
# Add new ones from glassware-cli (see Step 3)
```

---

### Step 3: Merge Dependencies from glassware-cli

**Add to glassware/Cargo.toml:**

```toml
[dependencies]
# Existing dependencies (keep all)...

# NEW: From glassware-cli (for scan command)
rayon = "1.10"                        # Parallel file scanning
ignore = "0.4"                        # File walking with gitignore
colored = "2"                         # Terminal colors (if not already covered)
globset = "0.4"                       # Glob pattern matching

# Update glassware-core features
glassware-core = { workspace = true, features = ["full", "binary"] }  # Added "full"
```

---

### Step 4: Create Commands Module Structure

```bash
# Create commands directory
mkdir -p glassware/src/commands
```

**Create `glassware/src/commands/mod.rs`:**
```rust
//! Command modules for unified glassware binary
//!
//! This module contains all subcommand implementations.

pub mod scan;       // Simple file/directory scanning
pub mod campaign;   // Campaign management

// Re-export for convenience
pub use scan::CmdScan;
pub use campaign::CmdCampaign;
```

---

### Step 5: Migrate Scan Command

**Create `glassware/src/commands/scan.rs`:**

This file will contain the migrated logic from `glassware-cli/src/main.rs`.

```rust
//! Simple file and directory scanning command
//!
//! Migrated from glassware-cli crate.

use anyhow::{Context, Result};
use clap::Args;
use glassware_core::{scan, Finding, Severity};
use rayon::prelude::*;
use ignore::{WalkBuilder, OverrideBuilder};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

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

    /// Suppress output, only set exit code
    #[arg(short, long)]
    pub quiet: bool,

    // ... (rest of CLI options from glassware-cli)
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Pretty,
    Json,
    Sarif,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum SeverityLevel {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl CmdScan {
    pub fn run(&self) -> Result<()> {
        // Migrated from glassware-cli/src/main.rs
        // Implementation details...
        todo!("Migrate from glassware-cli")
    }
}
```

**Migration Checklist:**
- [ ] Copy CLI args struct from glassware-cli
- [ ] Copy `collect_files()` function
- [ ] Copy `should_scan_file()` function
- [ ] Copy parallel scanning logic (rayon)
- [ ] Copy output formatters (pretty, json, sarif)
- [ ] Copy LLM integration
- [ ] Copy caching logic (or use SQLite from orchestrator)
- [ ] Update imports
- [ ] Test compilation

---

### Step 6: Update Main CLI Entry Point

**Update `glassware/src/cli.rs`:**

Add the new `scan` subcommand to the CLI structure.

```rust
use clap::{Parser, Subcommand, Args};

#[derive(Debug, Clone, Parser)]
#[command(name = "glassware")]
#[command(about = "Unified GlassWare detection tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    // ... (global options)
}

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    /// Scan files and directories for GlassWare attacks
    Scan(CmdScan),

    /// Campaign management
    #[command(subcommand)]
    Campaign(CampaignCommands),

    // ... (existing commands)
}
```

**Update `glassware/src/main.rs`:**

Add handler for new `scan` command.

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing...

    match cli.command {
        Commands::Scan(cmd) => {
            cmd.run()?;  // NEW: Handle scan command
        }
        Commands::Campaign(cmd) => {
            cmd_campaign(&cli, cmd).await?;
        }
        // ... (existing commands)
    }

    Ok(())
}
```

---

### Step 7: Update Workspace Configuration

**Update root `Cargo.toml`:**

```toml
[workspace]
members = [
    "glassware-core",
    "glassware",                       # Changed from glassware-orchestrator
    # "glassware-cli",                 # TEMPORARILY COMMENTED (deprecation)
]
resolver = "2"
```

---

### Step 8: Add Release Profile Optimization

**Add to root `Cargo.toml`:**

```toml
# Add at the end of the file
[profile.release]
opt-level = 3                         # Full optimization
lto = true                            # Link-time optimization
codegen-units = 1                     # Single codegen unit (slower build, smaller binary)
strip = true                          # Strip debug symbols
panic = "abort"                       # Smaller panic handling

# Debug builds remain fast
[profile.dev]
opt-level = 0
debug = true
```

---

### Step 9: Add Feature Flags

**Update `glassware/Cargo.toml` features:**

```toml
[features]
default = ["tui", "llm", "reports"]

# TUI support (campaign demo, campaign monitor)
tui = ["ratatui", "crossterm"]

# LLM analysis (Tier 1 and Tier 2)
llm = ["reqwest"]

# Markdown report generation
reports = ["tera"]

# Minimal build (no TUI, no LLM, no reports)
minimal = []

# Binary scanning (.node files)
binary = ["glassware-core/binary"]

# Full feature set (for development)
full = ["tui", "llm", "reports", "binary"]
```

**Update dependencies to be optional:**

```toml
[dependencies]
# ... (required dependencies)

# Optional: TUI
ratatui = { version = "0.24", optional = true }
crossterm = { version = "0.27", optional = true }

# Optional: LLM
reqwest = { workspace = true, optional = true }

# Optional: Reports
tera = { version = "1.19", optional = true }
```

---

### Step 10: Test Build

```bash
# Test default build
cd glassware
cargo build --release

# Check binary size
ls -lh target/release/glassware

# Test minimal build
cargo build --release --no-default-features --features minimal

# Test TUI-less build
cargo build --release --no-default-features --features llm,reports

# Run basic functionality test
./target/release/glassware --help
./target/release/glassware scan --help
./target/release/glassware campaign --help
```

---

### Step 11: Deprecate Old Binaries

**Create `DEPRECATION-NOTICE.md`:**

```markdown
# Deprecation Notice

**Effective:** Version 0.9.0
**Removal:** Version 1.0.0

## Deprecated Binaries

The following binaries are deprecated and will be removed in version 1.0.0:

- `glassware` (from glassware-cli crate)
- `glassware-orchestrator` (renamed to `glassware`)

## Migration Guide

### Old: Simple Scanning
```bash
# Old (v0.8.0)
glassware /path/to/code --format json

# New (v0.9.0+)
glassware scan /path/to/code --format json
```

### Old: Campaign Scanning
```bash
# Old (v0.8.0)
glassware-orchestrator campaign run wave6.toml

# New (v0.9.0+)
glassware campaign run wave6.toml
```

## Timeline

- **v0.9.0 (March 2026):** Deprecation warnings added
- **v1.0.0 (April 2026):** Old binaries removed

## Questions?

See `docs/binaryconsolidation/CONSOLIDATION-PLAN.md` for details.
```

**Add deprecation warning to glassware-cli/src/main.rs:**

```rust
fn main() {
    eprintln!("⚠️  DEPRECATION WARNING: The 'glassware' binary is deprecated.");
    eprintln!("  Please use 'glassware scan' instead.");
    eprintln!("  This binary will be removed in v1.0.0.");
    eprintln!();

    // ... (rest of main)
}
```

---

### Step 12: Update Documentation

**Update README.md:**

```markdown
## Quick Start

### Install

```bash
cargo build --release
```

### Commands

```bash
# Scan files/directories (new!)
./target/release/glassware scan /path/to/code

# Run campaign
./target/release/glassware campaign run campaigns/wave6.toml

# TUI demo
./target/release/glassware campaign demo

# Generate report
./target/release/glassware campaign report <case-id>
```
```

**Update QWEN.md:**

Update the build commands and project structure sections.

---

### Step 13: Final Testing

```bash
# Full test suite
cargo test --workspace

# Test scan command
./target/release/glassware scan examples/

# Test campaign command
./target/release/glassware campaign demo

# Test all subcommands
./target/release/glassware --help
./target/release/glassware scan --help
./target/release/glassware campaign --help
```

---

## Rollback Plan

If issues arise during migration:

```bash
# Revert to backup branch
git checkout backup/pre-consolidation

# Restore old structure
git restore .

# Rebuild old binaries
cargo build --release
```

---

## Verification Checklist

After migration, verify:

- [ ] `glassware scan` works (old glassware-cli functionality)
- [ ] `glassware campaign run` works
- [ ] `glassware campaign demo` works (TUI)
- [ ] `glassware campaign query` works (LLM)
- [ ] All output formats work (pretty, json, sarif, jsonl)
- [ ] Caching works
- [ ] LLM analysis works
- [ ] All tests pass
- [ ] Binary size is reduced
- [ ] No breaking changes for campaign users

---

## Post-Migration Cleanup

After successful migration and testing:

```bash
# Remove old glassware-cli crate
rm -rf glassware-cli

# Update .gitignore
# Remove references to old binary names

# Update CI/CD pipelines
# Change binary names in workflows
```

---

**Next Steps:** After completing this restructuring, proceed to Week 2 (Size Optimization) tasks in `IMPLEMENTATION-TRACKER.md`.
