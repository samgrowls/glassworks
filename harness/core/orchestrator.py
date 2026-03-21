"""
Orchestrator - Wave-Based Pipeline Runner

Reads waves.toml, runs fetch→scan→analyze→report pipeline per wave.
Checkpoint/resume via store.py.

CLI entry point with subcommands:
- run-wave: Run a specific wave
- status: Show wave progress
- report: Generate reports

Usage:
    python -m harness.core.orchestrator run-wave --wave 0
    python -m harness.core.orchestrator status
    python -m harness.core.orchestrator report --wave 0
"""

import argparse
import sys
import tomllib
from datetime import datetime
from pathlib import Path
from typing import Dict, Any, List, Optional

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from core.store import Store
from core.fetcher import Fetcher
from core.scanner import Scanner
from core.analyzer import Analyzer
from core.reporter import Reporter

# Configuration
WAVES_FILE = Path(__file__).parent.parent / "waves.toml"
DB_PATH = Path(__file__).parent.parent / "data" / "corpus.db"


class Orchestrator:
    """
    Wave-based pipeline orchestrator.
    
    Reads waves.toml, runs fetch→scan→analyze→report pipeline per wave.
    Supports checkpoint/resume.
    """

    def __init__(self, db_path: Optional[Path] = None):
        """
        Initialize orchestrator.
        
        Args:
            db_path: Database path (defaults to data/corpus.db)
        """
        self.db_path = db_path or DB_PATH
        self.store = Store(self.db_path)
        self.fetcher = Fetcher()
        self.scanner = Scanner()
        self.analyzer = Analyzer(self.store)
        self.reporter = Reporter(self.store)
        self.waves = self._load_waves()

    def _load_waves(self) -> Dict[str, Any]:
        """Load wave configurations."""
        if not WAVES_FILE.exists():
            return {}
        
        with open(WAVES_FILE, "rb") as f:
            return tomllib.load(f)

    def run_wave(
        self,
        wave_id: int,
        use_llm: bool = False,
        max_workers: int = 5,
        resume: bool = True,
    ) -> Dict[str, Any]:
        """
        Run a wave.
        
        Args:
            wave_id: Wave ID to run
            use_llm: Enable LLM analysis
            max_workers: Maximum parallel workers
            resume: Resume from checkpoint
            
        Returns:
            Wave results summary
        """
        wave_config = self.waves.get(f"wave_{wave_id}")
        if not wave_config:
            return {"error": f"Wave {wave_id} not found"}

        print(f"🚀 Starting Wave {wave_id}: {wave_config.get('name', 'Unknown')}")
        print(f"   Description: {wave_config.get('description', '')}")
        print()

        # Create scan run
        run_id = self.store.create_scan_run(
            wave_id=wave_id,
            filter_params={"wave_config": wave_config},
            notes=wave_config.get("description", ""),
        )
        print(f"📋 Run ID: {run_id}")
        print()

        # Collect packages from all sources
        packages = self._collect_packages(wave_config)
        print(f"📦 Collected {len(packages)} packages")

        # Filter already scanned
        if resume:
            pending = self.store.get_pending_packages(run_id, packages)
            print(f"⏭  Resuming: {len(pending)} pending (skipped {len(packages) - len(pending)})")
            packages = pending
        print()

        # Process packages
        scanned = 0
        flagged = 0
        
        for i, pkg_spec in enumerate(packages):
            print(f"[{i+1}/{len(packages)}] Processing {pkg_spec}...")
            
            # Download
            dl_info = self.fetcher.download_package(pkg_spec)
            if not dl_info:
                print(f"   ❌ Download failed")
                self.store.save_checkpoint(run_id, pkg_spec.split("@")[0], pkg_spec.split("@")[-1] if "@" in pkg_spec else "latest", "failed")
                continue

            # Scan
            findings = self.scanner.scan_tarball(
                dl_info["tarball_path"],
                severity="info",
                use_llm=False,  # Cerebras triage is in CLI, NVIDIA is separate
            )
            
            # Save results
            if findings:
                flagged += 1
                print(f"   ⚠️  Flagged: {len(findings)} findings")
            else:
                print(f"   ✅ Clean")

            self.store.save_package_scan(
                run_id=run_id,
                name=dl_info["name"],
                version=dl_info["version"],
                findings=findings,
                tarball_sha256=dl_info["tarball_sha256"],
                vault_path=str(dl_info["tarball_path"]) if findings else "",
            )
            scanned += 1

            # LLM analysis for high-severity
            if use_llm and findings:
                high_sev = [f for f in findings if f.get("severity", "").lower() in ["critical", "high"]]
                if high_sev:
                    print(f"   🧠 Running NVIDIA LLM analysis...")
                    llm_result = self.analyzer.analyze_package(
                        dl_info["name"],
                        dl_info["version"],
                        dl_info["tarball_path"],
                        high_sev,
                    )
                    if llm_result:
                        print(f"   📊 LLM verdict: {llm_result.get('malicious', 'unknown')}")

        # Finish run
        self.store.finish_scan_run(run_id)
        
        # Generate report
        report_paths = self.reporter.save_report(run_id)
        print()
        print(f"📄 Reports saved:")
        print(f"   Markdown: {report_paths['markdown']}")
        print(f"   JSON: {report_paths['json']}")
        print()

        # Summary
        progress = self.store.get_run_progress(run_id)
        return {
            "run_id": run_id,
            "wave_id": wave_id,
            "packages_scanned": scanned,
            "packages_flagged": flagged,
            "progress": progress,
            "reports": {k: str(v) for k, v in report_paths.items()},
        }

    def _collect_packages(self, wave_config: Dict) -> List[str]:
        """Collect packages from wave configuration."""
        packages = []
        
        for section, config in wave_config.items():
            if not isinstance(config, dict):
                continue
            
            # Known malicious packages
            if "packages" in config:
                packages.extend(config["packages"])
            
            # Sample from keywords
            if "keywords" in config:
                count = config.get("count", 50)
                sampled = self.fetcher.sample_packages(
                    categories={"custom": config["keywords"]},
                    samples_per_category=count // len(config["keywords"]),
                )
                packages.extend(sampled[:count])
            
            # Recent publishes
            if "days" in config:
                count = config.get("count", 100)
                recent = self.fetcher.get_recent_packages(
                    days=config["days"],
                    limit=count,
                )
                packages.extend(recent)
            
            # Typosquats
            if "base_packages" in config:
                count = config.get("count", 25)
                typosquats = self.fetcher.get_typosquats(config["base_packages"])
                packages.extend(typosquats[:count])

        return list(set(packages))  # Deduplicate

    def get_status(self, wave_id: Optional[int] = None) -> Dict[str, Any]:
        """
        Get wave status.
        
        Args:
            wave_id: Optional wave ID (defaults to all waves)
            
        Returns:
            Status dict
        """
        if wave_id is not None:
            runs = self.store.get_wave_runs(wave_id)
            wave_summary = self.reporter.generate_wave_summary(wave_id)
            return {
                "wave_id": wave_id,
                "runs": len(runs),
                "summary": wave_summary,
            }
        
        # All waves
        all_status = {}
        for wave_key in self.waves:
            if wave_key.startswith("wave_"):
                wid = int(wave_key.split("_")[1])
                all_status[wid] = self.get_status(wid)
        
        return all_status

    def generate_report(self, wave_id: int, output_dir: Optional[Path] = None) -> Path:
        """
        Generate wave report.
        
        Args:
            wave_id: Wave ID
            output_dir: Output directory
            
        Returns:
            Path to report
        """
        runs = self.store.get_wave_runs(wave_id)
        if not runs:
            raise ValueError(f"No runs found for wave {wave_id}")
        
        # Generate report for latest run
        latest_run = runs[0]
        report_paths = self.reporter.save_report(latest_run["id"], output_dir)
        return report_paths["markdown"]


def main():
    """CLI entry point."""
    parser = argparse.ArgumentParser(description="GlassWorm Wave Orchestrator")
    subparsers = parser.add_subparsers(dest="command", help="Commands")

    # run-wave
    run_parser = subparsers.add_parser("run-wave", help="Run a wave")
    run_parser.add_argument("--wave", type=int, required=True, help="Wave ID")
    run_parser.add_argument("--llm", action="store_true", help="Enable LLM analysis")
    run_parser.add_argument("--workers", type=int, default=5, help="Max workers")
    run_parser.add_argument("--no-resume", action="store_true", help="Don't resume")

    # status
    status_parser = subparsers.add_parser("status", help="Show wave status")
    status_parser.add_argument("--wave", type=int, help="Wave ID")

    # report
    report_parser = subparsers.add_parser("report", help="Generate report")
    report_parser.add_argument("--wave", type=int, required=True, help="Wave ID")
    report_parser.add_argument("--output", type=Path, help="Output directory")

    args = parser.parse_args()
    orchestrator = Orchestrator()

    if args.command == "run-wave":
        result = orchestrator.run_wave(
            wave_id=args.wave,
            use_llm=args.llm,
            max_workers=args.workers,
            resume=not args.no_resume,
        )
        print(f"\n✅ Wave {args.wave} complete!")
        print(f"   Scanned: {result.get('packages_scanned', 0)}")
        print(f"   Flagged: {result.get('packages_flagged', 0)}")
        
    elif args.command == "status":
        status = orchestrator.get_status(args.wave)
        print(f"Wave Status:")
        print(f"  {status}")
        
    elif args.command == "report":
        report_path = orchestrator.generate_report(args.wave, args.output)
        print(f"Report generated: {report_path}")
        
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
