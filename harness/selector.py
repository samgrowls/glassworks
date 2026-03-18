"""
npm Registry Selector

Queries the npm registry to find packages matching Tier 1 filter criteria:
- Published within last 30 days
- Has preinstall or postinstall scripts
- Weekly downloads < 1000
- Prefer single-version packages
"""

import asyncio
import time
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from typing import Optional

import aiohttp
from rich.console import Console

console = Console()

# npm Registry API endpoints
NPM_REGISTRY_BASE = "https://registry.npmjs.org"
NPM_DOWNLOADS_API = "https://api.npmjs.org/downloads/point/last-week"


@dataclass
class PackageCandidate:
    """A package that matches our selection criteria."""

    name: str
    version: str
    tarball_url: str
    published_at: str
    author: Optional[str] = None
    downloads_weekly: int = 0
    has_install_scripts: bool = False
    scripts: list[str] = field(default_factory=list)
    description: str = ""
    homepage: Optional[str] = None
    repository: Optional[str] = None


class NPMSelector:
    """Query npm registry for packages matching Tier 1 criteria."""

    def __init__(
        self,
        days_back: int = 30,
        download_threshold: int = 1000,
        max_packages: int = 100,
        rate_limit_delay: float = 0.5,
    ):
        self.days_back = days_back
        self.download_threshold = download_threshold
        self.max_packages = max_packages
        self.rate_limit_delay = rate_limit_delay
        self.cutoff_date = datetime.utcnow() - timedelta(days=days_back)
        self.session: Optional[aiohttp.ClientSession] = None

    async def __aenter__(self):
        self.session = aiohttp.ClientSession(
            headers={"User-Agent": "glassware-scanner/0.1.0"},
            timeout=aiohttp.ClientTimeout(total=30),
        )
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()

    async def search_packages(
        self, query: str = "", size: int = 250
    ) -> list[dict]:
        """Search npm registry for packages."""
        if not self.session:
            raise RuntimeError("Session not initialized. Use async context manager.")

        url = f"{NPM_REGISTRY_BASE}/-/v1/search"
        params = {"text": query, "size": size}

        try:
            async with self.session.get(url, params=params) as resp:
                if resp.status == 429:
                    retry_after = int(resp.headers.get("Retry-After", 60))
                    console.print(
                        f"[yellow]Rate limited. Waiting {retry_after}s...[/yellow]"
                    )
                    await asyncio.sleep(retry_after)
                    return await self.search_packages(query, size)

                resp.raise_for_status()
                data = await resp.json()
                return data.get("objects", [])
        except aiohttp.ClientError as e:
            console.print(f"[red]Search failed: {e}[/red]")
            return []

    async def get_package_metadata(self, name: str) -> Optional[dict]:
        """Get full metadata for a package."""
        if not self.session:
            raise RuntimeError("Session not initialized.")

        url = f"{NPM_REGISTRY_BASE}/{name}"

        try:
            async with self.session.get(url) as resp:
                if resp.status == 429:
                    await asyncio.sleep(60)
                    return await self.get_package_metadata(name)

                if resp.status == 404:
                    return None

                resp.raise_for_status()
                return await resp.json()
        except aiohttp.ClientError as e:
            console.print(f"[red]Failed to fetch {name}: {e}[/red]")
            return None

    async def get_download_count(self, name: str) -> int:
        """Get weekly download count for a package."""
        if not self.session:
            raise RuntimeError("Session not initialized.")

        url = f"{NPM_DOWNLOADS_API}/{name}"

        try:
            async with self.session.get(url) as resp:
                if resp.status == 200:
                    data = await resp.json()
                    return data.get("downloads", 0)
                return 0
        except aiohttp.ClientError:
            return 0

    def _has_install_scripts(self, metadata: dict) -> tuple[bool, list[str]]:
        """Check if package has preinstall/postinstall scripts."""
        scripts = metadata.get("scripts", {})
        install_scripts = []

        for script_name in ["preinstall", "postinstall", "install"]:
            if script_name in scripts:
                install_scripts.append(script_name)

        return len(install_scripts) > 0, install_scripts

    def _get_latest_version_metadata(
        self, metadata: dict
    ) -> tuple[str, str, Optional[str]]:
        """Extract latest version info from package metadata."""
        dist_tags = metadata.get("dist-tags", {})
        latest_version = dist_tags.get("latest", "")

        versions = metadata.get("versions", {})
        version_data = versions.get(latest_version, {})

        dist = version_data.get("dist", {})
        tarball_url = dist.get("tarball", "")
        publish_time = dist.get("publish_time")

        # Fallback for publish time
        if not publish_time:
            publish_time = version_data.get("publish-time", "")

        return latest_version, tarball_url, publish_time

    def _is_recently_published(
        self, publish_time: Optional[str]
    ) -> tuple[bool, datetime]:
        """Check if package was published within cutoff date."""
        if not publish_time:
            return False, datetime.min

        try:
            # Handle both timestamp formats
            if isinstance(publish_time, (int, float)):
                published = datetime.fromtimestamp(publish_time)
            else:
                published = datetime.fromisoformat(publish_time.replace("Z", "+00:00"))
                published = published.replace(tzinfo=None)

            return published >= self.cutoff_date, published
        except (ValueError, TypeError):
            return False, datetime.min

    async def evaluate_package(
        self, package_obj: dict
    ) -> Optional[PackageCandidate]:
        """Evaluate a package against Tier 1 criteria."""
        package_data = package_obj.get("package", {})
        name = package_data.get("name", "")

        if not name:
            return None

        # Skip scoped packages for now (can be added later)
        if name.startswith("@"):
            return None

        console.print(f"[dim]Evaluating: {name}[/dim]")

        # Get full metadata
        metadata = await self.get_package_metadata(name)
        if not metadata:
            return None

        await asyncio.sleep(self.rate_limit_delay)

        # Check install scripts
        has_scripts, scripts = self._has_install_scripts(metadata)
        if not has_scripts:
            return None  # Tier 1 requires install scripts

        # Get latest version info
        version, tarball_url, publish_time = self._get_latest_version_metadata(metadata)
        if not version or not tarball_url:
            return None

        # Check if recently published
        is_recent, published_date = self._is_recently_published(publish_time)
        if not is_recent:
            return None

        # Get download count
        downloads = await self.get_download_count(name)
        if downloads >= self.download_threshold:
            return None  # Too popular for Tier 1

        await asyncio.sleep(self.rate_limit_delay)

        # Extract author info
        author = None
        if "author" in metadata and isinstance(metadata["author"], dict):
            author = metadata["author"].get("name")
        elif "maintainers" in metadata and metadata["maintainers"]:
            author = metadata["maintainers"][0].get("name")

        return PackageCandidate(
            name=name,
            version=version,
            tarball_url=tarball_url,
            published_at=published_date.isoformat(),
            author=author,
            downloads_weekly=downloads,
            has_install_scripts=has_scripts,
            scripts=scripts,
            description=package_data.get("description", ""),
            homepage=metadata.get("homepage"),
            repository=metadata.get("repository", {}).get("url")
            if isinstance(metadata.get("repository"), dict)
            else None,
        )

    async def select_packages(
        self, search_queries: list[str] | None = None
    ) -> list[PackageCandidate]:
        """
        Select packages matching Tier 1 criteria.

        Args:
            search_queries: Optional list of search terms. If None, uses broad search.

        Returns:
            List of PackageCandidate objects matching criteria.
        """
        if search_queries is None:
            # Broad search for recently published packages
            search_queries = ["", "util", "helper", "tools", "dev", "cli"]

        console.print(
            f"\n[bold blue]Searching npm for Tier 1 candidates...[/bold blue]"
        )
        console.print(f"  Days back: {self.days_back}")
        console.print(f"  Download threshold: < {self.download_threshold}/week")
        console.print(f"  Max packages: {self.max_packages}")

        candidates: list[PackageCandidate] = []
        seen_names: set[str] = set()

        for query in search_queries:
            if len(candidates) >= self.max_packages:
                break

            console.print(f"\n[dim]Search query: '{query or '(empty)'}'[/dim]")
            packages = await self.search_packages(query, size=250)

            for pkg_obj in packages:
                if len(candidates) >= self.max_packages:
                    break

                name = pkg_obj.get("package", {}).get("name", "")
                if name in seen_names or name.startswith("@"):
                    continue

                seen_names.add(name)
                candidate = await self.evaluate_package(pkg_obj)

                if candidate:
                    candidates.append(candidate)
                    console.print(
                        f"  [green]✓ {name}@{candidate.version}[/green] "
                        f"[dim]({candidate.downloads_weekly} downloads/wk, "
                        f"scripts: {', '.join(candidate.scripts)})[/dim]"
                    )

        console.print(
            f"\n[bold]Found {len(candidates)} Tier 1 candidates[/bold]"
        )
        return candidates


async def main():
    """Test the selector."""
    async with NPMSelector(
        days_back=30, download_threshold=1000, max_packages=10
    ) as selector:
        candidates = await selector.select_packages()

        console.print("\n[bold]Selected packages:[/bold]")
        for c in candidates:
            console.print(
                f"  {c.name}@{c.version} - {c.tarball_url[:50]}..."
            )


if __name__ == "__main__":
    asyncio.run(main())
