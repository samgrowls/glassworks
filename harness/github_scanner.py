#!/usr/bin/env python3
"""
GitHub Repository Scanner

Scans GitHub repositories for GlassWare attack patterns.
Supports multiple discovery strategies: keyword search, maintainer tracking, trending.
"""

import requests
import os
from pathlib import Path
from datetime import datetime
import time
import subprocess

GITHUB_API = "https://api.github.com"

# GitHub token (optional but recommended for higher rate limits)
GITHUB_TOKEN = os.environ.get("GITHUB_TOKEN", "")

HEADERS = {
    "Accept": "application/vnd.github+json",
    "User-Agent": "glassware-scanner",
}

if GITHUB_TOKEN:
    HEADERS["Authorization"] = f"Bearer {GITHUB_TOKEN}"

# High-priority search queries
SEARCH_QUERIES = {
    "mcp": [
        "mcp-server",
        "model-context-protocol",
        "mcp extension",
        "mcp-server language",
    ],
    "vscode": [
        "vscode-extension",
        "vsce",
        "visual-studio-code extension",
        "vscode theme",
    ],
    "cursor": [
        "cursor-extension",
        "cursor ide",
        "cursor plugin",
    ],
    "devtools": [
        "node-gyp",
        "prebuild",
        "bindings native",
    ],
}

def search_github(query: str, sort: str = "updated", order: str = "desc", per_page: int = 100) -> list:
    """Search GitHub for repositories"""
    repos = []
    page = 1
    
    while True:
        try:
            url = f"{GITHUB_API}/search/repositories"
            params = {
                "q": query,
                "sort": sort,
                "order": order,
                "per_page": min(per_page, 100),
                "page": page,
            }
            
            resp = requests.get(url, headers=HEADERS, params=params, timeout=30)
            
            if resp.status_code == 403:
                print(f"    Rate limited. Waiting 60s...")
                time.sleep(60)
                continue
            
            if resp.status_code != 200:
                print(f"    Error: {resp.status_code} - {resp.text}")
                break
            
            data = resp.json()
            items = data.get("items", [])
            
            if not items:
                break
            
            for item in items:
                repos.append({
                    "full_name": item["full_name"],
                    "clone_url": item["clone_url"],
                    "html_url": item["html_url"],
                    "stargazers_count": item.get("stargazers_count", 0),
                    "updated_at": item.get("updated_at", ""),
                    "language": item.get("language", ""),
                })
            
            # Check if we have more pages
            if len(items) < 100:
                break
            
            page += 1
            
            # Rate limiting - be respectful
            time.sleep(0.5)
            
        except Exception as e:
            print(f"    Error searching '{query}': {e}")
            break
    
    return repos

def clone_repo(clone_url: str, dest_dir: Path) -> Path:
    """Clone a repository to destination directory"""
    try:
        repo_name = clone_url.split("/")[-1].replace(".git", "")
        dest_path = dest_dir / repo_name
        
        if dest_path.exists():
            # Pull if already cloned
            print(f"    Pulling updates...")
            subprocess.run(
                ["git", "pull"],
                cwd=dest_path,
                capture_output=True,
                timeout=60
            )
        else:
            # Clone fresh
            print(f"    Cloning...")
            subprocess.run(
                ["git", "clone", "--depth", "1", clone_url, str(dest_path)],
                capture_output=True,
                timeout=300
            )
        
        return dest_path
        
    except Exception as e:
        print(f"    Clone error: {e}")
        return None

def scan_repo(repo_path: Path, scanner_binary: str) -> dict:
    """Scan a cloned repository"""
    try:
        result = subprocess.run(
            [scanner_binary, "--format", "json", str(repo_path)],
            capture_output=True,
            text=True,
            timeout=60
        )
        
        if result.returncode == 0:
            import json
            findings = json.loads(result.stdout)
            return {
                "scanned": True,
                "findings": len(findings.get("findings", [])),
                "critical": len([f for f in findings.get("findings", []) if f.get("severity") == "critical"]),
                "details": findings,
            }
        else:
            return {
                "scanned": False,
                "error": result.stderr,
            }
            
    except Exception as e:
        return {
            "scanned": False,
            "error": str(e),
        }

def main():
    import argparse
    
    parser = argparse.ArgumentParser(description="GitHub Repository Scanner")
    parser.add_argument("--queries", nargs="*", default=None, help="Search queries")
    parser.add_argument("--repos-per-query", type=int, default=50, help="Repos per query")
    parser.add_argument("--output", "-o", default="github-scan-results.json", help="Output file")
    parser.add_argument("--clone-dir", default="data/github-clones", help="Clone directory")
    parser.add_argument("--scanner", default="./glassware-scanner", help="Scanner binary")
    parser.add_argument("--max-repos", type=int, default=500, help="Max repos to scan")
    
    args = parser.parse_args()
    
    queries = args.queries if args.queries else list(SEARCH_QUERIES.keys())
    
    print("="*70)
    print("GITHUB REPOSITORY SCANNER")
    print("="*70)
    print(f"Queries: {len(queries)}")
    print(f"Repos per query: {args.repos_per_query}")
    print(f"Max repos: {args.max_repos}")
    print(f"Started: {datetime.utcnow().isoformat()}Z")
    print()
    
    # Create clone directory
    clone_dir = Path(args.clone_dir)
    clone_dir.mkdir(parents=True, exist_ok=True)
    
    all_repos = []
    
    # Search for repositories
    for query in queries:
        print(f"[{query}] Searching...")
        
        # Get search keywords
        keywords = SEARCH_QUERIES.get(query, [query])
        
        for keyword in keywords:
            print(f"  → Searching '{keyword}'...")
            repos = search_github(keyword, per_page=args.repos_per_query)
            print(f"    Found {len(repos)} repos")
            all_repos.extend(repos)
            
            # Rate limiting
            time.sleep(1)
    
    # Remove duplicates
    unique_repos = {r["full_name"]: r for r in all_repos}.values()
    unique_repos = list(unique_repos)[:args.max_repos]
    
    print()
    print(f"Total unique repos: {len(unique_repos)}")
    print()
    
    # Clone and scan
    results = {
        "scan_date": datetime.utcnow().isoformat() + "Z",
        "total_repos": len(unique_repos),
        "scanned": 0,
        "flagged": 0,
        "errors": 0,
        "repos": [],
    }
    
    for i, repo in enumerate(unique_repos, 1):
        print(f"[{i}/{len(unique_repos)}] {repo['full_name']}")
        
        # Clone
        repo_path = clone_repo(repo["clone_url"], clone_dir)
        
        if not repo_path:
            results["errors"] += 1
            results["repos"].append({
                "name": repo["full_name"],
                "status": "clone_failed",
            })
            continue
        
        # Scan
        scan_result = scan_repo(repo_path, args.scanner)
        
        if scan_result["scanned"]:
            results["scanned"] += 1
            
            if scan_result["findings"] > 0:
                results["flagged"] += 1
                print(f"    ⚠️  Flagged: {scan_result['findings']} findings ({scan_result['critical']} critical)")
                
                results["repos"].append({
                    "name": repo["full_name"],
                    "url": repo["html_url"],
                    "status": "flagged",
                    "findings": scan_result["findings"],
                    "critical": scan_result["critical"],
                    "details": scan_result["details"],
                })
            else:
                print(f"    ✅ Clean")
                results["repos"].append({
                    "name": repo["full_name"],
                    "url": repo["html_url"],
                    "status": "clean",
                })
        else:
            results["errors"] += 1
            results["repos"].append({
                "name": repo["full_name"],
                "status": "scan_failed",
                "error": scan_result.get("error"),
            })
    
    # Save results
    import json
    Path(args.output).write_text(json.dumps(results, indent=2))
    
    print()
    print("="*70)
    print("SCAN COMPLETE")
    print("="*70)
    print(f"Total repos: {results['total_repos']}")
    print(f"Scanned: {results['scanned']}")
    print(f"Flagged: {results['flagged']}")
    print(f"Errors: {results['errors']}")
    print(f"Results: {args.output}")
    print("="*70)

if __name__ == "__main__":
    main()
