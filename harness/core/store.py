"""
Store - SQLite Database with Checkpoint/Resume Support

Consolidated from database.py and background_scanner.py.

Schema:
- scan_runs: Metadata about scanning sessions (includes wave_id)
- packages: Package information and scan results
- findings: Individual security findings
- llm_analyses: LLM analysis cache
- checkpoints: Checkpoint/resume state

Usage:
    from harness.core import Store
    
    store = Store("data/corpus.db")
    run_id = store.create_scan_run(wave_id=0, filter_params={"categories": ["ai-ml"]})
    store.save_package_scan(run_id, pkg_name, version, findings)
"""

import json
import sqlite3
from contextlib import contextmanager
from datetime import datetime
from pathlib import Path
from typing import Optional, Dict, Any, List
from uuid import uuid4


@contextmanager
def get_connection(db_path: Path):
    """Get a database connection with proper settings."""
    conn = sqlite3.connect(str(db_path))
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA journal_mode=WAL")
    conn.execute("PRAGMA synchronous=NORMAL")
    try:
        yield conn
    finally:
        conn.close()


def init_database(db_path: Path) -> None:
    """Initialize the database schema."""
    with get_connection(db_path) as conn:
        conn.executescript("""
            -- Scan runs table (with wave_id)
            CREATE TABLE IF NOT EXISTS scan_runs (
                id              TEXT PRIMARY KEY,
                wave_id         INTEGER DEFAULT 0,
                started_at      TEXT NOT NULL,
                finished_at     TEXT,
                filter_params   TEXT,               -- JSON blob of selection criteria
                packages_total  INTEGER DEFAULT 0,
                packages_flagged INTEGER DEFAULT 0,
                glassware_version TEXT,
                notes           TEXT
            );

            -- Packages table
            CREATE TABLE IF NOT EXISTS packages (
                id              INTEGER PRIMARY KEY,
                name            TEXT NOT NULL,
                version         TEXT NOT NULL,
                published_at    TEXT,
                author          TEXT,
                downloads_weekly INTEGER,
                has_install_scripts BOOLEAN,
                tarball_url     TEXT,
                tarball_sha256  TEXT,
                scanned_at      TEXT NOT NULL,
                scan_run_id     TEXT NOT NULL REFERENCES scan_runs(id),
                finding_count   INTEGER DEFAULT 0,
                scan_duration_ms INTEGER,
                glassware_output TEXT,  -- Raw JSON output from glassware
                vault_path      TEXT,  -- Path to archived tarball if flagged
                UNIQUE(name, version)
            );

            -- Findings table
            CREATE TABLE IF NOT EXISTS findings (
                id              INTEGER PRIMARY KEY,
                package_id      INTEGER NOT NULL REFERENCES packages(id),
                file_path       TEXT NOT NULL,
                line            INTEGER,
                column          INTEGER,
                rule_id         TEXT,
                category        TEXT,
                severity        TEXT,
                message         TEXT,
                confidence      REAL,
                decoded_payload TEXT,  -- JSON blob if present
                raw_json        TEXT,  -- Full finding JSON from glassware
                triage_status   TEXT DEFAULT 'untriaged',
                triage_notes    TEXT,
                llm_verdict     TEXT,  -- LLM analysis result if available
                llm_confidence  REAL
            );

            -- LLM analyses cache
            CREATE TABLE IF NOT EXISTS llm_analyses (
                id              INTEGER PRIMARY KEY,
                package_name    TEXT NOT NULL,
                package_version TEXT NOT NULL,
                tarball_sha256  TEXT NOT NULL,
                analysis_result TEXT NOT NULL,  -- JSON blob
                analyzed_at     TEXT NOT NULL,
                UNIQUE(package_name, package_version, tarball_sha256)
            );

            -- Checkpoints for resume support
            CREATE TABLE IF NOT EXISTS checkpoints (
                id              INTEGER PRIMARY KEY,
                scan_run_id     TEXT NOT NULL REFERENCES scan_runs(id),
                package_name    TEXT NOT NULL,
                version         TEXT NOT NULL,
                status          TEXT NOT NULL,  -- 'pending', 'scanned', 'failed'
                checkpoint_at   TEXT NOT NULL,
                UNIQUE(scan_run_id, package_name, version)
            );

            -- Indexes for common queries
            CREATE INDEX IF NOT EXISTS idx_packages_run ON packages(scan_run_id);
            CREATE INDEX IF NOT EXISTS idx_packages_flagged
                ON packages(scan_run_id, finding_count)
                WHERE finding_count > 0;
            CREATE INDEX IF NOT EXISTS idx_findings_package ON findings(package_id);
            CREATE INDEX IF NOT EXISTS idx_findings_severity ON findings(severity);
            CREATE INDEX IF NOT EXISTS idx_findings_triage ON findings(triage_status);
            CREATE INDEX IF NOT EXISTS idx_llm_sha256 ON llm_analyses(tarball_sha256);
            CREATE INDEX IF NOT EXISTS idx_checkpoints_run ON checkpoints(scan_run_id);
        """)
        conn.commit()


class Store:
    """
    High-level database interface for the scanning harness.
    
    Consolidates functionality from database.py and background_scanner.py.
    Adds wave_id column and checkpoint/resume support.
    """

    def __init__(self, db_path: Path):
        self.db_path = Path(db_path)
        if not self.db_path.parent.exists():
            self.db_path.parent.mkdir(parents=True)
        init_database(self.db_path)

    # === Scan Runs ===

    def create_scan_run(
        self,
        wave_id: int = 0,
        filter_params: Optional[Dict] = None,
        glassware_version: str = "unknown",
        notes: str = "",
    ) -> str:
        """Create a new scan run record and return its ID."""
        run_id = str(uuid4())
        started_at = datetime.utcnow().isoformat()

        with get_connection(self.db_path) as conn:
            conn.execute(
                """
                INSERT INTO scan_runs
                (id, wave_id, started_at, filter_params, glassware_version, notes)
                VALUES (?, ?, ?, ?, ?, ?)
            """,
                (
                    run_id,
                    wave_id,
                    started_at,
                    json.dumps(filter_params or {}),
                    glassware_version,
                    notes,
                ),
            )
            conn.commit()

        return run_id

    def finish_scan_run(self, run_id: str) -> None:
        """Mark a scan run as finished."""
        with get_connection(self.db_path) as conn:
            conn.execute(
                "UPDATE scan_runs SET finished_at = ? WHERE id = ?",
                (datetime.utcnow().isoformat(), run_id),
            )
            conn.commit()

    def get_scan_run(self, run_id: str) -> Optional[Dict]:
        """Get a scan run by ID."""
        with get_connection(self.db_path) as conn:
            row = conn.execute(
                "SELECT * FROM scan_runs WHERE id = ?", (run_id,)
            ).fetchone()
            if row:
                return dict(row)
        return None

    def get_wave_runs(self, wave_id: int) -> List[Dict]:
        """Get all scan runs for a wave."""
        with get_connection(self.db_path) as conn:
            rows = conn.execute(
                "SELECT * FROM scan_runs WHERE wave_id = ? ORDER BY started_at DESC",
                (wave_id,),
            ).fetchall()
            return [dict(row) for row in rows]

    # === Packages ===

    def save_package_scan(
        self,
        run_id: str,
        name: str,
        version: str,
        findings: List[Dict],
        glassware_output: str = "",
        scan_duration_ms: int = 0,
        tarball_sha256: str = "",
        vault_path: str = "",
    ) -> int:
        """
        Save a package scan result.
        Returns the package ID.
        """
        finding_count = len(findings)
        scanned_at = datetime.utcnow().isoformat()

        with get_connection(self.db_path) as conn:
            # Insert or replace package
            conn.execute(
                """
                INSERT OR REPLACE INTO packages
                (name, version, scanned_at, scan_run_id, finding_count,
                 scan_duration_ms, glassware_output, tarball_sha256, vault_path)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
                (
                    name,
                    version,
                    scanned_at,
                    run_id,
                    finding_count,
                    scan_duration_ms,
                    glassware_output,
                    tarball_sha256,
                    vault_path,
                ),
            )

            # Get package ID
            pkg_id = conn.execute(
                "SELECT id FROM packages WHERE name = ? AND version = ?",
                (name, version),
            ).fetchone()[0]

            # Save findings
            for finding in findings:
                conn.execute(
                    """
                    INSERT INTO findings
                    (package_id, file_path, line, column, rule_id, category,
                     severity, message, confidence, decoded_payload, raw_json)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                    (
                        pkg_id,
                        finding.get("file", ""),
                        finding.get("line", 0),
                        finding.get("column", 0),
                        finding.get("category", ""),
                        finding.get("category", ""),
                        finding.get("severity", "info"),
                        finding.get("description", ""),
                        finding.get("confidence"),
                        json.dumps(finding.get("decoded_payload"))
                        if finding.get("decoded_payload")
                        else None,
                        json.dumps(finding),
                    ),
                )

            # Update checkpoint
            conn.execute(
                """
                INSERT OR REPLACE INTO checkpoints
                (scan_run_id, package_name, version, status, checkpoint_at)
                VALUES (?, ?, ?, ?, ?)
            """,
                (run_id, name, version, "scanned", scanned_at),
            )

            # Update run stats
            if finding_count > 0:
                conn.execute(
                    """
                    UPDATE scan_runs
                    SET packages_flagged = packages_flagged + 1
                    WHERE id = ?
                """,
                    (run_id,),
                )

            conn.commit()

        return pkg_id

    def get_package(self, name: str, version: str) -> Optional[Dict]:
        """Get a package by name and version."""
        with get_connection(self.db_path) as conn:
            row = conn.execute(
                "SELECT * FROM packages WHERE name = ? AND version = ?",
                (name, version),
            ).fetchone()
            if row:
                return dict(row)
        return None

    def get_flagged_packages(self, run_id: Optional[str] = None) -> List[Dict]:
        """Get all packages with findings."""
        with get_connection(self.db_path) as conn:
            if run_id:
                rows = conn.execute(
                    """
                    SELECT * FROM packages
                    WHERE scan_run_id = ? AND finding_count > 0
                    ORDER BY finding_count DESC
                """,
                    (run_id,),
                ).fetchall()
            else:
                rows = conn.execute(
                    """
                    SELECT * FROM packages
                    WHERE finding_count > 0
                    ORDER BY scanned_at DESC
                """
                ).fetchall()
            return [dict(row) for row in rows]

    def get_findings_for_package(
        self, name: str, version: str, severity: Optional[str] = None
    ) -> List[Dict]:
        """Get findings for a specific package."""
        with get_connection(self.db_path) as conn:
            if severity:
                rows = conn.execute(
                    """
                    SELECT f.* FROM findings f
                    JOIN packages p ON f.package_id = p.id
                    WHERE p.name = ? AND p.version = ? AND f.severity = ?
                """,
                    (name, version, severity),
                ).fetchall()
            else:
                rows = conn.execute(
                    """
                    SELECT f.* FROM findings f
                    JOIN packages p ON f.package_id = p.id
                    WHERE p.name = ? AND p.version = ?
                """,
                    (name, version),
                ).fetchall()
            return [dict(row) for row in rows]

    # === Checkpoints ===

    def save_checkpoint(
        self, run_id: str, package_name: str, version: str, status: str
    ) -> None:
        """Save a checkpoint for resume support."""
        with get_connection(self.db_path) as conn:
            conn.execute(
                """
                INSERT OR REPLACE INTO checkpoints
                (scan_run_id, package_name, version, status, checkpoint_at)
                VALUES (?, ?, ?, ?, ?)
            """,
                (run_id, package_name, version, status, datetime.utcnow().isoformat()),
            )
            conn.commit()

    def get_pending_packages(self, run_id: str, package_list: List[str]) -> List[str]:
        """Get packages from list that haven't been scanned yet."""
        with get_connection(self.db_path) as conn:
            placeholders = ",".join("?" * len(package_list))
            rows = conn.execute(
                f"""
                SELECT name || '@' || version as pkg
                FROM checkpoints
                WHERE scan_run_id = ? AND status = 'scanned'
            """,
                (run_id,),
            ).fetchall()
            scanned = {row[0] for row in rows}

        return [pkg for pkg in package_list if pkg not in scanned]

    def get_run_progress(self, run_id: str) -> Dict[str, int]:
        """Get progress stats for a run."""
        with get_connection(self.db_path) as conn:
            total = conn.execute(
                "SELECT COUNT(*) FROM checkpoints WHERE scan_run_id = ?", (run_id,)
            ).fetchone()[0]
            scanned = conn.execute(
                "SELECT COUNT(*) FROM checkpoints WHERE scan_run_id = ? AND status = 'scanned'",
                (run_id,),
            ).fetchone()[0]
            failed = conn.execute(
                "SELECT COUNT(*) FROM checkpoints WHERE scan_run_id = ? AND status = 'failed'",
                (run_id,),
            ).fetchone()[0]

        return {"total": total, "scanned": scanned, "failed": failed, "pending": total - scanned - failed}

    # === LLM Analyses ===

    def save_llm_analysis(
        self,
        package_name: str,
        package_version: str,
        tarball_sha256: str,
        analysis_result: Dict,
    ) -> None:
        """Save LLM analysis result."""
        with get_connection(self.db_path) as conn:
            conn.execute(
                """
                INSERT OR REPLACE INTO llm_analyses
                (package_name, package_version, tarball_sha256, analysis_result, analyzed_at)
                VALUES (?, ?, ?, ?, ?)
            """,
                (
                    package_name,
                    package_version,
                    tarball_sha256,
                    json.dumps(analysis_result),
                    datetime.utcnow().isoformat(),
                ),
            )
            conn.commit()

    def get_llm_analysis(
        self, package_name: str, package_version: str, tarball_sha256: str
    ) -> Optional[Dict]:
        """Get cached LLM analysis."""
        with get_connection(self.db_path) as conn:
            row = conn.execute(
                """
                SELECT analysis_result FROM llm_analyses
                WHERE package_name = ? AND package_version = ? AND tarball_sha256 = ?
            """,
                (package_name, package_version, tarball_sha256),
            ).fetchone()
            if row:
                return json.loads(row[0])
        return None

    def get_flagged_for_llm(self, min_severity: str = "high") -> List[Dict]:
        """Get packages flagged for LLM analysis."""
        with get_connection(self.db_path) as conn:
            rows = conn.execute(
                """
                SELECT p.*, l.analysis_result
                FROM packages p
                LEFT JOIN llm_analyses l
                    ON p.name = l.package_name AND p.version = l.package_version
                    AND p.tarball_sha256 = l.tarball_sha256
                WHERE p.finding_count > 0
                AND l.analysis_result IS NULL
                ORDER BY p.finding_count DESC
            """
            ).fetchall()
            return [dict(row) for row in rows]

    # === Stats ===

    def get_stats(self) -> Dict[str, Any]:
        """Get overall scan statistics."""
        with get_connection(self.db_path) as conn:
            total_runs = conn.execute("SELECT COUNT(*) FROM scan_runs").fetchone()[0]
            total_packages = conn.execute("SELECT COUNT(*) FROM packages").fetchone()[0]
            flagged_packages = conn.execute(
                "SELECT COUNT(*) FROM packages WHERE finding_count > 0"
            ).fetchone()[0]
            total_findings = conn.execute("SELECT COUNT(*) FROM findings").fetchone()[0]
            llm_analyses = conn.execute("SELECT COUNT(*) FROM llm_analyses").fetchone()[0]

        return {
            "total_runs": total_runs,
            "total_packages": total_packages,
            "flagged_packages": flagged_packages,
            "total_findings": total_findings,
            "llm_analyses": llm_analyses,
        }
