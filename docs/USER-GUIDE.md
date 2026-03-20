# glassware v0.5.0 — User Guide

**Quick reference for users and agents**

---

## Quick Start

### Basic Scan

```bash
# Scan a directory
glassware src/

# Scan specific files
glassware src/index.js package.json

# JSON output
glassware --format json src/ > results.json

# SARIF output (for GitHub Advanced Security)
glassware --format sarif src/ > results.sarif
```

---

## Command Line Options

### Output Format (`-f, --format`)

```bash
# Pretty (default, human-readable)
glassware --format pretty src/

# JSON (machine-readable)
glassware --format json src/ > results.json

# SARIF (for GitHub, Azure DevOps)
glassware --format sarif src/ > results.sarif
```

### Severity Filter (`-s, --severity`)

```bash
# Only critical findings
glassware --severity critical src/

# High and critical only
glassware --severity high src/

# Medium and above (default: low)
glassware --severity medium src/
```

### Caching (`--cache-file`, `--cache-ttl`, `--no-cache`)

```bash
# Enable caching (default: .glassware-cache.json)
glassware --cache-file .glassware-cache.json src/

# Custom cache TTL (default: 7 days)
glassware --cache-ttl 30 src/

# Disable caching
glassware --no-cache src/
```

### Advanced Options

```bash
# Custom file extensions
glassware --extensions "js,mjs,ts,py,rs" src/

# Exclude directories
glassware --exclude "node_modules,dist,build" src/

# Quiet mode (exit code only)
glassware --quiet src/
echo $?  # 0 = clean, 1 = findings, 2 = error

# LLM analysis (requires API key)
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="your-key"
glassware --llm src/
```

---

## v0.5.0 Features

### Cross-File Analysis (`--cross-file`)

**Detects split payloads across multiple files**

```bash
# Enable cross-file analysis
glassware --cross-file package/

# Cross-file + attack graph
glassware --cross-file --attack-graph package/
```

**What it detects:**
- Decoder in file A, payload execution in file B
- Multi-file data flows
- Import chain tracking

### Attack Graph (`--attack-graph`)

**Correlates findings into attack chains**

```bash
# Enable attack graph correlation
glassware --attack-graph src/

# Get threat score (0.0-10.0)
glassware --attack-graph --format json src/ | jq '.threat_score'
```

**Attack chain types:**
- GlassWareStego (Unicode stego → decoder → exec)
- EncryptedExec (High-entropy → decrypt → exec)
- HeaderC2Chain (HTTP header → decrypt → exec)
- BlockchainC2 (Blockchain API → data extraction → exec)
- GeofencedExec (Locale check → delay → exec)
- SupplyChainCompromise (Multi-package coordinated)

### Campaign Intelligence (`--campaign`)

**Tracks attacker infrastructure reuse**

```bash
# Enable campaign tracking
glassware --campaign src/

# Get campaign info
glassware --campaign --format json src/ | jq '.campaign_info'
```

**Campaign types detected:**
- GlassWorm (Unicode stego + blockchain C2)
- PhantomRaven (RDD attacks)
- ForceMemo (Python repo injection)
- ChromeRAT (Chrome extension patterns)
- ShaiHulud (Self-propagating stego)
- SandwormMode (Time delay + MCP injection)

---

## Recommended Workflows

### Workflow 1: Quick Security Check

```bash
# Fast scan with defaults
glassware src/

# Check exit code
if [ $? -eq 0 ]; then
    echo "✅ Clean"
else
    echo "⚠️ Findings detected"
fi
```

### Workflow 2: CI/CD Integration

```bash
# SARIF output for GitHub
glassware --format sarif src/ > results.sarif

# Upload to GitHub
gh api \
  --method POST \
  -H "Accept: application/vnd.github+json" \
  /repos/OWNER/REPO/code-scanning/sarifs \
  -f "sarif=$(base64 -i results.sarif)" \
  -f "ref=refs/heads/main"
```

### Workflow 3: Threat Hunting

```bash
# Full intelligence scan
glassware \
  --cross-file \
  --attack-graph \
  --campaign \
  --format json \
  src/ > threat-intel.json

# Analyze results
cat threat-intel.json | jq '{
  findings: .findings | length,
  attack_chains: .attack_chains | length,
  campaigns: .campaign_info,
  threat_score: .threat_score
}'
```

### Workflow 4: High-Security Scan

```bash
# Scan including bundled code (slower, more FPs)
glassware --analyze-bundled src/

# Disable tiered detection (run all detectors)
glassware --no-tiered src/

# Maximum sensitivity
glassware \
  --severity info \
  --analyze-bundled \
  --no-tiered \
  --cross-file \
  --attack-graph \
  --campaign \
  src/
```

---

## Interpreting Results

### Pretty Output

```
⚠ CRITICAL
  File: src/payload.js
  Line: 42
  Type: rc4_pattern
  RC4-like cipher implementation detected near dynamic execution
  CRITICAL: This pattern is consistent with GlassWare payload decryption.
```

### JSON Output

```json
{
  "version": "0.5.0",
  "findings": [
    {
      "file": "src/payload.js",
      "line": 42,
      "column": 1,
      "severity": "critical",
      "category": "rc4_pattern",
      "message": "RC4-like cipher..."
    }
  ],
  "attack_chains": [
    {
      "id": "chain-001",
      "classification": "GlassWareStego",
      "confidence": 0.95,
      "steps": [...]
    }
  ],
  "campaign_info": {
    "campaign_id": "GlassWorm-Wave5",
    "related_packages": [...]
  },
  "threat_score": 9.5,
  "summary": {
    "files_scanned": 50,
    "findings_count": 1
  }
}
```

### Threat Score Interpretation

| Score | Level | Action |
|-------|-------|--------|
| 0.0-2.0 | Low | Monitor |
| 2.1-5.0 | Medium | Review |
| 5.1-7.0 | High | Investigate |
| 7.1-10.0 | Critical | Immediate action |

---

## Common Scenarios

### Scenario 1: Scan npm Package Before Install

```bash
# Download package
npm pack suspicious-package@1.0.0
tar -xzf *.tgz

# Scan
glassware package/

# Clean up
rm -rf package *.tgz
```

### Scenario 2: Scan GitHub Repository

```bash
# Clone repository
git clone https://github.com/user/repo.git
cd repo

# Scan
glassware src/

# Clean up
cd .. && rm -rf repo
```

### Scenario 3: Continuous Monitoring

```bash
# Create watch script
cat > monitor.sh << 'EOF'
#!/bin/bash
while true; do
    glassware --cache-file .glassware-cache.json src/
    sleep 3600  # Check every hour
done
EOF

chmod +x monitor.sh
./monitor.sh
```

---

## Troubleshooting

### High False Positive Rate

```bash
# Use tiered detection (default)
glassware src/

# Skip bundled code
glassware --exclude "dist,build,node_modules" src/

# Increase severity threshold
glassware --severity high src/
```

### Slow Scan Times

```bash
# Enable caching
glassware --cache-file .glassware-cache.json src/

# Reduce workers (if system is overloaded)
# Edit optimized_scanner.py: WORKERS = 5

# Exclude large directories
glassware --exclude "node_modules,dist,build,.git" src/
```

### Cache Not Working

```bash
# Check cache file exists
ls -la .glassware-cache.json

# Check cache stats
glassware --cache-file .glassware-cache.json src/ 2>&1 | grep "Cache:"

# Clear cache and rescan
rm .glassware-cache.json
glassware --cache-file .glassware-cache.json src/
```

---

## Performance Tips

### Optimal Configuration

```bash
# Production scan (balanced)
glassware \
  --cache-file .glassware-cache.json \
  --cache-ttl 7 \
  --exclude "node_modules,dist,build" \
  src/

# Expected performance:
# - First scan: ~2.5s per 500 files
# - Re-scan (cached): ~0.5s per 500 files
```

### Memory Usage

| Configuration | Memory |
|--------------|--------|
| Default | ~85MB |
| With cross-file | ~95MB |
| With campaign | ~100MB |
| All features | ~110MB |

---

## Examples

### Example 1: Scan Before Dependency Update

```bash
# Check what will be installed
npm install --dry-run new-dependency

# Scan the dependency
npm pack new-dependency
tar -xzf *.tgz
glassware package/
rm -rf package *.tgz

# Install if clean
npm install new-dependency
```

### Example 2: Audit Existing Dependencies

```bash
# Scan node_modules
glassware \
  --exclude "node_modules/.cache" \
  --cache-file .glassware-cache.json \
  node_modules/
```

### Example 3: Pre-Commit Hook

```bash
# Add to .git/hooks/pre-commit
#!/bin/bash
glassware --quiet src/
if [ $? -ne 0 ]; then
    echo "⚠️ Unicode attacks detected!"
    exit 1
fi
exit 0
```

---

## Getting Help

```bash
# CLI help
glassware --help

# Version info
glassware --version

# Documentation
cat HANDOFF.md
cat docs/WORKFLOW-GUIDE.md
cat harness/reports/V0.5.0-REAL-WORLD-VALIDATION.md
```

---

**Version:** 0.5.0  
**Repository:** https://github.com/samgrowls/glassware  
**Installation:** `cargo install --path glassware-cli`
