# Attack Graph Engine Implementation Summary

## Overview

Successfully implemented an attack graph engine that correlates individual security findings into unified attack narratives (attack chains) for the glassware threat detection system.

## Files Created

### 1. `/home/property.sightlines/samgrowls/glassworks/glassware-core/src/attack_graph.rs`
- Main module for attack graph capabilities
- Re-exports correlation types
- Provides `AttackGraphResult` wrapper
- Defines `ScanEngineAttackGraphExt` trait for integration

### 2. `/home/property.sightlines/samgrowls/glassworks/glassware-core/src/correlation.rs`
- Core correlation logic and data structures
- Attack chain detection algorithms
- Confidence scoring system
- Threat score calculation

### 3. `/home/property.sightlines/samgrowls/glassworks/glassware-core/tests/integration_attack_graph.rs`
- Integration tests for attack graph engine
- Tests for all major attack chain types
- Validation of threat scoring

## Files Modified

### 1. `/home/property.sightlines/samgrowls/glassworks/glassware-core/src/lib.rs`
- Added `correlation` and `attack_graph` modules
- Re-exported `AttackChain`, `AttackGraphEngine`, `AttackLocation`, `AttackType`
- Re-exported `AttackGraphResult`, `ScanEngineAttackGraphExt`

### 2. `/home/property.sightlines/samgrowls/glassworks/glassware-core/src/engine.rs`
- Added `attack_graph: Option<AttackGraphEngine>` field to `ScanEngine`
- Added `with_attack_graph(bool)` builder method
- Updated `ScanResult` to include:
  - `attack_chains: Vec<AttackChain>`
  - `threat_score: f32`
- Integrated attack graph correlation into `scan_internal()`

### 3. `/home/property.sightlines/samgrowls/glassworks/glassware-core/src/minified.rs`
- Fixed test compilation error (unrelated bug fix)

## Attack Chain Types Implemented

### 1. GlassWareStego
- **Pattern**: Unicode steganography → decoder function → eval/exec
- **Required indicators**:
  - SteganoPayload OR InvisibleCharacter OR PipeDelimiterStego
  - DecoderFunction OR GlasswarePattern
  - Dynamic execution (eval, Function, etc.)
- **Confidence**: 0.95 (all 3 indicators), 0.75 (2 indicators)
- **Severity**: Critical

### 2. EncryptedExec
- **Pattern**: High-entropy blob → decryption → dynamic execution
- **Required indicators**:
  - EncryptedPayload OR HardcodedKeyDecryption OR Rc4Pattern
  - Dynamic execution
- **Confidence**: 0.9 (multiple decrypt indicators), 0.75 (single indicator)
- **Severity**: High

### 3. HeaderC2Chain
- **Pattern**: HTTP header extraction → decryption → execution
- **Required indicators**:
  - HeaderC2 finding
  - Dynamic execution
- **Confidence**: 0.95
- **Severity**: Critical

### 4. BlockchainC2
- **Pattern**: Blockchain API calls → data extraction → decryption → execution
- **Required indicators**:
  - BlockchainC2 finding
  - Dynamic execution
- **Confidence**: 0.9
- **Severity**: Critical

### 5. GeofencedExec
- **Pattern**: Locale/timezone check → time delay → execution
- **Required indicators**:
  - LocaleGeofencing OR TimeDelaySandboxEvasion
  - Dynamic execution
- **Confidence**: 0.85 (both indicators), 0.7 (single indicator)
- **Severity**: High

### 6. SupplyChainCompromise
- **Pattern**: Similar attack patterns across multiple packages
- **Required indicators**:
  - 2+ packages with high-severity findings
- **Confidence**: 0.9
- **Severity**: Critical

## Key Data Structures

### AttackChain
```rust
pub struct AttackChain {
    pub id: String,              // Unique identifier
    pub steps: Vec<Finding>,     // Ordered attack steps
    pub confidence: f32,         // 0.0-1.0 confidence score
    pub classification: AttackType,
    pub location: AttackLocation,
}
```

### AttackType
```rust
pub enum AttackType {
    GlassWareStego,
    EncryptedExec,
    HeaderC2Chain,
    RemoteDependencyDelivery,
    SupplyChainCompromise,
    BlockchainC2,
    GeofencedExec,
    Unknown,
}
```

### AttackLocation
```rust
pub struct AttackLocation {
    pub package: String,
    pub files: Vec<String>,
    pub version: Option<String>,
}
```

## Threat Scoring System

Threat score ranges from 0.0 to 10.0, calculated based on:

1. **Base score** from chain confidence (0-2 points per chain)
2. **Severity multiplier**:
   - Critical: 2.0x
   - High: 1.5x
   - Medium: 1.0x
   - Low: 0.5x
   - Info: 0.25x

3. **Attack type criticality**:
   - SupplyChainCompromise: 2.0x
   - GlassWareStego: 1.8x
   - HeaderC2Chain: 1.8x
   - BlockchainC2: 1.6x
   - EncryptedExec: 1.4x
   - RemoteDependencyDelivery: 1.3x
   - GeofencedExec: 1.2x

4. **Step count bonus**: +0.2 per step (max +1.0)

## Integration with ScanEngine

### Usage Example

```rust
use glassware_core::{ScanEngine, AttackType};
use std::path::Path;

// Create engine with attack graph enabled
let engine = ScanEngine::default_detectors()
    .with_attack_graph(true);

// Scan with attack graph correlation
let result = engine.scan_with_stats(Path::new("src/index.js"), content);

// Access attack chains
for chain in &result.attack_chains {
    println!("Attack chain: {:?}", chain.classification);
    println!("Confidence: {:.2}", chain.confidence);
    println!("Steps: {}", chain.step_count());
}

// Get overall threat score
println!("Threat score: {:.1}/10.0", result.threat_score);
```

### Builder Pattern

```rust
// Enable attack graph
let engine = ScanEngine::default_detectors()
    .with_attack_graph(true)
    .with_llm(true)
    .with_deduplication(true);

// Disable attack graph (default)
let engine = ScanEngine::default_detectors()
    .with_attack_graph(false);
```

## Test Results

### Unit Tests (correlation module)
- ✅ `test_attack_chain_creation` - Verifies chain requires all indicators
- ✅ `test_attack_location_extraction` - Tests package name extraction
- ✅ `test_attack_type_description` - Verifies descriptions exist
- ✅ `test_attack_type_severity` - Tests severity mappings
- ✅ `test_encrypted_exec_chain_detection` - Tests encrypted exec detection
- ✅ `test_glassware_chain_detection` - Tests GlassWare stego chain
- ✅ `test_header_c2_chain_detection` - Tests Header C2 chain
- ✅ `test_no_chain_for_clean_code` - Verifies no false positives
- ✅ `test_threat_score_calculation` - Tests threat scoring

### Integration Tests
- ✅ `test_attack_graph_clean_code` - Clean code produces no chains
- ✅ `test_attack_graph_disabled` - Feature can be disabled
- ✅ `test_attack_graph_blockchain_c2` - Blockchain C2 pattern
- ✅ `test_attack_graph_header_c2` - Header C2 pattern detection
- ✅ `test_attack_graph_multiple_chains` - Multiple chain detection
- ✅ `test_attack_graph_wave5_aes_decrypt_eval` - Wave 5 AES pattern

**Total: 15 tests passing**

## Performance Characteristics

- **Time complexity**: O(n) for correlation where n = number of findings
- **Space complexity**: O(n) for storing chains
- **Overhead**: Minimal - correlation runs after detectors complete
- **Caching**: Attack chains are NOT cached (computed on-demand)

## Design Decisions

1. **Opt-in feature**: Attack graph correlation is disabled by default to maintain backward compatibility
2. **Immutable correlation**: `AttackGraphEngine` is cloned for each scan to avoid mutation issues
3. **File-based grouping**: Findings are correlated by file path for spatial proximity
4. **Priority-based detection**: Chain types are checked in priority order (GlassWare first)
5. **Confidence scoring**: Based on indicator count and pattern completeness
6. **Threat score capping**: Maximum score of 10.0 prevents runaway values

## Future Enhancements

1. **Cross-file correlation**: Track attack chains spanning multiple files
2. **Temporal analysis**: Add timestamp-based correlation for runtime detection
3. **Graph visualization**: Export attack chains to DOT/GraphViz format
4. **Machine learning**: Use historical data to improve confidence scoring
5. **SARIF integration**: Export chains as related locations in SARIF format
6. **JSON output**: Include chains in JSON output format

## Compatibility

- **Rust edition**: 2021
- **Minimum Rust version**: 1.70
- **Features**: Works with `full`, `minimal`, `semantic`, `llm` feature flags
- **Backward compatible**: Existing code continues to work without changes

## Build Verification

```bash
# Debug build
cargo build --features "full,llm"
# ✅ SUCCESS

# Release build
cargo build --release --features "full,llm"
# ✅ SUCCESS

# Run tests
cargo test --features "full,llm" -p glassware-core --lib correlation
cargo test --features "full,llm" -p glassware-core --lib attack_graph
cargo test --features "full,llm" -p glassware-core --test integration_attack_graph
# ✅ All 15 tests pass
```

## Conclusion

The attack graph engine successfully transforms individual security findings into coherent attack narratives, providing:

1. **Higher-level insights**: Correlates low-level findings into attack stories
2. **Confidence scoring**: Reduces false positives through multi-indicator correlation
3. **Threat prioritization**: Threat score helps prioritize remediation efforts
4. **Extensibility**: Easy to add new attack chain types
5. **Production-ready**: All tests pass, release build succeeds

This implementation fulfills the requirements from CODEREVIEW_193_2 sections 2.1 and 6.1 for evolving glassware from a threat detection framework into a threat intelligence system.
