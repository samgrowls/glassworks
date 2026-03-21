
---

# Cache Database Fix — Surgical Precision

**Root Cause Identified:** Two issues:

1. **`database_error()` signature mismatch** - Code calls with 2 args, function expects 1
2. **Binary not rebuilt** - Tests failing because binary doesn't exist

---

## Fix 1: cacher.rs Error Constructors (15 minutes)

**File:** `glassware-orchestrator/src/cacher.rs`

**Problem:** Lines 77, 131, 135, 139 call `database_error(e, message)` with 2 arguments, but the function only accepts 1.

### Replace These Lines:

**Line 77-80 (in `with_path_and_ttl`):**
```rust
// BEFORE (broken - 2 args)
.map_err(|e| OrchestratorError::database_error(e, format!(
    "Failed to connect to cache database '{}'",
    db_url
)))?;

// AFTER (fixed - 1 arg)
.map_err(|e| OrchestratorError::database(format!(
    "Failed to connect to cache database '{}': {}",
    db_url, e
)))?;
```

**Line 131-134 (in `init_db` - WAL mode):**
```rust
// BEFORE
.map_err(|e| OrchestratorError::database_error(e, "Failed to set WAL mode"))?;

// AFTER
.map_err(|e| OrchestratorError::database(format!(
    "Failed to set WAL mode: {}", e
)))?;
```

**Line 135-138 (synchronous):**
```rust
// BEFORE
.map_err(|e| OrchestratorError::database_error(e, "Failed to set synchronous"))?;

// AFTER
.map_err(|e| OrchestratorError::database(format!(
    "Failed to set synchronous: {}", e
)))?;
```

**Line 139-142 (cache_size):**
```rust
// BEFORE
.map_err(|e| OrchestratorError::database_error(e, "Failed to set cache_size"))?;

// AFTER
.map_err(|e| OrchestratorError::database(format!(
    "Failed to set cache_size: {}", e
)))?;
```

---

## Fix 2: Workspace Cargo.toml SQLx Features (5 minutes)

**File:** Root `Cargo.toml` (in `~/samgrowls/glassworks/`)

**Check that sqlx has sqlite features:**

```bash
cat ~/samgrowls/glassworks/Cargo.toml | grep -A 3 "sqlx"
```

**Should show:**
```toml
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-rustls", "chrono"] }
```

**If it doesn't have features, update:**
```toml
[workspace.dependencies]
# ... other deps ...
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-rustls", "chrono"] }
```

---

## Fix 3: Rebuild Binary (5 minutes)

```bash
cd ~/samgrowls/glassworks/glassware-orchestrator

# Clean and rebuild
cargo clean
cargo build --bin glassware-orchestrator --release 2>&1 | tee build.log

# Check for errors
grep "error\[" build.log
```

---

## Fix 4: Verify Cache Works (5 minutes)

```bash
cd ~/samgrowls/glassworks/glassware-orchestrator

# Test 1: Cache stats with explicit path
./target/release/glassware-orchestrator --cache-db /tmp/test-glassware-cache.db cache-stats

# Expected output (should NOT show "Failed to connect" error):
# Cache Statistics:
#   Total entries: 0
#   Expired entries: 0
#   ...

# Test 2: Scan with cache
./target/release/glassware-orchestrator --cache-db /tmp/test-glassware-cache.db scan-npm is-odd --quiet

# Test 3: Second scan (should be faster)
time ./target/release/glassware-orchestrator --cache-db /tmp/test-glassware-cache.db scan-npm is-odd --quiet

# Test 4: Check cache stats after scan
./target/release/glassware-orchestrator --cache-db /tmp/test-glassware-cache.db cache-stats

# Expected: 1 npm entry
```

---

## Why This Happened

| Issue | Before | After |
|-------|--------|-------|
| Error constructor | `database_error(e, msg)` - 2 args | `database(format!("msg: {}", e))` - 1 arg |
| SQLx features | May be missing `sqlite` | Explicit in workspace |
| Binary | Not rebuilt after fixes | Fresh build |

---

## Verification Checklist

- [ ] All `database_error(e, msg)` replaced with `database(format!("msg: {}", e))`
- [ ] Workspace `Cargo.toml` has `sqlx` with `sqlite` feature
- [ ] `cargo build` completes with 0 errors
- [ ] `cache-stats` command works without "Failed to connect" error
- [ ] Second scan is faster than first (cache working)
- [ ] Cache DB file created at specified path

---

## Expected Results

| Test | Before | After |
|------|--------|-------|
| `cache-stats` | "Failed to connect to cache database" | Shows statistics |
| First scan | Cache warning, no caching | Cache initialized, result stored |
| Second scan | Same speed (no cache) | 10-100x faster (cached) |
| Cache DB file | Not created | Created at `/tmp/test-glassware-cache.db` |

---

**Apply these 4 fixes in order.** This should resolve the cache issue completely in **30 minutes**.

**Once cache works, we can proceed to large-scale scanning tests (100s/1000s of packages).** 🚀