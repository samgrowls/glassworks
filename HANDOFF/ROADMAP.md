# Roadmap

**Version:** 0.12.2
**Date:** March 22, 2026

---

## Current Status

✅ **Phase 1 Complete** - Core campaign system implemented
⏳ **Wave 6 Pending** - Validation campaign ready to run
⏳ **Phase 2 Planned** - Resume, commands, reports

---

## Immediate Next Steps

### 1. Run Wave 6 Validation ⏳ **DO THIS FIRST**

**Goal:** Validate the campaign system works end-to-end.

**Command:**
```bash
./target/release/glassware-orchestrator campaign run campaigns/wave6.toml
```

**Expected Results:**
- Campaign completes without errors
- Wave 6A: 2/2 malicious packages flagged
- Wave 6B: 0/5 clean packages flagged
- Wave 6C: Variable (hunt mode)

**Duration:** 30-60 minutes

**If Issues:**
- Document in `HANDOFF/WAVE6-ISSUES.md`
- Fix and re-run

---

## Phase 2: Complete Campaign System

### 2.1 Campaign Resume

**Status:** ⏳ Not Started
**Priority:** High
**Estimated:** 2-3 hours

**Description:** Resume interrupted campaigns from checkpoint.

**Tasks:**
- [ ] Add checkpoint saving after each wave
- [ ] Implement checkpoint loading
- [ ] Skip completed waves on resume
- [ ] Update `cmd_campaign_resume` handler

**Files:**
- `src/campaign/executor.rs`
- `src/campaign/state_manager.rs`
- `src/main.rs`

**Acceptance Criteria:**
- Can resume campaign after Ctrl+C
- Completed waves are skipped
- Progress continues from interruption point

---

### 2.2 Live Command Steering

**Status:** ⏳ Not Started
**Priority:** High
**Estimated:** 2-3 hours

**Description:** Send commands to running campaigns.

**Commands:**
- `pause` - Pause execution
- `resume` - Resume paused campaign
- `cancel` - Cancel with checkpoint
- `skip-wave <wave-id>` - Skip current wave
- `set-concurrency <N>` - Adjust concurrency
- `set-rate-limit <N>` - Adjust rate limit

**Tasks:**
- [ ] Store running campaign handle
- [ ] Implement command dispatch
- [ ] Handle commands in executor loop
- [ ] Update `cmd_campaign_command` handler

**Files:**
- `src/campaign/executor.rs`
- `src/main.rs`

**Acceptance Criteria:**
- Can pause/resume running campaign
- Can cancel with checkpoint
- Can skip waves mid-execution

---

### 2.3 Markdown Reports

**Status:** ⏳ Not Started
**Priority:** Medium
**Estimated:** 3-4 hours

**Description:** Generate markdown reports for completed campaigns.

**Tasks:**
- [ ] Add `tera` dependency for templating
- [ ] Create report template
- [ ] Implement report generator
- [ ] Update `cmd_campaign_report` handler

**Files:**
- `src/campaign/report.rs` (new)
- `templates/report.md.tera` (new)
- `src/main.rs`

**Template Sections:**
- Executive summary
- Wave results
- LLM analysis summary
- Findings by category
- Evidence manifest
- Appendix

**Acceptance Criteria:**
- Can generate markdown report
- Report includes all sections
- Report saved to `reports/<case-id>/`

---

## Phase 3: TUI Implementation

### 3.1 TUI Skeleton

**Status:** ⏳ Not Started
**Priority:** Medium
**Estimated:** 4-6 hours

**Description:** Basic TUI framework with tabs.

**Tasks:**
- [ ] Add `ratatui` and `crossterm` dependencies
- [ ] Create TUI app structure
- [ ] Implement tab navigation
- [ ] Subscribe to event bus
- [ ] Render basic state

**Files:**
- `src/tui/` (new directory)
- `src/main.rs` (add TUI command)

**Tabs:**
- Campaign (overview, waves, packages)
- Findings (list, detail view)
- Evidence (manifest, chain of custody)
- Logs (event log)

**Acceptance Criteria:**
- TUI launches with `campaign tui`
- Can switch between tabs
- Shows campaign state

---

### 3.2 Live Progress

**Status:** ⏳ Not Started
**Priority:** Medium
**Estimated:** 4-6 hours

**Description:** Real-time progress display.

**Tasks:**
- [ ] Subscribe to event bus
- [ ] Update progress bars
- [ ] Show active package
- [ ] Display recent events
- [ ] Calculate and show ETA

**Files:**
- `src/tui/app.rs`
- `src/tui/widgets/progress.rs` (new)

**Acceptance Criteria:**
- Progress bars update in real-time
- Active package shown
- ETA calculated and displayed

---

### 3.3 Interactive Commands

**Status:** ⏳ Not Started
**Priority:** Low
**Estimated:** 4-6 hours

**Description:** Send commands from TUI.

**Tasks:**
- [ ] Keyboard shortcuts (p=pause, r=resume, etc.)
- [ ] Command palette
- [ ] Confirmation dialogs
- [ ] Command feedback

**Files:**
- `src/tui/app.rs`
- `src/tui/widgets/command_palette.rs` (new)

**Keyboard Shortcuts:**
- `p` - Pause/Resume
- `x` - Cancel
- `s` - Skip wave
- `c` - Adjust concurrency
- `?` - Help

**Acceptance Criteria:**
- Can send commands via keyboard
- Command feedback shown
- Help screen available

---

## Future Enhancements

### 4.1 Multi-Campaign Orchestration

**Status:** 📋 Planned
**Priority:** Low
**Estimated:** 1-2 days

**Description:** Run multiple campaigns simultaneously.

**Features:**
- Campaign queue
- Priority scheduling
- Resource allocation

---

### 4.2 Distributed Scanning

**Status:** 📋 Planned
**Priority:** Low
**Estimated:** 3-5 days

**Description:** Distribute scanning across multiple nodes.

**Features:**
- Worker nodes
- Task distribution
- Result aggregation

---

### 4.3 Web Dashboard

**Status:** 📋 Planned
**Priority:** Low
**Estimated:** 1-2 weeks

**Description:** Web-based monitoring dashboard.

**Features:**
- Real-time progress
- Historical data
- Campaign management

---

## Technical Debt

| Issue | Priority | Estimated |
|-------|----------|-----------|
| Unused `llm` parameters | Low | 30 min |
| Pre-existing warnings | Low | 1 hour |
| No SQLite checkpoint persistence | Medium | 4 hours |
| No integration tests | Medium | 1 day |

---

## Known Limitations

| Limitation | Impact | Workaround |
|------------|--------|------------|
| No resume | Must restart on failure | Run smaller campaigns |
| No live commands | Can't steer running campaigns | Plan ahead |
| No markdown reports | Console output only | Use JSON format |
| No TUI | CLI only | Use `--format json` for scripting |

---

## Success Metrics

### Phase 2

- [ ] Wave 6 completes successfully
- [ ] Resume works after interruption
- [ ] Commands can steer running campaigns
- [ ] Markdown reports generated

### Phase 3

- [ ] TUI launches and displays state
- [ ] Progress updates in real-time
- [ ] Commands can be sent via keyboard

### Overall

- [ ] Can run 500+ package campaigns
- [ ] Can resume interrupted campaigns
- [ ] Can monitor progress in TUI
- [ ] Can generate reports for stakeholders

---

## Timeline

| Phase | Start | End | Status |
|-------|-------|-----|--------|
| Phase 1A | Mar 22 | Mar 22 | ✅ Complete |
| Phase 1B | Mar 22 | Mar 22 | ✅ Complete |
| Phase 1C | Mar 22 | Mar 22 | ✅ Complete |
| Wave 6 | Mar 22 | Mar 22 | ⏳ Pending |
| Phase 2 | Mar 22-23 | Mar 23 | 📋 Planned |
| Phase 3 | Mar 24-26 | Mar 26 | 📋 Planned |

---

## Getting Started

1. **Read handoff** - `HANDOFF/README.md`
2. **Review architecture** - `HANDOFF/ARCHITECTURE-OVERVIEW.md`
3. **Run Wave 6** - Validate system
4. **Pick next task** - From this roadmap

---

**Good luck! The foundation is solid - build on it confidently.**
