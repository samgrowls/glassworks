# glassware Workflow Guide

**Purpose:** Make scan/analyze/improve workflow obvious for developers and agents

---

## Quick Start (5 Minutes)

### Scan npm Packages

```bash
cd harness

# Scan 100 packages
python3 diverse_sampling.py --samples-per-keyword 10 -o packages.txt
python3 optimized_scanner.py packages.txt -w 10 -o results.json

# Check results
cat results.json | jq '{scanned, flagged, errors}'
```

### Scan GitHub Repos

```bash
cd harness

# Scan 50 repos
python3 github_scanner.py --queries "mcp" --max-repos 50 -o github-results.json

# Check results
cat github-results.json | jq '{scanned, flagged, errors}'
```

---

## Complete Workflow (Scan → Analyze → Improve)

### Phase 1: Scan

#### Option A: npm Package Scan

```bash
cd harness

# 1. Choose categories
# See diverse_sampling.py for available categories:
# - ai-ml, native-build, install-scripts, devtools, crypto, etc.

# 2. Sample packages
python3 diverse_sampling.py \
  --categories ai-ml native-build install-scripts \
  --samples-per-keyword 20 \
  --delay-between-keywords 0.5 \
  --output packages.txt

# Expected: 200-600 packages in 10-20 minutes

# 3. Scan packages
python3 optimized_scanner.py \
  packages.txt \
  -w 10 \              # Workers (10 = fast, reduce if system slow)
  -e data/evidence/scan-1 \
  -o scan-1-results.json \
  -n scan-1

# Expected: 200 packages in 2-5 minutes
```

#### Option B: GitHub Repo Scan

```bash
cd harness

# 1. Choose queries
# Common queries: "mcp", "vscode", "cursor", "node-gyp", "prebuild"

# 2. Scan repos
python3 github_scanner.py \
  --queries "mcp" "vscode" "cursor" \
  --repos-per-query 50 \
  --max-repos 200 \
  --scanner ./glassware-scanner \
  --output github-scan.json

# Expected: 200 repos in 1-2 hours
```

#### Option C: Targeted Scan

```bash
cd harness

# 1. Create package list
cat > target.txt << EOF
suspicious-package@1.0.0
another-package@2.0.0
EOF

# 2. Scan
python3 optimized_scanner.py target.txt -w 5 -e data/evidence/targeted
```

---

### Phase 2: Analyze

#### Step 1: Check Scan Results

```bash
cd harness

# Overall stats
cat scan-1-results.json | jq '{scanned, flagged, cached, errors}'

# Flagged packages
cat scan-1-results.json | jq '.flagged_packages[] | {package, findings, critical}' | head -20
```

#### Step 2: Extract Flagged Packages

```bash
# Extract package names
cat scan-1-results.json | jq -r '.flagged_packages[].package' > flagged.txt

# Sort by critical count
cat scan-1-results.json | jq -r '.flagged_packages | sort_by(-.critical) | .[].package' > flagged-priority.txt
```

#### Step 3: LLM Analysis (Optional)

```bash
# Set API key
export NVIDIA_API_KEY="nvapi-..."

# Run LLM on flagged packages
python3 batch_llm_analyzer.py \
  flagged.txt \
  -w 2 \                # Workers (2 = safe for rate limits)
  -e data/evidence/llm-1 \
  -o llm-1-results.json

# Check LLM results
cat llm-1-results.json | jq '.results[] | {package, llm_classification, confidence}'
```

#### Step 4: Manual Review

```bash
# Download suspicious package
cd /tmp
npm pack "suspicious-package@1.0.0"
tar -xzf *.tgz

# Scan with verbose output
cd harness
./glassware-scanner --format json /tmp/package/ | jq '.'

# Review findings
./glassware-scanner /tmp/package/
```

---

### Phase 3: Improve

#### Step 1: Identify False Positives

```bash
# Common FP patterns:
# - Minified code (>100KB files)
# - Legitimate i18n (locale checks without exit)
# - UI polling (setInterval without network)
# - Parser code (hex decoding in parsers)

# Check flagged packages for FP patterns
cat llm-1-results.json | jq '.results[] | select(.llm_classification == "FALSE_POSITIVE")'
```

#### Step 2: Tune Detectors

**If minified code FPs:**
- Already handled: Files >100KB skip homoglyph/bidi detection

**If i18n FPs:**
- Already handled: i18n files skipped

**If new FP pattern:**
1. Identify pattern (e.g., `setInterval` for UI polling)
2. Add context check to detector
3. Test on FP package
4. Rebuild: `cargo build --release`

#### Step 3: Add Allowlist (Optional)

```python
# harness/allowlist.py
ALLOWLIST = [
    "prettier",
    "eslint",
    "typescript",
    # Add known legitimate packages
]
```

#### Step 4: Re-scan

```bash
# Re-scan with tuned detectors
python3 optimized_scanner.py packages.txt -w 10 -e data/evidence/scan-2 -o scan-2-results.json

# Compare results
echo "Before:" && cat scan-1-results.json | jq '.flagged'
echo "After:" && cat scan-2-results.json | jq '.flagged'
```

---

## Monitoring Long Scans

### GitHub Scan (Hours)

```bash
# Start in background
cd harness
nohup python3 github_scanner.py --queries "mcp" --max-repos 500 > github-scan.log 2>&1 &

# Monitor
tail -f github-scan.log

# Check progress
ps aux | grep github_scanner | grep -v grep

# Check results (updates during scan)
cat github-scan-results.json | jq '{scanned, flagged, errors}'
```

### npm Scan (Minutes)

```bash
# Start scan
python3 optimized_scanner.py packages.txt -w 10 -o results.json

# Monitor in real-time
tail -f results.json | jq '.scanned'  # Updates during scan
```

---

## Common Scenarios

### Scenario 1: Quick Sanity Check

```bash
# Scan 10 packages
cd harness
echo "prettier" > test.txt
python3 optimized_scanner.py test.txt -w 2 -o test-results.json
cat test-results.json | jq '{scanned, flagged}'
```

### Scenario 2: High-Risk Scan

```bash
# Scan high-risk categories
python3 diverse_sampling.py \
  --categories ai-ml native-build install-scripts crypto \
  --samples-per-keyword 30 \
  --output high-risk.txt

python3 optimized_scanner.py high-risk.txt -w 10 -o high-risk-results.json
```

### Scenario 3: Campaign Hunting

```bash
# Scan for specific campaign (PhantomRaven)
cat > phantomraven.txt << EOF
unused-imports
eslint-comments
sort-keys-fix
typescript-compat
EOF

python3 optimized_scanner.py phantomraven.txt -w 5 -o phantomraven-results.json
```

### Scenario 4: Validation Scan

```bash
# Scan known malicious package
cd /tmp
npm pack "@iflow-mcp/ref-tools-mcp@3.0.0"
tar -xzf *.tgz

cd harness
./glassware-scanner --format json /tmp/package/ | jq '.findings | length'
# Expected: 15+ findings
```

---

## Decision Tree

```
Start
│
├─ Want to scan npm packages?
│  ├─ Yes → optimized_scanner.py
│  │  ├─ Have package list? → Use it directly
│  │  └─ Need packages? → diverse_sampling.py first
│  │
│  └─ No → Continue
│
├─ Want to scan GitHub repos?
│  ├─ Yes → github_scanner.py
│  │  ├─ Have token? → export GITHUB_TOKEN
│  │  └─ No token? → Works without (slower)
│  │
│  └─ No → Continue
│
├─ Have flagged packages?
│  ├─ Yes → Analyze
│  │  ├─ Want LLM analysis? → batch_llm_analyzer.py
│  │  └─ Manual review? → Download + scan
│  │
│  └─ No → Done (clean scan)
│
└─ Have false positives?
   ├─ Yes → Improve
   │  ├─ Pattern identified? → Tune detector
   │  └─ Known legitimate? → Add to allowlist
   │
   └─ No → Done (accurate scan)
```

---

## Quick Reference

### Commands

| Command | Purpose |
|---------|---------|
| `diverse_sampling.py` | Sample npm packages by category |
| `optimized_scanner.py` | Scan npm packages |
| `github_scanner.py` | Scan GitHub repositories |
| `batch_llm_analyzer.py` | LLM analysis on flagged packages |
| `monitor-30k-scan.sh` | Monitor long scans |

### Flags

| Flag | Description |
|------|-------------|
| `-w` | Workers (parallel threads) |
| `-e` | Evidence directory |
| `-o` | Output file |
| `-n` | Scan name |
| `--queries` | GitHub search queries |
| `--categories` | npm categories |

### Files

| File | Purpose |
|------|---------|
| `packages.txt` | Package list (one per line) |
| `results.json` | Scan results |
| `flagged.txt` | Flagged package names |
| `llm-results.json` | LLM analysis results |

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| npm 429 error | Increase `--delay-between-keywords` |
| GitHub 403 error | Add `GITHUB_TOKEN` or wait |
| No findings | Check file paths, try `--format json` |
| Too many FPs | Review FP patterns, tune detectors |
| Scan too slow | Reduce `-w` workers or use cache |

---

**End of Workflow Guide**
