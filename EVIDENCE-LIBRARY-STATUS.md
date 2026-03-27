# Evidence Library Status

**Date:** 2026-03-27
**Status:** ⚠️ NEEDS CURATION

---

## Current State

### Available Evidence (20 tarballs)

**Real Attack (1):**
- ✅ `iflow-mcp-watercrawl-watercrawl-mcp-1.3.4.tgz` - **DETECTED** (7.00, malicious)

**Synthetic Combos (4):**
- ❌ `glassworm-combo-001.tgz` - Score 3.50 (NOT malicious)
- ❌ `glassworm-combo-002.tgz` - Score 3.50 (NOT malicious)
- ❌ `glassworm-combo-003.tgz` - Score 3.50 (NOT malicious)
- ❌ `glassworm-combo-004.tgz` - Score 3.50 (NOT malicious)

**Other Synthetics (15):**
- `glassworm-steg-001.tgz` through `004.tgz` - Untested
- `glassworm-c2-001.tgz` through `004.tgz` - Untested
- `glassworm-evasion-001.tgz` through `003.tgz` - Untested
- `glassworm-exfil-001.tgz` through `004.tgz` - Untested

---

## Problem

The 3 synthetic combo packages (combo-002, 003, 004) that were documented as "good" in the README are **NOT scoring above 7.0 threshold**. They appear to be missing critical attack patterns (likely invisible characters).

**Root Cause:** These synthetics were created during earlier development but were later deemed "not very good" and archived. The README documentation was not updated to reflect this.

---

## Required Action

### Option 1: User Provides Correct Evidence (RECOMMENDED)

User will fetch:
- 2 real GlassWorm malicious packages (confirmed attacks)
- 2 validated synthetic packages (properly constructed)

**Total:** 4 curated evidence packages for validation

### Option 2: Fix Existing Synthetics

Examine and repair the combo-002/003/004 packages to include:
- Invisible Unicode characters (ZWSP, ZWNJ, variation selectors)
- Decoder patterns
- C2 or evasion patterns

**Risk:** May not accurately represent real GlassWorm attacks

---

## Interim Validation

For now, we have **1 confirmed working evidence package**:
- `iflow-mcp-watercrawl-watercrawl-mcp-1.3.4.tgz` ✅

This is sufficient to verify detection is working, but not comprehensive enough for full validation.

---

## Next Steps

1. ⏳ **User provides 2 real + 2 synthetic evidence packages**
2. ⏳ **Package as tarballs in `evidence/` directory**
3. ⏳ **Update README with correct evidence list**
4. ⏳ **Re-run validation with complete evidence set**

---

**Last Updated:** 2026-03-27
**Action Required:** User to provide 4 curated evidence packages
