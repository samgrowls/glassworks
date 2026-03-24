#!/bin/bash
# Batch LLM investigation of Wave 9 flagged packages

PACKAGES_FILE="/tmp/wave9-flagged-packages.txt"
OUTPUT_FILE="/tmp/wave9-llm-batch-results.txt"

echo "Wave 9 LLM Batch Investigation" > $OUTPUT_FILE
echo "==============================" >> $OUTPUT_FILE
echo "" >> $OUTPUT_FILE

count=0
total=$(wc -l < $PACKAGES_FILE)

while IFS= read -r pkg; do
    count=$((count + 1))
    echo "[$count/$total] Scanning $pkg..."
    
    # Clear cache for fresh scan
    rm -f .glassware-orchestrator-cache.db
    
    # Scan with LLM and extract key info
    result=$(./target/release/glassware scan-npm "$pkg" --llm 2>&1)
    
    # Extract findings count, score, and LLM verdict
    findings=$(echo "$result" | grep "Total findings:" | awk '{print $3}')
    score=$(echo "$result" | grep "Average threat score:" | awk '{print $4}')
    llm_verdict=$(echo "$result" | grep "LLM verdict:" | sed 's/.*LLM verdict: malicious=\([^,]*\), confidence=\([0-9.]*\).*/\1,\2/')
    flagged=$(echo "$result" | grep "Malicious packages:" | awk '{print $3}')
    
    echo "$pkg | findings=$findings | score=$score | LLM=$llm_verdict | flagged=$flagged" >> $OUTPUT_FILE
    echo "$pkg | findings=$findings | score=$score | LLM=$llm_verdict | flagged=$flagged"
    
done < $PACKAGES_FILE

echo "" >> $OUTPUT_FILE
echo "Batch complete!" >> $OUTPUT_FILE

echo ""
echo "Results saved to $OUTPUT_FILE"
echo "Summary:"
grep -E "LLM verdict" $OUTPUT_FILE | awk -F',' '{print $1}' | sort | uniq -c
