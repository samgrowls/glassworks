#!/bin/bash
# GlassWorm FP/Success Rate Benchmark
# 
# This script measures:
# 1. Evidence detection rate (known malicious packages)
# 2. False positive rate (known clean packages)
# 3. Combined score
#
# Usage: ./benchmarks/fp-success-benchmark.sh [--quick]
#
# Output: JSON metrics to stdout, detailed results to benchmarks/results/

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$SCRIPT_DIR/results"
BINARY="$ROOT_DIR/target/release/glassware"

# Parse arguments
QUICK_MODE=false
if [[ "$1" == "--quick" ]]; then
    QUICK_MODE=true
fi

# Create results directory
mkdir -p "$RESULTS_DIR"

echo "=== GlassWorm FP/Success Benchmark ==="
echo "Root: $ROOT_DIR"
echo "Quick mode: $QUICK_MODE"
echo ""

# Check if binary exists, build if needed
if [[ ! -f "$BINARY" ]]; then
    echo "Building glassware..."
    cd "$ROOT_DIR"
    cargo build --release -p glassware
fi

# All evidence tarballs (known malicious)
EVIDENCE_DIR="$ROOT_DIR/evidence"
EVIDENCE_TARBALLS=(
    # Original evidence from glassworks-archive
    "$EVIDENCE_DIR/react-native-country-select-0.3.91.tgz"
    "$EVIDENCE_DIR/react-native-international-phone-number-0.11.8.tgz"
    "$EVIDENCE_DIR/iflow-mcp-watercrawl-watercrawl-mcp-1.3.4.tgz"
    "$EVIDENCE_DIR/aifabrix-miso-client-4.7.2.tgz"
    # Synthetic evidence - blockchain_c2
    "$EVIDENCE_DIR/glassworm-c2-001.tgz"
    "$EVIDENCE_DIR/glassworm-c2-002.tgz"
    "$EVIDENCE_DIR/glassworm-c2-003.tgz"
    "$EVIDENCE_DIR/glassworm-c2-004.tgz"
    # Synthetic evidence - combined
    "$EVIDENCE_DIR/glassworm-combo-001.tgz"
    "$EVIDENCE_DIR/glassworm-combo-002.tgz"
    "$EVIDENCE_DIR/glassworm-combo-003.tgz"
    "$EVIDENCE_DIR/glassworm-combo-004.tgz"
    # Synthetic evidence - exfiltration
    "$EVIDENCE_DIR/glassworm-exfil-001.tgz"
    "$EVIDENCE_DIR/glassworm-exfil-002.tgz"
    "$EVIDENCE_DIR/glassworm-exfil-003.tgz"
    "$EVIDENCE_DIR/glassworm-exfil-004.tgz"
    # Synthetic evidence - steganography
    "$EVIDENCE_DIR/glassworm-steg-001.tgz"
    "$EVIDENCE_DIR/glassworm-steg-002.tgz"
    "$EVIDENCE_DIR/glassworm-steg-003.tgz"
    "$EVIDENCE_DIR/glassworm-steg-004.tgz"
    # Synthetic evidence - time_delay
    "$EVIDENCE_DIR/glassworm-evasion-001.tgz"
    "$EVIDENCE_DIR/glassworm-evasion-002.tgz"
    "$EVIDENCE_DIR/glassworm-evasion-003.tgz"
)

# Clean packages (known safe - top npm packages)
# In quick mode, use fewer packages
if [[ "$QUICK_MODE" == true ]]; then
    CLEAN_PACKAGES=(
        "express@4.19.2"
        "lodash@4.17.21"
        "axios@1.6.7"
        "chalk@5.3.0"
        "debug@4.3.4"
        "moment@2.30.1"
        "uuid@9.0.1"
        "async@3.2.5"
        "glob@10.3.10"
        "ws@8.16.0"
    )
else
    CLEAN_PACKAGES=(
        "express@4.19.2"
        "lodash@4.17.21"
        "axios@1.6.7"
        "chalk@5.3.0"
        "debug@4.3.4"
        "moment@2.30.1"
        "uuid@9.0.1"
        "async@3.2.5"
        "request@2.88.2"
        "commander@12.0.0"
        "glob@10.3.10"
        "mkdirp@3.0.1"
        "semver@7.6.0"
        "ws@8.16.0"
        "yargs@17.7.2"
        "dotenv@16.4.5"
        "eslint@8.57.0"
        "prettier@3.2.5"
        "typescript@5.4.2"
        "jest@29.7.0"
    )
fi

# Function to scan evidence tarballs
scan_evidence_tarballs() {
    echo "=== Scanning Evidence Tarballs ==="
    local detected=0
    local total=0
    
    for pkg in "${EVIDENCE_TARBALLS[@]}"; do
        if [[ -f "$pkg" ]]; then
            total=$((total + 1))
            local pkg_name=$(basename "$pkg" .tgz)
            echo "Scanning: $pkg_name"
            
            # Run scan and capture output
            local result
            result=$("$BINARY" scan-tarball "$pkg" 2>&1) || true
            
            # Check if flagged as malicious or suspicious
            if echo "$result" | grep -qE "(malicious|Threat score: [7-9])"; then
                echo "  ✓ DETECTED"
                detected=$((detected + 1))
            else
                echo "  ✗ NOT DETECTED"
            fi
            
            # Save detailed result
            echo "$result" > "$RESULTS_DIR/evidence-${pkg_name}.txt"
        else
            echo "Warning: Evidence package not found: $pkg"
        fi
    done
    
    if [[ $total -eq 0 ]]; then
        echo "No evidence tarballs found!"
        echo "0"
    else
        echo "Evidence tarball detection: $detected/$total"
        # Return rate as integer percentage (0-100)
        echo $((detected * 100 / total))
    fi
}

# Function to scan clean packages
scan_clean() {
    echo ""
    echo "=== Scanning Clean Packages ==="
    local flagged=0
    local total=${#CLEAN_PACKAGES[@]}
    
    for pkg in "${CLEAN_PACKAGES[@]}"; do
        echo "Scanning: $pkg"
        
        # Run scan and capture output
        local result
        result=$("$BINARY" scan-npm "$pkg" 2>&1) || true
        
        # Check if flagged as malicious (score >= 7.0)
        if echo "$result" | grep -qE "Malicious packages: [1-9]|flagged as malicious"; then
            echo "  ✗ FALSE POSITIVE"
            flagged=$((flagged + 1))
        elif echo "$result" | grep -qE "suspicious|Threat score: [4-6]"; then
            echo "  ⚠ Suspicious (not counting as FP)"
        else
            echo "  ✓ Clean"
        fi
        
        # Save detailed result
        local safe_name=$(echo "$pkg" | tr '@/' '__' | tr ':' '_')
        echo "$result" > "$RESULTS_DIR/clean-${safe_name}.txt"
    done
    
    echo "False positives: $flagged/$total"
    # Return rate as integer percentage (0-100)
    echo $((flagged * 100 / total))
}

# Run benchmarks
echo ""
TARBALL_RESULTS=$(scan_evidence_tarballs)
TARBALL_RATE=$(echo "$TARBALL_RESULTS" | tail -1)

CLEAN_RESULTS=$(scan_clean)
FP_RATE=$(echo "$CLEAN_RESULTS" | tail -1)

# Calculate combined score
# combined_score = (evidence_rate * 0.6) + ((100 - fp_rate) * 0.4)
EVIDENCE_WEIGHT=60
CLEAN_WEIGHT=40
COMBINED_SCORE=$(( (TARBALL_RATE * EVIDENCE_WEIGHT + (100 - FP_RATE) * CLEAN_WEIGHT) / 100 ))

# Output results
echo ""
echo "=== Benchmark Results ==="
echo "Evidence Detection Rate: ${TARBALL_RATE}%"
echo "False Positive Rate: ${FP_RATE}%"
echo "Combined Score: ${COMBINED_SCORE}"
echo ""

# Helper function to convert to decimal (without bc)
to_decimal() {
    local val=$1
    local int_part=$((val / 100))
    local frac_part=$((val % 100))
    printf "%d.%02d" "$int_part" "$frac_part"
}

# Output JSON for autoresearch
cat << EOF
{
    "evidence_detection_rate": $(to_decimal $TARBALL_RATE),
    "fp_rate": $(to_decimal $FP_RATE),
    "combined_score": $(to_decimal $COMBINED_SCORE),
    "timestamp": "$(date -Iseconds)"
}
EOF

# Save full results
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_FILE="$RESULTS_DIR/benchmark_${TIMESTAMP}.json"

cat << EOF > "$RESULTS_FILE"
{
    "evidence_detection_rate": $(to_decimal $TARBALL_RATE),
    "fp_rate": $(to_decimal $FP_RATE),
    "combined_score": $(to_decimal $COMBINED_SCORE),
    "timestamp": "$(date -Iseconds)",
    "quick_mode": $QUICK_MODE,
    "evidence_tarballs_scanned": ${#EVIDENCE_TARBALLS[@]},
    "clean_packages_scanned": ${#CLEAN_PACKAGES[@]}
}
EOF

echo ""
echo "Results saved to: $RESULTS_FILE"
