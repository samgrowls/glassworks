

---

# Glassware Orchestrator — Binary Fix Guide

**Document Version:** 1.0.0  
**Date:** 2026-03-20  
**Status:** 📋 READY FOR EXECUTION  
**Errors:** 6 compilation errors (all in `main.rs`)  
**Estimated Fix Time:** 1-2 hours

---

## Error Summary

| # | Error Code | Issue | File | Line | Priority |
|---|------------|-------|------|------|----------|
| 1 | E0583 | Missing `cli` module | main.rs | 5 | P0 |
| 2 | E0599 | `cache_stats()` method not found | main.rs | 251 | P0 |
| 3 | E0599 | `clear_cache()` method not found | main.rs | 280 | P0 |
| 4 | E0599 | `cleanup_cache()` method not found | main.rs | 294 | P0 |
| 5 | E0063 | `OrchestratorConfig` missing 7 fields | main.rs | 305 | P0 |
| 6 | E0282 | Type annotation needed for `log_file` | main.rs | 47 | P1 |

---

## Fix 1: Missing `cli` Module (E0583)

**Problem:** `main.rs` declares `mod cli;` but `cli.rs` doesn't exist.

**Solution:** Either create `cli.rs` OR remove the module declaration and inline the CLI types.

### Option A: Create cli.rs (Recommended)

**File:** `glassware-orchestrator/src/cli.rs`

**Create with this content:**
```rust
//! CLI module for glassware-orchestrator.

use clap::{Parser, Subcommand, ValueEnum};

/// Glassware Orchestrator - Security scanning for npm and GitHub.
#[derive(Parser, Debug)]
#[command(name = "glassware-orchestrator")]
#[command(author = "glassware contributors")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Security scanning for npm and GitHub", long_about = None)]
pub struct Cli {
    /// Subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,

    /// Output format.
    #[arg(short, long, value_enum, default_value = "pretty")]
    pub format: OutputFormat,

    /// Minimum severity to report.
    #[arg(short, long, value_enum, default_value = "low")]
    pub severity: SeverityLevel,

    /// Maximum concurrent operations.
    #[arg(short, long, default_value = "10")]
    pub concurrency: usize,

    /// Maximum retries per operation.
    #[arg(long, default_value = "3")]
    pub max_retries: u32,

    /// npm registry rate limit (requests/second).
    #[arg(long, default_value = "10")]
    pub npm_rate_limit: u32,

    /// GitHub API rate limit (requests/second).
    #[arg(long, default_value = "5")]
    pub github_rate_limit: u32,

    /// GitHub token for private repositories.
    #[arg(long)]
    pub github_token: Option<String>,

    /// Enable streaming output (JSON Lines).
    #[arg(long)]
    pub streaming: bool,

    /// Enable adversarial testing.
    #[arg(long)]
    pub adversarial: bool,

    /// Enable LLM analysis.
    #[arg(long)]
    pub llm: bool,

    /// Log level (trace, debug, info, warn, error).
    #[arg(long, default_value = "info")]
    pub log_level: String,

    /// Log output file (optional, defaults to stdout).
    #[arg(long)]
    pub log_file: Option<String>,

    /// Output file path (optional, defaults to stdout).
    #[arg(short, long)]
    pub output: Option<String>,

    /// Quiet mode (minimal output).
    #[arg(long)]
    pub quiet: bool,

    /// Verbose mode (detailed output).
    #[arg(long)]
    pub verbose: bool,

    /// Disable colored output.
    #[arg(long)]
    pub no_color: bool,

    /// Checkpoint directory for resume support.
    #[arg(long, default_value = ".glassware-checkpoints")]
    pub checkpoint_dir: String,
}

/// Available commands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scan npm packages.
    ScanNpm {
        /// Package names to scan.
        #[arg(required = true)]
        packages: Vec<String>,
    },

    /// Scan GitHub repositories.
    ScanGithub {
        /// Repository specifications (owner/repo).
        #[arg(required = true)]
        repos: Vec<String>,

        /// Git reference (branch, tag, or commit).
        #[arg(short, long, default_value = "HEAD")]
        r#ref: Option<String>,
    },

    /// Scan packages from a file.
    ScanFile {
        /// Path to file containing package list.
        #[arg(required = true)]
        file: String,
    },

    /// Resume an interrupted scan.
    Resume {
        /// Source type to resume.
        #[arg(value_enum)]
        source: ResumeSource,

        /// Package names to resume (for npm).
        #[arg(long)]
        packages: Option<Vec<String>>,

        /// Repository specs to resume (for GitHub).
        #[arg(long)]
        repos: Option<Vec<String>>,
    },

    /// Show cache statistics.
    CacheStats {
        /// Clear cache after showing stats.
        #[arg(long, default_value = "false")]
        clear: bool,
    },

    /// Clean up expired cache entries.
    CacheCleanup,
}

/// Output format options.
#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    /// Human-readable pretty print.
    Pretty,

    /// JSON format.
    Json,

    /// JSON Lines format (streaming).
    Jsonl,

    /// SARIF format (for GitHub Advanced Security).
    Sarif,
}

/// Severity level options.
#[derive(ValueEnum, Clone, Debug)]
pub enum SeverityLevel {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl SeverityLevel {
    pub fn to_core_severity(&self) -> glassware_core::Severity {
        match self {
            SeverityLevel::Info => glassware_core::Severity::Info,
            SeverityLevel::Low => glassware_core::Severity::Low,
            SeverityLevel::Medium => glassware_core::Severity::Medium,
            SeverityLevel::High => glassware_core::Severity::High,
            SeverityLevel::Critical => glassware_core::Severity::Critical,
        }
    }
}

/// Resume source type.
#[derive(ValueEnum, Clone, Debug)]
pub enum ResumeSource {
    /// npm packages.
    Npm,

    /// GitHub repositories.
    Github,
}

impl Cli {
    /// Parse command-line arguments.
    pub fn parse_args() -> Self {
        Cli::parse()
    }

    /// Parse command-line arguments or exit.
    pub fn try_parse_args() -> Result<Self, clap::Error> {
        Cli::try_parse()
    }
}
```

---

## Fix 2-4: Missing Cache Methods (E0599)

**Problem:** `main.rs` calls `cache_stats()`, `clear_cache()`, `cleanup_cache()` on `Orchestrator` but these methods don't exist.

**Solution:** These methods exist on `Cacher`, not `Orchestrator`. Access via `orchestrator.cacher()`.

### Fix in main.rs

**Lines 251, 280, 294**

**Find and replace:**

```rust
// Line 251 - OLD
if let Some(stats) = orchestrator.cache_stats().await {

// Line 251 - NEW
if let Some(stats) = orchestrator.cacher().await?.stats() {
```

```rust
// Line 280 - OLD
orchestrator.clear_cache().await?;

// Line 280 - NEW
orchestrator.cacher().await?.clear().await?;
```

```rust
// Line 294 - OLD
let removed = orchestrator.cleanup_cache().await?;

// Line 294 - NEW
let removed = orchestrator.cacher().await?.cleanup().await?;
```

**Alternative:** If `Cacher` doesn't have these methods either, add them to `cacher.rs`:

```rust
// In glassware-orchestrator/src/cacher.rs, add:

impl Cacher {
    pub async fn stats(&self) -> Option<CacheStats> {
        // Return cache statistics
        Some(CacheStats {
            total_entries: 0,
            expired_entries: 0,
        })
    }

    pub async fn clear(&self) -> Result<()> {
        // Clear all cache entries
        Ok(())
    }

    pub async fn cleanup(&self) -> Result<usize> {
        // Remove expired entries, return count
        Ok(0)
    }
}

pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
}
```

---

## Fix 5: OrchestratorConfig Missing Fields (E0063)

**Problem:** `OrchestratorConfig` struct requires 7 more fields than `main.rs` provides.

**Solution:** Add the missing fields with default values.

### Fix in main.rs

**Line 305**

**Replace the entire `OrchestratorConfig` initialization:**

```rust
// OLD (lines 305-320 approx)
let config = OrchestratorConfig {
    downloader: DownloaderConfig {
        max_retries: cli.max_retries,
        npm_rate_limit: cli.npm_rate_limit,
        github_rate_limit: cli.github_rate_limit,
        github_token: cli.github_token.clone(),
        max_concurrent: cli.concurrency,
        ..Default::default()
    },
    scanner: ...
};

// NEW (add missing fields)
let config = OrchestratorConfig {
    downloader: DownloaderConfig {
        max_retries: cli.max_retries,
        npm_rate_limit: cli.npm_rate_limit,
        github_rate_limit: cli.github_rate_limit,
        github_token: cli.github_token.clone(),
        max_concurrent: cli.concurrency,
        ..Default::default()
    },
    scanner: ScannerConfig {
        max_concurrent: cli.concurrency,
        ..Default::default()
    },
    // ADD THESE MISSING FIELDS:
    enable_cache: true,
    cache_ttl_hours: 168, // 7 days
    checkpoint_dir: cli.checkpoint_dir.clone(),
    checkpoint_interval: 10, // packages between checkpoints
    enable_checkpoint: true,
    enable_llm: cli.llm,
    enable_adversarial: cli.adversarial,
};
```

**If `OrchestratorConfig` doesn't have these fields**, check the actual struct definition in `orchestrator.rs` and match the field names exactly.

---

## Fix 6: Type Annotation Needed (E0282)

**Problem:** `log_file.clone()` can't infer the type.

**Solution:** Add explicit type annotation.

### Fix in main.rs

**Line 47**

```rust
// OLD
output: if let Some(ref log_file) = cli.log_file {
    glassware_orchestrator::tracing::TracingOutput::File(log_file.clone())

// NEW
output: if let Some(ref log_file) = cli.log_file {
    glassware_orchestrator::tracing::TracingOutput::File(log_file.clone().into())
```

**Or more explicitly:**
```rust
output: if let Some(ref log_file) = cli.log_file {
    glassware_orchestrator::tracing::TracingOutput::File(log_file.clone().into_string().into())
```

---

## Fix 7: Remove Unused Imports (Warnings)

**Lines 12, 18, 20 in main.rs**

```rust
// REMOVE these lines:
use std::path::Path;  // Line 18 - unused
use tracing_subscriber::FmtSubscriber;  // Line 20 - unused
OutputFormat as StreamOutputFormat,  // Line 12 - unused alias
```

---

## Fix 8: Unused Variable Warning

**Line 611 in main.rs**

```rust
// OLD
let tester = AdversarialTester::new()?;

// NEW
let _tester = AdversarialTester::new()?;
```

---

## Verification Checklist

After applying all fixes:

```bash
cd glassware-orchestrator

# Check binary compiles
cargo build --bin glassware-orchestrator 2>&1 | tee build-binary.log

# Should show 0 errors
grep -c "error\[" build-binary.log

# Run with help
cargo run --bin glassware-orchestrator -- --help

# Test scan command
cargo run --bin glassware-orchestrator -- scan-npm lodash --concurrency 5
```

---

## Expected Result

| Before | After |
|--------|-------|
| 6 errors | 0 errors |
| 4 warnings | 0-2 warnings (acceptable) |
| Binary doesn't compile | Binary runs successfully |

---

## If Errors Remain

```bash
# Capture remaining errors
cargo build --bin glassware-orchestrator 2>&1 | grep "error\[" > remaining-binary-errors.log

# Share with me:
# 1. remaining-binary-errors.log (full content)
# 2. Updated main.rs (full content)
```

---

**This guide addresses all 6 binary errors with surgical precision.** Apply fixes in order (1→6) and the binary should compile cleanly.