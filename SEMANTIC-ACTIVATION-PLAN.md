# Semantic Detection Activation Plan

**Date:** 2026-03-27
**Starting Tag:** `v0.75.0-semantic-activation-start`
**Target:** Activate unused semantic analysis infrastructure for 50%+ FP reduction

---

## Executive Summary

**Current State:** We have extensive **unused semantic analysis infrastructure** that is:
- ✅ Integrated in `glassware-core`
- ✅ Feature-gated behind `semantic` feature (enabled by default via `full`)
- ✅ Compiling successfully
- ❌ **NOT being called by the scan engine**

**Critical Finding:** The semantic detectors (GW005-GW008) exist but are **never invoked** during scanning because:
1. The `scan_internal()` method checks for AST but the condition is wrong
2. Semantic detectors are registered but not properly integrated into the scan flow
3. No context-aware filtering for test/data/build files

**Goal:** Activate semantic detection to achieve:
- 50%+ FP reduction on test/data/build files
- Better detection of real encrypted payloads via taint tracking
- More precise C2 detection with flow tracking

---

## Current Infrastructure Status

### ✅ What Exists and Works

| Component | Location | Status |
|-----------|----------|--------|
| **OXC AST Parser** | `glassware-core/src/ir.rs` | ✅ Integrated, compiles |
| **Semantic Analysis** | `glassware-core/src/semantic.rs` | ✅ Full implementation |
| **Taint Analysis** | `glassware-core/src/taint.rs` | ✅ Sources, sinks, flows |
| **GW005 Semantic** | `glassware-core/src/gw005_semantic.rs` | ✅ Encrypted payload |
| **GW006 Semantic** | `glassware-core/src/gw006_semantic.rs` | ✅ Hardcoded key |
| **GW007 Semantic** | `glassware-core/src/gw007_semantic.rs` | ✅ RC4 pattern |
| **GW008 Semantic** | `glassware-core/src/gw008_semantic.rs` | ✅ Header C2 |
| **Detector Trait** | `glassware-core/src/detector.rs` | ✅ `SemanticDetector` trait |
| **Engine Registration** | `glassware-core/src/engine.rs` | ✅ `register_semantic()` |

### ❌ What's Broken

1. **Semantic detectors never run** - The condition in `scan_internal()` at line ~773-787 checks `ir.ast()` but AST parsing may not be happening correctly
2. **No context-aware filtering** - Test/data/build files are scanned same as production code
3. **Regex detectors still primary** - GW005-GW008 regex versions run instead of semantic versions

---

## Implementation Plan

### Phase 1: Diagnose & Fix AST Parsing (Day 1)

**Goal:** Verify AST is being parsed and semantic detectors are being called

**Tasks:**
1. Add logging to verify AST parsing in `FileIR::build()`
2. Check if `ir.ast()` returns `Some` in `scan_internal()`
3. Verify semantic detectors are registered in `default_detectors()`
4. Test on a simple JS file with known patterns

**Expected Outcome:** AST parsed for JS/TS files, semantic detectors running

---

### Phase 2: Add Context-Aware File Filtering (Day 2)

**Goal:** Skip or downweight test/data/build files using AST analysis

**Tasks:**
1. Add `is_test_file()` detection using AST (looks for `describe`, `it`, `test`)
2. Add `is_data_file()` detection (only imports/exports, no logic)
3. Add `is_build_output()` detection (webpack/rollup wrappers)
4. Integrate filtering in `scan_internal()` before running detectors

**Expected Outcome:** Test/data/build files skipped or flagged with low confidence

---

### Phase 3: Replace Regex Detectors with Semantic (Days 3-4)

**Goal:** Replace regex-based GW005-GW008 with semantic versions

**Priority:**
1. **GW005 (EncryptedPayload)** - Highest FP rate
2. **GW008 (HeaderC2)** - Second highest FP rate
3. **GW007 (RC4Pattern)** - Medium FP rate
4. **GW006 (HardcodedKey)** - Lowest FP rate

**For Each:**
1. Run semantic + regex in parallel
2. Compare findings
3. Disable regex when semantic is ready
4. Update tests

**Expected Outcome:** 50% FP reduction

---

### Phase 4: Taint Analysis Integration (Days 5-6)

**Goal:** Full taint tracking for encrypted payload detection

**Tasks:**
1. Integrate `find_sources()`, `find_sinks()`, `find_flows()` in GW005
2. Require taint flow for detection (not just pattern match)
3. Test on evidence packages
4. Benchmark performance

**Expected Outcome:** Detect only real encrypted payloads with flows

---

### Phase 5: Validation & Benchmarking (Day 7)

**Goal:** Validate FP reduction and detection accuracy

**Tasks:**
1. Re-scan Wave18-21 packages
2. Measure FP rate reduction
3. Test on evidence packages (must maintain 100% detection)
4. Benchmark performance overhead

**Success Metrics:**
- FP rate: < 0.2% (from 0.57%)
- Evidence detection: 100% maintained
- Performance: < 20% overhead

---

## File-by-File Changes Required

### 1. `glassware-core/src/ir.rs`

**Current:** AST parsing exists but may not be triggered

**Change:** Add logging to verify AST parsing:
```rust
#[cfg(feature = "semantic")]
let ast = if is_js_or_ts_path(path) {
    let result = JavaScriptAST::parse(content, path);
    tracing::debug!("AST parse for {:?}: {:?}", path, result.as_ref().map(|_| "success"));
    result.map(Arc::new)
} else {
    None
};
```

---

### 2. `glassware-core/src/engine.rs`

**Current:** Semantic detector call at line ~773-787

**Issue:** Condition may not be met

**Change:** Fix the condition and add logging:
```rust
// Run semantic detectors on JS/TS files only
#[cfg(feature = "semantic")]
if !self.semantic_detectors.is_empty() {
    tracing::debug!("Running {} semantic detectors", self.semantic_detectors.len());
    
    // Check if we have a valid AST
    if let Some(ast) = ir.ast() {
        if ast.is_valid() {
            tracing::debug!("AST is valid, building semantic analysis");
            
            // Build semantic analysis from source
            if let Some(analysis) = crate::semantic::build_semantic(content, path) {
                let sources = crate::taint::find_sources(&analysis);
                let sinks = crate::taint::find_sinks(&analysis);
                let flows = crate::taint::check_flows(&analysis, &sources, &sinks);

                tracing::debug!("Found {} sources, {} sinks, {} flows", 
                    sources.len(), sinks.len(), flows.len());

                for detector in &self.semantic_detectors {
                    let findings = detector.detect_semantic(content, path, &flows, &sources, &sinks);
                    tracing::debug!("Detector {} found {} findings", detector.id(), findings.len());
                    findings.extend(findings);
                }
            } else {
                tracing::debug!("Semantic analysis failed for {:?}", path);
            }
        } else {
            tracing::debug!("AST invalid for {:?}: {:?}", path, ast.errors);
        }
    } else {
        tracing::debug!("No AST available for {:?}", path);
    }
}
```

---

### 3. `glassware-core/src/detector.rs`

**Current:** `SemanticDetector` trait exists

**Change:** Add `should_skip_file()` method for context-aware detection:
```rust
#[cfg(feature = "semantic")]
pub trait SemanticDetector: Send + Sync {
    /// Unique identifier matching a GW rule (e.g., "GW005")
    fn id(&self) -> &str;

    /// Run detection using semantic analysis + taint flows
    fn detect_semantic(
        &self,
        source_code: &str,
        path: &Path,
        flows: &[crate::taint::TaintFlow],
        sources: &[crate::taint::TaintSource],
        sinks: &[crate::taint::TaintSink],
    ) -> Vec<Finding>;

    /// Check if this file should be skipped (test/data/build files)
    fn should_skip_file(&self, _analysis: &crate::semantic::SemanticAnalysis) -> bool {
        false  // Default: don't skip
    }
}
```

---

### 4. New File: `glassware-core/src/context_filter.rs`

**Purpose:** Context-aware file classification using AST

```rust
//! Context-Aware File Filter
//!
//! Uses AST analysis to classify files as:
//! - Test files (should skip or downweight)
//! - Data files (should skip or downweight)
//! - Build output (should skip)
//! - Production code (full detection)

use crate::semantic::SemanticAnalysis;

/// File classification based on AST analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileClassification {
    /// Test file - skip or downweight
    Test,
    /// Data file - skip or downweight
    Data,
    /// Build output - skip
    BuildOutput,
    /// Production code - full detection
    Production,
}

/// Classify a file based on semantic analysis
pub fn classify_file(analysis: &SemanticAnalysis, path: &Path) -> FileClassification {
    // Check for test patterns first
    if is_test_file(analysis) {
        return FileClassification::Test;
    }

    // Check for data files
    if is_data_file(analysis) {
        return FileClassification::Data;
    }

    // Check for build output
    if is_build_output(analysis, path) {
        return FileClassification::BuildOutput;
    }

    FileClassification::Production
}

/// Check if file is a test file
fn is_test_file(analysis: &SemanticAnalysis) -> bool {
    // Look for test framework calls
    let test_calls = ["describe", "it", "test", "expect", "beforeEach", "afterEach"];
    
    analysis.call_sites.iter().any(|call| {
        test_calls.contains(&call.callee.as_str())
    })
}

/// Check if file is a data file
fn is_data_file(analysis: &SemanticAnalysis) -> bool {
    // Data files have:
    // - Many string literals (data)
    // - Few or no function calls (no logic)
    // - Many declarations (constants)
    
    let string_count = analysis.string_literals.len();
    let call_count = analysis.call_sites.len();
    let decl_count = analysis.declarations.len();
    
    // Heuristic: lots of strings, very few calls
    string_count > 10 && call_count < 3 && decl_count > 5
}

/// Check if file is build output
fn is_build_output(analysis: &SemanticAnalysis, path: &Path) -> bool {
    // Check path patterns
    let path_str = path.to_string_lossy();
    if path_str.contains("/dist/") || 
       path_str.contains("/build/") || 
       path_str.contains(".min.") ||
       path_str.contains(".bundle.") {
        return true;
    }

    // Check for webpack/rollup wrapper patterns
    analysis.call_sites.iter().any(|call| {
        call.callee_chain.contains(&"__webpack_require__".to_string()) ||
        call.callee_chain.contains(&"__rollup__".to_string())
    })
}
```

---

### 5. `glassware-core/src/gw005_semantic.rs`

**Current:** Basic implementation exists

**Change:** Add context-aware filtering and taint flow requirement:
```rust
impl SemanticDetector for Gw005SemanticDetector {
    fn detect_semantic(
        &self,
        source_code: &str,
        path: &Path,
        flows: &[TaintFlow],
        sources: &[TaintSource],
        sinks: &[TaintSink],
    ) -> Vec<Finding> {
        // Build semantic analysis for context check
        if let Some(analysis) = crate::semantic::build_semantic(source_code, path) {
            // Skip test/data/build files
            match classify_file(&analysis, path) {
                FileClassification::Test => {
                    tracing::debug!("Skipping test file: {:?}", path);
                    return vec![];
                }
                FileClassification::Data => {
                    tracing::debug!("Skipping data file: {:?}", path);
                    return vec![];
                }
                FileClassification::BuildOutput => {
                    tracing::debug!("Skipping build output: {:?}", path);
                    return vec![];
                }
                FileClassification::Production => {}  // Continue detection
            }
        }

        // Only flag if there's a suspicious taint flow
        flows
            .iter()
            .filter_map(|flow| {
                // Require high-entropy source flowing to dynamic exec
                let is_entropy_source = matches!(flow.source, TaintSource::HighEntropyString { .. });
                if !is_entropy_source {
                    return None;
                }

                let is_exec_sink = matches!(flow.sink, TaintSink::DynamicExec { .. });
                if !is_exec_sink {
                    return None;
                }

                // ... rest of detection logic
            })
            .collect()
    }
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_semantic_detects_test_files() {
    let source = r#"
        describe('test suite', () => {
            it('should work', () => {
                expect(true).toBe(true);
            });
        });
    "#;
    let path = Path::new("test.js");
    let analysis = build_semantic(source, path).unwrap();
    
    assert!(is_test_file(&analysis));
    assert_eq!(classify_file(&analysis, path), FileClassification::Test);
}

#[test]
fn test_taint_finds_encrypted_payload() {
    let source = r#"
        const key = "hardcoded_key_12345";
        const encrypted = atob("YmFzZTY0IGRhdGE=");
        const decrypted = decrypt(encrypted, key);
        eval(decrypted);
    "#;
    let path = Path::new("test.js");
    let semantic = build_semantic(source, path).unwrap();
    let sources = find_sources(&semantic);
    let sinks = find_sinks(&semantic);
    let flows = find_flows(&semantic, &sources, &sinks);

    assert!(flows.iter().any(|f| f.is_suspicious()));
}
```

### Integration Tests

1. **Test on Wave18-21 FPs** - Verify 6 FPs are now skipped
2. **Test on evidence packages** - Verify 100% detection maintained
3. **Performance benchmark** - Measure AST parsing overhead

---

## Success Metrics

| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| FP Rate | 0.57% | < 0.2% | Wave18-21 re-scan |
| Test/Data FPs | 6 | 0 | Manual review |
| Evidence Detection | 100% | 100% | Evidence packages |
| Performance | ~50k LOC/s | ~40k LOC/s | Benchmark |
| Semantic Coverage | 0% | >80% | AST parse rate |

---

## Risks and Mitigations

### Risk 1: AST Parsing Overhead

**Risk:** OXC parsing adds 20-30% overhead

**Mitigation:**
- Parse once in `FileIR::build()`, share across all detectors
- Only parse JS/TS files (skip JSON, etc.)
- Cache AST for re-scans (future optimization)

### Risk 2: Semantic Detectors Miss Attacks

**Risk:** AST-based detection misses edge cases

**Mitigation:**
- Run semantic + regex in parallel initially
- Compare results before disabling regex
- Keep regex as fallback for non-JS/TS files

### Risk 3: OXC Version Compatibility

**Risk:** OXC API changes break integration

**Mitigation:**
- Pin OXC version in `Cargo.toml` (currently 0.40)
- Add version check in build script
- Test on OXC updates

---

## Timeline

| Phase | Duration | End Date |
|-------|----------|----------|
| Phase 1: Diagnose & Fix AST | 1 day | Mar 28 |
| Phase 2: Context-Aware Filtering | 1 day | Mar 29 |
| Phase 3: Replace Regex Detectors | 2 days | Mar 31 |
| Phase 4: Taint Analysis | 2 days | Apr 2 |
| Phase 5: Validation | 1 day | Apr 3 |
| **Total** | **7 days** | **Apr 3, 2026** |

---

## Next Steps

1. ✅ Create starting checkpoint tag (`v0.75.0-semantic-activation-start`)
2. ⏳ **Phase 1:** Add logging and verify AST parsing
3. ⏳ **Phase 2:** Add context-aware filtering
4. ⏳ **Phase 3:** Replace regex detectors
5. ⏳ **Phase 4:** Integrate taint analysis
6. ⏳ **Phase 5:** Validate and benchmark

---

**Last Updated:** 2026-03-27
**Author:** AI Agent
**Reviewers:** Previous agent (via handoff doc)
