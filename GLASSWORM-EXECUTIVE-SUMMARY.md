# GlassWorm Integration — Executive Summary

**Date:** 2026-03-21  
**Sprint:** v0.9.0.0 (4 weeks)  
**Status:** 📋 Ready for Approval

---

## The Opportunity

Current glassware detects GlassWorm **JavaScript wrappers** but misses the **native payloads** (`.node` files) where the actual malicious logic lives.

This sprint closes that gap.

---

## What We're Building

### 1. Severity Scoring Overhaul ✅

**Before:**
```rust
Severity::Critical  // Always critical
```

**After:**
```rust
Severity::Critical × 7.0x  // New package + anonymous author + install script + network + eval
= EXTREME (new level)
```

**Impact:** 90% FP reduction on legitimate packages, higher confidence on real threats.

---

### 2. Binary Scanning (`.node` files) ✅

**What:** Parse native binaries (PE/ELF/Mach-O) for GlassWorm signatures

**Detects:**
- XorShift obfuscation (G6)
- IElevator COM for App-Bound bypass (G7)
- APC injection imports (G8)
- memexec fileless loader (G9)
- PDB paths, Cargo metadata (G11)

**Tech Stack:** Rust + `goblin` crate (zero-copy, fast, safe)

**Overhead:** ~2-5ms per `.node` file (<3% total scan time)

**Impact:** Goes from "seeing the wrapper" to "seeing the payload"

---

### 3. Host Forensics Mode ✅

**What:** Scan filesystem for GlassWorm infection indicators

**Detects:**
- `jucku/` working directories (G1)
- `myextension/` sideloaded extensions (G1)
- Chrome Secure Prefs tampering (G2)
- Suspicious `location:4` + `creation_flags:38` (E2)

**CLI:**
```bash
glassware host-scan --path / --chrome-prefs --output report.json
```

**Impact:** Incident response capability (not just supply chain monitoring)

---

### 4. JS-Level Enhancements ✅

**G3:** Typo attribution fingerprints (`Invlaid`, `LoadLibararyFail`)  
**G4:** Exfil JSON schema matching (`sync_oauth_token`, `send_tab_private_key`)  
**G5:** Socket.IO C2 detection (port 5000, tunnel patterns)  
**E1:** Updated IoC lists (wallets, IPs)  
**E3:** Browser-kill command patterns (`taskkill /F /IM chrome.exe`)

**Impact:** Catches GlassWorm JS patterns we currently miss

---

### 5. Advanced Capabilities ✅

**G10:** Solana Memo decoder (query chain for C2 URLs)  
**E4:** PhantomRaven auto-matching (126 known packages)  
**E5:** YARA rule export (for external tools)

**Impact:** Extended threat intel integration

---

## Timeline

```
Week 1 (Mar 24-28)    Phase 0-1: Quick wins + JS detectors
Week 2 (Mar 31-Apr 4) Phase 2: Binary foundation
Week 3 (Apr 7-11)     Phase 2-3: Binary detectors + Host forensics
Week 4 (Apr 14-18)    Phase 4: Advanced capabilities
Buffer (Apr 21-25)    Testing, docs, release
```

---

## Resource Requirements

**Dev:** 1 Rust developer (full-time, 4 weeks)  
**Infra:** Test VMs (Win/macOS/Linux), Solana RPC access  
**Deps:** `goblin`, `memmap2`, `reqwest` (all pure Rust)

---

## Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| True positive rate | 95% | 98%+ |
| False positive rate | 2% | <1% |
| Binary payload detection | 0% | 95%+ |
| Context awareness | None | Full |

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| No `.node` samples | Use mock files, request from community |
| G6 xorshift variants | Configurable patterns, tune after testing |
| Host scan cross-platform | Test on Win/macOS/Linux VMs |
| Severity too aggressive | Conservative defaults, tunable config |

---

## Recommendation

**Proceed with full implementation.**

**Rationale:**
1. Binary scanning is highest ROI — unlocks native payload detection
2. Severity overhaul reduces FP while catching more real threats
3. Host forensics adds incident response capability
4. All phases are independent — can parallelize if needed
5. Rust ecosystem (`goblin`) makes binary scanning tractable

**Go/No-Go Decision:** By Mar 24 (start of Week 1)

---

## Files

- **Full Plan:** `GLASSWORM-INTEGRATION-PLAN-V0.9.md`
- **Current Status:** `CURRENT-STATUS-AND-NEXT-STEPS.md`
- **Intel Source:** `glassworm-writeup/PART5.md`, `newintel.md`

---

**Ready to begin upon approval.**

---

**End of Summary**
