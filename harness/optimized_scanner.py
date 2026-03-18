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

GLASSWARE = "/home/property.sightlines/samgrowls/glassworks/target/release/glassware"
EVIDENCE_DIR = Path("data/evidence/mcp-optimized")
EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)

def download_and_scan_package(pkg_name: str, timeout: int = 60) -> dict:
    """Download and scan a single package (optimized)"""
    result = {
        "package": pkg_name,
        "status": "pending",
        "findings": 0,
        "critical": 0,
        "error": None,
    }
    
    try:
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
                status_icon = "⚠️" if result["findings"] > 0 else "✅" if result["status"] == "scanned" else "❌"
                print(f"[{i}/{len(packages)}] {pkg} {status_icon} ({result['findings']} findings, {result['critical']} critical)")
                
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
    
    args = parser.parse_args()
    
    # Load package list
    with open(args.packages_file) as f:
        packages = [line.strip() for line in f if line.strip()]
    
    print("="*60)
    print("MCP OPTIMIZED PARALLEL SCAN")
    print("="*60)
    
    # Run scan
    results = scan_packages_parallel(packages, max_workers=args.workers)
    
    # Aggregate results
    scanned = [r for r in results if r["status"] == "scanned"]
    flagged = [r for r in scanned if r["findings"] > 0]
    errors = [r for r in results if r["status"] not in ["scanned"]]
    
    # Save results
    output_data = {
        "scan_date": datetime.utcnow().isoformat() + "Z",
        "total_packages": len(packages),
        "scanned": len(scanned),
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
    print(f"Flagged: {len(flagged)}")
    print(f"Errors: {len(errors)}")
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
