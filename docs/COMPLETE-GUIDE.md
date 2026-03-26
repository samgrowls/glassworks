# GlassWorm Detection System - Complete Guide

**Version:** v0.69.0
**Last Updated:** 2026-03-26
**Status:** Production Ready (FP Rate < 1%)

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Campaign Creation Guide](#campaign-creation-guide)
3. [Running Campaigns](#running-campaigns)
4. [False Positive Investigation](#false-positive-investigation)
5. [Detector Tuning Methodology](#detector-tuning-methodology)
6. [Architecture Overview](#architecture-overview)
7. [Troubleshooting](#troubleshooting)

---

## Quick Start

### Installation

```bash
# Clone and build
git clone https://github.com/samgrowls/glassworks.git
cd glassworks
cargo build --release

# Verify installation
./target/release/glassware --help
```

### Environment Setup

```bash
# Copy example environment
cp .env.example .env

# Configure LLM API keys (optional, for LLM analysis)
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="csk-..."
export GLASSWARE_LLM_MODEL="llama-3.3-70b"

# NVIDIA API for Tier 2 deep analysis (optional)
export NVIDIA_API_KEY="nvapi-..."
```

### First Scan

```bash
# Scan single package
./target/release/glassware scan-npm express@4.19.2

# Scan with LLM analysis
./target/release/glassware scan-npm express@4.19.2 --llm

# Scan evidence package
./target/release/glassware scan-tarball evidence/iflow-mcp.tgz
```

---

## Campaign Creation Guide

### What Is a Campaign?

A **campaign** is a large-scale scanning operation that processes hundreds or thousands of packages in organized waves with:
- Checkpoint/resume capability
- Progress monitoring via TUI
- Markdown/JSON report generation
- LLM-powered analysis

### Campaign File Structure

Create a TOML configuration file (e.g., `campaigns/wave17.toml`):

```toml
# ============================================================
# Campaign Metadata
# ============================================================
[campaign]
name = "Wave 17 - Large Scale Validation"
description = "Validate FP rate on 1000+ new packages"
created_by = "security-team"
priority = "high"
tags = ["validation", "wave17", "production"]

# ============================================================
# Global Settings
# ============================================================
[settings]
concurrency = 20              # Parallel package scans
rate_limit_npm = 10.0         # npm API rate limit (req/sec)
rate_limit_github = 5.0       # GitHub API rate limit (req/sec)
cache_enabled = true          # Enable scan result caching
cache_ttl_days = 7            # Cache expiration (days)

# ============================================================
# Scoring Configuration
# ============================================================
[settings.scoring]
malicious_threshold = 7.0     # Score >= 7.0 = malicious
suspicious_threshold = 4.0    # Score >= 4.0 = suspicious

# Tiered scoring - CRITICAL for FP prevention
[settings.scoring.tier_config]
mode = "tiered"

# Tier 1: Primary GlassWorm signals (ALWAYS runs)
[[settings.scoring.tiers]]
tier = 1
detectors = ["invisible_char", "glassware_pattern", "obfuscation"]
threshold = 0.0               # Always execute
weight_multiplier = 1.0

# Tier 2: Secondary confirmation (only if Tier 1 score >= 2.0)
[[settings.scoring.tiers]]
tier = 2
detectors = ["header_c2", "exfil_schema", "blockchain_c2"]
threshold = 2.0               # Only run if Tier 1 >= 2.0
weight_multiplier = 0.8

# Tier 3: Behavioral indicators (only if Tier 1+2 score >= 10.0)
[[settings.scoring.tiers]]
tier = 3
detectors = ["locale_geofencing", "time_delay_sandbox_evasion"]
threshold = 10.0              # Only run if Tier 1+2 >= 10.0
weight_multiplier = 0.8

# Conditional rules for high-confidence patterns
[[settings.scoring.conditional_rules]]
name = "glassworm_signature"
description = "Invisible + C2 = confirmed GlassWorm"
condition = "invisible_char.count >= 10 AND blockchain_c2.count >= 1"
action = "final_score = 25.0"

# Detector weights
[settings.scoring.weights]
invisible_char = 6.0
glassware_pattern = 10.0
obfuscation = 10.0
blockchain_c2 = 5.0
header_c2 = 4.0
exfil_schema = 6.0
locale_geofencing = 7.0
time_delay_sandbox_evasion = 7.0

# ============================================================
# LLM Configuration
# ============================================================
[settings.llm]
tier1_enabled = false         # Disable for FP testing
tier1_provider = "cerebras"
tier1_threshold = 6.0
tier2_enabled = false         # Enable for FP investigation
tier2_threshold = 8.0

# ============================================================
# Output Configuration
# ============================================================
[settings.output]
formats = ["json", "markdown"]
evidence_collection = true
evidence_dir = "evidence/wave17"
report_dir = "reports/wave17"

# ============================================================
# Wave 1: Evidence Validation (MUST detect 100%)
# ============================================================
[[waves]]
id = "wave_17a"
name = "Evidence Validation"
description = "Validate 100% detection on curated evidence"
depends_on = []
mode = "validate"

[[waves.sources]]
type = "packages"
list = [
    "iflow-mcp-watercrawl-watercrawl-mcp@1.3.4",
]

[waves.expectations]
must_flag_all = true
min_threat_score = 7.0
min_malicious_count = 1

[waves.reporting]
include_clean_summary = false

# ============================================================
# Wave 2: Clean Baseline (MUST NOT flag)
# ============================================================
[[waves]]
id = "wave_17b"
name = "Clean Baseline - 1000+ Packages"
description = "Measure FP rate on 1000+ clean packages"
depends_on = ["wave_17a"]
mode = "validate"

[[waves.sources]]
type = "packages"
list = [
    # Add 1000+ package specifications here
    "express@4.19.2",
    "lodash@4.17.21",
    # ...
]

[waves.expectations]
must_flag_all = false
max_threat_score = 5.0
max_malicious_count = 10      # Allow up to 1% FP rate

[waves.reporting]
include_clean_summary = true
```

### Wave Types

**1. Evidence Validation Wave**
- Purpose: Verify 100% detection of known GlassWorm attacks
- Mode: `validate`
- Expectation: `must_flag_all = true`

**2. Clean Baseline Wave**
- Purpose: Measure FP rate on known clean packages
- Mode: `validate`
- Expectation: `max_malicious_count = <1% of total>`

**3. Hunt Wave**
- Purpose: Discover new GlassWorm attacks in high-risk categories
- Mode: `hunt`
- Sources: `npm_search`, `github_search`

### Package Source Types

**Explicit Package List:**
```toml
[[waves.sources]]
type = "packages"
list = [
    "express@4.19.2",
    "lodash@4.17.21",
]
```

**npm Search:**
```toml
[[waves.sources]]
type = "npm_search"
keywords = ["react-native", "crypto", "wallet"]
samples_per_keyword = 50
max_downloads = 100000
```

**npm Category:**
```toml
[[waves.sources]]
type = "npm_category"
category = "ai-ml"
samples = 100
sort_by = "popularity"
```

**GitHub Search:**
```toml
[[waves.sources]]
type = "github_search"
query = "react-native-component"
max_results = 50
sort_by = "stars"
```

---

## Running Campaigns

### Start Campaign

```bash
# Run campaign
./target/release/glassware campaign run campaigns/wave17.toml

# Run with LLM analysis
./target/release/glassware campaign run campaigns/wave17.toml --llm

# Run with deep LLM analysis (Tier 2)
./target/release/glassware campaign run campaigns/wave17.toml --deep-llm
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

### Background Execution with Logging

```bash
# Start campaign in background with logging
mkdir -p logs
nohup ./target/release/glassware campaign run campaigns/wave17.toml > logs/wave17.log 2>&1 &

# Monitor progress
tail -f logs/wave17.log

# Check for completion
grep "Campaign completed" logs/wave17.log
```

### Generate Reports

```bash
# Generate markdown report
./target/release/glassware campaign report <case-id>

# Generate JSON report
./target/release/glassware campaign report <case-id> --format json

# Generate SARIF report (for GitHub)
./target/release/glassware campaign report <case-id> --format sarif

# Save to file
./target/release/glassware campaign report <case-id> --output reports/wave17-final.md
```

### Query Campaign with LLM

```bash
# Ask about campaign
./target/release/glassware campaign query <case-id> "How many packages were flagged?"

# Ask about specific package
./target/release/glassware campaign query <case-id> "Why was package X flagged?"

# Ask for analysis
./target/release/glassware campaign query <case-id> "What patterns are common in flagged packages?"
```

---

## False Positive Investigation

### Step 1: Identify FP Package

```bash
# Get list of flagged packages
grep "flagged as malicious" logs/wave17.log

# Example output:
# Package casual@1.6.2 flagged as malicious (threat score: 8.50)
# Package three@0.160.0 flagged as malicious (threat score: 10.00)
```

### Step 2: Analyze Findings

```bash
# Scan package individually for detailed output
./target/release/glassware scan-npm casual@1.6.2

# Get JSON output for programmatic analysis
./target/release/glassware scan-npm casual@1.6.2 --format json > casual-analysis.json
```

### Step 3: Use LLM Tier 2 for Investigation

```bash
# Enable Tier 2 LLM for deep analysis
./target/release/glassware scan-npm casual@1.6.2 --deep-llm

# Or use campaign query
./target/release/glassware campaign query <case-id> "Analyze casual@1.6.2 - is this a true positive or false positive? Provide detailed reasoning."
```

### Step 4: Manual Code Review

```bash
# Download and extract package
npm pack casual@1.6.2
tar -xzf casual-1.6.2.tgz
cd package

# Review flagged files
# Look for:
# - Invisible Unicode characters
# - Obfuscation patterns
# - C2 communication patterns
```

### Step 5: Determine FP vs True Positive

**False Positive Indicators:**
- No invisible Unicode characters (Tier 1 signals)
- Build tool output (webpack, babel, etc.)
- Telemetry headers (X-Telemetry, X-Analytics)
- Legitimate SDK usage (blockchain, crypto)

**True Positive Indicators:**
- Invisible Unicode characters present
- Decoder patterns with VS-specific constants
- Known C2 wallets/IPs
- GlassWorm C2 signature (5-min polling)

### Step 6: Fix Detector (if FP)

**Example: Skip TypeScript Definition Files**

```rust
// In glassware-core/src/detectors/glassware.rs
fn detect_obfuscation_patterns(&self, content: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    // Skip TypeScript definition files (.d.ts) - type definitions, not code
    if file_path.ends_with(".d.ts") {
        return findings;
    }

    // ... rest of detection logic
}
```

**Example: Require Multiple Indicators**

```rust
// In glassware-core/src/detectors/blockchain_polling.rs
fn has_glassworm_c2_patterns(content: &str) -> bool {
    // Must have BOTH decodeCommand AND executeCommand (GlassWorm signature)
    let has_command_patterns = content.contains("decodeCommand") 
        && content.contains("executeCommand(");
    
    let has_wallet_patterns = glassworm_patterns[2..].iter()
        .any(|p| content.contains(p));

    has_command_patterns || has_wallet_patterns
}
```

---

## Detector Tuning Methodology

### Core Principle

**GlassWorm attacks MUST have invisible Unicode characters.** This is the defining characteristic.

Without Tier 1 signals (InvisibleCharacter, GlasswarePattern), the max score should be capped below the malicious threshold.

### Tuning Process

**1. Identify FP Pattern**
```bash
# Run wave campaign
./target/release/glassware campaign run campaigns/wave17.toml

# Analyze FPs
grep "flagged as malicious" logs/wave17.log
```

**2. Determine Root Cause**
- Check which detectors fired
- Check if Tier 1 signals present
- Identify detector specificity issue

**3. Implement Fix**
- Add context-aware detection
- Require multiple indicators
- Skip known benign patterns

**4. Validate Fix**
```bash
# Test on FP package
./target/release/glassware scan-npm <fp-package>

# Verify score reduced below threshold
# Expected: score < 7.0

# Test on evidence
./target/release/glassware scan-tarball evidence/iflow-mcp.tgz

# Verify evidence still detected
# Expected: score >= 7.0
```

**5. Re-run Wave**
```bash
# Clear cache
./target/release/glassware cache-clear

# Re-run wave
./target/release/glassware campaign run campaigns/wave17.toml
```

### Common FP Patterns and Fixes

**FP Pattern 1: Build Tool Output**
```rust
// Fix: Skip build directories
if file_path.contains("/dist/") || file_path.contains("/build/") {
    return findings;
}
```

**FP Pattern 2: Telemetry Headers**
```rust
// Fix: Whitelist known telemetry headers
const TELEMETRY_HEADERS = ["X-Telemetry", "X-Analytics", "X-Prisma-"];
if TELEMETRY_HEADERS.iter().any(|h| header.starts_with(h)) {
    continue;  // Skip telemetry
}
```

**FP Pattern 3: SDK Usage**
```rust
// Fix: Require specific C2 patterns, not generic SDK usage
if content.contains("decodeCommand") && content.contains("executeCommand(") {
    // GlassWorm signature
    findings.push(CRITICAL());
}
```

**FP Pattern 4: No Tier 1 Signals**
```rust
// Fix: Cap score without Tier 1 signals
if !has_tier1_signal {
    return score.min(3.5);  // Below suspicious threshold
}
```

---

## Architecture Overview

### System Components

```
┌─────────────────────────────────────────────────────────────┐
│                    glassware Binary                          │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ CLI Layer                                            │   │
│  │ scan-npm | scan-tarball | campaign | cache-*        │   │
│  └─────────────────────────────────────────────────────┘   │
│                          │                                  │
│  ┌───────────────────────▼───────────────────────────────┐ │
│  │ Orchestrator                                           │ │
│  │ Campaign Executor | Scanner | Checkpoint Manager      │ │
│  └───────────────────────────────────────────────────────┘ │
│                          │                                  │
│  ┌───────────────────────▼───────────────────────────────┐ │
│  │ glassware-core Library                                 │ │
│  │ Detectors (13+) | Scoring Engine | IR Builder         │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Detector Tiers

| Tier | Detectors | Purpose | Execution |
|------|-----------|---------|-----------|
| **Tier 1** | InvisibleCharacter, GlasswarePattern, Obfuscation | Primary GlassWorm indicators | Always runs |
| **Tier 2** | HeaderC2, ExfilSchema, BlockchainC2 | Secondary confirmation | Only if Tier 1 >= threshold |
| **Tier 3** | LocaleGeofencing, TimeDelay | Behavioral analysis | Only if Tier 1+2 >= threshold |

### Scoring System

**Category Caps (with Tier 1 signals):**
- 1 category: max 5.0
- 2 categories: max 7.0
- 3 categories: max 8.5
- 4+ categories: max 10.0

**Without Tier 1 signals:**
- Max score: 3.5 (below suspicious threshold)

---

## Troubleshooting

### Cache Issues

```bash
# Clear all caches
./target/release/glassware cache-clear
rm -rf .glassware-orchestrator-cache.db
rm -rf .glassware-checkpoints.db
```

### Campaign Stuck

```bash
# Check status
./target/release/glassware campaign status <case-id>

# Resume if paused
./target/release/glassware campaign command <case-id> resume

# Cancel and restart
./target/release/glassware campaign command <case-id> cancel
./target/release/glassware campaign run campaigns/wave17.toml
```

### Rate Limiting

```bash
# Reduce rate limit
./target/release/glassware campaign command <case-id> set-rate-limit 5

# Reduce concurrency
./target/release/glassware campaign command <case-id> set-concurrency 10
```

### High FP Rate

```bash
# 1. Identify FP packages
grep "flagged as malicious" logs/wave17.log

# 2. Analyze each FP
./target/release/glassware scan-npm <fp-package>

# 3. Check for Tier 1 signals
# If no InvisibleCharacter/GlasswarePattern, scoring cap should prevent FP

# 4. Fix detector specificity
# See Detector Tuning Methodology section

# 5. Re-run wave
./target/release/glassware cache-clear
./target/release/glassware campaign run campaigns/wave17.toml
```

---

## References

- [User Guide](USER-GUIDE.md)
- [Architecture](ARCHITECTURE.md)
- [Detectors](DETECTORS.md)
- [Campaign Operator Guide](CAMPAIGN-OPERATOR-GUIDE.md)

---

**Version:** v0.69.0
**Last Updated:** 2026-03-26
**Maintained By:** GlassWorm Detection Team
