#!/usr/bin/env python3
"""
Version History Package Sampler

Samples diverse npm packages for version history scanning.

Criteria:
- Recently updated (last 30 days)
- New packages (published last 30 days)
- Popular packages (high download count)
- Diverse categories (native-build, install-scripts, web-frameworks, etc.)

Usage:
    python version_sampler.py \
        --output packages.txt \
        --samples 500 \
        --days 30 \
        --categories ai-ml native-build web-frameworks
"""

import argparse
import json
import requests
import sys
from datetime import datetime, timedelta
from pathlib import Path
from typing import List, Dict, Optional
import time

# Configuration
NPM_SEARCH = "https://registry.npmjs.org/-/v1/search"
NPM_REGISTRY = "https://registry.npmjs.org"
NPM_DOWNLOADS = "https://api.npmjs.org/downloads"

# Category buckets with search keywords
CATEGORY_BUCKETS = {
    # High-risk categories (install scripts, native code)
    "native-build": ["node-gyp", "bindings", "prebuild", "nan", "node-addon-api", "neon", "cmake-js"],
    "install-scripts": ["preinstall", "postinstall", "install", "husky", "patch-package"],
    
    # Popular ecosystem packages (high impact if compromised)
    "web-frameworks": ["react", "vue", "angular", "svelte", "next", "nuxt", "gatsby", "remix"],
    "backend": ["express", "fastify", "koa", "hapi", "nest", "adonis", "feathers"],
    "database": ["mongoose", "sequelize", "prisma", "typeorm", "knex", "pg", "mysql2"],
    
    # Developer tools (trusted, high privilege)
    "devtools": ["eslint", "prettier", "typescript", "babel", "webpack", "vite", "rollup", "esbuild"],
    "testing": ["jest", "mocha", "vitest", "cypress", "playwright", "testing-library"],
    
    # AI/ML trend (currently targeted by attackers)
    "ai-ml": ["ai", "llm", "langchain", "openai", "anthropic", "huggingface", "transformers"],
    
    # Utilities (widely used)
    "utils": ["lodash", "async", "moment", "dayjs", "axios", "got", "chalk", "commander"],
    "logging": ["winston", "pino", "log4js", "debug", "bunyan"],
    
    # Security/crypto (legitimate crypto usage)
    "crypto": ["bcrypt", "jsonwebtoken", "jose", "node-forge", "crypto-js", "tweetnacl"],
    "security": ["helmet", "cors", "rate-limit", "validator", "sanitize", "dompurify"],
}


def search_npm(keyword: str, size: int = 100, max_retries: int = 3) -> List[Dict]:
    """Search npm registry for packages"""
    params = {"text": keyword, "size": size}
    
    for attempt in range(max_retries):
        try:
            resp = requests.get(NPM_SEARCH, params=params, timeout=30)
            
            if resp.status_code == 429:
                retry_after = int(resp.headers.get('Retry-After', 2 ** attempt))
                print(f"    Rate limited. Waiting {retry_after}s...", file=sys.stderr)
                time.sleep(retry_after)
                continue
            
            resp.raise_for_status()
            data = resp.json()
            return data.get("objects", [])
            
        except requests.exceptions.RequestException as e:
            if attempt < max_retries - 1:
                wait_time = 2 ** attempt
                print(f"    Error: {e}. Retrying in {wait_time}s...", file=sys.stderr)
                time.sleep(wait_time)
            else:
                print(f"    Error searching '{keyword}': {e}", file=sys.stderr)
                return []
    
    return []


def fetch_package_metadata(package: str) -> Optional[Dict]:
    """Fetch full package metadata from npm registry"""
    try:
        resp = requests.get(f"{NPM_REGISTRY}/{package}", timeout=30)
        if resp.status_code == 200:
            return resp.json()
    except Exception as e:
        print(f"    Error fetching metadata for {package}: {e}", file=sys.stderr)
    return None


def get_download_count(package: str) -> int:
    """Get weekly download count"""
    try:
        resp = requests.get(f"{NPM_DOWNLOADS}/point/last-week/{package}", timeout=30)
        if resp.status_code == 200:
            data = resp.json()
            return data.get("downloads", 0)
    except Exception:
        pass
    return 0


def parse_date(date_str: str) -> Optional[datetime]:
    """Parse ISO date string"""
    try:
        # Handle various formats
        date_str = date_str.replace('Z', '+00:00')
        dt = datetime.fromisoformat(date_str)
        # Convert to local time if timezone-aware
        if dt.tzinfo is not None:
            dt = dt.astimezone()
        return dt
    except Exception:
        return None


def filter_by_date(packages: List[Dict], days: int) -> List[Dict]:
    """Filter packages updated in last N days"""
    cutoff = datetime.now().astimezone() - timedelta(days=days)
    filtered = []
    
    for pkg_obj in packages:
        pkg = pkg_obj.get("package", {})
        name = pkg.get("name")
        
        if not name:
            continue
        
        # Fetch metadata to get modification date
        metadata = fetch_package_metadata(name)
        if metadata:
            time_info = metadata.get("time", {})
            modified_str = time_info.get("modified")
            
            if modified_str:
                modified = parse_date(modified_str)
                if modified:
                    # Make cutoff timezone-aware if modified is aware
                    if modified.tzinfo is not None and cutoff.tzinfo is None:
                        cutoff = cutoff.replace(tzinfo=None)
                    if modified > cutoff:
                        pkg["category"] = "unknown"  # Will be set by caller
                        pkg["last_updated"] = modified_str
                        filtered.append(pkg)
    
    return filtered


def sample_by_category(
    categories: List[str],
    samples_per_category: int = 50,
    days: int = 30,
    delay_between_keywords: float = 0.5
) -> List[Dict]:
    """Sample packages from categories"""
    packages = []
    
    for category in categories:
        if category not in CATEGORY_BUCKETS:
            print(f"⚠️  Unknown category: {category}", file=sys.stderr)
            continue
        
        keywords = CATEGORY_BUCKETS[category]
        category_packages = set()
        
        print(f"\n[{category}] ({len(keywords)} keywords)", file=sys.stderr)
        
        for keyword in keywords:
            if len(category_packages) >= samples_per_category:
                break
            
            print(f"  Sampling '{keyword}'...", file=sys.stderr)
            results = search_npm(keyword, size=samples_per_category * 2)
            
            # Filter by date
            recent = filter_by_date(results, days)
            
            for pkg in recent[:samples_per_category]:
                name = pkg.get("name")
                if name and name not in category_packages:
                    pkg["category"] = category
                    category_packages.add(name)
                    packages.append(pkg)
            
            # Rate limiting
            time.sleep(delay_between_keywords)
        
        print(f"  → {len(category_packages)} packages sampled", file=sys.stderr)
    
    return packages


def sample_popular(
    samples: int = 100,
    min_downloads: int = 10000
) -> List[Dict]:
    """Sample popular packages by download count"""
    packages = []
    
    # Search for popular packages across categories
    all_keywords = []
    for keywords in CATEGORY_BUCKETS.values():
        all_keywords.extend(keywords)
    
    for keyword in all_keywords[:20]:  # Limit to avoid rate limiting
        if len(packages) >= samples:
            break
        
        print(f"  Searching popular: '{keyword}'...", file=sys.stderr)
        results = search_npm(keyword, size=50)
        
        for pkg_obj in results:
            if len(packages) >= samples:
                break
            
            pkg = pkg_obj.get("package", {})
            name = pkg.get("name")
            
            if not name:
                continue
            
            # Check download count
            downloads = get_download_count(name)
            if downloads >= min_downloads:
                pkg["downloads"] = downloads
                pkg["category"] = "popular"
                packages.append(pkg)
        
        time.sleep(0.5)
    
    return packages


def sample_new(
    samples: int = 100,
    days: int = 30
) -> List[Dict]:
    """Sample newly published packages"""
    packages = []
    
    # Search for new packages (npm search doesn't have date filter, so we fetch and filter)
    all_keywords = []
    for keywords in CATEGORY_BUCKETS.values():
        all_keywords.extend(keywords[:3])  # Limit keywords
    
    for keyword in all_keywords:
        if len(packages) >= samples:
            break
        
        print(f"  Searching new: '{keyword}'...", file=sys.stderr)
        results = search_npm(keyword, size=50)
        
        for pkg_obj in results:
            if len(packages) >= samples:
                break
            
            pkg = pkg_obj.get("package", {})
            name = pkg.get("name")
            
            if not name:
                continue
            
            # Fetch metadata to check publish date
            metadata = fetch_package_metadata(name)
            if metadata:
                time_info = metadata.get("time", {})
                created_str = time_info.get("created")
                
                if created_str:
                    created = parse_date(created_str)
                    if created and created > datetime.now() - timedelta(days=days):
                        pkg["created"] = created_str
                        pkg["category"] = "new"
                        packages.append(pkg)
        
        time.sleep(0.5)
    
    return packages


def save_packages(packages: List[Dict], output: Path, format: str = "plain"):
    """Save packages to file"""
    if format == "plain":
        # Plain text: one package per line
        with open(output, 'w') as f:
            for pkg in packages:
                f.write(f"{pkg['name']}\n")
    elif format == "json":
        # JSON format with metadata
        with open(output, 'w') as f:
            json.dump(packages, f, indent=2)
    elif format == "csv":
        # CSV format
        with open(output, 'w') as f:
            f.write("name,version,category,downloads,last_updated\n")
            for pkg in packages:
                f.write(f"{pkg['name']},{pkg.get('version', '')},{pkg.get('category', '')},{pkg.get('downloads', 0)},{pkg.get('last_updated', '')}\n")


def main():
    parser = argparse.ArgumentParser(
        description="Sample diverse npm packages for version history scanning"
    )
    parser.add_argument(
        "--output", "-o",
        required=True,
        help="Output file path"
    )
    parser.add_argument(
        "--samples",
        type=int,
        default=50,
        help="Samples per category (default: 50)"
    )
    parser.add_argument(
        "--categories",
        nargs="+",
        default=list(CATEGORY_BUCKETS.keys()),
        help="Categories to sample from"
    )
    parser.add_argument(
        "--days",
        type=int,
        default=30,
        help="Sample packages updated in last N days (default: 30)"
    )
    parser.add_argument(
        "--delay",
        type=float,
        default=0.5,
        help="Delay between keyword searches in seconds (default: 0.5)"
    )
    parser.add_argument(
        "--format",
        choices=["plain", "json", "csv"],
        default="plain",
        help="Output format (default: plain)"
    )
    parser.add_argument(
        "--include-popular",
        action="store_true",
        help="Include popular packages by download count"
    )
    parser.add_argument(
        "--include-new",
        action="store_true",
        help="Include newly published packages"
    )
    
    args = parser.parse_args()
    
    print("=" * 70, file=sys.stderr)
    print("VERSION HISTORY PACKAGE SAMPLER", file=sys.stderr)
    print("=" * 70, file=sys.stderr)
    print(f"Categories: {len(args.categories)}", file=sys.stderr)
    print(f"Samples per category: {args.samples}", file=sys.stderr)
    print(f"Days: {args.days}", file=sys.stderr)
    print(f"Output: {args.output}", file=sys.stderr)
    print(f"Started: {datetime.now().isoformat()}", file=sys.stderr)
    print("=" * 70, file=sys.stderr)
    
    # Sample by category
    packages = sample_by_category(
        args.categories,
        args.samples,
        args.days,
        args.delay
    )
    
    # Optionally include popular packages
    if args.include_popular:
        print("\n[popular] Sampling by download count...", file=sys.stderr)
        popular = sample_popular(args.samples)
        packages.extend(popular)
        print(f"  → {len(popular)} popular packages sampled", file=sys.stderr)
    
    # Optionally include new packages
    if args.include_new:
        print("\n[new] Sampling newly published packages...", file=sys.stderr)
        new = sample_new(args.samples, args.days)
        packages.extend(new)
        print(f"  → {len(new)} new packages sampled", file=sys.stderr)
    
    # Remove duplicates
    seen = set()
    unique_packages = []
    for pkg in packages:
        name = pkg.get("name")
        if name and name not in seen:
            seen.add(name)
            unique_packages.append(pkg)
    
    # Save
    output_path = Path(args.output)
    save_packages(unique_packages, output_path, args.format)
    
    # Summary
    print("\n" + "=" * 70, file=sys.stderr)
    print("SAMPLING COMPLETE", file=sys.stderr)
    print("=" * 70, file=sys.stderr)
    print(f"Total packages: {len(unique_packages)}", file=sys.stderr)
    print(f"Output: {output_path}", file=sys.stderr)
    print(f"Finished: {datetime.now().isoformat()}", file=sys.stderr)
    print("=" * 70, file=sys.stderr)
    
    # Category breakdown
    category_counts = {}
    for pkg in unique_packages:
        cat = pkg.get("category", "unknown")
        category_counts[cat] = category_counts.get(cat, 0) + 1
    
    if category_counts:
        print("\nCategory breakdown:", file=sys.stderr)
        for cat, count in sorted(category_counts.items(), key=lambda x: -x[1]):
            print(f"  {cat}: ~{count} packages", file=sys.stderr)


if __name__ == "__main__":
    main()
