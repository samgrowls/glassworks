# Glassworks Campaign System - Developer Handoff

**Version:** 0.12.2
**Date:** March 22, 2026
**Status:** Phase 1 Complete - Ready for Wave 6 Execution

---

## Quick Start for Next Developer

### What You're Stepping Into

The Glassworks Campaign System is a **Rust-based orchestration system** for running large-scale security scanning campaigns. It detects GlassWare-style steganographic attacks in npm packages and GitHub repositories.

### Current State

✅ **Phase 1 Complete:**
- Core campaign infrastructure (event bus, state manager, command channel)
- Wave execution engine with DAG scheduling
- CLI commands for running campaigns
- Wave 6 calibration configuration ready

⏳ **Phase 2 Pending:**
- Campaign resume from checkpoint
- Live command steering (pause, cancel, skip wave)
- Markdown/SARIF report generation

### First Steps

1. **Read this handoff** (you're here!)
2. **Review architecture** - See `HANDOFF/ARCHITECTURE-OVERVIEW.md`
3. **Run Wave 6** - Validate the system works end-to-end
4. **Pick next task** - See `HANDOFF/ROADMAP.md`

---

## Project Structure

```
glassworks/
├── glassware-orchestrator/      # Rust orchestrator (PRIMARY FOCUS)
│   └── src/
│       ├── campaign/            # Campaign system (Phase 1 complete)
│       │   ├── types.rs         # Core types
│       │   ├── event_bus.rs     # Event pub/sub
│       │   ├── state_manager.rs # Queryable state
│       │   ├── command_channel.rs # Command steering
│       │   ├── config.rs        # TOML config parsing
│       │   ├── wave.rs          # Wave execution
│       │   ├── executor.rs      # Campaign executor (DAG)
│       │   └── mod.rs           # Module root
│       ├── cli.rs               # CLI definitions
│       ├── main.rs              # CLI handlers
│       └── ...
│
├── glassware-core/              # Detection engine (stable)
├── campaigns/                   # Campaign configurations
│   └── wave6.toml              # Calibration campaign
├── harness/                     # Python tools (legacy, use Rust)
├── docs/                        # Documentation
└── HANDOFF/                     # This directory
```

---

## Key Concepts

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
│   │ State     │   │ Command     │   │ TUI (Phase 3)│        │
│   │ Manager   │   │ Channel     │   │              │        │
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

## Development Workflow

### Building

```bash
# Debug build
cargo build -p glassware-orchestrator

# Release build
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

# Check status
./target/release/glassware-orchestrator campaign status <case-id>

# List campaigns
./target/release/glassware-orchestrator campaign list
```

---

## Current Issues & Known Limitations

### Phase 2 Stubs

These commands print "not yet implemented":

1. **`campaign resume`** - Needs checkpoint loading logic
2. **`campaign command`** - Needs running campaign handle
3. **`campaign report`** - Needs report generator

### Technical Debt

| Issue | Location | Priority |
|-------|----------|----------|
| Unused `llm` parameters in handlers | `main.rs:cmd_campaign_run` | Low |
| Pre-existing warnings in codebase | Various | Low |
| No SQLite checkpoint persistence | `campaign/` | Medium |
| No TUI | Future feature | Medium |

---

## Next Tasks (Priority Order)

### 1. Run Wave 6 Validation ⏳ **NEXT**

**Goal:** Validate the campaign system works end-to-end.

**Steps:**
```bash
# 1. Run Wave 6
./target/release/glassware-orchestrator campaign run campaigns/wave6.toml

# 2. Verify results
# - Wave 6A: Both malicious packages flagged
# - Wave 6B: Clean packages not flagged
# - Wave 6C: Results vary (hunt mode)

# 3. Document any issues
```

**Expected Duration:** 30-60 minutes

**Success Criteria:**
- Campaign completes without errors
- Known malicious packages detected
- Clean packages not flagged

---

### 2. Implement Campaign Resume (Phase 2)

**Goal:** Resume interrupted campaigns from checkpoint.

**Files to Modify:**
- `src/campaign/executor.rs` - Add checkpoint saving
- `src/main.rs:cmd_campaign_resume` - Implement handler
- `src/campaign/state_manager.rs` - Add state serialization

**Approach:**
1. Save state to SQLite after each wave
2. Load state on resume
3. Skip completed waves
4. Continue from interrupted wave

**Expected Duration:** 2-3 hours

---

### 3. Implement Live Commands (Phase 2)

**Goal:** Send commands to running campaigns.

**Commands:**
- `pause` - Pause execution
- `resume` - Resume paused campaign
- `cancel` - Cancel with checkpoint
- `skip-wave` - Skip current wave

**Files to Modify:**
- `src/main.rs:cmd_campaign_command` - Implement handler
- `src/campaign/executor.rs` - Handle commands during execution

**Approach:**
1. Store running campaign handle in global state
2. Command sends signal via channel
3. Executor checks channel between waves

**Expected Duration:** 2-3 hours

---

### 4. Implement Markdown Reports (Phase 2)

**Goal:** Generate markdown reports for completed campaigns.

**Note:** **SARIF output is already fully implemented** in `formatters/sarif.rs`. Use:
```bash
glassware-orchestrator scan-npm --format sarif express lodash > results.sarif
```

**Files to Create:**
- `src/campaign/report.rs` - Report generator
- `templates/report.md.tera` - Tera template

**Files to Modify:**
- `src/main.rs:cmd_campaign_report` - Implement handler

**Approach:**
1. Use `tera` crate for templating
2. Load campaign state from SQLite
3. Render template with campaign data
4. Output to file or stdout

**Expected Duration:** 3-4 hours

---

### 5. TUI Implementation (Phase 3)

**Goal:** Interactive terminal UI for campaign monitoring.

**See:** `design/RFC-001-TUI-ARCHITECTURE.md`

**Dependencies:**
- `ratatui` - TUI framework
- `crossterm` - Terminal manipulation

**Approach:**
1. Basic skeleton with tabs
2. Subscribe to event bus
3. Render campaign progress
4. Send commands via keyboard

**Expected Duration:** 1-2 days

---

## Architecture Deep Dives

### Event Bus

**Location:** `src/campaign/event_bus.rs`

**Purpose:** Pub/sub system for campaign events.

**Key Types:**
```rust
pub enum CampaignEvent {
    CampaignStarted { case_id, campaign_name, .. },
    WaveProgress { wave_id, scanned, flagged, malicious },
    PackageMalicious { package, version, threat_score },
    // ... more events
}

pub struct EventBus {
    sender: broadcast::Sender<CampaignEvent>,
}
```

**Usage:**
```rust
// Publish event
event_bus.publish(CampaignEvent::WaveProgress { ... });

// Subscribe to events
let mut rx = event_bus.subscribe();
while let Ok(event) = rx.recv().await {
    // Handle event
}
```

---

### State Manager

**Location:** `src/campaign/state_manager.rs`

**Purpose:** Thread-safe queryable campaign state.

**Key Types:**
```rust
pub struct StateManager {
    state: Arc<RwLock<CampaignState>>,
    event_bus: EventBus,
}

pub struct CampaignState {
    case_id: String,
    status: CampaignStatus,
    waves: HashMap<String, WaveState>,
    stats: CampaignStats,
    // ...
}
```

**Usage:**
```rust
// Snapshot state (for TUI/CLI)
let state = state_manager.snapshot().await;

// Update with event
state_manager.update(
    |s| { s.status = CampaignStatus::Running; },
    CampaignEvent::CampaignStarted { ... }
).await;
```

---

### DAG Scheduler

**Location:** `src/campaign/executor.rs`

**Purpose:** Schedule waves based on dependencies.

**Algorithm:**
1. Find waves with no unmet dependencies
2. Group into parallel stage
3. Mark as completed
4. Repeat until all waves scheduled

**Example:**
```
Waves: 6A (no deps), 6B (deps: 6A), 6C (deps: 6A)

Stage 1: [6A]
Stage 2: [6B, 6C]  // Can run in parallel
```

---

## Configuration System

### Campaign TOML

**Location:** `campaigns/*.toml`

**Schema:**
```toml
[campaign]
name = "Campaign Name"
priority = "high"

[settings]
concurrency = 10
rate_limit_npm = 10.0

[settings.llm]
tier1_enabled = true
tier2_threshold = 5.0

[[waves]]
id = "wave_1"
mode = "hunt"
depends_on = []

[[waves.sources]]
type = "packages"
list = ["express@4.19.2"]
```

**Validation:**
- Duplicate wave IDs detected
- Circular dependencies detected
- Source validation (non-empty lists, etc.)

---

## Testing Strategy

### Unit Tests

**Location:** In each module (`#[cfg(test)] mod tests`)

**Coverage:**
- Type conversions
- Event bus pub/sub
- State manager updates
- Config validation
- DAG scheduling

### Integration Tests

**Location:** `tests/` (to be created)

**Planned:**
- End-to-end campaign execution
- Wave dependency resolution
- Command handling

### Manual Testing

**Wave 6:** Primary validation campaign

---

## Environment Setup

### Required

```bash
# Rust 1.70+
rustup update

# Build
cargo build -p glassware-orchestrator --release
```

### Optional (for LLM features)

```bash
# ~/.env
GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
GLASSWARE_LLM_API_KEY="csk-..."
NVIDIA_API_KEY="nvapi-..."
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
| `PHASE-1*.md` | Phase completion reports |

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

---

## Getting Help

### When Stuck

1. **Check this handoff** - Common issues documented
2. **Review architecture docs** - `design/` directory
3. **Read phase reports** - `PHASE-1*.md` files
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
- [ ] Review `ARCHITECTURE-OVERVIEW.md`
- [ ] Build the project: `cargo build`
- [ ] Run Wave 6: `campaign run campaigns/wave6.toml`
- [ ] Review phase completion reports
- [ ] Pick next task from roadmap
- [ ] Update this handoff with learnings

---

**Good luck! The campaign system is solid - Phase 1 is complete and ready for Wave 6 validation.**
