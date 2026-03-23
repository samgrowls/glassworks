# Agent Guidance: What's Already Implemented

**Date:** March 23, 2026
**For:** Binary consolidation agent (v0.20.0-tests-fixed)
**From:** Previous developer (deepest codebase context)
**Purpose:** Prevent reinventing the wheel and correct misconceptions

---

## ⚠️ CRITICAL: Read This First

**You are working with a PRODUCTION-READY system that already has:**

1. ✅ **LLM integration (BOTH tiers)** - Not future work, ALREADY WORKING
2. ✅ **Report generation** - Not future work, ALREADY WORKING
3. ✅ **Checkpoint/resume** - Not future work, ALREADY WORKING
4. ✅ **TUI with drill-down** - Not future work, ALREADY WORKING
5. ✅ **Whitelist system** - NOT CONFIGURED in your wave7-real-hunt.toml!

**DO NOT:**
- ❌ Say "Add --llm flag" as future work (it exists!)
- ❌ Say "Enable checkpointing" as future work (it exists!)
- ❌ Flag moment.js as malicious without checking whitelist
- ❌ Create new configs without looking at existing ones

**DO:**
- ✅ Read this document thoroughly
- ✅ Look at `campaigns/wave6.toml` for proper config examples
- ✅ Read `docs/CAMPAIGN-USER-GUIDE.md` for all working commands
- ✅ Check `HANDOFF/FINAL-SESSION-SUMMARY.md` for complete feature list

---

## 1. LLM Integration - ALREADY COMPLETE ✅

### What Exists

**Tier 1 (Cerebras - Fast Triage):**
```bash
# Campaign-level LLM
glassware campaign run wave6.toml --llm

# Package-specific in TUI
glassware campaign demo
# Navigate to package → Press 'l' → LLM analysis runs
```

**Tier 2 (NVIDIA - Deep Analysis):**
```bash
# Campaign-level deep LLM
glassware campaign run wave6.toml --deep-llm

# Package-specific query in TUI
glassware campaign demo
# Navigate to package → Press 'l' → Press '?' → Type question
```

**Environment Variables (from ~/.env):**
```bash
# Tier 1
GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
GLASSWARE_LLM_API_KEY="csk-..."

# Tier 2
NVIDIA_API_KEY="nvapi-..."
```

### Implementation Details

**Files:**
- `glassware/src/llm.rs` - LLM analyzer (Cerebras + NVIDIA fallback chain)
- `glassware/src/campaign/query/handler.rs` - Package-specific queries
- `glassware/src/tui/app.rs` - TUI integration ('l' and '?' keys)

**CLI Commands:**
- `--llm` - Tier 1 Cerebras during campaign
- `--deep-llm` - Tier 2 NVIDIA during campaign
- `campaign query <case-id> "question"` - Post-campaign LLM query

### What You Said (WRONG)

> "Run with LLM - Add --llm flag for Tier 1 analysis on flagged packages"

**This implies it's future work. IT'S NOT.** It's already implemented and working.

### Correct Guidance

If user wants LLM analysis:
```bash
# During campaign
glassware campaign run wave6.toml --llm

# After campaign (query specific findings)
glassware campaign query <case-id> "Why was this package flagged?"

# In TUI (interactive)
glassware campaign demo
# Then navigate and press 'l' for LLM analysis
```

---

## 2. Report Generation - ALREADY COMPLETE ✅

### What Exists

**Markdown Reports:**
```bash
glassware campaign report <case-id>
# Saves to reports/<case-id>/report.md
```

**SARIF Reports (GitHub Advanced Security):**
```bash
glassware campaign report <case-id> --format sarif
# Saves to reports/<case-id>/report.sarif
```

**JSON Reports:**
```bash
glassware campaign report <case-id> --format json
# Saves to reports/<case-id>/report.json
```

### Implementation Details

**Files:**
- `glassware/src/campaign/report.rs` (392 lines) - Report generator
- `glassware/templates/report.md.tera` - Markdown template
- `glassware/src/formatters/sarif.rs` - SARIF formatter

**Report Sections:**
- Executive summary
- Wave-by-wave results
- Findings by category
- LLM analysis summary (if run)
- Evidence manifest
- Configuration appendix

### Checkpointing

**Already SQLite-based and automatic:**
- Checkpoints saved after each wave
- Stored in `.glassware-checkpoints.db`
- Resume uses same checkpoint DB

**Files:**
- `glassware/src/campaign/checkpoint.rs` - Checkpoint manager
- `glassware/src/checkpoint.rs` - Checkpoint system

### What You Said (WRONG)

> "Generate reports - Enable checkpointing for report generation"

**This implies checkpointing needs to be enabled. IT'S ALREADY ENABLED.** Reports already work.

### Correct Guidance

If user wants reports:
```bash
# After campaign completes
glassware campaign report <case-id>

# With specific format
glassware campaign report <case-id> --format markdown
glassware campaign report <case-id> --format sarif
glassware campaign report <case-id> --format json

# Save to specific file
glassware campaign report <case-id> --output my-report.md
```

---

## 3. Whitelist System - CRITICAL MISSING CONFIG ⚠️

### The Problem

**Your `wave7-real-hunt.toml` is MISSING the whitelist configuration!**

This is why `moment@2.30.1` was flagged with 194 findings. **moment.js is an i18n library** - it naturally contains Unicode characters for internationalization. It should be whitelisted.

### What Exists

**Whitelist configuration in TOML:**
```toml
[settings.whitelist]
# Packages to never flag (locale libraries, etc.)
packages = [
    "moment",
    "moment-timezone",
    "date-fns",
    "dayjs",
    "globalize",
    "prettier",
    "typescript",
    "eslint",
    # i18n/locale packages (naturally contain unicode)
    "i18n",
    "i18n-",
    "i18next",
    "locale",
    "cldr",
    "country-data",
    "timezone",
    "country",
    "globalize",
    "polyglot",
    "babelfish",
    "transliteration"
]

# Crypto libraries (blockchain API calls are legitimate)
crypto_packages = [
    "ethers",
    "web3",
    "viem",
    "wagmi",
    "@solana/web3.js",
    "bitcoinjs-lib",
    "hdkey",
    "@metamask/*"
]

# Build tools (time delays are legitimate for watch mode, debouncing)
build_tools = [
    "webpack",
    "vite",
    "rollup",
    "esbuild",
    "parcel",
    "gulp",
    "grunt",
    "core-js",
    # Web frameworks (complex patterns are legitimate)
    "fastify",
    "express",
    "koa",
    "hapi"
]
```

### Reference: wave6.toml (CORRECT EXAMPLE)

**See `campaigns/wave6.toml` lines 18-50** for proper whitelist configuration.

### What You Should Do

**Fix `wave7-real-hunt.toml` by adding:**

```toml
[settings.whitelist]
packages = [
    "moment",
    "moment-timezone",
    "date-fns",
    "dayjs",
    "i18next",
    "react-intl",
    # Add other known i18n/locale packages
]
```

**Then re-run:**
```bash
glassware campaign run campaigns/wave7-real-hunt.toml
```

**moment.js should NOT be flagged.**

### Known False Positives

These packages commonly trigger false positives due to their nature:

| Package | Reason | Should Be Whitelisted |
|---------|--------|----------------------|
| `moment`, `moment-timezone` | i18n data with Unicode | ✅ Yes |
| `lodash` | Complex patterns, utility code | ✅ Yes |
| `express` | Middleware patterns, i18n | ✅ Yes |
| `react-intl`, `i18next` | Internationalization | ✅ Yes |
| `date-fns`, `dayjs` | Date formatting with Unicode | ✅ Yes |
| `globalize` | Globalization library | ✅ Yes |

---

## 4. TUI Features - ALREADY COMPLETE ✅

### What Exists

**Live Monitoring:**
```bash
glassware campaign monitor <case-id>
# Opens TUI with live progress
```

**Demo Mode:**
```bash
glassware campaign demo
# Opens TUI with sample data
```

**Keyboard Shortcuts:**
| Key | Action |
|-----|--------|
| `q` | Quit TUI |
| `Tab` | Switch tabs (Campaign/Findings/Logs) |
| `p` | Pause campaign |
| `x` | Cancel campaign |
| `r` | Resume campaign |
| `c` | Adjust concurrency |
| `Enter` | Drill down into package |
| `l` | Run LLM analysis on package |
| `?` | Ask question about package |
| `↑/↓` or `j/k` | Navigate lists |

### Implementation Details

**Files:**
- `glassware/src/tui/app.rs` (900+ lines) - TUI application
- `glassware/src/tui/ui.rs` (600+ lines) - TUI rendering
- `glassware/src/campaign/event_bus.rs` - Event subscription

### What You Should Say

If user wants TUI:
```bash
# Demo mode (sample data)
glassware campaign demo

# Live monitoring
glassware campaign monitor <case-id>

# Keyboard shortcuts
# q = Quit, Tab = Switch tabs, l = LLM analysis, ? = Query
```

---

## 5. Where To Find Working Examples

### Campaign Configurations

| File | Purpose | Use This For |
|------|---------|--------------|
| `campaigns/wave6.toml` | **Calibration campaign** | ✅ **REFERENCE FOR PROPER CONFIG** |
| `campaigns/wave7-real-hunt.toml` | Real-world hunt | ⚠️ **MISSING WHITELIST - FIX IT** |
| `config-examples/default.toml` | Default configuration | ✅ Good baseline |
| `config-examples/strict.toml` | Strict detection | For high-security environments |
| `config-examples/ci-cd.toml` | CI/CD integration | For automated scanning |

### Documentation

| Document | Location | Purpose |
|----------|----------|---------|
| **User Guide** | `docs/CAMPAIGN-USER-GUIDE.md` | ✅ **READ FIRST for commands** |
| **Final Summary** | `HANDOFF/FINAL-SESSION-SUMMARY.md` | ✅ Complete feature list |
| **README** | `HANDOFF/README.md` | Developer handoff |
| **Architecture** | `design/CAMPAIGN-ARCHITECTURE.md` | System architecture |

### Code References

| Feature | File | Lines |
|---------|------|-------|
| LLM integration | `glassware/src/llm.rs` | 800+ |
| Report generation | `glassware/src/campaign/report.rs` | 392 |
| Checkpoint system | `glassware/src/campaign/checkpoint.rs` | 200+ |
| TUI application | `glassware/src/tui/app.rs` | 900+ |
| TUI rendering | `glassware/src/tui/ui.rs` | 600+ |
| Event bus | `glassware/src/campaign/event_bus.rs` | 350+ |

---

## 6. Common Mistakes To Avoid

### ❌ DON'T Say "Future Work" For Existing Features

**Wrong:**
> "Add --llm flag for Tier 1 analysis"

**Right:**
> "Use --llm flag for Tier 1 analysis (already implemented)"

**Wrong:**
> "Enable checkpointing for reports"

**Right:**
> "Reports use existing checkpoint system (already enabled)"

---

### ❌ DON'T Flag Known False Positives

**Wrong:**
> "moment@2.30.1 flagged with 194 findings - MALICIOUS!"

**Right:**
> "moment@2.30.1 flagged - this is EXPECTED (i18n library). Add to whitelist."

**Known false positives:**
- `moment`, `moment-timezone` - i18n data
- `lodash` - Complex utility patterns
- `express` - Middleware patterns
- `react-intl`, `i18next` - Internationalization

---

### ❌ DON'T Create New Configs Without Checking Existing Ones

**Wrong:**
> Creating new wave8.toml without looking at wave6.toml

**Right:**
> "Copy wave6.toml as base, modify for specific needs"

**Always check:**
1. `campaigns/wave6.toml` - Proper whitelist, scoring, LLM config
2. `config-examples/default.toml` - Default settings
3. Existing configs before creating new ones

---

### ❌ DON'T Ignore Whitelist Configuration

**Wrong:**
```toml
# wave7-real-hunt.toml (MISSING WHITELIST)
[settings]
concurrency = 15
# ... no whitelist section
```

**Right:**
```toml
# wave6.toml (HAS WHITELIST)
[settings]
concurrency = 10

[settings.whitelist]
packages = [
    "moment",
    "moment-timezone",
    "date-fns",
    # ... etc
]
```

---

## 7. Correct Next Steps

### If You Want To Improve The System

**Priority 1: Fix wave7-real-hunt.toml**
```bash
# Add whitelist section to campaigns/wave7-real-hunt.toml
[settings.whitelist]
packages = [
    "moment",
    "moment-timezone",
    "date-fns",
    "dayjs",
    "i18next",
    "react-intl",
]

# Re-run campaign
glassware campaign run campaigns/wave7-real-hunt.toml
```

**Priority 2: Test LLM Integration**
```bash
# Run with Tier 1 LLM
glassware campaign run campaigns/wave7-real-hunt.toml --llm

# Run with Tier 2 LLM
glassware campaign run campaigns/wave7-real-hunt.toml --deep-llm

# Query specific findings
glassware campaign query <case-id> "Why was moment flagged?"
```

**Priority 3: Generate Reports**
```bash
# After campaign completes
glassware campaign report <case-id>
glassware campaign report <case-id> --format markdown
glassware campaign report <case-id> --format sarif
```

---

## 8. Questions To Ask Before Implementing

**Before saying "We should implement X":**

1. ✅ **Check `HANDOFF/FINAL-SESSION-SUMMARY.md`** - Is it already implemented?
2. ✅ **Check `docs/CAMPAIGN-USER-GUIDE.md`** - Is there a command for it?
3. ✅ **Check `campaigns/wave6.toml`** - Is there a config example?
4. ✅ **Check `glassware/src/`** - Is there code for it?

**Example:**

**Question:** "Should we add LLM support?"

**Check:**
1. `FINAL-SESSION-SUMMARY.md` → "LLM Tier 1 + Tier 2 ✅"
2. `CAMPAIGN-USER-GUIDE.md` → "--llm and --deep-llm flags documented"
3. `glassware/src/llm.rs` → 800+ lines of LLM code

**Answer:** "LLM support already exists. Use --llm or --deep-llm flags."

---

## 9. Summary Checklist

### Before Making Changes

- [ ] Read `HANDOFF/FINAL-SESSION-SUMMARY.md`
- [ ] Read `docs/CAMPAIGN-USER-GUIDE.md`
- [ ] Check `campaigns/wave6.toml` for config example
- [ ] Search `glassware/src/` for existing implementation
- [ ] Ask: "Is this already implemented?"

### Before Flagging Packages

- [ ] Check if package is in whitelist
- [ ] Check if package is i18n/locale library
- [ ] Check if package is build tool
- [ ] Consider: "Is this a known false positive?"

### Before Creating New Configs

- [ ] Look at `campaigns/wave6.toml` first
- [ ] Copy wave6.toml as base
- [ ] Add whitelist section
- [ ] Add LLM configuration
- [ ] Add scoring thresholds

---

## 10. Contact/Questions

**If unsure about something:**

1. **Check documentation first** - `HANDOFF/` and `docs/` directories
2. **Search codebase** - `grep -r "feature name" glassware/src/`
3. **Look at working examples** - `campaigns/wave6.toml`
4. **Ask** - Previous developer has full context

---

**Last Updated:** March 23, 2026
**Version:** 1.0
**Status:** Production ready (v0.20.0-tests-fixed)
