# glassware - Concurrent Scan & Analysis Guide

**Purpose:** Run multiple scans concurrently while analyzing results without conflicts

---

## Directory Structure

```
harness/data/evidence/
├── scan-001-mcp/           # Scan 1 evidence (tarballs)
├── scan-002-vscode/        # Scan 2 evidence (tarballs)
├── scan-003-random/        # Scan 3 evidence (tarballs)
├── llm-analysis-001/       # LLM analysis 1 (JSON results)
├── llm-analysis-002/       # LLM analysis 2 (JSON results)
└── batch-llm/              # Default LLM directory
```

---

## Running Concurrent Scans

### Scan 1: MCP Packages (Background)

```bash
cd harness
source .venv/bin/activate

# Start scan with custom evidence directory
python optimized_scanner.py mcp_packages.txt \
  --workers 10 \
  --evidence-dir data/evidence/scan-001-mcp \
  --output scan-001-results.json \
  --scan-name mcp &

# Store PID for monitoring
echo $! > scan-001.pid
```

### Scan 2: VS Code Extensions (Concurrent)

```bash
# Different evidence directory = no conflicts!
python optimized_scanner.py vscode_packages.txt \
  --workers 10 \
  --evidence-dir data/evidence/scan-002-vscode \
  --output scan-002-results.json \
  --scan-name vscode &

echo $! > scan-002.pid
```

### Scan 3: Random Sample (Concurrent)

```bash
python optimized_scanner.py random_packages.txt \
  --workers 10 \
  --evidence-dir data/evidence/scan-003-random \
  --output scan-003-results.json \
  --scan-name random &

echo $! > scan-003.pid
```

---

## Running LLM Analysis Concurrently

### Analyze Scan 1 Results

```bash
# Extract flagged packages from scan 1
cat scan-001-results.json | jq -r '.flagged_packages[].package' > flagged-001.txt

# Run LLM analysis with custom output directory
python batch_llm_analyzer.py flagged-001.txt \
  --workers 3 \
  --evidence-dir data/evidence/llm-analysis-001 \
  --output llm-analysis-001/results.json &

echo $! > llm-001.pid
```

### Analyze Scan 2 Results (Concurrent)

```bash
# Can run while scan 2 is still running!
cat scan-002-results.json | jq -r '.flagged_packages[].package' > flagged-002.txt

python batch_llm_analyzer.py flagged-002.txt \
  --workers 3 \
  --evidence-dir data/evidence/llm-analysis-002 \
  --output llm-analysis-002/results.json &

echo $! > llm-002.pid
```

---

## Monitoring Progress

### Check Scan Status

```bash
# Check if scan is still running
ps aux | grep -f scan-001.pid | grep -v grep

# View scan log
tail -f scan-001-results.json | jq '.scanned, .flagged'
```

### Check LLM Analysis Status

```bash
# Check if LLM analysis is running
ps aux | grep -f llm-001.pid | grep -v grep

# View partial results
cat llm-analysis-001/results.json | jq '.analyzed, .malicious, .suspicious'
```

---

## Environment Variables

Set these for global configuration:

```bash
# For scanner
export GLASSWARE_EVIDENCE_DIR="data/evidence/custom-scan"

# For LLM analyzer
export GLASSWARE_LLM_EVIDENCE_DIR="data/evidence/custom-llm"
```

---

## Example: Full Concurrent Workflow

```bash
#!/bin/bash
# concurrent-scan.sh - Run 3 scans + analyses concurrently

cd /path/to/glassworks/harness
source .venv/bin/activate

# Start 3 scans
python optimized_scanner.py mcp.txt -e data/evidence/scan-mcp -o scan-mcp.json &
python optimized_scanner.py vscode.txt -e data/evidence/scan-vscode -o scan-vscode.json &
python optimized_scanner.py random.txt -e data/evidence/scan-random -o scan-random.json &

# Wait for scans to complete
wait

# Extract flagged from each scan
jq -r '.flagged_packages[].package' scan-mcp.json > flagged-mcp.txt
jq -r '.flagged_packages[].package' scan-vscode.json > flagged-vscode.txt
jq -r '.flagged_packages[].package' scan-random.json > flagged-random.txt

# Start 3 LLM analyses
python batch_llm_analyzer.py flagged-mcp.txt -e data/evidence/llm-mcp -o llm-mcp/results.json &
python batch_llm_analyzer.py flagged-vscode.txt -e data/evidence/llm-vscode -o llm-vscode/results.json &
python batch_llm_analyzer.py flagged-random.txt -e data/evidence/llm-random -o llm-random/results.json &

# Wait for all analyses
wait

# Generate summary
echo "=== SCAN SUMMARY ==="
echo "MCP: $(jq '.malicious | length' llm-mcp/results.json) malicious"
echo "VSCode: $(jq '.malicious | length' llm-vscode/results.json) malicious"
echo "Random: $(jq '.malicious | length' llm-random/results.json) malicious"
```

---

## Best Practices

### 1. Use Descriptive Names

```bash
# Good
--evidence-dir data/evidence/scan-2026-03-18-mcp
--evidence-dir data/evidence/llm-2026-03-18-mcp

# Bad
--evidence-dir data/evidence/scan1
--evidence-dir data/evidence/test
```

### 2. Limit Concurrent Workers

```bash
# Scanner: 10 workers max (I/O bound)
--workers 10

# LLM: 3 workers max (API rate limit)
--workers 3
```

### 3. Monitor Disk Space

```bash
# Each scan ~50-100MB for 1000 packages
du -sh data/evidence/scan-*

# Clean up old scans
rm -rf data/evidence/scan-2026-03-1*
```

### 4. Backup Important Evidence

```bash
# Before cleanup, backup confirmed malicious
tar -czf evidence-backup-$(date +%Y%m%d).tar.gz \
  data/evidence/scan-mcp/*.tgz \
  data/evidence/llm-mcp/*_llm.json
```

---

## Troubleshooting

### Scan Hanging

```bash
# Check for stuck processes
ps aux | grep optimized_scanner | grep -v grep

# Kill if necessary
kill $(cat scan-001.pid)
```

### LLM API Rate Limit

```bash
# Reduce workers
python batch_llm_analyzer.py packages.txt --workers 1

# Or add delay between calls (modify code)
```

### Disk Space Full

```bash
# Find largest evidence directories
du -sh data/evidence/* | sort -rh | head -10

# Remove old scans
rm -rf data/evidence/scan-old-*
```

---

## Performance Estimates

| Operation | Speed | 1000 packages |
|-----------|-------|---------------|
| Scan (10 workers) | ~2.5s/pkg | ~42 minutes |
| LLM (3 workers) | ~30s/pkg | ~8 hours |
| LLM (10 workers)* | ~10s/pkg | ~3 hours |

*Limited by NVIDIA API rate limit (30 RPM)

---

**Key Benefit:** Run scans and analyses in parallel without conflicts!
