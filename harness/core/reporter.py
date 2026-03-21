"""
Reporter - Markdown and JSON Report Generation

Extended from existing reporter.py with wave-awareness,
detector distribution stats, and JSON export.

Usage:
    from harness.core import Reporter, Store
    
    store = Store("data/corpus.db")
    reporter = Reporter(store)
    
    # Generate markdown report
    md_report = reporter.generate_markdown(run_id)
    
    # Generate JSON report
    json_report = reporter.generate_json(run_id)
    
    # Generate wave summary
    wave_summary = reporter.generate_wave_summary(wave_id=0)
"""

import json
from datetime import datetime
from pathlib import Path
from typing import Dict, Any, List, Optional

REPORTS_DIR = Path(__file__).parent.parent / "reports"
REPORTS_DIR.mkdir(parents=True, exist_ok=True)


class Reporter:
    """
    Report generator with wave-awareness.
    
    Extended from existing reporter.py.
    Generates Markdown and JSON reports.
    """

    def __init__(self, store: Any):
        """
        Initialize reporter.
        
        Args:
            store: Store instance for database access
        """
        self.store = store

    def generate_markdown(self, run_id: str) -> str:
        """
        Generate a markdown report for a scan run.
        
        Args:
            run_id: Scan run ID
            
        Returns:
            Markdown report string
        """
        run = self.store.get_scan_run(run_id)
        if not run:
            return f"Run {run_id} not found"

        flagged = self.store.get_flagged_packages(run_id)
        stats = self.store.get_stats()

        # Parse filter params
        filter_params = json.loads(run["filter_params"]) if run["filter_params"] else {}
        wave_id = run.get("wave_id", 0)

        # Build report
        lines = [
            f"# GlassWorm Scan Report",
            "",
            f"**Run ID:** `{run_id}`",
            f"**Wave:** {wave_id}",
            f"**Generated:** {datetime.utcnow().isoformat()}",
            "",
            "## Summary",
            "",
            "| Metric | Value |",
            "|--------|-------|",
            f"| Packages scanned | {run.get('packages_total', 'N/A')} |",
            f"| Packages flagged | {run.get('packages_flagged', 0)} |",
        ]
        
        if run.get("packages_total", 0) > 0:
            rate = run["packages_flagged"] / run["packages_total"] * 100
            lines.append(f"| Detection rate | {rate:.1f}% |")
        else:
            lines.append("| Detection rate | N/A |")
        
        lines.extend([
            "",
            "## Scan Parameters",
            "",
            f"- **Days back:** {filter_params.get('days_back', 'N/A')}",
            f"- **Categories:** {', '.join(filter_params.get('categories', ['N/A']))}",
            f"- **Glassware version:** {run.get('glassware_version', 'unknown')}",
            "",
        ])

        # Timeline
        if run.get("started_at") and run.get("finished_at"):
            started = datetime.fromisoformat(run["started_at"])
            finished = datetime.fromisoformat(run["finished_at"])
            duration = finished - started

            lines.extend([
                "## Timeline",
                "",
                f"- **Started:** {run['started_at']}",
                f"- **Finished:** {run['finished_at']}",
                f"- **Duration:** {duration}",
                "",
            ])

        # Get findings breakdown from flagged packages
        severity_counts = {"critical": 0, "high": 0, "medium": 0, "low": 0, "info": 0}
        category_counts = {}
        
        for pkg in flagged:
            findings = self.store.get_findings_for_package(pkg["name"], pkg["version"])
            for f in findings:
                sev = f.get("severity", "info").lower()
                severity_counts[sev] = severity_counts.get(sev, 0) + 1
                
                cat = f.get("category", "unknown")
                category_counts[cat] = category_counts.get(cat, 0) + 1

        # Findings by severity
        lines.extend([
            "## Findings by Severity",
            "",
            "| Severity | Count |",
            "|----------|-------|",
        ])
        for sev in ["critical", "high", "medium", "low", "info"]:
            count = severity_counts.get(sev, 0)
            if count > 0:
                lines.append(f"| {sev.capitalize()} | {count} |")
        lines.append("")

        # Findings by category
        if category_counts:
            lines.extend([
                "## Findings by Category",
                "",
                "| Category | Count |",
                "|----------|-------|",
            ])
            for cat, count in sorted(category_counts.items(), key=lambda x: -x[1]):
                lines.append(f"| {cat} | {count} |")
            lines.append("")

        # Flagged packages detail
        if flagged:
            lines.extend([
                "## Flagged Packages",
                "",
                "| Package | Version | Findings | Max Severity |",
                "|---------|---------|----------|--------------|",
            ])
            
            for pkg in flagged[:50]:  # Limit to top 50
                findings = self.store.get_findings_for_package(pkg["name"], pkg["version"])
                max_sev = max(
                    (f.get("severity", "info").lower() for f in findings),
                    key=lambda s: ["info", "low", "medium", "high", "critical"].index(s)
                    if s in ["info", "low", "medium", "high", "critical"] else 0,
                    default="info",
                )
                lines.append(
                    f"| {pkg['name']} | {pkg['version']} | {pkg['finding_count']} | {max_sev.upper()} |"
                )
            
            if len(flagged) > 50:
                lines.append(f"\n*... and {len(flagged) - 50} more*")
            
            lines.append("")

        # LLM analyses
        llm_count = sum(
            1 for pkg in flagged
            if self.store.get_llm_analysis(pkg["name"], pkg["version"], pkg.get("tarball_sha256", ""))
        )
        
        lines.extend([
            "## LLM Analysis",
            "",
            f"- **Packages analyzed:** {llm_count}",
            f"- **Packages pending:** {len(flagged) - llm_count}",
            "",
        ])

        # Footer
        lines.extend([
            "---",
            "",
            f"*Generated by GlassWorm Harness v0.1.0*",
        ])

        return "\n".join(lines)

    def generate_json(self, run_id: str) -> Dict[str, Any]:
        """
        Generate a JSON report for a scan run.
        
        Args:
            run_id: Scan run ID
            
        Returns:
            JSON report dict
        """
        run = self.store.get_scan_run(run_id)
        if not run:
            return {"error": f"Run {run_id} not found"}

        flagged = self.store.get_flagged_packages(run_id)
        stats = self.store.get_stats()
        filter_params = json.loads(run["filter_params"]) if run["filter_params"] else {}

        # Build detailed package info
        packages = []
        for pkg in flagged:
            findings = self.store.get_findings_for_package(pkg["name"], pkg["version"])
            llm_analysis = self.store.get_llm_analysis(
                pkg["name"], pkg["version"], pkg.get("tarball_sha256", "")
            )
            
            packages.append({
                "name": pkg["name"],
                "version": pkg["version"],
                "finding_count": pkg["finding_count"],
                "findings": findings,
                "llm_analysis": llm_analysis,
                "scan_duration_ms": pkg.get("scan_duration_ms"),
            })

        report = {
            "run_id": run_id,
            "wave_id": run.get("wave_id", 0),
            "generated_at": datetime.utcnow().isoformat(),
            "summary": {
                "packages_scanned": run.get("packages_total", 0),
                "packages_flagged": run.get("packages_flagged", 0),
                "total_findings": sum(pkg["finding_count"] for pkg in packages),
            },
            "scan_parameters": filter_params,
            "glassware_version": run.get("glassware_version", "unknown"),
            "timeline": {
                "started_at": run.get("started_at"),
                "finished_at": run.get("finished_at"),
            },
            "findings_by_severity": self._count_by_severity(flagged),
            "findings_by_category": self._count_by_category(flagged),
            "packages": packages,
        }

        return report

    def _count_by_severity(self, packages: List[Dict]) -> Dict[str, int]:
        """Count findings by severity."""
        counts = {"critical": 0, "high": 0, "medium": 0, "low": 0, "info": 0}
        
        for pkg in packages:
            findings = self.store.get_findings_for_package(pkg["name"], pkg["version"])
            for f in findings:
                sev = f.get("severity", "info").lower()
                if sev in counts:
                    counts[sev] += 1
        
        return counts

    def _count_by_category(self, packages: List[Dict]) -> Dict[str, int]:
        """Count findings by category."""
        counts = {}
        
        for pkg in packages:
            findings = self.store.get_findings_for_package(pkg["name"], pkg["version"])
            for f in findings:
                cat = f.get("category", "unknown")
                counts[cat] = counts.get(cat, 0) + 1
        
        return counts

    def generate_wave_summary(self, wave_id: int) -> Dict[str, Any]:
        """
        Generate summary for a wave.
        
        Args:
            wave_id: Wave ID
            
        Returns:
            Wave summary dict
        """
        runs = self.store.get_wave_runs(wave_id)
        
        total_scanned = sum(r.get("packages_total", 0) for r in runs)
        total_flagged = sum(r.get("packages_flagged", 0) for r in runs)
        
        # Aggregate findings
        all_severity = {"critical": 0, "high": 0, "medium": 0, "low": 0, "info": 0}
        all_categories = {}
        
        for run in runs:
            flagged = self.store.get_flagged_packages(run["id"])
            for pkg in flagged:
                findings = self.store.get_findings_for_package(pkg["name"], pkg["version"])
                for f in findings:
                    sev = f.get("severity", "info").lower()
                    if sev in all_severity:
                        all_severity[sev] += 1
                    
                    cat = f.get("category", "unknown")
                    all_categories[cat] = all_categories.get(cat, 0) + 1

        return {
            "wave_id": wave_id,
            "generated_at": datetime.utcnow().isoformat(),
            "runs": len(runs),
            "packages_scanned": total_scanned,
            "packages_flagged": total_flagged,
            "detection_rate": (total_flagged / total_scanned * 100) if total_scanned > 0 else 0,
            "findings_by_severity": all_severity,
            "findings_by_category": all_categories,
            "run_ids": [r["id"] for r in runs],
        }

    def save_report(self, run_id: str, output_dir: Optional[Path] = None) -> Dict[str, Path]:
        """
        Generate and save reports.
        
        Args:
            run_id: Scan run ID
            output_dir: Output directory (defaults to reports/)
            
        Returns:
            Dict with paths to generated files
        """
        output_dir = output_dir or REPORTS_DIR
        output_dir.mkdir(parents=True, exist_ok=True)
        
        # Generate markdown
        md_content = self.generate_markdown(run_id)
        md_path = output_dir / f"scan-{run_id[:8]}.md"
        md_path.write_text(md_content)
        
        # Generate JSON
        json_content = self.generate_json(run_id)
        json_path = output_dir / f"scan-{run_id[:8]}.json"
        json_path.write_text(json.dumps(json_content, indent=2))
        
        return {"markdown": md_path, "json": json_path}
