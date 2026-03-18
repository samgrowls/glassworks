# glassware LLM Analyzer - Usage Guide

## Setup

### 1. Install Dependencies

```bash
cd llm-analyzer
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

### 2. Set API Key

```bash
export NVIDIA_API_KEY="nvapi-..."  # Your NVIDIA NIM API key
```

### 3. Test Connection

```bash
python test_connection.py
```

---

## Usage

### Analyze a Single Package

```bash
# After running glassware scan
./target/release/glassware --format json /path/to/package/ > scan_result.json

# Run LLM analyzer
python analyzer.py scan_result.json /path/to/package/ --output analysis.json
```

### Example Workflow

```bash
# 1. Download and extract package
cd /tmp
npm pack "@iflow-mcp/ref-tools-mcp@3.0.0"
tar -xzf iflow-mcp-ref-tools-mcp-3.0.0.tgz

# 2. Run glassware scan
/home/property.sightlines/samgrowls/glassworks/target/release/glassware \
  --format json package/ > ref-tools-scan.json

# 3. Run LLM analyzer
cd /home/property.sightlines/samgrowls/glassworks/llm-analyzer
python analyzer.py \
  /tmp/ref-tools-scan.json \
  /tmp/package/ \
  --output /tmp/ref-tools-analysis.json
```

---

## Output Format

```json
{
  "package_name": "@iflow-mcp/ref-tools-mcp",
  "package_version": "3.0.0",
  "scan_date": "2026-03-18T18:00:00Z",
  "total_findings": 17,
  "malicious_count": 15,
  "suspicious_count": 1,
  "false_positive_count": 1,
  "overall_classification": "MALICIOUS",
  "analyses": [
    {
      "finding": {...},
      "classification": "MALICIOUS",
      "confidence": 0.95,
      "reasoning": "RC4 cipher with 4/5 indicators...",
      "indicators": ["MOD_256", "XOR_OP", ...],
      "recommended_action": "REPORT_IMMEDIATELY"
    }
  ]
}
```

---

## Classification Logic

| Overall Classification | Criteria |
|----------------------|----------|
| `MALICIOUS` | >50% of findings classified as malicious |
| `SUSPICIOUS` | 20-50% malicious OR >50% suspicious |
| `FALSE_POSITIVE` | >80% false positives |
| `NEEDS_REVIEW` | Everything else |

---

## Next Steps

### Parallelization (Future)

```python
# TODO: Implement parallel analysis
from concurrent.futures import ThreadPoolExecutor

with ThreadPoolExecutor(max_workers=5) as executor:
    results = list(executor.map(analyze_finding, findings))
```

### Batch Processing (Future)

```python
# TODO: Process entire scan directory
for package in scan_results_dir.glob("*/scan_result.json"):
    analyze_package(package)
```

---

## Troubleshooting

### API Key Issues
```
Error: NVIDIA_API_KEY not set
```
**Fix:** `export NVIDIA_API_KEY="nvapi-..."`

### Rate Limiting
```
Error: 429 Too Many Requests
```
**Fix:** Add retry logic with backoff (TODO)

### Token Limits
```
Error: Request too large
```
**Fix:** Already truncates source context to 50K chars

---

**Model:** Qwen/Qwen3.5-397B-A17B via NVIDIA NIM  
**API URL:** https://integrate.api.nvidia.com/v1/chat/completions
