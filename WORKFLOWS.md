# Glassware Workflows - Complete User Guide

**Version:** v0.8.8.5  
**Last Updated:** 2026-03-21

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Workflow 1: Single Package Scan](#workflow-1-single-package-scan)
3. [Workflow 2: Version History Scan](#workflow-2-version-history-scan)
4. [Workflow 3: Large-Scale Background Scan](#workflow-3-large-scale-background-scan)
5. [Workflow 4: Targeted Agent Package Scan](#workflow-4-targeted-agent-package-scan)
6. [Workflow 5: GitHub Repository Scan](#workflow-5-github-repository-scan)
7. [Troubleshooting](#troubleshooting)
8. [Best Practices](#best-practices)

---

## Quick Start

### Installation

```bash
# Clone repository
git clone https://github.com/samgrowls/glassworks.git
cd glassworks

# Build
cargo build --release -p glassware-orchestrator

# Verify installation
./target/release/glassware-orchestrator --version
```

### Environment Setup (Optional - for LLM analysis)

```bash
# Create .env file
cat > .env << 'EOF'
GLASSWARE_LLM_BASE_URL=https://api.cerebras.ai/v1
GLASSWARE_LLM_API_KEY=csk-your-api-key-here
GLASSWARE_LLM_MODEL=llama-3.3-70b
EOF

# Or export directly
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="csk-..."
```

---

## Workflow 1: Single Package Scan

**Use Case:** Quick security check of a specific package

### Step-by-Step

```bash
# Navigate to glassworks directory
cd /path/to/glassworks

# Scan a single package
./target/release/glassware-orchestrator scan-npm express

# Expected output:
# ============================================================
# SCAN SUMMARY
# ============================================================
# Total packages scanned: 1
# Malicious packages: 0
# Total findings: 0
# Average threat score: 0.00
# ============================================================
# ✅ No security issues detected
```

### With Version History

```bash
# Scan last 10 versions
./target/release/glassware-orchestrator scan-npm --versions last-10 express

# Expected output:
# ============================================================
# VERSION SCAN SUMMARY
# ============================================================
# Packages scanned: 1
# Total versions: 10
# Total findings: 0
# Malicious versions: 0
# ============================================================
```

### With LLM Analysis

```bash
# Ensure LLM env vars are set
source .env

# Scan with LLM analysis
./target/release/glassware-orchestrator --llm scan-npm suspicious-pkg
```

### Expected Duration

| Operation | Time |
|-----------|------|
| Single package | ~0.5s |
| 10 versions | ~2s |
| With LLM | +5s per finding |

---

## Workflow 2: Version History Scan

**Use Case:** Detect when malicious code was introduced in package versions

### Step-by-Step

```bash
cd /path/to/glassworks

# Scan last 20 versions of a package
./target/release/glassware-orchestrator \
  scan-npm \
  --versions last-20 \
  lodash

# Output shows each version being scanned:
# 2026-03-21T07:00:00Z  INFO Scanning 1/20: lodash@4.17.21
# 2026-03-21T07:00:01Z  INFO   lodash@4.17.21: 0 findings, threat score: 0.00
# ...
```

### Interpreting Results

**Clean Package:**
```
VERSION SCAN SUMMARY
============================================================
Packages scanned: 1
Total versions: 20
Total findings: 0
Malicious versions: 0
============================================================
✅ No security issues detected
```

**Suspicious Package:**
```
VERSION SCAN SUMMARY
============================================================
Packages scanned: 1
Total versions: 20
Total findings: 15
Malicious versions: 2
============================================================
🚨 Malicious versions detected!
```

### Expected Duration

| Versions | Time |
|----------|------|
| last-5 | ~1s |
| last-10 | ~2s |
| last-20 | ~4s |
| last-50 | ~10s |

---

## Workflow 3: Large-Scale Background Scan

**Use Case:** Scan hundreds of packages with checkpoint/resume support

### Step 1: Sample Packages

```bash
cd /path/to/glassworks/harness

# Sample packages from specific categories
python3 version_sampler.py \
  --output packages-500.txt \
  --samples 50 \
  --categories ai-ml native-build web-frameworks utils crypto \
  --days 30 \
  --include-popular

# Expected output:
# ======================================================================
# SAMPLING COMPLETE
# ======================================================================
# Total packages: 250
# Output: packages-500.txt
# ======================================================================
```

### Step 2: Run Background Scan

```bash
# Start scan with checkpointing
python3 background_scanner.py \
  --packages packages-500.txt \
  --policy last-10 \
  --output results.db \
  --state scan-state.json \
  --log scan.log \
  --workers 10

# Expected output:
# Initialized database: results.db
# Starting scan of 250 packages
# Policy: last-10
# Workers: 10
# ======================================================================
#   Scanning 10 versions of package-1...
# [2026-03-21T07:00:00] Package 1/250 | Versions: 10 (failures: 10) | ...
# ...
```

### Step 3: Monitor Progress

**Terminal 1 - Watch log:**
```bash
tail -f scan.log
```

**Terminal 2 - Check state:**
```bash
watch -n 5 'cat scan-state.json | jq .versions_scanned'
```

**Terminal 3 - Query database:**
```bash
# Install sqlite3 if needed
# Ubuntu: sudo apt install sqlite3
# macOS: brew install sqlite3

sqlite3 results.db "SELECT COUNT(*) FROM version_scans;"
```

### Step 4: Analyze Results

```bash
# Find malicious versions
sqlite3 results.db <<EOF
SELECT package_name, version, threat_score, findings_count
FROM version_scans
WHERE is_malicious = 1
ORDER BY threat_score DESC;
EOF

# View auto-generated report
cat version-scan-report.md
```

### Expected Duration

| Packages | Versions | Workers | Time |
|----------|----------|---------|------|
| 100 | 1,000 | 5 | ~5 min |
| 250 | 2,500 | 10 | ~10 min |
| 500 | 5,000 | 10 | ~20 min |
| 1,000 | 10,000 | 20 | ~40 min |

### Resume Interrupted Scan

```bash
# If scan is interrupted, resume from checkpoint
python3 background_scanner.py \
  --packages packages-500.txt \
  --policy last-10 \
  --output results.db \
  --state scan-state.json \
  --log scan.log \
  --workers 10 \
  --resume
```

---

## Workflow 4: Targeted Agent Package Scan

**Use Case:** Scan packages commonly used by AI agents (high-risk targets)

### Step 1: Create Target List

```bash
cd /path/to/glassworks/harness

# Create file with agent-related packages
cat > agent-packages.txt << 'EOF'
@modelcontextprotocol/inspector
@anthropic-ai/mcp-server
langchain
@langchain/core
@langchain/openai
@langchain/anthropic
openai
anthropic
ai
@ai-sdk/openai
@ai-sdk/anthropic
@vercel/ai
llamaindex
@e2b/code-interpreter
e2b
crewai
@crewai/crewai
autoagent
@autoagent/core
babyagi
agentops
@agentops/agentops
smolagents
@huggingface/smolagents
litellm
@litellm/core
EOF
```

### Step 2: Scan with Version History

```bash
# Scan last 20 versions of each agent package
./target/release/glassware-orchestrator \
  scan-file agent-packages.txt \
  --versions last-20

# Or use background scanner for more thorough scan
python3 background_scanner.py \
  --packages agent-packages.txt \
  --policy last-20 \
  --output agent-scan-results.db \
  --workers 5
```

### Step 3: Analyze for GlassWorm Patterns

```bash
# Look for specific patterns in results
sqlite3 agent-scan-results.db <<EOF
SELECT package_name, version, threat_score
FROM version_scans
WHERE threat_score > 5.0
ORDER BY threat_score DESC;
EOF
```

### Why These Packages?

GlassWorm attackers target:
1. **MCP servers** - Direct access to AI agent tools
2. **LangChain** - Popular agent framework
3. **AI SDKs** - Core infrastructure
4. **Code interpreters** - Execution environment
5. **Agent frameworks** - Control plane

---

## Workflow 5: GitHub Repository Scan

**Use Case:** Scan GitHub repositories for malicious code

### Prerequisites

```bash
# Get GitHub token (optional, increases rate limits)
export GITHUB_TOKEN="ghp_your-token-here"

# Or create .env file
echo "GITHUB_TOKEN=ghp_..." >> .env
```

### Step-by-Step

```bash
cd /path/to/glassworks

# Scan a single repository
./target/release/glassware-orchestrator \
  scan-github \
  owner/repo

# Scan with specific branch
./target/release/glassware-orchestrator \
  scan-github \
  -r main \
  owner/repo

# Scan multiple repositories
cat > repos.txt << 'EOF'
anthropics/mcp-servers
langchain-ai/langchain
vercel/ai
EOF

# Note: GitHub scanning requires repository access
# Public repos work without token
# Private repos require GITHUB_TOKEN
```

### Expected Duration

| Repo Size | Files | Time |
|-----------|-------|------|
| Small | <100 | ~1s |
| Medium | 100-1,000 | ~5s |
| Large | 1,000-10,000 | ~30s |
| Very Large | >10,000 | ~2min |

---

## Troubleshooting

### npm 404 Errors

**Error:** `Package not found: pkg@1.0.0`

**Cause:** Old versions unpublished from npm

**Fix:** Use `last-N` policy
```bash
# Good
--policy last-10

# Avoid
--policy all
```

### Rate Limiting

**Error:** `429 Too Many Requests`

**Fix:** Reduce workers
```bash
python3 background_scanner.py --workers 3 ...
```

### LLM Not Working

**Error:** `--llm requires GLASSWARE_LLM_BASE_URL`

**Fix:** Set environment variables
```bash
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="csk-..."
```

### Database Locked

**Error:** `database is locked`

**Fix:** Close other connections or use WAL mode
```bash
sqlite3 results.db "PRAGMA journal_mode = WAL;"
```

### Scan Timeout

**Error:** `Timeout after 120s`

**Fix:** Increase timeout in `background_scanner.py` (line 171)
```python
timeout=300  # Change from 120 to 300
```

---

## Best Practices

### 1. Start Small

```bash
# Test with single package first
./target/release/glassware-orchestrator scan-npm express

# Then scale up
python3 background_scanner.py --packages packages-100.txt ...
```

### 2. Use Caching

```bash
# Enable caching for 20x speedup on re-scans
./target/release/glassware-orchestrator \
  --cache-db cache.json \
  scan-npm lodash

# Re-scan uses cache automatically
```

### 3. Monitor Long Scans

```bash
# Run in background with nohup
nohup python3 background_scanner.py \
  --packages packages-500.txt \
  --output results.db \
  > scan-output.log 2>&1 &

# Save PID for later
echo $! > scanner.pid

# Check progress
cat scan-state.json | jq .versions_scanned
```

### 4. Export Results

```bash
# Export malicious versions to CSV
sqlite3 -header -csv results.db \
  "SELECT package_name, version, threat_score FROM version_scans WHERE is_malicious = 1;" \
  > malicious-versions.csv

# Export to JSON
sqlite3 results.db \
  "SELECT json_group_array(json_object('package', package_name, 'version', version)) FROM version_scans WHERE is_malicious = 1;" \
  > malicious-versions.json
```

### 5. Set Up Alerts

```bash
# Create watch script
cat > watch-malicious.sh << 'EOF'
#!/bin/bash
while true; do
  count=$(sqlite3 results.db "SELECT COUNT(*) FROM version_scans WHERE is_malicious = 1;")
  if [ "$count" -gt 0 ]; then
    echo "🚨 ALERT: $count malicious versions found!"
    sqlite3 results.db "SELECT package_name, version FROM version_scans WHERE is_malicious = 1;"
  fi
  sleep 60
done
EOF

chmod +x watch-malicious.sh
./watch-malicious.sh &
```

---

## Performance Tuning

### Optimal Worker Count

| System | Workers |
|--------|---------|
| 2 CPU, 4GB RAM | 3-5 |
| 4 CPU, 8GB RAM | 5-10 |
| 8 CPU, 16GB RAM | 10-20 |
| 16+ CPU, 32GB+ RAM | 20-50 |

### Batch Processing

```bash
# Split large package lists
split -l 100 packages-500.txt batch-

# Process batches in parallel
for batch in batch-*; do
    python3 background_scanner.py \
      --packages "$batch" \
      --output "${batch}.db" \
      --workers 5 &
done
wait

# Merge results
for db in batch-*.db; do
    sqlite3 merged.db "ATTACH '$db' AS db; INSERT INTO version_scans SELECT * FROM db.version_scans; DETACH db;"
done
```

---

## Quick Reference

### Common Commands

```bash
# Single package
./target/release/glassware-orchestrator scan-npm express

# Version history
./target/release/glassware-orchestrator scan-npm --versions last-10 lodash

# File list
./target/release/glassware-orchestrator scan-file packages.txt

# Background scan
python3 background_scanner.py --packages packages.txt --output results.db

# Check scan history
./target/release/glassware-orchestrator scan-list

# Show scan details
./target/release/glassware-orchestrator scan-show <scan-id>
```

### Common Queries

```sql
-- Find malicious versions
SELECT package_name, version, threat_score
FROM version_scans
WHERE is_malicious = 1;

-- Findings by package
SELECT package_name, SUM(findings_count) as total
FROM version_scans
GROUP BY package_name
ORDER BY total DESC;

-- Scan statistics
SELECT 
  COUNT(*) as total,
  SUM(findings_count) as findings,
  AVG(threat_score) as avg_threat
FROM version_scans;
```

---

**End of Workflows Guide**
