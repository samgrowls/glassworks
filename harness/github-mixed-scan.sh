#!/bin/bash
# GitHub Mixed Scan - MCP + VSCode + Cursor + DevTools

echo "=== GitHub Mixed Repository Scan ==="
echo "Target:"
echo "  - 200 MCP servers"
echo "  - 400 VSCode extensions"
echo "  - 100 Cursor extensions"
echo "  - 200 Dev tools"
echo "  - Total: 900 repos"
echo ""
echo "Starting scan..."
echo ""

python3 github_scanner.py \
  --queries \
    "mcp-server" \
    "model-context-protocol" \
    "mcp extension" \
    "mcp-server language" \
    "mcp ai" \
    "vscode-extension" \
    "vsce" \
    "visual-studio-code extension" \
    "vscode theme" \
    "vscode language" \
    "cursor-extension" \
    "cursor ide" \
    "cursor plugin" \
    "node-gyp" \
    "prebuild" \
    "bindings native" \
    "webpack plugin" \
    "babel plugin" \
  --repos-per-query 50 \
  --max-repos 900 \
  --scanner ./glassware-scanner \
  --output github-mixed-scan-results.json \
  --clone-dir data/github-clones-mixed \
  2>&1 | tee github-mixed-scan.log

echo ""
echo "=== Scan Complete ==="
echo "Results: github-mixed-scan-results.json"
echo "Log: github-mixed-scan.log"
