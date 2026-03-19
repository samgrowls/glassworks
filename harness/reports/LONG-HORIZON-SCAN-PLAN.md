# Long-Horizon Scan Plan

**Date:** 2026-03-19  
**Target:** 500-1,000 packages (quality over quantity)  
**Focus:** High-risk categories with GlassWare patterns  

---

## Problem with Previous "30k" Scan

**Issue:** Sampling script only had hardcoded categories  
**Result:** 2,242 packages instead of 30,000  
**Root cause:** Categories like `mcp-servers`, `vscode-extensions` weren't in predefined list

---

## New Strategy: Targeted High-Risk Sampling

Instead of trying to scan 30k random packages, we'll scan **500-1,000 high-risk packages** from categories most likely to contain GlassWare:

### Priority 1: MCP/AI Ecosystem (200 packages)
**Why:** Primary GlassWare target, high-value, recent infections
```
Keywords: mcp, model-context, ai-agent, llm-tool, ai-plugin, copilot, cursor
Samples: 200 (10 per keyword × 20 keywords)
```

### Priority 2: VS Code Extensions (150 packages)
**Why:** 72+ extensions compromised in Wave 5
```
Keywords: vscode-extension, vsce, openvsx, vscode-plugin
Samples: 150
```

### Priority 3: Install Scripts (150 packages)
**Why:** Delivery mechanism for GlassWare
```
Keywords: preinstall, postinstall, install-script, node-gyp, bindings
Samples: 150
```

### Priority 4: Developer Tools (100 packages)
**Why:** High trust, widely installed
```
Keywords: devtools, build-tool, bundler, transpiler, linter
Samples: 100
```

### Priority 5: Crypto/Security (100 packages)
**Why:** Legitimate crypto vs malicious crypto hard to distinguish
```
Keywords: crypto, encryption, auth, jwt, wallet, blockchain
Samples: 100
```

**Total:** 700 packages

---

## Sampling Parameters

```bash
python3 diverse_sampling.py \
  --categories \
    mcp \
    ai-agent \
    llm-tool \
    vscode-extension \
    preinstall \
    postinstall \
    node-gyp \
    devtools \
    crypto \
    wallet \
  --samples-per-keyword 15 \
  --delay-between-keywords 0.5 \
  --npm-retries 5 \
  --output high-risk-700.txt
```

**Expected:** 700 packages in ~30 minutes

---

## Scan Parameters

```bash
python3 optimized_scanner.py \
  high-risk-700.txt \
  -w 10 \
  -e data/evidence/scan-high-risk-700 \
  -o scan-high-risk-700-results.json \
  -n high-risk-700
```

**Expected:** 700 packages in ~1 hour (with 10 workers)

---

## Success Metrics

| Metric | Target | Notes |
|--------|--------|-------|
| Packages scanned | 700 | High-risk categories |
| Flagged rate | 3-7% | Expected 21-49 packages |
| Confirmed malicious | >0 | At least 1 real find |
| False positive rate | <10% | After tuning |
| Scan duration | <2 hours | Total time |

---

## Why This Approach is Better

### Quality Over Quantity
- **700 targeted** > 30,000 random
- Higher likelihood of finding real threats
- Faster to scan and analyze
- Easier to tune detectors

### Focused Categories
- MCP/AI: Confirmed GlassWare target
- VSCode: 72+ compromised extensions
- Install scripts: Known delivery mechanism
- Dev tools: High trust, wide installation

### Manageable Analysis
- 21-49 flagged packages (vs 91 from 2,242)
- Can manually review all flagged
- Faster iteration on detector tuning

---

## Execution Plan

### Phase 1: Sampling (30 min)
```bash
cd harness
python3 diverse_sampling.py --categories mcp ai-agent llm-tool vscode-extension preinstall postinstall node-gyp devtools crypto wallet --samples-per-keyword 15 --delay-between-keywords 0.5 -o high-risk-700.txt
```

### Phase 2: Scanning (1 hour)
```bash
python3 optimized_scanner.py high-risk-700.txt -w 10 -e data/evidence/scan-high-risk-700 -o scan-high-risk-700-results.json
```

### Phase 3: LLM Analysis (30 min)
```bash
# Extract flagged
cat scan-high-risk-700-results.json | jq -r '.flagged_packages[].package' > flagged-700.txt

# Run LLM
python3 batch_llm_analyzer.py flagged-700.txt -w 3 -e data/evidence/llm-high-risk-700 -o llm-high-risk-700-results.json
```

### Phase 4: Manual Review (30 min)
- Review MALICIOUS classifications
- Investigate high-risk patterns
- Prepare disclosure if confirmed

**Total Time:** ~2.5 hours

---

## Alternative: Full npm Registry Scan

If we want to scan more packages, we can:

### Option A: Multiple Batches
```bash
# Batch 1: MCP/AI (200 packages)
python3 diverse_sampling.py --categories mcp ai-agent llm-tool --samples-per-keyword 20 -o batch1-mcp.txt
python3 optimized_scanner.py batch1-mcp.txt -w 10 -e data/evidence/batch1-mcp

# Batch 2: VSCode (200 packages)
python3 diverse_sampling.py --categories vscode-extension vsce openvsx --samples-per-keyword 20 -o batch2-vscode.txt
python3 optimized_scanner.py batch2-vscode.txt -w 10 -e data/evidence/batch2-vscode

# Continue for more batches...
```

### Option B: Random Sampling
```bash
# Sample from npm registry top downloads
# (Requires npm API access or pre-built list)
python3 sample-top-downloads.py --top 10000 --output top-10k.txt
python3 optimized_scanner.py top-10k.txt -w 10 -e data/evidence/top-10k
```

**Expected:** 10,000 packages in ~10 hours

---

## Recommendation

**Start with 700 high-risk packages** (2.5 hours)
- Quick results
- High likelihood of finding threats
- Manageable analysis workload

**Then scale based on results:**
- If we find malicious: Expand to 2,000-5,000 packages
- If no finds: Re-evaluate categories or detector sensitivity

---

**Ready to execute. Awaiting approval to start 700-package high-risk scan.**
