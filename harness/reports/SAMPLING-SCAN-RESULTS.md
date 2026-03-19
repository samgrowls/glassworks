# Sampling Scan Results - 3 Categories

**Date:** 2026-03-18  
**Total Packages:** 166 (25 + 41 + 100)  
**Status:** ✅ COMPLETE

---

## Category A: VS Code Extensions (25 packages)

| Metric | Count |
|--------|-------|
| Scanned | 24 |
| Flagged | 8 |
| Errors | 1 |
| **Flagged Rate** | **33%** |

### Top Flagged
| Package | Findings | Critical |
|---------|----------|----------|
| `@_davideast/stitch-mcp` | 147 | 122 |
| `@modelcontextprotocol/server-video-resource` | 71 | 56 |
| `@salesforce/mcp-provider-aura-experts` | 29 | 22 |
| `@sparkleideas/mcp` | 2 | 2 |

---

## Category B: Popular Packages (41 packages)

| Metric | Count |
|--------|-------|
| Scanned | 39 |
| Flagged | 8 |
| Errors | 2 |
| **Flagged Rate** | **21%** |

### Top Flagged
| Package | Findings | Critical |
|---------|----------|----------|
| `@googlemaps/code-assist-mcp` | 52 | 37 |
| `@google-cloud/gcloud-mcp` | 11 | 10 |
| `@google-cloud/observability-mcp` | 11 | 10 |
| `@google-cloud/storage-mcp` | 11 | 10 |
| `@ai-sdk/google-vertex` | 15 | 0 |

---

## Category C: Recent/AI Packages (100 packages)

| Metric | Count |
|--------|-------|
| Scanned | 94 |
| Flagged | 29 |
| Errors | 6 |
| **Flagged Rate** | **31%** |

### Top Flagged
| Package | Findings | Critical |
|---------|----------|----------|
| `@midscene/mcp` | 100 | 72 |
| `@launchdarkly/mcp-server` | 69 | 46 |
| `@larksuiteoapi/lark-mcp` | 56 | 56 |
| `@mcp-ui/server` | 40 | 38 |
| `@mcp-ui/client` | 34 | 32 |
| `mcp-proxy` | 34 | 23 |
| `@notionhq/notion-mcp-server` | 20 | 18 |
| `@sentry/mcp-server` | 18 | 18 |
| `@gleanwork/mcp-config-schema` | 24 | 18 |
| `@composio/mcp` | 13 | 12 |
| `@aikidosec/mcp` | 22 | 11 |
| `@railway/mcp-server` | 11 | 7 |

---

## Combined Summary

| Category | Scanned | Flagged | Flagged % |
|----------|---------|---------|-----------|
| VS Code Extensions | 24 | 8 | 33% |
| Popular Packages | 39 | 8 | 21% |
| Recent/AI Packages | 94 | 29 | 31% |
| **TOTAL** | **157** | **45** | **29%** |

---

## Key Observations

### High-Risk Categories
1. **VS Code Extensions** - 33% flagged (highest rate)
2. **Recent/AI Packages** - 31% flagged (AI/LLM trend)
3. **Popular Packages** - 21% flagged (lowest, but still significant)

### Notable Finds
- `@_davideast/stitch-mcp` - 147 findings (already known from MCP scan)
- `@launchdarkly/mcp-server` - 69 findings (bundled code, likely FP)
- `@midscene/mcp` - 100 findings (bundled code, likely FP)
- `@aikidosec/mcp` - 22 findings (Aikido's own package - needs review)

### Priority for LLM Analysis
1. `@railway/mcp-server` - 11 findings, 7 critical (legitimate company)
2. `@gleanwork/mcp-config-schema` - 24 findings, 18 critical (enterprise company)
3. `@sentry/mcp-server` - 18 findings, 18 critical (well-known monitoring tool)
4. `@notionhq/notion-mcp-server` - 20 findings, 18 critical (Notion official)

---

## Next Steps

### LLM Analysis Priority
1. **High-count, legitimate companies** (likely FPs but verify)
   - `@launchdarkly/mcp-server`
   - `@gleanwork/mcp-config-schema`
   - `@sentry/mcp-server`
   - `@notionhq/notion-mcp-server`

2. **Medium-count, unknown publishers** (higher risk)
   - `@mcp-ui/server`
   - `@mcp-ui/client`
   - `mcp-proxy`
   - `@railway/mcp-server`

3. **Previously analyzed** (skip or re-verify)
   - `@_davideast/stitch-mcp` (already in LLM queue)
   - `@midscene/mcp` (likely bundled FP)
   - `@aikidosec/mcp` (already analyzed)

---

**Evidence saved to:**
- `data/evidence/scan-vscode/`
- `data/evidence/scan-popular/`
- `data/evidence/scan-recent/`

**Results saved to:**
- `scan-vscode-results.json`
- `scan-popular-results.json`
- `scan-recent-results.json`
