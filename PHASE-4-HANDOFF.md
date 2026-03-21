# Phase 4 Handoff — Remaining Detectors + Polish

**Date:** 2026-03-21
**Version:** v0.10.0
**Author:** Phase 2 implementer
**For:** Phase 4 implementer (G10, E4, E5 + polish)

---

## What's Done

### Phase 2 ✅ (Binary Scanning)

All binary detectors implemented and tested:

| ID | Detector | Status | Notes |
|----|----------|--------|-------|
| G6 | XorShift128 | ✅ Complete | Heuristic-based (entropy + constants + patterns) |
| G7 | IElevator CLSID | ✅ Complete | Exact CLSIDs from PART4.md |
| G8 | APC Injection | ✅ Complete | Import table analysis |
| G9 | memexec | ✅ Complete | Typo fingerprint (LoadLibararyFail) |
| G11 | .node Metadata | ✅ Complete | PDB paths, build attribution |

**Test Status:** 400 passed, 1 failed (pre-existing), 2 ignored

**Key Files:**
- `glassware-core/src/binary/` — All binary module code
- `glassware-core/src/binary/extractor.rs` — Goblin-based parser
- `glassware-core/src/binary/{xorshift,ielevator,apc_injection,memexec,metadata}.rs` — Detectors

---

## What's Remaining (Phase 4 Scope)

### G10: Solana Memo Parser

**What:** Parse Solana transaction memos for C2 communication patterns.

**Intel Source:** PART5.md section on Chrome RAT blockchain C2

**Approach:**
- Use Solana RPC or public explorers to fetch transactions
- Parse memo instructions for encoded commands
- Look for GlassWorm-specific memo patterns

**Estimated Effort:** 2-3 days

---

### E4: PhantomRaven Campaign Matching

**What:** Match packages against known PhantomRaven infrastructure.

**Intel Source:** PhantomRaven campaign intelligence (separate from GlassWorm)

**Key Indicators:**
- RDD (Remote Dynamic Dependencies) — URL-based npm dependencies
- JPD author signature — "JPD" author field in package.json
- Infrastructure reuse — same domains, IPs, GitHub accounts

**Implementation:**
- Extend existing RDD and JPD detectors with campaign correlation
- Add infrastructure matching (domain/IP lookups)
- Cluster packages by shared infrastructure

**Estimated Effort:** 2-3 days

---

### E5: YARA Rule Export

**What:** Export detection patterns as YARA rules for integration with other tools.

**Approach:**
- Add `export_yara()` method to each detector
- Generate per-detector rules (not single monolithic rule)
- Include metadata: severity, MITRE ATT&CK, references

**Example Output:**
```yara
rule GlassWorm_XorShift128 {
    meta:
        severity = "high"
        reference = "https://codeberg.org/tip-o-deincognito/glassworm-writeup"
    strings:
        $xor_const = "0x6c62272e"
        $memexec_typo = "LoadLibararyFail"
    condition:
        any of them
}
```

**Estimated Effort:** 1-2 days

---

## Known Issues / Technical Debt

### Pre-existing Test Failures (NOT your responsibility)

1. **`adversarial::strategies::variable::tests::test_variable_renaming_decoder`**
   - File: `glassware-core/src/adversarial/strategies/variable.rs:79`
   - Status: Pre-existing, not caused by Phase 2

### Documentation Updates Needed

1. **`CURRENT-STATUS-AND-NEXT-STEPS.md`** — Still stale, should be updated or deprecated
2. **`README.md`** — Add binary scanning usage examples
3. **`docs/WORKFLOW-GUIDE.md`** — Add .node scanning workflow

### Potential Improvements (NOT required for Phase 4)

1. **Binary detector performance** — Currently parses entire binary on every scan. Consider caching parsed `BinaryFeatures`.
2. **Mach-O support** — Tested minimally. May need refinement for real-world Mach-O binaries.
3. **ELF import/export detection** — Simplified implementation. Could be more comprehensive.

---

## Architecture Notes (from Phase 2 implementer)

### Binary Scanner Design Decisions

1. **Feature-gated (`binary` feature)** — Avoids goblin dependency for users who don't need binary scanning
2. **String extraction first** — Works on all formats, provides most value for GlassWorm detection
3. **Heuristic-based G6** — Deliberately avoided hardcoded byte patterns (they looked invented)
4. **Exact CLSIDs for G7** — Extracted from PART4.md, not guessed
5. **Additive scoring** — Followed G5 pattern (not multiplicative stacking)

### Goblin 0.7 API Quirks

- `pe.imports` returns `Vec<Import>` with `name`, `dll` fields
- `pe.debug_data.codeview_pdb70_debug_info.filename` is `&[u8]`, not `&str`
- `elf.dynstrtab.get_at()` returns `Option<&str>`, not `Result`
- `macho.imports()` and `macho.exports()` return `Result<Vec<_>>`
- Mach-O segments: `segment.name()` returns `Result<&str>`

### Test Patterns

- Each detector has 6-7 tests covering: positive match, negative match, edge cases
- Binary tests use mock `BinaryFeatures` (no real PE/ELF files needed)
- Format detection tests use magic byte headers

---

## Getting Started

```bash
# Verify environment
cargo test -p glassware-core --lib --features "full,binary"

# Should see: 400 passed, 1 failed (pre-existing), 2 ignored

# Run binary detector tests specifically
cargo test -p glassware-core --lib binary:: --features "full,binary"
```

---

## Success Criteria for Phase 4

- [ ] G10: Solana memo parser implemented and tested
- [ ] E4: PhantomRaven campaign matching (RDD + JPD + infrastructure)
- [ ] E5: YARA rule export working
- [ ] Documentation updated (README, WORKFLOW-GUIDE)
- [ ] No NEW test failures introduced
- [ ] Tagged as v0.11.0

---

## Contact

- **Primary:** Via GitHub issues/PRs on samgrowls/glassworks
- **Intel Sources:** https://codeberg.org/tip-o-deincognito/glassworm-writeup

---

**Good luck! The binary scanning foundation is solid, and the architecture is ready for Phase 4.** 🎯
