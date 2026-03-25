# Autoresearch Strategy Pivot

**Date:** March 25, 2026
**Status:** 🟡 **RETHINKING APPROACH**
**Inspired by:** karpathy/autoresearch, davebcn87/pi-autoresearch

---

## Key Insights from karpathy/autoresearch

### The Real Approach (Simplified)

**karpathy/autoresearch has ONLY 3 files:**

```
autoresearch/
├── prepare.py       # Fixed environment (data, tokenizer, eval) - NEVER MODIFIED
├── train.py         # Modifiable code (model, optimizer, hyperparams) - EDITED BY LLM
└── program.md       # Natural language instructions for the LLM agent
```

### How It Actually Works

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    KARPATHY'S AUTORESEARCH LOOP                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. HUMAN WRITES program.md (English instructions):                         │
│     "Try to improve validation loss. You can modify:                       │
│      - Model architecture (attention patterns, layers)                     │
│      - Hyperparameters (batch size, learning rate)                         │
│      - Optimizer settings                                                   │
│      Everything else is fixed."                                             │
│                                                                             │
│  2. LLM AGENT reads program.md and:                                        │
│     - Edits train.py (ONE change at a time)                                 │
│     - Commits to git                                                        │
│     - Runs training for 5 minutes                                           │
│     - Extracts val_bpb (validation bits per byte)                           │
│                                                                             │
│  3. DECISION:                                                               │
│     - If val_bpb improved → KEEP commit                                    │
│     - If val_bpb worse/same → REVERT (git reset)                           │
│                                                                             │
│  4. REPEAT (~12 experiments/hour, ~100/night)                              │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### The KEY Difference

**karpathy does NOT tune parameters directly.**

Instead:
1. **LLM reads English instructions** (program.md)
2. **LLM decides what code to change** (train.py)
3. **LLM runs experiment and evaluates**
4. **LLM keeps or reverts based on metric**

**The "parameter" is implicit** - it's whatever the LLM decides to change in the code!

---

## How This Applies to Glassworks

### Current Approach (Wrong)

```
❌ We try to tune 11 parameters directly:
   - malicious_threshold = 6.5, 6.75, 7.0, ...
   - suspicious_threshold = 3.0, 3.25, 3.5, ...
   - reputation_tier_1 = 0.2, 0.25, 0.3, ...
   - etc.

❌ Problems:
   - Only 2 of 11 parameters implemented
   - Parameters are independent (miss interactions)
   - Grid search is inefficient
   - 100% FP rate persists (detectors too sensitive)
```

### New Approach (karpathy-style)

```
✅ Single "meta-parameter" optimized by LLM:
   - The LLM decides what to change in scoring.rs
   - Based on English instructions in program.md
   - Evaluates based on F1 score
   - Keeps or reverts changes

✅ Structure:
   glassworks-autoresearch/
   ├── glassware-core/        # Fixed (detectors, findings)
   ├── glassware/src/scoring.rs  # Modifiable (scoring logic)
   └── program.md             # Instructions for LLM
```

---

## Proposed Glassworks Autoresearch Architecture

### Three-Layer Structure (Like karpathy)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         GLASSWORKS AUTORESEARCH                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Layer 1: FIXED ENVIRONMENT (prepare.py equivalent)                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ glassware-core/                                                      │   │
│  │ - Detectors (invisible, homoglyph, bidi, etc.)                      │   │
│  │ - Finding generation                                                 │   │
│  │ - Evidence + clean package benchmarks                                │   │
│  │ NEVER MODIFIED by autoresearch loop                                 │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  Layer 2: MODIFIABLE CODE (train.py equivalent)                            │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ glassware/src/scoring.rs (or scoring_config.rs)                     │   │
│  │ - calculate_threat_score() function                                 │   │
│  │ - apply_category_caps()                                             │   │
│  │ - apply_reputation_multiplier()                                     │   │
│  │ - All scoring logic                                                  │   │
│  EDITED BY LLM in autoresearch loop                                    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  Layer 3: INSTRUCTIONS (program.md equivalent)                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ AUTORESEARCH-INSTRUCTIONS.md                                        │   │
│  │ - "Reduce false positive rate while maintaining detection"          │   │
│  │ - "You can modify: scoring thresholds, category caps, reputation"   │   │
│  │ - "Target: FP rate ≤5%, detection ≥90%"                             │   │
│  │ - "Current problem: Popular packages (react, prisma) flagged"       │   │
│  WRITTEN BY HUMAN, read by LLM                                         │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### The Optimization Loop

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         GLASSWORKS OPTIMIZATION LOOP                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. BASELINE:                                                               │
│     - Run glassware on benchmark (76 clean + 23 evidence)                  │
│     - Measure: FP rate, detection rate, F1 score                           │
│     - Current: FP=100%, Detection=100%, F1=0.48                            │
│                                                                             │
│  2. LLM AGENT (external, e.g., Claude via API):                            │
│     - Reads AUTORESEARCH-INSTRUCTIONS.md                                   │
│     - Reads current scoring.rs                                             │
│     - Reads benchmark results (autoresearch.jsonl)                         │
│     - Proposes code change to scoring.rs                                  │
│                                                                             │
│  3. EXPERIMENT:                                                             │
│     - Apply code change (git commit)                                       │
│     - Build: cargo build --release                                         │
│     - Run benchmark: glassware scan-tarball <packages>                     │
│     - Extract metrics: FP rate, detection, F1                              │
│                                                                             │
│  4. DECISION:                                                               │
│     - If F1 improved AND FP ≤5% AND detection ≥90% → KEEP                 │
│     - Else → REVERT (git reset)                                            │
│                                                                             │
│  5. REPEAT (~10-20 experiments/hour)                                       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## The Single Meta-Parameter

Instead of tuning 11 parameters independently, we optimize:

**`F1_SCORE` = 2 × (Precision × Recall) / (Precision + Recall)**

Where:
- **Precision** = 1 - FP_rate (how many flagged are actually malicious)
- **Recall** = Detection_rate (how many malicious are caught)

**The LLM's job:** Modify scoring.rs to maximize F1_SCORE while satisfying:
- FP_rate ≤ 5% (hard constraint)
- Detection_rate ≥ 90% (hard constraint)

**Why this works:**
- F1 balances precision and recall (can't game one without other)
- LLM can explore creative solutions (not just parameter tuning)
- Code changes can capture interactions (e.g., "lower threshold BUT increase reputation bonus")

---

## Implementation Plan

### Phase 1: Infrastructure (2-3 hours)

**Create:**
1. `AUTORESEARCH-INSTRUCTIONS.md` - English instructions for LLM
2. `scripts/run-benchmark.sh` - Runs benchmark, outputs F1 score
3. `scripts/llm-agent.py` - Python script that:
   - Calls LLM API (Claude/GPT-4)
   - Proposes code changes
   - Runs benchmark
   - Decides keep/revert

**Structure:**
```python
# scripts/llm-agent.py (simplified)
import openai  # or anthropic
import subprocess
import json

def main():
    # Read instructions
    with open("AUTORESEARCH-INSTRUCTIONS.md") as f:
        instructions = f.read()
    
    # Read current scoring.rs
    with open("glassware/src/scoring.rs") as f:
        current_code = f.read()
    
    # Read benchmark history
    with open("output/autoresearch/autoresearch.jsonl") as f:
        history = [json.loads(line) for line in f]
    
    # Ask LLM for code change
    response = openai.ChatCompletion.create(
        model="gpt-4",
        messages=[
            {"role": "system", "content": instructions},
            {"role": "user", "content": f"""
Current scoring.rs:
{current_code}

Benchmark history (last 5):
{json.dumps(history[-5:], indent=2)}

Propose a code change to improve F1 score while keeping FP ≤5% and detection ≥90%.
Return ONLY the modified function(s).
"""}
        ]
    )
    
    proposed_change = response.choices[0].message.content
    
    # Apply change, build, run benchmark
    apply_change(proposed_change)
    subprocess.run(["cargo", "build", "--release"])
    result = run_benchmark()
    
    # Decide keep/revert
    if result.f1 > best_f1 and result.fp_rate <= 0.05:
        commit_change(result)
        print(f"✓ Kept: F1={result.f1:.3f}")
    else:
        revert_change()
        print(f"✗ Reverted: F1={result.f1:.3f}")
```

### Phase 2: Test Run (1 hour)

**Run:**
```bash
python scripts/llm-agent.py --iterations 10
```

**Expected:**
- 10 iterations (~30-60 minutes)
- Some kept, some reverted
- F1 score should improve gradually

### Phase 3: Full Run (overnight)

**Run:**
```bash
python scripts/llm-agent.py --iterations 100
```

**Expected:**
- 100 iterations (~5-10 hours)
- Converge to local optimum
- Final F1 ≥ 0.90, FP ≤ 5%

---

## Comparison: Old vs. New Approach

| Aspect | Old (11 parameters) | New (LLM + code changes) |
|--------|---------------------|--------------------------|
| **What's optimized** | 11 independent parameters | Scoring logic (code) |
| **How** | Grid search + random sampling | LLM proposes code changes |
| **Flexibility** | Limited to parameter ranges | Unlimited (any code change) |
| **Interactions** | Missed (independent tuning) | Captured (code can combine) |
| **Implementation** | 9 parameters missing | Just need LLM API + script |
| **Time** | 4-6 days | 1 day setup + overnight run |
| **Inspired by** | Traditional hyperparam tuning | karpathy/autoresearch |

---

## Why This Will Work Better

### 1. LLM Understands Context

**Old approach:**
```
malicious_threshold = 7.5  # Why 7.5? No idea, just testing
```

**New approach:**
```rust
// LLM proposes:
fn calculate_threat_score(&self, findings: &[Finding]) -> f32 {
    // Popular packages (100K+ downloads) get 50% score reduction
    // This addresses FP in react, lodash, etc.
    if self.package_context.weekly_downloads > 100_000 {
        base_score *= 0.5;
    }
    // ... rest of logic
}
```

**LLM can write context-aware logic**, not just tune numbers!

### 2. Captures Interactions

**Old approach:** Tests parameters independently
- Test: `malicious_threshold = 7.5` (with default everything else)
- Test: `reputation_tier_1 = 0.3` (with default everything else)

**New approach:** LLM can combine changes
```rust
// LLM learns from history and proposes:
if package.is_popular() && findings.len() < 5 {
    // Lower threshold for popular packages with few findings
    return base_score * 0.4;  // Combines threshold + reputation
}
```

### 3. Creative Solutions

**LLM might discover:**
- "Skip telemetry endpoints from known companies (prisma.io, sentry.io)"
- "CI detection in /scripts/ or /build/ directories is likely legitimate"
- "Blockchain SDKs have >100 findings but are legitimate - use finding count as signal"

**These are CODE changes, not parameter tuning!**

---

## What We Need

### 1. LLM API Access

**Options:**
- **OpenAI (GPT-4):** ~$0.03/1K tokens, ~100 calls = ~$10-20
- **Anthropic (Claude):** Similar pricing
- **Cerebras (existing):** Faster, cheaper, but smaller context

**Recommendation:** Use existing Cerebras for speed, GPT-4 for complex reasoning

### 2. Benchmark Script

Already have most of this - just need to:
- Output F1 score in parseable format
- Include FP rate, detection rate
- Run in <5 minutes

### 3. LLM Agent Script

New Python script (~200 lines):
- Call LLM API
- Parse proposed code change
- Apply to scoring.rs
- Build and test
- Decide keep/revert

### 4. Instructions Document

`AUTORESEARCH-INSTRUCTIONS.md` (~50 lines):
- Goal: Maximize F1, FP ≤5%, detection ≥90%
- What can be modified: scoring.rs functions
- What's fixed: detectors, evidence list
- Current problems: popular packages flagged

---

## Risks & Mitigation

| Risk | Mitigation |
|------|------------|
| LLM makes breaking changes | Rust compiler catches errors, auto-revert |
| LLM gets stuck in local optimum | Random restart every 20 iterations |
| LLM changes are too conservative | Encourage "bold hypotheses" in instructions |
| LLM changes are too aggressive | Require changes to compile + pass basic tests |
| Takes too long | Limit to 5-minute benchmark, 100 iterations max |

---

## Timeline

| Phase | Task | Time |
|-------|------|------|
| **Today** | Stop current run, study karpathy approach | 2 hours |
| **Today** | Create AUTORESEARCH-INSTRUCTIONS.md | 1 hour |
| **Today** | Create run-benchmark.sh | 1 hour |
| **Today** | Create llm-agent.py | 2 hours |
| **Tonight** | Test run (10 iterations) | 1 hour |
| **Tomorrow** | Full run (100 iterations, overnight) | 8 hours |
| **Day 3** | Analyze results, apply best config | 2 hours |
| **Day 4** | Phase A validation | 2 hours |

**Total:** 3-4 days (vs. 4-6 days for old approach)

---

## Decision Required

**Should we:**

**A) Continue current run** (2 parameters, grid search)
- Pros: Already running
- Cons: Limited, may not achieve target

**B) Pivot to LLM-driven approach** (karpathy-style)
- Pros: More flexible, captures interactions, creative solutions
- Cons: Need to implement new infrastructure (4-6 hours)

**C) Hybrid** (finish current test, then pivot)
- Pros: Get data from current run first
- Cons: Delays pivot by ~1 hour

**My recommendation:** **Option C** - Let current test finish (~30 min), then pivot to LLM-driven approach.

---

**Prepared By:** Glassworks Development Agent
**Date:** March 25, 2026
**Status:** 🟡 **AWAITING DECISION**
