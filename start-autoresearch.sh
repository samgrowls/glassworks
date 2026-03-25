#!/bin/bash
# Start GlassWorm autoresearch session
# 
# This script starts pi-autoresearch in background mode for FP rate tuning
#
# Usage: ./start-autoresearch.sh [--background]
#
# Monitoring:
#   - View log: tail -f autoresearch-session.log
#   - View results: tail -20 autoresearch.jsonl | jq .
#   - View session doc: cat autoresearch.md

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Set NVIDIA API key
export NVIDIA_NIM_API_KEY="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"

# Autoresearch parameters
GOAL="Reduce false positive rate from 10% to <5% while maintaining 100% evidence detection rate"
COMMAND="./benchmarks/fp-success-benchmark.sh --quick"
METRIC="combined_score"
OPTIMIZATION="maximize"

# Create initial prompt for autoresearch
PROMPT="
GlassWorm FP Rate Tuning Session

GOAL: $GOAL

BENCHMARK COMMAND: $COMMAND

METRIC TO OPTIMIZE: $METRIC ($OPTIMIZATION)

CURRENT BASELINE:
- Evidence Detection Rate: 100% (23/23 packages)
- False Positive Rate: 10% (1/10 clean packages)
- Combined Score: 0.96

ROOT CAUSE OF FP:
- moment@2.30.1 is flagged as malicious (threat score: 7.00)
- Cause: InvisibleCharacter detector finding zero-width characters (ZWNJ U+200C, ZWJ U+200D) in locale/i18n data files
- These are LEGITIMATE Unicode characters for Persian/Arabic script rendering

FILES IN SCOPE:
- glassware-core/src/invisible.rs (InvisibleCharacter detector)
- glassware-core/src/scanner.rs (scoring logic)
- glassware/src/scanner.rs (threat score calculation)
- glassware/src/config.rs (configuration)

PREFERRED APPROACH:
1. First experiment: Add file type/path awareness to InvisibleCharacter detector
   - Skip files in locale/i18n directories
   - Skip .json files (often contain i18n data)
   - Skip files with high Unicode density (likely i18n data)
2. Alternative: Adjust scoring to require multiple signal categories
3. Test: Run benchmark and measure combined_score

Start the autoresearch session now.
"

LOG_FILE="$SCRIPT_DIR/autoresearch-session.log"

echo "=== GlassWorm Autoresearch Session ==="
echo "Goal: $GOAL"
echo "Benchmark: $COMMAND"
echo "Metric: $METRIC ($OPTIMIZATION)"
echo "Log file: $LOG_FILE"
echo ""

# Check if running in background mode
if [[ "$1" == "--background" || "$1" == "-b" ]]; then
    echo "Starting in background mode..."
    
    # Create a wrapper script that runs pi with the prompt
    cat > /tmp/autoresearch-runner.sh << 'RUNNER'
#!/bin/bash
cd /home/shva/samgrowls/glassworks-v0.41
export NVIDIA_NIM_API_KEY="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"

# Run pi with autoresearch skill
pi -p "$PROMPT_TEXT" 2>&1
RUNNER
    
    export PROMPT_TEXT="$PROMPT"
    chmod +x /tmp/autoresearch-runner.sh
    
    # Start in background with nohup
    nohup /tmp/autoresearch-runner.sh > "$LOG_FILE" 2>&1 &
    BG_PID=$!
    
    echo "Autoresearch session started in background (PID: $BG_PID)"
    echo ""
    echo "Monitoring commands:"
    echo "  tail -f $LOG_FILE          # View live log"
    echo "  tail -20 autoresearch.jsonl | jq .  # View experiment results"
    echo "  cat autoresearch.md        # View session document"
    echo "  ps aux | grep pi           # Check if running"
    echo ""
    echo "To stop: kill $BG_PID"
    
    # Save PID for later
    echo $BG_PID > /tmp/autoresearch.pid
    
else
    echo "Starting in foreground mode..."
    echo "Press Ctrl+C to interrupt"
    echo ""
    
    # Run pi interactively with the prompt
    pi -p "$PROMPT" 2>&1 | tee "$LOG_FILE"
fi
