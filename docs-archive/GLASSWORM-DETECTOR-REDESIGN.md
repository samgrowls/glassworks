# GlassWorm Detector Redesign Plan

**Date:** 2026-03-25  
**Status:** Critical Fix Required

---

## Current Problem

**Detector Name:** `GlasswarePattern` (misleading - should be `GlassWorm`)

**Current Logic:**
```rust
// Flags as "GlassWare attack" if 2+ of these found:
decoder_patterns = [
    r"String\.fromCharCode\s*\(",      // ❌ TOO BROAD - used everywhere
    r"String\.fromCodePoint\s*\(",     // ❌ TOO BROAD - standard Unicode
    r"\.filter\s*\(\s*c\s*=>\s*c\s*!==\s*null\s*\)",  // ❌ TOO BROAD
]

encoding_patterns = [
    r"\batob\s*\(",                     // ❌ TOO BROAD - standard base64
    r"\bbtoa\s*\(",                     // ❌ TOO BROAD
    r"Buffer\.from\s*\([^,]+,\s*base64", // ❌ TOO BROAD
]
```

**Result:** Firebase, Web3, Prisma, Webpack all flagged (10% FP rate)

---

## What Real GlassWorm Looks Like

Based on confirmed evidence (react-native-country-select, iflow-mcp-watercrawl):

### Attack Pattern

1. **Invisible Unicode Payload** - Hidden in source code
   - Variation Selectors (U+FE00-U+FE0F, U+E0100-U+E01EF)
   - Zero-Width characters (U+200B-ZWSP, U+200C-ZWNJ, U+200D-ZWJ)
   - Embedded in strings, comments, identifiers

2. **Decoder** - Specifically extracts invisible payload
   - `codePointAt()` with VS constants (0xFE00, 0xE0100)
   - Filters specifically for invisible ranges
   - Reconstructs hidden string from VS sequences

3. **Execution** - Runs decoded payload
   - `eval()`, `Function()`, dynamic `require()`
   - Often with obfuscation (hex encoding, split strings)

### Key Insight

**GlassWorm = Invisible + Decoder + Execution**

All THREE must be present. Not just decoder alone.

---

## Redesigned Detector Logic

### Rename: `GlasswarePattern` → `GlassWorm`

**File:** `glassware-core/src/detectors/glassware.rs` → `glassware-core/src/detectors/glassworm.rs`

### New Detection Logic

```rust
fn detect(&self, ir: &FileIR) -> Vec<Finding> {
    let content = ir.content();
    let path = &ir.metadata.path;
    
    // Step 1: Check for invisible Unicode characters
    let invisible_chars = find_invisible_chars(content);
    if invisible_chars.is_empty() {
        return Vec::new();  // No invisible chars = not GlassWorm
    }
    
    // Step 2: Check for GlassWorm-specific decoder patterns
    let decoder_indicators = detect_glassworm_decoder(content);
    if decoder_indicators.is_empty() {
        return Vec::new();  // Decoder without invisible chars = likely legitimate
    }
    
    // Step 3: Check for payload execution
    let execution_indicators = detect_payload_execution(content);
    
    // Step 4: Calculate confidence based on combination
    let confidence = calculate_glassworm_confidence(
        invisible_chars.len(),
        decoder_indicators.len(),
        execution_indicators.len(),
    );
    
    // Only flag if confidence >= 0.70 (70%)
    if confidence >= 0.70 {
        findings.push(Finding {
            severity: if confidence >= 0.90 { Critical } else { High },
            description: format!("GlassWorm steganography detected (confidence: {:.0}%)", confidence * 100),
            confidence: Some(confidence),
            // ... include details about what was found
        });
    }
    
    findings
}
```

### Specific Patterns

#### 1. Invisible Character Detection

```rust
fn find_invisible_chars(content: &str) -> Vec<FoundChar> {
    content.char_indices()
        .filter(|(_, ch)| {
            let cp = *ch as u32;
            // Variation Selectors (GlassWorm primary)
            (cp >= 0xFE00 && cp <= 0xFE0F) ||  // VS1-16
            (cp >= 0xE0100 && cp <= 0xE01EF) || // VS17-256
            // Zero-Width characters
            cp == 0x200B ||  // ZWSP
            cp == 0x200C ||  // ZWNJ
            cp == 0x200D ||  // ZWJ
            cp == 0x2060     // Word Joiner
        })
        .collect()
}
```

#### 2. GlassWorm-Specific Decoder

```rust
fn detect_glassworm_decoder(content: &str) -> Vec<DecoderIndicator> {
    let mut indicators = Vec::new();
    
    // VS-specific decoding (GlassWorm signature)
    if content.contains("codePointAt") && 
       (content.contains("0xFE00") || content.contains("0xE0100")) {
        indicators.push(DecoderIndicator {
            pattern: "vs_codepoint_decode",
            confidence: 0.95,  // Very specific to GlassWorm
        });
    }
    
    // Filtering invisible chars specifically
    if content.contains(".filter") && 
       (content.contains("0x200") || content.contains("0xFE0")) {
        indicators.push(DecoderIndicator {
            pattern: "invisible_filter",
            confidence: 0.90,
        });
    }
    
    // Reconstructing from VS sequences
    if content.contains("fromCodePoint") &&
       content.contains("map") &&
       (content.contains("0xFE0") || content.contains("0xE01")) {
        indicators.push(DecoderIndicator {
            pattern: "vs_reconstruction",
            confidence: 0.92,
        });
    }
    
    // Skip generic patterns (atob, fromCharCode without VS context)
    // These are legitimate and NOT GlassWorm-specific
    
    indicators
}
```

#### 3. Payload Execution Detection

```rust
fn detect_payload_execution(content: &str) -> Vec<ExecutionIndicator> {
    let mut indicators = Vec::new();
    
    // Dynamic code execution (high risk)
    if content.contains("eval(") || content.contains("new Function(") {
        indicators.push(ExecutionIndicator {
            pattern: "dynamic_exec",
            confidence: 0.85,
        });
    }
    
    // Dynamic require (common in GlassWorm)
    if content.contains("require(") && 
       (content.contains("+") || content.contains("[")) {  // String concat
        indicators.push(ExecutionIndicator {
            pattern: "dynamic_require",
            confidence: 0.80,
        });
    }
    
    // Obfuscated execution
    if content.contains(".call(null,") || content.contains(".apply(null,") {
        indicators.push(ExecutionIndicator {
            pattern: "obfuscated_call",
            confidence: 0.75,
        });
    }
    
    indicators
}
```

#### 4. Confidence Calculation

```rust
fn calculate_glassworm_confidence(
    invisible_count: usize,
    decoder_count: usize,
    execution_count: usize,
) -> f32 {
    let mut score = 0.0;
    
    // Invisible chars (required)
    if invisible_count > 0 {
        score += 0.30;  // Base requirement
        if invisible_count > 10 {
            score += 0.10;  // Significant payload
        }
        if invisible_count > 50 {
            score += 0.10;  // Large payload
        }
    }
    
    // Decoder patterns (required)
    if decoder_count > 0 {
        score += 0.30;  // Base decoder
        // Add confidence from specific patterns
        for indicator in decoder_indicators {
            score += indicator.confidence * 0.20;
        }
    }
    
    // Execution patterns (boosts confidence)
    if execution_count > 0 {
        score += 0.20;  // Execution detected
    }
    
    // Cap at 1.0
    score.min(1.0)
}

// Minimum thresholds for flagging
const MIN_CONFIDENCE_TO_FLAG: f32 = 0.70;  // 70%
const MIN_CONFIDENCE_CRITICAL: f32 = 0.90; // 90%
```

---

## Testing Strategy

### Test Set 1: Known Malicious (Must Detect)

```bash
# Original evidence
./target/release/glassware scan-tarball evidence/react-native-country-select-0.3.91.tgz
# Expected: Detected with confidence >= 0.90

./target/release/glassware scan-tarball evidence/iflow-mcp-watercrawl-mcp-1.3.4.tgz
# Expected: Detected with confidence >= 0.95
```

### Test Set 2: Known Clean (Must NOT Detect)

```bash
# Major SDKs
./target/release/glassware scan-npm firebase@10.7.2
# Expected: NOT detected (confidence < 0.50)

./target/release/glassware scan-npm @solana/web3.js@1.87.6
# Expected: NOT detected

./target/release/glassware scan-npm @prisma/client@5.8.1
# Expected: NOT detected

./target/release/glassware scan-npm webpack@5.89.0
# Expected: NOT detected
```

### Test Set 3: Wave 10 Clean Baseline

```bash
./target/release/glassware campaign run campaigns/wave10-1000plus.toml
# Expected: FP rate < 1% (≤6/650 packages)
```

---

## Implementation Plan

### Phase 1: Rename and Restructure (Today)

1. **Rename detector:** `glassware.rs` → `glassworm.rs`
2. **Update module references** in `mod.rs`
3. **Update finding category:** `GlasswarePattern` → `GlassWorm`

### Phase 2: Implement New Logic (Today)

1. **Add invisible char detection** (reuse from invisible.rs detector)
2. **Implement GlassWorm-specific decoder patterns** (VS-specific, not generic)
3. **Add execution pattern detection**
4. **Implement confidence calculation**
5. **Require 70% confidence minimum to flag**

### Phase 3: Test on Evidence (Today)

1. **Test on 4 original tarballs** - Must all detect with >= 90% confidence
2. **Test on firebase, web3, prisma** - Must NOT detect (< 50% confidence)

### Phase 4: Test on Wave 10 (Today)

1. **Run full Wave 10** - FP rate must be < 1%
2. **Analyze any remaining FPs** - Tune confidence thresholds

### Phase 5: Documentation (Tomorrow)

1. **Update detector documentation**
2. **Document GlassWorm attack pattern**
3. **Add test cases to test suite**

---

## Success Criteria

### Detector Quality

- ✅ Evidence detection: 4/4 (100%)
- ✅ Firebase/Web3/Prisma: NOT flagged
- ✅ Wave 10 FP rate: < 1% (<6/650 packages)
- ✅ Confidence scores: Meaningful (90% = very likely real)

### Code Quality

- ✅ Clear naming: `GlassWorm` not `GlasswarePattern`
- ✅ Well-documented patterns
- ✅ Test coverage for both positive and negative cases
- ✅ Confidence calculation is transparent

---

## Timeline

**Day 1 (Today):**
- Rename detector
- Implement new logic
- Test on evidence + major SDKs

**Day 2 (Tomorrow):**
- Run Wave 10 validation
- Tune confidence thresholds
- Documentation

**Day 3 (If Needed):**
- Additional tuning
- Final validation
- Prepare for tag (v0.57.0 or v0.58.0)

---

**Status:** Ready to implement  
**Priority:** CRITICAL - blocks release  
**Owner:** Qwen-Coder
