# Orchestrator Compilation Fix Progress

**Date:** 2026-03-20 17:30 UTC  
**Status:** 🟡 IN PROGRESS  
**Errors Remaining:** ~120 (down from 144)  

---

## Completed Fixes ✅

### Phase 1: Dependencies ✅
- ✅ Root workspace updated with glassware-orchestrator member
- ✅ Workspace dependencies defined (tokio, clap, reqwest, sqlx, etc.)
- ✅ Orchestrator Cargo.toml updated to use workspace dependencies
- ✅ Added missing dependencies: parking_lot, urlencoding, rand, tempfile, tokio-test
- ✅ Flattened orchestrator structure (removed nested workspace)

### Phase 2: Module Structure ✅
- ✅ All source files moved to `glassware-orchestrator/src/`
- ✅ main.rs copied from orchestrator-cli
- ✅ lib.rs exists with module declarations
- ✅ formatters/ directory with mod.rs, json.rs, sarif.rs

---

## Remaining Errors (~120)

### By Category

| Category | Count | Priority | Fix Approach |
|----------|-------|----------|--------------|
| **Error helper signatures** | 67 | P0 | Add 1-arg helper methods |
| **GitHub struct variant** | 9 | P0 | Fix GitHub() constructor |
| **DownloadFailed/ScanFailed source field** | 6 | P0 | Remove source field from variants |
| **NotFound/Cache/InvalidPackageName** | 11 | P0 | Fix helper methods |
| **tracing_subscriber API** | 6 | P0 | Update to 0.3.x API |
| **scan_content method** | 2 | P1 | Add method to Scanner |
| **Io/InvalidPath variants** | 4 | P1 | Fix constructors |
| **tempfile (dev)** | 4 | P2 | Move to dev-dependencies properly |

---

## Fix Plan

### Step 1: Fix Error Helper Methods (67 errors)

**Problem:** Helper methods take 2 args (source + message) but called with 1 arg.

**Solution:** Add 1-argument versions of all helper methods in error.rs:

```rust
// Add these methods to OrchestratorError impl:

pub fn io_simple(message: impl Into<String>) -> Self {
    OrchestratorError::Io {
        message: message.into(),
        context: ErrorContext::new(),
    }
}

pub fn cache_simple(message: impl Into<String>) -> Self {
    OrchestratorError::Cache {
        message: message.into(),
        context: ErrorContext::new(),
    }
}

pub fn not_found_simple(resource: impl Into<String>) -> Self {
    OrchestratorError::NotFound {
        resource: resource.into(),
        context: ErrorContext::new(),
    }
}

// ... etc for all variants
```

**Files to update:**
- `glassware-orchestrator/src/error.rs`

**Estimated time:** 1-2 hours

---

### Step 2: Fix GitHub Error (9 errors)

**Problem:** GitHub variant has wrong constructor.

**Current:**
```rust
OrchestratorError::GitHub(message, source)
```

**Should be:**
```rust
OrchestratorError::github(message)  // Use helper method
```

**Files to update:**
- `glassware-orchestrator/src/github.rs`
- `glassware-orchestrator/src/downloader.rs`

**Estimated time:** 30 min

---

### Step 3: Fix DownloadFailed/ScanFailed (6 errors)

**Problem:** Variants have `source` field that doesn't exist.

**Current:**
```rust
OrchestratorError::DownloadFailed { package, source: e, message: msg }
```

**Should be:**
```rust
OrchestratorError::download_failed(&package, &msg)
```

**Files to update:**
- `glassware-orchestrator/src/downloader.rs`
- `glassware-orchestrator/src/scanner.rs`

**Estimated time:** 30 min

---

### Step 4: Fix tracing_subscriber API (6 errors)

**Problem:** API changed in 0.3.x.

**Current (broken):**
```rust
Registry::default()
    .with(fmt::layer())
    .with_line_numbers(true)  // Doesn't exist
```

**Fixed:**
```rust
tracing_subscriber::registry()
    .with(fmt::layer().with_line_number(true))  // Method on layer, not Registry
    .init()
```

**Files to update:**
- `glassware-orchestrator/src/tracing.rs`
- `glassware-orchestrator/src/main.rs`

**Estimated time:** 1 hour

---

### Step 5: Add scan_content Method (2 errors)

**Problem:** Scanner doesn't have scan_content method.

**Solution:** Add method to Scanner:

```rust
impl Scanner {
    pub async fn scan_content(&self, content: &str, file_path: &Path) -> OrchestratorResult<ScanResult> {
        // Use glassware-core to scan content
        let engine = ScanEngine::default_detectors();
        let findings = engine.scan(file_path, content);
        Ok(ScanResult { findings, .. })
    }
}
```

**Files to update:**
- `glassware-orchestrator/src/scanner.rs`

**Estimated time:** 30 min

---

## Total Estimated Time

**Total:** 3-5 hours of focused work

**Breakdown:**
- Error helpers: 1-2h
- GitHub error: 30min
- DownloadFailed/ScanFailed: 30min
- tracing_subscriber: 1h
- scan_content: 30min

---

## Commands to Verify Progress

```bash
cd /home/property.sightlines/samgrowls/glassworks

# Check error count
cargo check -p glassware-orchestrator 2>&1 | grep "error\[" | wc -l

# Check specific error types
cargo check -p glassware-orchestrator 2>&1 | grep "error\[E0061\]" | wc -l  # Helper signatures
cargo check -p glassware-orchestrator 2>&1 | grep "error\[E0533\]" | wc -l  # Struct variants
cargo check -p glassware-orchestrator 2>&1 | grep "error\[E0599\]" | wc -l  # Missing methods
```

---

## Current Status

**Errors:** ~120 (was 144, then 67+, now ~120 after dependency fixes)  
**Progress:** 20% complete  
**Next:** Fix error helper methods (67 errors)

---

**Timestamp:** 2026-03-20 17:30 UTC  
**Next Step:** Add 1-argument error helper methods to error.rs
