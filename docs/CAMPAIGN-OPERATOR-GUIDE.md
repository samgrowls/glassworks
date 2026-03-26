# Campaign Operator Guide

**Version:** v0.67.0
**Last Updated:** 2026-03-26

---

## Overview

This guide covers campaign operations for glassware operators running large-scale scanning campaigns.

---

## Campaign Lifecycle

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Planning   │────►│  Execution  │────►│  Reporting  │
└─────────────┘     └─────────────┘     └─────────────┘
       │                   │                   │
       ▼                   ▼                   ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ Config TOML │     │ Monitoring  │     │ Analysis    │
│ Wave Design │     │ Commands    │     │ Export      │
└─────────────┘     └─────────────┘     └─────────────┘
```

---

## Planning Phase

### Campaign Configuration

Create a TOML configuration file:

```toml
[campaign]
name = "Wave 16 - Production Scan"
description = "Large-scale production scanning"
created_by = "security-team"
priority = "high"

[settings]
concurrency = 20
rate_limit_npm = 10.0
rate_limit_github = 5.0
cache_enabled = true
cache_ttl_days = 7

[settings.scoring]
malicious_threshold = 7.0
suspicious_threshold = 4.0

[settings.scoring.tier_config]
mode = "tiered"

[[settings.scoring.tiers]]
tier = 1
detectors = ["invisible_char", "glassware_pattern", "obfuscation"]
threshold = 0.0
weight_multiplier = 1.0

[[settings.scoring.tiers]]
tier = 2
detectors = ["header_c2", "exfil_schema"]
threshold = 2.0
weight_multiplier = 0.8

[settings.llm]
tier1_enabled = true
tier1_provider = "cerebras"
tier1_threshold = 6.0
tier2_enabled = true
tier2_threshold = 8.0

[settings.output]
formats = ["json", "markdown"]
evidence_collection = true
evidence_dir = "evidence/wave16"
report_dir = "reports/wave16"
```

### Wave Design

```toml
# Wave 1: Evidence Validation
[[waves]]
id = "wave_1"
name = "Evidence Validation"
description = "Validate 100% evidence detection"
depends_on = []
mode = "validate"

[[waves.sources]]
type = "packages"
list = [
    "iflow-mcp-watercrawl-mcp@1.3.4",
]

[waves.expectations]
must_flag_all = true
min_threat_score = 7.0

# Wave 2: Clean Baseline
[[waves]]
id = "wave_2"
name = "Clean Baseline"
description = "Measure FP rate on clean packages"
depends_on = ["wave_1"]
mode = "validate"

[[waves.sources]]
type = "packages"
list = [
    "express@4.19.2",
    "lodash@4.17.21",
    # ... 200 more clean packages
]

[waves.expectations]
must_flag_all = false
max_threat_score = 5.0
max_malicious_count = 2

# Wave 3: High-Risk Hunt
[[waves]]
id = "wave_3"
name = "High-Risk Categories"
description = "Scan high-risk categories"
depends_on = ["wave_2"]
mode = "hunt"

[[waves.sources]]
type = "npm_search"
keywords = ["react-native", "crypto", "wallet"]
samples_per_keyword = 50
max_downloads = 100000

[waves.expectations]
min_threat_score = 7.0
```

### Pre-Flight Checklist

- [ ] Evidence packages validated (100% detection)
- [ ] Clean baseline defined (expected FP rate < 1%)
- [ ] Rate limits configured (npm: 10/s, github: 5/s)
- [ ] Concurrency set (start with 20, adjust based on rate limits)
- [ ] LLM API keys configured (if using LLM)
- [ ] Output directories created
- [ ] Cache cleared (if re-running)

---

## Execution Phase

### Start Campaign

```bash
# Run campaign
./target/release/glassware campaign run campaigns/wave16.toml

# Run with LLM analysis
./target/release/glassware campaign run campaigns/wave16.toml --llm

# Run with deep LLM analysis
./target/release/glassware campaign run campaigns/wave16.toml --deep-llm
```

### Monitor Progress

```bash
# List campaigns
./target/release/glassware campaign list

# Show status
./target/release/glassware campaign status <case-id>

# Live TUI monitoring
./target/release/glassware campaign monitor <case-id>

# Demo mode (sample data)
./target/release/glassware campaign demo
```

### TUI Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `q` | Quit TUI |
| `Tab` | Switch tabs (Overview, Waves, Packages, Findings) |
| `p` | Pause/Resume campaign |
| `x` | Cancel campaign |
| `c` | Adjust concurrency |
| `r` | Set rate limit |
| `Enter` | Package drill-down |
| `l` | Run LLM analysis on package |
| `?` | Ask question about package |

### Runtime Commands

```bash
# Pause campaign
./target/release/glassware campaign command <case-id> pause

# Resume campaign
./target/release/glassware campaign command <case-id> resume

# Cancel campaign
./target/release/glassware campaign command <case-id> cancel

# Skip current wave
./target/release/glassware campaign command <case-id> skip-wave

# Adjust concurrency
./target/release/glassware campaign command <case-id> set-concurrency 30

# Adjust rate limit
./target/release/glassware campaign command <case-id> set-rate-limit 15
```

### Checkpoint/Resume

Campaigns automatically checkpoint progress:

```bash
# Campaign interrupted - resume from checkpoint
./target/release/glassware campaign resume <case-id>

# Check checkpoint files
ls -la .glassware-checkpoints*
```

---

## Monitoring Best Practices

### 1. Watch for Rate Limiting

```bash
# Monitor logs for rate limit errors
tail -f logs/wave16-*.log | grep -i "rate limit"

# If rate limited, reduce rate limit
./target/release/glassware campaign command <case-id> set-rate-limit 5
```

### 2. Monitor Concurrency

```bash
# Check current concurrency
./target/release/glassware campaign status <case-id>

# Adjust based on system load
./target/release/glassware campaign command <case-id> set-concurrency 10
```

### 3. Watch Evidence Wave

```bash
# Evidence wave should complete with 100% detection
tail -f logs/wave16-*.log | grep "Evidence Validation"

# If evidence not detected, STOP and investigate
./target/release/glassware campaign command <case-id> pause
```

### 4. Monitor FP Rate

```bash
# Check clean baseline results
tail -f logs/wave16-*.log | grep "Clean Baseline"

# If FP rate > 1%, investigate
./target/release/glassware campaign command <case-id> pause
```

---

## Reporting Phase

### Generate Reports

```bash
# Generate markdown report
./target/release/glassware campaign report <case-id>

# Generate JSON report
./target/release/glassware campaign report <case-id> --format json

# Generate SARIF report (for GitHub)
./target/release/glassware campaign report <case-id> --format sarif

# Save to file
./target/release/glassware campaign report <case-id> --output reports/wave16-final.md
```

### Report Contents

**Markdown Report:**
- Executive summary
- Campaign statistics
- Malicious packages list
- Evidence detection status
- FP analysis
- Recommendations

**JSON Report:**
- Machine-readable format
- All findings with metadata
- Suitable for automated processing

**SARIF Report:**
- GitHub Advanced Security compatible
- Import into GitHub Security tab

### Query Campaign with LLM

```bash
# Ask about campaign
./target/release/glassware campaign query <case-id> "How many packages were flagged?"

# Ask about specific package
./target/release/glassware campaign query <case-id> "Why was prisma flagged?"

# Ask for analysis
./target/release/glassware campaign query <case-id> "What patterns are common in flagged packages?"
```

---

## Troubleshooting

### Campaign Stuck

```bash
# Check status
./target/release/glassware campaign status <case-id>

# Check logs
tail -f logs/wave16-*.log

# Resume if paused
./target/release/glassware campaign command <case-id> resume

# Cancel and restart if needed
./target/release/glassware campaign command <case-id> cancel
./target/release/glassware campaign run campaigns/wave16.toml
```

### High FP Rate

```bash
# Pause campaign
./target/release/glassware campaign command <case-id> pause

# Review flagged packages
./target/release/glassware campaign query <case-id> "List all flagged packages with scores"

# Adjust scoring threshold
# Edit campaign TOML, then resume
./target/release/glassware campaign resume <case-id>
```

### Evidence Not Detected

```bash
# STOP immediately
./target/release/glassware campaign command <case-id> cancel

# Investigate evidence packages
./target/release/glassware scan-tarball evidence/iflow-mcp.tgz

# Check detector configuration
# Review logs for detector errors
```

### Rate Limiting

```bash
# Reduce rate limit
./target/release/glassware campaign command <case-id> set-rate-limit 5

# Reduce concurrency
./target/release/glassware campaign command <case-id> set-concurrency 10

# Resume campaign
./target/release/glassware campaign command <case-id> resume
```

---

## Campaign Templates

### Evidence Validation Campaign

```toml
[campaign]
name = "Evidence Validation"
priority = "critical"

[[waves]]
id = "evidence"
name = "Evidence Validation"

[[waves.sources]]
type = "packages"
list = [
    "iflow-mcp-watercrawl-mcp@1.3.4",
    "glassworm-combo-002",
    "glassworm-combo-003",
    "glassworm-combo-004",
]

[waves.expectations]
must_flag_all = true
min_threat_score = 7.0
```

### Clean Baseline Campaign

```toml
[campaign]
name = "Clean Baseline"
priority = "high"

[[waves]]
id = "clean"
name = "Clean Baseline"

[[waves.sources]]
type = "packages"
list = [
    "express@4.19.2",
    "lodash@4.17.21",
    # ... 200 more
]

[waves.expectations]
must_flag_all = false
max_threat_score = 5.0
max_malicious_count = 2
```

### Production Hunt Campaign

```toml
[campaign]
name = "Production Hunt"
priority = "high"

[[waves]]
id = "hunt"
name = "High-Risk Categories"

[[waves.sources]]
type = "npm_search"
keywords = ["react-native", "crypto", "wallet", "i18n"]
samples_per_keyword = 100
max_downloads = 100000

[waves.expectations]
min_threat_score = 7.0
```

---

## Best Practices

### 1. Start Small

```toml
# Start with evidence validation
[[waves]]
id = "wave_1"
name = "Evidence"
# ...

# Then clean baseline
[[waves]]
id = "wave_2"
name = "Clean Baseline"
# ...

# Finally production hunt
[[waves]]
id = "wave_3"
name = "Production Hunt"
# ...
```

### 2. Monitor Continuously

- Watch first wave completion before proceeding
- Check FP rate after clean baseline
- Adjust parameters based on results

### 3. Document Everything

- Campaign configuration
- Parameter adjustments
- FP analysis
- Evidence detection status

### 4. Clear Cache Between Runs

```bash
# Before re-running campaign
./target/release/glassware cache-clear
rm -rf .glassware-checkpoints*
```

---

## References

- [User Guide](USER-GUIDE.md)
- [Architecture](ARCHITECTURE.md)
- [Detectors](DETECTORS.md)
