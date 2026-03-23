# Rust Orchestrator Wave Campaign Design

**Date:** March 22, 2026
**Status:** Design Proposal - Awaiting Feedback

---

## Executive Summary

This document proposes a **wave campaign system** for the Rust orchestrator that:
1. Inherits the proven concepts from Python's `waves.toml`
2. Adds robust features unique to Rust (parallel scanning, streaming, SARIF)
3. Enables large-scale scanning (1000s of packages) with checkpoint/resume
4. Generates comprehensive markdown reports automatically

---

## Design Goals

| Goal | Priority | Rationale |
|------|----------|-----------|
| **Feature parity with Python** | High | Preserve existing wave workflows |
| **More robust configuration** | High | TOML validation, schema enforcement |
| **Better performance** | High | Leverage Rust parallelism |
| **Rich reporting** | Medium | Markdown + SARIF + JSON |
| **Evidence collection** | High | Chain of custody for findings |
| **Resume capability** | High | Long-running scans must resume |

---

## Wave Campaign Architecture

### Configuration Model

We should support **two configuration styles**:

#### Option A: Follow Python's `waves.toml` (Compatible)

```toml
# waves.toml - Same structure as Python
[wave_0]
name = "Wave 0: Calibration"
description = "Validate pipeline with known malicious + clean packages"
packages_total = 50

[wave_0.known_malicious]
packages = [
    "react-native-country-select@0.3.91",
    "react-native-international-phone-number@0.11.8",
]

[wave_0.clean_baseline]
count = 20
packages = ["express", "lodash", "axios", ...]

[wave_0.react_native_ecosystem]
count = 15
keywords = ["react-native-phone", "react-native-country"]
```

**Pros:**
- ✅ Backward compatible with existing wave configs
- ✅ Users can migrate easily
- ✅ Same mental model

**Cons:**
- ❌ Limited flexibility (fixed structure)
- ❌ Can't express complex dependencies between waves
- ❌ No conditional logic

---

#### Option B: Enhanced Wave Campaigns (Recommended)

```toml
# campaigns/wave6.toml
[campaign]
name = "Wave 6: React Native Hunt"
description = "Targeted hunting in React Native ecosystem"
created_by = "security-team"
priority = "high"

# Global settings
[settings]
concurrency = 15
rate_limit_npm = 15.0
rate_limit_github = 5.0
llm_tier1 = true       # Cerebras during scan
llm_tier2_threshold = 5.0  # NVIDIA for threat_score >= 5.0
output_formats = ["json", "markdown", "sarif"]
evidence_collection = true

# Wave definitions (can run sequentially or in parallel)
[[waves]]
id = "wave_6a"
name = "Known Malicious Baseline"
description = "Validate detection with confirmed malicious packages"
depends_on = []  # No dependencies, runs first
mode = "validate"  # Special mode: must flag all as malicious

[[waves.sources]]
type = "packages"
list = [
    "react-native-country-select@0.3.91",
    "react-native-international-phone-number@0.11.8",
]

[[waves]]
id = "wave_6b"
name = "React Native Ecosystem"
description = "Hunt for GlassWorm in React Native packages"
depends_on = ["wave_6a"]  # Runs after 6a completes
mode = "hunt"

[[waves.sources]]
type = "npm_search"
keywords = ["react-native-phone", "react-native-country", "react-native-locale"]
samples_per_keyword = 30
days_recent = 365

[[waves.sources]]
type = "npm_category"
category = "react-native"
samples = 50

[[waves]]
id = "wave_6c"
name = "MCP/AI Infrastructure"
description = "Scan AI/LLM infrastructure packages"
depends_on = []  # Can run in parallel with 6b
mode = "hunt"

[[waves.sources]]
type = "npm_search"
keywords = ["mcp", "llm", "langchain", "agent"]
samples_per_keyword = 25

# Whitelist overrides (per-wave)
[[waves.whitelist]]
wave_id = "wave_6b"
packages = ["react-native", "react-native-maps"]  # Known clean

# Reporting configuration
[reporting]
summary_email = "security@example.com"
slack_webhook = "https://hooks.slack.com/..."
auto_generate = true
include_clean_summary = false  # Don't report clean packages
```

**Pros:**
- ✅ Flexible wave definitions
- ✅ Dependencies between waves
- ✅ Multiple source types (search, category, explicit list, GitHub)
- ✅ Per-wave settings
- ✅ Evidence collection built-in
- ✅ Can express complex campaigns

**Cons:**
- ❌ Not backward compatible with Python waves.toml
- ❌ More complex configuration

---

### Recommended Approach

**Start with Option B (Enhanced)** because:
1. We're making Rust the primary orchestrator
2. Python waves can be migrated manually (there are only a few)
3. The enhanced model supports our long-term vision (overnight scans, 10k+ packages)

---

## CLI Design

### Campaign Commands

```bash
# List available campaigns
glassware-orchestrator campaign list

# Show campaign details
glassware-orchestrator campaign show wave6

# Run a campaign
glassware-orchestrator campaign run wave6.toml

# Run specific wave
glassware-orchestrator campaign run wave6.toml --wave wave_6b

# Resume interrupted campaign
glassware-orchestrator campaign resume wave6

# Show campaign status
glassware-orchestrator campaign status wave6

# Generate report for completed campaign
glassware-orchestrator campaign report wave6 --format markdown
```

### Campaign Run Options

```bash
# Full campaign with all waves
glassware-orchestrator campaign run wave6.toml

# Dry run (validate config, show what would be scanned)
glassware-orchestrator campaign run wave6.toml --dry-run

# Run with overrides
glassware-orchestrator campaign run wave6.toml \
  --concurrency 20 \
  --llm-tier1 \
  --llm-tier2-threshold 5.0

# Run specific waves only
glassware-orchestrator campaign run wave6.toml --wave wave_6b,wave_6c

# Run until first malicious found (hunt mode)
glassware-orchestrator campaign run wave6.toml --stop-on-malicious
```

---

## Markdown Report Design

### Report Structure

```markdown
# GlassWorm Campaign Report: Wave 6

**Campaign:** Wave 6: React Native Hunt
**Run ID:** `wave6-20260322-143052`
**Date:** March 22, 2026
**Duration:** 2h 34m 15s

---

## Executive Summary

| Metric | Value |
|--------|-------|
| Packages Scanned | 523 |
| Malicious | 2 |
| Suspicious | 15 |
| Clean | 506 |
| Detection Rate | 0.38% |

### Malicious Packages

| Package | Version | Threat Score | LLM Verdict | Category |
|---------|---------|--------------|-------------|----------|
| `react-native-phone-input` | 1.3.7 | 8.5 | Malicious (0.92) | GlassWare Pattern |
| `react-native-locale-check` | 2.0.1 | 7.2 | Suspicious (0.65) | Locale Geofencing |

---

## Wave Results

### Wave 6a: Known Malicious Baseline

**Status:** ✅ PASS (2/2 malicious detected)

| Package | Expected | Detected | Threat Score |
|---------|----------|----------|--------------|
| `react-native-country-select@0.3.91` | Malicious | ✅ Malicious | 9.2 |
| `react-native-international-phone-number@0.11.8` | Malicious | ✅ Malicious | 8.8 |

### Wave 6b: React Native Ecosystem

**Status:** ⚠️ 2 malicious, 12 suspicious

[Detailed findings table...]

### Wave 6c: MCP/AI Infrastructure

**Status:** ✅ Clean (0 malicious, 3 suspicious)

[Detailed findings table...]

---

## LLM Analysis Summary

### Tier 1 (Cerebras) Triage

- Packages analyzed: 523
- Average analysis time: 3.2s
- Model: `qwen-3-235b-a22b-instruct-2507`

### Tier 2 (NVIDIA) Deep Analysis

- Packages analyzed: 17 (threat_score >= 5.0)
- Average analysis time: 18.5s
- Models used:
  - `qwen/qwen3.5-397b-a17b`: 15 packages
  - `moonshotai/kimi-k2.5`: 2 packages (fallback)

---

## Findings by Category

| Category | Count | Malicious | Suspicious |
|----------|-------|-----------|------------|
| InvisibleCharacter | 45 | 1 | 3 |
| GlasswarePattern | 12 | 1 | 2 |
| LocaleGeofencing | 8 | 0 | 5 |
| TimeDelaySandboxEvasion | 5 | 0 | 2 |

---

## Evidence Collected

| Package | Evidence Type | Path |
|---------|--------------|------|
| `react-native-phone-input@1.3.7` | Tarball, Scan Results, LLM Analysis | `evidence/wave6/react-native-phone-input-1.3.7/` |
| `react-native-locale-check@2.0.1` | Tarball, Scan Results | `evidence/wave6/react-native-locale-check-2.0.1/` |

---

## Recommendations

1. **Immediate Action:** Report `react-native-phone-input@1.3.7` to npm Security
2. **Monitor:** 15 suspicious packages for future updates
3. **Expand Scan:** Consider scanning all React Native ecosystem (5000+ packages)

---

## Appendix

### Scan Configuration

```toml
concurrency = 15
rate_limit = 15.0
llm_tier1 = true
llm_tier2_threshold = 5.0
```

### Command Used

```bash
glassware-orchestrator campaign run wave6.toml --llm-tier1 --llm-tier2
```

### Full Results

- JSON: `reports/wave6-results.json`
- SARIF: `reports/wave6-results.sarif`
- Evidence: `evidence/wave6/`
```

---

## Evidence Collection

### Evidence Structure

```
evidence/
└── wave6/
    ├── manifest.json          # Campaign manifest
    ├── react-native-phone-input-1.3.7/
    │   ├── package.tgz        # Original tarball
    │   ├── package.tgz.sha256 # Integrity hash
    │   ├── scan-results.json  # Full scan results
    │   ├── llm-tier1.json     # Cerebras analysis
    │   ├── llm-tier2.json     # NVIDIA analysis
    │   └── metadata.json      # Collection metadata
    └── react-native-locale-check-2.0.1/
        └── ...
```

### Evidence Metadata

```json
{
  "package": "react-native-phone-input",
  "version": "1.3.7",
  "collected_at": "2026-03-22T14:35:42Z",
  "collected_by": "glassware-orchestrator v0.8.0",
  "campaign_id": "wave6",
  "run_id": "wave6-20260322-143052",
  "chain_of_custody": [
    {
      "action": "downloaded",
      "timestamp": "2026-03-22T14:35:40Z",
      "actor": "glassware-orchestrator"
    },
    {
      "action": "scanned",
      "timestamp": "2026-03-22T14:35:42Z",
      "actor": "glassware-orchestrator"
    },
    {
      "action": "llm_analyzed_tier1",
      "timestamp": "2026-03-22T14:35:45Z",
      "actor": "cerebras-llm"
    }
  ],
  "integrity": {
    "tarball_sha256": "abc123...",
    "scan_results_sha256": "def456..."
  }
}
```

---

## Implementation Phases

### Phase 1: Core Wave Campaign (Week 1)

- [ ] Campaign config parsing with validation
- [ ] Wave execution engine (sequential + parallel)
- [ ] Checkpoint/resume for campaigns
- [ ] Basic markdown report generation

### Phase 2: Enhanced Features (Week 2)

- [ ] Multiple source types (npm search, category, GitHub)
- [ ] Evidence collection with integrity
- [ ] Tier 2 LLM auto-trigger
- [ ] SARIF report generation

### Phase 3: Polish & Scale (Week 3)

- [ ] Campaign progress tracking with ETA
- [ ] Slack/email notifications
- [ ] Wave dependency visualization
- [ ] Performance optimization for 10k+ packages

---

## Questions for Discussion

1. **Configuration Model:** Option A (compatible) or Option B (enhanced)?
   - My recommendation: **Option B** for long-term flexibility

2. **Evidence Collection:** Should we collect evidence for ALL packages or only flagged?
   - My recommendation: **Flagged only** (storage costs)

3. **Wave Dependencies:** Should waves support complex DAGs or just linear sequences?
   - My recommendation: **DAG** (allows parallel waves)

4. **Report Frequency:** Generate reports after each wave or only at campaign end?
   - My recommendation: **Both** (wave reports + final summary)

5. **Overnight Scan Prep:** What reliability features do we need before 10k+ scans?
   - My recommendation: **Robust checkpointing, rate limit handling, memory management**

---

## Next Steps

1. **Get feedback** on this design
2. **Finalize configuration model** (Option A vs B)
3. **Implement Phase 1** (core campaign support)
4. **Create Wave 6 campaign config**
5. **Run Wave 6** (500+ packages)
6. **Iterate based on results**
