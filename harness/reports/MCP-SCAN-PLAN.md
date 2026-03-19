# MCP Ecosystem Scan Plan

**Date:** 2026-03-18  
**Status:** In Progress  
**Total MCP Packages:** 1,167

---

## Phase 1: Scan All @iflow-mcp/ Packages (ATTACKER SCOPE)

**Packages:** 22  
**Priority:** CRITICAL  
**Status:** Ready to scan

### Package List

1. @iflow-mcp/blake365-options-chain
2. @iflow-mcp/cameroncooke_xcodebuildmcp
3. @iflow-mcp/cursor-mcp
4. @iflow-mcp/deployto-dev-namecheap-domains-mcp
5. @iflow-mcp/figma-mcp
6. @iflow-mcp/garethcott-enhanced-postgres-mcp-server
7. @iflow-mcp/garethcott_enhanced-postgres-mcp-server
8. @iflow-mcp/hemanth-mcp-ui-server
9. @iflow-mcp/jageenshukla-hello-world-mcp-server
10. @iflow-mcp/mailgun-mcp-server
11. @iflow-mcp/matthewdailey-mcp-starter
12. @iflow-mcp/matthewdailey-rime-mcp
13. @iflow-mcp/mcp-starter
14. @iflow-mcp/minecraft-mcp-server
15. @iflow-mcp/modelcontextprotocol-servers-whois-mcp
16. @iflow-mcp/openai-gpt-image-mcp
17. @iflow-mcp/playwright-mcp
18. @iflow-mcp/puppeteer-mcp-server
19. @iflow-mcp/ref-tools-mcp
20. @iflow-mcp/strato-space-media-gen-mcp
21. @iflow-mcp/tsai1030-ziwei-mcp-server
22. @iflow-mcp/wizd-airylark-mcp-server

**Hypothesis:** ALL are malicious (fork-and-publish pattern)

---

## Phase 2: Scan Other High-Value MCP Packages

**Exclude:** @iflow-mcp/, @aifabrix/ (already scanned)  
**Priority:** HIGH  
**Target:** Popular packages, official SDKs

### Priority Targets

- @modelcontextprotocol/sdk (official SDK)
- @modelcontextprotocol/* (all official packages)
- @anthropic-ai/mcpb (Anthropic)
- @azure/mcp (Microsoft)
- @cloudflare/mcp-server-cloudflare
- Popular community packages

---

## Phase 3: Scan VS Code Extensions

**Search queries:**
- vscode-extension
- cursor-extension
- vscode
- openvsx

**Estimated:** 50,000+ packages  
**Strategy:** Sample popular ones, check recent updates

---

## Evidence Preservation

**Location:** `harness/data/evidence/mcp-scan/`

For each flagged package:
1. Download tarball
2. Extract and scan
3. Backup to evidence folder
4. Document findings

---

## Configuration Notes

**Hardcoded values to parameterize:**
- Download threshold (currently 1000)
- Days back (currently 365)
- Max packages per batch (currently 100)
- Severity threshold (currently info)

**Future config file:** `scan-config.json`
```json
{
  "download_threshold": 1000,
  "days_back": 365,
  "max_packages": 100,
  "severity_threshold": "info",
  "exclude_scopes": ["@iflow-mcp", "@aifabrix"],
  "target_scopes": ["@modelcontextprotocol", "@anthropic-ai"],
  "evidence_dir": "harness/data/evidence"
}
```

---

## QA Notes

**What to watch for:**
1. False positives (document patterns)
2. Detection gaps (what we miss)
3. Performance bottlenecks
4. Edge cases (large packages, .node files)
5. LLM analysis effectiveness

---

**Status:** Ready to execute Phase 1
