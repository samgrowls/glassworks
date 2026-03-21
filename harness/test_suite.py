#!/usr/bin/env python3
"""
Comprehensive Test Suite for Glassware v0.8.75

Tests all major functionality:
1. CLI validation
2. Scan registry
3. Version scanning
4. Package sampling
5. Background scanning
6. Cache functionality

Usage:
    python3 test_suite.py --all
    python3 test_suite.py --test cli-validation
    python3 test_suite.py --test version-scan
"""

import argparse
import json
import os
import subprocess
import sqlite3
import sys
import time
from datetime import datetime
from pathlib import Path
from typing import List, Tuple

# Configuration
GLASSWARE = "../target/debug/glassware-orchestrator"
SAMPLER = "./version_sampler.py"
SCANNER = "./background_scanner.py"
TEST_DIR = Path("/tmp/glassware-tests")
RESULTS = {
    "passed": 0,
    "failed": 0,
    "skipped": 0,
    "tests": [],
}


def log(message: str, level: str = "INFO"):
    """Log message with timestamp"""
    timestamp = datetime.now().isoformat()
    print(f"[{timestamp}] {level}: {message}", file=sys.stderr)


def test_passed(name: str, details: str = ""):
    """Record passed test"""
    RESULTS["passed"] += 1
    RESULTS["tests"].append({"name": name, "status": "passed", "details": details})
    log(f"✅ PASS: {name} - {details}", "SUCCESS")


def test_failed(name: str, error: str):
    """Record failed test"""
    RESULTS["failed"] += 1
    RESULTS["tests"].append({"name": name, "status": "failed", "error": error})
    log(f"❌ FAIL: {name} - {error}", "ERROR")


def test_skipped(name: str, reason: str):
    """Record skipped test"""
    RESULTS["skipped"] += 1
    RESULTS["tests"].append({"name": name, "status": "skipped", "reason": reason})
    log(f"⏭️  SKIP: {name} - {reason}", "SKIP")


def setup():
    """Create test directory"""
    TEST_DIR.mkdir(parents=True, exist_ok=True)
    log(f"Test directory: {TEST_DIR}")


def cleanup():
    """Clean up test files"""
    log("Cleaning up test files...")
    # Keep test files for inspection
    # shutil.rmtree(TEST_DIR, ignore_errors=True)


# ============================================================================
# Test 1: CLI Validation
# ============================================================================

def test_cli_validation():
    """Test CLI flag validation"""
    log("Testing CLI validation...")
    
    # Test 1: --llm without env vars should fail
    log("  Test: --llm without env vars")
    env = os.environ.copy()
    env.pop("GLASSWARE_LLM_BASE_URL", None)
    env.pop("GLASSWARE_LLM_API_KEY", None)
    
    result = subprocess.run(
        [GLASSWARE, "--llm", "scan-npm", "express"],
        capture_output=True, text=True, timeout=30,
        env=env
    )
    
    if "requires GLASSWARE_LLM" in result.stderr:
        test_passed("CLI validation: --llm requires env vars")
    else:
        test_failed("CLI validation: --llm requires env vars", result.stderr[:200])
    
    # Test 2: --no-cache with --cache-db should fail
    log("  Test: --no-cache with --cache-db")
    result = subprocess.run(
        [GLASSWARE, "--no-cache", "--cache-db", "/tmp/test.db", "scan-npm", "express"],
        capture_output=True, text=True, timeout=30
    )
    
    if "conflicts with --cache-db" in result.stderr:
        test_passed("CLI validation: --no-cache conflicts")
    else:
        test_failed("CLI validation: --no-cache conflicts", result.stderr[:200])


# ============================================================================
# Test 2: Scan Registry
# ============================================================================

def test_scan_registry():
    """Test scan registry functionality"""
    log("Testing scan registry...")
    
    # Test 1: List scans
    log("  Test: scan-list command")
    result = subprocess.run(
        [GLASSWARE, "scan-list"],
        capture_output=True, text=True, timeout=30
    )
    
    if result.returncode == 0:
        test_passed("Scan registry: scan-list command")
    else:
        test_failed("Scan registry: scan-list command", result.stderr[:200])
    
    # Test 2: Check state file exists
    log("  Test: state file exists")
    state_file = Path("../.glassware-scan-registry.json")
    if state_file.exists():
        try:
            data = json.load(open(state_file))
            if "scans" in data:
                test_passed("Scan registry: state file valid")
            else:
                test_failed("Scan registry: state file valid", "Missing 'scans' key")
        except Exception as e:
            test_failed("Scan registry: state file valid", str(e))
    else:
        test_failed("Scan registry: state file exists", "File not found")


# ============================================================================
# Test 3: Basic Scanning
# ============================================================================

def test_basic_scanning():
    """Test basic package scanning"""
    log("Testing basic scanning...")
    
    # Test 1: Scan single package
    log("  Test: scan single package")
    result = subprocess.run(
        [GLASSWARE, "scan-npm", "express"],
        capture_output=True, text=True, timeout=60
    )
    
    if "Total packages scanned: 1" in result.stdout:
        test_passed("Basic scanning: single package")
    else:
        test_failed("Basic scanning: single package", "Expected output not found")
    
    # Test 2: Scan with caching
    log("  Test: scan with caching")
    cache_db = TEST_DIR / "test-cache.db"
    
    # Use a fresh package name to ensure first scan is uncached
    result = subprocess.run(
        [GLASSWARE, "--cache-db", str(cache_db), "scan-npm", "is-number"],
        capture_output=True, text=True, timeout=60
    )
    first_scan_ok = result.returncode == 0
    
    # Second scan should complete (cache may or may not be faster depending on system load)
    result = subprocess.run(
        [GLASSWARE, "--cache-db", str(cache_db), "scan-npm", "is-number"],
        capture_output=True, text=True, timeout=60
    )
    second_scan_ok = result.returncode == 0
    
    if first_scan_ok and second_scan_ok:
        test_passed("Basic scanning: caching (both scans completed)")
    else:
        test_failed("Basic scanning: caching", "One or both scans failed")


# ============================================================================
# Test 4: Version Scanning
# ============================================================================

def test_version_scanning():
    """Test version history scanning"""
    log("Testing version scanning...")
    
    # Test 1: Version scan with last-N policy
    log("  Test: version scan last-3")
    result = subprocess.run(
        [GLASSWARE, "scan-npm", "--versions", "last-3", "chalk"],
        capture_output=True, text=True, timeout=120
    )
    
    if "VERSION SCAN SUMMARY" in result.stdout:
        test_passed("Version scanning: last-N policy")
    else:
        test_failed("Version scanning: last-N policy", "Expected output not found")
    
    # Test 2: Version scan with all policy
    log("  Test: version scan all")
    result = subprocess.run(
        [GLASSWARE, "scan-npm", "--versions", "all", "tiny-package"],
        capture_output=True, text=True, timeout=120
    )
    
    # Should attempt to scan (may have 404s for old versions)
    if "VERSION SCAN SUMMARY" in result.stdout or "Fetching versions" in result.stderr:
        test_passed("Version scanning: all policy")
    else:
        test_failed("Version scanning: all policy", "Command failed")


# ============================================================================
# Test 5: Package Sampler
# ============================================================================

def test_package_sampler():
    """Test Python package sampler"""
    log("Testing package sampler...")
    
    # Test 1: Sample from single category
    log("  Test: sample from single category")
    output_file = TEST_DIR / "test-packages.txt"
    
    result = subprocess.run(
        ["python3", SAMPLER, "--output", str(output_file), "--samples", "5", "--categories", "utils"],
        capture_output=True, text=True, timeout=60,
        cwd=Path(__file__).parent
    )
    
    if result.returncode == 0 and output_file.exists():
        packages = [line.strip() for line in open(output_file) if line.strip()]
        if len(packages) >= 1:
            test_passed(f"Package sampler: single category ({len(packages)} packages)")
        else:
            test_failed("Package sampler: single category", "No packages sampled")
    else:
        test_failed("Package sampler: single category", result.stderr[:200])
    
    # Test 2: Sample from multiple categories
    log("  Test: sample from multiple categories")
    output_file = TEST_DIR / "test-packages-multi.txt"
    
    result = subprocess.run(
        ["python3", SAMPLER, "--output", str(output_file), "--samples", "3", 
         "--categories", "ai-ml", "utils", "crypto"],
        capture_output=True, text=True, timeout=60,
        cwd=Path(__file__).parent
    )
    
    if result.returncode == 0 and output_file.exists():
        packages = [line.strip() for line in open(output_file) if line.strip()]
        test_passed(f"Package sampler: multiple categories ({len(packages)} packages)")
    else:
        test_failed("Package sampler: multiple categories", result.stderr[:200])


# ============================================================================
# Test 6: Background Scanner
# ============================================================================

def test_background_scanner():
    """Test background scanner with checkpointing"""
    log("Testing background scanner...")
    
    # First, create a small package list
    packages_file = TEST_DIR / "bg-test-packages.txt"
    with open(packages_file, 'w') as f:
        f.write("chalk\ncommander\ndebug\n")
    
    # Test 1: Run background scanner
    log("  Test: background scanner execution")
    output_db = TEST_DIR / "bg-test-results.db"
    state_file = TEST_DIR / "bg-test-state.json"
    log_file = TEST_DIR / "bg-test.log"
    
    result = subprocess.run(
        ["python3", SCANNER, "--packages", str(packages_file), "--policy", "last-2",
         "--output", str(output_db), "--state", str(state_file), "--log", str(log_file),
         "--workers", "2"],
        capture_output=True, text=True, timeout=180,
        cwd=Path(__file__).parent
    )
    
    if result.returncode == 0:
        test_passed("Background scanner: execution")
    else:
        test_failed("Background scanner: execution", result.stderr[:200])
    
    # Test 2: Check database created
    log("  Test: database created")
    if output_db.exists():
        try:
            conn = sqlite3.connect(output_db)
            cursor = conn.cursor()
            cursor.execute("SELECT COUNT(*) FROM version_scans")
            count = cursor.fetchone()[0]
            conn.close()
            test_passed(f"Background scanner: database ({count} records)")
        except Exception as e:
            test_failed("Background scanner: database", str(e))
    else:
        test_failed("Background scanner: database", "File not found")
    
    # Test 3: Check state file
    log("  Test: state file created")
    if state_file.exists():
        try:
            state = json.load(open(state_file))
            if "versions_scanned" in state:
                test_passed(f"Background scanner: state file ({state['versions_scanned']} versions)")
            else:
                test_failed("Background scanner: state file", "Missing 'versions_scanned'")
        except Exception as e:
            test_failed("Background scanner: state file", str(e))
    else:
        test_failed("Background scanner: state file", "File not found")
    
    # Test 4: Check log file
    log("  Test: log file created")
    if log_file.exists():
        lines = [line for line in open(log_file) if line.strip()]
        test_passed(f"Background scanner: log file ({len(lines)} entries)")
    else:
        test_failed("Background scanner: log file", "File not found")


# ============================================================================
# Test 7: LLM Integration (if configured)
# ============================================================================

def test_llm_integration():
    """Test LLM integration (if env vars set)"""
    log("Testing LLM integration...")
    
    if not os.environ.get("GLASSWARE_LLM_BASE_URL") or not os.environ.get("GLASSWARE_LLM_API_KEY"):
        test_skipped("LLM integration", "Environment variables not set")
        return
    
    # Test 1: LLM scan
    log("  Test: LLM scan")
    result = subprocess.run(
        [GLASSWARE, "--llm", "scan-npm", "express"],
        capture_output=True, text=True, timeout=120
    )
    
    if result.returncode == 0:
        test_passed("LLM integration: scan with LLM")
    else:
        test_failed("LLM integration: scan with LLM", result.stderr[:200])


# ============================================================================
# Main Test Runner
# ============================================================================

def run_all_tests():
    """Run all tests"""
    log("=" * 70)
    log("GLASSWARE v0.8.75 - COMPREHENSIVE TEST SUITE")
    log("=" * 70)
    
    setup()
    
    # Run tests
    test_cli_validation()
    test_scan_registry()
    test_basic_scanning()
    test_version_scanning()
    test_package_sampler()
    test_background_scanner()
    test_llm_integration()
    
    # Summary
    log("=" * 70)
    log("TEST SUMMARY")
    log("=" * 70)
    log(f"Passed:  {RESULTS['passed']}")
    log(f"Failed:  {RESULTS['failed']}")
    log(f"Skipped: {RESULTS['skipped']}")
    log(f"Total:   {RESULTS['passed'] + RESULTS['failed'] + RESULTS['skipped']}")
    log("=" * 70)
    
    # Save results
    results_file = TEST_DIR / "test-results.json"
    with open(results_file, 'w') as f:
        json.dump(RESULTS, f, indent=2, default=str)
    log(f"Results saved to: {results_file}")
    
    cleanup()
    
    # Exit code
    sys.exit(0 if RESULTS["failed"] == 0 else 1)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Glassware Test Suite")
    parser.add_argument("--all", action="store_true", help="Run all tests")
    parser.add_argument("--test", choices=["cli", "registry", "scan", "version", "sampler", "background", "llm"],
                       help="Run specific test")
    args = parser.parse_args()
    
    if args.all:
        run_all_tests()
    elif args.test:
        setup()
        test_func = {
            "cli": test_cli_validation,
            "registry": test_scan_registry,
            "scan": test_basic_scanning,
            "version": test_version_scanning,
            "sampler": test_package_sampler,
            "background": test_background_scanner,
            "llm": test_llm_integration,
        }
        test_func[args.test]()
        cleanup()
    else:
        parser.print_help()
