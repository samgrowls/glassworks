#!/bin/bash
# Monitor Wave 10 campaign progress
# Usage: ./monitor-wave10.sh

LOG_FILE="/home/shva/samgrowls/glassworks-v0.57.0-longwave/wave10-campaign.log"

echo "=== Wave 10 Campaign Monitor ==="
echo "Time: $(date)"
echo ""

# Check if process is running
if pgrep -f "glassware.*wave10" > /dev/null; then
    echo "Status: RUNNING ✅"
    pgrep -f "glassware.*wave10" | head -1 > /tmp/wave10.pid
else
    echo "Status: NOT RUNNING"
    if [ -f /tmp/wave10.pid ]; then
        echo "Last PID: $(cat /tmp/wave10.pid)"
    fi
fi

echo ""
echo "=== Log File ==="
if [ -f "$LOG_FILE" ]; then
    ls -lh "$LOG_FILE"
    echo ""
    echo "=== Last 30 lines ==="
    tail -30 "$LOG_FILE"
else
    echo "Log file not found: $LOG_FILE"
fi

echo ""
echo "=== Quick Stats ==="
if [ -f "$LOG_FILE" ]; then
    grep -c "scanned:" "$LOG_FILE" 2>/dev/null | xargs echo "Packages scanned:"
    grep -c "Wave.*completed" "$LOG_FILE" 2>/dev/null | xargs echo "Waves completed:"
    grep -c "malicious" "$LOG_FILE" 2>/dev/null | xargs echo "Malicious detected:"
fi

echo ""
echo "=== To Follow Logs ==="
echo "tail -f $LOG_FILE"
