# Scan Status Dashboard

**Last Updated:** 2026-03-21T07:50:00Z  
**Version:** v0.8.8.5

---

## Active Scans

### 1. Rust Orchestrator - AI/Agent Targeted Scan ✅ COMPLETE

**Status:** ✅ Completed  
**Packages:** 40 high-value AI/agent targets  
**Time:** 25 seconds  
**Results:**
- Scanned: 30 packages
- Failed: 10 (404 - scoped packages unavailable)
- Malicious: 0
- Findings: 0

**Targets included:**
- MCP servers (@modelcontextprotocol/*)
- LangChain ecosystem (@langchain/*)
- AI SDKs (@ai-sdk/*, @vercel/ai)
- Agent frameworks (crewai, autoagent, agentops)
- Infrastructure (prisma, express, axios, etc.)

**Conclusion:** Popular AI/agent packages are clean (expected)

---

### 2. Python Harness - Large Scale Scan 🔄 RUNNING

**Status:** 🔄 Sampling in progress  
**Target:** 250 packages (5 categories × 50 samples)  
**Categories:**
- ai-ml (AI/ML packages)
- native-build (node-gyp, bindings)
- install-scripts (preinstall, postinstall)
- utils (lodash, axios, etc.)
- crypto (bcrypt, jsonwebtoken, jose)

**Time Range:** Last 60 days  
**Estimated completion:** ~10 minutes for sampling, ~20 minutes for scanning

**Command:**
```bash
python3 version_sampler.py \
  --output scan-500.txt \
  --samples 50 \
  --categories ai-ml native-build install-scripts utils crypto \
  --days 60 \
  --include-popular
```

---

### 3. Agent-Targeted Scan ✅ COMPLETE

**Status:** ✅ Completed  
**Packages:** 180 agent-related packages  
**Versions:** 503 (last-3 of each)  
**Results:**
- Scanned: 503 versions
- Failed: 0
- Malicious: 0
- Findings: 0

**Conclusion:** Well-maintained agent packages are clean

---

## Historical Scans

### Phase 5 QA Scan ✅
- **Packages:** 119
- **Versions:** 1,067
- **Rate:** 8.4 ver/s
- **Malicious:** 0

### Phase 4 Sampler Test ✅
- **Packages:** 10
- **Time:** 10 seconds
- **Success:** 100%

---

## Scan Statistics (Total)

| Metric | Value |
|--------|-------|
| Total packages scanned | 763 |
| Total versions scanned | 2,073 |
| Malicious found | 0 |
| Total findings | 0 |
| Scan failures | 10 (404s) |
| Average scan rate | 6.5 ver/s |

---

## Key Learnings

### What Works ✅
1. **Rust orchestrator** - Fast for targeted scans (30 pkgs in 25s)
2. **Python harness** - Better for large batches
3. **Version sampling** - Now correctly gets newest versions
4. **JSON parsing** - Fixed to handle log output
5. **Path resolution** - Fixed for background scanner

### What Doesn't Work ❌
1. **Scoped package availability** - Many @org/package return 404
2. **Old versions** - npm unpubs old versions (expected behavior)

### Recommendations
1. ✅ Use `last-3` or `last-5` for recent versions
2. ✅ Focus on unscoped packages or verify scoped package names
3. ✅ Run both Rust and Python scanners for different use cases

---

## Next Actions

### Immediate
1. ⏳ Wait for 500-package sampler to complete
2. ⏳ Run background scan on 500 packages
3. ⏳ Monitor for any malicious findings

### Short-term
4. ⏳ GitHub repository scanning
5. ⏳ Typosquatting detection scan
6. ⏳ Recently published (< 7 days) scan

### Strategic
7. ⏳ Set up automated daily scanning
8. ⏳ Create alert system for new malicious packages
9. ⏳ Prepare disclosure pipeline

---

## Resource Usage

| Scanner | CPU | RAM | Disk |
|---------|-----|-----|------|
| Rust orchestrator | ~100% (1 core) | ~200MB | ~50MB cache |
| Python harness | ~400% (4 cores) | ~500MB | ~200MB DB |
| GitHub scanner | ~50% (1 core) | ~100MB | ~10MB cache |

---

## Files Generated

| File | Size | Purpose |
|------|------|---------|
| `/tmp/rust-scan-cache.db` | ~1MB | Rust scanner cache |
| `harness/agent-scan-results.db` | ~200KB | Agent-targeted results |
| `harness/phase5-qa-results.db` | ~200KB | Phase 5 QA results |
| `harness/scan-500.txt` | ~5KB | 500 package list (generating) |

---

**End of Dashboard**
