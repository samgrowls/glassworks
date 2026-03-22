# GlassWorm Usage Guide

**Version:** v0.11.0+  
**Last Updated:** 2026-03-21

---

## Quick Start

### Installation

```bash
# Clone repository
git clone https://github.com/samgrowls/glassworks.git
cd glassworks

# Build CLI
cargo build -p glassware-cli --release

# Build orchestrator
cargo build -p glassware-orchestrator --release

# Binaries located at:
# - target/release/glassware
# - target/release/glassware-orchestrator
```

### Basic Usage

```bash
# Scan a directory
./target/release/glassware /path/to/project

# Scan npm packages
./target/release/glassware-orchestrator scan-npm express lodash axios

# Scan tarball files
./target/release/glassware-orchestrator scan-tarball package-1.0.0.tgz
```

---

## glassware CLI

### Commands

```bash
# Scan directory
glassware [OPTIONS] <PATHS>...

# Options:
#   --format <FORMAT>        Output format: pretty, json, sarif [default: pretty]
#   --severity <SEVERITY>    Minimum severity: info, low, medium, high, critical [default: low]
#   --quiet                  Suppress output, only set exit code
#   --extensions <EXTS>      File extensions to include [default: js,mjs,cjs,ts,tsx,jsx,py,rs,...]
#   --exclude <DIRS>         Directories to exclude [default: .git,node_modules,target,...]
#   --cache-file <PATH>      Cache file for incremental scanning
#   --no-cache               Disable caching
#   --llm                    Enable LLM analysis (requires GLASSWARE_LLM_*)
```

### Examples

```bash
# Scan current directory
glassware .

# Scan with JSON output
glassware --format json . > results.json

# Scan with SARIF output (GitHub Advanced Security)
glassware --format sarif . > results.sarif

# Only report HIGH and CRITICAL
glassware --severity high .

# Scan specific file types
glassware --extensions js,ts,tsx src/

# Exclude directories
glassware --exclude node_modules,build,dist .

# Enable incremental scanning
glassware --cache-file .glassware-cache.json .

# Run LLM analysis on flagged files
glassware --llm .
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | No findings at or above severity threshold |
| 1 | Findings detected |
| 2 | Error (file not found, permission denied) |

---

## glassware-orchestrator

### Commands

```bash
# Scan npm packages
glassware-orchestrator scan-npm <PACKAGES>... [--versions <POLICY>]

# Scan GitHub repositories
glassware-orchestrator scan-github <REPOS>... [--ref <REF>]

# Scan tarball files
glassware-orchestrator scan-tarball <FILES>...

# Search GitHub
glassware-orchestrator search-github <QUERY> [--max-results <N>]

# Sample packages by category
glassware-orchestrator sample-packages --category <CATEGORIES>... [--samples <N>]

# Show cache stats
glassware-orchestrator cache-stats [--clear]

# Resume interrupted scan
glassware-orchestrator resume <SOURCE> [--packages <PKGS>] [--repos <REPOS>]
```

### Common Options

```bash
# Global options:
#   --format <FORMAT>        Output format: pretty, json, sarif, jsonl
#   --severity <SEVERITY>    Minimum severity to report
#   --concurrency <N>        Maximum concurrent operations [default: 10]
#   --output <PATH>          Output file path
#   --llm                    Enable LLM analysis
#   --no-cache               Disable caching
#   --threat-threshold <N>   Threat score threshold for malicious [default: 7.0]
```

### Examples

```bash
# Scan npm packages
glassware-orchestrator scan-npm express lodash axios

# Scan specific versions
glassware-orchestrator scan-npm express --versions last-10

# Scan GitHub repository
glassware-orchestrator scan-github owner/repo

# Scan specific branch
glassware-orchestrator scan-github owner/repo --ref main

# Scan tarball files
glassware-orchestrator scan-tarball package-1.0.0.tgz package-2.0.0.tgz

# Search GitHub for MCP servers
glassware-orchestrator search-github "mcp server" --max-results 50

# Sample AI/ML packages
glassware-orchestrator sample-packages --category ai-ml native-build --samples 50

# Scan with LLM analysis
glassware-orchestrator --llm scan-npm suspicious-pkg

# Output to file
glassware-orchestrator --format json --output results.json scan-npm pkg1 pkg2
```

---

## Python Harness

### Setup

```bash
cd harness

# Install dependencies (if needed)
pip install requests aiohttp sqlite3

# Set environment variables
export NVIDIA_API_KEY="nvapi-..."
export NVIDIA_MODELS="qwen/qwen3.5-397b-a17b,..."
```

### Commands

```bash
# Run wave campaign
python3 -m core.orchestrator run-wave --wave <WAVE_ID> [--llm]

# Show status
python3 -m core.orchestrator status [--wave <WAVE_ID>]

# Generate report
python3 -m core.orchestrator report --wave <WAVE_ID>
```

### Examples

```bash
# Run Wave 0 (calibration)
python3 -m core.orchestrator run-wave --wave 0

# Run with LLM analysis
python3 -m core.orchestrator run-wave --wave 0 --llm

# Check status
python3 -m core.orchestrator status

# Generate report
python3 -m core.orchestrator report --wave 0
```

---

## LLM Integration

### Cerebras (Rust CLI)

```bash
# Set environment variables
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="csk-..."
export GLASSWARE_LLM_MODEL="llama-3.3-70b"

# Run with LLM
glassware --llm /path/to/scan
glassware-orchestrator --llm scan-npm pkg1 pkg2
```

### NVIDIA (Python Harness)

```bash
# Set environment variables
export NVIDIA_API_KEY="nvapi-..."
export NVIDIA_BASE_URL="https://integrate.api.nvidia.com/v1"
export NVIDIA_MODELS="qwen/qwen3.5-397b-a17b,moonshotai/kimi-k2.5,z-ai/glm5,meta/llama-3.3-70b-instruct"

# Run with LLM
python3 -m core.orchestrator run-wave --wave 0 --llm
```

### Supported Providers

| Provider | Base URL | Recommended Model |
|----------|----------|-------------------|
| Cerebras | `https://api.cerebras.ai/v1` | `llama-3.3-70b` |
| NVIDIA NIM | `https://integrate.api.nvidia.com/v1` | `qwen/qwen3.5-397b-a17b` |
| Groq | `https://api.groq.com/openai/v1` | `llama-3.3-70b-versatile` |
| OpenAI | `https://api.openai.com/v1` | `gpt-4o` |
| Ollama (local) | `http://localhost:11434/v1` | `llama3.3` |

---

## Output Formats

### Pretty (Default)

```
⚠ CRITICAL
  File: src/index.js
  Line: 42
  Type: glassware pattern
  GlassWare attack pattern detected: eval_pattern (confidence: 95%)
  CRITICAL: This code exhibits strong GlassWare attack characteristics.
---

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
2 findings in 15 files (1 critical, 0 high, 1 medium, 0 low)
Scanned 15 files in 0.25s
```

### JSON

```json
{
  "findings": [
    {
      "file": "src/index.js",
      "line": 42,
      "severity": "critical",
      "category": "GlasswarePattern",
      "description": "GlassWare attack pattern detected"
    }
  ],
  "summary": {
    "files_scanned": 15,
    "findings_total": 2
  }
}
```

### SARIF (GitHub Advanced Security)

```json
{
  "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
  "runs": [
    {
      "tool": {
        "driver": {
          "name": "glassware",
          "version": "0.11.0"
        }
      },
      "results": [
        {
          "ruleId": "GW005",
          "level": "error",
          "message": {
            "text": "GlassWare attack pattern detected"
          },
          "locations": [
            {
              "physicalLocation": {
                "artifactLocation": {
                  "uri": "src/index.js"
                },
                "region": {
                  "startLine": 42
                }
              }
            }
          ]
        }
      ]
    }
  ]
}
```

---

## Threat Scores

| Score | Classification | Action |
|-------|---------------|--------|
| 0-3 | Clean | No action |
| 3-6 | Suspicious | Flag for review |
| 6-10 | Likely malicious | Quarantine |
| 10+ | Confirmed malicious | Block + report |

### Example Output

```
============================================================
SCAN SUMMARY
============================================================
Total packages scanned: 3
Malicious packages: 1
Total findings: 15
Average threat score: 4.33
============================================================

⚠️  MALICIOUS PACKAGES DETECTED:
  - suspicious-pkg (1.0.0) [threat score: 10.00]

❌ Malicious packages detected (15 findings below threshold)
```

---

## Troubleshooting

### "Package not found"

```bash
# Check package name and version
npm view package-name versions

# Use available version
glassware-orchestrator scan-npm package-name@1.0.0
```

### "LLM analysis failed"

```bash
# Verify API key
echo $GLASSWARE_LLM_API_KEY
echo $NVIDIA_API_KEY

# Test connection
curl -H "Authorization: Bearer $NVIDIA_API_KEY" \
  $NVIDIA_BASE_URL/models
```

### "Rate limit exceeded"

```bash
# Reduce concurrency
glassware-orchestrator --concurrency 5 scan-npm pkg1 pkg2

# Reduce rate limit
glassware-orchestrator --npm-rate-limit 2 scan-npm pkg1 pkg2
```

### High false positive rate

```bash
# Increase severity threshold
glassware --severity high .

# Exclude specific directories
glassware --exclude node_modules,build,dist .

# Review whitelist settings
# (moment, prettier, typescript, crypto packages are whitelisted)
```

---

## References

- [GlassWorm Writeup](https://codeberg.org/tip-o-deincognito/glassworm-writeup)
- [Detection Capabilities](detection-capabilities.md)
- [Architecture](architecture.md)
