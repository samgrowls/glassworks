# Rust Orchestrator — Specification

**Version:** 0.1.0 (Draft)  
**Date:** 2026-03-20  
**Status:** 📝 SPECIFICATION  
**Effort:** 24 hours  
**Priority:** P3  

---

## Executive Summary

**Problem:** Python harness has serialization overhead, hard to scale, duplicate logic with Rust core.

**Solution:** Move orchestration to Rust with Tokio-based async execution.

**Impact:**
- Eliminate JSON serialization overhead
- Enable true parallel scanning (not just process-per-package)
- Unified codebase (no Python/Rust split)
- Better error handling and retry logic
- Native support for streaming results

---

## Background

### Current Architecture

**Python Harness (`harness/optimized_scanner.py`):**
```python
# Current flow
for package in packages:
    # Download package
    subprocess.run(["npm", "pack", package])
    
    # Extract
    subprocess.run(["tar", "-xzf", tarball])
    
    # Scan (spawn Rust binary)
    result = subprocess.run(
        ["./glassware-scanner", "--format", "json", "package/"],
        capture_output=True
    )
    
    # Parse JSON output
    findings = json.loads(result.stdout)
```

**Issues:**
1. **Serialization Overhead:** JSON in/out for every package
2. **Process Overhead:** Spawn Rust binary per package
3. **Duplicate Logic:** Caching in both Python and Rust
4. **Error Handling:** Complex subprocess error handling
5. **Scaling:** Hard to scale beyond process-per-package

### Target Architecture

**Rust Orchestrator:**
```rust
// Target flow
let orchestrator = Orchestrator::new(config);

// Async package scanning
let results = orchestrator
    .scan_packages(packages)
    .buffer_unordered(10)  // 10 concurrent scans
    .try_collect::<Vec<_>>()
    .await?;

// Direct access to findings (no JSON parsing)
for result in results {
    process_findings(&result.findings);
}
```

---

## Requirements

### Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| FR1 | Scan npm packages by name | P0 |
| FR2 | Scan GitHub repositories | P0 |
| FR3 | Support parallel scanning (configurable concurrency) | P0 |
| FR4 | Cache scan results (7-day TTL) | P0 |
| FR5 | Output results in JSON/SARIF format | P0 |
| FR6 | Support incremental scanning | P1 |
| FR7 | Support LLM analysis integration | P1 |
| FR8 | Support adversarial testing integration | P2 |

### Non-Functional Requirements

| ID | Requirement | Target |
|----|-------------|--------|
| NFR1 | Scan speed | >100 packages/min |
| NFR2 | Memory usage | <500MB peak |
| NFR3 | Error rate | <1% (network retries) |
| NFR4 | Cache hit rate | >50% on re-scans |
| NFR5 | Serialization overhead | <5% of total time |

---

## Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────────┐
│                      CLI Interface                           │
│  - Parse command-line arguments                              │
│  - Load configuration                                        │
│  - Start Tokio runtime                                       │
└─────────────────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                     Orchestrator                             │
│  - Manage package download queue                             │
│  - Coordinate parallel scanning                              │
│  - Handle retries and errors                                 │
│  - Aggregate results                                         │
└─────────────────────────────────────────────────────────────┘
                         │
         ┌───────────────┼───────────────┐
         ▼               ▼               ▼
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│  Downloader │  │   Scanner   │  │   Cacher    │
│  (npm/git)  │  │  (glassware │  │  (SQLite)   │
│             │  │   core)     │  │             │
└─────────────┘  └─────────────┘  └─────────────┘
```

### Component Specifications

#### 1. CLI Interface

**Purpose:** Parse arguments, load config, start runtime.

**Commands:**
```bash
# Scan npm packages
glassware-orchestrator scan-npm --packages "lodash,axios,moment" --output results.json

# Scan GitHub repos
glassware-orchestrator scan-github --query "mcp-server" --max-results 500

# Scan package list from file
glassware-orchestrator scan-file --input packages.txt --output results.json

# Resume interrupted scan
glassware-orchestrator resume --checkpoint checkpoint.json
```

**CLI Structure:**
```rust
#[derive(Parser, Debug)]
#[command(name = "glassware-orchestrator")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Scan npm packages
    ScanNpm(ScanNpmArgs),
    
    /// Scan GitHub repositories
    ScanGithub(ScanGithubArgs),
    
    /// Scan package list from file
    ScanFile(ScanFileArgs),
    
    /// Resume interrupted scan
    Resume(ResumeArgs),
}
```

---

#### 2. Orchestrator

**Purpose:** Coordinate parallel scanning, handle retries, aggregate results.

**API:**
```rust
pub struct Orchestrator {
    config: OrchestratorConfig,
    downloader: Downloader,
    scanner: Scanner,
    cacher: Cacher,
    semaphore: Semaphore,  // Limit concurrency
}

impl Orchestrator {
    pub fn new(config: OrchestratorConfig) -> Self;
    
    pub async fn scan_packages(
        &self,
        packages: Vec<String>,
    ) -> Result<Vec<ScanResult>>;
    
    pub async fn scan_with_progress(
        &self,
        packages: Vec<String>,
        progress: impl Fn(ProgressUpdate),
    ) -> Result<Vec<ScanResult>>;
}

pub struct ProgressUpdate {
    pub current: usize,
    pub total: usize,
    pub scanned: usize,
    pub flagged: usize,
    pub errors: usize,
    pub elapsed: Duration,
}
```

**Concurrency Model:**
```rust
// Use Tokio semaphore to limit concurrency
let semaphore = Arc::new(Semaphore::new(config.concurrency));

// Scan packages in parallel
let tasks: Vec<_> = packages
    .into_iter()
    .map(|pkg| {
        let permit = semaphore.clone().acquire_owned().await?;
        let result = orchestrator.scan_single_package(pkg).await;
        drop(permit);  // Release semaphore
        result
    })
    .collect();

let results = futures::future::join_all(tasks).await;
```

---

#### 3. Downloader

**Purpose:** Download npm packages and GitHub repositories.

**API:**
```rust
pub struct Downloader {
    npm_registry: String,
    github_api: String,
    temp_dir: PathBuf,
    client: reqwest::Client,
}

impl Downloader {
    pub async fn download_npm(&self, package: &str) -> Result<PathBuf>;
    pub async fn download_github(&self, repo: &str) -> Result<PathBuf>;
    pub async fn cleanup(&self, path: &Path) -> Result<()>;
}
```

**Retry Logic:**
```rust
use retry::{retry, delay::Fixed};

pub async fn download_npm_with_retry(
    &self,
    package: &str,
    max_retries: u32,
) -> Result<PathBuf> {
    retry(Fixed::from_millis(1000).take(max_retries as usize), || {
        self.download_npm(package)
    })
}
```

---

#### 4. Scanner

**Purpose:** Integrate with glassware-core for scanning.

**API:**
```rust
pub struct Scanner {
    engine: ScanEngine,
    config: ScanConfig,
}

impl Scanner {
    pub fn new(config: ScanConfig) -> Self;
    
    pub async fn scan_directory(&self, path: &Path) -> Result<ScanResult>;
    pub async fn scan_file(&self, path: &Path) -> Result<ScanResult>;
}
```

**Integration with glassware-core:**
```rust
// Direct integration (no subprocess)
use glassware_core::{ScanEngine, FileIR};

pub async fn scan_directory(&self, path: &Path) -> Result<ScanResult> {
    let mut findings = Vec::new();
    
    // Walk directory
    for entry in walkdir::WalkDir::new(path) {
        let entry = entry?;
        let content = tokio::fs::read_to_string(entry.path()).await?;
        let ir = FileIR::build(entry.path(), &content);
        
        // Scan with engine
        let file_findings = self.engine.scan_with_ir(&ir);
        findings.extend(file_findings);
    }
    
    Ok(ScanResult { findings, .. })
}
```

---

#### 5. Cacher

**Purpose:** Cache scan results with TTL.

**API:**
```rust
pub struct Cacher {
    db: SqlitePool,
    ttl_days: u32,
}

impl Cacher {
    pub async fn new(db_path: &str, ttl_days: u32) -> Result<Self>;
    
    pub async fn get(&self, package_hash: &str) -> Result<Option<ScanResult>>;
    pub async fn set(&self, package_hash: &str, result: &ScanResult) -> Result<()>;
    pub async fn cleanup_expired(&self) -> Result<usize>;
}
```

**Database Schema:**
```sql
CREATE TABLE IF NOT EXISTS scan_results (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    package_hash TEXT UNIQUE NOT NULL,
    package_name TEXT NOT NULL,
    package_version TEXT,
    findings_json TEXT NOT NULL,
    threat_score REAL,
    scanned_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME NOT NULL
);

CREATE INDEX idx_package_hash ON scan_results(package_hash);
CREATE INDEX idx_expires_at ON scan_results(expires_at);
```

---

## Implementation Plan

### Phase 1: Core Infrastructure (8h)

**Tasks:**
1. Create new crate `glassware-orchestrator`
2. Set up CLI with `clap`
3. Implement `Orchestrator` struct
4. Implement `Downloader` (npm only initially)
5. Implement `Scanner` integration with glassware-core
6. Implement `Cacher` with SQLite
7. Basic parallel scanning (no progress reporting)
8. Unit tests for each component

**Files to Create:**
- `glassware-orchestrator/Cargo.toml`
- `glassware-orchestrator/src/main.rs`
- `glassware-orchestrator/src/cli.rs`
- `glassware-orchestrator/src/orchestrator.rs`
- `glassware-orchestrator/src/downloader.rs`
- `glassware-orchestrator/src/scanner.rs`
- `glassware-orchestrator/src/cacher.rs`
- `glassware-orchestrator/src/error.rs`

**Dependencies:**
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
clap = { version = "4", features = ["derive"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio"] }
walkdir = "2"
thiserror = "1"
futures = "0.3"
tokio-semaphore = "0.1"
```

---

### Phase 2: Advanced Features (8h)

**Tasks:**
1. Add GitHub repository downloading
2. Add progress reporting
3. Add checkpoint/resume support
4. Add JSON/SARIF output formatters
5. Add LLM analysis integration
6. Add retry logic with exponential backoff
7. Add rate limiting (npm API)
8. Integration tests

**Files to Create:**
- `glassware-orchestrator/src/github.rs`
- `glassware-orchestrator/src/progress.rs`
- `glassware-orchestrator/src/checkpoint.rs`
- `glassware-orchestrator/src/formatters/` (json.rs, sarif.rs)
- `glassware-orchestrator/src/llm.rs`

---

### Phase 3: Performance & Polish (8h)

**Tasks:**
1. Benchmark scan speed
2. Optimize memory usage
3. Add streaming results (optional)
4. Add adversarial testing integration
5. Add comprehensive error handling
6. Add logging (tracing)
7. Add metrics (optional)
8. Documentation and examples

**Files to Create:**
- `glassware-orchestrator/benches/` (benchmarks)
- `glassware-orchestrator/examples/` (examples)
- `glassware-orchestrator/README.md`

---

## Performance Targets

### Benchmarks

| Scenario | Target | Measurement |
|----------|--------|-------------|
| **Single package scan** | <2s | npm package, 50 files |
| **100 packages (parallel)** | <3min | 10 concurrent workers |
| **500 packages (parallel)** | <15min | 10 concurrent workers |
| **Cache hit scan** | <0.5s | Cached package |
| **Memory usage** | <500MB | Peak during 100-package scan |

### Optimization Strategies

1. **Connection Pooling:** Reuse HTTP connections
2. **Streaming:** Stream results instead of buffering
3. **Zero-Copy:** Minimize data copying
4. **Async I/O:** Use Tokio for all I/O
5. **Batching:** Batch database writes

---

## Migration Guide

### From Python Harness

**Before (Python):**
```bash
cd harness
python3 optimized_scanner.py packages.txt -w 10 -o results.json
```

**After (Rust):**
```bash
glassware-orchestrator scan-file --input packages.txt --output results.json --concurrency 10
```

### API Compatibility

**Python Harness API:**
- Input: Package list (one per line)
- Output: JSON with findings
- Cache: SQLite with 7-day TTL

**Rust Orchestrator API:**
- Input: Package list (one per line) OR `--packages "pkg1,pkg2"`
- Output: JSON/SARIF with findings
- Cache: SQLite with 7-day TTL (same schema)

**Backward Compatible:** ✅ Same input/output format

---

## Testing Strategy

### Unit Tests

```rust
#[tokio::test]
async fn test_downloader_npm() {
    let downloader = Downloader::new();
    let path = downloader.download_npm("lodash@4.17.21").await.unwrap();
    assert!(path.exists());
}

#[tokio::test]
async fn test_cacher_roundtrip() {
    let cacher = Cacher::new(":memory:", 7).await.unwrap();
    let result = ScanResult { /* ... */ };
    
    cacher.set("hash123", &result).await.unwrap();
    let cached = cacher.get("hash123").await.unwrap().unwrap();
    
    assert_eq!(cached.findings.len(), result.findings.len());
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_end_to_end_scan() {
    let orchestrator = Orchestrator::new(test_config());
    let results = orchestrator
        .scan_packages(vec!["lodash@4.17.21".to_string()])
        .await
        .unwrap();
    
    assert_eq!(results.len(), 1);
    assert!(results[0].findings.is_empty());  // lodash is clean
}
```

### Performance Tests

```rust
#[tokio::test]
async fn test_parallel_scan_performance() {
    let orchestrator = Orchestrator::new(test_config().with_concurrency(10));
    let packages = vec!["lodash", "axios", "moment", /* ... */];
    
    let start = Instant::now();
    let results = orchestrator.scan_packages(packages).await.unwrap();
    let elapsed = start.elapsed();
    
    assert!(elapsed < Duration::from_secs(180));  // <3min for 100 packages
    assert_eq!(results.len(), 100);
}
```

---

## Risks and Mitigations

### Risk 1: Complexity Overhead

**Risk:** Rust orchestrator is more complex than Python harness.

**Mitigation:**
- Comprehensive documentation
- Example configurations
- Gradual migration (run both in parallel)

### Risk 2: Tokio Runtime Issues

**Risk:** Async bugs, deadlocks, resource leaks.

**Mitigation:**
- Use Tokio console for debugging
- Comprehensive tests
- Resource leak detection (cargo-leaks)

### Risk 3: Breaking Changes

**Risk:** Breaking changes to glassware-core API.

**Mitigation:**
- Version glassware-core dependency
- Semantic versioning
- Backward compatibility tests

---

## Timeline

| Phase | Tasks | Effort | Dependencies |
|-------|-------|--------|--------------|
| **Phase 1: Core** | 8 tasks | 8h | None |
| **Phase 2: Advanced** | 8 tasks | 8h | Phase 1 |
| **Phase 3: Performance** | 8 tasks | 8h | Phase 1-2 |
| **Total** | 24 tasks | 24h | - |

---

## Success Metrics

### Quantitative

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Scan speed** | >100 pkg/min | Packages per minute |
| **Speedup vs Python** | >2x | Same workload |
| **Memory usage** | <500MB | Peak RSS |
| **Error rate** | <1% | Failed downloads |
| **Cache hit rate** | >50% | On re-scans |

### Qualitative

- ✅ No serialization overhead
- ✅ True parallel scanning
- ✅ Unified codebase
- ✅ Better error messages
- ✅ Streaming results support

---

## Open Questions

1. **Should we support multiple output formats in v1.0?**
   - JSON: Yes (required)
   - SARIF: Yes (required for GitHub)
   - Pretty: Maybe (nice to have)

2. **Should we support PyPI/GitHub in v1.0?**
   - npm: Yes (primary target)
   - GitHub: Yes (Phase 2)
   - PyPI: Defer to v2.0

3. **Should we replace Python harness immediately?**
   - Run both in parallel for 1 sprint
   - Deprecate Python after validation
   - Remove Python in next major version

---

## Recommendations

### v1.0 Scope (Recommended)

**Include:**
- ✅ npm package scanning
- ✅ Parallel scanning (10 workers)
- ✅ SQLite caching (7-day TTL)
- ✅ JSON output
- ✅ Checkpoint/resume
- ✅ Retry logic

**Defer to v2.0:**
- ⏳ GitHub repository scanning
- ⏳ SARIF output
- ⏳ LLM integration
- ⏳ Streaming results

### Success Criteria for v1.0

- ✅ 2x speedup over Python harness
- ✅ All existing tests pass
- ✅ Memory usage <500MB
- ✅ Documentation complete
- ✅ Migration guide written

---

**Status:** 📝 SPECIFICATION COMPLETE  
**Next:** Review spec, get approval, start implementation

**Timestamp:** 2026-03-20 13:30 UTC  
**Author:** glassware AI Assistant
