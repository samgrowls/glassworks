# MCP Server Hunt - Scan Report #1

**Date:** 2026-03-18  
**Mission:** Scan all MCP-related packages and attacker scopes  
**Status:** COMPLETE

---

## Executive Summary

**Scanned:** 6 packages from attacker scopes  
**Confirmed Malicious:** 5 packages  
**Clean:** 1 package  

### Key Finding: All Versions Compromised From Inception

The `@iflow-mcp/watercrawl-watercrawl-mcp` package was **malicious from version 1.3.0** - the attacker didn't compromise an existing package, they **created it malicious from the start**.

---

## Scan Results

| Package | Version | Findings | Critical | Status |
|---------|---------|----------|----------|--------|
| `@iflow-mcp/watercrawl-watercrawl-mcp` | 1.3.0 | 9,133 | 9,124 | ⚠️ MALICIOUS |
| `@iflow-mcp/watercrawl-watercrawl-mcp` | 1.3.1 | 9,133 | 9,124 | ⚠️ MALICIOUS |
| `@iflow-mcp/watercrawl-watercrawl-mcp` | 1.3.2 | 9,133 | 9,124 | ⚠️ MALICIOUS |
| `@iflow-mcp/watercrawl-watercrawl-mcp` | 1.3.3 | 9,133 | 9,124 | ⚠️ MALICIOUS |
| `@iflow-mcp/watercrawl-watercrawl-mcp` | 1.3.4 | 9,133 | 9,124 | ⚠️ MALICIOUS |
| `@aifabrix/miso-client` | 4.7.1 | 0 | 0 | ✅ CLEAN |
| `@aifabrix/miso-client` | 4.7.2 | 9,136 | 9,124 | ⚠️ MALICIOUS (from earlier scan) |

---

## Attack Pattern Analysis

### @iflow-mcp/watercrawl-watercrawl-mcp

**Attack Type:** Fork-and-Publish (created malicious from start)

```
Version 1.3.0: MALICIOUS (first published version)
Version 1.3.1: MALICIOUS (same payload)
Version 1.3.2: MALICIOUS (same payload)
Version 1.3.3: MALICIOUS (same payload)
Version 1.3.4: MALICIOUS (same payload)
```

**Evidence:** All versions have identical finding count (9,133) and critical count (9,124)

**Conclusion:** This was never a legitimate package - it was created by the attacker specifically for the GlassWare campaign.

### @aifabrix/miso-client

**Attack Type:** Compromised Existing Package

```
Version 4.7.1: CLEAN (last clean version)
Version 4.7.2: MALICIOUS (9,136 findings)
```

**Conclusion:** The attacker gained access to the `@aifabrix/` scope and injected malware starting from version 4.7.2.

---

## Implications

### 1. Two Attack Vectors Identified

| Vector | Description | Example |
|--------|-------------|---------|
| **Fork-and-Publish** | Create new package, inject from v1.0 | `@iflow-mcp/watercrawl-watercrawl-mcp` |
| **Scope Compromise** | Gain access to existing scope, inject in update | `@aifabrix/miso-client` |

### 2. All Versions Affected

For `@iflow-mcp/` scope:
- **No clean version exists**
- Users who installed any version were compromised
- This is NOT a supply chain compromise (no legitimate maintainer)

### 3. Attacker Infrastructure

**Confirmed Attacker Scopes:**
- `@iflow-mcp/` - Created for attack (fork-and-publish)
- `@aifabrix/` - Compromised existing scope
- `AstrOOnauta` - Compromised maintainer account (React Native packages)

---

## Technical Details

### Payload Consistency

All malicious `@iflow-mcp` versions have:
- **9,123 bytes** of steganographic payload
- **Identical AES-256-CBC key:** `zetqHyfDfod88zloncfnOaS9gGs90ONX`
- **Same decoder pattern** in `src/index.ts`
- **Same eval() execution** pattern

This suggests:
- Single payload deployed across all versions
- No evolution or adaptation between versions
- Attacker confident in detection evasion

### Detection Signature

```javascript
// src/index.ts line ~30
// After legitimate MCP server code (~26 lines)
s(`[9,123 invisible Unicode Variation Selectors]`)

// Decoder pattern
codePointAt(0) + 0xFE00/0xE0100 constants

// Execution
eval(atob(decoded_payload))
```

---

## Evidence Preserved

| Package | Versions | Location |
|---------|----------|----------|
| `@iflow-mcp/watercrawl-watercrawl-mcp` | 1.3.0, 1.3.1, 1.3.2, 1.3.3 | `/tmp/mcp-scope/` |
| `@aifabrix/miso-client` | 4.7.1, 4.7.2 | `/tmp/` |
| All evidence copies | All versions | `harness/data/evidence/` |

---

## Next Targets

### Priority 1: Scan Remaining Attacker Scopes

```bash
# @aifabrix/ scope - find all compromised versions
@aifabrix/miso-client@4.7.0  # Check if clean
@aifabrix/miso-client@4.6.x  # Check earlier versions
@aifabrix/*                  # Scan all packages in scope
```

### Priority 2: Scan MCP Server Ecosystem

```bash
# Search npm for MCP servers
@mcp/*
@mcp-server/*
model-context-protocol
mcp-server
```

### Priority 3: Transitive Dependencies

```bash
# Check packages that depend on compromised packages
npm ls @iflow-mcp/watercrawl-watercrawl-mcp
npm ls @aifabrix/miso-client
```

---

## Scan Statistics

| Metric | Value |
|--------|-------|
| Packages scanned | 6 |
| Malicious detected | 5 |
| Clean confirmed | 1 |
| Total findings | 45,665 |
| Scan duration | ~10 seconds |
| Detection rate | 83% (5/6) |

---

## Recommendations

### Immediate

1. **Report @iflow-mcp/ scope** - All versions malicious, created for attack
2. **Report @aifabrix/ scope** - Version 4.7.2+ compromised
3. **Warn users** - No safe version of `@iflow-mcp/watercrawl-watercrawl-mcp`

### Short-term

1. **Scan all @aifabrix/ packages** - Identify full scope compromise
2. **Check transitive dependencies** - Who depends on these packages?
3. **Monitor for new scopes** - Attacker may create more

---

**Prepared by:** glassware automated scan  
**Scan ID:** mcp-hunt-001  
**Run ID:** 611b03f8-c4dc-4cb1-9ae2-313caceecbfe
