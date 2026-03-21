# Rust Orchestrator vs Python Harness — Capability Comparison

**Date:** 2026-03-21  
**Status:** Rust orchestrator needs version parsing fix

---

## Executive Summary

The Rust orchestrator has **feature parity** with the Python harness in terms of capabilities, but has a **bug in version parsing** that prevents scanning specific versions (e.g., `express@4.19.2`).

**Status:**
- ✅ Rust orchestrator: Full feature set, needs version parsing fix
- ✅ Python harness: Fully operational, tested with Wave 0

---

## Rust Orchestrator Capabilities

### CLI Commands

```bash
glassware-orchestrator --help

Commands:
  scan-npm          Scan npm packages
  scan-github       Scan GitHub repositories
  search-github     Search GitHub repositories
  scan-file         Scan packages from a file
  resume            Resume an interrupted scan
  cache-stats       Show cache statistics
  sample-packages   Sample packages from npm by category
  cache-cleanup     Clean up expired cache entries
  scan-list         List scan history
  scan-show         Show scan details
  scan-cancel       Cancel a running scan
```

### Features

| Feature | Rust Orchestrator | Python Harness |
|---------|-------------------|----------------|
| npm package scanning | ✅ | ✅ |
| GitHub repo scanning | ✅ | ✅ |
| Version scanning (last-10, etc.) | ✅ (buggy) | ✅ |
| SQLite caching | ✅ | ✅ |
| Checkpoint/resume | ✅ | ✅ |
| Rate limiting | ✅ | ✅ |
| Retry logic | ✅ | ✅ |
| Parallel scanning | ✅ (tokio async) | ✅ (ThreadPoolExecutor) |
| LLM analysis | ✅ (Cerebras) | ✅ (NVIDIA) |
| Adversarial testing | ✅ | ❌ |
| SARIF output | ✅ | ❌ |
| JSON Lines streaming | ✅ | ❌ |
| Category sampling | ✅ | ✅ |
| Scan history | ✅ | ❌ |

---

## Bug Identified: Version Parsing

### Problem

The Rust orchestrator fails to scan packages with specific versions:

```bash
glassware-orchestrator scan-npm express@4.19.2
# Error: Package not found: express@4.19.2
```

### Root Cause

In `downloader.rs:217-220`:
```rust
pub async fn get_npm_package_info(&self, package: &str) -> Result<NpmPackageInfo> {
    let url = format!("https://registry.npmjs.org/{}", package);
    // ...
}
```

The code passes the full package spec (`express@4.19.2`) to the npm registry API, but the API endpoint expects just the package name (`express`).

### Fix Required

Parse the package spec to extract name and version separately:

```rust
// Parse "express@4.19.2" → name="express", version="4.19.2"
// Then: https://registry.npmjs.org/express
// Then: select version from response.versions["4.19.2"]
```

---

## Python Harness Advantages

1. **Tested and working** — Wave 0 validation passed
2. **NVIDIA LLM integration** — Stronger models (qwen3.5-397b)
3. **Model fallback** — Automatic fallback through 4 models
4. **Wave-based orchestration** — waves.toml configuration
5. **Simpler API** — Easier to extend and modify

---

## Rust Orchestrator Advantages

1. **Performance** — Native async/await, no GIL
2. **Adversarial testing** — Built-in polymorphic testing
3. **SARIF output** — GitHub Advanced Security integration
4. **JSON Lines streaming** — Real-time output for large scans
5. **Scan history** --scan-list, --scan-show commands
6. **Better error handling** — Structured error types with context

---

## Recommendation

### Short-term (Next 1-2 weeks)

**Use Python harness for production scans:**
- Fully tested and operational
- Wave 0 validated end-to-end
- NVIDIA LLM integration working

### Medium-term (Next 2-4 weeks)

**Fix Rust orchestrator version parsing:**
1. Add package spec parser (name@version → name + version)
2. Update `get_npm_package_info()` to use parsed name
3. Update `download_npm_package()` to use parsed version
4. Add tests for version parsing
5. Re-test with Wave 0 packages

### Long-term (1-2 months)

**Feature consolidation:**
- Consider porting Python wave orchestration to Rust
- Add NVIDIA LLM integration to Rust (currently only Cerebras)
- Unify caching layer (both use SQLite but different schemas)

---

## Test Results

### Rust Orchestrator Test

```bash
glassware-orchestrator --concurrency 2 --no-cache scan-npm express@4.19.2 lodash@4.17.21

# Result: FAILED
# Error: Package not found: express@4.19.2
# Error: Package not found: lodash@4.17.21
```

### Python Harness Test

```bash
python3 -c "from harness.core import Orchestrator; ..."

# Result: SUCCESS
# Packages scanned: 10/10
# Packages flagged: 4
# Errors: 0
```

---

## Sign-Off

**Rust orchestrator has feature parity but needs version parsing fix before production use.**

**Python harness is ready for immediate production use.**
