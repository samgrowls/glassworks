#!/bin/bash
# Run autoresearch with proper config application
# This script generates a campaign config for each iteration

set -e

CONFIG_FILE="${1:-glassware-tools/autoresearch.toml}"
OUTPUT_DIR="output/autoresearch"
TEMP_CONFIG="/tmp/autoresearch-campaign.toml"

echo "=== Autoresearch Runner ==="
echo "Config: $CONFIG_FILE"
echo "Output: $OUTPUT_DIR"
echo ""

mkdir -p "$OUTPUT_DIR"

# Read parameters from TOML and generate campaign config
# For now, just run the existing autoresearch binary
echo "Starting autoresearch loop..."
./target/release/autoresearch --config "$CONFIG_FILE" --output-dir "$OUTPUT_DIR"
