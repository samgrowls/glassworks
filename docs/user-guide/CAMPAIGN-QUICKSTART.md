# GlassWorm Quick-Start Campaign Guide

**Goal:** Get from zero to campaign results in 10 minutes.

---

## Prerequisites (2 minutes)

```bash
# 1. Clone repo
git clone https://github.com/samgrowls/glassworks.git
cd glassworks

# 2. Build tools
cargo build -p glassware-cli --release
cargo build -p glassware-orchestrator --release

# 3. (Optional) Configure LLM API
cat >> ~/.env << 'EOF'
export NVIDIA_API_KEY="nvapi-YOUR_KEY_HERE"
export NVIDIA_MODELS="qwen/qwen3.5-397b-a17b,moonshotai/kimi-k2.5,meta/llama3-70b-instruct"
EOF
source ~/.env
```

---

## Option A: Quick Scan (3 minutes)

**Use case:** Scan a few specific packages.

```bash
# Scan 3 packages
./target/release/glassware-orchestrator scan-npm \
  express@4.19.2 \
  lodash@4.17.21 \
  axios@1.6.7

# See results in terminal
```

**Output:**
```
============================================================
SCAN SUMMARY
============================================================
Total packages scanned: 3
Malicious packages: 0
Total findings: 11
...
```

---

## Option B: Wave Campaign (5 minutes)

**Use case:** Scan a category of packages (e.g., all crypto wallets).

### Step 1: Configure Wave

Edit `harness/waves.toml`:

```toml
[wave_custom]
name = "My Crypto Scan"
packages_total = 50

[wave_custom.crypto]
count = 50
keywords = ["web3", "wallet", "ethereum", "solana"]
```

### Step 2: Run Wave

```bash
cd harness
python3 -m core.orchestrator run-wave --wave custom
```

### Step 3: View Results

```bash
# Markdown report
cat reports/scan-<run_id>.md

# JSON data
cat reports/scan-<run_id>.json | jq '.packages[] | select(.finding_count > 0)'
```

---

## Option C: Targeted Hunt (7 minutes)

**Use case:** Hunt for specific threat patterns.

### Step 1: Create Package List

```bash
cat > targets.txt << 'EOF'
react-native-country-select@0.3.91
react-native-international-phone-number@0.11.8
@scope/suspicious-package@1.0.0
EOF
```

### Step 2: Run Scan with SARIF

```bash
./target/release/glassware-orchestrator \
  --format sarif \
  --output results.sarif \
  scan-file targets.txt
```

### Step 3: Upload to GitHub Security

1. Go to repository → Security → Code scanning
2. Upload `results.sarif`
3. View findings in GitHub UI

---

## Option D: Full LLM Analysis (10 minutes)

**Use case:** Deep analysis with AI verdict.

### Step 1: Ensure NVIDIA API Key

```bash
export NVIDIA_API_KEY="nvapi-..."
```

### Step 2: Run Wave with LLM

```bash
cd harness
python3 -m core.orchestrator run-wave --wave 0 --llm
```

### Step 3: Review LLM Verdicts

```bash
cat reports/scan-<run_id>.json | jq '
  .packages[] | 
  select(.llm_analysis) | 
  {name, malicious: .llm_analysis.malicious, confidence: .llm_analysis.confidence}
'
```

**Output:**
```json
{
  "name": "axios",
  "malicious": "no",
  "confidence": "high"
}
```

---

## Interpreting Results

### Severity Levels

| Level | Action |
|-------|--------|
| **CRITICAL** | Immediate investigation required |
| **HIGH** | Investigate within 24 hours |
| **MEDIUM** | Review within 1 week |
| **LOW** | Monitor, low priority |
| **INFO** | Informational only |

### Finding Categories

| Category | Description |
|----------|-------------|
| `InvisibleCharacter` | Zero-width chars, variation selectors |
| `GlasswarePattern` | Stego decoder patterns |
| `EncryptedPayload` | High-entropy blob + decrypt |
| `BlockchainC2` | Solana/Google Calendar C2 |
| `LocaleGeofencing` | Russian locale checks |
| `SocketIOC2` | Socket.IO C2 patterns |
| `ExfilSchema` | Data exfiltration schema |
| `MemexecLoader` | Fileless PE loading |
| `IElevatorCom` | COM interface abuse |

---

## Next Steps

### If Findings Detected

1. **Review finding details**
   ```bash
   cat reports/scan-<run_id>.json | jq '.packages[] | select(.finding_count > 0)'
   ```

2. **Run LLM analysis** (if not already)
   ```bash
   python3 -m core.orchestrator run-wave --wave 0 --llm
   ```

3. **Manual verification**
   - Download package: `npm pack package@version`
   - Extract: `tar -xzf package.tgz`
   - Review flagged files manually

4. **Report findings**
   - npm Security: security@npmjs.com
   - GitHub: Use repository security advisory

### If No Findings

1. **Expand scope**
   - Increase wave size in `waves.toml`
   - Add more keywords

2. **Lower severity threshold**
   ```bash
   glassware --severity low project/
   ```

3. **Try different categories**
   ```toml
   [wave_custom.new_category]
   count = 50
   keywords = ["ai-ml", "native-build", "install-scripts"]
   ```

---

## Common Workflows

### Daily Monitoring

```bash
# Scan top 20 new packages daily
cat > daily-wave.toml << 'EOF'
[wave_daily]
packages_total = 20

[wave_daily.recent]
count = 20
days = 1
max_downloads = 1000
EOF

python3 -m core.orchestrator run-wave --wave daily
```

### Pre-merge Security Check

```bash
# Scan PR dependencies
glassware --format sarif node_modules/ > pr-scan.sarif
# Upload to GitHub Security
```

### Supply Chain Audit

```bash
# Scan all production dependencies
cat package.json | jq -r '.dependencies | keys[]' > prod-deps.txt
glassware-orchestrator scan-file prod-deps.txt --format json > audit.json
```

---

## Troubleshooting Quick Fixes

| Problem | Fix |
|---------|-----|
| "Package not found" | Update: `git pull && cargo build` |
| "NVIDIA_API_KEY not set" | Add to `~/.env` |
| Scan too slow | Add `--concurrency 20` |
| Too many false positives | Use `--severity high` |
| Memory issues | Reduce `--concurrency 5` |

---

## Getting Help

- **Full documentation:** `docs/WORKFLOW-GUIDE.md`
- **Detector details:** `FINAL-COMPLETION-REPORT.md`
- **Issues:** https://github.com/samgrowls/glassworks/issues
