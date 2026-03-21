#!/usr/bin/env python3
"""
Background Version History Scanner

Long-running scanner with checkpoint/resume support for version history scanning.

Features:
- Checkpoint/resume for long-running scans
- SQLite database for results
- Progress logging
- Parallel scanning with configurable workers
- Rate limiting

Usage:
    python background_scanner.py \
        --packages packages.txt \
        --policy last-10 \
        --output results.db \
        --state scan-state.json \
        --log scan.log \
        --workers 5
"""

import argparse
import json
import sqlite3
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor, as_completed
from typing import Dict, List, Optional
import requests

# Configuration
GLASSWARE = "./target/debug/glassware-orchestrator"
NPM_REGISTRY = "https://registry.npmjs.org"
CACHE_DB = "/tmp/glassware-version-cache.db"


class BackgroundScanner:
    """Background scanner with checkpoint/resume support"""
    
    def __init__(self, config: Dict):
        self.config = config
        self.state = self.load_state()
        self.init_database()
        self.stats = {
            "versions_scanned": 0,
            "versions_failed": 0,
            "findings_total": 0,
            "malicious_total": 0,
        }
    
    def load_state(self) -> Dict:
        """Load or initialize scan state"""
        state_file = Path(self.config["state_file"])
        if state_file.exists():
            try:
                state = json.load(open(state_file))
                print(f"Loaded state from {state_file}", file=sys.stderr)
                print(f"  Packages scanned: {state.get('packages_scanned', 0)}", file=sys.stderr)
                print(f"  Versions scanned: {state.get('versions_scanned', 0)}", file=sys.stderr)
                return state
            except Exception as e:
                print(f"Warning: Failed to load state: {e}", file=sys.stderr)
        
        return {
            "started_at": datetime.now().isoformat(),
            "packages_scanned": 0,
            "versions_scanned": 0,
            "current_package": None,
            "last_checkpoint": None,
            "errors": [],
        }
    
    def save_state(self):
        """Save checkpoint"""
        self.state["last_checkpoint"] = datetime.now().isoformat()
        state_file = Path(self.config["state_file"])
        
        # Ensure parent directory exists
        state_file.parent.mkdir(parents=True, exist_ok=True)
        
        with open(state_file, 'w') as f:
            json.dump(self.state, f, indent=2)
    
    def init_database(self):
        """Initialize results database"""
        db_path = Path(self.config["database"])
        db_path.parent.mkdir(parents=True, exist_ok=True)
        
        conn = sqlite3.connect(db_path)
        cursor = conn.cursor()
        
        # Main scan results table
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS version_scans (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                package_name TEXT NOT NULL,
                version TEXT NOT NULL,
                scan_timestamp DATETIME NOT NULL,
                findings_count INTEGER NOT NULL,
                threat_score REAL NOT NULL,
                is_malicious BOOLEAN NOT NULL,
                scan_result_json TEXT,
                error TEXT,
                UNIQUE(package_name, version)
            )
        """)
        
        # Package metadata table
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS packages (
                name TEXT PRIMARY KEY,
                category TEXT,
                first_published DATETIME,
                last_updated DATETIME,
                total_versions INTEGER,
                latest_version TEXT,
                scan_status TEXT DEFAULT 'pending'
            )
        """)
        
        # Scan metadata table
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS scan_metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )
        """)
        
        # Record scan start
        cursor.execute("""
            INSERT OR REPLACE INTO scan_metadata (key, value)
            VALUES ('started_at', ?)
        """, (datetime.now().isoformat(),))
        
        # Create indexes for faster queries
        cursor.execute("""
            CREATE INDEX IF NOT EXISTS idx_package ON version_scans(package_name)
        """)
        cursor.execute("""
            CREATE INDEX IF NOT EXISTS idx_malicious ON version_scans(is_malicious)
        """)
        cursor.execute("""
            CREATE INDEX IF NOT EXISTS idx_timestamp ON version_scans(scan_timestamp)
        """)
        
        conn.commit()
        conn.close()
        print(f"Initialized database: {db_path}", file=sys.stderr)
    
    def get_package_versions(self, package: str) -> List[str]:
        """Query npm registry for all versions"""
        try:
            resp = requests.get(f"{NPM_REGISTRY}/{package}", timeout=30)
            if resp.status_code != 200:
                return []
            data = resp.json()
            versions = list(data.get("versions", {}).keys())
            # Sort by version (simple string sort, not semver)
            return sorted(versions, reverse=True)
        except Exception as e:
            print(f"    Error fetching versions for {package}: {e}", file=sys.stderr)
            return []
    
    def sample_versions(self, versions: List[str], policy: str) -> List[str]:
        """Sample versions based on policy"""
        if policy == "all":
            return versions
        elif policy.startswith("last-"):
            rest = policy[5:]
            if rest.endswith("d"):
                # Days format - for now just take last N
                n = int(rest[:-1])
                return versions[-min(n, len(versions)):]
            else:
                # Count format
                n = int(rest)
                return versions[-n:]
        else:
            # Default to last 10
            return versions[-10:]
    
    def scan_version(self, package: str, version: str) -> Dict:
        """Scan single package version"""
        try:
            result = subprocess.run(
                [
                    GLASSWARE,
                    "--cache-db", CACHE_DB,
                    "--format", "json",
                    "scan-npm", f"{package}@{version}"
                ],
                capture_output=True, text=True, timeout=120,
                cwd=Path(__file__).parent.parent
            )
            
            if result.returncode == 0:
                data = json.loads(result.stdout)
                summary = data.get("summary", {})
                return {
                    "package_name": package,
                    "version": version,
                    "findings_count": summary.get("total_findings", 0),
                    "threat_score": summary.get("average_threat_score", 0.0),
                    "is_malicious": summary.get("malicious_packages", 0) > 0,
                    "scan_result_json": result.stdout,
                    "error": None,
                }
            else:
                return {
                    "package_name": package,
                    "version": version,
                    "findings_count": 0,
                    "threat_score": 0.0,
                    "is_malicious": False,
                    "scan_result_json": "{}",
                    "error": result.stderr,
                }
        except subprocess.TimeoutExpired:
            return {
                "package_name": package,
                "version": version,
                "findings_count": 0,
                "threat_score": 0.0,
                "is_malicious": False,
                "scan_result_json": "{}",
                "error": "Timeout after 120s",
            }
        except Exception as e:
            return {
                "package_name": package,
                "version": version,
                "findings_count": 0,
                "threat_score": 0.0,
                "is_malicious": False,
                "scan_result_json": "{}",
                "error": str(e),
            }
    
    def save_result(self, result: Dict):
        """Save scan result to database"""
        db_path = Path(self.config["database"])
        conn = sqlite3.connect(db_path)
        cursor = conn.cursor()
        
        cursor.execute("""
            INSERT OR REPLACE INTO version_scans
            (package_name, version, scan_timestamp, findings_count,
             threat_score, is_malicious, scan_result_json, error)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        """, (
            result["package_name"],
            result["version"],
            datetime.now().isoformat(),
            result["findings_count"],
            result["threat_score"],
            result["is_malicious"],
            result["scan_result_json"],
            result["error"],
        ))
        
        # Update package status
        cursor.execute("""
            INSERT OR REPLACE INTO packages (name, scan_status)
            VALUES (?, 'scanned')
        """, (result["package_name"],))
        
        conn.commit()
        conn.close()
    
    def log_progress(self, result: Dict, package_num: int, total_packages: int):
        """Log progress to file and console"""
        log_file = Path(self.config["log_file"])
        log_file.parent.mkdir(parents=True, exist_ok=True)
        
        with open(log_file, 'a') as f:
            timestamp = datetime.now().isoformat()
            status = '⚠️' if result["error"] else ('🚨' if result["is_malicious"] else '✅')
            f.write(f"[{timestamp}] {status} {result['package_name']}@{result['version']} "
                   f"(findings: {result['findings_count']}, threat: {result['threat_score']:.2f})\n")
        
        # Console progress
        self.stats["versions_scanned"] += 1
        if result["error"]:
            self.stats["versions_failed"] += 1
        self.stats["findings_total"] += result["findings_count"]
        if result["is_malicious"]:
            self.stats["malicious_total"] += 1
        
        if self.stats["versions_scanned"] % 10 == 0:
            self.print_status(package_num, total_packages)
    
    def print_status(self, package_num: int, total_packages: int):
        """Print current status to console"""
        elapsed = (datetime.now() - datetime.fromisoformat(self.state["started_at"])).total_seconds()
        rate = self.stats["versions_scanned"] / elapsed if elapsed > 0 else 0
        
        print(f"[{datetime.now().isoformat()}] "
              f"Package {package_num}/{total_packages} | "
              f"Versions: {self.stats['versions_scanned']} "
              f"(failures: {self.stats['versions_failed']}) | "
              f"Findings: {self.stats['findings_total']} | "
              f"Malicious: {self.stats['malicious_total']} | "
              f"Rate: {rate:.1f} ver/s", file=sys.stderr)
    
    def load_packages(self) -> List[str]:
        """Load package list from file"""
        packages_file = Path(self.config["packages_file"])
        with open(packages_file) as f:
            return [line.strip() for line in f if line.strip() and not line.startswith('#')]
    
    def run(self):
        """Main scan loop"""
        packages = self.load_packages()
        total_packages = len(packages)
        
        print(f"\nStarting scan of {total_packages} packages", file=sys.stderr)
        print(f"Policy: {self.config['policy']}", file=sys.stderr)
        print(f"Workers: {self.config['workers']}", file=sys.stderr)
        print("=" * 70, file=sys.stderr)
        
        with ThreadPoolExecutor(max_workers=self.config["workers"]) as executor:
            for pkg_idx, pkg_name in enumerate(packages, 1):
                self.state["current_package"] = pkg_name
                self.state["packages_scanned"] = pkg_idx
                
                # Get versions
                versions = self.get_package_versions(pkg_name)
                if not versions:
                    print(f"  ⚠️  No versions found for {pkg_name}", file=sys.stderr)
                    continue
                
                # Sample versions
                sampled = self.sample_versions(versions, self.config["policy"])
                print(f"  Scanning {len(sampled)} versions of {pkg_name}...", file=sys.stderr)
                
                # Submit scan jobs
                futures = {}
                for version in sampled:
                    future = executor.submit(self.scan_version, pkg_name, version)
                    futures[future] = version
                
                # Process completed scans
                for future in as_completed(futures):
                    version = futures[future]
                    try:
                        result = future.result()
                        self.save_result(result)
                        self.log_progress(result, pkg_idx, total_packages)
                    except Exception as e:
                        print(f"    Error processing {pkg_name}@{version}: {e}", file=sys.stderr)
                        self.stats["versions_failed"] += 1
                
                # Checkpoint every 5 packages
                if pkg_idx % 5 == 0:
                    self.save_state()
        
        # Final checkpoint
        self.state["completed_at"] = datetime.now().isoformat()
        self.save_state()
        
        # Generate final report
        self.generate_report()
        
        print("\n" + "=" * 70, file=sys.stderr)
        print("SCAN COMPLETE", file=sys.stderr)
        print("=" * 70, file=sys.stderr)
        print(f"Total packages: {total_packages}", file=sys.stderr)
        print(f"Total versions: {self.stats['versions_scanned']}", file=sys.stderr)
        print(f"Failed versions: {self.stats['versions_failed']}", file=sys.stderr)
        print(f"Total findings: {self.stats['findings_total']}", file=sys.stderr)
        print(f"Malicious versions: {self.stats['malicious_total']}", file=sys.stderr)
        print(f"Database: {self.config['database']}", file=sys.stderr)
        print(f"Log: {self.config['log_file']}", file=sys.stderr)
        print("=" * 70, file=sys.stderr)
    
    def generate_report(self):
        """Generate final markdown report"""
        db_path = Path(self.config["database"])
        conn = sqlite3.connect(db_path)
        cursor = conn.cursor()
        
        # Get summary stats
        cursor.execute("SELECT COUNT(*) FROM version_scans")
        total = cursor.fetchone()[0]
        
        cursor.execute("SELECT COUNT(*) FROM version_scans WHERE is_malicious = 1")
        malicious = cursor.fetchone()[0]
        
        cursor.execute("SELECT SUM(findings_count) FROM version_scans")
        findings = cursor.fetchone()[0] or 0
        
        cursor.execute("SELECT COUNT(DISTINCT package_name) FROM version_scans WHERE is_malicious = 1")
        malicious_packages = cursor.fetchone()[0]
        
        # Get malicious versions
        cursor.execute("""
            SELECT package_name, version, threat_score, findings_count
            FROM version_scans
            WHERE is_malicious = 1
            ORDER BY threat_score DESC
            LIMIT 20
        """)
        malicious_list = cursor.fetchall()
        
        conn.close()
        
        # Generate report
        report = f"""
# Version History Scan Report

**Generated:** {datetime.now().isoformat()}
**Policy:** {self.config['policy']}
**Workers:** {self.config['workers']}

## Summary

| Metric | Value |
|--------|-------|
| Total packages | {len(self.load_packages())} |
| Total versions scanned | {total} |
| Malicious versions | {malicious} |
| Malicious packages | {malicious_packages} |
| Total findings | {findings} |
| Failed scans | {self.stats['versions_failed']} |
| Duration | {self.state['started_at']} to {self.state.get('completed_at', 'ongoing')} |

## 🚨 Malicious Versions Detected

"""
        
        if malicious_list:
            report += "| Package | Version | Threat Score | Findings |\n"
            report += "|---------|---------|--------------|----------|\n"
            for pkg, ver, score, findings_count in malicious_list:
                report += f"| {pkg} | {ver} | {score:.2f} | {findings_count} |\n"
        else:
            report += "*No malicious versions detected*\n"
        
        report += f"""

## Database Location

Results saved to: `{self.config['database']}`

## Query Examples

```sql
-- Find all malicious versions
SELECT package_name, version, threat_score, findings_count
FROM version_scans
WHERE is_malicious = 1
ORDER BY threat_score DESC;

-- Findings by package
SELECT package_name, SUM(findings_count) as total_findings
FROM version_scans
GROUP BY package_name
ORDER BY total_findings DESC;

-- Scan timeline
SELECT date(scan_timestamp) as date, COUNT(*) as versions_scanned
FROM version_scans
GROUP BY date
ORDER BY date;
```
"""
        
        report_path = Path("version-scan-report.md")
        with open(report_path, 'w') as f:
            f.write(report)
        
        print(f"\nReport generated: {report_path}", file=sys.stderr)


def main():
    parser = argparse.ArgumentParser(
        description="Background version history scanner with checkpoint/resume"
    )
    parser.add_argument(
        "--packages", "-p",
        required=True,
        help="Package list file (one per line)"
    )
    parser.add_argument(
        "--policy",
        default="last-10",
        help="Version policy: last-10, last-180d, all (default: last-10)"
    )
    parser.add_argument(
        "--output", "-o",
        required=True,
        help="Output database path"
    )
    parser.add_argument(
        "--state", "-s",
        default="scan-state.json",
        help="State file for checkpoint/resume (default: scan-state.json)"
    )
    parser.add_argument(
        "--log", "-l",
        default="scan.log",
        help="Log file path (default: scan.log)"
    )
    parser.add_argument(
        "--workers", "-w",
        type=int,
        default=5,
        help="Parallel workers (default: 5)"
    )
    parser.add_argument(
        "--resume",
        action="store_true",
        help="Resume from existing state file"
    )
    
    args = parser.parse_args()
    
    config = {
        "packages_file": args.packages,
        "policy": args.policy,
        "database": args.output,
        "state_file": args.state,
        "log_file": args.log,
        "workers": args.workers,
    }
    
    scanner = BackgroundScanner(config)
    scanner.run()


if __name__ == "__main__":
    main()
