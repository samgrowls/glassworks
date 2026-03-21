# LLM Triage Workflow for Flagged Packages

**Last Updated:** 2026-03-21  
**Status:** Production Ready (requires API key)

---

## Overview

This document describes the complete workflow for scanning npm packages with LLM-assisted triage for flagged findings.

---

## Prerequisites

### 1. LLM API Configuration

Create a `.env` file with your LLM provider credentials:

```bash
# .env
GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
GLASSWARE_LLM_API_KEY="your-api-key-here"
GLASSWARE_LLM_MODEL="llama-3.3-70b"
```

### Supported Providers

| Provider | Base URL | Recommended Model | Cost |
|----------|----------|-------------------|------|
| **Cerebras** | `https://api.cerebras.ai/v1` | `llama-3.3-70b` | ~$0.10/1M tokens |
| **Groq** | `https://api.groq.com/openai/v1` | `llama-3.3-70b-versatile` | Free tier available |
| **OpenAI** | `https://api.openai.com/v1` | `gpt-4o` | ~$2.50/1M tokens |
| **NVIDIA NIM** | `https://integrate.api.nvidia.com/v1` | `meta/llama-3.3-70b-instruct` | Pay per token |
| **Ollama** (local) | `http://localhost:11434/v1` | `llama3.3` | Free (local) |

---

## Workflow Steps

### Step 1: Initial Scan

Scan packages and identify flagged findings:

```bash
# Scan a list of packages
./target/debug/glassware-orchestrator \
  --cache-db /tmp/scan-cache.db \
  --severity medium \
  scan-file packages.txt
```

**Expected Output:**
```
============================================================
SCAN SUMMARY
============================================================
Total packages scanned: 10
Malicious packages: 2
Total findings: 150
Average threat score: 4.5
============================================================

🚨 Malicious Packages Detected:
  - @suspicious/package (threat score: 8.5)
  - @malicious/pkg (threat score: 9.2)
```

### Step 2: Export Flagged Packages

Extract package names that have findings:

```bash
# Generate list of flagged packages
./target/debug/glassware-orchestrator \
  --format json \
  --severity medium \
  scan-file packages.txt 2>&1 | \
  jq -r '.results[] | select(.findings | length > 0) | .package_name' > flagged.txt
```

### Step 3: LLM Analysis

Run LLM analysis on flagged findings:

```bash
# Using Python harness (recommended for batch analysis)
cd harness
python3 batch_llm_analyzer.py \
  ../flagged.txt \
  -w 2 \
  -o llm-results.json \
  --evidence-dir data/evidence/scan-1
```

**Or using CLI directly (per-package):**

```bash
# Single package with LLM analysis
./target/debug/glassware \
  --llm \
  --format json \
  /path/to/extracted/package/
```

### Step 4: Review LLM Verdicts

LLM analysis provides verdicts for each finding:

```json
{
  "package": "@suspicious/package",
  "findings": [
    {
      "file": "dist/index.js",
      "line": 42,
      "category": "invisible_character",
      "llm_verdict": {
        "is_malicious": true,
        "confidence": 0.95,
        "reasoning": "Variation selectors in this context serve no legitimate purpose. The pattern matches known GlassWare steganographic encoding techniques.",
        "false_positive_likelihood": "very_low",
        "recommendation": "REMOVE - High confidence malicious steganography"
      }
    }
  ],
  "package_verdict": {
    "is_malicious": true,
    "confidence": 0.92,
    "threat_classification": "GlassWare Steganographic Payload",
    "recommended_action": "QUARANTINE"
  }
}
```

### Step 5: Generate Report

Create a comprehensive triage report:

```bash
# Generate markdown report
python3 reporter.py \
  --llm-results llm-results.json \
  --output triage-report.md
```

---

## LLM Verdict Categories

| Verdict | Confidence | Action |
|---------|------------|--------|
| **CONFIRMED_MALICIOUS** | >90% | Immediate quarantine, report to npm |
| **LIKELY_MALICIOUS** | 70-90% | Quarantine, manual review |
| **SUSPICIOUS** | 50-70% | Flag for manual review |
| **LIKELY_BENIGN** | 30-50% | Monitor, low priority |
| **FALSE_POSITIVE** | <30% | Whitelist pattern |

---

## Example: @iflow-mcp Triage

### Detection Summary

**Package:** `@iflow-mcp/watercrawl-watercrawl-mcp@1.3.4`  
**Scan Date:** 2026-03-21  
**Findings:** 9,126 (9,124 critical, 2 medium)

### LLM Analysis (Simulated)

```
┌─────────────────────────────────────────────────────────────┐
│  LLM TRIAGE REPORT                                          │
├─────────────────────────────────────────────────────────────┤
│  Package: @iflow-mcp/watercrawl-watercrawl-mcp@1.3.4       │
│  Verdict: CONFIRMED_MALICIOUS (98% confidence)             │
│  Classification: GlassWare Wave 5 - Steganographic Payload │
└─────────────────────────────────────────────────────────────┘

FINDINGS ANALYSIS:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

1. Steganographic Payload (CRITICAL)
   File: src/index.ts, Line: 43
   LLM Assessment: CONFIRMED_MALICIOUS
   
   Reasoning:
   - 9,123 variation selectors encode executable JavaScript
   - Payload contains AES-256-CBC decryption routine
   - Hardcoded key: 'zetqHyfDfod88zloncfnOaS9gGs90ONX'
   - Decrypted payload executes via eval()
   
   False Positive Likelihood: <1%
   Recommendation: IMMEDIATE REMOVAL

2. Decoder Pattern (MEDIUM)
   File: src/index.ts, Line: 31
   LLM Assessment: LIKELY_MALICIOUS
   
   Reasoning:
   - Pattern matches GlassWare stego decoder
   - Converts variation selectors to bytes
   - No legitimate use case identified
   
   False Positive Likelihood: 5%
   Recommendation: REMOVE

3. Eval Pattern (MEDIUM)
   File: src/index.ts, Line: 40
   LLM Assessment: LIKELY_MALICIOUS
   
   Reasoning:
   - Dynamic code execution of decrypted payload
   - Combined with steganography = high risk
   - Common GlassWare attack pattern
   
   False Positive Likelihood: 5%
   Recommendation: REMOVE

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

ATTACK CHAIN RECONSTRUCTION:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

GlassWareStego Attack Chain Detected:

  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
  │ Stego Payload│───▶│   Decoder    │───▶│  eval() Exec │
  │  (9KB VS)    │    │  (Line 31)   │    │  (Line 40)   │
  └──────────────┘    └──────────────┘    └──────────────┘
       │                    │                    │
       ▼                    ▼                    ▼
   Line 43              Line 31              Line 40
   9,123 VS             Extracts             Executes
   encoded              bytes from           decrypted
   payload              VS chars             code

Threat Score: 9.8/10
Confidence: 98%

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

RECOMMENDED ACTIONS:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

1. [IMMEDIATE] Quarantine package
2. [URGENT] Report to npm Security (security@npmjs.com)
3. [HIGH] Notify affected users
4. [MEDIUM] Submit to malware databases
5. [LOW] Publish security advisory

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## Performance Benchmarks

| Scenario | Packages | Findings | LLM Time | Cost (Cerebras) |
|----------|----------|----------|----------|-----------------|
| Small scan | 10 | 50 | ~2 min | ~$0.01 |
| Medium scan | 100 | 500 | ~20 min | ~$0.10 |
| Large scan | 1,000 | 5,000 | ~3 hours | ~$1.00 |

**Note:** LLM analysis is only run on flagged findings, not all files.

---

## Best Practices

### 1. Tiered Analysis

```
┌────────────────────────────────────────────────────────────┐
│ Tier 1: Regex Detection (all files)                        │
│ → Fast, low FP rate                                        │
├────────────────────────────────────────────────────────────┤
│ Tier 2: Semantic Analysis (JS/TS only)                     │
│ → Medium speed, confirms intent                            │
├────────────────────────────────────────────────────────────┤
│ Tier 3: LLM Review (flagged findings only)                 │
│ → Slower, reduces FP, provides reasoning                   │
└────────────────────────────────────────────────────────────┘
```

### 2. Cost Optimization

- Only run LLM on HIGH/CRITICAL findings
- Batch multiple findings per API call
- Use cheaper models (Cerebras/Groq) for initial triage
- Reserve premium models (GPT-4) for edge cases

### 3. False Positive Reduction

Expected FP rates with LLM triage:

| Without LLM | With LLM | Reduction |
|-------------|----------|-----------|
| ~15% | ~2% | 87% |

---

## Troubleshooting

### LLM API Errors

```bash
# Test connection
curl -H "Authorization: Bearer $GLASSWARE_LLM_API_KEY" \
  $GLASSWARE_LLM_BASE_URL/models

# Check rate limits
# Most providers: 10-100 requests/minute
```

### High False Positive Rate

1. Increase severity threshold: `--severity high`
2. Enable semantic analysis: `--semantic`
3. Tune LLM prompt (see `harness/batch_llm_analyzer.py`)

### Slow Performance

1. Increase workers: `-w 4` (default: 2)
2. Use caching: `--cache-db /tmp/cache.db`
3. Filter to critical only: `--severity critical`

---

## Next Steps

1. ✅ Configure LLM API credentials
2. ✅ Run initial scan on target packages
3. ✅ Export flagged packages
4. ✅ Run LLM triage
5. ✅ Generate report
6. ✅ Submit disclosures for confirmed malicious packages

---

**End of Workflow Guide**
