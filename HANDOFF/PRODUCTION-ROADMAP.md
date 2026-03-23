# Production Roadmap: Long-Running Campaign System

**Version:** 0.13.0
**Date:** March 23, 2026
**Focus:** Production-ready long-running scans with monitoring and reporting

---

## Executive Summary

We have a working foundation:
- ✅ Campaign orchestration with DAG scheduling
- ✅ glassware-core detector integration (working!)
- ✅ TUI skeleton (demo working!)
- ✅ Checkpoint system (SQLite)
- ✅ Event bus + state manager

**What's needed for production long-running scans:**
1. Reliable checkpoint/resume
2. Live command steering
3. Rich reporting
4. Streamlined UX/workflow
5. Monitoring and alerting

---

## Current Status Assessment

### ✅ What's Working

| Feature | Status | Notes |
|---------|--------|-------|
| Campaign orchestration | ✅ Complete | DAG scheduling, parallel waves |
| Detector integration | ✅ Complete | glassware-core wired up |
| TUI skeleton | ✅ Complete | Demo mode working |
| Event bus | ✅ Complete | Pub/sub working |
| State manager | ✅ Complete | Queryable state |
| Command channel | ✅ Complete | Infrastructure ready |

### ⚠️ What Needs Work

| Feature | Status | Gap |
|---------|--------|-----|
| Checkpoint persistence | ⚠️ Partial | SQLite schema exists, not integrated |
| Campaign resume | ⚠️ Stub | Handler exists, needs checkpoint integration |
| Live commands | ⚠️ Stub | Channel exists, not wired to executor |
| Reports | ⚠️ Stub | SARIF works, markdown needs templates |
| TUI live data | ⚠️ Demo only | Needs event bus subscription |
| Multi-campaign | ❌ Missing | No queue/management |

---

## Production Requirements

### For Long-Running Scans (10k+ packages, overnight)

1. **Reliability**
   - [ ] Auto-checkpoint every N packages
   - [ ] Resume from exact interruption point
   - [ ] Handle network failures gracefully
   - [ ] Rate limit recovery

2. **Visibility**
   - [ ] Real-time progress (TUI)
   - [ ] ETA calculation and updates
   - [ ] Active package display
   - [ ] Findings preview

3. **Control**
   - [ ] Pause without losing progress
   - [ ] Adjust concurrency mid-scan
   - [ ] Skip problematic waves
   - [ ] Cancel with full checkpoint

4. **Reporting**
   - [ ] Interim reports (don't wait for completion)
   - [ ] Final markdown report
   - [ ] SARIF for GitHub integration
   - [ ] JSON for programmatic access

5. **UX/Workflow**
   - [ ] Simple campaign start
   - [ ] Background execution
   - [ ] Detached monitoring
   - [ ] Notification on completion

---

## Phase 2.5: Production Foundation (2-3 days)

### 2.5.1: Checkpoint Integration

**Goal:** Reliable checkpoint save/load

**Tasks:**
- [ ] Integrate `CheckpointManager` with campaign executor
- [ ] Save checkpoint after each wave completion
- [ ] Save checkpoint every N packages (configurable)
- [ ] Load checkpoint on resume
- [ ] Skip completed waves/packages

**Files:**
- `src/campaign/executor.rs` - Add checkpoint calls
- `src/campaign/checkpoint.rs` - Already implemented
- `src/main.rs:cmd_campaign_resume` - Wire up

**Acceptance Criteria:**
```bash
# Start campaign
./glassware-orchestrator campaign run campaigns/wave6.toml

# Interrupt with Ctrl+C after wave 1

# Resume - should skip wave 1
./glassware-orchestrator campaign resume <case-id>

# Should see: "Resuming from checkpoint..."
# Should see: "Skipping wave_6a (completed)"
```

---

### 2.5.2: Live Command Integration

**Goal:** Commands actually steer running campaigns

**Tasks:**
- [ ] Store running campaign handle in global state
- [ ] Wire command channel to executor loop
- [ ] Implement pause (stop after current package)
- [ ] Implement cancel (checkpoint and exit)
- [ ] Implement skip-wave

**Files:**
- `src/campaign/executor.rs` - Check command channel in loop
- `src/main.rs` - Global campaign registry
- `src/campaign/command_channel.rs` - Already implemented

**Acceptance Criteria:**
```bash
# In one terminal: run campaign
./glassware-orchestrator campaign run campaigns/wave6.toml &

# In another terminal: send command
./glassware-orchestrator campaign command <case-id> pause

# Should see campaign pause after current package
```

---

### 2.5.3: Markdown Report Generation

**Goal:** Professional reports for stakeholders

**Tasks:**
- [ ] Add `tera` dependency
- [ ] Create report template
- [ ] Implement report generator
- [ ] Add interim report generation

**Template Structure:**
```markdown
# GlassWorm Campaign Report

## Executive Summary
- Campaign name, dates, duration
- Total packages, flagged, malicious
- Detection rate

## Wave Results
### Wave 6A: Known Malicious Baseline
- Status: PASS/FAIL
- Packages scanned, flagged
- Details table

## LLM Analysis Summary
- Tier 1 analyses run
- Tier 2 analyses run
- Model performance

## Findings by Category
- Category breakdown chart
- Top findings

## Evidence Manifest
- Collected evidence list
- Chain of custody

## Appendix
- Configuration used
- Full scan log
```

**Files:**
- `src/campaign/report.rs` (new)
- `templates/report.md.tera` (new)
- `src/main.rs:cmd_campaign_report`

---

## Phase 3: TUI Enhancement (2-3 days)

### 3.1: Live TUI Data

**Goal:** TUI shows real campaign data

**Tasks:**
- [ ] Subscribe TUI to event bus
- [ ] Update state on events
- [ ] Re-render on state change
- [ ] Show real ETA calculation

**Files:**
- `src/tui/app.rs` - Event subscription
- `src/tui/ui.rs` - Dynamic rendering

**Acceptance Criteria:**
```bash
# Launch TUI demo (sample data)
./glassware-orchestrator campaign demo

# Should show:
# - Live progress bar
# - Wave status updates
# - Recent events scrolling
# - ETA countdown
```

---

### 3.2: TUI Command Palette

**Goal:** Control campaigns from TUI

**Tasks:**
- [ ] Keyboard shortcuts (`p`=pause, `x`=cancel, `s`=skip)
- [ ] Command palette (`Ctrl+P`)
- [ ] Confirmation dialogs
- [ ] Command feedback display

**Keyboard Layout:**
```
Global:
  q      - Quit TUI (campaign continues)
  Ctrl+C - Pause campaign
  Ctrl+X - Cancel campaign
  
Tabs:
  1      - Campaign overview
  2      - Findings
  3      - Logs
  
Commands:
  p      - Pause/Resume
  s      - Skip wave (prompts for wave ID)
  c      - Adjust concurrency (prompts for value)
  r      - Adjust rate limit
  ?      - Help
```

---

### 3.3: Detached Monitoring

**Goal:** Monitor campaigns in background

**Workflow:**
```bash
# Start campaign in background
./glassware-orchestrator campaign run campaigns/wave6.toml &
# Returns case ID: wave-6-calibration-20260323-061440

# Attach TUI monitor
./glassware-orchestrator campaign monitor wave-6-calibration-20260323-061440

# Or check status
./glassware-orchestrator campaign status wave-6-calibration-20260323-061440
```

**Implementation:**
- [ ] Store running campaigns in registry
- [ ] `campaign monitor` attaches to existing campaign
- [ ] `campaign status` shows current state

---

## Phase 4: UX/Workflow Streamlining (1-2 days)

### 4.1: Campaign Templates

**Goal:** Easy campaign creation

**Templates:**
```bash
# Quick start with template
./glassware-orchestrator campaign new --template react-native
# Creates: campaigns/react-native-20260323.toml

# Available templates:
# - react-native
# - mcp-ai
# - crypto-defi
# - install-scripts
# - broad-sweep
```

**Template Structure:**
```toml
# campaigns/templates/react-native.toml
[template]
name = "React Native Ecosystem"
description = "Scan React Native packages for GlassWorm patterns"

[settings]
concurrency = 15
rate_limit_npm = 15.0

[[waves]]
name = "React Native Phone"
sources = [{ type = "npm_search", keywords = ["react-native-phone"] }]
```

---

### 4.2: Campaign Queue

**Goal:** Run multiple campaigns sequentially

**Commands:**
```bash
# Add campaign to queue
./glassware-orchestrator queue add campaigns/wave6.toml

# View queue
./glassware-orchestrator queue list

# Start queue processing
./glassware-orchestrator queue run

# Pause queue
./glassware-orchestrator queue pause
```

**Features:**
- Priority levels
- Parallel campaign limits
- Automatic retry on failure

---

### 4.3: Notifications

**Goal:** Know when campaigns complete

**Options:**
```bash
# Email notification
./glassware-orchestrator config set notify-email security@example.com

# Slack webhook
./glassware-orchestrator config set slack-webhook https://hooks.slack.com/...

# System notification (default)
./glassware-orchestrator campaign run campaigns/wave6.toml
# Desktop notification on completion
```

---

## Phase 5: Monitoring & Alerting (2-3 days)

### 5.1: Health Metrics

**Metrics to Track:**
- Packages scanned per minute
- Findings per wave
- False positive rate
- API rate limit usage
- Memory usage
- Cache hit rate

**Export Formats:**
- Prometheus metrics endpoint
- JSON stats file
- TUI dashboard widget

---

### 5.2: Anomaly Detection

**Alerts:**
- Scan rate drops below threshold
- Findings spike (possible campaign detection)
- Rate limit approaching
- Memory usage high

**Actions:**
- Auto-pause on critical alerts
- Send notification
- Log anomaly for review

---

### 5.3: Historical Trends

**Track Over Time:**
- Campaign duration trends
- Detection rate trends
- False positive trends
- Package ecosystem changes

**Visualization:**
- TUI trends tab
- Export to CSV for external analysis

---

## New Ideas: Enhanced Workflows

### Idea 1: Campaign Profiles

**Concept:** Pre-configured campaign setups for different use cases

**Profiles:**
```bash
# Quick triage (100 packages, 10 min)
./glassware-orchestrator campaign run --profile quick

# Deep scan (1000 packages, 2 hours)
./glassware-orchestrator campaign run --profile deep

# Overnight marathon (10k packages)
./glassware-orchestrator campaign run --profile marathon

# Custom
./glassware-orchestrator campaign run campaigns/custom.toml
```

**Profile Settings:**
| Profile | Concurrency | Rate Limit | LLM | Checkpoint Interval |
|---------|-------------|------------|-----|---------------------|
| quick | 20 | 20/s | No | Every wave |
| deep | 10 | 10/s | Tier 1 | Every 50 packages |
| marathon | 5 | 5/s | Tier 1+2 | Every 10 packages |

---

### Idea 2: Findings Review Workflow

**Concept:** Interactive findings review before finalizing reports

**Workflow:**
```bash
# After campaign completes
./glassware-orchestrator review <case-id>

# Opens TUI findings reviewer:
┌Findings Review─────────────────────────────────────┐
│ [1/47] express@4.19.2 - Time Delay Pattern         │
│ Severity: MEDIUM                                    │
│ File: lib/utils.js:142                              │
│                                                     │
│ [Accept] [Reject] [Mark False Positive] [Skip]     │
│                                                     │
│ LLM Analysis: "Likely false positive - common..."  │
└─────────────────────────────────────────────────────┘

# Finalize with reviewed findings
./glassware-orchestrator report finalize <case-id>
```

---

### Idea 3: Comparative Analysis

**Concept:** Compare multiple campaigns to find patterns

**Commands:**
```bash
# Compare campaigns
./glassware-orchestrator compare <case-id-1> <case-id-2>

# Find common malicious packages
./glassware-orchestrator compare <case-id-1> <case-id-2> --common

# Generate trend report
./glassware-orchestrator trend-report wave6 wave7 wave8
```

**Output:**
- Common malicious packages across campaigns
- Detection rate trends
- Ecosystem changes over time

---

### Idea 4: Package Watchlist

**Concept:** Monitor specific packages for changes

**Workflow:**
```bash
# Add package to watchlist
./glassware-orchestrator watchlist add react-native-country-select

# Run watchlist scan
./glassware-orchestrator watchlist scan

# Get notified of changes
./glassware-orchestrator watchlist scan --notify
```

**Features:**
- Version change detection
- New findings alert
- Author changes

---

## Prioritized Implementation Plan

### Week 1: Production Foundation
- [ ] 2.5.1 Checkpoint Integration (Day 1-2)
- [ ] 2.5.2 Live Command Integration (Day 2-3)
- [ ] 2.5.3 Markdown Reports (Day 4-5)

### Week 2: TUI Enhancement
- [ ] 3.1 Live TUI Data (Day 1-2)
- [ ] 3.2 TUI Command Palette (Day 3)
- [ ] 3.3 Detached Monitoring (Day 4-5)

### Week 3: UX/Workflow
- [ ] 4.1 Campaign Templates (Day 1)
- [ ] 4.2 Campaign Queue (Day 2-3)
- [ ] 4.3 Notifications (Day 4)
- [ ] 5.1 Health Metrics (Day 5)

### Week 4: Polish & Testing
- [ ] Integration testing
- [ ] Documentation
- [ ] Bug fixes
- [ ] Performance optimization

---

## Success Criteria

### For 10k Package Overnight Scan

**Before Starting:**
- [ ] Campaign configured with `marathon` profile
- [ ] Checkpoint interval set to 10 packages
- [ ] Email notification configured
- [ ] Queue position checked

**During Scan:**
- [ ] Monitor progress via TUI (`campaign monitor`)
- [ ] Adjust concurrency if needed (`campaign command`)
- [ ] Check interim report (`campaign report --interim`)

**After Completion:**
- [ ] Email notification received
- [ ] Final report generated
- [ ] Evidence collected for flagged packages
- [ ] SARIF uploaded to GitHub (optional)

**If Interrupted:**
- [ ] Campaign checkpointed automatically
- [ ] Resume with `campaign resume <case-id>`
- [ ] No progress lost

---

## Open Questions for Discussion

1. **Checkpoint Frequency:** Every N packages vs every wave?
   - Trade-off: Performance vs data loss risk
   - Recommendation: Configurable, default every 10 packages

2. **TUI vs Web Dashboard:**
   - TUI: Faster to implement, lower resource usage
   - Web: Better for remote monitoring, more flexible
   - Recommendation: TUI first, web dashboard later

3. **Multi-Campaign Parallelism:**
   - How many campaigns should run concurrently?
   - Resource limits (API rate limits, CPU, memory)?
   - Recommendation: Configurable, default 1 campaign at a time

4. **Evidence Storage:**
   - Keep evidence for all flagged packages or only malicious?
   - Compression for long-term storage?
   - Recommendation: Flagged packages, compressed after 30 days

---

## Next Steps

1. **Review this roadmap** - Prioritize features
2. **Choose Week 1 tasks** - Start with checkpoint integration
3. **Set up project board** - Track progress
4. **Begin implementation** - Phase 2.5.1

---

**This roadmap provides a clear path to production-ready long-running scans with excellent UX.**
