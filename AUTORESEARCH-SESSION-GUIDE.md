# GlassWorm Autoresearch Session Guide

**Version:** v0.56.0  
**Date:** 2026-03-25

---

## Quick Start

### 1. Open Terminal and Navigate

```bash
cd /home/shva/samgrowls/glassworks-v0.41
export NVIDIA_NIM_API_KEY="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"
```

### 2. Start pi Agent

```bash
pi
```

### 3. Start Autoresearch Session

In the pi interactive session, type:

```
/skill:autoresearch-create
```

The agent will prompt you for:
- **Goal:** `Reduce false positive rate from 10% to <5% while maintaining 100% evidence detection`
- **Command:** `./benchmarks/fp-success-benchmark.sh --quick`
- **Metric:** `combined_score`
- **Files in scope:** `glassware-core/src/invisible.rs, glassware-core/src/scanner.rs`

Or provide all at once:

```
/autoresearch Reduce false positive rate from 10% to <5% while maintaining 100% evidence detection, ./benchmarks/fp-success-benchmark.sh --quick, combined_score maximize, glassware-core/src/invisible.rs glassware-core/src/scanner.rs
```

---

## Session Monitoring

### Live Log (Background Session)

If running in background with nohup:

```bash
tail -f autoresearch-session.log
```

### Experiment Results

```bash
# View latest experiments
tail -20 autoresearch.jsonl | jq .

# Count experiments
wc -l autoresearch.jsonl

# View best result
cat autoresearch.jsonl | jq -s 'max_by(.combined_score)'
```

### Session Document

```bash
# View session progress
cat autoresearch.md

# View in real-time
watch -n 5 'head -50 autoresearch.md'
```

### Process Status

```bash
# Check if pi is running
ps aux | grep "pi " | grep -v grep

# Check background session PID
cat /tmp/autoresearch.pid 2>/dev/null
```

---

## Session Controls

### In Interactive Mode

| Command | Action |
|---------|--------|
| `Escape` | Interrupt loop and request summary |
| `/autoresearch off` | Stop auto-resume, keep logs |
| `/autoresearch clear` | Delete logs and reset |
| `/autoresearch continue` | Resume session |
| `Ctrl+X` | Toggle inline dashboard |
| `Ctrl+Shift+X` | Fullscreen overlay |

### Background Session

```bash
# Start background session
nohup pi -p "Your prompt" > autoresearch-session.log 2>&1 &

# Save PID
echo $! > /tmp/autoresearch.pid

# Stop session
kill $(cat /tmp/autoresearch.pid)

# View output
tail -f autoresearch-session.log
```

---

## Current Session Details

### Goal

Reduce FP rate from 10% to <5% while maintaining 100% evidence detection

### Baseline

| Metric | Value |
|--------|-------|
| Evidence Detection | 100% (23/23) |
| False Positive | 10% (1/10) |
| Combined Score | 0.96 |

### Root Cause

**moment@2.30.1** flagged due to InvisibleCharacter detector finding legitimate Unicode (ZWNJ, ZWJ) in locale files.

### Files in Scope

1. `glassware-core/src/invisible.rs` - InvisibleCharacter detector
2. `glassware-core/src/scanner.rs` - Scoring logic
3. `glassware/src/scanner.rs` - Threat score calculation

### Preferred Approach

1. Add file type awareness to InvisibleCharacter detector
   - Skip `.json` files
   - Skip `/locale/`, `/locales/`, `/i18n/` directories
   - Skip files with high Unicode density
2. Test with benchmark
3. Iterate based on results

---

## Troubleshooting

### Authentication Error

```
400 checking third-party user token: bad request
```

**Solution:** Run `pi` interactively first to authenticate.

### Benchmark Fails

```bash
# Check binary exists
ls -la target/release/glassware

# Test benchmark manually
./benchmarks/fp-success-benchmark.sh --quick
```

### Extension Not Loading

```bash
# Check extensions
pi list

# Should show:
# User packages:
#   file:~/.pi/agent/extensions/pi-autoresearch
#   file:~/.pi/agent/extensions/pi-nvidia-nim
```

### Session Stuck

```bash
# Check process
ps aux | grep "pi " | grep -v grep

# Kill if stuck
kill -9 <PID>

# Clean up and restart
rm -f autoresearch.jsonl
# Keep autoresearch.md for context
```

---

## Expected Timeline

| Phase | Duration | Outcome |
|-------|----------|---------|
| Baseline run | 5 min | Confirm 0.96 combined score |
| Experiment 1-3 | 15-30 min | File type filtering |
| Experiment 4-6 | 15-30 min | Unicode density threshold |
| Experiment 7-10 | 30-45 min | Score tuning |
| Validation | 10 min | Full benchmark run |

**Total:** ~1.5-2 hours for complete cycle

---

## Success Criteria

### Phase 1 Complete

- [ ] FP rate < 5% (0/10 or 1/20 clean packages)
- [ ] Evidence detection maintained at 100%
- [ ] Combined score > 0.98

### Phase 2 Complete

- [ ] Full benchmark (20 clean packages) passes
- [ ] Changes committed to git
- [ ] Session documented in autoresearch.md

---

## Post-Session Actions

### 1. Review Results

```bash
# View all experiments
cat autoresearch.jsonl | jq -s '.[] | {iteration, combined_score, fp_rate, description}'

# Find best experiment
cat autoresearch.jsonl | jq -s 'max_by(.combined_score)'
```

### 2. Validate Changes

```bash
# Run full benchmark
./benchmarks/fp-success-benchmark.sh

# Run Wave 10 campaign (if available)
./target/release/glassware campaign run campaigns/wave10-1000plus.toml
```

### 3. Commit Changes

```bash
git add -A
git commit -m "Tune InvisibleCharacter detector: reduce FP rate from 10% to <5%"
git tag v0.56.1-fp-tuned
git push origin v0.56.1-fp-tuned
```

### 4. Update Documentation

- Update `AUTORESEARCH-READY.md` with results
- Update `HANDOFF/` documentation
- Add experiment summary to session log

---

## Contact & Resources

- **Session files:** `autoresearch.md`, `autoresearch.jsonl`, `autoresearch.config.json`
- **Benchmark:** `benchmarks/fp-success-benchmark.sh`
- **Evidence:** `evidence/` directory (23 tarballs)
- **Logs:** `autoresearch-session.log`, `benchmarks/results/`

---

**Last Updated:** 2026-03-25 15:15 UTC  
**Status:** Ready to start session
