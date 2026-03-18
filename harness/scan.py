"""
npm Scanning Harness - Main Orchestrator

Orchestrates glassware CLI against npm packages:
1. Select packages via selector.py
2. Download and extract tarballs
3. Run glassware scanner
4. Archive flagged packages to vault
5. Store results in SQLite corpus
"""

import argparse
import hashlib
import json
import os
import shutil
import signal
import sqlite3
import subprocess
import sys
import tarfile
import tempfile
import time
from dataclasses import asdict
from datetime import datetime
from pathlib import Path
from typing import Optional

from rich.console import Console
from rich.progress import (
    BarColumn,
    MofNCompleteColumn,
    Progress,
    TaskProgressColumn,
    TextColumn,
    TimeRemainingColumn,
)
from rich.table import Table

from database import Database
from selector import NPMSelector, PackageCandidate

console = Console()

# Configuration
HARNESS_DIR = Path(__file__).parent
DATA_DIR = HARNESS_DIR / "data"
VAULT_DIR = DATA_DIR / "vault"
DB_PATH = DATA_DIR / "corpus.db"
REPORTS_DIR = HARNESS_DIR / "reports"

# Ensure directories exist
VAULT_DIR.mkdir(parents=True, exist_ok=True)
REPORTS_DIR.mkdir(parents=True, exist_ok=True)


class ScanInterrupted(Exception):
    """Raised when scan is interrupted by user."""

    pass


def sha256_file(path: Path) -> str:
    """Calculate SHA256 hash of a file."""
    sha256 = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            sha256.update(chunk)
    return sha256.hexdigest()


def download_tarball(url: str, dest: Path, session=None) -> bool:
    """Download a tarball from URL."""
    import requests

    try:
        response = requests.get(url, stream=True, timeout=30)
        response.raise_for_status()

        with open(dest, "wb") as f:
            for chunk in response.iter_content(chunk_size=8192):
                f.write(chunk)

        return True
    except Exception as e:
        console.print(f"[red]Download failed: {e}[/red]")
        return False


def extract_tarball(tarball: Path, dest_dir: Path) -> bool:
    """Extract a tarball to a directory."""
    try:
        with tarfile.open(tarball, "r:gz") as tar:
            tar.extractall(dest_dir)
        return True
    except Exception as e:
        console.print(f"[red]Extraction failed: {e}[/red]")
        return False


def find_glassware() -> Optional[str]:
    """Find the glassware binary in PATH or common locations."""
    # Try PATH first
    glassware = shutil.which("glassware")
    if glassware:
        return glassware

    # Try cargo bin
    cargo_bin = Path.home() / ".cargo" / "bin" / "glassware"
    if cargo_bin.exists():
        return str(cargo_bin)

    # Try workspace target directory
    workspace_root = HARNESS_DIR.parent
    target_debug = workspace_root / "target" / "debug" / "glassware"
    target_release = workspace_root / "target" / "release" / "glassware"

    if target_release.exists():
        return str(target_release)
    if target_debug.exists():
        return str(target_debug)

    return None


def run_glassware(
    scan_path: Path, glassware_path: str, use_llm: bool = False
) -> tuple[Optional[dict], int, str]:
    """
    Run glassware scanner on a directory.

    Returns:
        - Parsed JSON output (or None if failed)
        - Exit code
        - Raw stdout
    """
    cmd = [glassware_path, "--format", "json", str(scan_path)]

    if use_llm:
        cmd.append("--llm")

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=300,  # 5 minute timeout per package
        )

        exit_code = result.returncode
        stdout = result.stdout

        # Parse JSON output
        output_data = None
        if stdout.strip():
            try:
                output_data = json.loads(stdout)
            except json.JSONDecodeError:
                console.print(f"[yellow]Failed to parse glassware JSON output[/yellow]")

        return output_data, exit_code, stdout

    except subprocess.TimeoutExpired:
        console.print("[red]Glassware timed out (5min limit)[/red]")
        return None, -1, ""
    except Exception as e:
        console.print(f"[red]Glassware execution failed: {e}[/red]")
        return None, -2, ""


def archive_to_vault(
    tarball: Path, extracted: Path, name: str, version: str
) -> Optional[str]:
    """Archive a flagged package to the vault."""
    vault_name = f"{name}-{version}"
    vault_tarball = VAULT_DIR / f"{vault_name}.tgz"
    vault_source = VAULT_DIR / vault_name

    try:
        # Copy tarball
        shutil.copy2(tarball, vault_tarball)

        # Copy extracted source
        if extracted.exists():
            shutil.copytree(extracted, vault_source, dirs_exist_ok=True)

        return str(vault_tarball)
    except Exception as e:
        console.print(f"[red]Failed to archive to vault: {e}[/red]")
        return None


def parse_glassware_findings(output_data: dict) -> list[dict]:
    """Parse glassware JSON output into finding records."""
    findings = []

    for finding in output_data.get("findings", []):
        parsed = {
            "file_path": finding.get("file", ""),
            "line": finding.get("line"),
            "column": finding.get("column"),
            "rule_id": None,  # glassware uses category instead
            "category": finding.get("category", ""),
            "severity": finding.get("severity", "low"),
            "message": finding.get("message", ""),
            "confidence": None,
            "decoded_payload": finding.get("decoded"),
        }
        findings.append(parsed)

    return findings


class Scanner:
    """Main scanning orchestrator."""

    def __init__(
        self,
        max_packages: int = 100,
        days_back: int = 30,
        download_threshold: int = 1000,
        use_llm: bool = False,
        resume_run_id: Optional[str] = None,
    ):
        self.max_packages = max_packages
        self.days_back = days_back
        self.download_threshold = download_threshold
        self.use_llm = use_llm
        self.resume_run_id = resume_run_id

        self.db = Database(DB_PATH)
        self.glassware_path = find_glassware()
        self.current_package = 0
        self.total_packages = 0
        self.interrupted = False

        # Set up signal handlers
        signal.signal(signal.SIGINT, self._handle_interrupt)
        signal.signal(signal.SIGTERM, self._handle_interrupt)

    def _handle_interrupt(self, signum, frame):
        """Handle Ctrl+C gracefully."""
        console.print("\n[yellow]Scan interrupted. Committing current state...[/yellow]")
        self.interrupted = True
        raise ScanInterrupted()

    def _get_glassware_version(self) -> str:
        """Get glassware version string."""
        if not self.glassware_path:
            return "not found"

        try:
            result = subprocess.run(
                [self.glassware_path, "--version"],
                capture_output=True,
                text=True,
                timeout=5,
            )
            return result.stdout.strip() or "unknown"
        except Exception:
            return "unknown"

    def select_packages(self) -> list[PackageCandidate]:
        """Select packages to scan using Tier 1 criteria."""
        console.print("\n[bold blue]=== Package Selection ===[/bold blue]")

        if not self.glassware_path:
            console.print(
                "[red]Error: glassware binary not found.[/red]\n"
                "[dim]Install with: cargo install --path glassware-cli[/dim]"
            )
            sys.exit(1)

        console.print(f"[dim]Glassware: {self.glassware_path}[/dim]")

        async def run_selector():
            async with NPMSelector(
                days_back=self.days_back,
                download_threshold=self.download_threshold,
                max_packages=self.max_packages,
            ) as selector:
                return await selector.select_packages()

        import asyncio

        candidates = asyncio.run(run_selector())
        return candidates

    def scan_package(
        self, candidate: PackageCandidate, run_id: str, tarball_path: Path
    ) -> tuple[int, Optional[str], Optional[dict]]:
        """
        Scan a single package.

        Returns:
            - Finding count
            - Vault path (if flagged)
            - Raw glassware output data (for inserting findings)
        """
        name = candidate.name
        version = candidate.version

        with tempfile.TemporaryDirectory() as tmpdir:
            tmpdir = Path(tmpdir)
            extract_path = tmpdir / "extracted"

            # Calculate hash
            tarball_hash = sha256_file(tarball_path)

            # Extract
            extract_path.mkdir()
            if not extract_tarball(tarball_path, extract_path):
                return 0, None, None

            # Find package.json to get actual extracted dir
            package_dir = extract_path / "package"
            if not package_dir.exists():
                # Try to find the actual package directory
                for item in extract_path.iterdir():
                    if item.is_dir():
                        package_dir = item
                        break

            # Run glassware
            output_data, exit_code, stdout = run_glassware(
                package_dir, self.glassware_path, self.use_llm
            )

            # Parse findings
            finding_count = 0
            vault_path = None

            if output_data:
                findings = parse_glassware_findings(output_data)
                finding_count = len(findings)

            # Archive if flagged
            if finding_count > 0:
                vault_path = archive_to_vault(
                    tarball_path, package_dir, name, version
                )

            return finding_count, vault_path, output_data

    def scan_package_with_tarball(
        self, candidate: PackageCandidate, run_id: str, tarball_path: Path
    ) -> tuple[int, Optional[str], Optional[dict]]:
        """Wrapper for scan_package that takes tarball path."""
        return self.scan_package(candidate, run_id, tarball_path)

    def rescan_run(self, original_run_id: str) -> str:
        """Re-scan flagged packages from a previous run, optionally with LLM."""
        console.print(f"\n[bold blue]=== Re-scanning Run {original_run_id[:8]}... ===[/bold blue]")

        # Get flagged packages from original run
        flagged = self.db.get_flagged_packages(original_run_id)
        if not flagged:
            console.print("[yellow]No flagged packages found in that run[/yellow]")
            return ""

        console.print(f"Found {len(flagged)} flagged packages to re-scan")

        # Create new scan run for re-scan results
        filter_params = {"rescan_of": original_run_id, "with_llm": self.use_llm}
        run_id = self.db.create_scan_run(
            filter_params=filter_params,
            glassware_version=self._get_glassware_version(),
            notes=f"Re-scan of run {original_run_id}" + (" with LLM" if self.use_llm else ""),
        )

        packages_flagged = 0
        self.total_packages = len(flagged)

        try:
            with Progress(
                TextColumn("[bold blue]{task.description}"),
                BarColumn(),
                MofNCompleteColumn(),
                TaskProgressColumn(),
                TimeRemainingColumn(),
                console=console,
            ) as progress:
                scan_task = progress.add_task(
                    "Re-scanning packages", total=self.total_packages
                )

                for i, pkg in enumerate(flagged):
                    if self.interrupted:
                        break

                    self.current_package = i + 1
                    name = pkg["name"]
                    version = pkg["version"]

                    progress.update(
                        scan_task,
                        advance=1,
                        description=f"[bold blue]Re-scanning {name}@{version}[/bold blue]",
                    )

                    start_time = time.time()

                    # Download tarball from vault or re-download from npm
                    with tempfile.TemporaryDirectory() as tmpdir:
                        tmpdir = Path(tmpdir)
                        tarball_path = tmpdir / "package.tgz"

                        # Try vault first
                        vault_path = pkg["vault_path"]
                        if vault_path and Path(vault_path).exists():
                            shutil.copy2(vault_path, tarball_path)
                        else:
                            # Re-download from npm
                            tarball_url = pkg["tarball_url"]
                            if not download_tarball(tarball_url, tarball_path):
                                console.print(f"  [yellow]⚠ Download failed[/yellow]")
                                continue

                        tarball_hash = sha256_file(tarball_path)

                        # Scan the package
                        finding_count, vault_path_new, output_data = self.scan_package_with_tarball(
                            PackageCandidate(
                                name=name,
                                version=version,
                                tarball_url=pkg["tarball_url"],
                                published_at=pkg["published_at"],
                                author=pkg["author"],
                                downloads_weekly=pkg["downloads_weekly"],
                                has_install_scripts=pkg["has_install_scripts"],
                                scripts=[],
                            ),
                            run_id,
                            tarball_path,
                        )

                    scan_duration_ms = int((time.time() - start_time) * 1000)

                    # Add to database (will fail on UNIQUE constraint if already exists, which is fine)
                    try:
                        package_id = self.db.add_package(
                            run_id=run_id,
                            name=name,
                            version=version,
                            tarball_url=pkg["tarball_url"],
                            tarball_sha256=tarball_hash,
                            published_at=pkg["published_at"],
                            author=pkg["author"],
                            downloads_weekly=pkg["downloads_weekly"],
                            has_install_scripts=pkg["has_install_scripts"],
                        )

                        # Update with scan results
                        self.db.update_package_scan(
                            package_id=package_id,
                            finding_count=finding_count,
                            scan_duration_ms=scan_duration_ms,
                            glassware_output="",
                            vault_path=vault_path_new,
                        )

                        # Insert individual findings
                        if output_data:
                            findings = parse_glassware_findings(output_data)
                            for finding in findings:
                                self.db.add_finding(
                                    package_id=package_id,
                                    file_path=finding["file_path"],
                                    line=finding["line"],
                                    column=finding["column"],
                                    rule_id=finding["rule_id"],
                                    category=finding["category"],
                                    severity=finding["severity"],
                                    message=finding["message"],
                                    confidence=finding["confidence"],
                                    decoded_payload=finding["decoded_payload"],
                                    raw_json="",
                                )
                    except sqlite3.IntegrityError:
                        # Package already exists in this run - skip
                        pass

                    if finding_count > 0:
                        packages_flagged += 1
                        console.print(
                            f"  [red]⚠ {finding_count} findings[/red] "
                            f"[dim]({scan_duration_ms}ms)[/dim]"
                        )
                    else:
                        console.print(
                            f"  [green]✓ Clean[/green] [dim]({scan_duration_ms}ms)[/dim]"
                        )

        except ScanInterrupted:
            console.print(
                "\n[yellow]Re-scan interrupted. Finalizing run record...[/yellow]"
            )

        # Finalize run
        self.db.finalize_scan_run(run_id, self.total_packages, packages_flagged)

        # Print summary
        self.print_summary(run_id)

        return run_id

    def run(self) -> str:
        """Run the full scanning workflow."""
        console.print("\n[bold blue]╔════════════════════════════════════════╗[/bold blue]")
        console.print("[bold blue]║     glassware npm Scanning Harness    ║[/bold blue]")
        console.print("[bold blue]╚════════════════════════════════════════╝[/bold blue]")

        # Select packages
        candidates = self.select_packages()
        if not candidates:
            console.print("[yellow]No packages matched Tier 1 criteria[/yellow]")
            return ""

        self.total_packages = len(candidates)

        # Create scan run
        filter_params = {
            "days_back": self.days_back,
            "download_threshold": self.download_threshold,
            "tier": 1,
        }

        if self.resume_run_id:
            run_id = self.resume_run_id
            console.print(f"[dim]Resuming run: {run_id}[/dim]")
        else:
            run_id = self.db.create_scan_run(
                filter_params=filter_params,
                glassware_version=self._get_glassware_version(),
            )
            console.print(f"[dim]New run: {run_id}[/dim]")

        packages_flagged = 0

        try:
            with Progress(
                TextColumn("[bold blue]{task.description}"),
                BarColumn(),
                MofNCompleteColumn(),
                TaskProgressColumn(),
                TimeRemainingColumn(),
                console=console,
            ) as progress:
                scan_task = progress.add_task(
                    "Scanning packages", total=self.total_packages
                )

                for i, candidate in enumerate(candidates):
                    if self.interrupted:
                        break

                    self.current_package = i + 1
                    progress.update(
                        scan_task,
                        advance=1,
                        description=f"[bold blue]Scanning {candidate.name}@{candidate.version}[/bold blue]",
                    )

                    start_time = time.time()

                    # Download tarball to temp dir and calculate hash
                    with tempfile.TemporaryDirectory() as tmpdir:
                        tmpdir = Path(tmpdir)
                        tarball_path = tmpdir / "package.tgz"
                        
                        if not download_tarball(candidate.tarball_url, tarball_path):
                            console.print(f"  [yellow]⚠ Download failed[/yellow]")
                            continue
                        
                        tarball_hash = sha256_file(tarball_path)
                        
                        # Scan the package (includes extract + glassware run)
                        finding_count, vault_path, output_data = self.scan_package_with_tarball(
                            candidate, run_id, tarball_path
                        )

                    scan_duration_ms = int((time.time() - start_time) * 1000)

                    # Add to database
                    package_id = self.db.add_package(
                        run_id=run_id,
                        name=candidate.name,
                        version=candidate.version,
                        tarball_url=candidate.tarball_url,
                        tarball_sha256=tarball_hash,
                        published_at=candidate.published_at,
                        author=candidate.author,
                        downloads_weekly=candidate.downloads_weekly,
                        has_install_scripts=candidate.has_install_scripts,
                    )

                    # Update with scan results
                    self.db.update_package_scan(
                        package_id=package_id,
                        finding_count=finding_count,
                        scan_duration_ms=scan_duration_ms,
                        glassware_output="",  # Would need to capture this
                        vault_path=vault_path,
                    )

                    # Insert individual findings
                    if output_data:
                        findings = parse_glassware_findings(output_data)
                        for finding in findings:
                            self.db.add_finding(
                                package_id=package_id,
                                file_path=finding["file_path"],
                                line=finding["line"],
                                column=finding["column"],
                                rule_id=finding["rule_id"],
                                category=finding["category"],
                                severity=finding["severity"],
                                message=finding["message"],
                                confidence=finding["confidence"],
                                decoded_payload=finding["decoded_payload"],
                                raw_json="",  # Could store full finding JSON
                            )

                    if finding_count > 0:
                        packages_flagged += 1
                        console.print(
                            f"  [red]⚠ {finding_count} findings[/red] "
                            f"[dim]({scan_duration_ms}ms)[/dim]"
                        )
                    else:
                        console.print(
                            f"  [green]✓ Clean[/green] [dim]({scan_duration_ms}ms)[/dim]"
                        )

        except ScanInterrupted:
            console.print(
                "\n[yellow]Scan interrupted. Finalizing run record...[/yellow]"
            )

        # Finalize run
        self.db.finalize_scan_run(run_id, self.total_packages, packages_flagged)

        # Print summary
        self.print_summary(run_id)

        return run_id

    def print_summary(self, run_id: str) -> None:
        """Print scan run summary."""
        run = self.db.get_scan_run(run_id)
        if not run:
            return

        console.print("\n[bold blue]=== Scan Summary ===[/bold blue]")

        table = Table(show_header=False, box=None)
        table.add_column("Label", style="dim")
        table.add_column("Value")

        table.add_row("Run ID", run_id[:8] + "...")
        table.add_row("Packages scanned", str(run["packages_total"]))
        table.add_row("Packages flagged", str(run["packages_flagged"]))

        if run["finished_at"]:
            started = datetime.fromisoformat(run["started_at"])
            finished = datetime.fromisoformat(run["finished_at"])
            duration = finished - started
            table.add_row("Duration", str(duration))

        console.print(table)

        # Show flagged packages
        if run["packages_flagged"] > 0:
            console.print("\n[bold red]Flagged packages:[/bold red]")

            flagged = self.db.get_flagged_packages(run_id)
            for pkg in flagged[:10]:  # Show top 10
                findings = self.db.get_findings_for_package(pkg["id"])
                severities = [f["severity"] for f in findings]
                max_severity = max(severities, key=lambda s: ["low", "medium", "high", "critical"].index(s) if s in ["low", "medium", "high", "critical"] else 0)

                console.print(
                    f"  [red]{pkg['name']}@{pkg['version']}[/red] "
                    f"[dim]({pkg['finding_count']} findings, {max_severity})[/dim]"
                )


def main():
    parser = argparse.ArgumentParser(
        description="glassware npm Scanning Harness",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    parser.add_argument(
        "--max-packages",
        type=int,
        default=100,
        help="Maximum packages to scan (default: 100)",
    )
    parser.add_argument(
        "--days-back",
        type=int,
        default=30,
        help="Only scan packages published within N days (default: 30)",
    )
    parser.add_argument(
        "--download-threshold",
        type=int,
        default=1000,
        help="Max weekly downloads for Tier 1 (default: 1000)",
    )
    parser.add_argument(
        "--tier",
        type=int,
        choices=[1],
        default=1,
        help="Package selection tier (only Tier 1 implemented)",
    )
    parser.add_argument(
        "--resume",
        action="store_true",
        help="Resume last interrupted scan run",
    )
    parser.add_argument(
        "--rescan",
        type=str,
        metavar="RUN_ID",
        help="Re-scan flagged packages from a previous run",
    )
    parser.add_argument(
        "--with-llm",
        action="store_true",
        help="Enable LLM analysis (L3 layer) on re-scan",
    )
    parser.add_argument(
        "--stats",
        action="store_true",
        help="Show corpus statistics",
    )

    args = parser.parse_args()

    if args.stats:
        db = Database(DB_PATH)
        stats = db.get_statistics()

        console.print("\n[bold blue]=== Corpus Statistics ===[/bold blue]")
        table = Table(show_header=False, box=None)
        table.add_column("Label")
        table.add_column("Value", style="bold")

        table.add_row("Total packages", str(stats["total_packages"]))
        table.add_row("Flagged packages", str(stats["flagged_packages"]))
        table.add_row("Total findings", str(stats["total_findings"]))

        if stats["by_severity"]:
            table.add_row("", "")
            table.add_row("By severity:", "")
            for sev, count in stats["by_severity"].items():
                table.add_row(f"  {sev}", str(count))

        if stats["by_category"]:
            table.add_row("", "")
            table.add_row("By category:", "")
            for cat, count in stats["by_category"].items():
                table.add_row(f"  {cat}", str(count))

        console.print(table)
        return

    scanner = Scanner(
        max_packages=args.max_packages,
        days_back=args.days_back,
        download_threshold=args.download_threshold,
        use_llm=args.with_llm,
        resume_run_id=args.resume,
    )

    if args.rescan:
        # Re-scan flagged packages from previous run
        console.print(f"[dim]Re-scanning flagged packages from run {args.rescan}[/dim]")
        run_id = scanner.rescan_run(args.rescan)
    else:
        run_id = scanner.run()

    if run_id:
        console.print(f"\n[green]Scan complete. Run ID: {run_id}[/green]")
        console.print(
            f"[dim]Results stored in: {DB_PATH}[/dim]"
        )


if __name__ == "__main__":
    main()
