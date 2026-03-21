"""
Fetcher - npm Package Download and Caching

Consolidated from optimized_scanner.py, selector.py, and diverse_sampling.py.

Features:
- npm registry API queries
- Package download with SHA256 caching
- Category-based sampling
- Rate limiting

Usage:
    from harness.core import Fetcher
    
    fetcher = Fetcher()
    pkg_info = fetcher.get_package_info("express")
    tarball_path = fetcher.download_package("express@4.19.2")
    packages = fetcher.sample_packages(categories=["ai-ml"], samples_per_category=50)
"""

import hashlib
import json
import sqlite3
import subprocess
import tempfile
from datetime import datetime, timedelta
from pathlib import Path
from typing import Optional, Dict, List, Any
import requests

# Configuration
NPM_REGISTRY = "https://registry.npmjs.org"
NPM_SEARCH = "https://registry.npmjs.org/-/v1/search"
CACHE_DB = Path("/tmp/glassware-fetch-cache.db")
DEFAULT_RATE_LIMIT = 5.0  # requests per second


class Fetcher:
    """
    npm package fetcher with caching and rate limiting.
    
    Consolidates functionality from optimized_scanner.py, selector.py,
    and diverse_sampling.py.
    """

    def __init__(self, rate_limit: float = DEFAULT_RATE_LIMIT):
        self.rate_limit = rate_limit
        self.last_request_time: Optional[datetime] = None
        self._init_cache()

    def _init_cache(self):
        """Initialize download cache database."""
        CACHE_DB.parent.mkdir(parents=True, exist_ok=True)
        conn = sqlite3.connect(str(CACHE_DB))
        conn.execute("""
            CREATE TABLE IF NOT EXISTS downloads (
                package         TEXT NOT NULL,
                version         TEXT NOT NULL,
                tarball_path    TEXT NOT NULL,
                tarball_sha256  TEXT NOT NULL,
                downloaded_at   TEXT NOT NULL,
                PRIMARY KEY (package, version)
            )
        """)
        conn.commit()
        conn.close()

    def _rate_limit(self):
        """Apply rate limiting between requests."""
        if self.last_request_time:
            elapsed = (datetime.now() - self.last_request_time).total_seconds()
            min_interval = 1.0 / self.rate_limit
            if elapsed < min_interval:
                import time
                time.sleep(min_interval - elapsed)
        self.last_request_time = datetime.now()

    def _sha256_file(self, path: Path) -> str:
        """Calculate SHA256 hash of a file."""
        sha256 = hashlib.sha256()
        with open(path, "rb") as f:
            for chunk in iter(lambda: f.read(8192), b""):
                sha256.update(chunk)
        return sha256.hexdigest()

    def _get_cached_download(self, package: str, version: str) -> Optional[Dict]:
        """Get cached download info."""
        conn = sqlite3.connect(str(CACHE_DB))
        conn.row_factory = sqlite3.Row
        row = conn.execute(
            """
            SELECT * FROM downloads
            WHERE package = ? AND version = ?
            """,
            (package, version),
        ).fetchone()
        conn.close()

        if row:
            return dict(row)
        return None

    def _save_download(
        self, package: str, version: str, tarball_path: str, tarball_sha256: str
    ):
        """Save download to cache."""
        conn = sqlite3.connect(str(CACHE_DB))
        conn.execute(
            """
            INSERT OR REPLACE INTO downloads
            (package, version, tarball_path, tarball_sha256, downloaded_at)
            VALUES (?, ?, ?, ?, ?)
            """,
            (
                package,
                version,
                tarball_path,
                tarball_sha256,
                datetime.utcnow().isoformat(),
            ),
        )
        conn.commit()
        conn.close()

    def get_package_info(self, package: str) -> Optional[Dict]:
        """
        Fetch full package metadata from npm registry.
        
        Args:
            package: Package name (e.g., "express", "@scope/package")
            
        Returns:
            Package metadata dict or None if not found
        """
        self._rate_limit()
        
        try:
            resp = requests.get(f"{NPM_REGISTRY}/{package}", timeout=30)
            if resp.status_code != 200:
                return None
            
            data = resp.json()
            
            # Extract key info
            latest = data.get("dist-tags", {}).get("latest", "")
            versions = data.get("versions", {})
            latest_version = versions.get(latest, {})
            
            return {
                "name": data.get("name", package),
                "latest_version": latest,
                "description": latest_version.get("description", ""),
                "author": latest_version.get("author", {}),
                "downloads_weekly": 0,  # Would need separate API call
                "has_install_scripts": bool(
                    latest_version.get("scripts", {}).get("install")
                    or latest_version.get("scripts", {}).get("preinstall")
                    or latest_version.get("scripts", {}).get("postinstall")
                ),
                "versions": list(versions.keys()),
            }
        except Exception:
            return None

    def download_package(
        self, package: str, timeout: int = 60
    ) -> Optional[Dict]:
        """
        Download a package tarball.
        
        Args:
            package: Package spec (e.g., "express@4.19.2")
            timeout: Download timeout in seconds
            
        Returns:
            Dict with tarball_path, tarball_sha256, name, version
            or None if download failed
        """
        # Parse package spec
        if "@" in package and (package.startswith("@") or package.count("@") == 1):
            if package.startswith("@"):
                # Scoped package: @scope/name@version
                parts = package.rsplit("@", 1)
                name = parts[0]
                version = parts[1] if len(parts) > 1 else "latest"
            else:
                # Unscoped with version: name@version
                parts = package.split("@", 1)
                name = parts[0]
                version = parts[1]
        else:
            name = package
            version = "latest"

        # Check cache
        cached = self._get_cached_download(name, version)
        if cached and Path(cached["tarball_path"]).exists():
            return {
                "tarball_path": cached["tarball_path"],
                "tarball_sha256": cached["tarball_sha256"],
                "name": name,
                "version": version,
                "cached": True,
            }

        # Download
        with tempfile.TemporaryDirectory() as tmpdir:
            try:
                dl_result = subprocess.run(
                    ["npm", "pack", package],
                    capture_output=True,
                    text=True,
                    cwd=tmpdir,
                    timeout=timeout,
                )
                if dl_result.returncode != 0:
                    return None

                # Find tarball
                tarballs = list(Path(tmpdir).glob("*.tgz"))
                if not tarballs:
                    return None
                tarball = tarballs[0]

                # Calculate hash
                tarball_sha256 = self._sha256_file(tarball)

                # Move to permanent location
                cache_dir = Path.home() / ".glassware" / "cache" / "packages"
                cache_dir.mkdir(parents=True, exist_ok=True)
                safe_name = name.replace("/", "_")
                dest_path = cache_dir / f"{safe_name}-{version}.tgz"
                tarball.rename(dest_path)

                # Save to cache
                self._save_download(name, version, str(dest_path), tarball_sha256)

                return {
                    "tarball_path": str(dest_path),
                    "tarball_sha256": tarball_sha256,
                    "name": name,
                    "version": version,
                    "cached": False,
                }

            except subprocess.TimeoutExpired:
                return None
            except Exception:
                return None

    def search_packages(
        self,
        keywords: List[str],
        size: int = 250,
        from_: int = 0,
    ) -> List[str]:
        """
        Search npm registry for packages.
        
        Args:
            keywords: Search keywords
            size: Number of results
            from_: Offset for pagination
            
        Returns:
            List of package names
        """
        self._rate_limit()
        
        try:
            resp = requests.get(
                NPM_SEARCH,
                params={
                    "text": " ".join(keywords),
                    "size": size,
                    "from": from_,
                    "quality": 0.65,
                    "popularity": 0.98,
                    "maintenance": 0.5,
                },
                timeout=30,
            )
            if resp.status_code != 200:
                return []

            data = resp.json()
            return [obj["package"]["name"] for obj in data.get("objects", [])]

        except Exception:
            return []

    def sample_packages(
        self,
        categories: Optional[Dict[str, List[str]]] = None,
        samples_per_category: int = 50,
        days: int = 30,
    ) -> List[str]:
        """
        Sample diverse packages from npm registry.
        
        Args:
            categories: Dict mapping category names to keyword lists
            samples_per_category: Samples per category
            days: Sample from last N days (not enforced - just for metadata)
            
        Returns:
            List of package specs (name@version)
        """
        if categories is None:
            categories = {
                "ai-ml": ["machine learning", "artificial intelligence", "neural network"],
                "native-build": ["node-gyp", "native addon", "bindings"],
                "web-frameworks": ["web framework", "express", "koa", "fastify"],
                "crypto": ["crypto", "encryption", "blockchain", "wallet"],
                "utils": ["utility", "helper", "tools"],
            }

        packages = set()

        for category, keywords in categories.items():
            for keyword in keywords:
                results = self.search_packages([keyword], size=samples_per_category)
                for pkg in results[: samples_per_category // len(keywords)]:
                    packages.add(pkg)

        return list(packages)

    def get_recent_packages(self, days: int = 14, limit: int = 200) -> List[str]:
        """
        Get recently published packages.
        
        Note: This is a simplified implementation. The npm registry doesn't
        provide a direct "recent packages" API, so we sample from common
        categories and filter by published date.
        
        Args:
            days: Number of days to look back
            limit: Maximum packages to return
            
        Returns:
            List of package specs
        """
        # Sample from diverse categories
        packages = self.sample_packages(
            samples_per_category=limit // 5,
            days=days,
        )

        # Filter by publish date (simplified - would need full metadata)
        recent = []
        for pkg in packages[:limit]:
            info = self.get_package_info(pkg)
            if info:
                recent.append(f"{pkg}@{info['latest_version']}")

        return recent[:limit]

    def get_typosquats(self, top_packages: List[str]) -> List[str]:
        """
        Generate potential typosquat candidates for top packages.
        
        Args:
            top_packages: List of popular package names
            
        Returns:
            List of potential typosquat package names
        """
        typosquats = []
        
        for pkg in top_packages:
            # Common typosquat patterns
            patterns = [
                f"{pkg}-js",
                f"{pkg}js",
                f"{pkg}.js",
                f"{pkg}-official",
                f"{pkg}-lib",
                f"@{pkg}/core",
            ]
            typosquats.extend(patterns)

        return typosquats
