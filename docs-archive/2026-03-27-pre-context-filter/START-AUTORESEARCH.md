# Starting GlassWorm Autoresearch Session

**Issue:** pi coding agent requires an interactive terminal session - it cannot run in background mode with nohup.

**Solution:** Run the session interactively in a terminal or tmux/screen session.

---

## Option 1: Run in Current Terminal (Recommended)

### Step 1: Open Terminal and Set Environment

```bash
cd /home/shva/samgrowls/glassworks-v0.41
export NVIDIA_NIM_API_KEY="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"
```

### Step 2: Start pi Agent

```bash
pi --provider nvidia-nim --model "deepseek-ai/deepseek-v3.2"
```

### Step 3: Start Autoresearch Session

When pi starts, type:

```
/skill:autoresearch-create
```

### Step 4: Provide Session Parameters

The agent will ask you questions. Provide these answers:

**Goal:**
```
Reduce false positive rate from 10% to less than 5% while maintaining 100% evidence detection rate
```

**Command:**
```
./benchmarks/fp-success-benchmark.sh --quick
```

**Metric:**
```
combined_score
```

**Optimization:**
```
maximize
```

**Files in scope:**
```
glassware-core/src/invisible.rs glassware-core/src/scanner.rs
```

---

## Option 2: Run in tmux (For Long Sessions)

### Step 1: Start tmux Session

```bash
tmux new -s glassworks-autoresearch
```

### Step 2: Set Environment and Start pi

```bash
cd /home/shva/samgrowls/glassworks-v0.41
export NVIDIA_NIM_API_KEY="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"
pi --provider nvidia-nim --model "deepseek-ai/deepseek-v3.2"
```

### Step 3: Start Autoresearch

```
/skill:autoresearch-create
```

### Step 4: Detach (Session Continues Running)

Press `Ctrl+B` then `D` to detach.

### Step 5: Reattach Later

```bash
tmux attach -t glassworks-autoresearch
```

---

## Option 3: Run in screen

### Step 1: Start screen Session

```bash
screen -S glassworks-autoresearch
```

### Step 2: Set Environment and Start pi

```bash
cd /home/shva/samgrowls/glassworks-v0.41
export NVIDIA_NIM_API_KEY="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"
pi --provider nvidia-nim --model "deepseek-ai/deepseek-v3.2"
```

### Step 3: Start Autoresearch

```
/skill:autoresearch-create
```

### Step 4: Detach

Press `Ctrl+A` then `D` to detach.

### Step 5: Reattach Later

```bash
screen -r glassworks-autoresearch
```

---

## Monitoring Progress

### In Another Terminal

```bash
# Watch experiment results
watch -n 10 'tail -20 /home/shva/samgrowls/glassworks-v0.41/autoresearch.jsonl | jq .'

# Watch session document
watch -n 10 'head -60 /home/shva/samgrowls/glassworks-v0.41/autoresearch.md'

# Check benchmark results
ls -lh /home/shva/samgrowls/glassworks-v0.41/benchmarks/results/
```

### Session Controls (In pi Terminal)

| Key/Command | Action |
|-------------|--------|
| `Escape` | Interrupt loop and request summary |
| `/autoresearch off` | Stop auto-resume, keep logs |
| `/autoresearch clear` | Delete logs and reset |
| `Ctrl+X` | Toggle inline dashboard |
| `Ctrl+Shift+X` | Fullscreen overlay |

---

## Quick Start Command (Single Line)

If you want to start immediately:

```bash
cd /home/shva/samgrowls/glassworks-v0.41 && export NVIDIA_NIM_API_KEY="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" && pi --provider nvidia-nim --model "deepseek-ai/deepseek-v3.2"
```

Then type: `/skill:autoresearch-create`

---

## Expected Flow

1. **Agent initializes** - Reads `autoresearch.md` and `autoresearch.config.json`
2. **Runs baseline** - Executes benchmark to confirm current performance
3. **Starts experiments** - Begins making code changes and testing
4. **Logs results** - Each experiment logged to `autoresearch.jsonl`
5. **Continues iterating** - Up to 30 iterations (configured in `autoresearch.config.json`)

---

## Session Files

| File | Purpose |
|------|---------|
| `autoresearch.md` | Session document (objectives, progress) |
| `autoresearch.jsonl` | Experiment results (JSONL format) |
| `autoresearch.config.json` | Configuration (max iterations, benchmark) |
| `autoresearch-session.log` | Manual log file (if redirecting output) |
| `benchmarks/results/*.json` | Individual benchmark results |

---

## Troubleshooting

### "No models available"

```bash
# Ensure NVIDIA_NIM_API_KEY is set
export NVIDIA_NIM_API_KEY="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"

# Verify extension is loaded
pi list

# Should show pi-nvidia-nim
```

### "Skill not found"

```bash
# Check skill directory
ls -la ~/.pi/agent/skills/autoresearch-create/

# Reinstall if needed
pi install https://github.com/davebcn87/pi-autoresearch
```

### Benchmark fails

```bash
# Test benchmark manually
./benchmarks/fp-success-benchmark.sh --quick

# Check binary
ls -la target/release/glassware
```

---

## Current Baseline (For Reference)

| Metric | Value |
|--------|-------|
| Evidence Detection | 100% (23/23) |
| False Positive | 10% (1/10) - moment.js |
| Combined Score | 0.96 |

**Target:** FP rate < 5%, Combined score > 0.98

---

**Last Updated:** 2026-03-25 15:25 UTC  
**Status:** Ready for interactive session start
