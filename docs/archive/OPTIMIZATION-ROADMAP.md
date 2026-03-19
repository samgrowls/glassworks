# glassware Optimization Roadmap

**Created:** 2026-03-18  
**Status:** In Progress  
**Priority:** High → Low

---

## 📊 Current Performance Baseline

| Metric | Current | Target | Gap |
|--------|---------|--------|-----|
| Scan speed (per package) | ~1-2s | ~0.5s | 2-4x |
| Harness throughput | ~34 pkg/min | ~200 pkg/min | 6x |
| False positive rate | ~10% | <1% | 10x |
| LLM analysis time | ~10s/finding | ~2s/finding | 5x |

---

## 🎯 Phase 1: Quick Wins (This Week)

### 1.1 ✅ Parallel Package Scanning
**Status:** IMPLEMENTED (`optimized_scanner.py`)  
**Impact:** 4-6x speedup  
**Effort:** Low

```bash
# Already working:
python optimized_scanner.py packages.txt --workers 10
```

### 1.2 🔄 Package Cache (Avoid Re-scans)
**Status:** TODO  
**Impact:** 10x on re-runs  
**Effort:** Low

```python
# Add to database.py
def is_already_scanned(name, version, sha256):
    """Check if package with same hash was already scanned."""
    # Query corpus.db for matching name/version/hash
    pass
```

**Implementation:**
- [ ] Add `tarball_sha256` index to packages table
- [ ] Check cache before downloading
- [ ] Add `--force` flag to re-scan cached packages

### 1.3 🔄 Skip Known Clean Directories
**Status:** TODO  
**Impact:** 2x faster on large packages  
**Effort:** Low

```rust
// glassware-core/src/scanner.rs
const SKIP_DIRS: &[&str] = &[
    "node_modules",
    "test",
    "tests",
    "__tests__",
    "fixtures",
    "examples",
    "docs",
    ".git",
];

fn should_skip_file(path: &str) -> bool {
    SKIP_DIRS.iter().any(|dir| path.contains(dir))
}
```

### 1.4 🔄 LLM Batch Analysis
**Status:** TODO  
**Impact:** 5x faster LLM analysis  
**Effort:** Medium

```python
# Instead of 1 API call per finding:
async def analyze_batch(findings_batch, source_context):
    """Analyze 5-10 findings in single API call"""
    prompt = f"Analyze these {len(findings_batch)} findings:\n{findings_batch}"
    # Single API call returns analysis for all
```

---

## 🎯 Phase 2: Architecture Improvements (Next Week)

### 2.1 🔄 Streaming Scan (No Extraction)
**Status:** TODO  
**Impact:** 3x faster, less disk I/O  
**Effort:** Medium

```python
# Instead of: download → extract → scan
# Do: download → stream scan → discard

def scan_tarball_streaming(tarball_path):
    with tarfile.open(tarball_path, 'r:gz') as tar:
        for member in tar.getmembers():
            if member.isfile() and is_relevant_file(member.name):
                content = tar.extractfile(member).read()
                findings = scan_content(content, member.name)
```

### 2.2 🔄 Worker Pool (Rust-based)
**Status:** TODO  
**Impact:** 10x throughput  
**Effort:** Medium

```
Architecture:
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│   Python    │ →   │  Rust Worker │ →   │   Results   │
│ Orchestrator│     │   Pool (4x)  │     │  Aggregator │
└─────────────┘     └──────────────┘     └─────────────┘
```

### 2.3 🔄 Incremental Scanning
**Status:** TODO  
**Impact:** 100x on re-scans  
**Effort:** Medium

```python
# Hash-based change detection
def needs_rescan(package):
    old_hash = get_cached_hash(package)
    new_hash = compute_hash(package)
    return old_hash != new_hash
```

### 2.4 🔄 npm Changes API Integration
**Status:** TODO  
**Impact:** Real-time monitoring  
**Effort:** Low

```python
# Use npm's changes feed for new packages
# https://replicate.npmjs.com/_changes

def get_new_packages(since_sequence):
    """Fetch packages published since last scan."""
    response = requests.get(
        "https://replicate.npmjs.com/_changes",
        params={"since": since_sequence}
    )
    return parse_changes(response.json())
```

---

## 🎯 Phase 3: Advanced Optimizations (Future)

### 3.1 🔄 ML Pre-filter
**Status:** TODO  
**Impact:** Skip 90% of clean packages  
**Effort:** High

```
Train classifier on:
- Package metadata (author, downloads, age)
- File structure (presence of install scripts)
- Simple patterns (no ML needed for first pass)

Skip packages with <1% malicious probability
```

### 3.2 🔄 Distributed Scanning
**Status:** TODO  
**Impact:** 100x throughput  
**Effort:** High

```
Deploy on multiple nodes:
- Node 1: Packages A-M
- Node 2: Packages N-Z
- Central aggregator

Or use serverless (AWS Lambda, Cloudflare Workers)
```

### 3.3 🔄 GPU Entropy Calculation
**Status:** TODO  
**Impact:** 10x faster on large payloads  
**Effort:** High

```
Use SIMD/GPU for:
- Shannon entropy on large blobs
- Pattern matching across corpora
- Batch LLM inference
```

---

## 📋 False Positive Reduction

### Current FP Sources

| Source | Count | Fix | Status |
|--------|-------|-----|--------|
| README emoji | High | Skip .md files | ✅ DONE |
| Bundled code | High | Heuristics + LLM | 🔄 In Progress |
| i18n files | Medium | Skip ZWNJ/ZWJ in i18n | ✅ DONE |
| Test fixtures | Low | Skip test directories | 🔄 TODO |
| Minified code | Medium | LLM analysis | 🔄 TODO |

### FP Reduction Plan

1. **File type heuristics** (Week 1)
   - Skip `.min.js`, `.bundle.js`, `.umd.js`
   - Flag bundled files for LLM review only

2. **Size-based filtering** (Week 1)
   - Files >500KB → LLM review only
   - Files >1MB → Skip unless high-severity pattern

3. **LLM confidence tuning** (Week 2)
   - Adjust prompt for better FP/TP balance
   - Add FP examples to prompt

4. **Publisher allowlist** (Week 2)
   - Skip known-trusted publishers (npm, vercel, google, microsoft)
   - Flag unknown publishers for review

---

## 📊 Expected Performance After Optimization

| Phase | Packages/minute | 500 pkg time | 10K pkg time |
|-------|-----------------|--------------|--------------|
| **Current** | ~34 | ~15 min | ~5 hours |
| **Phase 1** | ~100 | ~5 min | ~1.5 hours |
| **Phase 2** | ~300 | ~1.5 min | ~30 min |
| **Phase 3** | ~1000+ | ~30 sec | ~10 min |

---

## 🎯 Immediate Next Steps (Today)

1. ✅ **Test LLM on LaunchDarkly** - Verify it's FP (bundled code)
2. 🔄 **Continue optimized scan** - Remaining ~1,000 MCP packages
3. 🔄 **Implement package cache** - Avoid re-downloads
4. 🔄 **Add skip directories** - node_modules, tests, etc.

---

## 📝 Implementation Priority

### This Week (High Priority)
- [x] Parallel scanning
- [ ] Package cache
- [ ] Skip directories
- [ ] LLM batch analysis
- [ ] File type heuristics (skip .min.js, etc.)

### Next Week (Medium Priority)
- [ ] Streaming scan
- [ ] Worker pool
- [ ] Publisher allowlist
- [ ] Size-based filtering

### Future (Low Priority)
- [ ] ML pre-filter
- [ ] Distributed scanning
- [ ] GPU acceleration

---

**Owner:** glassware team  
**Last Updated:** 2026-03-18  
**Next Review:** After Phase 1 complete
