# TUI Architecture RFC

**RFC:** 001
**Title:** TUI-Ready Campaign Architecture
**Date:** March 22, 2026
**Status:** Approved for Implementation

---

## 1. Overview

This RFC proposes architectural patterns to enable a future Terminal User Interface (TUI) for long-running campaign monitoring and control. The design ensures we can add interactive monitoring, real-time steering, and advanced campaign management without refactoring the core execution engine.

### Motivation

Long-running campaigns (overnight scans, 10k+ packages) require:
- **Real-time visibility** into progress, not just periodic checkpoints
- **Interactive control** (pause, resume, skip wave, adjust concurrency)
- **Live diagnostics** (current package, active findings, rate limit status)
- **Historical context** (trends, ETA adjustments, anomaly detection)

A TUI provides this, but requires specific architectural support.

---

## 2. Design Principles

| Principle | Rationale |
|-----------|-----------|
| **Separation of concerns** | Execution engine independent of UI layer |
| **Event-driven architecture** | UI subscribes to events, doesn't poll |
| **Queryable state** | TUI can snapshot state at any time |
| **Non-blocking control** | Commands don't block scan execution |
| **Multiple UI support** | CLI progress bars, TUI, web dashboard, headless |

---

## 3. Architecture

### 3.1 High-Level Design

```
┌─────────────────────────────────────────────────────────────────┐
│                      UI Layer (Pluggable)                        │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ CLI Progress│  │   TUI       │  │   Headless (CI/CD)    │  │
│  │ (indicatif) │  │ (ratatui)   │  │   (JSON output only)  │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
           │                      │                      │
           └──────────────────────┼──────────────────────┘
                                  │
                    ┌─────────────▼─────────────┐
                    │    Event Bus (Pub/Sub)    │
                    │  tokio::sync::broadcast   │
                    └─────────────┬─────────────┘
                                  │
┌─────────────────────────────────┼─────────────────────────────────┐
│                      Core Engine                                    │
├─────────────────────────────────┼─────────────────────────────────┤
│  ┌──────────────┐               │               ┌──────────────┐  │
│  │   Campaign   │◄──────────────┼──────────────►│   Command    │  │
│  │   Executor   │               │               │   Channel    │  │
│  └──────────────┘               │               └──────────────┘  │
│         │                       │                        │        │
│         │              ┌────────▼────────┐               │        │
│         │              │  State Manager  │◄──────────────┘        │
│         │              │  (In-Memory +   │                        │
│         │              │   SQLite)       │                        │
│         │              └─────────────────┘                        │
│         │                                                          │
│  ┌──────▼──────────────────────────────────────────────────────┐  │
│  │                    Wave Executors                            │  │
│  │  (Publish events, check command channel, update state)       │  │
│  └──────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Event Bus Design

```rust
use tokio::sync::broadcast;

/// Campaign events published to all subscribers
#[derive(Debug, Clone)]
pub enum CampaignEvent {
    // Campaign lifecycle
    CampaignStarted { case_id: String, config: CampaignConfig },
    CampaignPaused { case_id: String, reason: String },
    CampaignResumed { case_id: String },
    CampaignCompleted { case_id: String, result: CampaignResult },
    CampaignFailed { case_id: String, error: String },
    
    // Wave lifecycle
    WaveStarted { wave_id: String, name: String, packages_total: usize },
    WaveProgress { wave_id: String, scanned: usize, flagged: usize, malicious: usize },
    WaveCompleted { wave_id: String, result: WaveResult },
    WaveFailed { wave_id: String, error: String },
    
    // Package-level events
    PackageScanned { package: String, version: String, threat_score: f32 },
    PackageFlagged { package: String, findings_count: usize },
    PackageMalicious { package: String, threat_score: f32, llm_verdict: Option<String> },
    
    // LLM events
    LlmAnalysisStarted { package: String, tier: u8 },
    LlmAnalysisCompleted { package: String, verdict: LlmVerdict, model: String },
    
    // System events
    RateLimitWait { target: String, wait_ms: u64 },
    CheckpointSaved { path: String },
    EvidenceCollected { package: String, path: String },
    
    // Steering commands (echo back for confirmation)
    CommandReceived { command: CampaignCommand },
    CommandExecuted { command: CampaignCommand },
}

/// Event bus for campaign events
pub struct EventBus {
    sender: broadcast::Sender<CampaignEvent>,
}

impl EventBus {
    pub fn new(buffer_size: usize) -> Self {
        let (sender, _) = broadcast::channel(buffer_size);
        Self { sender }
    }
    
    pub fn publish(&self, event: CampaignEvent) {
        // Non-blocking, subscribers may lag
        let _ = self.sender.send(event);
    }
    
    pub fn subscribe(&self) -> broadcast::Receiver<CampaignEvent> {
        self.sender.subscribe()
    }
}
```

### 3.3 State Manager Design

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory campaign state (queryable by TUI)
#[derive(Debug, Clone)]
pub struct CampaignState {
    pub case_id: String,
    pub status: CampaignStatus,
    pub started_at: DateTime<Utc>,
    pub current_wave: Option<String>,
    pub waves: HashMap<String, WaveState>,
    pub stats: CampaignStats,
    pub active_package: Option<ActivePackage>,
    pub recent_events: VecDeque<CampaignEvent>,  // Last 100 events
}

#[derive(Debug, Clone)]
pub struct CampaignStats {
    pub packages_total: usize,
    pub packages_scanned: usize,
    pub packages_flagged: usize,
    pub packages_malicious: usize,
    pub findings_total: usize,
    pub llm_analyses_run: usize,
    pub evidence_collected: usize,
    pub eta: Option<Duration>,
    pub scan_rate: f32,  // packages per minute
}

#[derive(Debug, Clone)]
pub struct ActivePackage {
    pub name: String,
    pub version: String,
    pub wave_id: String,
    pub stage: PackageStage,  // downloading | scanning | llm_analysis | complete
    pub started_at: DateTime<Utc>,
}

/// Thread-safe state manager
pub struct StateManager {
    state: Arc<RwLock<CampaignState>>,
    event_bus: EventBus,
}

impl StateManager {
    pub fn new(case_id: String, event_bus: EventBus) -> Self {
        let state = CampaignState {
            case_id,
            status: CampaignStatus::Initializing,
            started_at: Utc::now(),
            current_wave: None,
            waves: HashMap::new(),
            stats: CampaignStats::default(),
            active_package: None,
            recent_events: VecDeque::with_capacity(100),
        };
        
        Self {
            state: Arc::new(RwLock::new(state)),
            event_bus,
        }
    }
    
    /// Get current state snapshot (for TUI rendering)
    pub async fn snapshot(&self) -> CampaignState {
        self.state.read().await.clone()
    }
    
    /// Update state and publish event
    pub async fn update<F>(&self, update_fn: F, event: CampaignEvent)
    where
        F: FnOnce(&mut CampaignState)
    {
        let mut state = self.state.write().await;
        update_fn(&mut state);
        state.recent_events.push_back(event.clone());
        if state.recent_events.len() > 100 {
            state.recent_events.pop_front();
        }
        self.event_bus.publish(event);
    }
}
```

### 3.4 Command Channel Design

```rust
use tokio::sync::mpsc;

/// Commands that can steer campaign execution
#[derive(Debug, Clone)]
pub enum CampaignCommand {
    // Execution control
    Pause { reason: String },
    Resume,
    Cancel { save_checkpoint: bool },
    
    // Wave control
    SkipWave { wave_id: String },
    RetryWave { wave_id: String },
    
    // Runtime adjustments
    SetConcurrency { concurrency: usize },
    SetRateLimit { rate_limit: f32 },
    ToggleLlm { enabled: bool },
    
    // Diagnostics
    DumpState,  // Write current state to file
    CollectMetrics,  // Force metrics collection
}

/// Command response
#[derive(Debug, Clone)]
pub enum CommandResponse {
    Accepted { command: CampaignCommand },
    Rejected { command: CampaignCommand, reason: String },
    Completed { command: CampaignCommand, result: String },
}

/// Command channel for steering campaigns
pub struct CommandChannel {
    sender: mpsc::Sender<(CampaignCommand, mpsc::Sender<CommandResponse>)>,
    receiver: Arc<Mutex<mpsc::Receiver<(CampaignCommand, mpsc::Sender<CommandResponse>)>>>,
}

impl CommandChannel {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            sender: tx,
            receiver: Arc::new(Mutex::new(rx)),
        }
    }
    
    pub fn sender(&self) -> mpsc::Sender<(CampaignCommand, mpsc::Sender<CommandResponse>)> {
        self.sender.clone()
    }
    
    /// Receive commands (called by executor)
    pub async fn recv(&self) -> Option<(CampaignCommand, mpsc::Sender<CommandResponse>)> {
        self.receiver.lock().await.recv().await
    }
    
    /// Send command (called by TUI)
    pub async fn send(&self, command: CampaignCommand) -> CommandResponse {
        let (response_tx, mut response_rx) = mpsc::channel(1);
        self.sender.send((command.clone(), response_tx)).await.unwrap();
        response_rx.recv().await.unwrap_or(CommandResponse::Rejected {
            command,
            reason: "Channel closed".to_string(),
        })
    }
}
```

---

## 4. TUI Architecture (Future Implementation)

### 4.1 Component Structure (ratatui)

```
TUI Application
├── App (main state machine)
│   ├── CampaignTab
│   │   ├── Overview panel (progress, ETA, stats)
│   │   ├── Waves panel (wave status, dependencies)
│   │   └── Packages panel (recent scans, flagged)
│   ├── FindingsTab
│   │   ├── Findings list (filterable, sortable)
│   │   └── Finding detail view
│   ├── EvidenceTab
│   │   ├── Evidence manifest
│   │   └── Chain of custody viewer
│   └── LogsTab
│       ├── Event log (filtered by severity)
│       └── Command history
│
├── Event Loop
│   ├── Subscribe to CampaignEvent bus
│   ├── Poll CommandChannel for responses
│   └── Handle keyboard input
│
└── Renderer (ratatui terminal)
    ├── Layout management
    ├── Widget rendering
    └── Screen refresh
```

### 4.2 TUI Event Loop

```rust
pub struct TuiApp {
    state: CampaignState,
    event_rx: broadcast::Receiver<CampaignEvent>,
    command_tx: mpsc::Sender<(CampaignCommand, mpsc::Sender<CommandResponse>)>,
    response_rx: mpsc::Receiver<CommandResponse>,
    active_tab: Tab,
    running: bool,
}

impl TuiApp {
    pub async fn run(&mut self, terminal: &mut Terminal<CrosstermBackend>) -> Result<()> {
        while self.running {
            // Render current state
            terminal.draw(|f| self.render(f))?;
            
            // Handle events with timeout
            if crossterm::event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = crossterm::event::read()? {
                    self.handle_key_input(key).await?;
                }
            }
            
            // Process campaign events (non-blocking)
            while let Ok(event) = self.event_rx.try_recv() {
                self.handle_campaign_event(event);
            }
            
            // Process command responses
            while let Ok(response) = self.response_rx.try_recv() {
                self.handle_command_response(response);
            }
            
            // Auto-refresh state periodically
            self.refresh_state().await;
        }
        
        Ok(())
    }
}
```

### 4.3 TUI Commands (Keyboard Shortcuts)

| Key | Action |
|-----|--------|
| `q` | Quit TUI (campaign continues in background) |
| `Ctrl+C` | Pause campaign |
| `Ctrl+R` | Resume campaign |
| `Ctrl+X` | Cancel campaign (with checkpoint) |
| `1`, `2`, `3` | Switch tabs (Campaign, Findings, Evidence) |
| `p` | Pause/Resume toggle |
| `s` | Skip current wave |
| `c` | Adjust concurrency |
| `r` | Adjust rate limit |
| `d` | Dump state to file |
| `?` | Show help |

---

## 5. Implementation Guidelines

### 5.1 Executor Integration

```rust
pub struct CampaignExecutor {
    state: StateManager,
    command_channel: CommandChannel,
    event_bus: EventBus,
    // ... other fields
}

impl CampaignExecutor {
    pub async fn run(&self, campaign: &CampaignConfig) -> Result<CampaignResult> {
        // Publish start event
        self.event_bus.publish(CampaignEvent::CampaignStarted {
            case_id: campaign.case_id(),
            config: campaign.clone(),
        });
        
        // Update state
        self.state.update(|s| {
            s.status = CampaignStatus::Running;
        }, CampaignEvent::CampaignStarted { ... }).await;
        
        // Execute waves
        for stage in self.build_execution_plan(campaign) {
            // Check for commands before each stage
            if let Some((cmd, response_tx)) = self.command_channel.recv().now_or_never() {
                self.handle_command(cmd, response_tx).await?;
            }
            
            // Execute waves in stage
            for wave_id in &stage.waves {
                // Update active wave in state
                self.state.update(|s| {
                    s.current_wave = Some(wave_id.clone());
                }, CampaignEvent::WaveStarted { ... }).await;
                
                // Execute wave with periodic command checks
                let result = self.execute_wave_with_commands(wave_id).await?;
                
                // Publish completion event
                self.event_bus.publish(CampaignEvent::WaveCompleted { ... });
            }
        }
        
        Ok(CampaignResult { ... })
    }
    
    /// Execute wave while checking for commands
    async fn execute_wave_with_commands(&self, wave_id: &str) -> Result<WaveResult> {
        let mut wave_executor = self.create_wave_executor(wave_id);
        
        loop {
            tokio::select! {
                // Check for commands
                Some((cmd, response_tx)) = self.command_channel.recv() => {
                    self.handle_command(cmd, response_tx).await?;
                }
                
                // Execute wave packages
                result = &mut wave_executor.execute() => {
                    return result;
                }
            }
        }
    }
}
```

### 5.2 State Update Patterns

```rust
// In wave executor, after scanning each package:
self.state.update(|state| {
    state.stats.packages_scanned += 1;
    if result.is_flagged() {
        state.stats.packages_flagged += 1;
    }
    if result.is_malicious() {
        state.stats.packages_malicious += 1;
    }
    state.active_package = None;  // Clear active package
}, CampaignEvent::PackageScanned {
    package: pkg.name.clone(),
    version: pkg.version.clone(),
    threat_score: result.threat_score,
}).await;

// Before scanning each package:
self.state.update(|state| {
    state.active_package = Some(ActivePackage {
        name: pkg.name.clone(),
        version: pkg.version.clone(),
        wave_id: wave_id.clone(),
        stage: PackageStage::Scanning,
        started_at: Utc::now(),
    });
}, CampaignEvent::PackageScanningStarted { ... }).await;
```

---

## 6. CLI Commands for TUI Integration

```bash
# Start campaign with TUI monitoring
glassware-orchestrator campaign run wave6.toml --tui

# Start campaign in background, attach TUI later
glassware-orchestrator campaign run wave6.toml --detach
glassware-orchestrator campaign attach <case_id>

# Send command to running campaign
glassware-orchestrator campaign command <case_id> pause
glassware-orchestrator campaign command <case_id> set-concurrency 20
glassware-orchestrator campaign command <case_id> skip-wave wave_6b

# View live status (without full TUI)
glassware-orchestrator campaign status <case_id> --live

# Export state snapshot
glassware-orchestrator campaign status <case_id> --export json
```

---

## 7. Milestones

### Phase 1: Event Bus + State Manager (Week 1)
- [ ] Implement `EventBus` with `tokio::sync::broadcast`
- [ ] Implement `StateManager` with `Arc<RwLock<CampaignState>>`
- [ ] Implement `CommandChannel` with `tokio::sync::mpsc`
- [ ] Integrate event publishing into wave executor
- [ ] Integrate command checking into executor loop

### Phase 2: CLI Monitoring (Week 2)
- [ ] `campaign status --live` command (polling state)
- [ ] `campaign command` subcommand
- [ ] State export (JSON)
- [ ] Event log export

### Phase 3: TUI Implementation (Week 4-5)
- [ ] Basic TUI skeleton (ratatui)
- [ ] Campaign tab with progress panels
- [ ] Event subscription and rendering
- [ ] Command sending from TUI
- [ ] Keyboard shortcuts

---

## 8. Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio` | 1.x | Async runtime, sync primitives |
| `ratatui` | 0.24+ | TUI framework (Phase 3) |
| `crossterm` | 0.27+ | Terminal manipulation (Phase 3) |

---

## 9. Success Criteria

| Criterion | Target |
|-----------|--------|
| **Event latency** | <100ms from event to TUI render |
| **Command latency** | <500ms from keypress to execution |
| **State consistency** | TUI state matches SQLite within 1 second |
| **Memory usage** | <50MB for event buffer + state |
| **No blocking** | UI events don't block scan execution |

---

**Approved:** March 22, 2026
**Implementation:** Phase 1 starts immediately after architecture approval
