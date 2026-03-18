#!/usr/bin/env python3
"""MCP Phase 2 Scanner - Batch processing with shallow documentation"""
import subprocess
import json
import tempfile
import tarfile
import os
from pathlib import Path
from datetime import datetime

GLASSWARE = "/home/property.sightlines/samgrowls/glassworks/target/release/glassware"
EVIDENCE_DIR = Path("data/evidence/mcp-phase2")
EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)

def scan_package(pkg_name):
    """Download and scan a single package"""
    try:
        with tempfile.TemporaryDirectory() as tmpdir:
            # Download
            result = subprocess.run(
                ["npm", "pack", pkg_name],
                capture_output=True, text=True, cwd=tmpdir, timeout=60
            )
            if result.returncode != 0:
                return {"error": result.stderr.strip()[:100]}
            
            # Find tarball
            tarballs = list(Path(tmpdir).glob("*.tgz"))
            if not tarballs:
                return {"error": "No tarball downloaded"}
            tarball = tarballs[0]
            
            # Extract
            with tarfile.open(tarball, "r:gz") as tar:
                tar.extractall(tmpdir)
            
            pkg_dir = Path(tmpdir) / "package"
            if not pkg_dir.exists():
                return {"error": "No package directory"}
            
            # Scan
            result = subprocess.run(
                [GLASSWARE, "--format", "json", str(pkg_dir)],
                capture_output=True, text=True, timeout=120
            )
            
            try:
                data = json.loads(result.stdout)
                findings = data.get("findings", [])
                critical = len([f for f in findings if f.get("severity") == "critical"])
                categories = {}
                for f in findings:
                    cat = f.get("category", "unknown")
                    categories[cat] = categories.get(cat, 0) + 1
                
                # Backup if flagged
                if len(findings) > 0:
                    import shutil
                    shutil.copy(tarball, EVIDENCE_DIR / f"{pkg_name.replace('/', '_')}.tgz")
                
                return {
                    "findings": len(findings),
                    "critical": critical,
                    "categories": categories,
                }
            except json.JSONDecodeError:
                return {"error": "JSON parse failed"}
    
    except Exception as e:
        return {"error": str(e)[:100]}

def main():
    # Load package list
    pkg_file = Path("mcp_phase2_batch1.txt")
    packages = [p.strip() for p in pkg_file.read_text().strip().split("\n") if p.strip()]
    
    print(f"=== MCP Phase 2 - Batch 1 ===")
    print(f"Packages: {len(packages)}")
    print(f"Started: {datetime.utcnow().isoformat()}Z")
    print()
    
    results = []
    flagged = []
    
    for i, pkg in enumerate(packages, 1):
        print(f"[{i}/{len(packages)}] {pkg}...", end=" ", flush=True)
        
        result = scan_package(pkg)
        result["package"] = pkg
        
        if "error" in result:
            print(f"ERROR: {result['error'][:50]}")
        else:
            findings = result.get("findings", 0)
            critical = result.get("critical", 0)
            if findings > 0:
                print(f"⚠️ {findings} findings ({critical} critical)")
                flagged.append(result)
            else:
                print(f"✅ Clean")
        
        results.append(result)
    
    # Save results
    results_file = EVIDENCE_DIR / "phase2-batch1-results.json"
    with open(results_file, "w") as f:
        json.dump({
            "scan_date": datetime.utcnow().isoformat() + "Z",
            "batch": 1,
            "total_packages": len(packages),
            "flagged_count": len(flagged),
            "results": results,
            "flagged": flagged,
        }, f, indent=2)
    
    # Summary
    print()
    print("=== SUMMARY ===")
    print(f"Scanned: {len(results)}")
    print(f"Flagged: {len(flagged)}")
    print(f"Errors: {len([r for r in results if 'error' in r])}")
    print(f"Results: {results_file}")
    print(f"Evidence: {EVIDENCE_DIR}")
    
    if flagged:
        print()
        print("=== FLAGGED PACKAGES ===")
        for pkg in flagged:
            print(f"  - {pkg['package']}: {pkg['findings']} findings ({pkg['critical']} critical)")

if __name__ == "__main__":
    main()
