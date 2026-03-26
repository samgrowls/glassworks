# Glassworks LLM Integration

**Version:** 0.34.0
**Last Updated:** March 24, 2026

---

## Overview

Glassworks implements a **multi-stage LLM pipeline** for security analysis, using different models for different stages of triage and analysis.

**Phase 5 Implementation:** 2026-03-24

---

## Multi-Stage Pipeline

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    MULTI-STAGE LLM PIPELINE                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Stage 1: TRIAGE (Cerebras - Fast, ~2s/pkg)                             │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │ Purpose: Identify obvious false positives                        │    │
│  │ Input: Package name, findings, code snippets                     │    │
│  │ Output: FP probability, skip recommendation                      │    │
│  │ Exit Condition: confidence < 0.25 = likely FP, stop here         │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                              │                                           │
│                              ▼                                           │
│  Stage 2: ANALYSIS (NVIDIA - Medium, ~15s/pkg)                          │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │ Purpose: Explain attack chain, assess severity                   │    │
│  │ Input: All findings, full context                                │    │
│  │ Output: Attack explanation, confidence score, severity adjust    │    │
│  │ Exit Condition: confidence ≥ 0.75 = confident, stop here         │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                              │                                           │
│                              ▼                                           │
│  Stage 3: DEEP DIVE (NVIDIA - Slow, ~30s/pkg)                           │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │ Purpose: Manual review assistance for borderline cases           │    │
│  │ Input: Score 4.0-7.0 packages, full code context                 │    │
│  │ Output: Detailed analysis, remediation suggestions               │    │
│  │ Trigger: threat_score between 4.0 and 7.0                        │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Stage Details

#### Stage 1: Triage (Fast FP Identification)

**Provider:** Cerebras API
**Model:** `llama-3.3-70b`
**Timeout:** 30 seconds
**Max Tokens:** 512

**Purpose:**
- Quickly identify obvious false positives
- Skip further analysis for clear FPs (confidence < 0.25)
- Reduce API costs by avoiding unnecessary deep analysis

**Exit Conditions:**
- `confidence < 0.25` → Stop, likely FP
- `confidence ≥ 0.25` → Continue to Stage 2

---

#### Stage 2: Analysis (Attack Chain Explanation)

**Provider:** NVIDIA API
**Models (in order):**
1. `qwen/qwen3.5-397b-a17b` (397B - strongest)
2. `moonshotai/kimi-k2.5`
3. `z-ai/glm5`
4. `meta/llama-3.3-70b-instruct` (fallback)

**Timeout:** 120 seconds
**Max Tokens:** 2048

**Purpose:**
- Analyze attack chains
- Provide detailed explanations
- Assess confidence level

**Exit Conditions:**
- `confidence ≥ 0.75` → Stop, confident verdict
- `confidence < 0.75` → Continue to Stage 3 (if borderline)

---

#### Stage 3: Deep Dive (Borderline Cases)

**Provider:** NVIDIA API
**Models:** Same as Stage 2
**Timeout:** 120 seconds
**Max Tokens:** 2048

**Purpose:**
- Analyze borderline cases (threat score 4.0-7.0)
- Provide detailed remediation suggestions
- Assist manual review

**Trigger:**
- `threat_score >= 4.0 && threat_score <= 7.0`

---

## Configuration

### Environment Variables

```bash
# Tier 1 LLM (Cerebras - fast triage)
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="csk-..."
export GLASSWARE_LLM_MODEL="llama-3.3-70b"

# Tier 2 LLM (NVIDIA - deep analysis)
export NVIDIA_API_KEY="nvapi-..."

# Optional: Multiple models for fallback
export GLASSWARE_LLM_MODELS="llama-3.3-70b,qwen-3.235b-a22b-instruct-2507"
```

### Programmatic Configuration

```rust
use glassware::llm::{MultiStagePipeline, PipelineBuilder, MultiStagePipelineConfig};

// Option 1: Use builder
let pipeline = PipelineBuilder::new()
    .with_triage(true)
    .with_analysis(true)
    .with_deep_dive(true)
    .with_score_thresholds(4.0, 7.0)
    .build()?;

// Option 2: Use preset configs
let config = MultiStagePipelineConfig::triage_only();  // Fast scanning
let config = MultiStagePipelineConfig::standard();     // Triage + analysis
let config = MultiStagePipelineConfig::deep_scan();    // All stages

let pipeline = MultiStagePipeline::with_config(config)?;
```

### Config Presets

| Preset | Stages | Use Case | Speed |
|--------|--------|----------|-------|
| `triage_only()` | Stage 1 only | Fast scanning, high FP tolerance | ~2s/pkg |
| `standard()` | Stage 1 + 2 | Balanced scanning | ~15s/pkg |
| `deep_scan()` | All 3 stages | Thorough analysis, borderline cases | ~30-50s/pkg |

---

## Usage

### Basic Usage

```rust
use glassware::llm::{MultiStagePipeline, LlmFinding};
use glassware_core::Finding;

// Create pipeline
let pipeline = MultiStagePipeline::new()?;

// Convert findings
let findings: Vec<LlmFinding> = core_findings
    .iter()
    .map(LlmFinding::from)
    .collect();

// Run pipeline
let result = pipeline.run(&findings, threat_score).await?;

// Check verdict
if result.verdict.is_malicious && result.verdict.confidence >= 0.75 {
    println!("Likely malicious: {}", result.verdict.explanation);
} else if result.verdict.confidence < 0.25 {
    println!("Likely FP: {}", result.verdict.false_positive_indicators.join(", "));
} else {
    println!("Uncertain - manual review recommended");
}
```

### Advanced Usage

```rust
// Run with specific config
let config = MultiStagePipelineConfig::deep_scan();
let pipeline = MultiStagePipeline::with_config(config)?;

let result = pipeline.run(&findings, threat_score).await?;

// Inspect stage results
println!("Pipeline completed at stage: {}", result.stage);
println!("Triage run: {}, Analysis run: {}, Deep dive run: {}", 
    result.triage_run, result.analysis_run, result.deep_dive_run);
println!("Total time: {}ms", result.total_time_ms);
println!("Models used: {:?}", result.models_used);

// Access individual stage verdicts
if let Some(triage) = &result.triage_verdict {
    println!("Triage: confidence={:.2}", triage.confidence);
}
if let Some(analysis) = &result.analysis_verdict {
    println!("Analysis: confidence={:.2}", analysis.confidence);
}
if let Some(deep_dive) = &result.deep_dive_verdict {
    println!("Deep dive: confidence={:.2}", deep_dive.confidence);
}
```

---

## Response Format

### PipelineResult

```rust
pub struct PipelineResult {
    pub verdict: LlmVerdict,           // Final verdict
    pub stage: u8,                     // Stage that produced verdict (1-3)
    pub triage_run: bool,              // Whether triage was run
    pub analysis_run: bool,            // Whether analysis was run
    pub deep_dive_run: bool,           // Whether deep dive was run
    pub triage_verdict: Option<LlmVerdict>,
    pub analysis_verdict: Option<LlmVerdict>,
    pub deep_dive_verdict: Option<LlmVerdict>,
    pub total_time_ms: u64,            // Total pipeline time
    pub models_used: Vec<String>,      // Models used at each stage
}
```

### LlmVerdict

```rust
pub struct LlmVerdict {
    pub is_malicious: bool,            // Likely malicious?
    pub confidence: f32,               // 0.0-1.0 confidence
    pub explanation: String,           // Detailed explanation
    pub recommendations: Vec<String>,  // Recommended actions
    pub false_positive_indicators: Vec<String>,  // FP indicators
}
```

---

## Confidence Guidelines

| Confidence | Interpretation | Action |
|------------|----------------|--------|
| **0.0-0.25** | Very likely FP | Skip further analysis, mark as clean |
| **0.25-0.50** | Likely FP | Review if other signals present |
| **0.50-0.75** | Uncertain | Manual review recommended |
| **0.75-1.0** | Likely malicious | Block, investigate, report |

---

## Caching

LLM results are cached in memory to avoid redundant API calls.

**Cache Key:** Hash of findings input

**Cache Behavior:**
- Enabled by default
- Can be disabled via `config.cache_enabled = false`
- Cache is cleared on process restart

**Future Enhancement:** Disk-based persistent cache

---

## Performance

### Expected Latency

| Stage | Provider | Model | Avg Time |
|-------|----------|-------|----------|
| Triage | Cerebras | llama-3.3-70b | ~2s |
| Analysis | NVIDIA | qwen3.5-397b | ~15s |
| Deep Dive | NVIDIA | qwen3.5-397b | ~30s |

### Total Pipeline Time

| Config | Avg Time | Max Time |
|--------|----------|----------|
| `triage_only()` | ~2s | ~30s (timeout) |
| `standard()` | ~15s | ~60s (2x timeout) |
| `deep_scan()` | ~30-50s | ~90s (3x timeout) |

---

## Error Handling

### Fallback Behavior

- If Stage 1 fails → Continue to Stage 2
- If Stage 2 fails → Continue to Stage 3 (if enabled)
- If all stages fail → Return default verdict with `confidence: 0.0`

### Error Types

```rust
pub enum OrchestratorError {
    Llm(String),           // LLM API error
    ConfigError(String),   // Configuration error
    Timeout,               // Request timeout
    RateLimit,             // API rate limit
}
```

---

## Testing

### Unit Tests

```bash
cargo test -p glassware llm::pipeline_tests
```

### Integration Tests

```bash
# Test with real API calls (requires API keys)
cargo test --features llm integration::llm_pipeline
```

---

## Troubleshooting

### API Key Issues

```bash
# Verify API keys are set
echo $GLASSWARE_LLM_API_KEY
echo $NVIDIA_API_KEY

# Test Cerebras API
curl -X POST https://api.cerebras.ai/v1/chat/completions \
  -H "Authorization: Bearer $GLASSWARE_LLM_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"model":"llama-3.3-70b","messages":[{"role":"user","content":"test"}]}'
```

### Timeout Issues

```rust
// Increase timeout
let mut config = MultiStagePipelineConfig::default();
if let Some(ref mut triage) = config.triage_config {
    triage.timeout_secs = 60;  // Increase from 30 to 60
}
```

### Rate Limiting

If hitting rate limits:
1. Enable caching (`config.cache_enabled = true`)
2. Reduce concurrency in campaign config
3. Use `triage_only()` mode for initial scanning

---

## References

- `glassware/src/llm.rs` - Main implementation
- `docs/DETECTION.md` - Detector reference
- `docs/SCORING.md` - Scoring system
- PROMPT.md Phase 5 - Implementation specification

---

## Future Enhancements

1. **Persistent Cache:** Disk-based caching across sessions
2. **Model Fine-tuning:** Fine-tune models on GlassWare dataset
3. **Conversational Mode:** Multi-turn Q&A about findings
4. **Batch Analysis:** Analyze multiple packages in single API call
5. **Provider Auto-selection:** Automatically select best provider based on latency
