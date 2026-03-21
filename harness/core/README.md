# GlassWorm Harness Core

**Version:** 0.1.0

Core infrastructure for GlassWorm package scanning, analysis, and reporting.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Orchestrator                              │
│  - Reads waves.toml                                              │
│  - Runs fetch→scan→analyze→report pipeline                       │
│  - Checkpoint/resume support                                     │
└─────────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
┌───────────────┐     ┌───────────────┐     ┌───────────────┐
│   Fetcher     │     │   Scanner     │     │   Analyzer    │
│               │     │               │     │               │
│ - npm API     │     │ - glassware   │     │ - NVIDIA LLM  │
│ - Download    │     │ - CLI wrapper │     │ - Deep analysis│
│ - SHA256 cache│     │ - JSON parse  │     │ - Caching     │
└───────────────┘     └───────────────┘     └───────────────┘
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              ▼
                     ┌───────────────┐
                     │    Store      │
                     │               │
                     │ - SQLite DB   │
                     │ - Checkpoints │
                     │ - LLM cache   │
                     └───────────────┘
                              │
                              ▼
                     ┌───────────────┐
                     │   Reporter    │
                     │               │
                     │ - Markdown    │
                     │ - JSON        │
                     │ - Wave stats  │
                     └───────────────┘
```

---

## Module Responsibilities

### `store.py` — Database Layer

**Consolidated from:** `database.py`, `background_scanner.py`

**Responsibilities:**
- SQLite database management
- Scan run tracking (with `wave_id` column)
- Package scan results
- Findings storage
- LLM analysis cache
- Checkpoint/resume support

**Key Methods:**
```python
store = Store("data/corpus.db")

# Create scan run
run_id = store.create_scan_run(wave_id=0, filter_params={"categories": ["ai-ml"]})

# Save scan results
store.save_package_scan(run_id, name, version, findings, tarball_sha256)

# Checkpoint/resume
pending = store.get_pending_packages(run_id, package_list)
progress = store.get_run_progress(run_id)

# LLM cache
store.save_llm_analysis(name, version, sha256, result)
cached = store.get_llm_analysis(name, version, sha256)
```

---

### `fetcher.py` — Package Download

**Consolidated from:** `optimized_scanner.py`, `selector.py`, `diverse_sampling.py`

**Responsibilities:**
- npm registry API queries
- Package download with SHA256 caching
- Category-based sampling
- Rate limiting

**Key Methods:**
```python
fetcher = Fetcher()

# Get package info
info = fetcher.get_package_info("express")

# Download package (cached)
dl = fetcher.download_package("express@4.19.2")
# Returns: {tarball_path, tarball_sha256, name, version, cached}

# Sample packages
packages = fetcher.sample_packages(
    categories={"ai-ml": ["machine learning"]},
    samples_per_category=50,
)

# Get recent publishes
recent = fetcher.get_recent_packages(days=14, limit=200)

# Generate typosquats
typosquats = fetcher.get_typosquats(["express", "lodash"])
```

---

### `scanner.py` — GlassWorm CLI Wrapper

**Responsibilities:**
- Shell out to `target/release/glassware`
- Parse JSON output
- `--llm` flag passthrough (Cerebras triage)

**Key Methods:**
```python
scanner = Scanner()

# Scan directory
findings = scanner.scan_directory("/path/to/package")

# Scan tarball
findings = scanner.scan_tarball("package.tgz")

# With LLM triage (Cerebras)
findings = scanner.scan_directory("/path/to/package", use_llm=True)

# Get summary
summary = scanner.get_scan_summary(findings)
# Returns: {total, by_severity, by_category, files}
```

**Note:** Cerebras triage is handled by the Rust CLI (`--llm` flag). Python does NOT need a separate Cerebras layer.

---

### `analyzer.py` — NVIDIA LLM Deep Analysis

**Refactored from:** `batch_llm_analyzer.py`

**Responsibilities:**
- NVIDIA LLM API integration
- Deep analysis of flagged findings
- Result caching

**Key Methods:**
```python
analyzer = Analyzer(store)

# Analyze single package
result = analyzer.analyze_package(
    name="react-native-country-select",
    version="0.3.91",
    tarball_path="path/to/package.tgz",
    findings=[...],  # High-severity findings
)
# Returns: {malicious, confidence, concerns, recommendation, reasoning}

# Batch analyze
results = analyzer.analyze_batch(packages, max_workers=3)
```

**Environment Variables:**
```bash
export NVIDIA_API_KEY="nvapi-..."
export NVIDIA_BASE_URL="https://integrate.api.nvidia.com/v1"
export NVIDIA_MODEL="meta/llama3-70b-instruct"
```

---

### `reporter.py` — Report Generation

**Extended from:** `reporter.py`

**Responsibilities:**
- Markdown report generation
- JSON export
- Wave-aware summaries
- Detector distribution stats

**Key Methods:**
```python
reporter = Reporter(store)

# Generate markdown
md = reporter.generate_markdown(run_id)

# Generate JSON
json_report = reporter.generate_json(run_id)

# Wave summary
wave_summary = reporter.generate_wave_summary(wave_id=0)

# Save both formats
paths = reporter.save_report(run_id)
# Returns: {markdown: Path, json: Path}
```

---

### `orchestrator.py` — Pipeline Runner

**Responsibilities:**
- Read `waves.toml`
- Run fetch→scan→analyze→report pipeline
- Checkpoint/resume
- CLI entry point

**CLI Commands:**
```bash
# Run a wave
python -m harness.core.orchestrator run-wave --wave 0

# With LLM analysis
python -m harness.core.orchestrator run-wave --wave 0 --llm

# Check status
python -m harness.core.orchestrator status

# Generate report
python -m harness.core.orchestrator report --wave 0
```

**Programmatic Usage:**
```python
orchestrator = Orchestrator()

# Run wave
result = orchestrator.run_wave(
    wave_id=0,
    use_llm=True,
    max_workers=5,
    resume=True,
)

# Get status
status = orchestrator.get_status(wave_id=0)

# Generate report
report_path = orchestrator.generate_report(wave_id=0)
```

---

## Wave Configuration (`waves.toml`)

```toml
[wave_0]
name = "Wave 0: Calibration"
description = "Validate pipeline"
packages_total = 50

[wave_0.known_malicious]
packages = ["react-native-country-select@0.3.91", ...]

[wave_0.clean_baseline]
count = 20
packages = ["express@4.19.2", "lodash@4.17.21", ...]

[defaults]
scan_severity = "info"
use_llm = false
max_workers = 5
```

---

## Quick Start

### 1. Set up environment

```bash
# Set NVIDIA API key (for deep analysis)
export NVIDIA_API_KEY="nvapi-..."

# Or create .env file
cp .env.example .env
# Edit .env with your credentials
```

### 2. Build glassware CLI

```bash
cd /path/to/glassworks
cargo build -p glassware-cli --release
```

### 3. Run a wave

```bash
cd harness

# Run Wave 0 (calibration)
python -m core.orchestrator run-wave --wave 0

# Run with LLM analysis
python -m core.orchestrator run-wave --wave 0 --llm

# Check status
python -m core.orchestrator status

# Generate report
python -m core.orchestrator report --wave 0
```

### 4. View results

```bash
# Markdown report
cat reports/scan-<run_id>.md

# JSON report
cat reports/scan-<run_id>.json | jq
```

---

## Database Schema

```sql
-- Scan runs
CREATE TABLE scan_runs (
    id              TEXT PRIMARY KEY,
    wave_id         INTEGER DEFAULT 0,
    started_at      TEXT,
    finished_at     TEXT,
    filter_params   TEXT,  -- JSON
    packages_total  INTEGER,
    packages_flagged INTEGER,
    glassware_version TEXT,
    notes           TEXT
);

-- Packages
CREATE TABLE packages (
    id              INTEGER PRIMARY KEY,
    name            TEXT,
    version         TEXT,
    scanned_at      TEXT,
    scan_run_id     TEXT,
    finding_count   INTEGER,
    tarball_sha256  TEXT,
    vault_path      TEXT,
    UNIQUE(name, version)
);

-- Findings
CREATE TABLE findings (
    id              INTEGER PRIMARY KEY,
    package_id      INTEGER,
    file_path       TEXT,
    line            INTEGER,
    severity        TEXT,
    category        TEXT,
    message         TEXT,
    llm_verdict     TEXT,
    llm_confidence  REAL
);

-- LLM cache
CREATE TABLE llm_analyses (
    id              INTEGER PRIMARY KEY,
    package_name    TEXT,
    package_version TEXT,
    tarball_sha256  TEXT,
    analysis_result TEXT,  -- JSON
    UNIQUE(package_name, package_version, tarball_sha256)
);

-- Checkpoints
CREATE TABLE checkpoints (
    id              INTEGER PRIMARY KEY,
    scan_run_id     TEXT,
    package_name    TEXT,
    version         TEXT,
    status          TEXT,  -- pending/scanned/failed
    UNIQUE(scan_run_id, package_name, version)
);
```

---

## File Structure

```
harness/
├── core/
│   ├── __init__.py       # Package init
│   ├── store.py          # Database layer
│   ├── fetcher.py        # Package download
│   ├── scanner.py        # CLI wrapper
│   ├── analyzer.py       # NVIDIA LLM
│   ├── reporter.py       # Reports
│   ├── orchestrator.py   # Pipeline runner
│   └── README.md         # This file
├── waves.toml            # Wave configurations
├── data/
│   └── corpus.db         # SQLite database
└── reports/              # Generated reports
```

---

## Notes

- **Cerebras triage** is handled by the Rust CLI (`--llm` flag). Python does NOT need a separate Cerebras layer.
- **NVIDIA analysis** is ONLY for deep analysis of flagged findings (high-severity).
- **SHA256 caching** prevents re-downloading the same package version.
- **Checkpoint/resume** allows long-running scans to be paused and resumed.
