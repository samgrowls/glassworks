# GlassWorm Intelligence Integration Plan

**Date:** 2026-03-21  
**Priority:** HIGH  
**Status:** 📋 Planning Phase

---

## Overview

Integrate new GlassWorm campaign intelligence into glassware detection engine and severity scoring system.

---

## New Intelligence Summary

Based on recent GlassWorm campaign analysis, we need to integrate:

### 1. Enhanced Severity Scoring

**Current System:**
- Fixed severity per detector (Critical, High, Medium, Low)
- No context awareness
- No package reputation tracking

**Proposed System:**
- Context-aware severity multipliers
- Package reputation scoring
- Ecosystem-based risk assessment
- Temporal factors (package age, update frequency)

### 2. New Detection Patterns

**Steganography Enhancements:**
- Improved variation selector detection
- Unicode tag character combinations
- Multi-layer encoding detection
- Cross-file payload reconstruction

**Behavioral Patterns:**
- Install script → network call chains
- Decryption → eval flows
- C2 communication patterns
- Persistence mechanisms

**Author/Package Patterns:**
- Known bad actor signatures (JPD, etc.)
- Typosquatting detection
- Dependency confusion indicators
- Rapid version release patterns

---

## Implementation Plan

### Phase 1: Severity Scoring Overhaul

**Files to Modify:**
- `glassware-core/src/finding.rs` - Severity enum
- `glassware-core/src/risk_scorer.rs` - Risk calculation
- `glassware-orchestrator/src/scanner.rs` - Threat score calculation

**New Module:**
- `glassware-core/src/severity_context.rs` - Context-aware scoring

**Features:**
```rust
pub struct SeverityContext {
    // Package metadata
    pub package_age_days: u32,
    pub author_reputation: ReputationScore,
    pub download_count: u64,
    pub ecosystem: Ecosystem,
    
    // Behavioral indicators
    pub has_install_script: bool,
    pub has_network_calls: bool,
    pub has_eval_usage: bool,
    
    // Historical data
    pub previous_versions_clean: bool,
    pub rapid_version_changes: bool,
}

pub enum ReputationScore {
    KnownGood,      // Established, reputable
    Unknown,        // New or no history
    Suspicious,     // Some red flags
    KnownBad,       // Confirmed malicious
}

pub enum Ecosystem {
    Npm,            // Lower risk (more scrutiny)
    GitHub,         // Higher risk (less scrutiny)
    PyPI,           // Medium risk
}
```

**Severity Multipliers:**
```rust
impl SeverityContext {
    pub fn calculate_multiplier(&self) -> f64 {
        let mut multiplier = 1.0;
        
        // Package age
        if self.package_age_days < 7 {
            multiplier *= 2.0;  // Very new
        } else if self.package_age_days < 30 {
            multiplier *= 1.5;  // Relatively new
        }
        
        // Author reputation
        match self.author_reputation {
            ReputationScore::KnownBad => multiplier *= 3.0,
            ReputationScore::Suspicious => multiplier *= 2.0,
            ReputationScore::Unknown => multiplier *= 1.0,
            ReputationScore::KnownGood => multiplier *= 0.5,
        }
        
        // Ecosystem
        match self.ecosystem {
            Ecosystem::GitHub => multiplier *= 1.5,
            Ecosystem::Npm => multiplier *= 1.0,
            Ecosystem::PyPI => multiplier *= 1.2,
        }
        
        // Behavioral indicators
        if self.has_install_script && self.has_network_calls {
            multiplier *= 2.0;  // High risk combination
        }
        
        if self.has_eval_usage {
            multiplier *= 1.5;
        }
        
        multiplier
    }
}
```

---

### Phase 2: New Detector Patterns

**New Detectors:**
1. `glassware-core/src/detectors/stego_enhanced.rs`
2. `glassware-core/src/detectors/behavioral_chain.rs`
3. `glassware-core/src/detectors/author_signature.rs`
4. `glassware-core/src/detectors/typosquat.rs`

**Steganography Enhancements:**
```rust
// Detect multi-layer encoding
pub struct EnhancedStegoDetector {
    // Layer 1: Invisible characters
    invisible_detector: InvisibleCharDetector,
    
    // Layer 2: Unicode tag combinations
    tag_combinator: UnicodeTagCombinator,
    
    // Layer 3: Payload entropy analysis
    entropy_analyzer: EntropyAnalyzer,
}

impl Detector for EnhancedStegoDetector {
    fn detect(&self, file: &Path, content: &str) -> Vec<Finding> {
        let mut findings = Vec::new();
        
        // Check for layered encoding
        let invisible = self.invisible_detector.detect(file, content);
        if !invisible.is_empty() {
            // Decode and analyze entropy
            let decoded = decode_stego(content);
            let entropy = calculate_entropy(&decoded);
            
            if entropy > HIGH_ENTROPY_THRESHOLD {
                findings.push(Finding::new(
                    file,
                    0,
                    0,
                    0,
                    '\0',
                    DetectionCategory::EnhancedSteganography,
                    Severity::Critical,
                    "Multi-layer steganographic payload detected",
                    "Review decoded payload for malicious code",
                ));
            }
        }
        
        findings
    }
}
```

**Behavioral Chain Detection:**
```rust
pub struct BehavioralChainDetector {
    // Track data flow
    taint_tracker: TaintTracker,
}

impl Detector for BehavioralChainDetector {
    fn detect(&self, file: &Path, content: &str) -> Vec<Finding> {
        let mut findings = Vec::new();
        
        // Look for: install script → network → decrypt → eval
        let has_install = content.contains("preinstall") || 
                         content.contains("postinstall");
        let has_network = content.contains("fetch(") || 
                         content.contains("axios.") ||
                         content.contains("https://");
        let has_decrypt = content.contains("createDecipheriv") ||
                         content.contains("decrypt");
        let has_eval = content.contains("eval(") ||
                      content.contains("Function(") ||
                      content.contains("vm.run");
        
        // Score based on chain completeness
        let chain_score = [has_install, has_network, has_decrypt, has_eval]
            .iter()
            .filter(|&&x| x)
            .count();
        
        if chain_score >= 3 {
            findings.push(Finding::new(
                file,
                0,
                0,
                0,
                '\0',
                DetectionCategory::BehavioralChain,
                Severity::Critical,
                format!("Suspicious behavioral chain detected ({} of 4 indicators)", chain_score),
                "Manual review required - potential GlassWorm attack",
            ));
        }
        
        findings
    }
}
```

---

### Phase 3: LLM Prompt Enhancement

**Current Prompt:** Basic finding analysis

**Enhanced Prompt:**
```
You are a security analyst specializing in npm supply chain attacks, 
particularly GlassWare/GlassWorm campaigns.

Analyze this finding with context:

## Package Context
- Name: {package_name}
- Version: {package_version}
- Age: {package_age_days} days
- Author: {author} (Reputation: {reputation})
- Downloads: {download_count}
- Ecosystem: {ecosystem}

## Finding Details
- Category: {category}
- Severity: {severity}
- File: {file_path}:{line}
- Code: {code_snippet}

## Behavioral Indicators
- Has install script: {has_install}
- Has network calls: {has_network}
- Has eval/Function: {has_eval}
- Has decryption: {has_decrypt}

## Analysis Tasks

1. **Context Assessment**
   - Is this package newly created (< 30 days)?
   - Does the author have a history of malicious packages?
   - Is this a typosquat of a popular package?

2. **Pattern Matching**
   - Does this match known GlassWorm patterns?
   - Is the code structure consistent with legitimate usage?
   - Are there hidden payloads or decoders?

3. **Risk Assessment**
   - What is the likelihood this is malicious?
   - What would be the impact if malicious?
   - Are there C2 communication patterns?

## Response Format

{{
  "classification": "MALICIOUS" | "SUSPICIOUS" | "LIKELY_BENIGN" | "FALSE_POSITIVE",
  "confidence": 0.0-1.0,
  "severity_adjustment": -2.0 to +2.0,
  "reasoning": "Detailed explanation",
  "indicators": ["specific technical indicators"],
  "recommended_action": "BLOCK" | "REVIEW" | "MONITOR" | "IGNORE",
  "glassworm_campaign_match": true/false,
  "similar_known_threats": ["threat1", "threat2"]
}}
```

---

### Phase 4: Integration & Testing

**Test Fixtures:**
- Add GlassWorm campaign samples to `glassware-core/tests/fixtures/glassworm/`
- Add false positive cases to `glassware-core/tests/fixtures/false_positives/`
- Add behavioral chain examples to `glassware-core/tests/fixtures/behavioral/`

**Test Cases:**
```rust
#[test]
fn test_severity_context_multiplier() {
    let context = SeverityContext {
        package_age_days: 3,
        author_reputation: ReputationScore::Unknown,
        ecosystem: Ecosystem::GitHub,
        has_install_script: true,
        has_network_calls: true,
        has_eval_usage: true,
        ..Default::default()
    };
    
    let multiplier = context.calculate_multiplier();
    assert!(multiplier > 5.0);  // High risk package
}

#[test]
fn test_behavioral_chain_detector() {
    let detector = BehavioralChainDetector::new();
    let content = r#"
        {
          "scripts": {
            "postinstall": "node install.js"
          }
        }
    "#;
    
    let findings = detector.detect(Path::new("test.json"), content);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].severity, Severity::Critical);
}
```

---

## Timeline

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| 1. Severity Scoring | 1 week | `severity_context.rs`, multipliers |
| 2. Detector Patterns | 1 week | 4 new detectors, test fixtures |
| 3. LLM Enhancement | 3 days | Enhanced prompts, response parsing |
| 4. Integration | 3 days | Tests, documentation, benchmarks |
| **Total** | **2.5 weeks** | **Full integration** |

---

## Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| True positive rate | ~95% | >98% |
| False positive rate | ~2% | <1% |
| GlassWorm detection | ~100% | 100% |
| Behavioral detection | Limited | Comprehensive |
| Context awareness | None | Full |

---

## Risks & Mitigations

### Risk 1: Increased False Positives

**Mitigation:**
- Conservative multipliers initially
- Extensive false positive testing
- Gradual rollout

### Risk 2: Performance Impact

**Mitigation:**
- Benchmark each new detector
- Optimize hot paths
- Cache context calculations

### Risk 3: Complexity

**Mitigation:**
- Modular design
- Comprehensive documentation
- Unit tests for each component

---

## Next Steps

1. ✅ Review and approve this plan
2. ⏳ Implement Phase 1 (Severity Scoring)
3. ⏳ Implement Phase 2 (Detector Patterns)
4. ⏳ Implement Phase 3 (LLM Enhancement)
5. ⏳ Integration testing
6. ⏳ Documentation update
7. ⏳ Release v0.9.0.0

---

**Ready to begin implementation upon approval.**

---

**End of Plan**
