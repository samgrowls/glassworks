# Glassworks Campaign System - Developer Handoff

**Version:** 0.15.0-final
**Date:** March 23, 2026
**Status:** ✅ Production Ready - All Features Complete

---

## Quick Start for Next Developer

### What You're Stepping Into

The Glassworks Campaign System is a **production-ready Rust-based orchestration system** for running large-scale security scanning campaigns. It detects GlassWare-style steganographic attacks in npm packages and GitHub repositories.

### Current State

✅ **All Features Complete:**
- Core campaign infrastructure (event bus, state manager, command channel)
- Wave execution engine with DAG scheduling
- Checkpoint save/load with SQLite
- Campaign resume with skip completed waves
- Markdown report generation with Tera templates
- LLM query CLI (natural language questions)
- TUI with live monitoring
- TUI command palette (p=pause, x=cancel, c=concurrency)
- TUI package drill-down with LLM analysis
- Package-specific queries

### First Steps

1. **Read this handoff** (you're here!)
2. **Review architecture** - See `HANDOFF/ARCHITECTURE-OVERVIEW.md`
3. **Review future roadmap** - See `HANDOFF/FUTURE/ROADMAP-2026.md`
4. **Pick first task** - Binary consolidation (see below)

---

## Immediate Next Task: Binary Consolidation

**Priority:** HIGH
**Timeline:** 3 weeks
**See:** `HANDOFF/FUTURE/BINARY-CONSOLIDATION.md`

### Current State

| Binary | Size | Purpose |
|--------|------|---------|
| `glassware` | ~11MB | Simple directory/file scanner |
| `glassware-orchestrator` | ~25MB | Full campaign orchestration |

**Total:** 2 binaries, ~36MB

### Goal

Consolidate into **one unified binary** with subcommands:
```bash
glassware scan /path/to/code          # Old glassware-cli
glassware campaign run wave6.toml     # Old orchestrator
glassware campaign demo               # TUI demo
glassware campaign query ...          # LLM queries
```

**Expected outcomes:**
- Binary size: ~10-15MB (-60%)
- Memory usage: ~25-35MB (-40%)
- Scan speed: ~65k LOC/sec (+30%)

### Implementation Plan

**Week 1:** Feature audit, code consolidation
**Week 2:** Size optimization (LTO, features, dependencies)
**Week 3:** Performance optimization (memory, speed)

**Resources:** 1 developer, 3 weeks
**Risk:** Low (incremental migration)

---

## Project Structure

```
glassworks/
├── glassware-core/              # Library (detection engine)
│   └── src/
│       ├── detector.rs          # Detector trait
│       ├── finding.rs           # Finding types
│       ├── engine.rs            # Scan engine
│       └── ...
│
├── glassware-cli/               # Binary 1 (to be consolidated)
│   └── src/main.rs              # Simple scanner (~11MB)
│
├── glassware-orchestrator/      # Binary 2 (primary, to be consolidated)
│   └── src/
│       ├── campaign/            # Campaign system
│       │   ├── types.rs         # Core types
│       │   ├── event_bus.rs     # Event pub/sub
│       │   ├── state_manager.rs # Queryable state
│       │   ├── command_channel.rs # Command steering
│       │   ├── config.rs        # TOML config parsing
│       │   ├── wave.rs          # Wave execution
│       │   ├── executor.rs      # Campaign executor (DAG)
│       │   ├── checkpoint.rs    # Checkpoint persistence
│       │   ├── report.rs        # Report generation
│       │   ├── query/           # LLM queries
│       │   └── mod.rs           # Module root
│       ├── tui/                 # TUI implementation
│       │   ├── app.rs           # TUI application
│       │   └── ui.rs            # TUI rendering
│       ├── cli.rs               # CLI definitions
│       └── main.rs              # CLI handlers
│
├── campaigns/                   # Campaign configurations
│   └── wave6.toml              # Calibration campaign
├── docs/                        # User documentation
└── HANDOFF/                     # Developer documentation
    ├── README.md                # This file
    ├── FINAL-SESSION-SUMMARY.md # Session summary
    ├── WEEK1-SUMMARY.md         # Week 1 progress
    ├── WEEK2-FINAL-SUMMARY.md   # Week 2 progress
    ├── FUTURE/                  # Future planning
    │   ├── ROADMAP-2026.md      # Strategic roadmap
    │   ├── PLUGIN-ARCHITECTURE.md # Plugin design
    │   ├── LONG-RUNNING-CAMPAIGNS.md # Long-run features
    │   ├── AUTO-RESEARCH-TUNING.md # Auto-tuning design
    │   └── BINARY-CONSOLIDATION.md # Binary consolidation
    └── ...
```

---

## Complete Feature Set

### Campaign Management

| Command | Description | Status |
|---------|-------------|--------|
| `campaign run <config>` | Run campaign from TOML | ✅ Production |
| `campaign resume <case-id>` | Resume interrupted | ✅ Production |
| `campaign list` | List recent campaigns | ✅ Production |
| `campaign status <case-id>` | Show status | ✅ Production |
| `campaign monitor <case-id>` | Live TUI monitoring | ✅ Production |
| `campaign demo` | TUI demo with sample data | ✅ Production |

### Analysis & Reporting

| Command | Description | Status |
|---------|-------------|--------|
| `campaign report <case-id>` | Generate markdown report | ✅ Production |
| `campaign query <case-id> "question"` | Ask about campaign | ✅ Production |
| `campaign command <case-id> <cmd>` | Send steering commands | ✅ Production |

### TUI Features

| Feature | Description | Status |
|---------|-------------|--------|
| Live progress | Real-time progress bar with ETA | ✅ Production |
| Event feed | Scrolling recent events | ✅ Production |
| Tab navigation | Campaign/Findings/Logs tabs | ✅ Production |
| Command palette | Keyboard shortcuts | ✅ Production |
| Concurrency dialog | Adjust concurrency mid-campaign | ✅ Production |
| Package drill-down | View specific package details | ✅ Production |
| Package LLM analysis | Run LLM on selected package | ✅ Production |
| Package query | Ask questions about package | ✅ Production |

---

## TUI Keyboard Shortcuts

### Global
| Key | Action |
|-----|--------|
| `q` | Quit TUI |
| `Ctrl+C` | Pause campaign |
| `Tab` | Switch tabs |

### Campaign Control
| Key | Action |
|-----|--------|
| `p` | Pause/Resume |
| `x` | Cancel campaign |
| `r` | Resume campaign |
| `s` | Skip wave |
| `c` | Adjust concurrency |

### Package Drill-Down (Findings Tab)
| Key | Action |
|-----|--------|
| `Enter` | Open package detail view |
| `l` | Run LLM analysis on package |
| `↑/↓` or `j/k` | Navigate package list |
| `?` | Ask question about package |
| `q`/`Esc` | Close detail view |

---

## Development Workflow

### Building

```bash
# Debug build
cargo build -p glassware-orchestrator

# Release build (optimized)
cargo build -p glassware-orchestrator --release

# Check only (fast)
cargo check -p glassware-orchestrator
```

### Testing

```bash
# Run campaign module tests
cargo test -p glassware-orchestrator campaign

# Run all tests
cargo test --features "full,llm"
```

### Running Campaigns

```bash
# Run Wave 6
./target/release/glassware-orchestrator campaign run campaigns/wave6.toml

# With LLM triage
./target/release/glassware-orchestrator campaign run campaigns/wave6.toml --llm

# TUI demo
./target/release/glassware-orchestrator campaign demo

# Monitor live campaign
./target/release/glassware-orchestrator campaign monitor <case-id>

# Generate report
./target/release/glassware-orchestrator campaign report <case-id>

# Query campaign
./target/release/glassware-orchestrator campaign query <case-id> "Why was express flagged?"
```

---

## Architecture Overview

### Campaign Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Campaign Executor                         │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐ │
│  │ Wave 6A      │────►│ Wave 6B      │────►│ Wave 6C      │ │
│  │ (validate)   │     │ (validate)   │     │ (hunt)       │ │
│  └──────────────┘     └──────────────┘     └──────────────┘ │
│         │                    │                    │          │
│         └────────────────────┴────────────────────┘          │
│                           │                                  │
│                  ┌────────▼────────┐                         │
│                  │ Event Bus       │                         │
│                  │ (pub/sub)       │                         │
│                  └────────┬────────┘                         │
│                           │                                  │
│         ┌─────────────────┼─────────────────┐               │
│         │                 │                 │               │
│   ┌─────▼─────┐   ┌──────▼──────┐   ┌──────▼──────┐        │
│   │ State     │   │ Command     │   │    TUI      │        │
│   │ Manager   │   │ Channel     │   │             │        │
│   └───────────┘   └─────────────┘   └──────────────┘        │
└─────────────────────────────────────────────────────────────┘
```

### Two-Tier LLM Strategy

| Tier | Provider | Speed | Purpose |
|------|----------|-------|---------|
| **Tier 1** | Cerebras | ~2-5s/pkg | Fast triage during scan |
| **Tier 2** | NVIDIA | ~15-30s/pkg | Deep analysis of flagged |

### Wave Dependencies (DAG)

```toml
[[waves]]
id = "wave_6a"
depends_on = []  # Runs first

[[waves]]
id = "wave_6b"
depends_on = ["wave_6a"]  # Runs after 6a

[[waves]]
id = "wave_6c"
depends_on = ["wave_6a"]  # Runs parallel with 6b
```

---

## Known Limitations

| Limitation | Impact | Workaround |
|------------|--------|------------|
| No partial wave resume | Mid-wave interruption loses progress | Run smaller waves |
| No checkpoint cleanup | Database grows over time | Manual cleanup: `rm .glassware-checkpoints.db` |
| LLM query single-shot | No conversational follow-up | Ask complete questions |
| Two binaries | User confusion, larger download | See BINARY-CONSOLIDATION.md |

---

## Environment Variables

```bash
# Tier 1 LLM (Cerebras)
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="csk-..."

# Tier 2 LLM (NVIDIA)
export NVIDIA_API_KEY="nvapi-..."

# GitHub (private repos)
export GITHUB_TOKEN="ghp_..."
```

---

## Debugging Tips

### Enable Debug Logging

```bash
RUST_LOG=debug ./target/release/glassware-orchestrator campaign run campaigns/wave6.toml
```

### Check Campaign State

```bash
# List recent campaigns
./target/release/glassware-orchestrator campaign list

# Show status
./target/release/glassware-orchestrator campaign status <case-id>
```

### Common Issues

**"Campaign not found":**
- Case ID is wrong
- Campaign was cleaned up

**"Circular dependency detected":**
- Check wave `depends_on` fields
- Use `cargo test` to validate config

**LLM analysis fails:**
- Check API keys in `~/.env`
- Try without `--llm` flag first

---

## Resources

### Documentation

| Document | Purpose |
|----------|---------|
| `docs/CAMPAIGN-USER-GUIDE.md` | End-user guide |
| `design/CAMPAIGN-ARCHITECTURE.md` | Technical architecture |
| `design/RFC-001-TUI-ARCHITECTURE.md` | TUI design |
| `HANDOFF/FINAL-SESSION-SUMMARY.md` | Complete session summary |
| `HANDOFF/FUTURE/ROADMAP-2026.md` | Strategic roadmap |

### Code References

| Module | Lines | Tests |
|--------|-------|-------|
| `campaign/types.rs` | 450 | 5 |
| `campaign/event_bus.rs` | 350 | 4 |
| `campaign/state_manager.rs` | 450 | 4 |
| `campaign/command_channel.rs` | 400 | 5 |
| `campaign/config.rs` | 740 | 7 |
| `campaign/wave.rs` | 400 | 3 |
| `campaign/executor.rs` | 550 | 2 |
| `campaign/report.rs` | 392 | 2 |
| `campaign/query/handler.rs` | 200+ | 2 |
| `tui/app.rs` | 500+ | 4 |
| `tui/ui.rs` | 400+ | 3 |

---

## Getting Help

### When Stuck

1. **Check this handoff** - Common issues documented
2. **Review architecture docs** - `design/` directory
3. **Read session summary** - `HANDOFF/FINAL-SESSION-SUMMARY.md`
4. **Ask for clarification** - Context may be missing

### Context Preservation

This handoff is designed to preserve context for:
- Architecture decisions
- Implementation patterns
- Known issues
- Next steps

If something is unclear, it's a gap in this document - please note it for improvement.

---

## Checklist for Next Developer

- [ ] Read this handoff document
- [ ] Review `FINAL-SESSION-SUMMARY.md`
- [ ] Review `FUTURE/ROADMAP-2026.md`
- [ ] Review `FUTURE/BINARY-CONSOLIDATION.md`
- [ ] Build the project: `cargo build --release`
- [ ] Test TUI: `campaign demo`
- [ ] Start first task: Binary consolidation

---

**Good luck! The system is production-ready with comprehensive documentation. The next major initiative is binary consolidation - see `HANDOFF/FUTURE/BINARY-CONSOLIDATION.md` for details.**
