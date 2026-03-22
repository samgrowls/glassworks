#!/usr/bin/env python3
"""
Wave 2 Scanner — 50 Package Real-World Hunt

Scans 50 diverse packages to hunt for real malicious packages:
- 4 known malicious (from evidence, must detect)
- 20 high-download legitimate (FP baseline)
- 16 diverse categories (risk-based sampling)
- 10 recent publishes (last 3 months)

Uses Cerebras for fast triage, NVIDIA for deep analysis of flagged packages.
"""

import subprocess
import json
import tempfile
import tarfile
import os
from pathlib import Path
from datetime import datetime
from concurrent.futures import ThreadPoolExecutor, as_completed
import time
import os

# Load environment variables from ~/.env
env_file = Path.home() / '.env'
if env_file.exists():
    with open(env_file) as f:
        for line in f:
            line = line.strip()
            if line and not line.startswith('#') and '=' in line:
                key, value = line.split('=', 1)
                os.environ[key.strip()] = value.strip().strip('"')

# Glassware CLI path - use absolute path
# wave2_scanner.py is in harness/, target/ is at repo root
GLASSWARE = (Path(__file__).resolve().parent.parent / "target" / "debug" / "glassware-orchestrator").resolve()

# Output directory
OUTPUT_DIR = Path(__file__).parent / "data" / "wave2-results"
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

# Evidence directory (malicious packages) - use absolute path
EVIDENCE_DIR = Path(__file__).parent.parent.parent / "glassworks-archive" / "evidence"

# Known malicious packages (MUST DETECT)
KNOWN_MALICIOUS = [
    str((EVIDENCE_DIR / "react-native-country-select-0.3.91.tgz").resolve()),
    str((EVIDENCE_DIR / "react-native-international-phone-number-0.11.8.tgz").resolve()),
    str((EVIDENCE_DIR / "aifabrix-miso-client-4.7.2.tgz").resolve()),
    str((EVIDENCE_DIR / "iflow-mcp-watercrawl-watercrawl-mcp-1.3.4.tgz").resolve()),
]

# High-download legitimate packages (FP baseline)
HIGH_DOWNLOAD_CLEAN = [
    "express@4.19.2",
    "lodash@4.17.21",
    "axios@1.6.7",
    "chalk@5.3.0",
    "debug@4.3.4",
    "moment@2.30.1",
    "uuid@9.0.1",
    "async@3.2.5",
    "commander@12.0.0",
    "glob@10.3.10",
    "mkdirp@3.0.1",
    "semver@7.6.0",
    "ws@8.16.0",
    "yargs@17.7.2",
    "dotenv@16.4.5",
    "prettier@3.2.5",
    "typescript@5.4.2",
    "jest@29.7.0",
    "react@18.3.1",
    "vue@3.4.21",
]

# Diverse categories (risk-based sampling)
DIVERSE_CATEGORIES = {
    # High-risk: native code, install scripts
    "native-build": ["node-gyp@10.1.0", "bindings@1.5.0", "prebuild@11.0.0"],
    "install-scripts": ["core-js@3.36.0", "esbuild@0.20.0"],
    
    # Crypto/Security (legitimate crypto usage)
    "crypto": ["ethers@6.11.1", "web3@4.6.0", "bcrypt@5.1.1", "jsonwebtoken@9.0.2"],
    
    # AI/ML (currently targeted by attackers)
    "ai-ml": ["langchain@0.1.0", "openai@4.28.0", "anthropic@0.18.0"],
    
    # Developer tools (high trust, but targeted)
    "devtools": ["eslint@8.57.0", "webpack@5.90.0", "vite@5.1.0", "babel-core@7.24.0"],
    
    # Utilities (widely used)
    "utils": ["dayjs@1.11.10", "axios@1.6.7", "got@14.2.0"],
}

def scan_package(pkg_spec: str, use_llm: bool = False) -> dict:
    """Download and scan a single package."""
    result = {
        "package": pkg_spec,
        "status": "pending",
        "findings": 0,
        "threat_score": 0.0,
        "is_malicious": False,
        "llm_analyzed": False,
        "llm_verdict": None,
        "error": None,
        "details": [],
        "scan_duration_ms": 0,
    }

    start_time = time.time()

    try:
        # Check if this is a local tarball path (evidence file)
        if pkg_spec.startswith("/") and pkg_spec.endswith(".tgz"):
            tarball = Path(pkg_spec)
            if not tarball.exists():
                result["status"] = "file_not_found"
                result["error"] = f"Evidence file not found: {pkg_spec}"
                return result
            
            # Scan tarball with JSON output to temp file
            import uuid
            output_file = f"/tmp/glassware-scan-{uuid.uuid4().hex}.json"
            
            cmd = [
                str(GLASSWARE),
                "scan-tarball",
                str(tarball),
                "--output", output_file,
                "--format", "json",
            ]
            
            # LLM not yet supported for scan-tarball
            # if use_llm:
            #     cmd.append("--llm")
            #     result["llm_analyzed"] = True
        else:
            # Download from npm and scan
            with tempfile.TemporaryDirectory() as tmpdir:
                # Download package
                dl_result = subprocess.run(
                    ["npm", "pack", pkg_spec],
                    capture_output=True, text=True, cwd=tmpdir, timeout=60
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

                # Scan tarball with JSON output to temp file
                import uuid
                output_file = f"/tmp/glassware-scan-{uuid.uuid4().hex}.json"
                
                cmd = [
                    str(GLASSWARE),
                    "scan-tarball",
                    str(tarball),
                    "--output", output_file,
                    "--format", "json",
                ]
                
                # LLM not yet supported for scan-tarball
                # if use_llm:
                #     cmd.append("--llm")
                #     result["llm_analyzed"] = True

        # Run scan (exit code 1 is expected for malicious packages)
        scan_result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=120,
            cwd=str(Path.home()),  # Run from home directory to avoid path issues
            env=os.environ  # Pass full environment including loaded .env vars
        )
        
        result["scan_duration_ms"] = int((time.time() - start_time) * 1000)

        # Read JSON output from file
        try:
            if not os.path.exists(output_file):
                result["status"] = "parse_error"
                result["error"] = f"Output file not created: {output_file}"
                return result
                
            with open(output_file, 'r') as f:
                content = f.read().strip()
                if not content:
                    result["status"] = "parse_error"
                    result["error"] = "Output file is empty"
                    # Clean up temp file
                    try:
                        os.unlink(output_file)
                    except:
                        pass
                    return result
                output = json.loads(content)
            
            # Clean up temp file
            try:
                os.unlink(output_file)
            except:
                pass
            
            summary = output.get("summary", {})
            result["findings"] = summary.get("total_findings", 0)
            result["threat_score"] = summary.get("average_threat_score", 0.0)
            result["is_malicious"] = summary.get("malicious_packages", 0) > 0
            
            # Get package results
            pkg_results = output.get("results", [])
            if pkg_results:
                result["details"] = pkg_results[0].get("findings", [])
                result["llm_verdict"] = pkg_results[0].get("llm_verdict")
            
            result["status"] = "scanned"
        except json.JSONDecodeError as e:
            result["status"] = "parse_error"
            result["error"] = f"Failed to parse JSON output: {e}"
            # Clean up temp file
            try:
                os.unlink(output_file)
            except:
                pass
        except Exception as e:
            result["status"] = "error"
            result["error"] = str(e)[:200]
            # Clean up temp file
            try:
                os.unlink(output_file)
            except:
                pass

    except subprocess.TimeoutExpired:
        result["status"] = "timeout"
        result["error"] = "Scan timed out"
    except Exception as e:
        result["status"] = "error"
        result["error"] = str(e)[:200]

    return result

def run_wave2():
    """Run Wave 2 scan with 50 packages."""
    print("=" * 70)
    print("Wave 2 Scanner — 50 Package Real-World Hunt")
    print("=" * 70)
    print()
    
    # Build package list
    all_packages = []
    
    # Add known malicious (4)
    print("Adding known malicious packages (4)...")
    for pkg in KNOWN_MALICIOUS:
        if Path(pkg).exists():
            all_packages.append(("malicious", pkg))
        else:
            print(f"  ⚠ Evidence file not found: {pkg}")
    
    # Add high-download clean (20)
    print("Adding high-download clean packages (20)...")
    for pkg in HIGH_DOWNLOAD_CLEAN:
        all_packages.append(("clean", pkg))
    
    # Add diverse categories (16)
    print("Adding diverse category packages (16)...")
    for category, packages in DIVERSE_CATEGORIES.items():
        for pkg in packages[:4]:  # Limit to 4 per category
            all_packages.append((category, pkg))
    
    # Add recent publishes (10) - sample dynamically
    print("Adding recent publishes (10)...")
    # For now, use some recent popular packages
    recent = [
        "next@14.1.0",
        "nuxt@3.10.0",
        "svelte@4.2.0",
        "vite@5.1.0",
        "prisma@5.10.0",
        "tailwindcss@3.4.0",
        "zod@3.22.0",
        "valtio@1.13.0",
        "zustand@4.5.0",
        "immer@10.0.0",
    ]
    for pkg in recent:
        all_packages.append(("recent", pkg))
    
    print()
    print(f"Total packages to scan: {len(all_packages)}")
    print()
    
    # Scan packages
    results = []
    malicious_count = 0
    clean_count = 0
    errors = 0
    
    print("Starting scans (sequential mode for debugging)...")
    print()
    
    # Run scans sequentially for now (ThreadPoolExecutor has issues with file output)
    for i, (category, pkg) in enumerate(all_packages):
        # Use LLM for suspicious packages only (to save API calls)
        use_llm = category in ["malicious", "native-build", "crypto"]
        result = scan_package(pkg, use_llm)
        result["category"] = category
        results.append(result)
        
        # Print progress
        status_icon = "✓" if result["status"] == "scanned" else "✗"
        if result["is_malicious"]:
            status_icon = "⚠"
            malicious_count += 1
        
        print(f"[{i+1}/{len(all_packages)}] {status_icon} {pkg}")
        if result["threat_score"] > 0:
            print(f"    Threat score: {result['threat_score']:.2f}")
        if result["llm_verdict"]:
            print(f"    LLM: {result['llm_verdict'].get('malicious', 'N/A')}")
        if result["error"]:
            print(f"    Error: {result['error']}")
            errors += 1
    
    # Save results
    timestamp = datetime.now().strftime("%Y%m%d-%H%M%S")
    output_file = OUTPUT_DIR / f"wave2-results-{timestamp}.json"
    
    report = {
        "scan_date": timestamp,
        "total_packages": len(all_packages),
        "malicious_detected": malicious_count,
        "errors": errors,
        "results": results,
    }
    
    with open(output_file, "w") as f:
        json.dump(report, f, indent=2)
    
    print()
    print("=" * 70)
    print("WAVE 2 SUMMARY")
    print("=" * 70)
    print(f"Total packages scanned: {len(results)}")
    print(f"Malicious detected: {malicious_count}")
    print(f"Errors: {errors}")
    print(f"Results saved to: {output_file}")
    print("=" * 70)
    
    # Print malicious packages
    if malicious_count > 0:
        print()
        print("⚠ MALICIOUS PACKAGES DETECTED:")
        for result in results:
            if result["is_malicious"]:
                print(f"  - {result['package']} (score: {result['threat_score']:.2f})")
    
    return report

if __name__ == "__main__":
    run_wave2()
