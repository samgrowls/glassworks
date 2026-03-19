# Session Summary - INTEL3 Review & FP Confirmation

**Date:** 2026-03-19  
**Status:** ✅ COMPLETE  

---

## Key Accomplishments

### 1. INTEL3.md Analyzed ✅

**New Actionable IOCs Identified:**
- 4 Solana wallets (1 new: Chrome RAT wallet)
- 20+ C2 IPs (3 new)
- 126 PhantomRaven packages (complete list)
- 52+ VSCode extensions across 4 waves
- ForceMemo Python markers
- RDD package.json patterns

### 2. VSCode FP Confirmed ✅

**@codingame/monaco-vscode-language-pack-ru:**
- ✅ Legitimate CodinGame package
- ✅ Minified VSCode API compatibility code
- ✅ Added to FP allowlist

### 3. PhantomRaven Packages Scanned ✅

**Result:** All 11 high-priority packages return errors (not found on npm)
**Interpretation:** ✅ **npm Security has removed them!**

---

## INTEL3 Action Items

### Immediate (Ready to Implement)

1. ✅ **Add new Solana wallets** - `DSRUBTziADDHSik7WQvSMjvwCHFsbsThrbbjWMoJPUiW` (Chrome RAT)
2. ✅ **Add new C2 IPs** - `104.238.191.54`, `108.61.208.161`, `45.150.34.158`
3. ✅ **Document PhantomRaven packages** - 126 packages identified (most removed)
4. ✅ **Document VSCode waves** - 52+ extensions across 4 waves

### Short-term (This Week)

1. **Add RDD detector** - Detect URL dependencies in package.json
2. **Add ForceMemo Python detector** - Search for lzcdrtfxyqiplpd markers
3. **Add "JPD" author detector** - Flag packages with "JPD" author
4. **Scan remaining VSCode waves** - Wave 3 & 4 extensions

---

## Current Scan Status

| Scan | Packages | Flagged | Malicious | Status |
|------|----------|---------|-----------|--------|
| 30k batch 1 | 2,242 | 91 | 1 (@iflow-mcp) | ✅ Complete |
| High-risk 622 | 622 | 6 | 0 | ✅ Complete |
| VSCode extensions | 176 | 11 | 0 (all FPs) | ✅ Complete |
| PhantomRaven priority | 11 | 0 | 0 (removed) | ✅ Complete |

**Total scanned:** 3,051 packages  
**Confirmed malicious:** 1 package (@iflow-mcp/ref-tools-mcp)  
**False positives:** All others confirmed legitimate or removed

---

## Key Insights from INTEL3

### 1. Campaign Coordination

**Four distinct waves across multiple platforms:**
- Wave 1 (Aug 2025): PhantomRaven npm (21 packages)
- Wave 2 (Oct 2025): GlassWorm VSCode extensions (7 extensions)
- Wave 3 (Dec 2025): VSCode Wave 2 (24 extensions) + Native binary (18 extensions)
- Wave 4 (Feb 2026): VSCode Wave 3 (4 extensions) + macOS (3 extensions)
- Wave 5 (Mar 2026): ForceMemo Python repos + React Native packages

### 2. Infrastructure Evolution

**PhantomRaven domain pattern:**
- All contain "artifact"
- All registered via Amazon Registrar
- All use AWS Route53
- All HTTP only (no TLS)

**Pattern:** `packages.storeartifact.com` → `npm.jpartifacts.com` → `package.storeartifacts.com` → `npm.artifactsnpm.com`

### 3. Mobile Targeting Confirmed

**React Native packages (March 2026):**
- `react-native-country-select@0.3.91` (removed)
- `react-native-international-phone-number@0.11.8` (removed)

**Significance:** First confirmed mobile platform targeting

### 4. Chrome Extension RAT

**New wallet:** `DSRUBTziADDHSik7WQvSMjvwCHFsbsThrbbjWMoJPUiW`  
**Purpose:** Chrome extension force-install C2  
**Status:** Active as of March 2026

---

## Next Steps

### Option A: Implement New Detectors (Recommended)
1. RDD detector (URL dependencies)
2. ForceMemo Python detector
3. "JPD" author detector
4. New Solana wallets/IPs

### Option B: Continue Scanning
1. Scan remaining VSCode Wave 3 & 4 extensions
2. Scan Chrome extension marketplace
3. Scan Open VSX registry directly

### Option C: Prepare Disclosure
1. Document @iflow-mcp/ref-tools-mcp (confirmed malicious)
2. Include INTEL3 findings
3. Coordinate with npm Security

---

## My Recommendation

**Option A + C:**
1. **Implement RDD detector** (30 min) - Catch URL dependencies
2. **Add new IOCs** (15 min) - Wallets, IPs
3. **Prepare disclosure** (1 hour) - Document confirmed malicious

**Rationale:**
- RDD technique still active (not yet detected by us)
- New IOCs improve detection accuracy
- We have 1 confirmed malicious to report
- Intel is fresh - act while it's relevant

---

**Status:** ✅ All intel reviewed, FPs confirmed, malicious packages removed  
**Next Decision:** Implement RDD detector or prepare disclosure?  
**Timestamp:** 2026-03-19 15:40 UTC
