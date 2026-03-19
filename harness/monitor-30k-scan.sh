#!/bin/bash
# 30k Scan Monitor Script
# Usage: ./monitor-30k-scan.sh

LOG_FILE="scan-30k-batch1.log"
TOTAL=2242

echo "=== 30k Scan Monitor ==="
echo "Updated: $(date)"
echo ""

if [ ! -f "$LOG_FILE" ]; then
    echo "❌ Log file not found. Scan may not have started."
    exit 1
fi

# Get current progress
CURRENT=$(grep -c "^\[" "$LOG_FILE" 2>/dev/null || echo "0")
PERCENT=$((CURRENT * 100 / TOTAL))

# Get last scanned package
LAST=$(tail -1 "$LOG_FILE" | grep -oE '\[[0-9]+/[0-9]+\].*' | head -1)

# Count categories
FLAGGED=$(grep "⚠️" "$LOG_FILE" | wc -l)
CACHED=$(grep "💾" "$LOG_FILE" | wc -l)
ERRORS=$(grep "❌" "$LOG_FILE" | wc -l)

# Calculate rates
if [ $CURRENT -gt 0 ]; then
    FLAGGED_RATE=$((FLAGGED * 100 / CURRENT))
    CACHE_RATE=$((CACHED * 100 / CURRENT))
    ERROR_RATE=$((ERRORS * 100 / CURRENT))
else
    FLAGGED_RATE=0
    CACHE_RATE=0
    ERROR_RATE=0
fi

# Estimate remaining time (rough estimate based on start time)
START_TIME=$(stat -c %Y "$LOG_FILE" 2>/dev/null || echo "0")
NOW=$(date +%s)
ELAPSED=$((NOW - START_TIME))
if [ $CURRENT -gt 0 ] && [ $ELAPSED -gt 0 ]; then
    SECS_PER_PKG=$((ELAPSED / CURRENT))
    REMAINING=$((TOTAL - CURRENT))
    ETA_MINS=$((REMAINING * SECS_PER_PKG / 60))
else
    ETA_MINS="unknown"
fi

echo "📊 Progress:"
echo "  Scanned: $CURRENT / $TOTAL ($PERCENT%)"
echo "  Last: $LAST"
echo ""

echo "📈 Metrics:"
echo "  Flagged: $FLAGGED ($FLAGGED_RATE%)"
echo "  Cached: $CACHED ($CACHE_RATE%)"
echo "  Errors: $ERRORS ($ERROR_RATE%)"
echo ""

echo "⏱️  ETA:"
echo "  Remaining: ~$ETA_MINS minutes"
echo ""

echo "🔍 Quick Commands:"
echo "  Watch: watch -n 30 './monitor-30k-scan.sh'"
echo "  Log: tail -f $LOG_FILE"
echo "  Flagged: grep '⚠️' $LOG_FILE | tail -20"
echo ""

# Show recent flagged packages
if [ $FLAGGED -gt 0 ]; then
    echo "🚨 Recent Flagged:"
    grep "⚠️" "$LOG_FILE" | tail -5
fi
