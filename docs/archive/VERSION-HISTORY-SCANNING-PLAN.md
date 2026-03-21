# Version History Scanning - Architecture Plan

**Date:** 2026-03-21  
**Status:** Planning Phase

---

## Problem Statement

Malicious infiltrations often target:
1. **Legacy versions** - Less scrutiny, older security standards
2. **Specific version ranges** - Before security improvements were added
3. **Recently published packages** - Before reputation is established
4. **Version updates** - Sneak malicious code in a minor/patch update

**Current Gap:** We only scan the latest version, missing historical context.

---

## Solution: Version History Scanning

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    VERSION HISTORY SCANNER                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌──────────────────┐         ┌──────────────────┐              │
│  │  Package List    │         │  Version Policy  │              │
│  │  (500 packages)  │         │  (how far back)  │              │
│  └────────┬─────────┘         └────────┬─────────┘              │
│           │                             │                        │
│           └──────────┬──────────────────┘                        │
│                      │                                           │
│                      ▼                                           │
│           ┌──────────────────────┐                              │
│           │  Version Sampler     │                              │
│           │  - Last N versions   │                              │
│           │  - Last X months     │                              │
│           │  - All versions      │                              │
│           └──────────┬───────────┘                              │
│                      │                                           │
│                      ▼                                           │
│           ┌──────────────────────┐                              │
│           │  glassware-orch      │                              │
│           │  (with version queue)│                              │
│           │                      │                              │
│           │  ┌────────────────┐  │                              │
│           │  │ Version Scanner│  │                              │
│           │  │ - Download v1  │  │                              │
│           │  │ - Scan v1      │  │                              │
│           │  │ - Cache result │  │                              │
│           │  │ - Download v2  │  │                              │
│           │  │ - Scan v2      │  │                              │
│           │  │ ...            │  │                              │
│           │  └────────────────┘  │                              │
│           └──────────┬───────────┘                              │
│                      │                                           │
│                      ▼                                           │
│           ┌──────────────────────┐                              │
│           │  Results Database    │                              │
│           │  (SQLite)            │                              │
│           │                      │                              │
│           │  pkg | ver | findings│                              │
│           │  ────┼─────┼─────────│                              │
│           │  foo | 1.0 | 0       │                              │
│           │  foo | 1.1 | 3 ⚠️    │  ← Malicious version!        │
│           │  foo | 1.2 | 0       │  ← Clean (fixed?)            │
│           │  foo | 1.3 | 0       │                              │
│           └──────────────────────┘                              │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Implementation Plan

### Phase 1: Rust Extension (glassware-orchestrator)

#### 1.1 Add Version History Module

**File:** `glassware-orchestrator/src/version_scanner.rs`

```rust
/// Query npm registry for all versions of a package
pub async fn get_package_versions(package: &str) -> Result<Vec<VersionInfo>>;

/// Sample versions based on policy
pub fn sample_versions(
    versions: &[VersionInfo],
    policy: &VersionPolicy
) -> Vec<String>;

/// Scan multiple versions of a package
pub async fn scan_package_versions(
    package: &str,
    versions: &[String],
    config: &ScanConfig
) -> Vec<VersionScanResult>;
```

#### 1.2 Version Policies

```rust
pub enum VersionPolicy {
    /// Scan last N versions (e.g., last 10)
    LastN(usize),
    
    /// Scan versions from last X days
    LastDays(u32),
    
    /// Scan all versions
    All,
    
    /// Scan specific versions
    Specific(Vec<String>),
    
    /// Scan major versions only (latest of each major)
    MajorReleases,
}
```

#### 1.3 CLI Integration

```bash
# Scan last 10 versions
glassware-orchestrator scan-npm --versions last-10 lodash

# Scan versions from last 6 months
glassware-orchestrator scan-npm --versions last-180d express

# Scan all versions (careful!)
glassware-orchestrator scan-npm --versions all suspicious-pkg

# Scan with custom policy
glassware-orchestrator scan-npm --versions "1.0.0,1.1.0,1.2.0" pkg
```

#### 1.4 Results Database Schema

```sql
CREATE TABLE version_scans (
    id INTEGER PRIMARY KEY,
    package_name TEXT NOT NULL,
    version TEXT NOT NULL,
    scan_timestamp DATETIME NOT NULL,
    findings_count INTEGER NOT NULL,
    threat_score REAL NOT NULL,
    is_malicious BOOLEAN NOT NULL,
    scan_result_json TEXT NOT NULL,
    UNIQUE(package_name, version)
);

CREATE INDEX idx_package ON version_scans(package_name);
CREATE INDEX idx_timestamp ON version_scans(scan_timestamp);
CREATE INDEX idx_malicious ON version_scans(is_malicious);
```

---

### Phase 2: Python Harness Extension

#### 2.1 Package Sampling Script

**File:** `harness/version_sampler.py`

```python
#!/usr/bin/env python3
"""
Sample packages for version history scanning

Criteria:
- Updated in last month/week
- New packages (published in last month)
- Random from diverse categories
- High download count (popular targets)
"""

import requests
from datetime import datetime, timedelta
from pathlib import Path

NPM_REGISTRY = "https://registry.npmjs.org"
NPM_SEARCH = "https://registry.npmjs.org/-/v1/search"

def sample_recent_packages(
    categories: list[str],
    samples_per_category: int = 50,
    updated_since_days: int = 30
) -> list[dict]:
    """Sample packages updated in last N days"""
    packages = []
    cutoff_date = datetime.now() - timedelta(days=updated_since_days)
    
    for category in categories:
        # Search npm for category
        results = search_npm(category, size=samples_per_category * 2)
        
        for pkg in results[:samples_per_category]:
            metadata = fetch_package_metadata(pkg['name'])
            if metadata:
                modified = parse_date(metadata.get('time', {}).get('modified'))
                if modified and modified > cutoff_date:
                    packages.append({
                        'name': pkg['name'],
                        'latest_version': pkg['version'],
                        'last_updated': modified.isoformat(),
                        'category': category,
                    })
    
    return packages


def sample_new_packages(
    samples: int = 100,
    published_since_days: int = 30
) -> list[dict]:
    """Sample newly published packages"""
    # Use npm search with date filters
    ...


def sample_popular_packages(
    samples: int = 100,
    min_downloads: int = 10000
) -> list[dict]:
    """Sample popular packages (high download count)"""
    # Use npm-stat or similar API
    ...
```

#### 2.2 Version History Scanner

**File:** `harness/version_history_scanner.py`

```python
#!/usr/bin/env python3
"""
Background version history scanner

Usage:
    python version_history_scanner.py \
        --packages packages.txt \
        --policy last-10 \
        --output results.db \
        --log scan.log
"""

import argparse
import sqlite3
import json
import subprocess
from datetime import datetime
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor

def get_package_versions(package: str) -> list[str]:
    """Query npm registry for all versions"""
    resp = requests.get(f"{NPM_REGISTRY}/{package}")
    resp.raise_for_status()
    data = resp.json()
    return list(data.get('versions', {}).keys())


def scan_package_version(package: str, version: str) -> dict:
    """Scan single package version with glassware"""
    result = subprocess.run(
        [
            GLASSWARE_ORCHESTRATOR,
            '--cache-db', CACHE_DB,
            '--format', 'json',
            'scan-npm', f'{package}@{version}'
        ],
        capture_output=True, text=True, timeout=120
    )
    
    if result.returncode == 0:
        return json.loads(result.stdout)
    else:
        return {'error': result.stderr}


def save_to_database(result: dict, db_path: str):
    """Save scan result to SQLite"""
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
    cursor.execute('''
        INSERT OR REPLACE INTO version_scans
        (package_name, version, scan_timestamp, findings_count, 
         threat_score, is_malicious, scan_result_json)
        VALUES (?, ?, ?, ?, ?, ?, ?)
    ''', (
        result['package_name'],
        result['version'],
        datetime.now().isoformat(),
        result.get('findings_count', 0),
        result.get('threat_score', 0.0),
        result.get('is_malicious', False),
        json.dumps(result)
    ))
    
    conn.commit()
    conn.close()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--packages', required=True)
    parser.add_argument('--policy', default='last-10')
    parser.add_argument('--output', required=True)
    parser.add_argument('--log', default='version-scan.log')
    parser.add_argument('--workers', type=int, default=5)
    args = parser.parse_args()
    
    # Load package list
    packages = load_packages(args.packages)
    
    # Initialize database
    init_database(args.output)
    
    # Scan with thread pool
    with ThreadPoolExecutor(max_workers=args.workers) as executor:
        for pkg in packages:
            versions = get_package_versions(pkg['name'])
            sampled = sample_versions(versions, args.policy)
            
            for version in sampled:
                executor.submit(
                    scan_and_save,
                    pkg['name'],
                    version,
                    args.output,
                    args.log
                )
```

#### 2.3 Analysis Scripts

**File:** `harness/version_analysis.py`

```python
#!/usr/bin/env python3
"""
Analyze version history scan results

Usage:
    python version_analysis.py results.db --report report.md
"""

import sqlite3
import argparse
from collections import defaultdict

def find_malicious_versions(db_path: str) -> list[dict]:
    """Find packages with malicious versions"""
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
    cursor.execute('''
        SELECT package_name, version, threat_score, findings_count
        FROM version_scans
        WHERE is_malicious = 1
        ORDER BY threat_score DESC
    ''')
    
    return cursor.fetchall()


def find_version_anomalies(db_path: str) -> list[dict]:
    """Find versions with sudden finding spikes"""
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
    # Find packages where one version has findings but adjacent don't
    cursor.execute('''
        SELECT package_name, version, findings_count
        FROM version_scans
        WHERE findings_count > 0
        ORDER BY package_name, version
    ''')
    
    # Analyze for anomalies
    ...


def generate_report(db_path: str, output: str):
    """Generate markdown report"""
    malicious = find_malicious_versions(db_path)
    anomalies = find_version_anomalies(db_path)
    
    report = f"""
# Version History Scan Report

## Summary
- Total packages scanned: {total_packages}
- Total versions scanned: {total_versions}
- Malicious versions found: {len(malicious)}
- Anomalous versions: {len(anomalies)}

## Malicious Versions

| Package | Version | Threat Score | Findings |
|---------|---------|--------------|----------|
"""
    
    for pkg, ver, score, findings in malicious:
        report += f"| {pkg} | {ver} | {score:.2f} | {findings} |\n"
    
    with open(output, 'w') as f:
        f.write(report)
```

---

### Phase 3: Background Job System

#### 3.1 Long-Running Scan Manager

**File:** `harness/background_scanner.py`

```python
#!/usr/bin/env python3
"""
Background scanner with checkpoint/resume

Usage:
    python background_scanner.py \
        --config scan-config.yaml \
        --state state.json \
        --log scan.log
"""

import yaml
import json
import time
from datetime import datetime
from pathlib import Path

class BackgroundScanner:
    def __init__(self, config: dict):
        self.config = config
        self.state = self.load_state()
        
    def load_state(self) -> dict:
        """Load or initialize scan state"""
        state_file = Path(self.config['state_file'])
        if state_file.exists():
            return json.load(open(state_file))
        return {
            'started_at': datetime.now().isoformat(),
            'packages_scanned': 0,
            'versions_scanned': 0,
            'current_package': None,
            'last_checkpoint': None,
        }
    
    def save_state(self):
        """Save checkpoint"""
        with open(self.config['state_file'], 'w') as f:
            json.dump(self.state, f, indent=2)
    
    def run(self):
        """Main scan loop"""
        while True:
            # Get next package from queue
            pkg = self.get_next_package()
            if not pkg:
                break
            
            self.state['current_package'] = pkg['name']
            
            # Get versions
            versions = get_package_versions(pkg['name'])
            sampled = sample_versions(versions, self.config['policy'])
            
            # Scan each version
            for version in sampled:
                result = scan_package_version(pkg['name'], version)
                save_to_database(result, self.config['database'])
                
                self.state['versions_scanned'] += 1
                
                # Checkpoint every N versions
                if self.state['versions_scanned'] % 10 == 0:
                    self.save_state()
                    self.log_progress()
            
            self.state['packages_scanned'] += 1
        
        self.save_state()
        self.generate_final_report()
```

---

## Sampling Strategy for 500 Packages

### Categories to Sample

```python
CATEGORIES = {
    # High-risk: install scripts, native code
    'native-build': ['node-gyp', 'bindings', 'prebuild', 'nan'],
    'install-scripts': ['preinstall', 'postinstall', 'install'],
    
    # Popular ecosystem (high impact if compromised)
    'web-frameworks': ['react', 'vue', 'angular', 'svelte'],
    'backend': ['express', 'fastify', 'koa', 'hapi'],
    'database': ['mongoose', 'sequelize', 'prisma', 'typeorm'],
    
    # Developer tools (trusted, high privilege)
    'devtools': ['eslint', 'prettier', 'typescript', 'babel'],
    'testing': ['jest', 'mocha', 'vitest', 'cypress'],
    
    # AI/ML trend (currently targeted)
    'ai-ml': ['ai', 'llm', 'openai', 'anthropic', 'langchain'],
    
    # Utilities (widely used)
    'utils': ['lodash', 'async', 'moment', 'dayjs', 'axios'],
    'logging': ['winston', 'pino', 'log4js', 'debug'],
    
    # Security/crypto (legitimate crypto usage)
    'crypto': ['crypto', 'bcrypt', 'jsonwebtoken', 'jose'],
}

# Sample strategy:
# - 50 packages from each category = 500 total
# - Mix of: recent updates, new packages, popular packages
```

### Version Sampling Policy

```python
VERSION_POLICY = {
    # For packages with <10 versions: scan all
    'small': 'all',
    
    # For packages with 10-100 versions: scan last 10 + random 5
    'medium': 'last-10-plus-random-5',
    
    # For packages with >100 versions: scan last 10 + major releases
    'large': 'last-10-plus-majors',
    
    # Time-based: scan versions from last 6 months
    'recent': 'last-180-days',
}
```

---

## Expected Output

### Database Schema

```sql
-- Main scan results
CREATE TABLE version_scans (
    id INTEGER PRIMARY KEY,
    package_name TEXT NOT NULL,
    version TEXT NOT NULL,
    scan_timestamp DATETIME NOT NULL,
    findings_count INTEGER NOT NULL,
    threat_score REAL NOT NULL,
    is_malicious BOOLEAN NOT NULL,
    scan_result_json TEXT NOT NULL,
    UNIQUE(package_name, version)
);

-- Package metadata
CREATE TABLE packages (
    name TEXT PRIMARY KEY,
    category TEXT,
    first_published DATETIME,
    last_updated DATETIME,
    total_versions INTEGER,
    latest_version TEXT
);

-- Findings timeline (for visualization)
CREATE VIEW findings_timeline AS
SELECT 
    package_name,
    version,
    scan_timestamp,
    findings_count,
    threat_score
FROM version_scans
ORDER BY package_name, scan_timestamp;
```

### Report Example

```markdown
# Version History Scan Report

**Scan Date:** 2026-03-21  
**Packages Scanned:** 500  
**Versions Scanned:** 4,823  
**Duration:** 18 hours

## Summary

| Metric | Value |
|--------|-------|
| Total packages | 500 |
| Total versions | 4,823 |
| Malicious versions | 3 |
| Suspicious versions | 12 |
| Clean versions | 4,808 |

## 🚨 Malicious Versions Detected

| Package | Version | Threat Score | Findings | Category |
|---------|---------|--------------|----------|----------|
| suspicious-pkg | 2.3.4 | 9.2 | 15 | ai-ml |
| evil-utils | 1.0.1 | 8.5 | 8 | utils |
| compromised-lib | 3.1.0 | 7.8 | 5 | database |

## ⚠️ Version Anomalies

Packages where malicious code appeared in specific versions:

### suspicious-pkg
```
v2.3.3: ✅ Clean (0 findings)
v2.3.4: 🚨 MALICIOUS (15 findings) ← Infiltration point!
v2.3.5: ✅ Clean (0 findings)  ← Fixed? Or different attack?
v2.4.0: ✅ Clean (0 findings)
```

### Timeline Visualization
[Graph showing findings count over version history]
```

---

## Next Steps

1. ✅ **Implement Rust version scanner** (`glassware-orchestrator/src/version_scanner.rs`)
2. ✅ **Add CLI flags** for version policies
3. ✅ **Create Python sampling script** (`harness/version_sampler.py`)
4. ✅ **Build background scanner** (`harness/background_scanner.py`)
5. ✅ **Create analysis scripts** (`harness/version_analysis.py`)
6. ⏳ **Run 500 package scan** (estimated 18-24 hours)
7. ⏳ **Analyze results** and publish report

---

**End of Plan**
