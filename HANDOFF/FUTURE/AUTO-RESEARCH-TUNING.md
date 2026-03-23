# Auto-Research & Detector Tuning Design

**Date:** March 23, 2026  
**Status:** Design Proposal  
**Author:** Security Research Team  

---

## Executive Summary

This document explores **automated detector tuning** and **threat research** capabilities for Glassworks, inspired by two reference projects:

1. **YPi** (rawwerks/ypi) - Recursive self-delegation for code intelligence
2. **PI Auto-Research** (davebcn87/pi-autoresearch) - Autonomous experiment loops with statistical validation

We analyze viable approaches for automating detector configuration tuning, reducing false positives, and discovering new threat patterns through LLM-assisted research.

---

## 1. Analysis of Referenced Projects

### 1.1 YPi (Recursive Language Models)

**Repository:** https://github.com/rawwerks/ypi  
**Stars:** 200 | **Languages:** Shell (63.5%), TypeScript (35.8%)

#### Architecture

YPi implements a **recursive self-delegation** pattern where an AI agent can spawn child agents to solve sub-problems:

```
┌──────────────────────────────────────────┐
│  ypi (depth 0)                           │
│  Tools: bash, rlm_query                  │
│  Workspace: default                      │
│                                          │
│  > grep -n "bug" src/*.py                │
│  > sed -n '50,80p' src/app.py \          │
│      | rlm_query "Fix this bug"          │
│            │                             │
│            ▼                             │
│    ┌────────────────────────────┐        │
│    │  ypi (depth 1)            │        │
│    │  Workspace: jj isolated   │        │
│    │  Edits files safely       │        │
│    │  Returns: patch on stdout │        │
│    └────────────────────────────┘        │
│                                          │
│  > jj squash --from <child-change>       │
│  # absorb the fix into working copy      │
└──────────────────────────────────────────┘
```

#### Key Features

| Feature | Description |
|---------|-------------|
| **Recursive Self-Delegation** | Agent calls itself via `rlm_query` to decompose problems |
| **jj Workspace Isolation** | Each recursive child gets isolated workspace via `jj` VCS |
| **Budget/Timeout Controls** | Configurable spend limits and wall-clock timeouts |
| **Five Guardrails** | Budget, timeout, call limit, model routing, depth limit guarantee termination |
| **Self-Hosting** | Agent contains its own source code and can modify recursion logic |

#### How Auto-Research Works

The system follows a **size-first → search → chunk → delegate → combine** pattern:

1. **Size Assessment:** Agent evaluates problem scope
2. **Search:** Uses `grep`, `cat`, `sed` to explore codebase
3. **Chunk:** Breaks problem into sub-tasks
4. **Delegate:** Calls `rlm_query` for each sub-task (spawns child Pi process)
5. **Combine:** Aggregates child outputs, applies patches

#### Learning/Adaptation Mechanism

**Not ML-based** — adaptation happens through:
- **Self-Modification:** Agent can edit `rlm_query` source (in its own context)
- **Decomposition Intelligence:** Universal pattern works at every scale
- **File-Based Memory:** Uses `$CONTEXT`, `$RLM_PROMPT_FILE`, and `jj` workspaces
- **Session Persistence:** Child sessions get trace-encoded filenames

#### Relevance to Glassworks

| YPi Feature | Applicability to Glassworks |
|-------------|----------------------------|
| Recursive delegation | ⭐⭐⭐ High - Could delegate package analysis to child agents |
| Workspace isolation | ⭐⭐ Medium - Useful for safe malware analysis |
| Budget controls | ⭐⭐⭐ High - Essential for cost-controlled LLM usage |
| Self-modification | ⭐ Low - Too risky for security tooling |
| File-based context | ⭐⭐⭐ High - Already using evidence collection |

---

### 1.2 PI Auto-Research

**Repository:** https://github.com/davebcn87/pi-autoresearch  
**Stars:** 2.8k | **Languages:** TypeScript 100%

#### Architecture

Domain-agnostic infrastructure (extension) + domain knowledge (skill) = unlimited domain support:

```
┌──────────────────────┐     ┌──────────────────────────┐
│  Extension (global)  │     │  Skill (per-domain)       │
│                      │     │                           │
│  run_experiment      │◄────│  command: pnpm test       │
│  log_experiment      │     │  metric: seconds (lower)  │
│  widget + dashboard  │     │  scope: vitest configs    │
│                      │     │  ideas: pool, parallel…   │
└──────────────────────┘     └──────────────────────────┘
```

#### The Autonomous Loop

```
edit code → commit → run_experiment → log_experiment → keep/revert → repeat
```

#### Key Features

| Feature | Description |
|---------|-------------|
| **Autonomous Experiment Loop** | Edit → commit → run → log → keep/revert → repeat |
| **Live Widget** | Status display showing runs, metric, confidence |
| **Dashboard** | Full results table and fullscreen overlay |
| **Session Persistence** | Survives restarts via `autoresearch.jsonl` + `autoresearch.md` |
| **Confidence Scoring** | Statistical significance using MAD (Median Absolute Deviation) |
| **Backpressure Checks** | Optional correctness validation (tests, types, lint) |
| **Branch-Aware** | Each branch maintains its own session |

#### Confidence Scoring Algorithm

**MAD-based (Median Absolute Deviation):**

```
Confidence = |best_improvement| / MAD
```

| Score | Color | Meaning |
|-------|-------|---------|
| ≥ 2.0× | 🟢 Green | Improvement likely real |
| 1.0–2.0× | 🟡 Yellow | Above noise but marginal |
| < 1.0× | 🔴 Red | Within noise — re-run to confirm |

**Characteristics:**
- Activates after 3+ experiments
- Robust to outliers (uses MAD, not standard deviation)
- Persisted per run
- Advisory only — guides agent but doesn't auto-discard

#### Session Files

| File | Purpose |
|------|---------|
| `autoresearch.md` | Living document: objective, what's tried, dead ends, wins |
| `autoresearch.sh` | Benchmark script with `METRIC name=number` output |
| `autoresearch.checks.sh` | Optional correctness checks (tests, types, lint) |
| `autoresearch.jsonl` | Append-only log: metric, status, commit, description per run |

#### Relevance to Glassworks

| PI Auto-Research Feature | Applicability to Glassworks |
|--------------------------|----------------------------|
| Autonomous experiment loop | ⭐⭐⭐ High - Perfect for detector tuning |
| MAD confidence scoring | ⭐⭐⭐ High - Statistical validation of tuning changes |
| Session persistence | ⭐⭐⭐ High - Long-running tuning sessions |
| Backpressure checks | ⭐⭐⭐ High - Ensure tuning doesn't break detection |
| Branch-aware sessions | ⭐⭐ Medium - Could use for A/B testing detectors |

---

## 2. Auto-Research Concepts for Security Detection

### 2.1 Automated Threat Pattern Discovery

**Goal:** Automatically discover new threat patterns from analyzed packages.

#### Approach

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│ Package Scan    │────►│ LLM Analysis    │────►│ Pattern         │
│ Results         │     │ (Cluster)       │     │ Extraction      │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                                        │
                                                        ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│ Detector        │◄────│ Human Review    │◄────│ Pattern         │
│ Generation      │     │ (Optional)      │     │ Clustering      │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

#### Implementation Strategy

1. **Collect findings** from scanned packages (both malicious and clean)
2. **Cluster similar findings** using embedding similarity
3. **LLM pattern extraction:** "What do these 50 packages with invisible chars have in common?"
4. **Generate detector rules** from extracted patterns
5. **Validate** against known clean packages (backpressure check)
6. **Deploy** new detector rules

#### Data Requirements

| Data Type | Minimum | Recommended | Source |
|-----------|---------|-------------|--------|
| Malicious packages | 100 | 1,000+ | npm security advisories, curated datasets |
| Clean packages | 500 | 10,000+ | Top npm packages, verified publishers |
| Labeled findings | 500 | 5,000+ | Historical scan results with human review |

---

### 2.2 Feedback Loop from Findings to Detector Tuning

**Goal:** Automatically adjust detector weights/thresholds based on false positive/negative analysis.

#### Current Glassworks Architecture

From `/home/property.sightlines/samgrowls/glassworks/glassware-core/src/risk_scorer.rs`:

```rust
pub const RISK_THRESHOLD_LOW: u32 = 10;
pub const RISK_THRESHOLD_MEDIUM: u32 = 25;
pub const RISK_THRESHOLD_HIGH: u32 = 50;
pub const RISK_THRESHOLD_CRITICAL: u32 = 100;

// Contextual multipliers
- Ecosystem: npm (1.0), PyPI (1.2), GitHub (1.5)
- Package type: Library (1.0), CLI (1.3), Extension (1.5)
- Novelty: <7 days (1.5), <30 days (1.2), >=30 days (1.0)
- Reputation: Known publisher (0.8), Unknown (1.0)
- File type: Minified (0.5), Source (1.0)
```

From `/home/property.sightlines/samgrowls/glassworks/WAVE6-FIXES.md`:

```toml
malicious_threshold = 7.0
category_weight = 2.0
critical_weight = 3.0
high_weight = 1.5
```

#### Proposed Feedback Loop

Inspired by PI Auto-Research:

```
┌─────────────────┐
│ Current Config  │
│ thresholds.toml │
└────────┬────────┘
         │
         ▼
┌─────────────────┐     ┌─────────────────┐
│ Run Experiment  │────►│ Measure Metrics │
│ (scan dataset)  │     │ - FP rate       │
└─────────────────┘     │ - FN rate       │
         │              │ - Precision     │
         │              │ - Recall        │
         ▼              └────────┬────────┘
┌─────────────────┐               │
│ Propose Change  │◄──────────────┘
│ (LLM or GA)     │
│ "Increase threshold │
│  from 7.0 to 7.5" │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Statistical     │
│ Validation      │
│ (MAD scoring)   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Keep/Revert     │
│ (auto or human) │
└─────────────────┘
```

#### Metrics to Track

| Metric | Formula | Target |
|--------|---------|--------|
| **False Positive Rate** | FP / (FP + TN) | < 5% |
| **False Negative Rate** | FN / (FN + TP) | < 1% |
| **Precision** | TP / (TP + FP) | > 90% |
| **Recall** | TP / (TP + FN) | > 95% |
| **F1 Score** | 2 × (Precision × Recall) / (Precision + Recall) | > 90% |

---

### 2.3 LLM-Assisted Detector Rule Generation

**Goal:** Use LLMs to generate detector rules from natural language descriptions or example patterns.

#### Approach

```
┌─────────────────┐
│ Security        │
│ Researcher      │
└────────┬────────┘
         │ "Create a detector for
         │ packages that exfiltrate
         │ environment variables to
         │ external URLs"
         ▼
┌─────────────────┐
│ LLM Prompt      │
│ - Detection     │
│   category      │
│ - Example code  │
│ - Known patterns│
│ - FP examples   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Generated Rule  │
│ (Rust code)     │
│ impl Detector   │
│ for NewDetector │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Validation      │
│ - Compile check │
│ - Unit tests    │
│ - FP check      │
└────────┬────────┘
```

#### Prompt Template

```markdown
You are a security detection engineer. Generate a Rust detector implementation.

## Detection Category
{category_description}

## Examples of Malicious Patterns
{malicious_examples}

## Examples of Legitimate Usage (False Positives to Avoid)
{false_positive_examples}

## Existing Detector Interface
{detector_trait_definition}

## Requirements
- Must implement the Detector trait
- Should have low false positive rate
- Should catch the malicious patterns above
- Include unit tests

Generate the detector implementation.
```

#### Validation Pipeline

1. **Compile Check:** Ensure generated code compiles
2. **Unit Test:** Run against known malicious/clean examples
3. **FP Check:** Scan top 100 npm packages, ensure < 5% FP rate
4. **Human Review:** Security researcher validates before merge

---

### 2.4 Clustering Similar Findings for Pattern Analysis

**Goal:** Group similar findings to identify attack campaigns and common patterns.

#### Approach

```
┌─────────────────┐
│ All Findings    │
│ (from scans)    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Embedding       │
│ Generation      │
│ (code2vec,      │
│  CodeBERT)      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Clustering      │
│ (DBSCAN,        │
│  HDBSCAN)       │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Cluster         │
│ Analysis        │
│ - Common        │
│   patterns      │
│ - Outliers      │
│ - Campaign      │
│   detection     │
└─────────────────┘
```

#### Clustering Algorithms

| Algorithm | Use Case | Pros | Cons |
|-----------|----------|------|------|
| **DBSCAN** | Density-based clustering | No need to specify K, finds arbitrary shapes | Struggles with varying densities |
| **HDBSCAN** | Hierarchical DBSCAN | Better with varying densities, hierarchical clusters | More computationally expensive |
| **K-Means** | Simple clustering | Fast, simple | Need to specify K, assumes spherical clusters |
| **Agglomerative** | Hierarchical clustering | Dendrogram visualization, no K needed upfront | O(n³) complexity |

#### Implementation in Rust

```rust
use linfa::clustering::Dbscan;
use linfa::traits::{Fit, Predict};

// Create feature vectors from findings
let findings_vectors = extract_features(&findings);

// Cluster with DBSCAN
let dbscan = Dbscan::new(0.5, 5); // eps=0.5, min_points=5
let model = dbscan.fit(&findings_vectors)?;
let clusters = model.predict(&findings_vectors)?;

// Analyze clusters
for cluster_id in clusters.iter().unique() {
    let cluster_findings = findings.iter()
        .zip(clusters.iter())
        .filter(|(_, c)| **c == cluster_id)
        .map(|(f, _)| f);
    
    // LLM analysis: "What pattern do these findings share?"
    let pattern = llm_analyze_cluster(cluster_findings)?;
}
```

---

### 2.5 False Positive Analysis and Rule Refinement

**Goal:** Systematically analyze false positives to refine detector rules.

#### Process

```
┌─────────────────┐
│ Flagged         │
│ Packages        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Human Review    │
│ (label as       │
│  FP/TN/TP/FN)   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ FP Analysis     │
│ - Which         │
│   detectors     │
│   triggered?    │
│ - What patterns │
│   caused FP?    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Rule Refinement │
│ - Adjust        │
│   thresholds    │
│ - Add           │
│   exceptions    │
│ - Whitelist     │
│   patterns      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Validate        │
│ (re-scan,       │
│  ensure no      │
│  new FNs)       │
└─────────────────┘
```

#### FP Analysis Dashboard

Track per-detector FP rates:

| Detector | Total Findings | False Positives | FP Rate | Trend |
|----------|---------------|-----------------|---------|-------|
| invisible_chars | 1,234 | 12 | 0.97% | ↓ |
| homoglyphs | 567 | 23 | 4.06% | → |
| glassware_pattern | 89 | 2 | 2.25% | ↓ |
| locale_geofencing | 45 | 8 | 17.8% | ↑ (needs tuning) |

#### Automatic Whitelist Generation

```rust
// Analyze false positives
let fp_packages = findings.iter()
    .filter(|f| f.label == FindingLabel::FalsePositive);

// Extract common patterns
let fp_patterns = fp_packages.iter()
    .flat_map(|p| extract_patterns(p))
    .collect::<Vec<_>>();

// LLM generates whitelist rules
let whitelist_rules = llm_generate_whitelist(&fp_patterns)?;

// Validate whitelist doesn't introduce FNs
let fn_check = validate_no_false_negatives(&whitelist_rules, known_malicious)?;

if fn_check.passed {
    apply_whitelist(whitelist_rules);
}
```

---

## 3. Detector Tuning Approaches

### 3.1 Manual Tuning

**Description:** Security researchers manually adjust weights/thresholds based on scan results.

#### Current Glassworks Implementation

```toml
# From WAVE6-FIXES.md
malicious_threshold = 7.0
category_weight = 2.0
critical_weight = 3.0
high_weight = 1.5
```

```rust
# From risk_scorer.rs
pub const RISK_THRESHOLD_LOW: u32 = 10;
pub const RISK_THRESHOLD_MEDIUM: u32 = 25;
pub const RISK_THRESHOLD_HIGH: u32 = 50;
pub const RISK_THRESHOLD_CRITICAL: u32 = 100;
```

#### Pros
- ✅ Full control
- ✅ Explainable decisions
- ✅ No training data needed
- ✅ Immediate feedback

#### Cons
- ❌ Time-consuming
- ❌ Requires expertise
- ❌ Doesn't scale
- ❌ Prone to human bias

#### Viability for Glassworks
**⭐⭐⭐ High** - Current approach, should remain as primary method with automation assistance.

---

### 3.2 Supervised Learning

**Description:** Train ML models on labeled malicious/clean packages to learn optimal thresholds.

#### Approach

```
┌─────────────────┐
│ Labeled Dataset │
│ - Malicious:    │
│   1,000+        │
│ - Clean:        │
│   10,000+       │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Feature         │
│ Extraction      │
│ - Detector      │
│   scores        │
│ - Package       │
│   metadata      │
│ - Context       │
│   features      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Model Training  │
│ - Random        │
│   Forest        │
│ - XGBoost       │
│ - Neural Net    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Threshold       │
│ Optimization    │
│ (ROC analysis)  │
└─────────────────┘
```

#### Feature Vector

```rust
struct DetectionFeatures {
    // Detector scores
    invisible_chars_count: u32,
    homoglyphs_count: u32,
    glassware_patterns: u32,
    locale_suspicion: f32,
    
    // Package metadata
    package_age_days: u32,
    publisher_reputation: f32,
    download_count: u32,
    
    // Context
    ecosystem: Ecosystem,
    package_type: PackageType,
    is_minified: bool,
    
    // Label (for training)
    is_malicious: bool,
}
```

#### Data Requirements

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| Malicious samples | 500 | 5,000+ |
| Clean samples | 2,000 | 50,000+ |
| Feature completeness | 80% | 95%+ |
| Label accuracy | 95% | 99%+ (human verified) |

#### Pros
- ✅ Can learn complex patterns
- ✅ Scalable once trained
- ✅ Objective optimization

#### Cons
- ❌ Requires large labeled dataset
- ❌ Risk of overfitting
- ❌ Black-box decisions
- ❌ Retraining needed for new threats

#### Viability for Glassworks
**⭐⭐ Medium** - Viable long-term, but data collection is a blocker. Start with simpler approaches.

---

### 3.3 Reinforcement Learning

**Description:** RL agent learns optimal tuning by receiving rewards for correct detections and penalties for false positives/negatives.

#### Approach

Based on research from securitybulldog.com:

**State Space:**
- Current detector thresholds
- Recent FP/FN rates
- Package characteristics distribution

**Action Space:**
- Increase/decrease threshold by Δ
- Adjust weight for specific detector
- Enable/disable detector

**Reward Scheme:**

| Outcome | Reward |
|---------|--------|
| True Positive | +1.0 |
| True Negative | +0.5 |
| False Positive | -2.0 (high penalty) |
| False Negative | -5.0 (very high penalty) |

**Algorithms:**
- Deep Q-Networks (DQN)
- Double DQN (DDQN)
- Policy Gradient (PG)
- Actor-Critic (AC)

#### Implementation Sketch

```rust
struct DetectorTuningEnv {
    state: TuningState,
    detectors: Vec<Box<dyn Detector>>,
    validation_dataset: Dataset,
}

impl Environment for DetectorTuningEnv {
    type State = TuningState;
    type Action = TuningAction;
    type Reward = f32;
    
    fn step(&mut self, action: TuningAction) -> (State, Reward, bool) {
        // Apply action (adjust thresholds)
        self.apply_action(action);
        
        // Run detection on validation set
        let results = self.run_detection();
        
        // Calculate reward
        let reward = self.calculate_reward(&results);
        
        // Check termination
        let done = self.is_converged();
        
        (self.state.clone(), reward, done)
    }
}

fn calculate_reward(&self, results: &DetectionResults) -> f32 {
    let tp_reward = results.true_positives as f32 * 1.0;
    let tn_reward = results.true_negatives as f32 * 0.5;
    let fp_penalty = results.false_positives as f32 * -2.0;
    let fn_penalty = results.false_negatives as f32 * -5.0;
    
    tp_reward + tn_reward + fp_penalty + fn_penalty
}
```

#### Pros
- ✅ Learns from feedback
- ✅ Adapts to new threats
- ✅ Optimizes for specific metrics

#### Cons
- ❌ High computational cost
- ❌ Risk of reward hacking
- ❌ Requires simulation environment
- ❌ Slow convergence

#### Viability for Glassworks
**⭐ Low** - Too complex for current stage. Consider after simpler approaches mature.

---

### 3.4 Genetic Algorithms

**Description:** Evolve detector parameters through selection, crossover, and mutation.

#### Approach

```
┌─────────────────┐
│ Initial         │
│ Population      │
│ (100 configs)   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐     ┌─────────────────┐
│ Evaluate        │────►│ Fitness         │
│ Fitness         │     │ Score           │
│ (scan dataset)  │     │ (F1, precision) │
└─────────────────┘     └────────┬────────┘
                                 │
         ┌───────────────────────┼───────────────────────┐
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│ Selection       │     │ Crossover       │     │ Mutation        │
│ (top 20%)       │────►│ (combine configs)│────►│ (random Δ)     │
└─────────────────┘     └─────────────────┘     └─────────────────┘
         │                                           │
         └───────────────────┬───────────────────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │ New Generation  │
                    │ (repeat)        │
                    └─────────────────┘
```

#### Configuration Encoding

```rust
#[derive(Clone, Debug)]
struct DetectorConfig {
    malicious_threshold: f32,      // 5.0 - 10.0
    category_weight: f32,          // 1.0 - 5.0
    critical_weight: f32,          // 1.0 - 5.0
    high_weight: f32,              // 1.0 - 3.0
    invisible_chars_weight: f32,   // 0.5 - 3.0
    homoglyphs_weight: f32,        // 0.5 - 3.0
    // ... more parameters
}

impl GeneticConfig {
    fn crossover(&self, other: &Self) -> Self {
        // Single-point or uniform crossover
        DetectorConfig {
            malicious_threshold: if random() < 0.5 {
                self.malicious_threshold
            } else {
                other.malicious_threshold
            },
            // ... blend other parameters
        }
    }
    
    fn mutate(&mut self, rate: f32) {
        if random() < rate {
            self.malicious_threshold += gaussian(0.0, 0.5);
            self.malicious_threshold = self.malicious_threshold.clamp(5.0, 10.0);
        }
        // ... mutate other parameters
    }
}
```

#### Fitness Function

```rust
fn fitness(config: &DetectorConfig, dataset: &Dataset) -> f32 {
    let results = run_detection(config, dataset);
    
    // Weighted F1 score (prioritize recall for security)
    let precision = results.precision();
    let recall = results.recall();
    
    // F2 score (recall weighted 2x more than precision)
    let f2_score = 5.0 * precision * recall / (4.0 * precision + recall);
    
    // Penalize high FP rate
    let fp_penalty = if results.fp_rate() > 0.05 {
        (results.fp_rate() - 0.05) * 10.0
    } else {
        0.0
    };
    
    f2_score - fp_penalty
}
```

#### Best Practices (from research)

| Parameter | Recommendation |
|-----------|----------------|
| Population size | 50-200 |
| Mutation rate | 0.01-0.1 |
| Crossover rate | 0.6-0.9 |
| Selection method | Tournament (size 3-5) or Roulette |
| Generations | 50-200 or until convergence |
| Elitism | Keep top 5-10% unchanged |

#### Pros
- ✅ No gradient needed
- ✅ Explores large search space
- ✅ Finds global optima (not local)
- ✅ Explainable (config is just parameters)

#### Cons
- ❌ Computationally expensive (100s of evaluations)
- ❌ No guarantee of optimality
- ❌ Requires careful parameter tuning
- ❌ Slow convergence

#### Viability for Glassworks
**⭐⭐ Medium-High** - Good fit for offline tuning. Can run overnight on validation dataset.

---

### 3.5 LLM-Assisted Tuning

**Description:** Use LLM to suggest tuning changes based on analysis of findings.

#### Approach

```
┌─────────────────┐
│ Scan Results    │
│ + FP/FN Labels  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ LLM Prompt      │
│ "Analyze these  │
│ false positives │
│ and suggest     │
│ threshold       │
│ adjustments"    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Suggested       │
│ Changes         │
│ - Increase      │
│   threshold     │
│ - Add exception │
│ - Adjust weight │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Human Review    │
│ (approve/reject)│
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Apply &         │
│ Validate        │
└─────────────────┘
```

#### Prompt Template

```markdown
You are a security detection tuning expert.

## Current Configuration
```toml
malicious_threshold = 7.0
category_weight = 2.0
critical_weight = 3.0
high_weight = 1.5
```

## Recent Scan Results
- Total packages scanned: 1,234
- Flagged as malicious: 45
- Confirmed false positives: 12
- Confirmed false negatives: 3

## False Positive Analysis
Top detectors causing FPs:
1. homoglyphs_detector: 8 FPs (legitimate i18n packages)
2. locale_detector: 4 FPs (legitimate geo-targeting)

## False Negative Analysis
Missed detections:
1. Package X: Used time-delay obfuscation (not detected)
2. Package Y: Used header-based C2 (not detected)

## Task
Suggest configuration changes to:
1. Reduce false positives from homoglyphs and locale detectors
2. Improve detection of time-delay and header-C2 attacks

Provide specific threshold/weight adjustments and explain your reasoning.
```

#### Expected Output

```toml
# Suggested changes

# Increase threshold to reduce FPs (7.0 → 7.5)
malicious_threshold = 7.5

# Reduce weight for homoglyphs (causing i18n FPs)
# category_weight = 2.0 → 1.5
category_weight = 1.5

# Keep critical weight (accurate)
critical_weight = 3.0

# Slightly reduce high weight
# high_weight = 1.5 → 1.3
high_weight = 1.3

# Add new weights for missed detectors
time_delay_weight = 2.5
header_c2_weight = 2.0
```

#### Pros
- ✅ Explainable suggestions
- ✅ Leverages security knowledge
- ✅ Fast iteration
- ✅ Human-in-the-loop

#### Cons
- ❌ LLM may hallucinate
- ❌ Requires careful prompt engineering
- ❌ Limited by LLM's security knowledge
- ❌ Needs validation pipeline

#### Viability for Glassworks
**⭐⭐⭐ High** - Excellent fit. Can integrate with existing LLM infrastructure (Cerebras, NVIDIA).

---

## 4. Feasibility Analysis

### 4.1 Data Requirements

| Approach | Dataset Size | Label Quality | Data Source |
|----------|-------------|---------------|-------------|
| Manual tuning | N/A | N/A | N/A |
| Supervised learning | 10,000+ samples | 99%+ accuracy | npm security advisories, manual labeling |
| Reinforcement learning | Simulation environment | Reward function design | Synthetic + real data |
| Genetic algorithms | 1,000+ samples | 95%+ accuracy | npm security advisories |
| LLM-assisted | 100+ samples | 90%+ accuracy | Recent scan results |

**Current Glassworks Data:**
- From wave campaigns: ~500-1,000 packages scanned
- Known malicious: ~10-20 packages (from calibration waves)
- Known clean: ~50-100 packages (express, lodash, etc.)

**Gap Analysis:**
- ❌ Insufficient labeled data for supervised learning
- ⚠️ Borderline for genetic algorithms
- ✅ Sufficient for LLM-assisted tuning

### 4.2 Computational Cost

| Approach | Training Cost | Inference Cost | Hardware |
|----------|--------------|----------------|----------|
| Manual tuning | N/A | N/A | N/A |
| Supervised learning | High (GPU hours) | Low (CPU ms) | GPU cluster |
| Reinforcement learning | Very High (GPU days) | Low (CPU ms) | GPU cluster |
| Genetic algorithms | Medium-High (CPU hours) | N/A | Multi-core CPU |
| LLM-assisted | Low (API calls) | N/A | API credits (~$10-100/run) |

**Estimates:**
- Genetic algorithms: 100 configs × 1,000 packages × 0.5s/pkg = 14 hours
- LLM-assisted: 10 API calls × $0.50/call = $5 per tuning session

### 4.3 Risk of Overfitting

| Approach | Overfitting Risk | Mitigation |
|----------|-----------------|------------|
| Manual tuning | Low (human judgment) | Cross-validation on multiple datasets |
| Supervised learning | High | Regularization, cross-validation, holdout set |
| Reinforcement learning | Medium-High | Diverse training scenarios, early stopping |
| Genetic algorithms | Medium | Validation set separate from fitness set |
| LLM-assisted | Low-Medium | Human review, backpressure checks |

### 4.4 Explainability

| Approach | Explainability | Audit Trail |
|----------|---------------|-------------|
| Manual tuning | ✅ High (human decisions documented) | ✅ Full |
| Supervised learning | ❌ Low (black-box model) | ⚠️ Feature importance only |
| Reinforcement learning | ❌ Low (learned policy) | ⚠️ Action logs |
| Genetic algorithms | ✅ Medium (config parameters) | ✅ Full config history |
| LLM-assisted | ✅ High (natural language reasoning) | ✅ Full conversation log |

### 4.5 Integration Complexity

| Approach | Integration Effort | Dependencies | Maintenance |
|----------|-------------------|--------------|-------------|
| Manual tuning | ✅ None (existing) | None | Low |
| Supervised learning | ❌ High | PyTorch/sklearn, data pipeline | High |
| Reinforcement learning | ❌ Very High | RL library, simulation env | Very High |
| Genetic algorithms | ⚠️ Medium | GA library (linfa, etc.) | Medium |
| LLM-assisted | ✅ Low-Medium | Existing LLM infra | Low |

---

## 5. Recommendations

### 5.1 Viable Approaches for Glassworks

**Immediate (Phase 1 - Q2 2026):**

1. **LLM-Assisted Tuning** ⭐⭐⭐
   - Leverage existing Cerebras/NVIDIA integration
   - Build FP/FN analysis prompts
   - Human-in-the-loop validation
   - **Effort:** 2-3 weeks
   - **Impact:** High

2. **Enhanced Manual Tuning** ⭐⭐⭐
   - Build tuning dashboard (FP rates per detector)
   - Statistical validation (MAD-based confidence)
   - Session persistence (like PI Auto-Research)
   - **Effort:** 1-2 weeks
   - **Impact:** Medium-High

**Short-Term (Phase 2 - Q3 2026):**

3. **Genetic Algorithms** ⭐⭐
   - Offline tuning overnight
   - Validate on separate dataset
   - Config versioning
   - **Effort:** 3-4 weeks
   - **Impact:** Medium

**Long-Term (Phase 3 - Q4 2026+):**

4. **Supervised Learning** ⭐
   - Requires data collection infrastructure
   - Start with simple models (logistic regression)
   - Hybrid approach (ML suggests, human approves)
   - **Effort:** 8-12 weeks
   - **Impact:** High (if data available)

5. **Reinforcement Learning** ⭐
   - Research project, not production-ready
   - Requires simulation environment
   - High risk, high reward
   - **Effort:** 12-16 weeks
   - **Impact:** Uncertain

---

### 5.2 Data Infrastructure Needed

**Minimum Viable Dataset:**

```
data/
├── malicious/
│   ├── package1.json      # Scan results + metadata
│   ├── package2.json
│   └── ...
├── clean/
│   ├── express.json
│   ├── lodash.json
│   └── ...
└── metadata.json          # Dataset manifest
```

**Package Metadata Schema:**

```json
{
  "package": "react-native-country-select",
  "version": "0.3.91",
  "label": "malicious",
  "label_confidence": 0.99,
  "label_source": "npm_security_advisory",
  "label_date": "2026-03-15",
  "scan_results": {
    "threat_score": 9.2,
    "findings": [...],
    "llm_verdict": "malicious"
  },
  "metadata": {
    "age_days": 45,
    "downloads": 1234,
    "publisher": "unknown",
    "ecosystem": "npm"
  }
}
```

**Data Collection Pipeline:**

1. **Automated collection** from wave campaigns
2. **Human labeling** for FP/FN (TUI integration)
3. **Versioned dataset** (Git LFS or DVC)
4. **Regular updates** (monthly refresh)

---

### 5.3 Implementation Phases

#### Phase 1: Foundation (Weeks 1-4)

**Goal:** Enable LLM-assisted tuning with statistical validation

**Tasks:**
- [ ] Build FP/FN labeling interface (TUI or CLI)
- [ ] Implement MAD-based confidence scoring (from PI Auto-Research)
- [ ] Create LLM prompts for tuning suggestions
- [ ] Build validation pipeline (backpressure checks)
- [ ] Session persistence (autoresearch.jsonl equivalent)

**Deliverables:**
- `glassware-tune` CLI command
- Tuning session logs
- Confidence scores for tuning changes

---

#### Phase 2: Automation (Weeks 5-8)

**Goal:** Add genetic algorithm tuning for offline optimization

**Tasks:**
- [ ] Implement GA framework (Rust: `linfa` or custom)
- [ ] Define fitness function (F2 score with FP penalty)
- [ ] Build overnight tuning campaigns
- [ ] Config versioning and rollback
- [ ] Dashboard for tuning progress

**Deliverables:**
- `glassware-orchestrator tune` command
- GA tuning reports
- Optimized config recommendations

---

#### Phase 3: Scale (Weeks 9-12)

**Goal:** Build data infrastructure for ML approaches

**Tasks:**
- [ ] Dataset collection pipeline
- [ ] Human labeling workflow integration
- [ ] Dataset versioning (DVC or Git LFS)
- [ ] Simple ML model (logistic regression baseline)
- [ ] Hybrid ML + human approval workflow

**Deliverables:**
- `data/` directory with labeled dataset
- `glassware-ml` prototype
- Dataset documentation

---

### 5.4 Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| **Overfitting to validation set** | Medium | High | Use separate test set, cross-validation |
| **LLM hallucination** | Medium | Medium | Human review, backpressure checks |
| **Data quality issues** | High | High | Rigorous labeling process, confidence scores |
| **Computational cost overrun** | Medium | Medium | Budget controls, early stopping |
| **False sense of security** | Low | High | Clear documentation of limitations |
| **Adversarial manipulation** | Low | Critical | Adversarial testing, robust validation |

---

## 6. Implementation Roadmap

### Timeline

```
Q2 2026 (Apr-Jun)          Q3 2026 (Jul-Sep)          Q4 2026 (Oct-Dec)
│                          │                          │
├─ Phase 1: Foundation     ├─ Phase 2: Automation     ├─ Phase 3: Scale
│  - LLM-assisted tuning   │  - Genetic algorithms    │  - Data infrastructure
│  - Statistical validation│  - Overnight tuning      │  - ML baseline
│  - Session persistence   │  - Config versioning     │  - Hybrid workflow
│                          │                          │
└──────────────────────────┴──────────────────────────┴─────────────────
     Weeks 1-4                   Weeks 5-8                 Weeks 9-12
```

### Milestones

| Milestone | Target Date | Success Criteria |
|-----------|-------------|------------------|
| **M1: LLM Tuning MVP** | Week 4 | Can suggest and validate tuning changes |
| **M2: Statistical Validation** | Week 4 | MAD confidence scores on all changes |
| **M3: GA Tuning MVP** | Week 8 | Overnight tuning produces better configs |
| **M4: Dataset v1** | Week 12 | 1,000+ labeled packages |
| **M5: ML Baseline** | Week 12 | Logistic regression beats random baseline |

---

## 7. Appendix

### 7.1 Reference Implementations

**PI Auto-Research Confidence Scoring:**
```typescript
// MAD-based confidence
function calculateConfidence(experiments: Experiment[]): number {
    if (experiments.length < 3) return 0;
    
    const metrics = experiments.map(e => e.metric);
    const median = calculateMedian(metrics);
    const mad = calculateMAD(metrics, median);
    
    const bestImprovement = Math.abs(median - Math.min(...metrics));
    return bestImprovement / mad;
}
```

**Genetic Algorithm Fitness Function:**
```rust
fn fitness(config: &Config, dataset: &Dataset) -> f32 {
    let results = detect(config, dataset);
    let precision = results.precision();
    let recall = results.recall();
    
    // F2 score (recall weighted 2x)
    let f2 = 5.0 * precision * recall / (4.0 * precision + recall);
    
    // Penalize high FP rate
    let fp_penalty = (results.fp_rate() - 0.05).max(0.0) * 10.0;
    
    f2 - fp_penalty
}
```

### 7.2 Key Files in Glassworks

| File | Purpose |
|------|---------|
| `/home/property.sightlines/samgrowls/glassworks/glassware-core/src/detector.rs` | Detector trait definition |
| `/home/property.sightlines/samgrowls/glassworks/glassware-core/src/risk_scorer.rs` | Risk scoring with thresholds |
| `/home/property.sightlines/samgrowls/glassworks/glassware-core/src/config.rs` | Configuration structures |
| `/home/property.sightlines/samgrowls/glassworks/WAVE6-FIXES.md` | Current tuning parameters |
| `/home/property.sightlines/samgrowls/glassworks/design/WAVE-CAMPAIGN-DESIGN.md` | Campaign architecture |

### 7.3 Glossary

| Term | Definition |
|------|------------|
| **FP (False Positive)** | Clean package incorrectly flagged as malicious |
| **FN (False Negative)** | Malicious package not detected |
| **MAD (Median Absolute Deviation)** | Robust statistical measure of variability |
| **Backpressure Check** | Validation that tuning changes don't break detection |
| **Session Persistence** | Saving tuning state across restarts |
| **Fitness Function** | Objective function for genetic algorithms |

---

## 8. Conclusion

**Recommended Approach:** Start with **LLM-assisted tuning** (Phase 1) combined with **statistical validation** inspired by PI Auto-Research. This provides immediate value with minimal risk.

**Next Steps:**
1. Implement FP/FN labeling interface
2. Build MAD confidence scoring
3. Create LLM tuning prompts
4. Run pilot tuning session on Wave 6 data

**Long-Term Vision:** Evolve toward hybrid approach combining LLM suggestions, genetic algorithm optimization, and ML-based pattern discovery—all with human oversight and statistical validation.
