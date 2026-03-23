#!/usr/bin/env python3
"""
MCP Package Scanner - Optimized Parallel Version

Scans MCP packages in parallel for maximum throughput
"""

import subprocess
import json
import tempfile
import tarfile
import os
from pathlib import Path
from datetime import datetime
from concurrent.futures import ThreadPoolExecutor, as_completed
import requests
import hashlib

# Use environment variable or default to built Rust binary
GLASSWARE = os.environ.get(
    "GLASSWARE_BINARY",
    str(Path(__file__).parent.parent / "target" / "release" / "glassware-orchestrator")
)
EVIDENCE_DIR = Path(os.environ.get("GLASSWARE_EVIDENCE_DIR", "data/evidence"))
EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)

# Package cache (SQLite)
DB_PATH = Path("data/corpus.db")
db_instance = None

def get_db():
    """Get or create database instance"""
    global db_instance
    if db_instance is None:
        from database import Database
        db_instance = Database(DB_PATH)
    return db_instance

def sha256_file(path: Path) -> str:
    """Calculate SHA256 hash of a file"""
    sha256 = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            sha256.update(chunk)
    return sha256.hexdigest()

def download_and_scan_package(pkg_name: str, timeout: int = 60) -> dict:
    """Download and scan a single package (optimized with caching)"""
    result = {
        "package": pkg_name,
        "status": "pending",
        "findings": 0,
        "critical": 0,
        "cached": False,
        "error": None,
    }
    
    try:
        # Parse package name and version
        if "@" in pkg_name and pkg_name.index("@") == 0:
            # Scoped package: @scope/name@version
            parts = pkg_name.rsplit("@", 1)
            name = parts[0]
            version = parts[1] if len(parts) > 1 else "latest"
        elif "@" in pkg_name:
            # Unscoped with version: name@version
            parts = pkg_name.rsplit("@", 1)
            name = parts[0]
            version = parts[1] if len(parts) > 1 else "latest"
        else:
            # No version
            name = pkg_name
            version = "latest"
        
        with tempfile.TemporaryDirectory() as tmpdir:
            # Download with timeout
            dl_result = subprocess.run(
                ["npm", "pack", pkg_name],
                capture_output=True, text=True, cwd=tmpdir, timeout=30
            )
            if dl_result.returncode != 0:
                result["status"] = "download_failed"
                result["error"] = dl_result.stderr[:100]
                return result
            
            # Find tarball
            tarballs = list(Path(tmpdir).glob("*.tgz"))
            if not tarballs:
                result["status"] = "no_tarball"
                return result
            tarball = tarballs[0]
            
            # Calculate hash
            tarball_hash = sha256_file(tarball)
            
            # Check cache
            db = get_db()
            if db.is_already_scanned(name, version, tarball_hash, max_age_days=7):
                cached_result = db.get_cached_scan_result(name, version, tarball_hash)
                if cached_result:
                    result["status"] = "cached"
                    result["findings"] = cached_result["finding_count"]
                    result["cached"] = True
                    result["cache_info"] = f"Scanned at {cached_result['scanned_at']}"
                    return result
            
            # Extract
            with tarfile.open(tarball, "r:gz") as tar:
                tar.extractall(tmpdir, filter="data")
            
            pkg_dir = Path(tmpdir) / "package"
            if not pkg_dir.exists():
                result["status"] = "no_package_dir"
                return result
            
            # Scan with timeout
            scan_result = subprocess.run(
                [GLASSWARE, "--format", "json", str(pkg_dir)],
                capture_output=True, text=True, timeout=timeout
            )
            
            try:
                data = json.loads(scan_result.stdout)
                findings = data.get("findings", [])
                result["findings"] = len(findings)
                result["critical"] = len([f for f in findings if f.get("severity") == "critical"])
                result["status"] = "scanned"

                # Backup if flagged
                if len(findings) > 0:
                    import shutil
                    shutil.copy(tarball, EVIDENCE_DIR / f"{pkg_name.replace('/', '_')}.tgz")
                
                # Record in database for caching
                try:
                    db = get_db()
                    # Create a dummy run_id for cache-only scans
                    cache_run_id = "cache-only"
                    pkg_id = db.add_package(
                        run_id=cache_run_id,
                        name=name,
                        version=version,
                        tarball_url="",
                        tarball_sha256=tarball_hash,
                    )
                    db.update_package_scan(
                        package_id=pkg_id,
                        finding_count=result["findings"],
                        scan_duration_ms=0,
                        glassware_output="",
                    )
                except Exception as db_err:
                    # Don't fail scan if DB recording fails
                    pass

            except json.JSONDecodeError:
                result["status"] = "parse_failed"
                result["error"] = "JSON parse failed"
    
    except subprocess.TimeoutExpired:
        result["status"] = "timeout"
        result["error"] = "Scan timeout"
    except Exception as e:
        result["status"] = "error"
        result["error"] = str(e)[:100]
    
    return result


def scan_packages_parallel(packages: list, max_workers: int = 10) -> list:
    """Scan multiple packages in parallel"""
    results = []
    
    print(f"Scanning {len(packages)} packages with {max_workers} workers...")
    print(f"Started: {datetime.utcnow().isoformat()}Z")
    print()
    
    with ThreadPoolExecutor(max_workers=max_workers) as executor:
        future_to_pkg = {
            executor.submit(download_and_scan_package, pkg): pkg 
            for pkg in packages
        }
        
        for i, future in enumerate(as_completed(future_to_pkg), 1):
            pkg = future_to_pkg[future]
            try:
                result = future.result()
                results.append(result)
                
                # Progress update
                if result.get("cached", False):
                    status_icon = "💾"  # Cached
                elif result["findings"] > 0:
                    status_icon = "⚠️"  # Flagged
                elif result["status"] == "scanned":
                    status_icon = "✅"  # Clean
                else:
                    status_icon = "❌"  # Error
                
                cache_note = f" (cached)" if result.get("cached", False) else ""
                print(f"[{i}/{len(packages)}] {pkg} {status_icon}{cache_note} ({result['findings']} findings, {result['critical']} critical)")
                
            except Exception as e:
                results.append({
                    "package": pkg,
                    "status": "error",
                    "error": str(e),
                    "findings": 0,
                    "critical": 0,
                })
                print(f"[{i}/{len(packages)}] {pkg} ❌ ERROR: {e}")
    
    return results


def main():
    import argparse

    parser = argparse.ArgumentParser(description="Optimized parallel MCP package scanner")
    parser.add_argument("packages_file", help="File with one package name per line")
    parser.add_argument("--workers", "-w", type=int, default=10, help="Number of parallel workers")
    parser.add_argument("--output", "-o", default="optimized_scan_results.json", help="Output file")
    parser.add_argument("--evidence-dir", "-e", default=None, help="Evidence directory (default: data/evidence)")
    parser.add_argument("--scan-name", "-n", default=None, help="Scan name prefix for evidence files")

    args = parser.parse_args()

    # Override evidence directory if specified
    if args.evidence_dir:
        global EVIDENCE_DIR
        EVIDENCE_DIR = Path(args.evidence_dir)
        EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)

    # Load package list
    with open(args.packages_file) as f:
        packages = [line.strip() for line in f if line.strip()]

    print("="*60)
    print("MCP OPTIMIZED PARALLEL SCAN")
    print("="*60)
    print(f"Packages: {len(packages)}")
    print(f"Workers: {args.workers}")
    print(f"Evidence dir: {EVIDENCE_DIR}")
    print(f"Started: {datetime.utcnow().isoformat()}Z")
    print()
    
    # Run scan
    results = scan_packages_parallel(packages, max_workers=args.workers)

    # Aggregate results
    scanned = [r for r in results if r["status"] == "scanned"]
    cached = [r for r in results if r.get("cached", False)]
    flagged = [r for r in scanned if r["findings"] > 0]
    errors = [r for r in results if r["status"] not in ["scanned", "cached"]]

    # Save results
    output_data = {
        "scan_date": datetime.utcnow().isoformat() + "Z",
        "total_packages": len(packages),
        "scanned": len(scanned),
        "cached": len(cached),
        "flagged": len(flagged),
        "errors": len(errors),
        "results": results,
        "flagged_packages": flagged,
    }
    
    with open(args.output, "w") as f:
        json.dump(output_data, f, indent=2)
    
    # Print summary
    print()
    print("="*60)
    print("SCAN SUMMARY")
    print("="*60)
    print(f"Total packages: {len(packages)}")
    print(f"Scanned: {len(scanned)}")
    print(f"Cached (skipped): {len(cached)}")
    print(f"Flagged: {len(flagged)}")
    print(f"Errors: {len(errors)}")
    
    if cached:
        cache_hit_rate = len(cached) / len(packages) * 100
        print(f"Cache hit rate: {cache_hit_rate:.1f}%")
    
    print(f"Results saved to: {args.output}")
    print("="*60)
    
    if flagged:
        print()
        print("FLAGGED PACKAGES:")
        for pkg in sorted(flagged, key=lambda x: -x["critical"])[:20]:
            print(f"  - {pkg['package']}: {pkg['findings']} findings ({pkg['critical']} critical)")
        if len(flagged) > 20:
            print(f"  ... and {len(flagged) - 20} more")


if __name__ == "__main__":
    main()
