# MCP Phase 2 - Interim Results (First 68 packages)

**Started:** 2026-03-18T18:03Z  
**Status:** IN PROGRESS (scan running)  
**Progress:** 68/100 packages scanned

---

## Preliminary Findings

### High-Severity Finds (Critical >50)

| Package | Findings | Critical | Notes |
|---------|----------|----------|-------|
| `@midscene/mcp` | 3,288 | 72 | 🚨 MAJOR |
| `chrome-devtools-mcp` | 1,315 | 107 | 🚨 MAJOR |
| `mcp-proxy` | 831 | 23 | 🚨 MAJOR |
| `@apify/actors-mcp-server` | 345 | 87 | 🚨 MAJOR |
| `@aikidosec/mcp` | 22 | 11 | (Aikido's own package!) |
| `@sap-ux/fiori-mcp-server` | 62 | 48 | 🚨 MAJOR |
| `@larksuiteoapi/lark-mcp` | 56 | 56 | 🚨 MAJOR |

### Medium-Severity (10-50 findings)

| Package | Findings | Critical |
|---------|----------|----------|
| `@mcp-ui/server` | 40 | 30 |
| `@mcp-ui/client` | 34 | 32 |
| `@modelcontextprotocol/ext-apps` | 32 | 30 |
| `@composio/mcp` | 13 | 12 |
| `@notionhq/notion-mcp-server` | 20 | 18 |
| `@sentry/mcp-server` | 18 | 18 |

### Low-Severity (<10 findings)

| Package | Findings | Critical |
|---------|----------|----------|
| `@upstash/context7-mcp` | 7 | 7 |
| `chrome-local-mcp` | 2 | 0 |
| `n8n-mcp` | 1 | 1 |
| `@clerk/mcp-tools` | 5 | 1 |
| `@azure-devops/mcp` | 2 | 2 |
| `browser-devtools-mcp` | 7 | 1 |
| `tavily-mcp` | 2 | 2 |
| `@ui5/mcp-server` | 4 | 4 |
| `mcp-framework` | 2 | 0 |
| `@payloadcms/plugin-mcp` | 4 | 0 |
| `@inkeep/agents-mcp` | 10 | 0 |
| `@heroku/mcp-server` | 1 | 0 |
| `mcp-use` | 6 | 0 |
| `@zereight/mcp-gitlab` | 1 | 0 |

---

## Key Observations

### 1. MAJOR Discovery: @midscene/mcp (3,288 findings!)

This is comparable to the watercrawl-mcp package (9,000+ findings). Likely large steganographic payload.

### 2. Aikido's Own Package Flagged!

`@aikidosec/mcp` - 22 findings (11 critical)  
**Irony:** Aikido Security reported GlassWare, but their own package is flagged!

**Possible explanations:**
- False positive (their detectors should catch this)
- They injected it intentionally (research?)
- They were compromised too
- Testing/dummy package

### 3. Multiple High-Profile Packages

- `@notionhq/notion-mcp-server` (Notion's official MCP)
- `@sentry/mcp-server` (Sentry)
- `@sap-ux/fiori-mcp-server` (SAP)
- `@azure-devops/mcp` (Microsoft Azure)

### 4. Pattern Suggests Widespread Contamination

**68 packages scanned:**
- ~20 flagged (29%)
- ~48 clean (71%)

This is NOT random - suggests either:
- Widespread compromise of MCP ecosystem
- **OR** many false positives from legitimate patterns

---

## Evidence Collected

**30 packages backed up** to `harness/data/evidence/mcp-phase2/`

Total size: ~66 MB

Notable files:
- `@midscene_mcp.tgz` (28 MB) - Largest
- `@aikidosec_mcp.tgz` (22 MB)
- `chrome-devtools-mcp.tgz` (2.5 MB)
- `n8n-mcp.tgz` (28 MB)

---

## Next Steps

1. **Let scan complete** (32 packages remaining)
2. **Deep analysis** of top 5 finds
3. **Verify @aikidosec/mcp** - contact them?
4. **Check if these are known** or new discoveries

---

**Last Updated:** 2026-03-18T18:13Z  
**Scan Status:** RUNNING
