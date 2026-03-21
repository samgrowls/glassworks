# GitHub Repository Scanning Workflow

**Version:** v0.8.9.0  
**Date:** 2026-03-21

---

## Overview

Scan GitHub repositories for GlassWare/GlassWorm patterns using the glassware orchestrator.

---

## Quick Start

```bash
# Set GitHub token (required for rate limits)
export GITHUB_TOKEN="ghp_..."

# Search for repos
./target/debug/glassware-orchestrator search-github "mcp-server" --max-results 50 -o repos.txt

# Scan repos
./target/debug/glassware-orchestrator scan-github $(cat repos.txt)

# Or scan with version control
./target/debug/glassware-orchestrator scan-github -r main owner/repo
```

---

## Step 1: Search GitHub Repositories

### Basic Search

```bash
# Search and display
./target/debug/glassware-orchestrator search-github "mcp-server"

# Search with limit
./target/debug/glassware-orchestrator search-github "langchain plugin" --max-results 100

# Search and save to file
./target/debug/glassware-orchestrator search-github "ai agent" --max-results 200 -o github-repos.txt
```

### Advanced Search Queries

```bash
# Recently created (< 30 days)
./target/debug/glassware-orchestrator search-github "mcp-server created:>2026-02-01" --max-results 100

# With many stars
./target/debug/glassware-orchestrator search-github "langchain stars:>100" --max-results 50

# Specific language
./target/debug/glassware-orchestrator search-github "agent language:typescript" --max-results 100

# By author
./target/debug/glassware-orchestrator search-github "mcp user:suspicious-author" --max-results 50
```

### Search Query Syntax

GitHub search supports advanced queries:

| Qualifier | Example | Description |
|-----------|---------|-------------|
| `created:` | `created:>2026-02-01` | Created after date |
| `pushed:` | `pushed:>2026-03-01` | Pushed after date |
| `stars:` | `stars:>100` | More than N stars |
| `forks:` | `forks:>50` | More than N forks |
| `language:` | `language:typescript` | Specific language |
| `user:` | `user:username` | By specific user |
| `org:` | `org:anthropics` | By specific org |
| `in:name` | `mcp in:name` | In repository name |
| `in:description` | `agent in:description` | In description |

**Examples:**
```bash
# New MCP servers (< 7 days)
search-github "mcp-server created:>2026-03-14"

# Popular AI projects
search-github "ai stars:>1000"

# TypeScript agent frameworks
search-github "agent framework language:typescript"
```

---

## Step 2: Scan Repositories

### Scan Single Repository

```bash
# Scan default branch
./target/debug/glassware-orchestrator scan-github owner/repo

# Scan specific branch
./target/debug/glassware-orchestrator scan-github -r main owner/repo

# Scan specific commit
./target/debug/glassware-orchestrator scan-github -r abc123 owner/repo

# Scan with verbose output
./target/debug/glassware-orchestrator -v scan-github owner/repo
```

### Scan Multiple Repositories

```bash
# From command line
./target/debug/glassware-orchestrator scan-github owner1/repo1 owner2/repo2 owner3/repo3

# From file
./target/debug/glassware-orchestrator scan-github $(cat repos.txt)

# With LLM analysis
./target/debug/glassware-orchestrator --llm scan-github $(cat repos.txt)
```

### Scan with Caching

```bash
# Enable cache for faster re-scans
./target/debug/glassware-orchestrator \
  --cache-db github-cache.db \
  scan-github $(cat repos.txt)

# Re-scan uses cache automatically
./target/debug/glassware-orchestrator \
  --cache-db github-cache.db \
  scan-github $(cat repos.txt)
```

---

## Step 3: Automated Workflow

### Complete Pipeline

```bash
#!/bin/bash
# scan-github-workflow.sh

set -e

# Configuration
SEARCH_QUERY="mcp-server created:>2026-03-01"
MAX_RESULTS=100
OUTPUT_DIR="github-scan-$(date +%Y%m%d)"

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo "=== GitHub Repository Scanning Workflow ==="
echo "Query: $SEARCH_QUERY"
echo "Max results: $MAX_RESULTS"
echo "Output: $OUTPUT_DIR"
echo

# Step 1: Search
echo "[1/4] Searching GitHub..."
./target/debug/glassware-orchestrator \
  search-github "$SEARCH_QUERY" \
  --max-results $MAX_RESULTS \
  -o "$OUTPUT_DIR/repos.txt"

REPO_COUNT=$(wc -l < "$OUTPUT_DIR/repos.txt")
echo "Found $REPO_COUNT repositories"
echo

# Step 2: Scan
echo "[2/4] Scanning repositories..."
./target/debug/glassware-orchestrator \
  --cache-db "$OUTPUT_DIR/cache.db" \
  scan-github $(cat "$OUTPUT_DIR/repos.txt") \
  2>&1 | tee "$OUTPUT_DIR/scan-output.log"

echo

# Step 3: Generate report
echo "[3/4] Generating report..."
cat > "$OUTPUT_DIR/report.md" << EOF
# GitHub Scan Report

**Date:** $(date -I)
**Query:** $SEARCH_QUERY
**Repositories:** $REPO_COUNT

## Scan Results

\`\`\`
$(tail -20 "$OUTPUT_DIR/scan-output.log")
\`\`\`

## Repositories Scanned

$(cat "$OUTPUT_DIR/repos.txt")
EOF

echo "Report: $OUTPUT_DIR/report.md"
echo

# Step 4: Summary
echo "[4/4] Summary"
echo "============="
echo "Output directory: $OUTPUT_DIR"
echo "Repositories: $REPO_COUNT"
echo "Report: $OUTPUT_DIR/report.md"
echo "Log: $OUTPUT_DIR/scan-output.log"
```

### Usage

```bash
chmod +x scan-github-workflow.sh
./scan-github-workflow.sh
```

---

## High-Risk Search Queries

### Recently Created (High Risk)

```bash
# Last 7 days
search-github "mcp-server created:>2026-03-14"

# Last 30 days
search-github "langchain plugin created:>2026-02-01"

# This week
search-github "ai agent created:>2026-03-17"
```

### Anonymous Authors

```bash
# No organization
search-github "mcp-server -org:*"

# User repos only
search-github "agent user:*"
```

### Install Scripts

```bash
# With preinstall
search-github "preinstall filename:package.json"

# With postinstall
search-github "postinstall filename:package.json"

# With install script
search-github "\"install\": \"node" filename:package.json
```

### Suspicious Patterns

```bash
# Base64 in code
search-github "Buffer.from base64 language:javascript"

# Eval usage
search-github "eval( language:javascript"

# Hidden characters
search-github "\u200B language:javascript"
```

---

## Output Formats

### Plain Text (Default)

```
owner/repo1
owner/repo2
owner/repo3
```

### JSON (with --format json)

```bash
./target/debug/glassware-orchestrator \
  --format json \
  search-github "mcp-server" --max-results 10
```

### SARIF (for GitHub Advanced Security)

```bash
./target/debug/glassware-orchestrator \
  --format sarif \
  scan-github owner/repo > results.sarif
```

Upload `results.sarif` to GitHub Security tab.

---

## Rate Limits

| Authentication | Limit |
|----------------|-------|
| None | 60 requests/hour |
| GITHUB_TOKEN | 5,000 requests/hour |

**Configure token:**
```bash
# Add to ~/.env
GITHUB_TOKEN="ghp_..."

# Or export
export GITHUB_TOKEN="ghp_..."

# Or use flag
./target/debug/glassware-orchestrator \
  --github-token ghp_... \
  search-github "query"
```

---

## Best Practices

### 1. Use Specific Queries

```bash
# Good: Specific and recent
search-github "mcp-server created:>2026-03-01 language:typescript"

# Bad: Too broad
search-github "mcp"
```

### 2. Scan in Batches

```bash
# Split into batches of 50
split -l 50 repos.txt batch-

# Scan each batch
for batch in batch-*; do
    ./target/debug/glassware-orchestrator \
      --cache-db cache.db \
      scan-github $(cat "$batch")
done
```

### 3. Use Caching

```bash
# First scan (slow)
./target/debug/glassware-orchestrator \
  --cache-db cache.db \
  scan-github $(cat repos.txt)

# Re-scan (fast - uses cache)
./target/debug/glassware-orchestrator \
  --cache-db cache.db \
  scan-github $(cat repos.txt)
```

### 4. Monitor Progress

```bash
# Use verbose mode
./target/debug/glassware-orchestrator -v scan-github owner/repo

# Or use JSON output for parsing
./target/debug/glassware-orchestrator \
  --format json \
  scan-github owner/repo | jq '.summary'
```

---

## Troubleshooting

### Rate Limit Exceeded

**Error:** `GitHub API rate limit exceeded`

**Fix:**
```bash
# Set GITHUB_TOKEN
export GITHUB_TOKEN="ghp_..."

# Or wait 1 hour for limit to reset
```

### Repository Not Found

**Error:** `Repository not found: owner/repo`

**Causes:**
- Repository was deleted
- Repository is private (need token with repo scope)
- Typo in repository name

**Fix:**
```bash
# Verify repo exists
curl -H "Authorization: token $GITHUB_TOKEN" \
  https://api.github.com/repos/owner/repo
```

### Scan Timeout

**Error:** `Timeout after 120s`

**Fix:**
```bash
# Scan smaller repos first
# Or increase timeout in code
```

---

## Example: Finding GlassWorm Targets

```bash
#!/bin/bash
# Find potential GlassWorm targets

echo "=== Searching for High-Risk Repos ==="

# 1. New MCP servers (< 7 days)
echo "[1/5] New MCP servers..."
./target/debug/glassware-orchestrator \
  search-github "mcp-server created:>2026-03-14" \
  --max-results 50 -o new-mcp.txt

# 2. AI packages with install scripts
echo "[2/5] AI packages with install scripts..."
./target/debug/glassware-orchestrator \
  search-github "postinstall ai language:javascript" \
  --max-results 50 -o ai-install.txt

# 3. Anonymous author repos
echo "[3/5] Anonymous author repos..."
./target/debug/glassware-orchestrator \
  search-github "agent -org:*" \
  --max-results 50 -o anon-agent.txt

# 4. Base64 encoded code
echo "[4/5] Base64 encoded code..."
./target/debug/glassware-orchestrator \
  search-github "Buffer.from base64 language:javascript" \
  --max-results 50 -o base64-code.txt

# 5. Scan all found repos
echo "[5/5] Scanning repositories..."
cat new-mcp.txt ai-install.txt anon-agent.txt base64-code.txt | \
  sort -u > high-risk-repos.txt

./target/debug/glassware-orchestrator \
  --llm \
  --cache-db high-risk-cache.db \
  scan-github $(cat high-risk-repos.txt)

echo "=== Scan Complete ==="
echo "Results in: high-risk-cache.db"
```

---

**End of Workflow Guide**
