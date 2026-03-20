# Adversarial Testing Framework — Specification

**Version:** 0.1.0 (Draft)  
**Date:** 2026-03-20  
**Status:** 📝 SPECIFICATION  
**Effort:** 16 hours  
**Priority:** P3  

---

## Executive Summary

**Problem:** We detect attacks but don't test detector robustness against evasion techniques.

**Solution:** Build an adversarial testing framework that:
1. Mutates known malicious patterns to test detector resilience
2. Fuzzes detector inputs to find blind spots
3. Simulates polymorphic payloads to test generalization
4. Generates evasion test cases automatically

**Impact:** Higher confidence in detector coverage, proactive identification of blind spots.

---

## Background

### Current State

**What We Have:**
- ✅ 17 detectors across 3 tiers
- ✅ 263 passing tests
- ✅ Test fixtures for known attacks (GlassWare waves 1-5)
- ✅ False positive test corpus

**What We're Missing:**
- ❌ Mutation testing (can attackers evade by modifying patterns?)
- ❌ Detector fuzzing (what inputs break detectors?)
- ❌ Polymorphic payload simulation (can attackers generate variants?)
- ❌ Evasion technique catalog (what techniques work?)

### Threat Model

**Attacker Capabilities:**
- Can modify Unicode characters (use different variation selectors)
- Can obfuscate decoder patterns (rename variables, change encoding)
- Can split payloads across files (bypass single-file detectors)
- Can add noise/benign code (dilute signal)
- Can use alternative APIs (different C2 mechanisms)

**Attacker Goals:**
- Evade detection while maintaining functionality
- Minimize code changes (low effort evasion)
- Exploit detector blind spots

---

## Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────────┐
│                  Adversarial Test Runner                     │
│  - Orchestrates mutation, fuzzing, polymorphic generation    │
│  - Runs detectors on mutated payloads                        │
│  - Collects evasion statistics                               │
└─────────────────────────────────────────────────────────────┘
                         │
         ┌───────────────┼───────────────┐
         ▼               ▼               ▼
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│  Mutation   │  │   Fuzzer    │  │  Polymorphic│
│   Engine    │  │   Engine    │  │   Generator │
└─────────────┘  └─────────────┘  └─────────────┘
```

### Component Specifications

#### 1. Mutation Engine

**Purpose:** Systematically modify known malicious patterns to test detector resilience.

**Mutation Types:**

| Type | Description | Example |
|------|-------------|---------|
| **Unicode Substitution** | Replace with equivalent Unicode | VS-16 → VS-17 |
| **Variable Renaming** | Rename decoder variables | `decoder` → `d3c0d3r` |
| **Encoding Variation** | Change encoding scheme | Base64 → Hex |
| **Control Flow** | Restructure control flow | if → switch, loops |
| **Dead Code** | Insert benign code | Add comments, no-ops |
| **API Substitution** | Use alternative APIs | `eval` → `Function` |
| **String Obfuscation** | Obfuscate strings | `"eval"` → `String.fromCharCode(101,118,97,108)` |

**API:**
```rust
pub struct MutationEngine {
    seed_payloads: Vec<MaliciousPayload>,
    mutation_strategies: Vec<Box<dyn MutationStrategy>>,
}

impl MutationEngine {
    pub fn mutate(&self, payload: &MaliciousPayload, rate: f32) -> Vec<MaliciousPayload>;
    pub fn mutate_all(&self, rate: f32) -> Vec<MaliciousPayload>;
}

pub trait MutationStrategy: Send + Sync {
    fn name(&self) -> &str;
    fn mutate(&self, payload: &MaliciousPayload, rng: &mut ThreadRng) -> MaliciousPayload;
}
```

**Test Output:**
```rust
pub struct MutationTestResult {
    pub original_detected: bool,
    pub mutated_detected: bool,
    pub mutation_type: String,
    pub mutation_rate: f32,
    pub evasion_successful: bool,
}
```

---

#### 2. Fuzzer Engine

**Purpose:** Generate random/semi-random inputs to find detector blind spots.

**Fuzzing Strategies:**

| Strategy | Description | Target |
|----------|-------------|--------|
| **Random Unicode** | Insert random Unicode chars | Invisible char detector |
| **Boundary Values** | Test edge cases (max line length, etc.) | All detectors |
| **Malformed Input** | Generate invalid syntax | Parser robustness |
| **Hybrid Patterns** | Mix benign + malicious patterns | Signal dilution |
| **Size Variation** | Vary payload size | Size-based heuristics |

**API:**
```rust
pub struct FuzzerEngine {
    strategies: Vec<Box<dyn FuzzStrategy>>,
    seed_corpus: Vec<String>,
}

impl FuzzerEngine {
    pub fn fuzz(&self, iterations: usize) -> Vec<FuzzResult>;
    pub fn fuzz_with_strategy(&self, strategy: &str, iterations: usize) -> Vec<FuzzResult>;
}

pub struct FuzzResult {
    pub input: String,
    pub strategy: String,
    pub findings: Vec<Finding>,
    pub crash: bool,
    pub timeout: bool,
}
```

---

#### 3. Polymorphic Payload Generator

**Purpose:** Generate variant malicious payloads that maintain functionality but evade detection.

**Generation Techniques:**

| Technique | Description | Complexity |
|-----------|-------------|------------|
| **Template-Based** | Use templates with variable slots | Low |
| **Grammar-Based** | Generate from malicious grammar | Medium |
| **LLM-Guided** | Use LLM to generate variants | High |
| **Genetic Algorithm** | Evolve payloads over generations | High |

**API:**
```rust
pub struct PolymorphicGenerator {
    templates: Vec<PayloadTemplate>,
    grammar: Option<MaliciousGrammar>,
}

impl PolymorphicGenerator {
    pub fn generate(&self, count: usize) -> Vec<MaliciousPayload>;
    pub fn generate_from_template(&self, template: &PayloadTemplate, count: usize) -> Vec<MaliciousPayload>;
}

pub struct PayloadTemplate {
    pub name: String,
    pub description: String,
    pub slots: Vec<Slot>,
    pub base_payload: String,
}
```

---

#### 4. Evasion Test Case Generator

**Purpose:** Automatically generate test cases from successful evasions.

**Output Format:**
```rust
pub struct EvasionTestCase {
    pub name: String,
    pub description: String,
    pub original_payload: MaliciousPayload,
    pub evasion_payload: MaliciousPayload,
    pub mutation_applied: String,
    pub detectors_evaded: Vec<String>,
    pub severity: EvasionSeverity,
}

pub enum EvasionSeverity {
    Critical,  // Evades all detectors
    High,      // Evades Tier 1-2 detectors
    Medium,    // Evades some Tier 3 detectors
    Low,       // Evades single detector
}
```

---

## Implementation Plan

### Phase 1: Mutation Engine (6h)

**Tasks:**
1. Define `MutationStrategy` trait
2. Implement 7 mutation strategies (Unicode, Variable, Encoding, etc.)
3. Build `MutationEngine` orchestrator
4. Add test runner
5. Generate mutation test report

**Files to Create:**
- `glassware-core/src/adversarial/mutation.rs`
- `glassware-core/src/adversarial/strategies/` (7 files)
- `glassware-core/src/adversarial/runner.rs`

**Tests:**
- Unit tests for each mutation strategy
- Integration tests for mutation engine
- Evasion rate benchmarks

---

### Phase 2: Fuzzer Engine (4h)

**Tasks:**
1. Define `FuzzStrategy` trait
2. Implement 5 fuzzing strategies
3. Build `FuzzerEngine` orchestrator
4. Add crash/timeout detection
5. Generate fuzzing report

**Files to Create:**
- `glassware-core/src/adversarial/fuzzer.rs`
- `glassware-core/src/adversarial/fuzz_strategies/` (5 files)

**Tests:**
- Unit tests for each fuzz strategy
- Integration tests for fuzzer engine
- Coverage reports

---

### Phase 3: Polymorphic Generator (4h)

**Tasks:**
1. Define payload templates (GlassWare, PhantomRaven, ForceMemo)
2. Implement template-based generation
3. Build `PolymorphicGenerator`
4. Generate polymorphic test corpus
5. Validate polymorphic payloads

**Files to Create:**
- `glassware-core/src/adversarial/polymorphic.rs`
- `glassware-core/src/adversarial/templates/` (5-10 templates)

**Tests:**
- Template validation tests
- Polymorphic payload tests
- Detector coverage tests

---

### Phase 4: Evasion Test Generator (2h)

**Tasks:**
1. Collect successful evasions from mutation/fuzzer
2. Generate test cases automatically
3. Export to test fixtures
4. Integrate with CI/CD

**Files to Create:**
- `glassware-core/src/adversarial/test_generator.rs`
- `glassware-core/tests/fixtures/evasion/` (auto-generated)

**Tests:**
- Test case validation
- CI/CD integration tests

---

## Success Metrics

### Quantitative

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Mutation Coverage** | >90% | % of mutations detected |
| **Fuzzer Coverage** | >80% | % of input space covered |
| **Polymorphic Detection** | >85% | % of variants detected |
| **Evasion Rate** | <10% | % of successful evasions |
| **Test Cases Generated** | >100 | Auto-generated test cases |

### Qualitative

- ✅ Detector blind spots identified
- ✅ Evasion techniques cataloged
- ✅ Test corpus expanded
- ✅ CI/CD integration working
- ✅ Documentation complete

---

## Integration with Existing Systems

### Test Integration

```rust
// In tests/adversarial.rs
#[test]
fn test_mutation_evasion_rate() {
    let engine = MutationEngine::new();
    let results = engine.mutate_all(0.1);  // 10% mutation rate
    
    let evasion_rate = results.iter()
        .filter(|r| r.evasion_successful)
        .count() as f32 / results.len() as f32;
    
    assert!(evasion_rate < 0.10, "Evasion rate too high: {}", evasion_rate);
}
```

### CI/CD Integration

```yaml
# .github/workflows/adversarial.yml
name: Adversarial Testing

on: [push, pull_request]

jobs:
  adversarial:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Run mutation testing
        run: cargo test --features adversarial -- mutation_evasion_rate
      
      - name: Run fuzzing
        run: cargo test --features adversarial -- fuzz_coverage
      
      - name: Check evasion rate
        run: |
          evasion_rate=$(cargo run --bin adversarial-report)
          if (( $(echo "$evasion_rate > 0.10" | bc -l) )); then
            echo "Evasion rate too high: $evasion_rate"
            exit 1
          fi
```

---

## Risks and Mitigations

### Risk 1: False Sense of Security

**Risk:** Passing adversarial tests doesn't guarantee real-world robustness.

**Mitigation:**
- Use real-world evasion techniques from literature
- Continuously update mutation strategies
- Combine with manual security review

### Risk 2: Performance Overhead

**Risk:** Adversarial testing slows down CI/CD.

**Mitigation:**
- Run full suite nightly, not on every PR
- Cache mutation results
- Parallelize test execution

### Risk 3: Arms Race

**Risk:** Attackers adapt to our adversarial testing.

**Mitigation:**
- Regularly add new mutation strategies
- Monitor real-world evasion techniques
- Update test corpus quarterly

---

## Timeline

| Phase | Tasks | Effort | Dependencies |
|-------|-------|--------|--------------|
| **Phase 1: Mutation** | 5 tasks | 6h | None |
| **Phase 2: Fuzzer** | 5 tasks | 4h | Phase 1 |
| **Phase 3: Polymorphic** | 5 tasks | 4h | Phase 1-2 |
| **Phase 4: Test Gen** | 4 tasks | 2h | Phase 1-3 |
| **Total** | 19 tasks | 16h | - |

---

## Open Questions

1. **Should we use existing fuzzing libraries?** (e.g., `cargo-fuzz`)
   - Pro: Less code to maintain
   - Con: Less control over strategies

2. **Should LLM-guided generation be v1.0 or v2.0?**
   - Pro: More sophisticated variants
   - Con: Adds complexity, API key dependency

3. **Should evasion test cases be committed to repo?**
   - Pro: Version controlled, reproducible
   - Con: Repo bloat, potential misuse

---

## Recommendations

### v1.0 Scope (Recommended)

**Include:**
- ✅ Mutation engine with 7 strategies
- ✅ Fuzzer engine with 5 strategies
- ✅ Template-based polymorphic generation
- ✅ Automatic test case generation
- ✅ CI/CD integration

**Defer to v2.0:**
- ⏳ LLM-guided generation
- ⏳ Genetic algorithm evolution
- ⏳ Real-time mutation strategy learning

### Success Criteria for v1.0

- ✅ Evasion rate <10% on known attacks
- ✅ 100+ auto-generated test cases
- ✅ CI/CD integration working
- ✅ Documentation complete

---

**Status:** 📝 SPECIFICATION COMPLETE  
**Next:** Review spec, get approval, start implementation

**Timestamp:** 2026-03-20 13:00 UTC  
**Author:** glassware AI Assistant
