# Glassware Implementation Plan v0.9.0

**Created:** 2026-03-21  
**Status:** Ready to Implement  
**Priority:** High → Low

---

## Phase 1: CLI Flag Validation & State Management (CRITICAL)

### Problem
Multiple scanning flags could conflict or create undefined behavior:
- `--llm` without API key configured
- `--versions` with incompatible policies
- `--cache-db` conflicts with `--no-cache`
- Running multiple scans simultaneously without tracking

### Solution: Flag Validation Layer

**File:** `glassware-orchestrator/src/cli_validator.rs` (NEW)

```rust
pub struct ValidationErrors {
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl Cli {
    pub fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = vec![];
        let mut warnings = vec![];
        
        // LLM validation
        #[cfg(feature = "llm")]
        if self.llm {
            if std::env::var("GLASSWARE_LLM_BASE_URL").is_err() {
                errors.push("--llm requires GLASSWARE_LLM_BASE_URL");
            }
            if std::env::var("GLASSWARE_LLM_API_KEY").is_err() {
                errors.push("--llm requires GLASSWARE_LLM_API_KEY");
            }
        }
        
        // Cache conflicts
        if self.no_cache && self.cache_db != DEFAULT_CACHE_DB {
            errors.push("--no-cache conflicts with --cache-db");
        }
        
        // Version policy validation
        if let Some(policy) = &self.versions {
            if !VersionPolicy::is_valid(policy) {
                errors.push(format!("Invalid version policy: {}", policy));
            }
        }
        
        // Warnings
        if self.llm && self.severity == SeverityLevel::Info {
            warnings.push("--llm with --severity info may be slow");
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationErrors { errors, warnings })
        }
    }
}
```

**CLI Changes:**
```bash
# Before: Silent failure or crash
./glassware-orchestrator --llm --no-cache scan-npm pkg

# After: Immediate error with fix suggestions
Error: Invalid flag combination

  × --llm requires GLASSWARE_LLM_API_KEY
  × --no-cache conflicts with --cache-db

Help:
  - Set GLASSWARE_LLM_API_KEY environment variable
  - Remove --no-cache or specify a different --cache-db

Usage: glassware-orchestrator [OPTIONS] <COMMAND>
```

---

## Phase 2: Scan State Tracking

### Problem
No visibility into:
- What scans are currently running
- What scans completed successfully
- Failed scans and why
- Scan history for auditing

### Solution: Scan Registry

**File:** `glassware-orchestrator/src/scan_registry.rs` (NEW)

```rust
pub struct ScanRegistry {
    state_file: PathBuf,
    scans: Vec<ScanRecord>,
}

pub struct ScanRecord {
    pub id: String,              // UUID
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: ScanStatus,
    pub command: String,
    pub packages: Vec<String>,
    pub version_policy: Option<String>,
    pub findings_count: usize,
    pub malicious_count: usize,
    pub error: Option<String>,
}

pub enum ScanStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl ScanRegistry {
    pub fn start_scan(&mut self, command: &str, packages: &[String]) -> String;
    pub fn complete_scan(&mut self, id: &str, findings: usize);
    pub fn fail_scan(&mut self, id: &str, error: &str);
    pub fn list_scans(&self, status: Option<ScanStatus>) -> Vec<&ScanRecord>;
    pub fn get_running_scans(&self) -> Vec<&ScanRecord>;
}
```

**State File:** `.glassware-scan-registry.json`

```json
{
  "scans": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "started_at": "2026-03-21T06:00:00Z",
      "completed_at": "2026-03-21T06:30:00Z",
      "status": "completed",
      "command": "scan-file /tmp/packages.txt --versions last-10",
      "packages": ["lodash", "express", "react"],
      "version_policy": "last-10",
      "findings_count": 15,
      "malicious_count": 2,
      "error": null
    },
    {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "started_at": "2026-03-21T06:35:00Z",
      "completed_at": null,
      "status": "running",
      "command": "scan-npm --versions last-5 suspicious-pkg",
      "packages": ["suspicious-pkg"],
      "version_policy": "last-5",
      "findings_count": 0,
      "malicious_count": 0,
      "error": null
    }
  ]
}
```

**CLI Commands:**
```bash
# List all scans
glassware-orchestrator scan-list

# List running scans only
glassware-orchestrator scan-list --status running

# Show scan details
glassware-orchestrator scan-show 550e8400-e29b-41d4-a716-446655440000

# Cancel running scan
glassware-orchestrator scan-cancel 660e8400-e29b-41d4-a716-446655440001
```

---

## Phase 3: Version History Scanning (Rust)

### Implementation

**File:** `glassware-orchestrator/src/version_scanner.rs` (NEW)

```rust
pub struct VersionScanner {
    client: Client,
    downloader: Downloader,
    scanner: Scanner,
}

impl VersionScanner {
    /// Fetch all versions of a package from npm
    pub async fn get_versions(&self, package: &str) -> Result<Vec<VersionInfo>>;
    
    /// Sample versions based on policy
    pub fn sample_versions(
        &self,
        versions: &[VersionInfo],
        policy: &VersionPolicy
    ) -> Vec<String>;
    
    /// Scan multiple versions
    pub async fn scan_versions(
        &self,
        package: &str,
        versions: &[String],
        config: &ScanConfig
    ) -> Vec<VersionScanResult>;
}

pub enum VersionPolicy {
    LastN(usize),           // last-10
    LastDays(u32),          // last-180d
    All,                    // all
    MajorReleases,          // major
    Specific(Vec<String>),  // "1.0.0,1.1.0"
}
```

**CLI Integration:**
```bash
# Add to existing scan-npm command
glassware-orchestrator scan-npm --versions last-10 lodash

# Or as separate command
glassware-orchestrator scan-versions --policy last-10 lodash
```

---

## Phase 4: Python Package Sampler

**File:** `harness/version_sampler.py` (NEW)

```python
#!/usr/bin/env python3
"""
Sample diverse packages for version history scanning
"""

import requests
import json
from datetime import datetime, timedelta
from pathlib import Path

NPM_SEARCH = "https://registry.npmjs.org/-/v1/search"
NPM_REGISTRY = "https://registry.npmjs.org"

CATEGORIES = {
    'native-build': ['node-gyp', 'bindings', 'prebuild', 'nan', 'node-addon-api'],
    'install-scripts': ['preinstall', 'postinstall', 'install'],
    'web-frameworks': ['react', 'vue', 'angular', 'svelte', 'next'],
    'backend': ['express', 'fastify', 'koa', 'hapi', 'nest'],
    'database': ['mongoose', 'sequelize', 'prisma', 'typeorm', 'knex'],
    'devtools': ['eslint', 'prettier', 'typescript', 'babel', 'webpack'],
    'testing': ['jest', 'mocha', 'vitest', 'cypress', 'playwright'],
    'ai-ml': ['ai', 'llm', 'openai', 'anthropic', 'langchain'],
    'utils': ['lodash', 'async', 'moment', 'dayjs', 'axios'],
    'crypto': ['bcrypt', 'jsonwebtoken', 'jose', 'node-forge'],
}

def sample_packages(
    samples_per_category: int = 50,
    updated_since_days: int = 30
) -> list[dict]:
    """Sample packages updated in last N days from each category"""
    packages = []
    
    for category, keywords in CATEGORIES.items():
        for keyword in keywords:
            results = search_npm(keyword, size=samples_per_category)
            for pkg in filter_by_date(results, updated_since_days):
                pkg['category'] = category
                packages.append(pkg)
            
            if len([p for p in packages if p['category'] == category]) >= samples_per_category:
                break
    
    return packages[:500]  # Limit to 500 total


def search_npm(keyword: str, size: int = 100) -> list:
    """Search npm registry"""
    resp = requests.get(
        NPM_SEARCH,
        params={'text': keyword, 'size': size},
        timeout=30
    )
    resp.raise_for_status()
    return resp.json().get('objects', [])


def filter_by_date(packages: list, days: int) -> list:
    """Filter packages updated in last N days"""
    cutoff = datetime.now() - timedelta(days=days)
    filtered = []
    
    for pkg_obj in packages:
        pkg = pkg_obj.get('package', {})
        # Check if package was recently updated
        # (npm search doesn't include date, so we fetch metadata)
        metadata = fetch_metadata(pkg['name'])
        if metadata:
            modified = parse_date(metadata.get('time', {}).get('modified'))
            if modified and modified > cutoff:
                filtered.append(pkg)
    
    return filtered


def main():
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument('--samples', type=int, default=50)
    parser.add_argument('--days', type=int, default=30)
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    
    packages = sample_packages(args.samples, args.days)
    
    # Output as plain text (package names only)
    with open(args.output, 'w') as f:
        for pkg in packages:
            f.write(f"{pkg['name']}\n")
    
    print(f"Sampled {len(packages)} packages → {args.output}")


if __name__ == '__main__':
    main()
```

---

## Phase 5: Background Scanner with Checkpointing

**File:** `harness/background_scanner.py` (NEW)

```python
#!/usr/bin/env python3
"""
Background version history scanner with checkpoint/resume

Usage:
    python background_scanner.py \
        --packages packages.txt \
        --policy last-10 \
        --output results.db \
        --state state.json \
        --log scan.log \
        --workers 5
"""

import argparse
import sqlite3
import json
import subprocess
import time
from datetime import datetime
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor, as_completed

GLASSWARE = "../target/debug/glassware-orchestrator"
CACHE_DB = "/tmp/glassware-version-cache.db"

class BackgroundScanner:
    def __init__(self, config: dict):
        self.config = config
        self.state = self.load_state()
        self.init_database()
        
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
            'errors': [],
        }
    
    def save_state(self):
        """Save checkpoint"""
        self.state['last_checkpoint'] = datetime.now().isoformat()
        with open(self.config['state_file'], 'w') as f:
            json.dump(self.state, f, indent=2)
    
    def init_database(self):
        """Initialize results database"""
        conn = sqlite3.connect(self.config['database'])
        cursor = conn.cursor()
        
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS version_scans (
                id INTEGER PRIMARY KEY,
                package_name TEXT NOT NULL,
                version TEXT NOT NULL,
                scan_timestamp DATETIME NOT NULL,
                findings_count INTEGER NOT NULL,
                threat_score REAL NOT NULL,
                is_malicious BOOLEAN NOT NULL,
                scan_result_json TEXT NOT NULL,
                UNIQUE(package_name, version)
            )
        ''')
        
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS scan_metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )
        ''')
        
        # Record scan start
        cursor.execute('''
            INSERT OR REPLACE INTO scan_metadata (key, value)
            VALUES ('started_at', ?)
        ''', (datetime.now().isoformat(),))
        
        conn.commit()
        conn.close()
    
    def get_package_versions(self, package: str) -> list[str]:
        """Query npm registry for all versions"""
        resp = requests.get(f"{NPM_REGISTRY}/{package}")
        if resp.status_code != 200:
            return []
        data = resp.json()
        return list(data.get('versions', {}).keys())
    
    def sample_versions(self, versions: list[str], policy: str) -> list[str]:
        """Sample versions based on policy"""
        if policy == 'all':
            return versions
        elif policy.startswith('last-'):
            n = int(policy.split('-')[1])
            return versions[-n:]
        elif policy.startswith('last-') and 'd' in policy:
            # Date-based sampling (simplified - just take last N)
            n = int(policy.replace('last-', '').replace('d', ''))
            return versions[-min(n, len(versions)):]
        else:
            return versions[-10:]  # Default to last 10
    
    def scan_version(self, package: str, version: str) -> dict:
        """Scan single package version"""
        try:
            result = subprocess.run(
                [
                    GLASSWARE,
                    '--cache-db', CACHE_DB,
                    '--format', 'json',
                    'scan-npm', f'{package}@{version}'
                ],
                capture_output=True, text=True, timeout=120
            )
            
            if result.returncode == 0:
                data = json.loads(result.stdout)
                return {
                    'package_name': package,
                    'version': version,
                    'findings_count': data.get('summary', {}).get('total_findings', 0),
                    'threat_score': data.get('summary', {}).get('average_threat_score', 0.0),
                    'is_malicious': data.get('summary', {}).get('malicious_packages', 0) > 0,
                    'scan_result_json': result.stdout,
                    'error': None,
                }
            else:
                return {
                    'package_name': package,
                    'version': version,
                    'findings_count': 0,
                    'threat_score': 0.0,
                    'is_malicious': False,
                    'scan_result_json': '{}',
                    'error': result.stderr,
                }
        except Exception as e:
            return {
                'package_name': package,
                'version': version,
                'findings_count': 0,
                'threat_score': 0.0,
                'is_malicious': False,
                'scan_result_json': '{}',
                'error': str(e),
            }
    
    def save_result(self, result: dict):
        """Save scan result to database"""
        conn = sqlite3.connect(self.config['database'])
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
            result['findings_count'],
            result['threat_score'],
            result['is_malicious'],
            result['scan_result_json'],
        ))
        
        conn.commit()
        conn.close()
    
    def log_progress(self, result: dict):
        """Log progress to file"""
        with open(self.config['log_file'], 'a') as f:
            timestamp = datetime.now().isoformat()
            status = '⚠️' if result['error'] else ('🚨' if result['is_malicious'] else '✅')
            f.write(f"[{timestamp}] {status} {result['package_name']}@{result['version']} "
                   f"(findings: {result['findings_count']})\n")
    
    def run(self):
        """Main scan loop"""
        packages = self.load_packages()
        
        with ThreadPoolExecutor(max_workers=self.config['workers']) as executor:
            futures = []
            
            for pkg_name in packages:
                self.state['current_package'] = pkg_name
                
                # Get versions
                versions = self.get_package_versions(pkg_name)
                if not versions:
                    continue
                
                # Sample versions
                sampled = self.sample_versions(versions, self.config['policy'])
                
                # Submit scan jobs
                for version in sampled:
                    future = executor.submit(self.scan_version, pkg_name, version)
                    futures.append((pkg_name, version, future))
                
                # Process completed scans
                for pkg, ver, future in futures:
                    result = future.result()
                    self.save_result(result)
                    self.log_progress(result)
                    
                    self.state['versions_scanned'] += 1
                    
                    # Checkpoint every 10 versions
                    if self.state['versions_scanned'] % 10 == 0:
                        self.save_state()
                        self.print_status()
                
                self.state['packages_scanned'] += 1
                futures = []
        
        self.save_state()
        self.generate_report()
    
    def print_status(self):
        """Print current status"""
        print(f"[{datetime.now().isoformat()}] "
              f"Packages: {self.state['packages_scanned']}, "
              f"Versions: {self.state['versions_scanned']}")
    
    def load_packages(self) -> list[str]:
        """Load package list from file"""
        with open(self.config['packages_file']) as f:
            return [line.strip() for line in f if line.strip()]
    
    def generate_report(self):
        """Generate final report"""
        conn = sqlite3.connect(self.config['database'])
        cursor = conn.cursor()
        
        cursor.execute('SELECT COUNT(*) FROM version_scans')
        total = cursor.fetchone()[0]
        
        cursor.execute('SELECT COUNT(*) FROM version_scans WHERE is_malicious = 1')
        malicious = cursor.fetchone()[0]
        
        cursor.execute('SELECT SUM(findings_count) FROM version_scans')
        findings = cursor.fetchone()[0] or 0
        
        report = f"""
# Version History Scan Report

**Generated:** {datetime.now().isoformat()}

## Summary
- Total versions scanned: {total}
- Malicious versions: {malicious}
- Total findings: {findings}
- Duration: {self.state['started_at']} to now
"""
        
        with open('version-scan-report.md', 'w') as f:
            f.write(report)
        
        conn.close()
        print(f"\nReport generated: version-scan-report.md")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--packages', required=True, help='Package list file')
    parser.add_argument('--policy', default='last-10',
                       help='Version policy: last-10, last-180d, all, major')
    parser.add_argument('--output', required=True, help='Output database')
    parser.add_argument('--state', default='scan-state.json', help='State file')
    parser.add_argument('--log', default='scan.log', help='Log file')
    parser.add_argument('--workers', type=int, default=5, help='Parallel workers')
    args = parser.parse_args()
    
    config = {
        'packages_file': args.packages,
        'policy': args.policy,
        'database': args.output,
        'state_file': args.state,
        'log_file': args.log,
        'workers': args.workers,
    }
    
    scanner = BackgroundScanner(config)
    scanner.run()


if __name__ == '__main__':
    main()
```

---

## Implementation Order

### Week 1: Foundation
1. ✅ CLI flag validation (Phase 1)
2. ✅ Scan registry (Phase 2)
3. ✅ Version scanner Rust module (Phase 3)

### Week 2: Scanning
4. ✅ Python package sampler (Phase 4)
5. ✅ Background scanner (Phase 5)
6. ⏳ Run first 500-package scan

### Week 3: Analysis
7. ⏳ Analysis scripts
8. ⏳ Report generation
9. ⏳ LLM integration for flagged versions

---

## Files to Create/Modify

### New Files
- `glassware-orchestrator/src/cli_validator.rs`
- `glassware-orchestrator/src/scan_registry.rs`
- `glassware-orchestrator/src/version_scanner.rs`
- `harness/version_sampler.py`
- `harness/background_scanner.py`
- `harness/version_analysis.py`
- `.glassware-scan-registry.json` (auto-created)

### Modified Files
- `glassware-orchestrator/src/main.rs` (add validation, registry)
- `glassware-orchestrator/src/cli.rs` (add --versions flag)
- `glassware-orchestrator/src/orchestrator.rs` (version scanning support)
- `glassware-orchestrator/src/lib.rs` (export new modules)

---

## Quick Reference: Scan Status

**Check running scans:**
```bash
cat .glassware-scan-registry.json | jq '.scans[] | select(.status == "running")'
```

**Check scan history:**
```bash
cat .glassware-scan-registry.json | jq '.scans[]'
```

**View scan log:**
```bash
tail -f scan.log
```

**View results database:**
```bash
sqlite3 results.db "SELECT package_name, version, findings_count FROM version_scans ORDER BY scan_timestamp DESC LIMIT 20;"
```

---

**End of Plan**
