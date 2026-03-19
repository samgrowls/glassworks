"""
SQLite Database Module

Manages the scan corpus database with tables for:
- scan_runs: Metadata about each scanning session
- packages: Package information and scan results
- findings: Individual security findings
"""

import json
import sqlite3
from contextlib import contextmanager
from dataclasses import asdict
from datetime import datetime
from pathlib import Path
from typing import Optional
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
            -- Scan runs table
            CREATE TABLE IF NOT EXISTS scan_runs (
                id              TEXT PRIMARY KEY,
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

            -- Indexes for common queries
            CREATE INDEX IF NOT EXISTS idx_packages_run ON packages(scan_run_id);
            CREATE INDEX IF NOT EXISTS idx_packages_flagged
                ON packages(scan_run_id, finding_count)
                WHERE finding_count > 0;
            CREATE INDEX IF NOT EXISTS idx_findings_package ON findings(package_id);
            CREATE INDEX IF NOT EXISTS idx_findings_severity ON findings(severity);
            CREATE INDEX IF NOT EXISTS idx_findings_triage ON findings(triage_status);
            
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
            
            -- Index for faster LLM cache lookups
            CREATE INDEX IF NOT EXISTS idx_llm_sha256 ON llm_analyses(tarball_sha256);
        """)
        conn.commit()


class Database:
    """High-level database interface for the scanning harness."""

    def __init__(self, db_path: Path):
        self.db_path = db_path
        if not db_path.parent.exists():
            db_path.parent.mkdir(parents=True)
        init_database(db_path)

    def create_scan_run(
        self,
        filter_params: dict,
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
                (id, started_at, filter_params, glassware_version, notes)
                VALUES (?, ?, ?, ?, ?)
            """,
                (
                    run_id,
                    started_at,
                    json.dumps(filter_params),
                    glassware_version,
                    notes,
                ),
            )
            conn.commit()

        return run_id

    def finalize_scan_run(
        self, run_id: str, packages_total: int, packages_flagged: int
    ) -> None:
        """Mark a scan run as finished with summary stats."""
        finished_at = datetime.utcnow().isoformat()

        with get_connection(self.db_path) as conn:
            conn.execute(
                """
                UPDATE scan_runs 
                SET finished_at = ?, packages_total = ?, packages_flagged = ?
                WHERE id = ?
            """,
                (finished_at, packages_total, packages_flagged, run_id),
            )
            conn.commit()

    def is_already_scanned(
        self,
        name: str,
        version: str,
        tarball_sha256: str,
        max_age_days: int = 7,
    ) -> bool:
        """
        Check if package with same hash was already scanned recently.
        
        Args:
            name: Package name
            version: Package version
            tarball_sha256: SHA256 hash of tarball
            max_age_days: Consider scans older than this as stale (default: 7 days)
        
        Returns:
            True if package was already scanned with same hash, False otherwise
        """
        from datetime import datetime, timedelta
        
        cutoff = (datetime.utcnow() - timedelta(days=max_age_days)).isoformat()
        
        with get_connection(self.db_path) as conn:
            cursor = conn.execute(
                """
                SELECT id, scanned_at, finding_count
                FROM packages
                WHERE name = ? AND version = ? AND tarball_sha256 = ?
                AND scanned_at > ?
                LIMIT 1
            """,
                (name, version, tarball_sha256, cutoff),
            )
            row = cursor.fetchone()
            return row is not None
    
    def get_cached_scan_result(
        self,
        name: str,
        version: str,
        tarball_sha256: str,
    ) -> Optional[dict]:
        """
        Get cached scan result for package.
        
        Returns:
            dict with finding_count, scan_duration_ms, vault_path or None if not found
        """
        with get_connection(self.db_path) as conn:
            cursor = conn.execute(
                """
                SELECT finding_count, scan_duration_ms, vault_path, scanned_at
                FROM packages
                WHERE name = ? AND version = ? AND tarball_sha256 = ?
                ORDER BY scanned_at DESC
                LIMIT 1
            """,
                (name, version, tarball_sha256),
            )
            row = cursor.fetchone()
            if row:
                return {
                    "finding_count": row[0],
                    "scan_duration_ms": row[1],
                    "vault_path": row[2],
                    "scanned_at": row[3],
                }
            return None
    
    def add_llm_analysis(
        self,
        package_name: str,
        package_version: str,
        tarball_sha256: str,
        analysis_result: dict,
    ) -> int:
        """Add LLM analysis result to cache"""
        with get_connection(self.db_path) as conn:
            cursor = conn.execute(
                """
                INSERT INTO llm_analyses
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
            return cursor.lastrowid
    
    def get_cached_llm_analysis(
        self,
        package_name: str,
        package_version: str,
        tarball_sha256: str,
        max_age_days: int = 7,
    ) -> Optional[dict]:
        """
        Get cached LLM analysis for package.
        
        Args:
            package_name: Package name
            package_version: Package version
            tarball_sha256: SHA256 hash of tarball
            max_age_days: Consider analyses older than this as stale (default: 7 days)
        
        Returns:
            dict with analysis result or None if not found/cached
        """
        from datetime import datetime, timedelta
        
        cutoff = (datetime.utcnow() - timedelta(days=max_age_days)).isoformat()
        
        with get_connection(self.db_path) as conn:
            cursor = conn.execute(
                """
                SELECT analysis_result, analyzed_at
                FROM llm_analyses
                WHERE package_name = ? AND package_version = ? AND tarball_sha256 = ?
                AND analyzed_at > ?
                ORDER BY analyzed_at DESC
                LIMIT 1
            """,
                (package_name, package_version, tarball_sha256, cutoff),
            )
            row = cursor.fetchone()
            if row:
                return {
                    "analysis": json.loads(row[0]),
                    "analyzed_at": row[1],
                }
            return None

    def add_package(
        self,
        run_id: str,
        name: str,
        version: str,
        tarball_url: str,
        tarball_sha256: str,
        published_at: Optional[str] = None,
        author: Optional[str] = None,
        downloads_weekly: int = 0,
        has_install_scripts: bool = False,
    ) -> int:
        """Add a package record and return its ID."""
        scanned_at = datetime.utcnow().isoformat()

        with get_connection(self.db_path) as conn:
            cursor = conn.execute(
                """
                INSERT INTO packages 
                (scan_run_id, name, version, published_at, author, 
                 downloads_weekly, has_install_scripts, tarball_url, 
                 tarball_sha256, scanned_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
                (
                    run_id,
                    name,
                    version,
                    published_at,
                    author,
                    downloads_weekly,
                    has_install_scripts,
                    tarball_url,
                    tarball_sha256,
                    scanned_at,
                ),
            )
            conn.commit()
            return cursor.lastrowid

    def update_package_scan(
        self,
        package_id: int,
        finding_count: int,
        scan_duration_ms: int,
        glassware_output: str,
        vault_path: Optional[str] = None,
    ) -> None:
        """Update a package with scan results."""
        with get_connection(self.db_path) as conn:
            conn.execute(
                """
                UPDATE packages 
                SET finding_count = ?, scan_duration_ms = ?, 
                    glassware_output = ?, vault_path = ?
                WHERE id = ?
            """,
                (finding_count, scan_duration_ms, glassware_output, vault_path, package_id),
            )
            conn.commit()

    def add_finding(
        self,
        package_id: int,
        file_path: str,
        line: Optional[int],
        column: Optional[int],
        rule_id: Optional[str],
        category: str,
        severity: str,
        message: str,
        confidence: Optional[float] = None,
        decoded_payload: Optional[dict] = None,
        raw_json: Optional[str] = None,
    ) -> int:
        """Add a finding record and return its ID."""
        with get_connection(self.db_path) as conn:
            cursor = conn.execute(
                """
                INSERT INTO findings 
                (package_id, file_path, line, column, rule_id, category, 
                 severity, message, confidence, decoded_payload, raw_json)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
                (
                    package_id,
                    file_path,
                    line,
                    column,
                    rule_id,
                    category,
                    severity,
                    message,
                    confidence,
                    json.dumps(decoded_payload) if decoded_payload else None,
                    raw_json,
                ),
            )
            conn.commit()
            return cursor.lastrowid

    def add_llm_verdict(
        self,
        finding_id: int,
        is_malicious: bool,
        confidence: float,
        reasoning: str,
        reclassified_severity: Optional[str] = None,
    ) -> None:
        """Update a finding with LLM analysis results."""
        with get_connection(self.db_path) as conn:
            conn.execute(
                """
                UPDATE findings 
                SET llm_verdict = ?, llm_confidence = ?, triage_status = 'llm_reviewed'
                WHERE id = ?
            """,
                (
                    json.dumps(
                        {
                            "is_malicious": is_malicious,
                            "confidence": confidence,
                            "reasoning": reasoning,
                            "reclassified_severity": reclassified_severity,
                        }
                    ),
                    confidence,
                    finding_id,
                ),
            )
            conn.commit()

    def get_flagged_packages(self, run_id: str) -> list[sqlite3.Row]:
        """Get all packages with findings from a scan run."""
        with get_connection(self.db_path) as conn:
            cursor = conn.execute(
                """
                SELECT p.*, MAX(f.severity) as max_severity
                FROM packages p
                LEFT JOIN findings f ON p.id = f.package_id
                WHERE p.scan_run_id = ? AND p.finding_count > 0
                GROUP BY p.id
                ORDER BY p.finding_count DESC
            """,
                (run_id,),
            )
            return cursor.fetchall()

    def get_findings_for_package(self, package_id: int) -> list[sqlite3.Row]:
        """Get all findings for a package."""
        with get_connection(self.db_path) as conn:
            cursor = conn.execute(
                "SELECT * FROM findings WHERE package_id = ? ORDER BY severity DESC",
                (package_id,),
            )
            return cursor.fetchall()

    def get_scan_run(self, run_id: str) -> Optional[sqlite3.Row]:
        """Get a scan run by ID."""
        with get_connection(self.db_path) as conn:
            cursor = conn.execute(
                "SELECT * FROM scan_runs WHERE id = ?", (run_id,)
            )
            return cursor.fetchone()

    def get_all_scan_runs(self) -> list[sqlite3.Row]:
        """Get all scan runs, most recent first."""
        with get_connection(self.db_path) as conn:
            cursor = conn.execute(
                "SELECT * FROM scan_runs ORDER BY started_at DESC"
            )
            return cursor.fetchall()

    def get_statistics(self) -> dict:
        """Get overall corpus statistics."""
        with get_connection(self.db_path) as conn:
            stats = {}

            # Total packages
            cursor = conn.execute("SELECT COUNT(*) FROM packages")
            stats["total_packages"] = cursor.fetchone()[0]

            # Flagged packages
            cursor = conn.execute(
                "SELECT COUNT(*) FROM packages WHERE finding_count > 0"
            )
            stats["flagged_packages"] = cursor.fetchone()[0]

            # Total findings
            cursor = conn.execute("SELECT COUNT(*) FROM findings")
            stats["total_findings"] = cursor.fetchone()[0]

            # Findings by severity
            cursor = conn.execute(
                "SELECT severity, COUNT(*) FROM findings GROUP BY severity"
            )
            stats["by_severity"] = {
                row[0]: row[1] for row in cursor.fetchall()
            }

            # Findings by category
            cursor = conn.execute(
                "SELECT category, COUNT(*) FROM findings GROUP BY category"
            )
            stats["by_category"] = {
                row[0]: row[1] for row in cursor.fetchall()
            }

            return stats

    def get_untriaged_findings(self) -> list[sqlite3.Row]:
        """Get findings that haven't been triaged."""
        with get_connection(self.db_path) as conn:
            cursor = conn.execute(
                """
                SELECT f.*, p.name, p.version 
                FROM findings f
                JOIN packages p ON f.package_id = p.id
                WHERE f.triage_status = 'untriaged'
                ORDER BY f.severity DESC
            """
            )
            return cursor.fetchall()
