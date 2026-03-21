
---

# Cache Database Fix — SQLite Error 14

**Root Cause:** sqlx requires a **connection URL** with `sqlite://` prefix, but the code passes just the file path.

**Evidence from your logs:**
```
Database error: error returned from database: (code: 14) unable to open database file
```

This happens even with `/tmp/test-glassware-cache.db` (always writable), confirming it's a **connection string format issue**, not a permissions issue.

---

## Fix: cacher.rs (10 minutes)

**File:** `glassware-orchestrator/src/cacher.rs`

**Line:** ~155-165 (in `with_path_and_ttl` method)

### Replace This Section:

```rust
// CURRENT (broken - missing sqlite:// prefix)
let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .min_connections(1)
    .acquire_timeout(StdDuration::from_secs(30))
    .connect(db_path.to_str().ok_or_else(|| {
        OrchestratorError::cache_error("Invalid database path".to_string())
    })?)
    .await
    .map_err(|e| OrchestratorError::database(e))?;
```

### With This (fixed):

```rust
// FIXED - Add sqlite:// prefix for sqlx connection URL
let db_url = format!("sqlite://{}", db_path.to_str().ok_or_else(|| {
    OrchestratorError::cache_error("Invalid database path".to_string())
})?);

let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .min_connections(1)
    .acquire_timeout(StdDuration::from_secs(30))
    .connect(&db_url)
    .await
    .map_err(|e| OrchestratorError::database(format!(
        "Failed to connect to cache database '{}': {}",
        db_url, e
    )))?;
```

---

## Also Add: SQLite PRAGMA Optimizations

**File:** `glassware-orchestrator/src/cacher.rs`

**Add to `init_db` method** (after line 185, after `CREATE INDEX` statements):

```rust
// Enable WAL mode for better concurrency
sqlx::query("PRAGMA journal_mode = WAL")
    .execute(&*self.pool)
    .await
    .map_err(|e| OrchestratorError::database(format!("Failed to set WAL mode: {}", e)))?;

// Set synchronous to NORMAL for better performance (safe with WAL)
sqlx::query("PRAGMA synchronous = NORMAL")
    .execute(&*self.pool)
    .await
    .map_err(|e| OrchestratorError::database(format!("Failed to set synchronous: {}", e)))?;

// Set cache size (64MB)
sqlx::query("PRAGMA cache_size = -64000")
    .execute(&*self.pool)
    .await
    .map_err(|e| OrchestratorError::database(format!("Failed to set cache_size: {}", e)))?;

info!("Cache database initialized with WAL mode");
```

---

## Verify Cargo.toml Has SQLite Features

**File:** `glassware-orchestrator/Cargo.toml`

**Ensure sqlx dependency has these features:**

```toml
[dependencies]
# ... other deps ...

# Database - MUST have sqlite and runtime-tokio features
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-rustls", "chrono"] }

# ... other deps ...
```

**Missing features will cause silent failures.**

---

## Verification Commands

```bash
cd ~/samgrowls/glassworks/glassware-orchestrator

# Rebuild
cargo build --bin glassware-orchestrator 2>&1 | tee build.log

# Test cache-stats (should work now)
./target/debug/glassware-orchestrator --cache-db /tmp/test-glassware-cache.db cache-stats

# Expected output:
# Cache Statistics:
#   Total entries: 0
#   Expired entries: 0
#   ...

# Test scan with cache
./target/debug/glassware-orchestrator --cache-db /tmp/test-glassware-cache.db scan-npm is-odd --quiet

# Second scan should be faster (cached)
time ./target/debug/glassware-orchestrator --cache-db /tmp/test-glassware-cache.db scan-npm is-odd --quiet

# Check cache stats after scan
./target/debug/glassware-orchestrator --cache-db /tmp/test-glassware-cache.db cache-stats

# Expected: 1 npm entry
```

---

## Expected Results

| Test | Before | After |
|------|--------|-------|
| `cache-stats` | "unable to open database file" | Shows statistics |
| First scan | Cache warning, no caching | Cache initialized, result stored |
| Second scan | Same speed (no cache) | 10-100x faster (cached) |
| Cache DB file | Not created | Created at specified path |

---

## Why This Happened

**sqlx connection string format:**

| Database | Connection URL Format |
|----------|----------------------|
| SQLite | `sqlite:///path/to/file.db` or `sqlite://file.db` |
| PostgreSQL | `postgres://user:pass@host:port/db` |
| MySQL | `mysql://user:pass@host:port/db` |

The code was passing `.glassware-orchestrator-cache.db` (just a path), but sqlx needs `sqlite://.glassware-orchestrator-cache.db` (URL with scheme).

---

## Apply This Fix Now

This is a **single, surgical change** that will fix the cache issue completely. Once applied:

1. ✅ Cache will initialize successfully
2. ✅ Scan results will be cached
3. ✅ Re-scans will be instant
4. ✅ 1000+ package scans will be feasible

**Ready to apply?** This should take 10 minutes to fix and verify.