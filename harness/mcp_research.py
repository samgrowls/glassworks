"""
MCP Ecosystem Research
Search npm for MCP-related packages systematically
"""
import requests
import json
from datetime import datetime

NPM_SEARCH = "https://registry.npmjs.org/-/v1/search"

# Search queries for MCP ecosystem
SEARCH_QUERIES = [
    "mcp",
    "mcp-server",
    "model-context-protocol",
    "@modelcontextprotocol",
    "@mcp",
    "anthropic-mcp",
    "claude-mcp",
    "cursor-mcp",
]

def search_npm(query, size=250):
    """Search npm for packages"""
    params = {
        "text": query,
        "size": size,
    }
    try:
        resp = requests.get(NPM_SEARCH, params=params, timeout=30)
        resp.raise_for_status()
        data = resp.json()
        return data.get("objects", [])
    except Exception as e:
        print(f"Error searching '{query}': {e}")
        return []

def extract_package_info(obj):
    """Extract package info from search result"""
    pkg = obj.get("package", {})
    return {
        "name": pkg.get("name"),
        "version": pkg.get("version"),
        "description": pkg.get("description", "")[:100],
        "links": pkg.get("links", {}),
    }

def main():
    print("=== MCP Ecosystem Research ===")
    print(f"Started: {datetime.utcnow().isoformat()}")
    print()
    
    all_packages = {}
    
    for query in SEARCH_QUERIES:
        print(f"Searching: {query}...")
        results = search_npm(query, size=250)
        print(f"  Found: {len(results)} packages")
        
        for obj in results:
            pkg_info = extract_package_info(obj)
            name = pkg_info["name"]
            if name and name not in all_packages:
                all_packages[name] = pkg_info
    
    print()
    print(f"Total unique packages: {len(all_packages)}")
    print()
    
    # Save results
    output_file = "mcp_packages.json"
    with open(output_file, "w") as f:
        json.dump({
            "search_date": datetime.utcnow().isoformat(),
            "queries": SEARCH_QUERIES,
            "packages": list(all_packages.values()),
            "count": len(all_packages),
        }, f, indent=2)
    
    print(f"Saved to: {output_file}")
    print()
    
    # Print summary
    print("=== Package List ===")
    for i, pkg in enumerate(sorted(all_packages.keys()), 1):
        print(f"{i:3}. {pkg}")
    
    return list(all_packages.keys())

if __name__ == "__main__":
    main()
