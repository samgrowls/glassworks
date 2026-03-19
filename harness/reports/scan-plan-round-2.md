# glassware Targeted Scan Plan - Round 2

**Date:** 2026-03-18  
**Intelligence Source:** https://codeberg.org/tip-o-deincognito/glassworm-writeup  
**Hypothesis:** GlassWare infections are the "tip of an iceberg" - many repos infected but dormant

---

## Intelligence Summary

### Attack Pattern (Confirmed)
1. **Injection:** Stego-encoded payload in `preinstall.js` or similar
2. **Decoder Signature:** `codePointAt` + `0xFE00`/`0xE0100` constants
3. **Execution:** Decrypted blob → AES-256-CBC → eval()
4. **Target:** VS Code/Cursor extensions, devtools with install scripts

### Known IOCs
| Pattern | Description | Our Detection |
|---------|-------------|---------------|
| `codePointAt` + `0xFE00` | Stego decoder | ✅ GW002 (decoder_function) |
| Variation Selectors U+E0100-U+E01EF | Payload encoding | ✅ GW001 (stegano_payload) |
| `Buffer.from` + decode | Payload extraction | ✅ GW005 (encrypted_payload) |
| AES-256-CBC decrypt | Stage 1 loader | ✅ GW006 (hardcoded_key) |
| Solana wallet C2 | Decentralized C2 | ⚠️ Not yet detected |
| Russian locale exclusion | Geo-fencing | ⚠️ Not yet detected |

### Researcher Findings
- **28 infected repos** found via GitHub search (Mar 14)
- **Peak: 11 repos in 26 hours**
- **Injections continued after kill switch** (operator had credentials)
- **5 months continuous operation** (Oct 2025 - Mar 2026)

---

## Scan Strategy

### Phase 1: High-Value Targets (Recommended First)

**Target:** VS Code / Cursor extensions with install scripts

```bash
# Search npm for VS Code extensions
cd harness
.venv/bin/python scan.py \
  --max-packages 100 \
  --days-back 730 \
  --download-threshold 50000 \
  --tier 1
```

**Keywords to search:**
- `vscode-extension`
- `cursor-extension`
- `vscode`
- `cursor`
- `devtools`
- `productivity`

### Phase 2: Pattern-Based Hunt

**Target:** Any package with decoder signature

```bash
# Manual GitHub search (like researcher did)
# Then download and scan suspicious packages

# Alternative: scan our fixtures for known patterns first
./target/release/glassware --format json \
  glassware-core/tests/fixtures/glassworm/
```

### Phase 3: Expanded Criteria

**Modify selector.py to remove install script requirement:**

```python
# Current: requires has_install_scripts
# Proposed: scan packages with:
# - Keywords in description
# - Recently published (<90 days)
# - Low downloads (<5000/wk)
# - Any suspicious patterns in package.json
```

---

## Detection Gaps to Address

| Gap | Priority | Notes |
|-----|----------|-------|
| Solana wallet C2 | Medium | Could add regex for wallet patterns |
| Russian locale check | Low | Geo-fencing, not malicious itself |
| Socket.IO C2 pattern | Medium | `_partner` token, `check_version` |
| Chrome extension patterns | High | Sideloader detection needed |
| Native binary (.node) analysis | High | Requires PE/Mach-O parsing |

---

## Recommended Next Steps

### Immediate (Today)
1. ✅ **Scan VS Code extensions** (50-100 packages)
2. ✅ **Run with --llm** on flagged packages
3. ✅ **Document any findings** for disclosure

### Short-term (This Week)
1. **Add Solana wallet detector** (regex for known wallets)
2. **Add Chrome extension detector** (manifest.json analysis)
3. **Partner with researcher** (share findings via Codeberg)

### Long-term
1. **Native binary analysis** (.node file scanning)
2. **Cross-file taint tracking** (multi-file flows)
3. **Continuous monitoring** (new packages daily)

---

## Evidence Preservation

All findings should be:
1. **Archived to vault** (harness already does this)
2. **Documented with timestamps** (for disclosure)
3. **Reported responsibly** (npm Security first)

---

## Success Metrics

| Metric | Target | Notes |
|--------|--------|-------|
| Packages scanned | 500 | By end of week |
| True positives | >0 | Even 1 find validates approach |
| False positives | <5% | Maintain quality |
| Time per package | <5s | Without LLM |
| LLM accuracy | >90% | On flagged files |

---

**Prepared by:** glassware team  
**Based on intel from:** tip-o-deincognito (Codeberg)
