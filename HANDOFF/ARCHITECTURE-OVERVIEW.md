# Architecture Overview

**Purpose:** High-level architecture of the Glassworks Campaign System

---

## System Context

```
┌─────────────────────────────────────────────────────────────────┐
│                    Glassworks Campaign System                    │
│                                                                  │
│  Input: Campaign TOML config                                     │
│  Process: Orchestrated security scanning                         │
│  Output: Scan results + evidence + reports                       │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    glassware-core                                │
│  Core detection engine (22+ detectors)                           │
│  - L1: Invisible chars, homoglyphs, bidi                         │
│  - L2: GlassWare patterns, encrypted payload                     │
│  - L3: Behavioral (locale, time delay, blockchain C2)            │
└─────────────────────────────────────────────────────────────────┘
```

---

## Campaign Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    CLI Layer                                     │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ campaign run/resume/status/command/list/report          │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Campaign Executor                             │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐    │
│  │ DAG          │     │ Wave         │     │ Command      │    │
│  │ Scheduler    │────►│ Executor     │◄────│ Handler      │    │
│  └──────────────┘     └──────────────┘     └──────────────┘    │
│         │                    │                                  │
│         │                    │                                  │
│  ┌──────▼────────────────────▼──────────────────────────────┐  │
│  │                    Event Bus (pub/sub)                    │  │
│  └──────┬────────────────────┬──────────────────────────────┘  │
│         │                    │                                  │
│  ┌──────▼────────┐   ┌───────▼────────┐                        │
│  │ State Manager │   │ Command Channel│                        │
│  │ (queryable)   │   │ (steering)     │                        │
│  └───────────────┘   └────────────────┘                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Package Scanner                               │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐    │
│  │ Download     │────►│ Scan         │────►│ LLM Analysis │    │
│  │ (npm/GitHub) │     │ (glassware-  │     │ (optional)   │    │
│  │              │     │  core)       │     │              │    │
│  └──────────────┘     └──────────────┘     └──────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

---

## Data Flow

### Campaign Execution

```
1. Load TOML config
         │
         ▼
2. Validate config (duplicates, circular deps)
         │
         ▼
3. Build DAG execution plan
         │
         ▼
4. For each stage:
   │
   ├─► Check for commands (pause/cancel)
   │
   ├─► Execute waves in parallel
   │   │
   │   ├─► Collect packages from sources
   │   │
   │   ├─► For each package:
   │   │   │
   │   │   ├─► Download
   │   │   │
   │   │   ├─► Scan with glassware-core
   │   │   │
   │   │   ├─► LLM analysis (if enabled)
   │   │   │
   │   │   └─► Publish events
   │   │
   │   └─► Update state
   │
   └─► Save checkpoint
         │
         ▼
5. Generate reports
         │
         ▼
6. Print summary
```

---

## Event Flow

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ Campaign    │     │ Wave        │     │ Package     │
│ Executor    │     │ Executor    │     │ Scanner     │
└──────┬──────┘     └──────┬──────┘     └──────┬──────┘
       │                   │                   │
       │ CampaignStarted   │                   │
       ├──────────────────────────────────────►│
       │                   │                   │
       │                   │ WaveStarted       │
       │                   ├──────────────────►│
       │                   │                   │
       │                   │                   │ PackageScanned
       │                   │                   ├──────────┐
       │                   │                   │          │
       │                   │                   │◄─────────┤
       │                   │                   │          │
       │                   │ WaveProgress      │          │
       │                   ├──────────────────────────────┤
       │                   │                   │          │
       │ CampaignCompleted │                   │          │
       ├───────────────────────────────────────────────────┤
       │                   │                   │          │
       ▼                   ▼                   ▼          ▼
┌─────────────────────────────────────────────────────────────┐
│                    Event Bus                                 │
│  (tokio::sync::broadcast)                                    │
└─────────────────────────────────────────────────────────────┘
       │                   │                   │
       ▼                   ▼                   ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ State       │     │ TUI         │     │ Logger      │
│ Manager     │     │ (future)    │     │ (tracing)   │
└─────────────┘     └─────────────┘     └─────────────┘
```

---

## State Model

```
CampaignState
├── case_id: String
├── campaign_name: String
├── status: CampaignStatus
├── started_at: DateTime
├── completed_at: Option<DateTime>
├── current_wave: Option<String>
├── waves: HashMap<String, WaveState>
│   ├── id: String
│   ├── name: String
│   ├── status: WaveStatus
│   ├── packages_total: usize
│   ├── packages_scanned: usize
│   └── ...
├── stats: CampaignStats
│   ├── packages_scanned: usize
│   ├── packages_flagged: usize
│   ├── packages_malicious: usize
│   └── ...
└── active_package: Option<ActivePackage>
```

---

## Command Pattern

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ TUI / CLI   │────►│ Command     │────►│ Campaign    │
│             │     │ Channel     │     │ Executor    │
└─────────────┘     └─────────────┘     └─────────────┘
                           │                   │
                           │                   │
                    ┌──────▼────────┐          │
                    │ Command       │          │
                    │ Handler       │◄─────────┘
                    └───────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │ Response    │
                    │ (accepted/  │
                    │ rejected/   │
                    │ completed)  │
                    └─────────────┘
```

**Commands:**
- `Pause` - Pause execution
- `Resume` - Resume paused
- `Cancel` - Cancel with checkpoint
- `SkipWave` - Skip current wave
- `SetConcurrency` - Adjust concurrency
- `SetRateLimit` - Adjust rate limit

---

## Concurrency Model

```
Campaign Executor (tokio async)
│
├─► Stage 1 (parallel)
│   ├─► Wave 6A (tokio::spawn)
│   │   └─► Package scans (buffered stream, concurrency=10)
│   │
│   └─► Wave 6C (tokio::spawn)
│       └─► Package scans (buffered stream, concurrency=10)
│
└─► Stage 2 (parallel)
    └─► Wave 6B (tokio::spawn)
        └─► Package scans (buffered stream, concurrency=10)
```

**Key Patterns:**
- `tokio::spawn` for parallel waves
- `futures::stream::buffered` for controlled concurrency
- `tokio::sync::Semaphore` for concurrency limiting
- `tokio::sync::broadcast` for event pub/sub
- `tokio::sync::RwLock` for state access

---

## Error Handling

```rust
pub enum CampaignError {
    WaveError(WaveError),      // Wave execution failed
    TaskError(String),         // Tokio task failed
    Cancelled,                 // Campaign cancelled
    ConfigError(String),       // Config validation failed
}

pub enum WaveError {
    CollectionError(String),   // Package collection failed
    ExecutorError(String),     // Executor failed
    IoError(std::io::Error),   // IO error
}
```

**Strategy:**
- Errors logged and reported
- Campaign continues with other waves
- Final summary includes error count
- Exit code 1 if any errors

---

## Configuration Hierarchy

```
1. Default values (code)
         │
         ▼
2. TOML config file
         │
         ▼
3. Environment variables
         │
         ▼
4. CLI overrides
         │
         ▼
5. Runtime commands
```

---

## File Locations

| Component | Location |
|-----------|----------|
| Campaign configs | `campaigns/*.toml` |
| Evidence | `evidence/<case-id>/` |
| Reports | `reports/<case-id>/` |
| Cache DB | `.glassware-orchestrator-cache.db` |
| Checkpoints | `.glassware-checkpoints/` |

---

## Dependencies

### Core

| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio` | 1.x | Async runtime |
| `serde` | 1.x | Serialization |
| `toml` | 0.8 | TOML parsing |
| `thiserror` | 1.x | Error types |
| `tracing` | 0.1 | Logging |

### CLI

| Crate | Version | Purpose |
|-------|---------|---------|
| `clap` | 4.x | CLI parsing |
| `indicatif` | 0.17 | Progress bars |

### Future (TUI)

| Crate | Version | Purpose |
|-------|---------|---------|
| `ratatui` | 0.24 | TUI framework |
| `crossterm` | 0.27 | Terminal control |

---

## Testing Strategy

### Unit Tests

- In-module tests (`#[cfg(test)]`)
- Focus on pure functions
- Mock external dependencies

### Integration Tests

- End-to-end campaign execution
- Wave dependency resolution
- Command handling

### Manual Testing

- Wave 6 calibration campaign
- Known malicious packages
- Known clean packages

---

## Performance Characteristics

| Metric | Target | Current |
|--------|--------|---------|
| Scan speed | 50k LOC/sec | ~50k LOC/sec |
| npm scan | ~0.5s/pkg | ~0.5s/pkg |
| Parallel waves | Yes | Yes |
| Concurrency | Configurable | Default 10 |
| Memory | <100MB | ~50MB |

---

## Security Considerations

- Scanned packages are untrusted
- Evidence stored with integrity hashes
- Chain of custody logging
- Rate limiting to avoid API abuse
- Credentials from environment variables

---

## Future Enhancements

### Phase 2

- [ ] Campaign resume from checkpoint
- [ ] Live command steering
- [ ] Markdown/SARIF reports

### Phase 3

- [ ] TUI implementation
- [ ] Live progress display
- [ ] Interactive command palette

### Future

- [ ] Multi-campaign orchestration
- [ ] Distributed scanning
- [ ] Web dashboard
- [ ] Automated disclosure workflow
