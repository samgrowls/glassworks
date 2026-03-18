# glassware npm Scanning Harness

Automated scanning harness for detecting malicious npm packages using glassware.

## Overview

This harness orchestrates the glassware Rust CLI to scan npm packages for:
- **GlassWare patterns**: Unicode steganography, invisible character payloads
- **Encrypted loaders**: Base64/hex encoded payloads with dynamic execution
- **C2 patterns**: HTTP header-based command & control
- **Credential harvesting**: npm token theft, git credential extraction

## Quick Start

### Prerequisites

1. **Install glassware CLI:**
   ```bash
   cargo install --path glassware-cli
   # Or build manually:
   cargo build --release -p glassware-cli
   ```

2. **Install Python dependencies:**
   ```bash
   cd harness
   python3 -m venv .venv
   source .venv/bin/activate
   pip install -r requirements.txt
   ```

3. **Verify glassware is in PATH:**
   ```bash
   glassware --version
   ```

### Running a Scan

**Basic Tier 1 scan (100 packages):**
```bash
python harness/scan.py --max-packages 100 --days-back 30
```

**Custom parameters:**
```bash
python harness/scan.py \
  --max-packages 500 \
  --days-back 14 \
  --download-threshold 500
```

**Resume interrupted scan:**
```bash
python harness/scan.py --resume
```

**Re-scan flagged packages with LLM:**
```bash
python harness/scan.py --rescan <run_id> --with-llm
```

**View corpus statistics:**
```bash
python harness/scan.py --stats
```

## Architecture

```
harness/
├── scan.py           # Main orchestrator
├── selector.py       # npm registry queries (Tier 1 filter)
├── database.py       # SQLite corpus management
├── reporter.py       # Report generation
├── DISCLOSURE.md     # Responsible disclosure policy
├── requirements.txt  # Python dependencies
├── data/
│   ├── corpus.db     # SQLite database
│   └── vault/        # Archived flagged packages
└── reports/          # Generated markdown reports
```

## Tier 1 Selection Criteria

The harness prioritizes high-signal packages using these filters:

| Criterion | Threshold | Rationale |
|-----------|-----------|-----------|
| Published | Last 30 days | Fresh attacks detected faster |
| Install scripts | `preinstall` or `postinstall` | Execution vector |
| Downloads | < 1000/week | Lower profile = less scrutiny |
| Version history | Single version | Newly published, no reputation |

## Output

### Console Output

```
╔════════════════════════════════════════╗
║     glassware npm Scanning Harness    ║
╚════════════════════════════════════════╝

Searching npm for Tier 1 candidates...
  Days back: 30
  Download threshold: < 1000/week
  Max packages: 100

Evaluating: suspicious-pkg
  ✓ suspicious-pkg@1.0.0 (42 downloads/wk, scripts: postinstall)

Scanning packages ████████████░░░░ 47/100
  ⚠ 3 findings (234ms)
  ✓ Clean (156ms)

=== Scan Summary ===
Run ID              a1b2c3d4...
Packages scanned    100
Packages flagged    12
Duration            0:04:32
```

### Database Schema

**packages table:**
- Package metadata (name, version, author, downloads)
- Scan results (finding_count, scan_duration)
- Archive location (vault_path)

**findings table:**
- File path, line/column
- Rule ID, category, severity
- Decoded payload (if present)
- LLM verdict (if analyzed)

### Reports

Reports are generated to `harness/reports/run-{id}.md`:

```markdown
# glassware Scan Report

**Run ID:** `a1b2c3d4-...`

## Summary
| Metric | Value |
|--------|-------|
| Packages scanned | 100 |
| Packages flagged | 12 |

## Flagged Packages

### 🔴 suspicious-pkg@1.0.0
- **Has install scripts:** Yes
- **Finding count:** 3
- **Max severity:** critical

**Findings:**
1. **[critical]** `index.js` (line 42)
   - High-entropy blob combined with dynamic code execution
```

## Vault

Flagged packages are archived to `harness/data/vault/`:

```
vault/
├── suspicious-pkg-1.0.0.tgz    # Original tarball
└── suspicious-pkg-1.0.0/       # Extracted source
    ├── package.json
    ├── index.js
    └── ...
```

Clean packages are NOT archived (storage efficiency).

## LLM Analysis (L3 Layer)

Enable LLM analysis for deeper insights:

```bash
# Requires environment variables:
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="your-key"

# Re-scan flagged packages with LLM:
python harness/scan.py --rescan a1b2c3d4-... --with-llm
```

LLM verdicts are stored in the database and included in reports.

## Responsible Disclosure

See [DISCLOSURE.md](./DISCLOSURE.md) for the full policy.

**Summary:**
1. Verify findings internally
2. Report to npm Security (security@npmjs.com)
3. Wait for package removal
4. Publish findings after remediation

## CLI Reference

```
usage: scan.py [-h] [--max-packages N] [--days-back N] 
               [--download-threshold N] [--tier 1] [--resume]
               [--rescan RUN_ID] [--with-llm] [--stats]

Options:
  --max-packages      Maximum packages to scan (default: 100)
  --days-back         Only scan packages from last N days (default: 30)
  --download-threshold  Max weekly downloads for Tier 1 (default: 1000)
  --resume            Resume last interrupted scan
  --rescan RUN_ID     Re-scan flagged packages from previous run
  --with-llm          Enable LLM analysis on re-scan
  --stats             Show corpus statistics
```

## Database Queries

Example queries for analysis:

```sql
-- Find all critical findings
SELECT p.name, p.version, f.file_path, f.message
FROM findings f
JOIN packages p ON f.package_id = p.id
WHERE f.severity = 'critical';

-- Find packages with install scripts and findings
SELECT name, version, finding_count
FROM packages
WHERE has_install_scripts = 1 AND finding_count > 0;

-- Get untriaged findings
SELECT p.name, f.severity, f.category, f.message
FROM findings f
JOIN packages p ON f.package_id = p.id
WHERE f.triage_status = 'untriaged';
```

## Troubleshooting

**glassware not found:**
```bash
# Add cargo bin to PATH:
export PATH="$HOME/.cargo/bin:$PATH"

# Or specify full path:
glassware=$(which glassware)  # Verify location
```

**Rate limited by npm:**
- The harness automatically backs off on 429 responses
- Reduce `--max-packages` or increase `--days-back` for slower scanning

**Database locked:**
- WAL mode is enabled; should not occur
- If it does, delete `data/corpus.db-wal` and `data/corpus.db-shm`

## License

MIT - same as glassware core
