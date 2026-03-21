# Malicious Package Hunting Strategy

**Goal:** Find GlassWorm/malicious packages in the wild

**Date:** 2026-03-21

---

## High-Risk Targets

### 1. AI/Agent Ecosystem (GlassWorm Priority Target)

**Why:** Direct access to AI tools, high trust, automatic execution

**Packages to scan:**
- MCP servers (Model Context Protocol)
- LangChain extensions
- AI SDKs and wrappers
- Code interpreters
- Agent frameworks
- Vector database clients

**Search terms:**
```
mcp-server
langchain-*
@langchain/*
ai-sdk
agent-*
@agent/*
code-interpreter
tool-calling
function-calling
```

### 2. Install Script Packages

**Why:** Code execution during install, easy to hide malicious code

**Search npm for:**
```json
{
  "scripts": {
    "preinstall": "*",
    "postinstall": "*",
    "install": "*"
  }
}
```

**High-risk keywords:**
- `preinstall`
- `postinstall`
- `install-script`
- `node-gyp`
- `bindings`

### 3. Recently Published Packages (< 30 days)

**Why:** Less scrutiny, attackers strike fast

**Strategy:**
- Sample from npm "new" endpoint
- Focus on AI/ML category
- Check for anonymous authors

### 4. Typosquatting Targets

**Why:** Users mistype popular package names

**Common patterns:**
- `lodahs` (lodash)
- `expess` (express)
- `reqeust` (request)
- `axois` (axios)
- `@org/packagee` (extra letter)

### 5. Dependency Confusion

**Why:** Internal package names leaked

**Strategy:**
- Scan for packages with corporate names
- Look for `internal-*`, `corp-*`, `*-internal`

---

## GitHub Scanning Strategy

### High-Risk Repo Patterns

1. **New repos (< 7 days old)**
   - Claiming to be AI tools
   - MCP servers
   - LangChain plugins

2. **Suspicious author patterns**
   - Multiple repos by same author
   - Author name matches known attacks (JPD, etc.)

3. **Code patterns**
   - Unicode steganography
   - Base64 encoded strings
   - `eval()`, `Function()` usage
   - Network calls in install scripts

### GitHub Search Queries

```
"preinstall" "postinstall" in:file language:JavaScript created:>2026-02-01
"Model Context Protocol" created:>2026-02-01
"langchain" "plugin" created:>2026-02-01
"AI agent" "tool" created:>2026-02-01
"mcp-server" created:>2026-02-01
```

---

## Scanning Workflow

### Phase 1: Rust Orchestrator (Targeted)

```bash
# Scan specific high-value targets
./target/release/glassware-orchestrator \
  scan-npm \
  --versions last-10 \
  @modelcontextprotocol/inspector \
  @anthropic-ai/mcp-server \
  langchain \
  @langchain/core

# With LLM analysis
./target/release/glassware-orchestrator \
  --llm \
  scan-npm \
  suspicious-package
```

### Phase 2: Python Harness (Large Scale)

```bash
cd harness

# 1. Sample high-risk packages
python3 version_sampler.py \
  --output high-risk-1000.txt \
  --samples 100 \
  --categories ai-ml native-build install-scripts \
  --days 30 \
  --include-new

# 2. Run background scan
python3 background_scanner.py \
  --packages high-risk-1000.txt \
  --policy last-5 \
  --output high-risk-results.db \
  --workers 15

# 3. Monitor for malicious
watch 'sqlite3 high-risk-results.db "SELECT COUNT(*) FROM version_scans WHERE is_malicious = 1;"'
```

### Phase 3: GitHub Scanning

```bash
# Scan GitHub repos
./target/release/glassware-orchestrator \
  scan-github \
  suspicious-author/suspicious-repo

# Or use Python harness
python3 github_scanner.py \
  --queries "mcp-server" "langchain plugin" \
  --max-repos 500 \
  --output github-scan-results.json
```

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Packages scanned | 1,000+ |
| Versions scanned | 5,000+ |
| Malicious found | 1+ (goal!) |
| False positive rate | <5% |
| Scan rate | >5 ver/s |

---

## Resource Allocation

### Compute
- **Rust orchestrator:** 1-2 cores, 500MB RAM
- **Python harness:** 4-8 cores, 2GB RAM
- **GitHub scanner:** 1 core, 500MB RAM

### Rate Limits
- **npm:** 10 req/s (default), increase with token
- **GitHub:** 10 req/min (anonymous), 5000 req/hr (with token)
- **LLM:** 30 RPM (Cerebras), adjust workers accordingly

### Time Estimates
| Scan Size | Time (10 workers) |
|-----------|-------------------|
| 100 packages × 5 ver | ~2 min |
| 500 packages × 5 ver | ~10 min |
| 1,000 packages × 5 ver | ~20 min |
| 5,000 packages × 5 ver | ~2 hours |

---

## Action Plan

### Immediate (Today)
1. ✅ Test Rust orchestrator at scale (100 packages)
2. ✅ Test Python harness at scale (500 packages)
3. ⏳ Scan AI/agent targeted packages
4. ⏳ Scan recently published packages (< 30 days)

### Short-term (This Week)
5. ⏳ GitHub repo scanning
6. ⏳ Typosquatting detection
7. ⏳ Install script analysis

### Long-term
8. ⏳ Automated daily scanning
9. ⏳ Alert system for new malicious packages
10. ⏳ Public disclosure pipeline

---

## Expected Findings

Based on GlassWorm campaign history:
- **Likely targets:** AI/agent packages, MCP servers
- **Attack patterns:** Unicode steganography, install scripts
- **Authors:** Anonymous or known bad actors (JPD, etc.)
- **Timeline:** Recent (< 6 months)

---

**End of Strategy Document**
