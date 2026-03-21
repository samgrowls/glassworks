"""
Scanner - GlassWorm CLI Wrapper

Thin wrapper that shells out to target/release/glassware CLI.
Accepts --llm flag passthrough. Parses JSON output into Finding dicts.

Usage:
    from harness.core import Scanner
    
    scanner = Scanner()
    findings = scanner.scan_directory("/path/to/package")
    findings_with_llm = scanner.scan_directory("/path/to/package", use_llm=True)
"""

import json
import subprocess
from pathlib import Path
from typing import List, Dict, Any, Optional

# Configuration - path to glassware CLI
GLASSWARE_CLI = Path(__file__).parent.parent.parent / "target" / "release" / "glassware"

# Fallback to debug build if release doesn't exist
if not GLASSWARE_CLI.exists():
    GLASSWARE_CLI = Path(__file__).parent.parent.parent / "target" / "debug" / "glassware"


class Scanner:
    """
    GlassWorm CLI scanner wrapper.
    
    Shells out to the Rust CLI and parses JSON output.
    Supports --llm flag passthrough for Cerebras triage.
    """

    def __init__(self, cli_path: Optional[Path] = None):
        """
        Initialize scanner.
        
        Args:
            cli_path: Path to glassware CLI (defaults to target/release/glassware)
        """
        self.cli_path = cli_path or GLASSWARE_CLI
        
        if not self.cli_path.exists():
            raise FileNotFoundError(
                f"GlassWorm CLI not found at {self.cli_path}. "
                "Run: cargo build -p glassware-cli --release"
            )

    def scan_directory(
        self,
        directory: str,
        severity: str = "info",
        use_llm: bool = False,
        timeout: int = 120,
    ) -> List[Dict[str, Any]]:
        """
        Scan a directory with glassware CLI.
        
        Args:
            directory: Path to directory to scan
            severity: Minimum severity to report (info, low, medium, high, critical)
            use_llm: Enable LLM analysis (requires GLASSWARE_LLM_BASE_URL and GLASSWARE_LLM_API_KEY)
            timeout: Scan timeout in seconds
            
        Returns:
            List of finding dicts
        """
        cmd = [
            str(self.cli_path),
            "--format", "json",
            "--severity", severity,
            str(directory),
        ]
        
        if use_llm:
            cmd.insert(3, "--llm")

        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=timeout,
            )
            
            # Parse JSON output
            try:
                data = json.loads(result.stdout)
                return data.get("findings", [])
            except json.JSONDecodeError:
                # CLI may output error message
                return []
                
        except subprocess.TimeoutExpired:
            return []
        except Exception:
            return []

    def scan_tarball(
        self,
        tarball_path: str,
        severity: str = "info",
        use_llm: bool = False,
        timeout: int = 120,
    ) -> List[Dict[str, Any]]:
        """
        Extract and scan a tarball.
        
        Args:
            tarball_path: Path to .tgz file
            severity: Minimum severity to report
            use_llm: Enable LLM analysis
            timeout: Scan timeout in seconds
            
        Returns:
            List of finding dicts
        """
        import tempfile
        import tarfile
        
        with tempfile.TemporaryDirectory() as tmpdir:
            try:
                # Extract tarball
                with tarfile.open(tarball_path, "r:gz") as tar:
                    tar.extractall(tmpdir, filter="data")
                
                # Find package directory
                pkg_dir = Path(tmpdir) / "package"
                if not pkg_dir.exists():
                    # Try to find any directory
                    dirs = [d for d in Path(tmpdir).iterdir() if d.is_dir()]
                    if dirs:
                        pkg_dir = dirs[0]
                
                if pkg_dir.exists():
                    return self.scan_directory(
                        str(pkg_dir),
                        severity=severity,
                        use_llm=use_llm,
                        timeout=timeout,
                    )
                return []
                
            except Exception:
                return []

    def get_scan_summary(self, findings: List[Dict[str, Any]]) -> Dict[str, Any]:
        """
        Generate summary statistics from findings.
        
        Args:
            findings: List of finding dicts
            
        Returns:
            Summary dict with counts by severity, category, etc.
        """
        summary = {
            "total": len(findings),
            "by_severity": {},
            "by_category": {},
            "files": set(),
        }
        
        for finding in findings:
            # Count by severity
            sev = finding.get("severity", "unknown").lower()
            summary["by_severity"][sev] = summary["by_severity"].get(sev, 0) + 1
            
            # Count by category
            cat = finding.get("category", "unknown")
            summary["by_category"][cat] = summary["by_category"].get(cat, 0) + 1
            
            # Track files
            if "file" in finding:
                summary["files"].add(finding["file"])
        
        summary["files"] = list(summary["files"])
        summary["critical"] = summary["by_severity"].get("critical", 0)
        summary["high"] = summary["by_severity"].get("high", 0)
        summary["medium"] = summary["by_severity"].get("medium", 0)
        summary["low"] = summary["by_severity"].get("low", 0)
        summary["info"] = summary["by_severity"].get("info", 0)
        
        return summary
