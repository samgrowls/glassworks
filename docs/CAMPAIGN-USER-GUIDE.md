# GlassWorm Campaign System User Guide

**Version:** 0.12.1
**Last Updated:** March 22, 2026

---

## Quick Start

### Run Your First Campaign

```bash
# Run the Wave 6 calibration campaign
glassware-orchestrator campaign run campaigns/wave6.toml

# View results
glassware-orchestrator campaign status <case-id>
```

---

## Table of Contents

1. [Overview](#overview)
2. [Campaign Commands](#campaign-commands)
3. [Configuration](#configuration)
4. [Wave 6 Example](#wave-6-example)
5. [Troubleshooting](#troubleshooting)

---

## Overview

The GlassWorm Campaign System enables large-scale, orchestrated security scanning campaigns with:

- **Wave-based execution** - Organize scans into logical phases with dependencies
- **Real-time monitoring** - Track progress with live status updates
- **Command steering** - Pause, resume, skip waves during execution
- **Evidence collection** - Automatic evidence gathering for flagged packages
- **Multiple output formats** - JSON, Markdown, and SARIF reports

### Two-Tier LLM Strategy

| Tier | Provider | Speed | Use Case |
|------|----------|-------|----------|
| **Tier 1** | Cerebras | ~2-5s/pkg | Fast triage during scan |
| **Tier 2** | NVIDIA | ~15-30s/pkg | Deep analysis of flagged packages |

---

## Campaign Commands

### `campaign run` - Execute a Campaign

Run a campaign from a TOML configuration file.

```bash
glassware-orchestrator campaign run <config.toml> [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--concurrency <N>` | Override concurrency setting |
| `--rate-limit <N>` | Override rate limit (requests/second) |
| `--llm` | Enable Tier 1 LLM (Cerebras) during scan |
| `--llm-deep` | Enable Tier 2 LLM (NVIDIA) for flagged packages |

**Example:**

```bash
# Basic run
glassware-orchestrator campaign run campaigns/wave6.toml

# With LLM triage
glassware-orchestrator campaign run campaigns/wave6.toml --llm

# With custom concurrency
glassware-orchestrator campaign run campaigns/wave6.toml --concurrency 20
```

**Output:**

```
🚀 Starting campaign 'Wave 6 Calibration' (case: wave6-calibration-20260322-150342)
Loaded campaign 'Wave 6 Calibration' with 3 waves
🚀 Starting campaign execution...
📦 Starting wave 'Known Malicious Baseline' (wave_6a)
Collected 2 packages for wave 'Known Malicious Baseline'
✅ Wave 'Known Malicious Baseline' completed: 2 scanned, 2 flagged, 2 malicious
...

============================================================
CAMPAIGN COMPLETE
============================================================
Case ID: wave6-calibration-20260322-150342
Status: Completed
Duration: 45.2s
Packages scanned: 11
Packages flagged: 3
Malicious packages: 2
============================================================
```

---

### `campaign status` - Show Campaign Status

View the status of a running or completed campaign.

```bash
glassware-orchestrator campaign status <case-id> [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--live` | Show live updating status (future feature) |

**Example:**

```bash
# Check status
glassware-orchestrator campaign status wave6-calibration-20260322-150342

# JSON output
glassware-orchestrator campaign status wave6-calibration-20260322-150342 --format json
```

**Output:**

```
Campaign: wave6-calibration-20260322-150342
Status: Running
Command: campaign
Findings: 5
Malicious: 2
Started: 2026-03-22T15:03:42Z
```

---

### `campaign list` - List Recent Campaigns

Show a list of recent campaigns.

```bash
glassware-orchestrator campaign list [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--limit <N>` | Maximum campaigns to show (default: 10) |
| `--status <STATUS>` | Filter by status (running, completed, failed, cancelled) |

**Example:**

```bash
# List recent campaigns
glassware-orchestrator campaign list

# Show only completed campaigns
glassware-orchestrator campaign list --status completed --limit 5
```

**Output:**

```
Case ID                                  Status       Findings Malicious Started             
------------------------------------------------------------------------------------------
wave6-calibration-20260322-150342        Completed    5        2         2026-03-22T15:03:42Z
wave5-hunt-20260321-100000               Completed    15       3         2026-03-21T10:00:00Z
```

---

### `campaign resume` - Resume Interrupted Campaign

Resume a campaign that was interrupted (not yet implemented).

```bash
glassware-orchestrator campaign resume <case-id>
```

**Status:** Coming in Phase 2

---

### `campaign command` - Send Command to Running Campaign

Send a steering command to a running campaign (not yet implemented).

```bash
glassware-orchestrator campaign command <case-id> <command> [argument]
```

**Commands:**

| Command | Description |
|---------|-------------|
| `pause` | Pause campaign execution |
| `resume` | Resume paused campaign |
| `cancel` | Cancel campaign |
| `skip-wave` | Skip current wave |
| `set-concurrency` | Set concurrency level |
| `set-rate-limit` | Set rate limit |

**Status:** Coming in Phase 2

---

### `campaign report` - Generate Campaign Report

Generate a report for a completed campaign (not yet implemented).

```bash
glassware-orchestrator campaign report <case-id> [OPTIONS]
```

**Options:**

| Option | Description |
|--------|-------------|
| `--format <FORMAT>` | Output format: markdown, json, sarif |
| `--output <FILE>` | Output file (default: stdout) |

**Status:** Coming in Phase 2

---

## Configuration

### Campaign TOML Structure

```toml
[campaign]
name = "Campaign Name"
description = "Campaign description"
created_by = "security-team"
priority = "high"  # low, medium, high, critical
tags = ["tag1", "tag2"]

[settings]
concurrency = 10
rate_limit_npm = 10.0
rate_limit_github = 5.0
cache_enabled = true
cache_ttl_days = 7

[settings.llm]
tier1_enabled = true
tier1_provider = "cerebras"
tier2_enabled = true
tier2_threshold = 5.0
tier2_models = [
    "qwen/qwen3.5-397b-a17b",
    "moonshotai/kimi-k2.5",
    "z-ai/glm5",
    "meta/llama-3.3-70b-instruct"
]

[settings.output]
formats = ["json", "markdown", "sarif"]
evidence_collection = true
evidence_dir = "evidence"
report_dir = "reports"

# Wave definitions
[[waves]]
id = "wave_1"
name = "Wave Name"
description = "Wave description"
depends_on = []  # Wave IDs that must complete first
mode = "hunt"  # validate, hunt, monitor

[[waves.sources]]
type = "packages"
list = ["package@version", ...]

[[waves]]
id = "wave_2"
name = "Wave 2"
depends_on = ["wave_1"]  # Runs after wave_1
mode = "hunt"

[[waves.sources]]
type = "npm_search"
keywords = ["keyword1", "keyword2"]
samples_per_keyword = 25
```

### Wave Modes

| Mode | Description |
|------|-------------|
| `validate` | Validate detection with known packages |
| `hunt` | Hunt for malicious packages |
| `monitor` | Monitor for changes (future) |

### Wave Sources

#### Package List

```toml
[[waves.sources]]
type = "packages"
list = [
    "express@4.19.2",
    "lodash@4.17.21"
]
```

#### npm Search

```toml
[[waves.sources]]
type = "npm_search"
keywords = ["react-native-phone", "react-native-country"]
samples_per_keyword = 25
days_recent = 365  # Optional: only packages from last N days
max_downloads = 10000  # Optional: filter by download count
```

#### npm Category

```toml
[[waves.sources]]
type = "npm_category"
category = "react-native"
samples = 50
sort_by = "recent"  # recent, popular, random
```

#### GitHub Search

```toml
[[waves.sources]]
type = "github_search"
query = "mcp server"
max_results = 50
sort_by = "stars"  # stars, forks, updated
```

### Validation Expectations

For `validate` mode waves:

```toml
[waves.expectations]
must_flag_all = true  # All malicious packages must be detected
min_threat_score = 7.0
```

---

## Wave 6 Example

### Configuration

The Wave 6 calibration campaign validates the pipeline with:

- **Wave 6A:** 2 known malicious packages (must be flagged)
- **Wave 6B:** 5 known clean packages (should not be flagged)
- **Wave 6C:** 4 React Native packages (hunt mode)

### Running Wave 6

```bash
# Run with default settings
glassware-orchestrator campaign run campaigns/wave6.toml

# Run with LLM triage
glassware-orchestrator campaign run campaigns/wave6.toml --llm

# Run with higher concurrency
glassware-orchestrator campaign run campaigns/wave6.toml --concurrency 20
```

### Expected Results

```
============================================================
CAMPAIGN COMPLETE
============================================================
Case ID: wave-6-calibration-20260322-150342
Status: Completed
Duration: 45.2s
Packages scanned: 11
Packages flagged: 3
Malicious packages: 2
============================================================
```

**Validation Criteria:**

- ✅ Wave 6A: Both malicious packages flagged (100% detection)
- ✅ Wave 6B: Clean packages not flagged (0% false positive)
- ✅ Wave 6C: Results vary (hunt mode)

---

## Troubleshooting

### "Failed to load campaign config"

**Cause:** Invalid TOML syntax or missing required fields.

**Solution:**

```bash
# Validate TOML syntax
cat campaigns/wave6.toml | python3 -c "import toml; toml.load(__import__('sys').stdin)"

# Check required fields
# - campaign.name
# - At least one wave with sources
```

### "Circular dependency detected"

**Cause:** Wave dependencies form a cycle.

**Solution:**

```toml
# ❌ Bad: wave_1 depends on wave_2, wave_2 depends on wave_1
[[waves]]
id = "wave_1"
depends_on = ["wave_2"]

[[waves]]
id = "wave_2"
depends_on = ["wave_1"]

# ✅ Good: Linear dependency chain
[[waves]]
id = "wave_1"
depends_on = []

[[waves]]
id = "wave_2"
depends_on = ["wave_1"]
```

### "Campaign not found"

**Cause:** Invalid case ID or campaign already cleaned up.

**Solution:**

```bash
# List recent campaigns to find valid case ID
glassware-orchestrator campaign list --limit 20
```

### Slow scanning

**Solutions:**

1. Increase concurrency: `--concurrency 20`
2. Increase rate limit: `--rate-limit 20`
3. Enable caching: Ensure `cache_enabled = true` in config

### LLM analysis fails

**Causes:**

- API key not set
- Rate limit exceeded
- Model unavailable

**Solutions:**

```bash
# Check environment variables
echo $GLASSWARE_LLM_BASE_URL
echo $GLASSWARE_LLM_API_KEY
echo $NVIDIA_API_KEY

# Try without LLM
glassware-orchestrator campaign run campaigns/wave6.toml

# Enable only Tier 1 (faster, more reliable)
glassware-orchestrator campaign run campaigns/wave6.toml --llm
```

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `GLASSWARE_LLM_BASE_URL` | LLM API base URL (Cerebras: `https://api.cerebras.ai/v1`) |
| `GLASSWARE_LLM_API_KEY` | LLM API key |
| `GLASSWARE_LLM_MODEL` | LLM model to use |
| `NVIDIA_API_KEY` | NVIDIA API key for Tier 2 analysis |
| `GITHUB_TOKEN` | GitHub token for private repositories |

---

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Campaign completed successfully, no malicious packages |
| 1 | Campaign completed, malicious packages detected |
| 2 | Campaign failed (error) |
| 130 | Campaign cancelled by user (Ctrl+C) |

---

## Getting Help

```bash
# Show all campaign commands
glassware-orchestrator campaign --help

# Show help for specific command
glassware-orchestrator campaign run --help
glassware-orchestrator campaign status --help
```

---

## See Also

- [CAMPAIGN-ARCHITECTURE.md](design/CAMPAIGN-ARCHITECTURE.md) - Technical architecture
- [RFC-001-TUI-ARCHITECTURE.md](design/RFC-001-TUI-ARCHITECTURE.md) - Future TUI design
- [WAVE6-FIXES.md](WAVE6-FIXES.md) - Wave 6 implementation notes
