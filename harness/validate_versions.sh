#!/bin/bash
# Pre-Scan Version Validator
# 
# Validates package versions before running a wave scan
# Replaces placeholder versions (0.0.0) with latest available
# Skips packages that don't exist
#
# Usage: ./validate_versions.sh <input.txt> [output.txt]
#   input.txt - One package@version per line
#   output.txt - Validated packages (optional, defaults to stdout)

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <input.txt> [output.txt]"
    echo ""
    echo "Validates package versions before scanning."
    echo "Replaces 0.0.0 placeholders with latest version."
    echo "Skips packages that don't exist."
    exit 1
fi

INPUT_FILE="$1"
OUTPUT_FILE="${2:-}"

if [ ! -f "$INPUT_FILE" ]; then
    echo "Error: Input file not found: $INPUT_FILE"
    exit 1
fi

echo "============================================================"
echo "Pre-Scan Version Validator"
echo "============================================================"
echo ""
echo "Input: $INPUT_FILE"
if [ -n "$OUTPUT_FILE" ]; then
    echo "Output: $OUTPUT_FILE"
fi
echo ""

# Counters
TOTAL=0
VALID=0
FIXED=0
SKIPPED=0
ERRORS=0

# Temporary file for output
TEMP_OUTPUT=$(mktemp)

# Read input file line by line
while IFS= read -r line || [ -n "$line" ]; do
    # Skip empty lines and comments
    [[ -z "$line" || "$line" =~ ^# ]] && continue
    
    TOTAL=$((TOTAL + 1))
    
    # Parse package@version
    if [[ "$line" =~ ^(@[^@]+)@(.+)$ ]]; then
        # Scoped package (@org/pkg@version)
        PACKAGE="${BASH_REMATCH[1]}"
        VERSION="${BASH_REMATCH[2]}"
    elif [[ "$line" =~ ^([^@]+)@(.+)$ ]]; then
        # Regular package (pkg@version)
        PACKAGE="${BASH_REMATCH[1]}"
        VERSION="${BASH_REMATCH[2]}"
    else
        # No version specified, use latest
        PACKAGE="$line"
        VERSION=""
    fi
    
    # Check if version is placeholder
    if [ "$VERSION" = "0.0.0" ] || [ -z "$VERSION" ]; then
        # Get latest version
        echo -n "  Fetching latest for $PACKAGE... "
        LATEST=$(npm view "$PACKAGE" version 2>/dev/null || echo "")
        
        if [ -n "$LATEST" ]; then
            echo "$LATEST"
            echo "$PACKAGE@$LATEST" >> "$TEMP_OUTPUT"
            FIXED=$((FIXED + 1))
            VALID=$((VALID + 1))
        else
            echo "NOT FOUND"
            echo "# SKIP: $PACKAGE (not found)" >> "$TEMP_OUTPUT"
            SKIPPED=$((SKIPPED + 1))
        fi
    else
        # Verify version exists
        echo -n "  Verifying $PACKAGE@$VERSION... "
        AVAILABLE=$(npm view "$PACKAGE" versions --json 2>/dev/null | grep -q "\"$VERSION\"" && echo "yes" || echo "no")
        
        if [ "$AVAILABLE" = "yes" ]; then
            echo "OK"
            echo "$PACKAGE@$VERSION" >> "$TEMP_OUTPUT"
            VALID=$((VALID + 1))
        else
            # Try to get latest instead
            echo "NOT FOUND (trying latest)"
            LATEST=$(npm view "$PACKAGE" version 2>/dev/null || echo "")
            
            if [ -n "$LATEST" ]; then
                echo "  Using $PACKAGE@$LATEST instead"
                echo "$PACKAGE@$LATEST" >> "$TEMP_OUTPUT"
                FIXED=$((FIXED + 1))
                VALID=$((VALID + 1))
            else
                echo "  SKIP: Package not found"
                echo "# SKIP: $PACKAGE@$VERSION (not found)" >> "$TEMP_OUTPUT"
                SKIPPED=$((SKIPPED + 1))
            fi
        fi
    fi
done < "$INPUT_FILE"

# Output results
echo ""
echo "============================================================"
echo "VALIDATION SUMMARY"
echo "============================================================"
echo "Total packages: $TOTAL"
echo "Valid: $VALID"
echo "  - Original versions: $((VALID - FIXED))"
echo "  - Fixed versions: $FIXED"
echo "Skipped: $SKIPPED"
echo ""

if [ -n "$OUTPUT_FILE" ]; then
    mv "$TEMP_OUTPUT" "$OUTPUT_FILE"
    echo "Output written to: $OUTPUT_FILE"
else
    echo "Validated packages:"
    echo "------------------------------------------------------------"
    cat "$TEMP_OUTPUT"
    rm "$TEMP_OUTPUT"
fi

echo ""
echo "Success rate: $((VALID * 100 / TOTAL))%"

if [ $SKIPPED -gt 0 ]; then
    echo ""
    echo "Warning: $SKIPPED packages were skipped."
    echo "Review the output file for details."
fi
