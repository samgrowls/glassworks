#!/usr/bin/env python3
"""
Batch LLM Analyzer - Process multiple flagged packages efficiently

Usage:
    python batch_llm_analyzer.py flagged_packages.txt --output results.json
"""

import json
import subprocess
import tempfile
import tarfile
from pathlib import Path
from datetime import datetime
from concurrent.futures import ThreadPoolExecutor, as_completed
import os

GLASSWARE = "/home/property.sightlines/samgrowls/glassworks/harness/glassware-scanner"
LLM_ANALYZER = "/home/property.sightlines/samgrowls/glassworks/llm-analyzer/analyzer.py"
NVIDIA_API_KEY = os.environ.get("NVIDIA_API_KEY", "")
EVIDENCE_DIR = Path(os.environ.get("GLASSWARE_LLM_EVIDENCE_DIR", "data/evidence/batch-llm"))
EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)

# LLM cache (SQLite)
DB_PATH = Path("data/corpus.db")
db_instance = None

def get_db():
    """Get or create database instance"""
    global db_instance
    if db_instance is None:
        from database import Database
        db_instance = Database(DB_PATH)
    return db_instance


def download_and_analyze_package(pkg_name: str) -> dict:
    """Download, scan, and LLM analyze a single package"""
    result = {
        "package": pkg_name,
        "status": "pending",
        "scan_findings": 0,
        "llm_classification": None,
        "llm_confidence_tier": None,
        "llm_recommendation": None,
        "error": None,
    }
    
    try:
        with tempfile.TemporaryDirectory() as tmpdir:
            # Download
            dl_result = subprocess.run(
                ["npm", "pack", pkg_name],
                capture_output=True, text=True, cwd=tmpdir, timeout=30
            )
            if dl_result.returncode != 0:
                result["status"] = "download_failed"
                result["error"] = dl_result.stderr[:100]
                return result
            
            # Extract
            tarballs = list(Path(tmpdir).glob("*.tgz"))
            if not tarballs:
                result["status"] = "no_tarball"
                return result
            tarball = tarballs[0]
            
            with tarfile.open(tarball, "r:gz") as tar:
                tar.extractall(tmpdir, filter="data")
            
            pkg_dir = Path(tmpdir) / "package"
            if not pkg_dir.exists():
                result["status"] = "no_package_dir"
                return result
            
            # Scan with glassware
            scan_result = subprocess.run(
                [GLASSWARE, "--format", "json", str(pkg_dir)],
                capture_output=True, text=True, timeout=60
            )
            
            try:
                scan_data = json.loads(scan_result.stdout)
                findings = scan_data.get("findings", [])
                result["scan_findings"] = len(findings)
                
                if len(findings) == 0:
                    result["status"] = "clean"
                    return result
                
                # Save scan result for LLM
                scan_json_path = Path(tmpdir) / "scan.json"
                with open(scan_json_path, "w") as f:
                    json.dump(scan_data, f)
                
                # Run LLM analyzer
                llm_output_path = Path(tmpdir) / "llm_analysis.json"
                llm_result = subprocess.run(
                    [
                        "python3", LLM_ANALYZER,
                        str(scan_json_path),
                        str(pkg_dir),
                        "--output", str(llm_output_path),
                    ],
                    capture_output=True, text=True, timeout=300,
                    env={**os.environ, "NVIDIA_API_KEY": NVIDIA_API_KEY}
                )
                
                if llm_result.returncode == 0 and llm_output_path.exists():
                    with open(llm_output_path) as f:
                        llm_data = json.load(f)
                    
                    result["llm_classification"] = llm_data.get("overall_classification")
                    result["llm_confidence_tier"] = llm_data.get("confidence_tier")
                    result["llm_recommendation"] = llm_data.get("recommendation")
                    result["status"] = "analyzed"
                    
                    # Save full LLM analysis to evidence
                    evidence_path = EVIDENCE_DIR / f"{pkg_name.replace('/', '_')}_llm.json"
                    with open(evidence_path, "w") as f:
                        json.dump(llm_data, f, indent=2)
                
                else:
                    result["status"] = "llm_failed"
                    result["error"] = llm_result.stderr[:200]
                
            except json.JSONDecodeError:
                result["status"] = "parse_failed"
                result["error"] = "JSON parse failed"
    
    except subprocess.TimeoutExpired:
        result["status"] = "timeout"
        result["error"] = "Operation timeout"
    except Exception as e:
        result["status"] = "error"
        result["error"] = str(e)[:200]
    
    return result


def main():
    import argparse
    
    parser = argparse.ArgumentParser(description="Batch LLM analyzer for flagged packages")
    parser.add_argument("packages_file", help="File with one package name per line")
    parser.add_argument("--workers", "-w", type=int, default=3, help="Parallel LLM workers (API rate limit)")
    parser.add_argument("--output", "-o", default="batch_llm_results.json", help="Output file")
    parser.add_argument("--evidence-dir", "-e", default=None, help="LLM evidence directory (default: data/evidence/batch-llm)")

    args = parser.parse_args()

    # Override evidence directory if specified
    if args.evidence_dir:
        global EVIDENCE_DIR
        EVIDENCE_DIR = Path(args.evidence_dir)
        EVIDENCE_DIR.mkdir(parents=True, exist_ok=True)

    # Load package list
    with open(args.packages_file) as f:
        packages = [line.strip() for line in f if line.strip()]

    print("="*70)
    print("BATCH LLM ANALYZER")
    print("="*70)
    print(f"Packages: {len(packages)}")
    print(f"Workers: {args.workers}")
    print(f"Evidence dir: {EVIDENCE_DIR}")
    print(f"Started: {datetime.utcnow().isoformat()}Z")
    print()
    
    results = []
    
    with ThreadPoolExecutor(max_workers=args.workers) as executor:
        future_to_pkg = {
            executor.submit(download_and_analyze_package, pkg): pkg
            for pkg in packages
        }
        
        for i, future in enumerate(as_completed(future_to_pkg), 1):
            pkg = future_to_pkg[future]
            try:
                result = future.result()
                results.append(result)
                
                status_icon = "✅" if result["status"] == "analyzed" else "⚠️" if result["status"] == "clean" else "❌"
                print(f"[{i}/{len(packages)}] {pkg} {status_icon}")
                if result["llm_classification"]:
                    print(f"      LLM: {result['llm_classification']} ({result['llm_confidence_tier']})")
                    print(f"      Recommendation: {result['llm_recommendation']}")
                
            except Exception as e:
                results.append({
                    "package": pkg,
                    "status": "error",
                    "error": str(e),
                })
                print(f"[{i}/{len(packages)}] {pkg} ❌ ERROR: {e}")
    
    # Aggregate results
    analyzed = [r for r in results if r["status"] == "analyzed"]
    clean = [r for r in results if r["status"] == "clean"]
    errors = [r for r in results if r["status"] not in ["analyzed", "clean"]]
    
    malicious = [r for r in analyzed if r["llm_classification"] == "MALICIOUS"]
    suspicious = [r for r in analyzed if r["llm_classification"] == "SUSPICIOUS"]
    fp = [r for r in analyzed if r["llm_classification"] == "FALSE_POSITIVE"]
    
    # Save results
    output_data = {
        "analysis_date": datetime.utcnow().isoformat() + "Z",
        "total_packages": len(packages),
        "analyzed": len(analyzed),
        "clean": len(clean),
        "errors": len(errors),
        "malicious": len(malicious),
        "suspicious": len(suspicious),
        "false_positive": len(fp),
        "results": results,
        "malicious_packages": malicious,
        "suspicious_packages": suspicious,
    }
    
    with open(args.output, "w") as f:
        json.dump(output_data, f, indent=2)
    
    # Print summary
    print()
    print("="*70)
    print("BATCH ANALYSIS SUMMARY")
    print("="*70)
    print(f"Total packages: {len(packages)}")
    print(f"Analyzed: {len(analyzed)}")
    print(f"Clean: {len(clean)}")
    print(f"Errors: {len(errors)}")
    print()
    print("LLM Classifications:")
    print(f"  - Malicious: {len(malicious)}")
    print(f"  - Suspicious: {len(suspicious)}")
    print(f"  - False Positive: {len(fp)}")
    print()
    print(f"Results saved to: {args.output}")
    print(f"Evidence saved to: {EVIDENCE_DIR}/")
    print("="*70)
    
    if malicious:
        print()
        print("MALICIOUS PACKAGES (Report Immediately):")
        for pkg in malicious:
            print(f"  - {pkg['package']}: {pkg['llm_classification']} ({pkg['llm_confidence_tier']})")
            print(f"    Recommendation: {pkg['llm_recommendation']}")
    
    if suspicious:
        print()
        print("SUSPICIOUS PACKAGES (Needs Human Review):")
        for pkg in suspicious[:10]:
            print(f"  - {pkg['package']}: {pkg['llm_classification']} ({pkg['llm_confidence_tier']})")
        if len(suspicious) > 10:
            print(f"  ... and {len(suspicious) - 10} more")


if __name__ == "__main__":
    main()
