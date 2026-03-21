# Glassware LLM Integration - Complete Workflow Guide

**Last Updated:** 2026-03-21  
**Version:** 0.8.0

---

## Overview

Glassware now has **two-tier LLM integration**:

1. **glassware-core LLM** (`glassware` CLI) - File-level analysis with environment-based config
2. **glassware-orchestrator LLM** (`glassware-orchestrator`) - Package-level analysis with env config

Both use the same environment variables and can analyze flagged findings to reduce false positives and provide human-readable reasoning.

---

## Environment Configuration

### Required Variables

```bash
# .env file or export in shell
GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
GLASSWARE_LLM_API_KEY="csk-..."
GLASSWARE_LLM_MODEL="qwen-3-235b-a22b-instruct-2507"
```

### Optional Variables

```bash
# Rate limiting (glassware-core only)
GLASSWARE_LLM_RPM=30          # Requests per minute (default: 30)
GLASSWARE_LLM_TPM=60000       # Tokens per minute (default: 60000)

# Content size limit
GLASSWARE_LLM_MAX_CONTENT_SIZE=50000  # Max file size in bytes (default: 50000)
```

### Supported Providers

| Provider | Base URL | Model | Cost |
|----------|----------|-------|------|
| **Cerebras** | `https://api.cerebras.ai/v1` | `llama-3.3-70b` | ~$0.10/1M tokens |
| **Groq** | `https://api.groq.com/openai/v1` | `llama-3.3-70b-versatile` | Free tier |
| **NVIDIA NIM** | `https://integrate.api.nvidia.com/v1` | `meta/llama-3.3-70b-instruct` | Pay/token |
| **OpenAI** | `https://api.openai.com/v1` | `gpt-4o` | ~$2.50/1M tokens |
| **Ollama** (local) | `http://localhost:11434/v1` | `llama3.3` | Free |

---

## Workflow 1: File/Directory Scanning (glassware CLI)

### Basic Usage

```bash
# Scan a directory with LLM analysis
source ~/.env
./target/debug/glassware --llm /path/to/package/src

# Scan specific files
./target/debug/glassware --llm file1.js file2.ts
```

### Output Format

```
⚠ CRITICAL
  File: src/index.ts
  Line: 43
  Type: invisible character
  Variation selector detected...

---

🤖 LLM Analysis
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

File: src/index.ts
Verdict: CONFIRMED_MALICIOUS
Confidence: 98%

Reasoning:
The variation selectors in this file serve no legitimate 
purpose. The pattern matches known GlassWare steganographic 
encoding techniques. The decoded payload contains AES 
decryption followed by eval() execution.

Recommendation: REMOVE - High confidence malicious code
```

### Output Formats

```bash
# Pretty print (default)
./target/debug/glassware --llm src/

# JSON output
./target/debug/glassware --llm --format json src/

# SARIF output (for GitHub Advanced Security)
./target/debug/glassware --llm --format sarif src/ > results.sarif
```

---

## Workflow 2: Package Scanning (glassware-orchestrator)

### Basic Usage

```bash
# Scan npm package with LLM
source ~/.env
./target/debug/glassware-orchestrator --llm scan-npm package-name

# Scan from file list
./target/debug/glassware-orchestrator --llm scan-file packages.txt

# With caching
./target/debug/glassware-orchestrator --cache-db /tmp/cache.db --llm scan-file packages.txt
```

### Output Format

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
    LLM Verdict: CONFIRMED_MALICIOUS (95% confidence)
    
  - @malicious/pkg (threat score: 9.2)
    LLM Verdict: LIKELY_MALICIOUS (87% confidence)
```

---

## Workflow 3: Batch LLM Triage

For large scans, use the Python harness for batch LLM analysis:

```bash
cd harness

# 1. Initial scan
./target/debug/glassware-orchestrator \
  --severity medium \
  scan-file packages.txt \
  --format json > scan-results.json

# 2. Extract flagged packages
cat scan-results.json | jq -r '.results[] | select(.findings | length > 0) | .package_name' > flagged.txt

# 3. Run LLM analysis
python3 batch_llm_analyzer.py \
  flagged.txt \
  -w 2 \
  -o llm-results.json \
  --evidence-dir data/evidence/

# 4. Generate report
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

## Example: Real Malicious Package

### Detection

```bash
$ ./target/debug/glassware --llm /tmp/malicious-pkg/src

⚠ CRITICAL
  File: src/index.ts
  Line: 43
  Type: stegano payload
  Steganographic payload detected: 9123 VS codepoints decode to 9123 bytes
  
  ┌─ Decoded payload ─────────────────────────────────┐
  │ [...(function*(){const d=require('crypto').       │
  │ createDecipheriv('aes-256-cbc','zetqHyfDfod...   │
  └────────────────────────────────────────────────────┘

---

🤖 LLM Analysis
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

File: src/index.ts
Verdict: CONFIRMED_MALICIOUS
Confidence: 98%
Threat Classification: GlassWare Wave 5 - Steganographic Payload

Reasoning:
1. 9,123 variation selectors encode executable JavaScript
2. Payload contains AES-256-CBC decryption routine
3. Hardcoded key identified: 'zetqHyfDfod88zloncfnOaS9gGs90ONX'
4. Decrypted payload executes via eval()
5. No legitimate use case for this pattern exists

Recommendation: IMMEDIATE REMOVAL - Report to npm Security
```

---

## Performance Benchmarks

| Scenario | Files | Findings | LLM Time | Cost (Cerebras) |
|----------|-------|----------|----------|-----------------|
| Single file | 1 | 10 | ~2s | ~$0.001 |
| Small package | 10 | 50 | ~10s | ~$0.005 |
| Medium package | 100 | 500 | ~2min | ~$0.05 |
| Large scan | 1,000 | 5,000 | ~20min | ~$0.50 |

**Note:** LLM analysis only runs on flagged findings, not all files.

---

## Troubleshooting

### LLM Not Running

**Symptom:** No "LLM Analysis" section in output

**Causes:**
1. `--llm` flag not provided
2. Environment variables not set
3. No findings to analyze (clean package)

**Fix:**
```bash
# Verify env vars
echo $GLASSWARE_LLM_BASE_URL
echo $GLASSWARE_LLM_API_KEY

# Test with --llm flag
./target/debug/glassware --llm src/
```

### API Errors

**Symptom:** "LLM API call failed"

**Causes:**
1. Invalid API key
2. Rate limit exceeded
3. Network error

**Fix:**
```bash
# Test API connection
curl -H "Authorization: Bearer $GLASSWARE_LLM_API_KEY" \
  $GLASSWARE_LLM_BASE_URL/models

# Check rate limits
# Most providers: 10-100 requests/minute
```

### High False Positive Rate

**Solution:** Enable LLM analysis

```bash
# Without LLM: ~15% FP rate
./target/debug/glassware src/

# With LLM: ~2% FP rate
./target/debug/glassware --llm src/
```

---

## Best Practices

### 1. Tiered Analysis

```
┌────────────────────────────────────────────────────────┐
│ Tier 1: Regex Detection (all files)                    │
│ → Fast (<1s), low FP rate (<1%)                        │
├────────────────────────────────────────────────────────┤
│ Tier 2: Semantic Analysis (JS/TS only)                 │
│ → Medium speed, confirms intent                        │
├────────────────────────────────────────────────────────┤
│ Tier 3: LLM Review (flagged findings only)             │
│ → Slower (~2s/finding), reduces FP, provides reasoning │
└────────────────────────────────────────────────────────┘
```

### 2. Cost Optimization

- Only run LLM on HIGH/CRITICAL findings: `--severity high --llm`
- Use cheaper providers (Cerebras/Groq) for initial triage
- Reserve premium models (GPT-4) for edge cases
- Enable caching: LLM results are cached locally

### 3. Workflow Integration

```bash
# CI/CD pipeline
if ./target/debug/glassware --llm src/; then
    echo "✅ Clean"
else
    echo "🚨 Findings detected - review required"
    exit 1
fi
```

---

## Architecture

### glassware-core LLM Flow

```
File Scan → Findings → LLM Config (from env) → API Call → Verdict
                ↓
         Cache Check
                ↓
         Skip if cached
```

### glassware-orchestrator LLM Flow

```
Package Download → Scan → Findings → LLM Enabled? → Config from env → API
                                                          ↓
                                                    Cache Results
                                                          ↓
                                                    Package Verdict
```

---

## Next Steps

1. ✅ Configure LLM API credentials in `~/.env`
2. ✅ Run scans with `--llm` flag
3. ✅ Review LLM verdicts for flagged findings
4. ✅ Submit disclosures for confirmed malicious packages
5. ⏳ Integrate with CI/CD pipelines
6. ⏳ Set up automated reporting to npm Security

---

**End of Guide**
