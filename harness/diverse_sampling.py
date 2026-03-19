#!/usr/bin/env python3
"""
Diverse NPM Package Sampling

Samples from different categories across the full npm ecosystem, not just MCP.
"""

import requests
import json
import argparse
from pathlib import Path
from datetime import datetime
import time

NPM_SEARCH = "https://registry.npmjs.org/-/v1/search"

# Diverse category buckets
CATEGORY_BUCKETS = {
    # High-risk categories (install scripts, native code)
    "native-build": ["node-gyp", "bindings", "prebuild", "nan", "node-addon-api"],
    "install-scripts": ["preinstall", "postinstall", "install"],

    # Popular ecosystem packages
    "web-frameworks": ["react", "vue", "angular", "svelte", "next", "nuxt"],
    "backend": ["express", "fastify", "koa", "hapi", "nest"],
    "database": ["mongoose", "sequelize", "typeorm", "prisma", "knex"],

    # Developer tools (high trust, but targeted)
    "devtools": ["eslint", "prettier", "typescript", "babel", "webpack", "vite"],
    "testing": ["jest", "mocha", "vitest", "cypress", "playwright"],
    "cli": ["commander", "yargs", "chalk", "ora", "inquirer"],

    # Security/crypto (legitimate crypto usage)
    "crypto": ["crypto", "bcrypt", "jsonwebtoken", "jose", "node-forge"],
    "security": ["helmet", "cors", "rate-limit", "validator", "sanitize"],

    # AI/ML trend (currently targeted)
    "ai-ml": ["ai", "llm", "openai", "anthropic", "langchain", "agent"],

    # Utility packages (widely used)
    "utils": ["lodash", "async", "moment", "dayjs", "axios", "got"],
    "logging": ["winston", "pino", "log4js", "debug"],
}

def search_npm(keywords, size=100, max_retries=3, backoff_seconds=2):
    """Search npm for packages with retry and backoff.
    
    Args:
        keywords: Can be a string (single keyword) or list of keywords
        size: Number of results per keyword
        max_retries: Max retries per keyword
        backoff_seconds: Base backoff time
    """
    # Support both single keyword and list of keywords
    if isinstance(keywords, str):
        keywords = [keywords]
    
    all_packages = []
    
    for keyword in keywords:
        params = {"text": keyword, "size": size}

        for attempt in range(max_retries):
            try:
                resp = requests.get(NPM_SEARCH, params=params, timeout=30)

                if resp.status_code == 429:
                    # Rate limited - wait and retry
                    retry_after = int(resp.headers.get('Retry-After', backoff_seconds * (attempt + 1)))
                    print(f"    Rate limited. Waiting {retry_after}s...")
                    time.sleep(retry_after)
                    continue

                resp.raise_for_status()
                data = resp.json()
                return data.get("objects", [])
                
            except requests.exceptions.RequestException as e:
                if attempt < max_retries - 1:
                    wait_time = backoff_seconds * (2 ** attempt)  # Exponential backoff
                    print(f"    Error: {e}. Retrying in {wait_time}s...")
                    time.sleep(wait_time)
                else:
                    print(f"    Error searching '{keyword}': {e}")
                    return []

    return []

def sample_packages_per_category(category_name: str, keywords: list, samples_per_keyword: int = 10, delay_between_keywords: float = 0.5) -> list:
    """Sample packages from a category with rate limiting"""
    packages = set()
    
    for i, keyword in enumerate(keywords):
        print(f"  Sampling '{keyword}'...")
        results = search_npm(keyword, size=samples_per_keyword * 2)
        
        for obj in results[:samples_per_keyword]:
            pkg = obj.get("package", {})
            name = pkg.get("name")
            version = pkg.get("version")
            if name and version:
                packages.add(f"{name}@{version}")
        
        # Add delay between keywords to avoid rate limiting
        if i < len(keywords) - 1 and delay_between_keywords > 0:
            time.sleep(delay_between_keywords)
    
    return list(packages)

def main():
    import argparse

    parser = argparse.ArgumentParser(description="Diverse NPM package sampling")
    parser.add_argument("--samples-per-keyword", type=int, default=10, help="Samples per keyword")
    parser.add_argument("--categories", nargs="*", default=None, help="Specific categories to sample (default: all)")
    parser.add_argument("--keywords", nargs="*", default=None, help="Direct keywords to search (bypasses categories)")
    parser.add_argument("--output", "-o", default="diverse-sample.txt", help="Output file")
    parser.add_argument("--delay-between-keywords", type=float, default=0.5, help="Delay between keyword searches (seconds, default: 0.5)")
    parser.add_argument("--npm-retries", type=int, default=3, help="Max retries for npm API (default: 3)")
    parser.add_argument("--npm-backoff", type=float, default=2, help="Base backoff time for npm API (seconds, default: 2)")

    args = parser.parse_args()

    print("="*70)
    print("DIVERSE NPM PACKAGE SAMPLING")
    print("="*70)
    
    all_packages = []

    # Support direct keywords (bypasses predefined categories)
    if args.keywords:
        print(f"Keywords: {len(args.keywords)}")
        print(f"Samples per keyword: {args.samples_per_keyword}")
        print(f"Started: {datetime.utcnow().isoformat()}Z")
        print()
        
        packages = sample_packages_per_category("custom", args.keywords, args.samples_per_keyword, args.delay_between_keywords)
        print(f"  → {len(packages)} packages sampled")
        all_packages.extend(packages)
    else:
        categories = args.categories if args.categories else list(CATEGORY_BUCKETS.keys())
        
        print(f"Categories: {len(categories)}")
        print(f"Samples per keyword: {args.samples_per_keyword}")
        print(f"Started: {datetime.utcnow().isoformat()}Z")
        print()

        for category in categories:
            if category not in CATEGORY_BUCKETS:
                print(f"⚠️  Unknown category: {category}")
                continue

            keywords = CATEGORY_BUCKETS[category]
            print(f"[{category}] ({len(keywords)} keywords)")

            packages = sample_packages_per_category(category, keywords, args.samples_per_keyword, args.delay_between_keywords)
            print(f"  → {len(packages)} packages sampled")

            all_packages.extend(packages)

    # Remove duplicates
    unique_packages = list(set(all_packages))

    # Save to file
    Path(args.output).write_text("\n".join(unique_packages))

    print()
    print("="*70)
    print("SAMPLING COMPLETE")
    print("="*70)
    print(f"Total packages: {len(unique_packages)}")
    print(f"Output: {args.output}")
    print("="*70)

    # Show category breakdown
    print()
    print("Category breakdown:")
    if args.keywords:
        for keyword in args.keywords:
            print(f"  {keyword}: ~{args.samples_per_keyword} packages")
    else:
        for category in categories:
            if category in CATEGORY_BUCKETS:
                keywords = CATEGORY_BUCKETS[category]
                estimated = len(keywords) * args.samples_per_keyword
                print(f"  {category}: ~{estimated} packages")

if __name__ == "__main__":
    main()
