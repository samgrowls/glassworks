# Deep Analysis: @aikidosec/mcp

**Date:** 2026-03-18  
**Status:** LIKELY FALSE POSITIVE (needs confirmation)

---

## Package Info

| Field | Value |
|-------|-------|
| Name | `@aikidosec/mcp` |
| Version | 1.0.3 |
| Author | Aikido Security |
| Description | Aikido MCP server |
| Size | 876KB (minified) |
| Lines | 139 (bundled) |

---

## Detection Summary

**Total Findings:** 22 (11 critical)

### Breakdown by Category

| Category | Count | Severity |
|----------|-------|----------|
| glassware_pattern | 11 | Critical |
| encrypted_payload | 4 | High-Medium |
| header_c2 | 1 | Critical |
| rc4_pattern | 1 | Info |
| homoglyph | 5 | Medium |

---

## Analysis

### Why This is Likely FALSE POSITIVE

1. **Author is Aikido Security** - They reported GlassWare, would they infect their own package?

2. **Minified/Bundled Code** - 876KB in 139 lines means heavy bundling. Common patterns trigger detectors:
   - Base64 encoding/decoding (file handling)
   - JWT crypto operations (authentication)
   - eval-like patterns (module loading)

3. **Legitimate Functionality**:
   - JWT verification (`jose` dependency)
   - File scanning in temp directories
   - API communication with aikido.dev

4. **No Steganographic Payload** - Unlike watercrawl-mcp (9,000+ invisible chars), this has encoded strings but not Unicode steganography

### Why We Should Verify

1. **header_c2 Detection** - HTTP header extraction + decrypt + exec
2. **RC4 Pattern** - Hand-rolled cipher detected
3. **High Finding Count** - 22 findings is significant

---

## Possible Explanations

### Scenario 1: False Positive (Most Likely)

Bundled dependencies + legitimate crypto = pattern matches

**Evidence:**
- Uses `jose` for JWT (standard auth)
- Uses `decompress` for file handling
- Minified code has high-entropy strings

### Scenario 2: Supply Chain Compromise

Aikido's build process was compromised

**Evidence:**
- None directly
- Would explain patterns

### Scenario 3: Research/Testing

Aikido testing their own detectors

**Evidence:**
- They're security researchers
- Could be intentional test package

---

## Recommendation

**Contact Aikido Security directly:**
- Email: security@aikido.dev (or similar)
- Ask: "Is @aikidosec/mcp your official package?"
- Share: Detection findings
- Request: Verification

**DO NOT publicly accuse** - could be:
- Their own testing
- Legitimate code with FP patterns
- Embarrassing if we're wrong

---

## Files of Interest

| File | Purpose |
|------|---------|
| `dist/index.js` | Bundled server code |
| `package.json` | Dependencies: jose, decompress, pino |

---

**Analyst:** glassware autonomous scan  
**Confidence:** 70% False Positive  
**Action Required:** External verification from Aikido
