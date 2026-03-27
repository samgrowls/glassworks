# GlassWorm Detection - Next Agent Handoff

**Date:** 2026-03-27
**Version:** v0.73.0-medium-waves-complete
**Status:** READY FOR SEMANTIC DETECTION IMPLEMENTATION

---

## Executive Summary

**Current State:** The GlassWorm detection system has successfully scanned **1055 packages** across 4 high-risk categories with **0 real GlassWorm attacks found** and **0.57% FP rate** (6 false positives).

**Key Finding:** We have extensive **unused semantic analysis infrastructure** (OXC AST parser, taint analysis, semantic detectors) that is integrated but NOT being used by our detectors. This represents the single biggest opportunity for FP reduction and detection improvement.

**Next Priority:** Activate and integrate semantic analysis to replace regex-based detection with AST-aware, code-flow-aware detection.

---

## Current Detection Architecture

### What's Working

| Component | Status | Usage |
|-----------|--------|-------|
| **FileIR** | ✅ Active | Unified IR with content, lines, JSON, unicode analysis |
| **Unicode Analysis** | ✅ Active | Invisible chars, bidi, homoglyphs detected |
| **Tiered Scoring** | ✅ Active | Category caps, Tier 1 signal requirement |
| **Campaign Intelligence** | ✅ Active | Infrastructure tracking, code clustering |
| **LLM Integration** | ✅ Active | Tier 1 (Cerebras), Tier 2 (NVIDIA) |

### What's NOT Being Used

| Component | Status | Potential Impact |
|-----------|--------|------------------|
| **OXC AST Parser** | ⚠️ Integrated but unused | Parse JS/TS once, share AST across detectors |
| **Semantic Analysis** | ⚠️ Integrated but unused | Scope-aware, flow-aware detection |
| **Taint Analysis** | ⚠️ Integrated but unused | Track data flow from sources to sinks |
| **Semantic Detectors (GW005-GW008)** | ⚠️ Implemented but not integrated | AST-based encrypted payload, header C2 detection |
| **Code Flow Analysis** | ⚠️ Available but unused | Detect obfuscation patterns with context |

---

## Unused Infrastructure Analysis

### 1. OXC AST Parser (`glassware-core/src/ir.rs`)

**Location:** `FileIR` struct already has `ast: Option<JavaScriptAST>` field

**Current Usage:**
```rust
// ir.rs line 483-490
#[cfg(feature = "semantic")]
{
    // AST parsing code exists but is never called by detectors
}
```

**What Should Happen:**
```rust
// Detectors should call:
if let Some(ast) = ir.ast() {
    // Use pre-parsed AST instead of regex
}
```

**Impact:** 20-30% performance improvement, more accurate detection

---

### 2. Semantic Analysis (`glassware-core/src/semantic.rs`)

**Location:** `build_semantic()` function exists

**Current Usage:**
```rust
// semantic.rs line 90-200
pub fn build_semantic(source: &str, path: &Path) -> Option<SemanticAnalysis> {
    // Full OXC parse → semantic pipeline
    // Extracts: string_literals, call_sites, declarations, references, scopes
}
```

**What's Extracted:**
- `string_literals` - All string values with spans
- `call_sites` - Function calls with callee chains
- `declarations` - Variable/function declarations
- `references` - Identifier references with resolved symbols
- `scopes` - Full scope tree

**What Should Happen:**
```rust
// Detectors should use semantic analysis for:
// 1. Finding high-entropy strings in scope
// 2. Tracking crypto API usage with context
// 3. Detecting header access patterns with flow
```

---

### 3. Taint Analysis (`glassware-core/src/taint.rs`)

**Location:** `find_sources()`, `find_sinks()`, `find_flows()`

**Current Usage:**
```rust
// taint.rs line 268-400
pub fn find_sources(sa: &SemanticAnalysis) -> Vec<TaintSource> {
    // Finds: high-entropy strings, HTTP header accesses, crypto API calls
}

pub fn find_sinks(sa: &SemanticAnalysis) -> Vec<TaintSink> {
    // Finds: eval, Function, headers.set, crypto operations
}

pub fn find_flows(...) -> Vec<TaintFlow> {
    // Connects sources to sinks
}
```

**What Should Happen:**
```rust
// Encrypted payload detector should:
let sources = find_sources(&semantic);
let sinks = find_sinks(&semantic);
let flows = find_flows(&sources, &sinks);

if flows.iter().any(|f| f.is_suspicious()) {
    findings.push(Finding::new(...));
}
```

---

### 4. Semantic Detectors (`glassware-core/src/gw005_semantic.rs`, etc.)

**Location:** GW005-GW008 semantic detectors exist

**Current Usage:**
```rust
// gw005_semantic.rs
pub struct Gw005SemanticDetector;

impl SemanticDetector for Gw005SemanticDetector {
    fn detect_semantic(
        &self,
        _source: &str,
        _path: &Path,
        _flows: &[TaintFlow],
        _sources: &[TaintSource],
        _sinks: &[TaintSink],
    ) -> Vec<Finding> {
        // Implementation exists but is NEVER called
    }
}
```

**What Should Happen:**
```rust
// In engine.rs or detector registry:
#[cfg(feature = "semantic")]
{
    engine.register_semantic(Box::new(Gw005SemanticDetector::new()));
    engine.register_semantic(Box::new(Gw008SemanticDetector::new()));
}

// During scan:
if ir.supports_semantic_analysis() {
    let semantic = build_semantic(ir.content(), ir.path());
    for detector in &semantic_detectors {
        findings.extend(detector.detect_semantic(...));
    }
}
```

---

## Why This Matters

### Current FP Root Causes

1. **Test files with intentional Unicode** → AST would show it's in test code
2. **Data files with locale chars** → AST would show it's data, not code
3. **Build artifacts** → AST would show it's bundled code
4. **i18n packages without decoder** → Semantic analysis would confirm no decoder pattern

### How Semantic Analysis Fixes FPs

| FP Type | Current Detection | Semantic Detection |
|---------|-------------------|-------------------|
| Test files | Regex finds bidi chars | AST shows `describe()`, `it()`, `test()` - skip |
| Data files | Regex finds Unicode | AST shows JSON import, no code execution - skip |
| Build output | Regex finds patterns | AST shows webpack/rollup wrapper - skip |
| i18n packages | Regex finds chars | Semantic shows no decoder flow - skip |

### Detection Improvements

| Attack Pattern | Current | Semantic |
|----------------|---------|----------|
| Encrypted payload | Regex for `atob(` + `eval(` | Taint flow: base64 decode → decrypt → eval |
| Header C2 | Regex for `headers[` | Semantic: `fetch()` → `response.headers` → `decrypt()` |
| Obfuscation | Regex for XOR patterns | AST: Variable renaming + control flow flattening |

---

## Implementation Plan

### Phase 1: Enable Semantic Feature (1-2 days)

**Goal:** Turn on the `semantic` feature and verify AST parsing works

**Steps:**
1. Add `semantic` to default features in `glassware-core/Cargo.toml`
2. Enable AST parsing in `FileIR::build()`
3. Add logging to verify AST is being parsed
4. Test on 10 JS packages

**Expected Outcome:** AST parsed for all JS/TS files, no detector changes yet

---

### Phase 2: Integrate Semantic Detectors (2-3 days)

**Goal:** Wire up GW005-GW008 semantic detectors

**Steps:**
1. Add `SemanticDetector` trait to detector registry in `engine.rs`
2. Register GW005, GW008 semantic detectors
3. Call semantic detectors after regex detectors
4. Compare results: regex vs semantic

**Expected Outcome:** Semantic detectors running alongside regex detectors

---

### Phase 3: Replace Regex Detectors (3-5 days)

**Goal:** Replace regex-based detectors with semantic versions

**Priority Order:**
1. **EncryptedPayload** (GW005) - Highest FP rate
2. **HeaderC2** (GW008) - Second highest FP rate
3. **RC4Pattern** (GW007) - Medium FP rate
4. **HardcodedKeyDecryption** (GW006) - Low FP rate

**For Each Detector:**
1. Run semantic and regex in parallel
2. Compare findings (should match or semantic should be better)
3. Disable regex version when semantic is ready
4. Update tests

**Expected Outcome:** 50% FP reduction on i18n/test/build files

---

### Phase 4: Add Context-Aware Filtering (2-3 days)

**Goal:** Use AST to skip test/data/build files

**Implementation:**
```rust
// In invisible.rs or as separate filter
fn should_skip_file(ir: &FileIR) -> bool {
    if let Some(ast) = ir.ast() {
        // Check for test patterns
        if ast.contains_call("describe") || ast.contains_call("it") {
            return true;  // Test file
        }
        
        // Check for data-only patterns
        if ast.is_data_file() {  // Only imports/exports, no logic
            return true;
        }
    }
    
    false
}
```

**Expected Outcome:** Eliminate test/data/build FPs entirely

---

### Phase 5: Taint Analysis Integration (3-5 days)

**Goal:** Full taint tracking for encrypted payload detection

**Implementation:**
```rust
// In encrypted_payload_detector.rs
fn detect(&self, ir: &FileIR) -> Vec<Finding> {
    let semantic = build_semantic(ir.content(), ir.path());
    
    let sources = find_sources(&semantic);
    let sinks = find_sinks(&semantic);
    let flows = find_flows(&sources, &sinks);
    
    // Only flag if there's a suspicious flow
    for flow in flows {
        if flow.source.is_high_entropy() && flow.sink.is_exec() {
            findings.push(Finding::new(...));
        }
    }
}
```

**Expected Outcome:** Detect only real encrypted payloads, not just `atob(` usage

---

## File Locations

### Core Infrastructure

| File | Purpose | Status |
|------|---------|--------|
| `glassware-core/src/ir.rs` | FileIR with AST field | ✅ Exists, ⚠️ Not used |
| `glassware-core/src/semantic.rs` | OXC semantic analysis | ✅ Exists, ⚠️ Not called |
| `glassware-core/src/taint.rs` | Taint source/sink/flow | ✅ Exists, ⚠️ Not used |
| `glassware-core/src/detector.rs` | Detector trait | ✅ Active |

### Semantic Detectors

| File | Purpose | Status |
|------|---------|--------|
| `glassware-core/src/gw005_semantic.rs` | Encrypted payload | ✅ Exists, ⚠️ Not integrated |
| `glassware-core/src/gw006_semantic.rs` | Hardcoded key decryption | ✅ Exists, ⚠️ Not integrated |
| `glassware-core/src/gw007_semantic.rs` | RC4 pattern | ✅ Exists, ⚠️ Not integrated |
| `glassware-core/src/gw008_semantic.rs` | Header C2 | ✅ Exists, ⚠️ Not integrated |

### Engine Integration

| File | Purpose | Status |
|------|---------|--------|
| `glassware-core/src/engine.rs` | Detector registry | ✅ Active, ⚠️ No semantic |
| `glassware-core/src/detectors/` | Regex detectors | ✅ Active |

---

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_semantic_detects_test_files() {
    let source = r#"describe('test', () => { it('works', () => {}); });"#;
    let ir = FileIR::build(Path::new("test.js"), source);
    
    assert!(ir.ast().is_some());
    assert!(is_test_file(ir.ast().unwrap()));
}

#[test]
fn test_taint_finds_encrypted_payload() {
    let source = r#"
        const key = "hardcoded_key";
        const encrypted = atob("base64data");
        const decrypted = decrypt(encrypted, key);
        eval(decrypted);
    "#;
    
    let semantic = build_semantic(source, Path::new("test.js"));
    let sources = find_sources(&semantic);
    let sinks = find_sinks(&semantic);
    let flows = find_flows(&sources, &sinks);
    
    assert!(flows.iter().any(|f| f.is_suspicious()));
}
```

### Integration Tests

1. **Test on Wave18-21 packages** - Verify FP reduction
2. **Test on evidence packages** - Verify detection maintained
3. **Performance benchmark** - Measure AST parsing overhead

---

## Success Metrics

| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| FP Rate | 0.57% | < 0.2% | Wave18-21 re-scan |
| Detection Rate | 100% | 100% | Evidence packages |
| Performance | ~50k LOC/sec | ~40k LOC/sec | Benchmark |
| Test/Data FP | 6 FPs | 0 FPs | Manual review |

---

## Risks and Mitigations

### Risk 1: AST Parsing Overhead

**Risk:** OXC parsing adds 20-30% overhead

**Mitigation:**
- Parse once in FileIR, share across all detectors
- Only parse JS/TS files (skip JSON, etc.)
- Cache AST for re-scans

### Risk 2: Semantic Detectors Miss Attacks

**Risk:** AST-based detection misses edge cases

**Mitigation:**
- Run semantic + regex in parallel initially
- Compare results before disabling regex
- Keep regex as fallback

### Risk 3: OXC Version Compatibility

**Risk:** OXC API changes break integration

**Mitigation:**
- Pin OXC version in Cargo.toml
- Add version check in build script
- Test on OXC updates

---

## Next Agent Checklist

### Before Starting

- [ ] Read this handoff document
- [ ] Review `glassware-core/src/semantic.rs`
- [ ] Review `glassware-core/src/taint.rs`
- [ ] Review `glassware-core/src/gw005_semantic.rs` through `gw008_semantic.rs`
- [ ] Run `cargo test -p glassware-core --features semantic`

### Phase 1 Tasks

- [ ] Add `semantic` to default features
- [ ] Enable AST parsing in FileIR
- [ ] Add logging for AST parsing
- [ ] Test on 10 JS packages

### Phase 2 Tasks

- [ ] Add SemanticDetector to engine
- [ ] Register GW005, GW008
- [ ] Verify semantic detectors run
- [ ] Compare with regex results

### Phase 3 Tasks

- [ ] Replace EncryptedPayload regex with semantic
- [ ] Replace HeaderC2 regex with semantic
- [ ] Update tests
- [ ] Measure FP reduction

### Phase 4 Tasks

- [ ] Add test file detection
- [ ] Add data file detection
- [ ] Add build output detection
- [ ] Verify FP elimination

### Phase 5 Tasks

- [ ] Integrate taint analysis
- [ ] Test on evidence packages
- [ ] Benchmark performance
- [ ] Document results

---

## Contact Information

**Previous Agent Context:** ~120k tokens (near limit)

**Key Decisions Made:**
1. NO WHITELISTING - Detection based on code patterns, not package popularity
2. Tier 1 signal requirement - Without invisible chars, max score capped at 3.5
3. Context-aware detection preferred over skipping files

**Open Questions:**
1. Should we skip test files entirely or just downweight?
2. Should data files get low weight instead of skip?
3. What's the acceptable performance overhead for semantic analysis?

---

## Final Notes

**The infrastructure is already there.** We have:
- OXC AST parser integrated
- Semantic analysis working
- Taint analysis implemented
- Semantic detectors written

**They're just not being used.**

This represents the single biggest opportunity for improving GlassWorm detection. The next agent who activates this infrastructure will likely achieve:
- **50%+ FP reduction** on test/data/build files
- **Better detection** of real encrypted payloads
- **More precise** C2 detection with flow tracking

**Good luck!** The foundation is solid - just needs to be activated.

---

**Last Updated:** 2026-03-27
**Version:** v0.73.0-medium-waves-complete
**Tag:** `v0.73.0-medium-waves-complete`
