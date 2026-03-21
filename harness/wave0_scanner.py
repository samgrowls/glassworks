#!/usr/bin/env python3
"""
Wave 0 Validation Scanner — Calibration Run

Scans 50 packages to validate the scanning pipeline:
- 2-5 known malicious (must detect)
- 20 top clean packages (FP baseline)
- 15 React Native ecosystem (proximity hunting)
- 10 random recent publishes (baseline noise)
"""

import subprocess
import json
import tempfile
import tarfile
import os
from pathlib import Path
from datetime import datetime
from concurrent.futures import ThreadPoolExecutor, as_completed

# Glassware CLI path
GLASSWARE = Path(__file__).parent.parent / "target" / "debug" / "glassware"

# Output directory
OUTPUT_DIR = Path(__file__).parent / "data" / "wave0-results"
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

# Known malicious packages (MUST DETECT) - from evidence directory
# These are confirmed GlassWorm-infected packages with install.js backdoors
KNOWN_MALICIOUS = [
    "/home/shva/samgrowls/glassworks/harness/data/evidence/react-native-country-select-0.3.91.tgz",
    "/home/shva/samgrowls/glassworks/harness/data/evidence/react-native-international-phone-number-0.11.8.tgz",
]

# Top clean packages (FP baseline)
TOP_CLEAN = [
    "express@4.19.2",
    "lodash@4.17.21",
    "axios@1.6.7",
    "chalk@5.3.0",
    "debug@4.3.4",
    "moment@2.30.1",
    "uuid@9.0.1",
    "async@3.2.5",
    "request@2.88.2",
    "commander@12.0.0",
    "glob@10.3.10",
    "mkdirp@3.0.1",
    "semver@7.6.0",
    "ws@8.16.0",
    "yargs@17.7.2",
    "dotenv@16.4.5",
    "eslint@8.57.0",
    "prettier@3.2.5",
    "typescript@5.4.2",
    "jest@29.7.0",
]

# React Native ecosystem (proximity hunting)
REACT_NATIVE_ECOSYSTEM = [
    "react-native-phone-input@1.3.8",
    "react-native-phone-number-input@2.1.0",
    "react-native-otp-inputs@1.2.0",
    "react-native-sms-retriever@1.0.2",
    "react-native-geolocation@3.2.1",
    "react-native-locale@1.0.0",
    "react-native-localize@3.0.6",
    "react-native-i18n@2.0.15",
    "react-native-intl@1.0.0",
    "react-native-country-picker@2.0.0",
    "react-native-flags@1.0.0",
    "react-native-phone-verify@1.0.0",
    "react-native-confirmation-code-input@1.0.0",
    "react-native-pin-code-input@1.0.0",
    "react-native-code-input@1.0.0",
]

# Random recent publishes (baseline noise)
RECENT_PUBLISHES = [
    # These will be sampled dynamically
]

def scan_package(pkg_spec: str) -> dict:
    """Download and scan a single package."""
    result = {
        "package": pkg_spec,
        "status": "pending",
        "findings": 0,
        "critical": 0,
        "high": 0,
        "medium": 0,
        "error": None,
        "details": [],
    }
    
    try:
        with tempfile.TemporaryDirectory() as tmpdir:
            # Check if this is a local tarball path
            if pkg_spec.startswith("/") and pkg_spec.endswith(".tgz"):
                tarball = Path(pkg_spec)
                if not tarball.exists():
                    result["status"] = "file_not_found"
                    result["error"] = f"Local tarball not found: {pkg_spec}"
                    return result
            else:
                # Download from npm
                dl_result = subprocess.run(
                    ["npm", "pack", pkg_spec],
                    capture_output=True, text=True, cwd=tmpdir, timeout=30
                )
                if dl_result.returncode != 0:
                    result["status"] = "download_failed"
                    result["error"] = dl_result.stderr[:200]
                    return result
                
                # Find tarball
                tarballs = list(Path(tmpdir).glob("*.tgz"))
                if not tarballs:
                    result["status"] = "no_tarball"
                    return result
                tarball = tarballs[0]
            
            # Extract
            extract_dir = Path(tmpdir) / "package"
            extract_dir.mkdir()
            with tarfile.open(tarball, "r:gz") as tf:
                tf.extractall(extract_dir, filter="data")
            
            # Scan with glassware
            scan_result = subprocess.run(
                [str(GLASSWARE), "--format", "json", "--severity", "info", str(extract_dir)],
                capture_output=True, text=True, timeout=60
            )
            
            if scan_result.returncode == 0:
                result["status"] = "clean"
            else:
                result["status"] = "flagged"
                try:
                    findings = json.loads(scan_result.stdout)
                    result["findings"] = len(findings.get("findings", []))
                    result["details"] = findings.get("findings", [])
                    
                    for f in result["details"]:
                        sev = f.get("severity", "").lower()
                        if sev == "critical":
                            result["critical"] += 1
                        elif sev == "high":
                            result["high"] += 1
                        elif sev == "medium":
                            result["medium"] += 1
                except:
                    result["error"] = "Failed to parse results"
            
            return result
            
    except subprocess.TimeoutExpired:
        result["status"] = "timeout"
        result["error"] = "Scan timed out"
    except Exception as e:
        result["status"] = "error"
        result["error"] = str(e)[:200]
    
    return result


def main():
    print("=" * 60)
    print("Wave 0 Validation Scanner — Calibration Run")
    print("=" * 60)
    print()
    
    all_results = []
    
    # Scan known malicious (MUST DETECT)
    print("🎯 Scanning KNOWN MALICIOUS packages...")
    for pkg in KNOWN_MALICIOUS:
        print(f"  → {pkg}")
        result = scan_package(pkg)
        all_results.append(result)
        status = "✅ FLAGGED" if result["status"] == "flagged" else "❌ MISSED"
        print(f"    {status}: {result['findings']} findings ({result['critical']} critical)")
    
    print()
    
    # Scan top clean packages (FP baseline)
    print("🧹 Scanning TOP CLEAN packages (FP baseline)...")
    fp_count = 0
    fp_medium_plus = 0  # MEDIUM+ severity false positives
    for pkg in TOP_CLEAN:
        print(f"  → {pkg}")
        result = scan_package(pkg)
        all_results.append(result)
        # Count MEDIUM+ severity findings as actual false positives
        medium_plus = result['medium'] + result['high'] + result['critical']
        if medium_plus > 0:
            fp_medium_plus += 1
            print(f"    ⚠️  FP (MEDIUM+): {medium_plus} findings")
        elif result["findings"] > 0:
            fp_count += 1
            print(f"    ℹ️  INFO: {result['findings']} low-severity findings")
        else:
            print(f"    ✅ Clean")
    
    print()
    print(f"False positives (MEDIUM+): {fp_medium_plus}/{len(TOP_CLEAN)}")
    print(f"False positives (including LOW): {(fp_count + fp_medium_plus)}/{len(TOP_CLEAN)}")
    print()
    
    # Scan React Native ecosystem
    print("📱 Scanning REACT NATIVE ECOSYSTEM (proximity hunting)...")
    rn_flagged = 0
    for pkg in REACT_NATIVE_ECOSYSTEM:
        print(f"  → {pkg}")
        result = scan_package(pkg)
        all_results.append(result)
        if result["status"] == "flagged":
            rn_flagged += 1
            print(f"    ⚠️  FLAGGED: {result['findings']} findings")
        else:
            print(f"    ✅ Clean")
    
    print()
    print(f"React Native flagged: {rn_flagged}/{len(REACT_NATIVE_ECOSYSTEM)}")
    print()
    
    # Summary
    print("=" * 60)
    print("SUMMARY")
    print("=" * 60)
    
    total = len(all_results)
    flagged = sum(1 for r in all_results if r["status"] == "flagged")
    clean = sum(1 for r in all_results if r["status"] == "clean")
    errors = sum(1 for r in all_results if r["error"])
    
    print(f"Total scanned: {total}")
    print(f"Flagged: {flagged}")
    print(f"Clean: {clean}")
    print(f"Errors: {errors}")
    print()
    
    # Known malicious detection rate
    malicious_results = [r for r in all_results if r["package"] in KNOWN_MALICIOUS]
    malicious_detected = sum(1 for r in malicious_results if r["status"] == "flagged")
    print(f"Known malicious detection: {malicious_detected}/{len(malicious_results)}")
    
    if malicious_detected < len(malicious_results):
        print("  ⚠️  WARNING: Not all known malicious packages were detected!")
        for r in malicious_results:
            if r["status"] != "flagged":
                print(f"    MISSED: {r['package']}")
    else:
        print("  ✅ All known malicious packages detected!")
    
    # Save results
    output_file = OUTPUT_DIR / f"wave0-results-{datetime.now().strftime('%Y%m%d-%H%M%S')}.json"
    with open(output_file, "w") as f:
        json.dump({
            "scan_date": datetime.now().isoformat(),
            "total": total,
            "flagged": flagged,
            "clean": clean,
            "errors": errors,
            "known_malicious_detection": f"{malicious_detected}/{len(malicious_results)}",
            "false_positives": f"{fp_count}/{len(TOP_CLEAN)}",
            "results": all_results,
        }, f, indent=2)
    
    print()
    print(f"Results saved to: {output_file}")


if __name__ == "__main__":
    main()
