#!/usr/bin/env python3
"""
Create sample package lists from different categories
"""
import json
from pathlib import Path

# Read the MCP scan results to get package patterns
with open("mcp_packages.json") as f:
    mcp_data = json.load(f)

all_packages = [p["name"] for p in mcp_data.get("packages", [])]

# Category A: VS Code / Extension-related (keywords)
vscode_keywords = ["vscode", "extension", "openvsx", "marketplace", "ide", "editor"]
vscode_packages = [p for p in all_packages if any(k in p.lower() for k in vscode_keywords)][:100]

# Category B: Popular packages (by naming patterns suggesting popularity)
popular_keywords = ["aws", "google", "azure", "microsoft", "facebook", "stripe", "shopify", "vercel", "netlify"]
popular_packages = [p for p in all_packages if any(k in p.lower() for k in popular_keywords)][:100]

# Category C: Recent packages (by naming patterns suggesting new/trendy)
recent_keywords = ["ai", "llm", "claude", "gpt", "mcp", "agent", "copilot", "auto"]
recent_packages = [p for p in all_packages if any(k in p.lower() for k in recent_keywords)][:100]

# Save lists
Path("sample-vscode.txt").write_text("\n".join(vscode_packages))
Path("sample-popular.txt").write_text("\n".join(popular_packages))
Path("sample-recent.txt").write_text("\n".join(recent_packages))

print(f"VS Code samples: {len(vscode_packages)}")
print(f"Popular samples: {len(popular_packages)}")
print(f"Recent samples: {len(recent_packages)}")
print(f"Total: {len(vscode_packages) + len(popular_packages) + len(recent_packages)}")
