```markdown
# Rust Orchestrator — Compilation Issues Resolution Plan

**Document Version:** 1.0.0  
**Date:** 2026-03-20  
**Status:** 📋 READY FOR EXECUTION  
**Priority:** P0 (Release Blocker)  
**Estimated Effort:** 6-10 hours

---

## Executive Summary

The Rust Orchestrator has **67+ compilation errors** concentrated in these areas:

| Category | Error Count | Priority | Estimated Time |
|----------|-------------|----------|----------------|
| Dependency/Version Mismatch | ~15 | P0 | 1h |
| tracing_subscriber API | ~12 | P0 | 2h |
| Error Type Constructors | ~18 | P0 | 3h |
| Module/Import Issues | ~10 | P0 | 1h |
| Async/Tokio Patterns | ~8 | P1 | 2h |
| SQLx/Database | ~4 | P1 | 1h |

---

## Phase 1: Dependency & Configuration Fixes (1 hour)

### 1.1 Update Root Workspace Cargo.toml

**File:** `Cargo.toml`

**Current Issue:** `glassware-orchestrator` not in workspace members.

**Fix:**
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
# Core
glassware-core = { path = "./glassware-core", version = "0.7.0" }

# Async
tokio = { version = "1.35", features = ["full"] }
tokio-retry = "0.3"

# CLI
clap = { version = "4.4", features = ["derive"] }

# HTTP
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Database
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-rustls"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Utilities
futures = "0.3"
walkdir = "2.4"
uuid = { version = "1.6", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
```

---

### 1.2 Fix glassware-orchestrator/Cargo.toml

**File:** `glassware-orchestrator/Cargo.toml`

**Current Issue:** Dependencies not aligned with workspace, missing features.

**Fix:**
```toml
[package]
name = "glassware-orchestrator"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Rust-based package scanning orchestrator"
authors = ["glassware team"]

[dependencies]
# Core - use workspace dependency
glassware-core = { workspace = true }

# Async runtime
tokio = { workspace = true }
tokio-retry = { workspace = true }

# CLI
clap = { workspace = true }

# HTTP client
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
tokio-semaphore = "0.1"

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3.8"

[[bin]]
name = "glassware-orchestrator"
path = "src/main.rs"
```

---

### 1.3 Verify Rust Version

**Command:**
```bash
rustc --version
# Must be 1.70 or later
# If not: rustup update stable
```

---

## Phase 2: Module Structure Fixes (1 hour)

### 2.1 Create Module Declaration File

**File:** `glassware-orchestrator/src/lib.rs`

**Create if missing:**
```rust
//! Glassware Orchestrator - High-performance package scanning
//!
//! This crate provides async orchestration for scanning npm packages
//! with caching, parallel execution, and multiple output formats.

pub mod cli;
pub mod orchestrator;
pub mod downloader;
pub mod scanner;
pub mod cacher;
pub mod error;
pub mod progress;
pub mod checkpoint;

pub mod formatters {
    pub mod json;
    pub mod sarif;
}

// Re-export commonly used types
pub use error::{OrchestratorError, OrchestratorResult};
pub use orchestrator::Orchestrator;
pub use cacher::CacheManager;
```

---

### 2.2 Verify File Structure

**Command:**
```bash
cd glassware-orchestrator
find src -type f -name "*.rs" | sort
```

**Expected Structure:**
```
src/
├── main.rs
├── lib.rs
├── cli.rs
├── orchestrator.rs
├── downloader.rs
├── scanner.rs
├── cacher.rs
├── error.rs
├── progress.rs
├── checkpoint.rs
└── formatters/
    ├── mod.rs
    ├── json.rs
    └── sarif.rs
```

**Fix Missing Files:**
```bash
cd glassware-orchestrator/src
touch cli.rs orchestrator.rs downloader.rs scanner.rs cacher.rs error.rs progress.rs checkpoint.rs
mkdir -p formatters
touch formatters/mod.rs formatters/json.rs formatters/sarif.rs
```

---

## Phase 3: tracing_subscriber API Fixes (2 hours)

### 3.1 Fix tracing_subscriber Initialization

**File:** `glassware-orchestrator/src/main.rs` or `glassware-orchestrator/src/lib.rs`

**Current Issue (from error doc):**
```
error: no method named `with` found for struct `Registry`
error: no method named `with_line_numbers` found for struct `Layer`
```

**Broken Code Pattern:**
```rust
// ❌ DON'T USE - API changed in tracing-subscriber 0.3.x
use tracing_subscriber::prelude::*;

let subscriber = Registry::default()
    .with(fmt::layer())
    .with_line_numbers(true)  // This method doesn't exist
    .init();
```

**Fixed Code:**
```rust
// ✅ CORRECT - tracing-subscriber 0.3.x API
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

pub fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,glassware_orchestrator=debug"));

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_level(true)
        .with_line_number(true)  // Method on fmt::layer(), not Registry
        .with_file(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();

    Ok(())
}
```

---

### 3.2 Fix All tracing Macro Calls

**Files to Check:** All `.rs` files using `info!`, `error!`, `debug!`, `warn!`, `trace!`

**Common Issue:** Format string syntax changed.

**Broken Pattern:**
```rust
// ❌ Old syntax may cause issues
info!("Package: {}", package_name);
error!("Error: " + &error_message);  // String concat doesn't work
```

**Fixed Pattern:**
```rust
// ✅ Correct syntax
info!(package = %package_name, "Scanning package");
error!(error = %error_message, "Scan failed");
// Or simple:
info!("Package: {}", package_name);
error!("Error: {}", error_message);
```

---

### 3.3 Update tracing Imports

**Add to each file using tracing:**
```rust
use tracing::{debug, error, info, trace, warn, Instrument};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
```

---

## Phase 4: Error Type Fixes (3 hours)

### 4.1 Fix error.rs ErrorContext

**File:** `glassware-orchestrator/src/error.rs`

**Current Issue:** ErrorContext methods and OrchestratorError variant constructors mismatched.

**Complete Fixed Implementation:**
```rust
use std::fmt;
use thiserror::Error;

/// Context information for errors
#[derive(Debug, Clone, Default)]
pub struct ErrorContext {
    pub operation: Option<String>,
    pub package: Option<String>,
    pub path: Option<String>,
    pub retry_count: Option<u32>,
    pub details: Option<String>,
}

impl ErrorContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }

    pub fn with_package(mut self, package: impl Into<String>) -> Self {
        self.package = Some(package.into());
        self
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn with_retry_count(mut self, count: u32) -> Self {
        self.retry_count = Some(count);
        self
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if let Some(op) = &self.operation {
            parts.push(format!("operation: {}", op));
        }
        if let Some(pkg) = &self.package {
            parts.push(format!("package: {}", pkg));
        }
        if let Some(path) = &self.path {
            parts.push(format!("path: {}", path));
        }
        if let Some(retry) = self.retry_count {
            parts.push(format!("retry: {}", retry));
        }
        if let Some(details) = &self.details {
            parts.push(format!("details: {}", details));
        }
        write!(f, "{}", parts.join(", "))
    }
}

/// Error category for classification and retry logic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    Network,
    Authentication,
    RateLimit,
    Timeout,
    NotFound,
    Internal,
    Cancelled,
    Configuration,
    Scanner,
    Cache,
    Io,
}

impl ErrorCategory {
    pub fn is_retryable(self) -> bool {
        matches!(
            self,
            ErrorCategory::Network
                | ErrorCategory::RateLimit
                | ErrorCategory::Timeout
                | ErrorCategory::Internal
        )
    }
}

/// Main orchestrator error type
#[derive(Error, Debug)]
pub enum OrchestratorError {
    #[error("Network error: {message}")]
    Network {
        message: String,
        context: ErrorContext,
    },

    #[error("Authentication failed: {message}")]
    Authentication {
        message: String,
        context: ErrorContext,
    },

    #[error("Rate limit exceeded. Retry after {retry_after} seconds")]
    RateLimitExceeded {
        retry_after: u64,
        context: ErrorContext,
    },

    #[error("Operation timed out: {message}")]
    Timeout {
        message: String,
        context: ErrorContext,
    },

    #[error("Resource not found: {resource}")]
    NotFound {
        resource: String,
        context: ErrorContext,
    },

    #[error("Internal error: {message}")]
    Internal {
        message: String,
        context: ErrorContext,
    },

    #[error("Operation cancelled")]
    Cancelled {
        context: ErrorContext,
    },

    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
        context: ErrorContext,
    },

    #[error("Scanner error: {message}")]
    Scanner {
        message: String,
        context: ErrorContext,
    },

    #[error("Cache error: {message}")]
    Cache {
        message: String,
        context: ErrorContext,
    },

    #[error("IO error: {message}")]
    Io {
        message: String,
        context: ErrorContext,
    },

    #[error("Max retries exceeded for operation: {operation}")]
    MaxRetriesExceeded {
        operation: String,
        context: ErrorContext,
    },

    #[error("Download failed for {package}: {message}")]
    DownloadFailed {
        package: String,
        message: String,
        context: ErrorContext,
    },

    #[error("Scan failed for {package}: {message}")]
    ScanFailed {
        package: String,
        message: String,
        context: ErrorContext,
    },

    #[error("GitHub API error: {0}")]
    GitHub(String, #[source] Option<reqwest::Error>),

    #[error("NPM API error: {0}")]
    Npm(String, #[source] Option<reqwest::Error>),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("UTF-8 encoding error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Invalid path: {path} - {reason}")]
    InvalidPath {
        path: String,
        reason: String,
    },
}

impl OrchestratorError {
    pub fn category(&self) -> ErrorCategory {
        match self {
            OrchestratorError::Network { .. } => ErrorCategory::Network,
            OrchestratorError::Authentication { .. } => ErrorCategory::Authentication,
            OrchestratorError::RateLimitExceeded { .. } => ErrorCategory::RateLimit,
            OrchestratorError::Timeout { .. } => ErrorCategory::Timeout,
            OrchestratorError::NotFound { .. } => ErrorCategory::NotFound,
            OrchestratorError::Internal { .. } => ErrorCategory::Internal,
            OrchestratorError::Cancelled { .. } => ErrorCategory::Cancelled,
            OrchestratorError::Configuration { .. } => ErrorCategory::Configuration,
            OrchestratorError::Scanner { .. } => ErrorCategory::Scanner,
            OrchestratorError::Cache { .. } => ErrorCategory::Cache,
            OrchestratorError::Io { .. } => ErrorCategory::Io,
            OrchestratorError::MaxRetriesExceeded { .. } => ErrorCategory::Internal,
            OrchestratorError::DownloadFailed { .. } => ErrorCategory::Network,
            OrchestratorError::ScanFailed { .. } => ErrorCategory::Scanner,
            OrchestratorError::GitHub(..) => ErrorCategory::Network,
            OrchestratorError::Npm(..) => ErrorCategory::Network,
            OrchestratorError::Database(..) => ErrorCategory::Internal,
            OrchestratorError::Json(..) => ErrorCategory::Internal,
            OrchestratorError::Utf8(..) => ErrorCategory::Internal,
            OrchestratorError::InvalidPath { .. } => ErrorCategory::Configuration,
        }
    }

    pub fn is_retryable(&self) -> bool {
        self.category().is_retryable()
    }

    /// Helper for creating errors with context - 1 argument version
    pub fn network(message: impl Into<String>) -> Self {
        OrchestratorError::Network {
            message: message.into(),
            context: ErrorContext::new(),
        }
    }

    /// Helper for creating errors with context - 2 argument version
    pub fn network_with_context(message: impl Into<String>, context: ErrorContext) -> Self {
        OrchestratorError::Network {
            message: message.into(),
            context,
        }
    }

    pub fn io(message: impl Into<String>) -> Self {
        OrchestratorError::Io {
            message: message.into(),
            context: ErrorContext::new(),
        }
    }

    pub fn io_with_context(message: impl Into<String>, context: ErrorContext) -> Self {
        OrchestratorError::Io {
            message: message.into(),
            context,
        }
    }

    pub fn cache(message: impl Into<String>) -> Self {
        OrchestratorError::Cache {
            message: message.into(),
            context: ErrorContext::new(),
        }
    }

    pub fn scanner(message: impl Into<String>) -> Self {
        OrchestratorError::Scanner {
            message: message.into(),
            context: ErrorContext::new(),
        }
    }

    pub fn download_failed(package: impl Into<String>, message: impl Into<String>) -> Self {
        OrchestratorError::DownloadFailed {
            package: package.into(),
            message: message.into(),
            context: ErrorContext::new(),
        }
    }

    pub fn scan_failed(package: impl Into<String>, message: impl Into<String>) -> Self {
        OrchestratorError::ScanFailed {
            package: package.into(),
            message: message.into(),
            context: ErrorContext::new(),
        }
    }

    pub fn rate_limit(retry_after: u64) -> Self {
        OrchestratorError::RateLimitExceeded {
            retry_after,
            context: ErrorContext::new(),
        }
    }

    pub fn not_found(resource: impl Into<String>) -> Self {
        OrchestratorError::NotFound {
            resource: resource.into(),
            context: ErrorContext::new(),
        }
    }

    pub fn timeout(message: impl Into<String>) -> Self {
        OrchestratorError::Timeout {
            message: message.into(),
            context: ErrorContext::new(),
        }
    }

    pub fn github(message: impl Into<String>) -> Self {
        OrchestratorError::GitHub(message.into(), None)
    }

    pub fn npm(message: impl Into<String>) -> Self {
        OrchestratorError::Npm(message.into(), None)
    }
}

/// Result type alias for orchestrator operations
pub type OrchestratorResult<T> = Result<T, OrchestratorError>;
```

---

### 4.2 Update All Error Constructor Call Sites

**Search Pattern:** Find all `OrchestratorError::` usages

**Command:**
```bash
cd glassware-orchestrator
grep -rn "OrchestratorError::" src/ --include="*.rs"
```

**Fix Pattern:**

| Old Code | New Code |
|----------|----------|
| `OrchestratorError::Network("msg".to_string())` | `OrchestratorError::network("msg")` |
| `OrchestratorError::Io { message: "msg".to_string(), context: ErrorContext::new() }` | `OrchestratorError::io("msg")` |
| `OrchestratorError::Cache { message: "msg".to_string(), .. }` | `OrchestratorError::cache("msg")` |
| `OrchestratorError::DownloadFailed { package: pkg, message: msg, context: ErrorContext::new() }` | `OrchestratorError::download_failed(&pkg, &msg)` |

---

## Phase 5: Async/Tokio Pattern Fixes (2 hours)

### 5.1 Fix Async Function Signatures

**Common Issue:** Missing `async` keyword or incorrect return types.

**Broken Pattern:**
```rust
// ❌ Missing async
pub fn scan_package(&self, package: &str) -> OrchestratorResult<ScanResult> {
    // async code here
}

// ❌ Wrong return type
pub async fn scan_package(&self, package: &str) -> Result<ScanResult> {
    // missing error type
}
```

**Fixed Pattern:**
```rust
// ✅ Correct
pub async fn scan_package(&self, package: &str) -> OrchestratorResult<ScanResult> {
    // async code here
}
```

---

### 5.2 Fix Await Usage

**Common Issue:** Missing `.await` on async calls.

**Broken Pattern:**
```rust
// ❌ Missing await
let result = self.download_package(package);
let cached = self.cache.get(package);
```

**Fixed Pattern:**
```rust
// ✅ Correct
let result = self.download_package(package).await?;
let cached = self.cache.get(package).await?;
```

---

### 5.3 Fix Tokio Spawn Patterns

**File:** `glassware-orchestrator/src/orchestrator.rs`

**Broken Pattern:**
```rust
// ❌ No error handling on spawn
tokio::spawn(async move {
    scan_package(pkg).await
});
```

**Fixed Pattern:**
```rust
// ✅ With error handling
let handle = tokio::spawn(async move {
    scan_package(pkg).await
});

// Later, await the handle
match handle.await {
    Ok(Ok(result)) => Ok(result),
    Ok(Err(e)) => Err(e),
    Err(join_err) => Err(OrchestratorError::internal(format!("Task panicked: {}", join_err))),
}
```

---

### 5.4 Fix Semaphore Usage

**File:** `glassware-orchestrator/src/orchestrator.rs` or `scanner.rs`

**Broken Pattern:**
```rust
// ❌ tokio-semaphore 0.1.x API changed
let permit = semaphore.acquire().await;
```

**Fixed Pattern:**
```rust
// ✅ Correct for tokio-semaphore 0.1.x
use tokio_semaphore::Semaphore;

let semaphore = Semaphore::new(concurrency_limit);
let _permit = semaphore.acquire().await;
// Or with tokio::sync::Semaphore:
use tokio::sync::Semaphore;
let semaphore = Arc::new(Semaphore::new(concurrency_limit));
let permit = semaphore.acquire().await?;
```

**Recommendation:** Use `tokio::sync::Semaphore` instead of external crate:

```toml
# In Cargo.toml, remove tokio-semaphore
# tokio already includes Semaphore with "sync" feature (enabled by "full")
```

```rust
use std::sync::Arc;
use tokio::sync::{Semaphore, OwnedSemaphorePermit};

pub struct Orchestrator {
    semaphore: Arc<Semaphore>,
    // ...
}

impl Orchestrator {
    pub fn new(concurrency: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(concurrency)),
            // ...
        }
    }

    async fn acquire_permit(&self) -> OrchestratorResult<OwnedSemaphorePermit> {
        self.semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| OrchestratorError::internal(format!("Semaphore error: {}", e)))
    }
}
```

---

## Phase 6: SQLx/Database Fixes (1 hour)

### 6.1 Fix SQLx Pool Initialization

**File:** `glassware-orchestrator/src/cacher.rs`

**Broken Pattern:**
```rust
// ❌ sqlx 0.7.x API
let pool = SqlitePool::connect(&database_url).await?;
```

**Fixed Pattern:**
```rust
// ✅ sqlx 0.7.x correct API
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .min_connections(1)
    .acquire_timeout(std::time::Duration::from_secs(30))
    .connect(&database_url)
    .await?;
```

---

### 6.2 Fix SQLx Query Macros

**Common Issue:** `query!` macro requires offline mode or running database.

**Option A: Use `query_as!` with explicit types:**
```rust
use sqlx::query_as;

let result: Option<CacheEntry> = query_as!(
    CacheEntry,
    "SELECT id, package, version, findings, scanned_at FROM cache WHERE package = ?",
    package_name
)
.fetch_optional(&self.pool)
.await?;
```

**Option B: Enable offline mode:**
```bash
# Create .env with DATABASE_URL
echo "DATABASE_URL=sqlite://./glassware-cache.db" > .env

# Generate offline metadata
cargo sqlx prepare --workspace
```

---

### 6.3 Fix Database Schema

**File:** `glassware-orchestrator/src/cacher.rs` or migration file

**Schema:**
```sql
-- Enable WAL mode for better concurrency
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = -64000;

CREATE TABLE IF NOT EXISTS cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    package TEXT NOT NULL UNIQUE,
    version TEXT NOT NULL,
    findings TEXT NOT NULL,
    scanned_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_package ON cache(package);
CREATE INDEX IF NOT EXISTS idx_expires ON cache(expires_at);
```

---

## Phase 7: Build & Verify (1 hour)

### 7.1 Clean Build

```bash
# Clean all build artifacts
cargo clean

# Update dependencies
cargo update

# Check (faster than build, catches most errors)
cargo check --package glassware-orchestrator --all-targets 2>&1 | tee check-errors.log

# Build
cargo build --package glassware-orchestrator --release 2>&1 | tee build.log

# Run tests
cargo test --package glassware-orchestrator 2>&1 | tee test.log
```

---

### 7.2 Clippy Linting

```bash
cargo clippy --package glassware-orchestrator --all-targets -- -D warnings 2>&1 | tee clippy.log
```

---

### 7.3 Format Check

```bash
cargo fmt --package glassware-orchestrator -- --check
```

---

## Error Resolution Checklist

Use this checklist to track progress:

### Dependencies
- [ ] Root `Cargo.toml` updated with workspace members
- [ ] `glassware-orchestrator/Cargo.toml` uses workspace dependencies
- [ ] Rust version verified (1.70+)
- [ ] `cargo update` completed successfully

### Module Structure
- [ ] `src/lib.rs` created with module declarations
- [ ] All source files exist in correct locations
- [ ] `formatters/` directory with mod.rs, json.rs, sarif.rs

### tracing_subscriber
- [ ] `init_logging()` function uses correct 0.3.x API
- [ ] All `info!`, `error!`, `debug!` macros use correct syntax
- [ ] All files have correct tracing imports

### Error Types
- [ ] `error.rs` has complete `ErrorContext` implementation
- [ ] `error.rs` has complete `OrchestratorError` enum
- [ ] Helper methods added (network, io, cache, etc.)
- [ ] All call sites updated to use helper methods
- [ ] `OrchestratorResult` type alias defined

### Async/Tokio
- [ ] All async functions have `async` keyword
- [ ] All async calls have `.await`
- [ ] Return types use `OrchestratorResult<T>`
- [ ] Semaphore uses `tokio::sync::Semaphore`
- [ ] Spawn handles are properly awaited

### Database
- [ ] SQLx pool uses `SqlitePoolOptions`
- [ ] Query macros use `query_as!` or offline mode
- [ ] Database schema includes indexes
- [ ] WAL mode enabled

### Build Verification
- [ ] `cargo check` passes with no errors
- [ ] `cargo build` completes successfully
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes
- [ ] `cargo fmt` shows no changes needed

---

## Quick Reference: Common Error → Fix

| Error Message | Fix |
|--------------|-----|
| `no method named 'with' found for struct 'Registry'` | Use `tracing_subscriber::registry().with(layer)` not `Registry::default().with()` |
| `no method named 'with_line_numbers'` | Move to `fmt::layer().with_line_number(true)` |
| `expected struct 'ErrorContext', found '()'` | Add `.context: ErrorContext::new()` to error variant |
| `no method named 'acquire' found for struct 'Semaphore'` | Use `acquire_owned().await` and import from `tokio::sync` |
| `the trait 'From<sqlx::Error>' is not implemented` | Add `#[from]` attribute or manual conversion |
| `failed to resolve: use of undeclared crate or module` | Add module declaration in `lib.rs` |
| `expected 'impl Future', found '()'` | Add `.await` to async function call |
| `cannot find value 'env' in this scope` | Add `use tracing_subscriber::EnvFilter` |

---

## Escalation Path

If issues persist after following this guide:

1. **Capture Full Error Output:**
   ```bash
   cargo check --package glassware-orchestrator --all-targets 2>&1 | tee full-errors.log
   ```

2. **Share These Files:**
   - `full-errors.log`
   - `glassware-orchestrator/src/error.rs`
   - `glassware-orchestrator/src/main.rs`
   - `glassware-orchestrator/Cargo.toml`

3. **Expert Contacts:**
   - Async Rust expert: Tokio runtime patterns
   - SQLx expert: Database connection pooling
   - tracing expert: Subscriber configuration

---

## Post-Fix Validation

Once compilation succeeds:

```bash
# Run the orchestrator with help
cargo run --package glassware-orchestrator -- --help

# Test with a single package
cargo run --package glassware-orchestrator -- scan --package lodash --concurrency 5

# Verify cache database created
ls -la glassware-cache.db

# Check logs output
tail -f orchestrator.log 2>/dev/null || echo "No log file yet"
```

---

**Document End**

**Next Steps After Fix:**
1. Run full test suite
2. Compare performance vs Python harness
3. Document any remaining limitations
4. Prepare v0.8.0 release notes

```