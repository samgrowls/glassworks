# 30k Long-Horizon Scan Design

**Date:** 2026-03-19  
**Purpose:** Strategic category selection for GlassWare detection  
**Target:** 30,000 packages over ~6-8 hours  

---

## Attack Pattern Analysis

### Known GlassWare Characteristics

From intel reports (Koi Security, Aikido, Endor Labs):

| Characteristic | Target Categories | Priority |
|----------------|-------------------|----------|
| **MCP servers** | MCP, AI/ML, LLM tools | 🔴 Critical |
| **VS Code extensions** | vscode, cursor, openvsx | 🔴 Critical |
| **Install scripts** | preinstall, postinstall, node-gyp | 🔴 Critical |
| **Developer tools** | CLI, build tools, bundlers | 🟡 High |
| **Crypto/security** | crypto, auth, JWT | 🟡 High |
| **Recently published** | <30 days old | 🟡 High |
| **Low download count** | <1000/week | 🟡 High |

### Categories to AVOID (Low Yield)

| Category | Reason | Action |
|----------|--------|--------|
| Type definitions (@types/*) | No executable code, high FP rate | Skip entirely |
| Major frameworks (react, angular, vue) | Heavily audited, low risk | Reduce sampling |
| Well-known tools (lodash, moment, axios) | Too popular, attackers avoid | Reduce sampling |
| Test utilities | Rarely targeted | Reduce sampling |

---

## Category Weighting Strategy

### High-Priority Categories (60% of samples)

```python
HIGH_PRIORITY = {
    # MCP/AI ecosystem - primary GlassWare target
    "mcp-servers": ["mcp", "model-context-protocol", "mcp-server"],  # 3000 samples
    
    # VS Code extensions - 72+ compromised in Wave 5
    "vscode-extensions": ["vscode", "vsce", "vscode-extension"],  # 3000 samples
    
    # Install scripts - delivery mechanism
    "install-scripts": ["preinstall", "postinstall", "install-script"],  # 3000 samples
    
    # Native build tools - common injection point
    "native-build": ["node-gyp", "bindings", "prebuild", "nan"],  # 3000 samples
    
    # AI/ML tools - emerging target
    "ai-ml": ["ai", "llm", "langchain", "agent", "copilot"],  # 3000 samples
    
    # Crypto/auth - high-value targets
    "crypto-auth": ["crypto", "jwt", "auth", "oauth", "sso"],  # 3000 samples
}
```

### Medium-Priority Categories (30% of samples)

```python
MEDIUM_PRIORITY = {
    # Developer tools
    "cli-tools": ["cli", "command-line", "terminal"],  # 2000 samples
    
    # Build tools
    "build-tools": ["webpack", "vite", "rollup", "babel"],  # 2000 samples
    
    # Security tools (attackers target security tools)
    "security": ["security", "scanner", "audit", "lint"],  # 2000 samples
    
    # Database tools
    "database": ["orm", "query-builder", "migration"],  # 2000 samples
    
    # API tools
    "api": ["rest", "graphql", "rpc", "http-client"],  # 2000 samples
}
```

### Low-Priority Categories (10% of samples)

```python
LOW_PRIORITY = {
    # General utilities (broad sampling)
    "utilities": ["util", "helper", "common"],  # 1000 samples
    
    # Testing tools (low risk)
    "testing": ["test", "mock", "fixture"],  # 1000 samples
    
    # Documentation tools (very low risk)
    "docs": ["markdown", "doc", "readme"],  # 1000 samples
}
```

---

## Sampling Parameters

### Recommended Configuration

```bash
python3 diverse_sampling.py \
  --categories \
    mcp-servers \
    vscode-extensions \
    install-scripts \
    native-build \
    ai-ml \
    crypto-auth \
    cli-tools \
    build-tools \
    security \
    database \
    api \
    utilities \
    testing \
    docs \
  --samples-per-keyword 100 \
  --delay-between-keywords 0.3 \
  --npm-retries 5 \
  --npm-backoff 2 \
  --output diverse-30k.txt
```

**Expected output:**
- 13 categories × ~15 keywords × 100 samples = ~19,500 packages
- Plus overlap/duplicates = ~25-30k unique packages
- Sampling time: ~2-3 hours (0.3s delay between searches)

---

## Scan Parameters

### Recommended Scan Configuration

```bash
python3 optimized_scanner.py \
  diverse-30k.txt \
  -w 10 \
  -e data/evidence/scan-30k-long-horizon \
  -o scan-30k-results.json \
  -n 30k-long-horizon
```

**Expected performance:**
- 30k packages ÷ 10 workers = 3k packages per worker
- ~0.5s per package (with cache)
- Total time: ~4-5 hours
- Cache hit rate: Expected 15-25% (some packages scanned in previous runs)

---

## Monitoring Strategy

### Real-Time Metrics

```bash
# Watch scan progress
watch -n 60 'cat scan-30k-results.json | jq "{scanned: .scanned, flagged: .flagged, cached: .cached, errors: .errors}"'

# Track flagged rate
watch -n 300 'cat scan-30k-results.json | jq ".flagged / .scanned * 100 | floor"'

# Monitor for high-severity findings
watch -n 60 'cat scan-30k-results.json | jq "[.flagged_packages[] | select(.critical > 10)] | length"'
```

### Alert Thresholds

| Metric | Warning | Critical | Action |
|--------|---------|----------|--------|
| Flagged rate | >10% | >20% | Review for FP pattern |
| Error rate | >10% | >20% | Check npm API, disk space |
| High-critical findings | >5 | >20 | Immediate LLM analysis |
| Scan speed | <1 pkg/s | <0.5 pkg/s | Check system resources |

---

## Post-Scan Analysis Plan

### Phase 1: Triage (30 min)

```bash
# Extract top 50 by critical count
cat scan-30k-results.json | jq '.flagged_packages | sort_by(-.critical) | .[:50] | .[].package' > top50-critical.txt

# Run LLM on top 50
python3 batch_llm_analyzer.py top50-critical.txt -w 3 -e data/evidence/llm-30k-top50 -o llm-30k-top50.json
```

### Phase 2: Deep Analysis (2 hours)

```bash
# Analyze LLM results
cat llm-30k-top50.json | jq '.results[] | select(.llm_classification == "MALICIOUS")'

# Manual review of MALICIOUS classifications
# Prepare disclosure if confirmed
```

### Phase 3: Pattern Learning (1 hour)

```bash
# Extract FP patterns
cat llm-30k-top50.json | jq '.results[] | select(.llm_classification == "FALSE_POSITIVE")'

# Document new FP patterns
# Update bundled code filters if needed
```

---

## Expected Outcomes

### Based on 506-Package Validation

| Metric | Expected | Notes |
|--------|----------|-------|
| Total packages | 30,000 | Target |
| Flagged rate | 4-6% | Based on 4.7% from 506 scan |
| MALICIOUS (LLM) | 0-5 | Random sampling, low yield expected |
| SUSPICIOUS (LLM) | 50-100 | Need human review |
| FALSE_POSITIVE (LLM) | 100-200 | Legitimate frameworks/tools |

### Success Criteria

| Criterion | Target | Status |
|-----------|--------|--------|
| System stability | No crashes | TBD |
| Rate limit handling | Zero 429 errors | TBD |
| Cache effectiveness | >15% hit rate | TBD |
| FP rate | <10% | TBD |
| MALICIOUS findings | Any confirmed | TBD |

---

## Rollback Plan

**If scan fails:**
1. Check disk space: `df -h`
2. Check npm API: `curl https://registry.npmjs.org/-/ping`
3. Reduce workers: `-w 5` instead of `-w 10`
4. Resume from checkpoint: Scan results are incremental

**If too many FPs:**
1. Review top 20 flagged manually
2. Identify common patterns
3. Update bundled code filters
4. Re-run scan on remaining packages

---

**Ready to execute. Awaiting approval to start 30k scan.**
