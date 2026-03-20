# Python Harness Baseline Metrics

**Date:** 2026-03-20 15:45 UTC  
**Version:** Python harness (v0.7.0 glassware-scanner binary)  
**Status:** ✅ BASELINE RECORDED  

---

## Test Configuration

**Package List:** `diverse-500.txt` (110 unique packages after dedup)  
**Workers:** 10  
**Cache:** Enabled (7-day TTL)  
**Evidence Dir:** `data/evidence/baseline`  

---

## Baseline Metrics

### With Cache (Realistic Production)

| Metric | Value |
|--------|-------|
| **Total packages** | 110 |
| **Scanned (new)** | 0 (all cached) |
| **Cached** | 106 (96% hit rate) |
| **Flagged** | 0 |
| **Errors** | 4 (@types/* extraction issues) |
| **Total time** | 1m35s |
| **Speed** | ~1.2 pkg/s (with cache) |

### Without Cache (Cold Start)

| Metric | Value |
|--------|-------|
| **Speed** | ~0.5 pkg/s (from previous scans) |
| **Memory** | ~200MB peak |
| **Error rate** | ~3-5% |

---

## Performance Characteristics

### Bottlenecks Identified

1. **Subprocess overhead** - Spawns glassware-scanner per package
2. **JSON serialization** - Parse JSON output per package
3. **npm download** - Sequential package downloads
4. **Tar extraction** - Sequential tarball extraction

### Optimization Opportunities

1. **Parallel downloads** - Currently sequential
2. **Streaming results** - Currently buffer all
3. **Direct integration** - Eliminate subprocess (Rust orchestrator)

---

## Targets for Rust Orchestrator

| Metric | Python Baseline | Rust Target | Improvement |
|--------|-----------------|-------------|-------------|
| **Speed (with cache)** | 1.2 pkg/s | 3+ pkg/s | 2.5x |
| **Speed (no cache)** | 0.5 pkg/s | 1.5+ pkg/s | 3x |
| **Memory** | ~200MB | <500MB | Same |
| **Error rate** | 3-5% | <1% | 3-5x better |

---

## Notes

**Cache effectiveness:** 96% hit rate on second scan (excellent)  
**Error pattern:** @types/* packages fail extraction (expected)  
**Flagged rate:** 0% (diverse-500.txt has clean packages)

---

**Timestamp:** 2026-03-20 15:45 UTC  
**Measured by:** glassware AI Assistant
