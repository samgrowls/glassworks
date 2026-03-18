#!/bin/bash
# Scan all @iflow-mcp/ packages

EVIDENCE_DIR="data/evidence/mcp-scan"
mkdir -p "$EVIDENCE_DIR"

echo "=== @iflow-mcp/ SCOPE SCAN ==="
echo "Started: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo ""

# Package list
PACKAGES=(
"@iflow-mcp/blake365-options-chain"
"@iflow-mcp/cameroncooke_xcodebuildmcp"
"@iflow-mcp/cursor-mcp"
"@iflow-mcp/deployto-dev-namecheap-domains-mcp"
"@iflow-mcp/figma-mcp"
"@iflow-mcp/garethcott-enhanced-postgres-mcp-server"
"@iflow-mcp/garethcott_enhanced-postgres-mcp-server"
"@iflow-mcp/hemanth-mcp-ui-server"
"@iflow-mcp/jageenshukla-hello-world-mcp-server"
"@iflow-mcp/mailgun-mcp-server"
"@iflow-mcp/matthewdailey-mcp-starter"
"@iflow-mcp/matthewdailey-rime-mcp"
"@iflow-mcp/mcp-starter"
"@iflow-mcp/minecraft-mcp-server"
"@iflow-mcp/modelcontextprotocol-servers-whois-mcp"
"@iflow-mcp/openai-gpt-image-mcp"
"@iflow-mcp/playwright-mcp"
"@iflow-mcp/puppeteer-mcp-server"
"@iflow-mcp/ref-tools-mcp"
"@iflow-mcp/strato-space-media-gen-mcp"
"@iflow-mcp/tsai1030-ziwei-mcp-server"
"@iflow-mcp/wizd-airylark-mcp-server"
)

GLASSWARE="/home/property.sightlines/samgrowls/glassworks/target/release/glassware"
RESULTS_FILE="$EVIDENCE_DIR/iflow-scan-results.json"

echo "[" > "$RESULTS_FILE"
FIRST=true

for i in "${!PACKAGES[@]}"; do
    PKG="${PACKAGES[$i]}"
    echo "[$((i+1))/${#PACKAGES[@]}] Scanning $PKG..."
    
    # Download
    cd /tmp
    rm -rf iflow-pkg
    mkdir iflow-pkg
    cd iflow-pkg
    
    TARBALL=$(npm pack "$PKG" 2>&1 | tail -1)
    if [[ ! -f "$TARBALL" ]]; then
        echo "  ERROR: Failed to download"
        continue
    fi
    
    # Extract
    tar -xzf "$TARBALL"
    PKG_DIR="package"
    
    # Scan
    RESULT=$($GLASSWARE --format json "$PKG_DIR" 2>&1)
    FINDINGS=$(echo "$RESULT" | jq -c '.findings | length' 2>/dev/null || echo "0")
    CRITICAL=$(echo "$RESULT" | jq -c '[.findings[] | select(.severity == "critical")] | length' 2>/dev/null || echo "0")
    
    echo "  Findings: $FINDINGS (Critical: $CRITICAL)"
    
    # Backup if flagged
    if [[ "$FINDINGS" -gt "0" ]]; then
        cp "$TARBALL" "$EVIDENCE_DIR/"
        echo "$RESULT" | jq --arg pkg "$PKG" --arg tarball "$TARBALL" '. + {package: $pkg, tarball: $tarball}' >> "$EVIDENCE_DIR/iflow-flagged.jsonl"
    fi
    
    # Add to results
    if [ "$FIRST" = true ]; then
        FIRST=false
    else
        echo "," >> "$RESULTS_FILE"
    fi
    
    echo "$RESULT" | jq --arg pkg "$PKG" --arg tarball "$TARBALL" --argjson findings "$FINDINGS" --argjson critical "$CRITICAL" \
        '{package: $pkg, tarball: $tarball, findings: $findings, critical: $critical}' >> "$RESULTS_FILE"
done

echo "]" >> "$RESULTS_FILE"

echo ""
echo "=== SUMMARY ==="
echo "Completed: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "Results: $RESULTS_FILE"
echo "Flagged packages: $(wc -l < "$EVIDENCE_DIR/iflow-flagged.jsonl" 2>/dev/null || echo 0)"
