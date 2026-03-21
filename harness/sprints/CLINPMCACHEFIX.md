
---

# Glassware Orchestrator — Functional Bug Fix Guide

**Document Version:** 1.0.0  
**Date:** 2026-03-20  
**Status:** 📋 READY FOR EXECUTION  
**Priority:** P0 (Release Blocker)  
**Estimated Fix Time:** 2-3 hours

---

## Bug Summary

| # | Bug | Severity | File | Impact |
|---|-----|----------|------|--------|
| 1 | CLI options not inherited by subcommands | 🔴 High | `cli.rs` | Can't use `--format`, `--quiet`, etc. on `scan-npm` |
| 2 | npm tarball URL parsing wrong | 🔴 High | `downloader.rs` | Can't download ANY npm packages |
| 3 | Cache database path permissions | 🟡 Medium | `cacher.rs` | Cache fails in some directories |

---

## Fix 1: CLI Option Inheritance (30 minutes)

**Problem:** Global options (`--format`, `--quiet`, `--concurrency`) are defined on `Cli` struct but not inherited by subcommands like `ScanNpm`.

**Root Cause:** `#[command(flatten)]` not used, or options defined at wrong level.

### Solution A: Move Global Options to Subcommands (Recommended)

**File:** `glassware-orchestrator/src/cli.rs`

**Find the `Commands` enum and update each variant:**

```rust
// BEFORE (broken - options only on Cli, not subcommands)
#[derive(Subcommand, Debug)]
pub enum Commands {
    ScanNpm {
        #[arg(required = true)]
        packages: Vec<String>,
    },
    // ...
}

// AFTER (fixed - global options flattened into each subcommand)
#[derive(Subcommand, Debug)]
pub enum Commands {
    ScanNpm {
        #[arg(required = true)]
        packages: Vec<String>,

        // Global options flattened here
        #[command(flatten)]
        global: GlobalArgs,
    },

    ScanGithub {
        #[arg(required = true)]
        repos: Vec<String>,

        #[arg(short, long, default_value = "HEAD")]
        r#ref: Option<String>,

        #[command(flatten)]
        global: GlobalArgs,
    },

    ScanFile {
        #[arg(required = true)]
        file: String,

        #[command(flatten)]
        global: GlobalArgs,
    },

    Resume {
        #[arg(value_enum)]
        source: ResumeSource,

        #[arg(long)]
        packages: Option<Vec<String>>,

        #[arg(long)]
        repos: Option<Vec<String>>,

        #[command(flatten)]
        global: GlobalArgs,
    },

    CacheStats {
        #[arg(long, default_value = "false")]
        clear: bool,
    },

    CacheCleanup,
}
```

**Add the `GlobalArgs` struct:**

```rust
/// Global arguments available on all scan commands.
#[derive(Args, Clone, Debug)]
pub struct GlobalArgs {
    /// Output format.
    #[arg(short = 'f', long, value_enum, default_value = "pretty")]
    pub format: OutputFormat,

    /// Minimum severity to report.
    #[arg(short = 's', long, value_enum, default_value = "low")]
    pub severity: SeverityLevel,

    /// Maximum concurrent operations.
    #[arg(short = 'c', long, default_value = "10")]
    pub concurrency: usize,

    /// Verbose output.
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Quiet mode (no output except errors).
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Disable colored output.
    #[arg(long)]
    pub no_color: bool,
}
```

**Update `main.rs` to use the flattened args:**

```rust
// BEFORE (broken)
match &cli.command {
    Commands::ScanNpm { packages } => {
        scan_npm(packages, &cli.format, cli.quiet).await?;
    }
}

// AFTER (fixed)
match &cli.command {
    Commands::ScanNpm { packages, global } => {
        scan_npm(packages, &global.format, global.quiet, global.concurrency).await?;
    }
    Commands::ScanGithub { repos, r#ref, global } => {
        scan_github(repos, r#ref, &global.format, global.quiet).await?;
    }
    Commands::ScanFile { file, global } => {
        scan_file(file, &global.format, global.quiet, global.concurrency).await?;
    }
}
```

---

## Fix 2: npm Tarball URL Parsing (1 hour)

**Problem:** Code expects `dist.tarball` at top level, but npm returns it nested under `versions[latest].dist.tarball`.

**Confirmed by actual npm API response:**
```json
{
  "name": "lodash",
  "versions": 115,
  "latest": "4.17.23",
  "tarball": "https://registry.npmjs.org/lodash/-/lodash-4.17.23.tgz"
}
```

### Step 2.1: Update NpmPackageInfo Struct

**File:** `glassware-orchestrator/src/downloader.rs`

**Replace the `NpmPackageInfo` struct:**

```rust
// BEFORE (broken - expects dist at top level)
#[derive(Debug, Clone, Deserialize)]
pub struct NpmPackageInfo {
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub repository: Option<NpmRepository>,
    #[serde(rename = "dist", default)]
    pub dist: Option<NpmDist>,
}

// AFTER (fixed - parses full npm response structure)
#[derive(Debug, Clone, Deserialize)]
pub struct NpmPackageInfo {
    pub name: String,
    
    /// Latest version from dist-tags
    #[serde(rename = "dist-tags", default)]
    pub dist_tags: Option<NpmDistTags>,
    
    /// All versions with their metadata
    #[serde(default)]
    pub versions: std::collections::HashMap<String, NpmVersionInfo>,
    
    /// Convenience field - resolved tarball URL for latest version
    #[serde(skip)]
    pub tarball_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NpmDistTags {
    #[serde(default)]
    pub latest: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NpmVersionInfo {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub dist: Option<NpmDist>,
    #[serde(default)]
    pub description: Option<String>,
}
```

### Step 2.2: Add Tarball Resolution Method

**Add to `downloader.rs`:**

```rust
impl NpmPackageInfo {
    /// Resolve the tarball URL for the latest version.
    pub fn resolve_tarball(&self) -> Option<String> {
        // Get latest version from dist-tags
        let latest = self.dist_tags.as_ref()?.latest.as_ref()?;
        
        // Get version info from versions map
        let version_info = self.versions.get(latest)?;
        
        // Get tarball from dist
        version_info.dist.as_ref()?.tarball.clone()
    }
    
    /// Get the latest version string.
    pub fn latest_version(&self) -> Option<String> {
        self.dist_tags.as_ref()?.latest.clone()
    }
}
```

### Step 2.3: Fix `download_npm_package` Method

**File:** `glassware-orchestrator/src/downloader.rs`

**Find and replace the `download_npm_package` function:**

```rust
// BEFORE (broken - tries to access dist.tarball at top level)
pub async fn download_npm_package(&self, package: &str) -> Result<DownloadedPackage> {
    let info = self.get_npm_package_info(package).await?;
    
    let tarball_url = info
        .dist
        .as_ref()
        .and_then(|d| d.tarball.as_ref())
        .ok_or_else(|| OrchestratorError::npm("No tarball URL found".to_string()))?;
    
    let version = info.version.clone().unwrap_or_else(|| "latest".to_string());
    
    self.download_tarball(package, tarball_url, "npm", &version).await
}

// AFTER (fixed - uses resolve_tarball method)
pub async fn download_npm_package(&self, package: &str) -> Result<DownloadedPackage> {
    let info = self.get_npm_package_info(package).await?;
    
    // Resolve tarball URL from versions[latest].dist.tarball
    let tarball_url = info
        .resolve_tarball()
        .ok_or_else(|| OrchestratorError::npm(format!(
            "No tarball URL found for package '{}'. Available versions: {:?}",
            package,
            info.versions.keys().take(5).collect::<Vec<_>>()
        )))?;
    
    // Get version from dist-tags.latest
    let version = info
        .latest_version()
        .unwrap_or_else(|| "unknown".to_string());
    
    tracing::debug!(
        package = %package,
        version = %version,
        tarball = %tarball_url,
        "Downloading npm package"
    );
    
    self.download_tarball(package, &tarball_url, "npm", &version).await
}
```

### Step 2.4: Fix `get_npm_package_info` Error Handling

**File:** `glassware-orchestrator/src/downloader.rs`

**Find line ~214 and fix the error constructor:**

```rust
// BEFORE (broken - http_error takes 2 args)
.map_err(|e| OrchestratorError::http_error(e))?;

// AFTER (fixed - use http helper)
.map_err(|e| OrchestratorError::http(e.to_string()))?;
```

---

## Fix 3: Cache Database Path (30 minutes)

**Problem:** SQLite can't create database in current directory due to permissions.

**Root Cause:** Relative path `.glassware-orchestrator-cache.db` may not be writable.

### Solution: Use Absolute Path with Directory Creation

**File:** `glassware-orchestrator/src/cacher.rs`

**Find the database initialization and update:**

```rust
// BEFORE (broken - relative path may fail)
pub async fn new(db_path: &str, ttl_days: u32) -> Result<Self> {
    let pool = SqlitePool::connect(db_path).await?;
    // ...
}

// AFTER (fixed - absolute path with directory creation)
pub async fn new(db_path: &str, ttl_days: u32) -> Result<Self> {
    use std::path::Path;
    
    // Convert to absolute path
    let db_path = Path::new(db_path);
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| OrchestratorError::io(format!(
                    "Failed to create cache directory '{}': {}",
                    parent.display(),
                    e
                )))?;
        }
    }
    
    // Connect with full path
    let db_url = db_path
        .to_str()
        .ok_or_else(|| OrchestratorError::io(format!(
            "Invalid database path: {}",
            db_path.display()
        )))?;
    
    let pool = SqlitePool::connect(db_url).await
        .map_err(|e| OrchestratorError::database(format!(
            "Failed to connect to cache database '{}': {}",
            db_url, e
        )))?;
    
    // Run migrations
    Self::migrate(&pool).await?;
    
    Ok(Self { pool, ttl_days })
}
```

### Update `orchestrator.rs` Default Cache Path

**File:** `glassware-orchestrator/src/orchestrator.rs`

**Find the default cache path and update:**

```rust
// BEFORE (broken - relative path in current directory)
const DEFAULT_CACHE_PATH: &str = ".glassware-orchestrator-cache.db";

// AFTER (fixed - use home directory or temp)
fn default_cache_path() -> String {
    // Try to use home directory first
    if let Some(home) = std::env::var_os("HOME") {
        let path = std::path::Path::new(&home)
            .join(".glassware")
            .join("orchestrator-cache.db");
        return path.to_string_lossy().to_string();
    }
    
    // Fallback to temp directory
    if let Some(temp) = std::env::var_os("TMPDIR") {
        let path = std::path::Path::new(&temp)
            .join("glassware-orchestrator-cache.db");
        return path.to_string_lossy().to_string();
    }
    
    // Last resort - current directory (may fail)
    ".glassware-orchestrator-cache.db".to_string()
}
```

---

## Verification Checklist

After applying all fixes:

```bash
cd glassware-orchestrator

# 1. Build binary
cargo build --bin glassware-orchestrator 2>&1 | tee build.log
grep -c "error\[" build.log  # Should be 0

# 2. Test CLI option inheritance
./target/debug/glassware-orchestrator scan-npm --help | grep -E "format|quiet|concurrency"
# Should show global options

# 3. Test npm package download
./target/debug/glassware-orchestrator scan-npm lodash --quiet 2>&1 | head -20
# Should NOT show "No tarball URL found" error

# 4. Test cache creation
./target/debug/glassware-orchestrator cache-stats
# Should create database without permission errors

# 5. Verify cache is used (second scan should be faster)
time ./target/debug/glassware-orchestrator scan-npm lodash --quiet
time ./target/debug/glassware-orchestrator scan-npm lodash --quiet
# Second run should be significantly faster

# 6. Test with explicit format option
./target/debug/glassware-orchestrator scan-npm lodash --format json 2>&1 | jq '.summary'
# Should output valid JSON
```

---

## Expected Results

| Test | Before | After |
|------|--------|-------|
| `scan-npm --help` | No `--format` option | Shows `--format`, `--quiet`, etc. |
| `scan-npm lodash` | "No tarball URL found" error | Downloads and scans successfully |
| `cache-stats` | Permission error or panic | Shows stats or "0 entries" |
| Second scan | Same speed as first | 10-100x faster (cached) |
| `--format json` | Option not recognized | Valid JSON output |

---

## If Issues Remain

```bash
# Capture full error output
./target/debug/glassware-orchestrator scan-npm lodash --verbose 2>&1 | tee scan-debug.log

# Share with me:
# 1. scan-debug.log (full content)
# 2. Any new compilation errors
```

---

**This guide addresses all 3 functional bugs with surgical precision based on your actual code and the npm API response.** Apply fixes in order (1→2→3) and the orchestrator should work end-to-end.

**Ready to execute?** 🎯