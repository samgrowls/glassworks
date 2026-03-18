"""
Report Generator

Generates markdown reports for scan runs.
"""

import json
from datetime import datetime
from pathlib import Path

from rich.console import Console

from database import Database

console = Console()

REPORTS_DIR = Path(__file__).parent / "reports"


def generate_report(db: Database, run_id: str) -> str:
    """Generate a markdown report for a scan run."""
    run = db.get_scan_run(run_id)
    if not run:
        return f"Run {run_id} not found"

    flagged = db.get_flagged_packages(run_id)
    stats = db.get_statistics()

    # Parse filter params
    filter_params = json.loads(run["filter_params"]) if run["filter_params"] else {}

    # Build report
    lines = [
        f"# glassware Scan Report",
        "",
        f"**Run ID:** `{run_id}`",
        f"**Generated:** {datetime.utcnow().isoformat()}",
        "",
        "## Summary",
        "",
        f"| Metric | Value |",
        f"|--------|-------|",
        f"| Packages scanned | {run['packages_total']} |",
        f"| Packages flagged | {run['packages_flagged']} |",
        f"| Detection rate | {run['packages_flagged']/run['packages_total']*100:.1f}% |" if run['packages_total'] > 0 else "| Detection rate | N/A |",
        "",
        "## Scan Parameters",
        "",
        f"- **Days back:** {filter_params.get('days_back', 'N/A')}",
        f"- **Download threshold:** < {filter_params.get('download_threshold', 'N/A')} weekly",
        f"- **Tier:** {filter_params.get('tier', 'N/A')}",
        f"- **Glassware version:** {run.get('glassware_version', 'unknown')}",
        "",
    ]

    # Timeline
    if run["started_at"] and run["finished_at"]:
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

    # Findings breakdown
    if stats["by_severity"]:
        lines.extend([
            "## Findings by Severity",
            "",
            "| Severity | Count |",
            "|----------|-------|",
        ])
        for sev in ["critical", "high", "medium", "low", "info"]:
            count = stats["by_severity"].get(sev, 0)
            if count > 0:
                lines.append(f"| {sev.capitalize()} | {count} |")
        lines.append("")

    if stats["by_category"]:
        lines.extend([
            "## Findings by Category",
            "",
            "| Category | Count |",
            "|----------|-------|",
        ])
        for cat, count in sorted(stats["by_category"].items(), key=lambda x: -x[1]):
            lines.append(f"| {cat} | {count} |")
        lines.append("")

    # Flagged packages detail
    if flagged:
        lines.extend([
            "## Flagged Packages",
            "",
            "### Highest Priority (Install Scripts + Findings)",
            "",
        ])

        for pkg in flagged:
            findings = db.get_findings_for_package(pkg["id"])

            # Get max severity
            severities = [f["severity"] for f in findings if f["severity"]]
            max_sev = max(
                severities,
                key=lambda s: ["critical", "high", "medium", "low", "info"].index(s)
                if s in ["critical", "high", "medium", "low", "info"]
                else -1,
                default="unknown",
            )

            has_scripts = pkg["has_install_scripts"]
            priority_marker = "🔴" if has_scripts else "🟡"

            lines.extend([
                f"### {priority_marker} {pkg['name']}@{pkg['version']}",
                "",
                f"- **Published:** {pkg['published_at'] or 'Unknown'}",
                f"- **Author:** {pkg['author'] or 'Unknown'}",
                f"- **Weekly downloads:** {pkg['downloads_weekly']}",
                f"- **Has install scripts:** {'Yes' if has_scripts else 'No'}",
                f"- **Finding count:** {pkg['finding_count']}",
                f"- **Max severity:** {max_sev}",
                f"- **Vault:** `{pkg['vault_path'] or 'Not archived'}`",
                "",
                "**Findings:**",
                "",
            ])

            for i, finding in enumerate(findings[:10], 1):  # Show first 10
                lines.extend([
                    f"{i}. **[{finding['severity']}]** `{finding['file_path']}` (line {finding['line'] or '?'})",
                    f"   - {finding['message']}",
                    f"   - Category: `{finding['category']}`",
                    "",
                ])

            if len(findings) > 10:
                lines.append(f"*...and {len(findings) - 10} more findings*")
                lines.append("")

    # Recommendations
    lines.extend([
        "## Recommendations",
        "",
    ])

    high_priority = [p for p in flagged if p["has_install_scripts"]]
    if high_priority:
        lines.append("### Immediate Action Required")
        lines.append("")
        lines.append("The following packages have install scripts AND security findings. ")
        lines.append("These should be reported to npm Security immediately:")
        lines.append("")
        for pkg in high_priority[:5]:
            lines.append(f"- `{pkg['name']}@{pkg['version']}`")
        lines.append("")

    # Disclosure instructions
    lines.extend([
        "## Disclosure",
        "",
        "Per our responsible disclosure policy:",
        "",
        "1. Report findings to npm Security (security@npmjs.com)",
        "2. Wait for package removal before public disclosure",
        "3. Include this report ID in disclosure communications",
        "",
        f"**Report ID:** `{run_id}`",
        "",
    ])

    return "\n".join(lines)


def print_report(db: Database, run_id: str) -> None:
    """Print a report summary to console."""
    run = db.get_scan_run(run_id)
    if not run:
        console.print(f"[red]Run {run_id} not found[/red]")
        return

    console.print(f"\n[bold]Scan Report: {run_id}[/bold]")
    console.print(f"Packages: {run['packages_total']} total, {run['packages_flagged']} flagged")

    flagged = db.get_flagged_packages(run_id)
    if flagged:
        console.print("\n[bold red]Flagged packages:[/bold red]")
        for pkg in flagged[:5]:
            console.print(f"  - {pkg['name']}@{pkg['version']} ({pkg['finding_count']} findings)")
        if len(flagged) > 5:
            console.print(f"  ...and {len(flagged) - 5} more")


def save_report(db: Database, run_id: str) -> Path:
    """Generate and save a report to the reports directory."""
    markdown = generate_report(db, run_id)

    report_path = REPORTS_DIR / f"run-{run_id[:8]}.md"
    report_path.write_text(markdown)

    return report_path


def main():
    """CLI for generating reports."""
    import argparse

    parser = argparse.ArgumentParser(description="Generate scan reports")
    parser.add_argument("run_id", help="Scan run ID")
    parser.add_argument("--save", action="store_true", help="Save report to file")
    args = parser.parse_args()

    db = Database(Path(__file__).parent / "data" / "corpus.db")

    if args.save:
        path = save_report(db, args.run_id)
        console.print(f"[green]Report saved to: {path}[/green]")
    else:
        print_report(db, args.run_id)


if __name__ == "__main__":
    main()
