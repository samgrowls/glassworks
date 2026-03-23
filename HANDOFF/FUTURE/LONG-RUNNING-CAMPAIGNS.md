# Long-Running Campaigns Design Document

**Version:** 1.0.0
**Date:** March 23, 2026
**Status:** Design Specification
**Author:** Glassworks Security Team

---

## Executive Summary

This document specifies enhancements to support **days-long campaign runs** (100k+ packages) in the Glassworks GlassWare detection system. Current campaigns can run for hours (10k+ packages); this design extends reliability, monitoring, and management capabilities for extended operations.

### Current State

| Metric | Current Capability | Target |
|--------|-------------------|--------|
| Package scale | 10k+ packages | 100k+ packages |
| Duration | Hours | Days |
| Checkpoint frequency | Every 10 packages | Adaptive (time + count) |
| Monitoring | TUI (local) | Remote + web dashboard |
| Recovery | Manual resume | Automatic retry + backoff |
| Notifications | None | Email, Slack, webhook |

### Key Challenges for Days-Long Runs

1. **Reliability**: Network failures, rate limits, resource exhaustion over extended periods
2. **Visibility**: Operators need to monitor progress without being tied to terminal
3. **Resource Management**: Memory, CPU, disk usage over days of continuous operation
4. **Interruption Recovery**: Power failures, network outages, system updates
5. **Alert Fatigue**: Distinguishing normal slowdowns from actual problems

---

## 1. Reliability Features

### 1.1 Checkpoint Frequency Optimization

**Current:** Fixed interval (every N packages)

**Proposed:** Adaptive checkpointing based on multiple factors

```rust
pub struct AdaptiveCheckpointConfig {
    /// Base interval (packages)
    pub base_interval: usize,
    /// Minimum time between checkpoints
    pub min_interval_secs: u64,
    /// Maximum time between checkpoints
    pub max_interval_secs: u64,
    /// Checkpoint on wave boundaries (always)
    pub wave_boundary: bool,
    /// Increase frequency when error rate is high
    pub error_rate_threshold: f32,
}

impl Default for AdaptiveCheckpointConfig {
    fn default() -> Self {
        Self {
            base_interval: 50,              // Every 50 packages
            min_interval_secs: 30,          // But at least 30s apart
            max_interval_secs: 300,         // But at most every 5 minutes
            wave_boundary: true,            // Always checkpoint wave completion
            error_rate_threshold: 0.05,     // 5% error rate triggers more frequent checkpoints
        }
    }
}
```

**Adaptive Logic:**

```
IF error_rate > threshold THEN
    interval = base_interval / 2          // More frequent on errors
ELSE IF scan_rate < expected_rate * 0.5 THEN
    interval = base_interval / 2          // More frequent on slowdown
ELSE IF time_since_last_checkpoint > max_interval THEN
    checkpoint()                          // Force time-based checkpoint
ELSE IF packages_since_checkpoint >= base_interval THEN
    checkpoint()                          // Normal count-based checkpoint
```

**Implementation Complexity:** Medium
**Estimated Effort:** 2-3 days

---

### 1.2 Database Compaction for Checkpoint DB

**Problem:** SQLite checkpoint database grows unbounded during long runs

**Current:** rusqlite with simple INSERT/UPDATE operations

**Proposed:** Periodic compaction and vacuum

```rust
pub struct CheckpointCompactor {
    db_path: PathBuf,
    /// Compact when DB size exceeds this threshold
    size_threshold_mb: u64,
    /// Compact after N waves completed
    wave_interval: usize,
    /// Last compaction time
    last_compaction: Option<DateTime<Utc>>,
}

impl CheckpointCompactor {
    /// Compact the checkpoint database
    pub async fn compact(&self) -> Result<CompactionStats> {
        // 1. Archive old completed wave data
        // 2. DELETE processed package_scans older than retention period
        // 3. VACUUM SQLite database
        // 4. Report size reclaimed
    }

    /// Check if compaction is needed
    pub fn needs_compaction(&self) -> bool {
        let db_size = fs::metadata(&self.db_path)?.len();
        db_size > self.size_threshold_mb * 1024 * 1024
    }
}
```

**Retention Policy:**
```toml
[checkpoint.retention]
# Keep detailed package data for recent waves
recent_waves_count = 3
# Keep only summary for older waves
archive_old_waves = true
# Delete package scan details older than
package_detail_ttl_hours = 24
```

**Implementation Complexity:** Medium
**Estimated Effort:** 1-2 days

---

### 1.3 Automatic Retry with Exponential Backoff

**Current:** Basic retry module exists (`retry.rs`) with exponential backoff

**Enhancement:** Context-aware retry for long-running scenarios

```rust
pub struct LongRunRetryConfig {
    /// Initial backoff
    pub base_delay: Duration,
    /// Maximum backoff
    pub max_delay: Duration,
    /// Multiplier
    pub multiplier: f32,
    /// Enable jitter
    pub jitter: bool,
    /// Circuit breaker: max failures before giving up
    pub circuit_breaker_threshold: u32,
    /// Circuit breaker reset timeout
    pub circuit_breaker_reset_secs: u64,
    /// Retry budget: max retries per hour (prevent runaway)
    pub retry_budget_per_hour: u32,
}

/// Circuit breaker state
pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    last_failure_time: Option<DateTime<Utc>>,
}

enum CircuitState {
    Closed,     // Normal operation
    Open,       // Failing, reject requests
    HalfOpen,   // Testing if recovered
}
```

**Retry Categories:**

| Error Type | Retry Strategy | Max Retries |
|------------|---------------|-------------|
| HTTP 429 (Rate Limit) | Exponential backoff + rate limit adaptation | 10 |
| HTTP 5xx (Server Error) | Exponential backoff | 5 |
| Network Timeout | Exponential backoff | 5 |
| DNS Failure | Exponential backoff | 3 |
| Disk Full | No retry, alert operator | 0 |
| LLM API Error | Exponential backoff, fallback tier | 5 |

**Implementation Complexity:** Medium
**Estimated Effort:** 2 days

---

### 1.4 Network Failure Recovery

**Current:** Per-operation retry

**Enhancement:** Connection pool health monitoring and recovery

```rust
pub struct ConnectionHealthMonitor {
    /// Track consecutive failures
    consecutive_failures: u32,
    /// Track success rate over time window
    success_rate_window: VecDeque<bool>,
    /// Last successful connection time
    last_success: Option<DateTime<Utc>>,
}

impl ConnectionHealthMonitor {
    /// Record operation result
    pub fn record(&mut self, success: bool) {
        self.success_rate_window.push_back(success);
        if self.success_rate_window.len() > 100 {
            self.success_rate_window.pop_front();
        }

        if success {
            self.consecutive_failures = 0;
            self.last_success = Some(Utc::now());
        } else {
            self.consecutive_failures += 1;
        }
    }

    /// Check if connection appears healthy
    pub fn is_healthy(&self) -> bool {
        let success_rate = self.success_rate_window.iter()
            .filter(|&&s| s).count() as f32 / self.success_rate_window.len() as f32;
        success_rate > 0.8 && self.consecutive_failures < 5
    }

    /// Get recommended action
    pub fn recommended_action(&self) -> HealthAction {
        if self.is_healthy() {
            HealthAction::Continue
        } else if self.consecutive_failures > 10 {
            HealthAction::ReinitializeConnection
        } else {
            HealthAction::SlowDown
        }
    }
}
```

**Recovery Actions:**
1. **SlowDown**: Reduce concurrency, increase delays
2. **ReinitializeConnection**: Drop and recreate HTTP client
3. **Failover**: Switch to backup API endpoint (if configured)

**Implementation Complexity:** Medium
**Estimated Effort:** 2 days

---

### 1.5 Rate Limit Adaptation

**Current:** Fixed rate limiter using `governor` crate

**Enhancement:** Adaptive rate limiting based on API responses

```rust
pub struct AdaptiveRateLimiter {
    /// Current rate limit
    current_rate: f32,
    /// Minimum rate (floor)
    min_rate: f32,
    /// Maximum rate (ceiling)
    max_rate: f32,
    /// Rate increase step
    increase_step: f32,
    /// Rate decrease factor (on 429)
    decrease_factor: f32,
    /// Last rate adjustment time
    last_adjustment: DateTime<Utc>,
}

impl AdaptiveRateLimiter {
    /// Adjust rate based on response
    pub fn on_response(&mut self, status: StatusCode) {
        match status {
            429 => {
                // Hit rate limit - reduce aggressively
                self.current_rate = (self.current_rate * self.decrease_factor)
                    .max(self.min_rate);
            }
            200..=299 => {
                // Success - slowly probe higher
                if self.last_adjustment.elapsed() > Duration::from_secs(60) {
                    self.current_rate = (self.current_rate + self.increase_step)
                        .min(self.max_rate);
                }
            }
            _ => {}
        }
    }
}
```

**Implementation Complexity:** Low-Medium
**Estimated Effort:** 1 day

---

## 2. Monitoring Enhancements

### 2.1 Progress Trends (Packages/Hour Over Time)

**Current:** Simple `packages_scanned / elapsed_time`

**Enhancement:** Time-series tracking with trend analysis

```rust
pub struct ProgressTrendTracker {
    /// Sampling interval in seconds
    sample_interval_secs: u64,
    /// Historical samples (rolling window)
    samples: VecDeque<ProgressSample>,
    /// Maximum samples to retain
    max_samples: usize,
}

#[derive(Clone)]
pub struct ProgressSample {
    pub timestamp: DateTime<Utc>,
    pub packages_scanned: usize,
    pub packages_flagged: usize,
    pub packages_malicious: usize,
    /// Instantaneous rate (packages/hour)
    pub rate_per_hour: f32,
}

impl ProgressTrendTracker {
    /// Calculate trend (increasing, stable, decreasing)
    pub fn trend(&self) -> TrendDirection {
        if self.samples.len() < 3 {
            return TrendDirection::Unknown;
        }

        let recent_avg = self.samples.iter().take(5)
            .map(|s| s.rate_per_hour).sum::<f32>() / 5.0;
        let older_avg = self.samples.iter().skip(5).take(5)
            .map(|s| s.rate_per_hour).sum::<f32>() / 5.0;

        let change = (recent_avg - older_avg) / older_avg;

        if change > 0.1 {
            TrendDirection::Increasing
        } else if change < -0.1 {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        }
    }

    /// Predict completion time based on trend
    pub fn predict_completion(&self, total_packages: usize) -> Option<DateTime<Utc>> {
        let current_rate = self.samples.back()?.rate_per_hour;
        let trend = self.trend();

        // Adjust rate based on trend
        let adjusted_rate = match trend {
            TrendDirection::Decreasing => current_rate * 0.9,
            TrendDirection::Increasing => current_rate * 1.1,
            _ => current_rate,
        };

        let remaining = total_packages - self.samples.back()?.packages_scanned;
        let hours_remaining = remaining as f32 / adjusted_rate;

        Some(Utc::now() + Duration::from_secs((hours_remaining * 3600.0) as u64))
    }
}
```

**Visualization (TUI):**
```
Progress Rate (packages/hour)
  500 ┤     ╭──╮
  400 ┤  ╭──╯  ╰──╮
  300 ┤──╯        ╰──╮
  200 ┤              ╰──
  100 ┤
    0 ┼────────────────────
      0h   2h   4h   6h   8h

Trend: ↓ Decreasing (network slowdown detected)
```

**Implementation Complexity:** Medium
**Estimated Effort:** 2 days

---

### 2.2 ETA Refinement Based on Actual Performance

**Current:** Simple linear extrapolation

**Enhancement:** Multi-factor ETA calculation

```rust
pub struct EtaCalculator {
    /// Base ETA from simple linear projection
    linear_eta: Duration,
    /// Trend-adjusted ETA
    trend_adjusted_eta: Duration,
    /// Wave-specific ETAs (some waves slower)
    wave_etAs: HashMap<String, Duration>,
    /// Confidence level (0.0 - 1.0)
    confidence: f32,
}

impl EtaCalculator {
    pub fn calculate(&self, state: &CampaignState) -> EtaEstimate {
        let linear = self.linear_projection(state);
        let trend_adjusted = self.apply_trend(linear);
        let wave_adjusted = self.apply_wave_factors(trend_adjusted);

        EtaEstimate {
            optimistic: wave_adjusted * 0.8,
            expected: wave_adjusted,
            pessimistic: wave_adjusted * 1.5,
            confidence: self.calculate_confidence(),
        }
    }

    fn calculate_confidence(&self) -> f32 {
        // Higher confidence with:
        // - More samples
        // - Stable trend
        // - Low variance in recent samples
        // - No recent errors
    }
}

pub struct EtaEstimate {
    pub optimistic: Duration,
    pub expected: Duration,
    pub pessimistic: Duration,
    pub confidence: f32,  // 0.0 - 1.0
}
```

**Display Format:**
```
ETA: 4h 30m (expected)
     3h 36m - 6h 45m (80% confidence)
```

**Implementation Complexity:** Medium
**Estimated Effort:** 1-2 days

---

### 2.3 Anomaly Detection

**Detect:** Sudden slowdowns, high false positive rates, unusual patterns

```rust
pub struct AnomalyDetector {
    /// Baseline scan rate
    baseline_rate: f32,
    /// Baseline flag rate
    baseline_flag_rate: f32,
    /// Standard deviation of rate
    rate_stddev: f32,
    /// Alert thresholds (standard deviations)
    slowdown_threshold: f32,
    flag_rate_threshold: f32,
}

#[derive(Debug, Clone)]
pub enum Anomaly {
    /// Scan rate dropped significantly
    Slowdown {
        expected_rate: f32,
        actual_rate: f32,
        severity: Severity,
    },
    /// Unusual flag rate (possible bad wave config)
    HighFlagRate {
        expected_rate: f32,
        actual_rate: f32,
        severity: Severity,
    },
    /// Error rate spike
    ErrorSpike {
        error_rate: f32,
        severity: Severity,
    },
    /// Memory usage growing unbounded
    MemoryLeak {
        growth_rate_mb_per_hour: f32,
        severity: Severity,
    },
}

impl AnomalyDetector {
    pub fn check(&self, current: &ProgressSample) -> Option<Anomaly> {
        // Check for slowdown
        let rate_drop = (self.baseline_rate - current.rate_per_hour) / self.baseline_rate;
        if rate_drop > 0.5 {
            return Some(Anomaly::Slowdown {
                expected_rate: self.baseline_rate,
                actual_rate: current.rate_per_hour,
                severity: Severity::High,
            });
        }

        // Check for unusual flag rate
        let flag_rate = current.packages_flagged as f32 / current.packages_scanned as f32;
        if flag_rate > self.baseline_flag_rate * 3.0 {
            return Some(Anomaly::HighFlagRate {
                expected_rate: self.baseline_flag_rate,
                actual_rate: flag_rate,
                severity: Severity::Medium,
            });
        }

        None
    }
}
```

**Alert Actions:**
- Log warning with details
- Send notification (if configured)
- Suggest operator action
- Auto-adjust (optional): reduce concurrency, increase checkpoint frequency

**Implementation Complexity:** Medium-High
**Estimated Effort:** 3 days

---

### 2.4 Resource Usage Monitoring

**Track:** Memory, CPU, disk usage over time

```rust
pub struct ResourceMonitor {
    /// Sampling interval
    sample_interval: Duration,
    /// Historical data
    samples: VecDeque<ResourceSample>,
}

#[derive(Clone)]
pub struct ResourceSample {
    pub timestamp: DateTime<Utc>,
    /// Memory usage in MB
    pub memory_mb: u64,
    /// CPU usage percentage (0-100)
    pub cpu_percent: f32,
    /// Disk usage for checkpoint DB in MB
    pub disk_mb: u64,
    /// Open file descriptors
    pub open_fds: u32,
}

impl ResourceMonitor {
    pub fn collect_sample(&mut self) -> Result<ResourceSample> {
        let memory = self.get_memory_usage()?;
        let cpu = self.get_cpu_usage()?;
        let disk = self.get_disk_usage()?;
        let fds = self.get_open_fds()?;

        let sample = ResourceSample {
            timestamp: Utc::now(),
            memory_mb: memory,
            cpu_percent: cpu,
            disk_mb: disk,
            open_fds: fds,
        };

        self.samples.push_back(sample.clone());
        if self.samples.len() > 360 {  // Keep 6 hours at 1/min
            self.samples.pop_front();
        }

        Ok(sample)
    }

    /// Check for resource issues
    pub fn check_thresholds(&self, config: &ResourceThresholds) -> Vec<ResourceAlert> {
        let mut alerts = Vec::new();
        let latest = self.samples.back().unwrap();

        if latest.memory_mb > config.memory_warning_mb {
            alerts.push(ResourceAlert::MemoryHigh { current: latest.memory_mb });
        }

        if latest.disk_mb > config.disk_warning_mb {
            alerts.push(ResourceAlert::DiskHigh { current: latest.disk_mb });
        }

        alerts
    }
}
```

**Dependencies:** `sysinfo` crate for cross-platform resource monitoring

```toml
[dependencies]
sysinfo = "0.30"
```

**Implementation Complexity:** Low-Medium
**Estimated Effort:** 1 day

---

### 2.5 Alert Thresholds

**Configurable thresholds for proactive alerting:**

```toml
[alerts]
# Scan rate alerts
[alerts.scan_rate]
# Alert if rate drops below this percentage of baseline
slowdown_threshold_percent = 50
# Minimum samples before alerting (avoid false alarms)
min_samples = 10

# Flag rate alerts
[alerts.flag_rate]
# Alert if flag rate exceeds this percentage
high_flag_rate_threshold_percent = 20

# Error rate alerts
[alerts.error_rate]
# Alert if error rate exceeds this percentage
high_error_rate_threshold_percent = 10

# Resource alerts
[alerts.resources]
memory_warning_mb = 800
memory_critical_mb = 950
disk_warning_mb = 1000
disk_critical_mb = 1800

# Notification channels
[alerts.notifications]
# Enable/disable notification types
email_enabled = false
slack_enabled = true
webhook_enabled = true
```

**Implementation Complexity:** Low
**Estimated Effort:** 0.5 days

---

## 3. Management Features

### 3.1 Campaign Queue with Priorities

**Enable:** Multiple campaigns waiting to run

```rust
pub struct CampaignQueue {
    /// Queue of pending campaigns
    pending: Vec<CampaignQueueEntry>,
    /// Currently running campaign
    running: Option<CampaignQueueEntry>,
    /// Recently completed (for history)
    completed: VecDeque<CampaignQueueEntry>,
}

#[derive(Clone)]
pub struct CampaignQueueEntry {
    pub id: String,
    pub name: String,
    pub priority: Priority,
    pub config_path: PathBuf,
    pub estimated_duration: Option<Duration>,
    pub estimated_packages: usize,
    pub submitted_at: DateTime<Utc>,
    pub submitted_by: String,
    pub status: QueueStatus,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum QueueStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl CampaignQueue {
    /// Add campaign to queue
    pub fn enqueue(&mut self, entry: CampaignQueueEntry) {
        self.pending.push(entry);
        // Sort by priority (critical > high > medium > low), then by submission time
        self.pending.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then(a.submitted_at.cmp(&b.submitted_at))
        });
    }

    /// Get next campaign to run
    pub fn dequeue_next(&mut self) -> Option<CampaignQueueEntry> {
        if self.running.is_some() {
            return None;  // Already running
        }
        self.pending.pop(0).map(|entry| {
            self.running = Some(entry.clone());
            entry
        })
    }
}
```

**CLI Commands:**
```bash
# Submit campaign to queue
glassware-orchestrator queue submit wave7.toml --priority high

# View queue status
glassware-orchestrator queue status

# Reorder queue
glassware-orchestrator queue reorder <campaign-id> --position 2

# Cancel queued campaign
glassware-orchestrator queue cancel <campaign-id>
```

**Implementation Complexity:** Medium
**Estimated Effort:** 2-3 days

---

### 3.2 Multi-Campaign Orchestration

**Enable:** Running multiple campaigns with resource allocation

```rust
pub struct MultiCampaignOrchestrator {
    /// Campaign queue
    queue: CampaignQueue,
    /// Resource manager
    resources: ResourceManager,
    /// Active campaign executors
    active_campaigns: HashMap<String, CampaignExecutor>,
    /// Global configuration
    config: MultiCampaignConfig,
}

#[derive(Clone)]
pub struct MultiCampaignConfig {
    /// Maximum concurrent campaigns
    max_concurrent: usize,
    /// Default concurrency per campaign
    default_concurrency: usize,
    /// Resource allocation strategy
    allocation_strategy: AllocationStrategy,
}

#[derive(Clone)]
pub enum AllocationStrategy {
    /// Equal share among all campaigns
    EqualShare,
    /// Priority-based allocation
    PriorityBased,
    /// Fixed allocation per campaign
    FixedAllocation { concurrency_per_campaign: usize },
}

impl MultiCampaignOrchestrator {
    /// Start a new campaign if resources available
    pub async fn try_start_campaign(&mut self) -> Result<Option<String>> {
        if self.active_campaigns.len() >= self.config.max_concurrent {
            return Ok(None);  // At capacity
        }

        if let Some(entry) = self.queue.dequeue_next() {
            let concurrency = self.resources.allocate_concurrency(&entry);

            let executor = self.create_executor(&entry, concurrency).await?;
            let case_id = executor.case_id().to_string();

            self.active_campaigns.insert(case_id.clone(), executor);

            info!("Started campaign {} (case: {})", entry.name, case_id);
            return Ok(Some(case_id));
        }

        Ok(None)
    }
}
```

**Implementation Complexity:** High
**Estimated Effort:** 5-7 days

---

### 3.3 Resource Allocation Per Campaign

**Enable:** Fair resource distribution

```rust
pub struct ResourceManager {
    /// Total available concurrency
    total_concurrency: usize,
    /// Currently allocated
    allocated: usize,
    /// Per-campaign allocations
    allocations: HashMap<String, usize>,
}

impl ResourceManager {
    pub fn allocate_concurrency(&mut self, campaign: &CampaignQueueEntry) -> usize {
        let available = self.total_concurrency - self.allocated;

        // Priority-based allocation
        let allocation = match campaign.priority {
            Priority::Critical => available,  // Take all available
            Priority::High => (available * 2 / 3).max(1),
            Priority::Medium => (available / 2).max(1),
            Priority::Low => (available / 4).max(1),
        };

        self.allocated += allocation;
        self.allocations.insert(campaign.id.clone(), allocation);

        allocation
    }

    pub fn release_concurrency(&mut self, campaign_id: &str) {
        if let Some(allocated) = self.allocations.remove(campaign_id) {
            self.allocated -= allocated;
        }
    }
}
```

**Implementation Complexity:** Medium
**Estimated Effort:** 1-2 days

---

### 3.4 Pause/Resume Scheduling

**Enable:** Automatic pause during business hours, resume overnight

```rust
pub struct ScheduledPause {
    /// Pause schedule (cron-like)
    schedule: PauseSchedule,
    /// Timezone for schedule interpretation
    timezone: Tz,
}

#[derive(Clone)]
pub enum PauseSchedule {
    /// Pause during specific hours
    DailyHours { start_hour: u8, end_hour: u8 },
    /// Pause on specific days
    WeeklyDays { days: Vec<DayOfWeek> },
    /// Custom cron expression
    Cron(String),
    /// One-time pause at specific time
    OneTime(DateTime<Utc>),
}

impl ScheduledPause {
    /// Check if campaign should be paused now
    pub fn should_pause(&self, now: DateTime<Utc>) -> bool {
        let local = now.with_timezone(&self.timezone);

        match &self.schedule {
            PauseSchedule::DailyHours { start_hour, end_hour } => {
                let hour = local.hour() as u8;
                hour >= *start_hour && hour < *end_hour
            }
            PauseSchedule::WeeklyDays { days } => {
                days.contains(&local.weekday())
            }
            PauseSchedule::OneTime(time) => {
                now >= *time
            }
            _ => false,
        }
    }
}
```

**Configuration:**
```toml
[schedule]
# Pause during business hours (9 AM - 6 PM local time)
pause_hours = { start = 9, end = 18, timezone = "America/New_York" }

# Or: Pause on weekends
pause_days = ["saturday", "sunday"]

# Or: Custom cron (pause at 2 AM daily for maintenance)
pause_cron = "0 2 * * *"
```

**Implementation Complexity:** Medium
**Estimated Effort:** 2 days

---

### 3.5 Automatic Report Generation on Milestones

**Generate:** Reports at campaign milestones

```rust
pub struct MilestoneReporter {
    /// Milestones to trigger reports
    milestones: Vec<Milestone>,
    /// Report formats to generate
    formats: Vec<ReportFormat>,
    /// Output directory
    output_dir: PathBuf,
}

#[derive(Clone)]
pub enum Milestone {
    /// After N packages scanned
    PackagesScanned(usize),
    /// After N% completion
    PercentageComplete(u8),
    /// After each wave completes
    WaveComplete,
    /// On finding malicious package
    MaliciousFound,
    /// Time-based (every N hours)
    TimeInterval(Duration),
}

impl MilestoneReporter {
    pub fn check_milestones(&mut self, state: &CampaignState) -> Vec<ReportType> {
        let mut reports = Vec::new();

        for milestone in &self.milestones {
            if self.should_trigger(milestone, state) {
                reports.push(ReportType::Milestone {
                    milestone: milestone.clone(),
                    timestamp: Utc::now(),
                });
            }
        }

        reports
    }
}
```

**Configuration:**
```toml
[reporting]
output_dir = "reports"
formats = ["markdown", "json", "sarif"]

[reporting.milestones]
# Generate report every 25% completion
percentage_intervals = [25, 50, 75, 100]
# Generate report after each wave
on_wave_complete = true
# Generate report when malicious package found
on_malicious = true
# Generate hourly progress report
hourly_interval = true
```

**Implementation Complexity:** Low-Medium
**Estimated Effort:** 1-2 days

---

## 4. UX for Long Runs

### 4.1 Detached Monitoring (Start in Background, Attach Later)

**Enable:** Start campaign in background, monitor from different session

```rust
// Campaign runs as daemon process
// State persisted to SQLite
// CLI can attach to running campaign

// Start detached
glassware-orchestrator campaign run wave6.toml --detach

// Attach to running campaign
glassware-orchestrator campaign attach <case-id>

// List running campaigns
glassware-orchestrator campaign list
```

**Architecture:**
```
┌─────────────────┐     ┌─────────────────┐
│   Campaign      │     │   CLI Client    │
│   (daemon)      │◄───►│   (attach)      │
│                 │     │                 │
│ ┌─────────────┐ │     │ ┌─────────────┐ │
│ │ State Mgr   │ │     │ │ TUI/CLI     │ │
│ │ (SQLite)    │ │     │ │ (display)   │ │
│ └─────────────┘ │     │ └─────────────┘ │
└─────────────────┘     └─────────────────┘
```

**Implementation Complexity:** Medium
**Estimated Effort:** 2-3 days

---

### 4.2 Remote Monitoring (Web Dashboard, SSH TUI)

**Option A: Web Dashboard**

```
┌─────────────────────────────────────────────────────────┐
│  Glassworks Campaign Dashboard                           │
├─────────────────────────────────────────────────────────┤
│  Campaign: Wave 6 Calibration                    Running │
│  Progress: ████████████░░░░░░░░ 67%  ETA: 2h 30m        │
├─────────────────────────────────────────────────────────┤
│  Waves:                                                   │
│  ✅ 6A: Known Malicious    2/2 scanned, 2 flagged       │
│  🟡 6B: Clean Baseline     3/5 scanned, 0 flagged       │
│  ⏳ 6C: React Native       0/4 scanned, 0 flagged       │
├─────────────────────────────────────────────────────────┤
│  Scan Rate: 450 packages/hour  (trend: ↓ decreasing)    │
│  Malicious: 2  |  Flagged: 15  |  Clean: 120           │
└─────────────────────────────────────────────────────────┘
```

**Tech Stack:**
- Backend: `axum` or `actix-web`
- Frontend: Simple HTML + HTMX or React
- Real-time: WebSocket or Server-Sent Events

**Implementation Complexity:** High
**Estimated Effort:** 5-7 days

**Option B: SSH TUI (Remote Terminal)**

```bash
# Connect to remote campaign via SSH
ssh user@server -- glassware-orchestrator campaign attach <case-id>
```

**Implementation Complexity:** Low (uses existing TUI)
**Estimated Effort:** 0.5 days

---

### 4.3 Notification System (Email, Slack, Webhook)

**Enable:** Alerts and progress notifications

```rust
pub struct NotificationManager {
    channels: Vec<NotificationChannel>,
    /// Events that trigger notifications
    triggers: Vec<NotificationTrigger>,
}

#[derive(Clone)]
pub enum NotificationChannel {
    Email { to: String, smtp: SmtpConfig },
    Slack { webhook_url: String },
    Webhook { url: String, headers: HashMap<String, String> },
}

#[derive(Clone)]
pub enum NotificationTrigger {
    CampaignStarted,
    CampaignCompleted,
    CampaignFailed,
    MaliciousFound,
    AnomalyDetected(Anomaly),
    MilestoneReached(Milestone),
    ResourceAlert(ResourceAlert),
}

impl NotificationManager {
    pub async fn notify(&self, event: NotificationEvent) {
        for channel in &self.channels {
            if self.should_notify(&event, channel) {
                channel.send(event.clone()).await;
            }
        }
    }
}
```

**Configuration:**
```toml
[notifications]
# Slack notifications
[notifications.slack]
webhook_url = "https://hooks.slack.com/..."
# Events to notify
notify_on = ["malicious_found", "campaign_completed", "anomaly_detected"]

# Email notifications
[notifications.email]
smtp_server = "smtp.example.com"
smtp_port = 587
from = "glassworks@example.com"
to = ["security-team@example.com"]
notify_on = ["campaign_failed", "resource_alert"]

# Webhook notifications
[notifications.webhook]
url = "https://api.example.com/glassworks-events"
headers = { Authorization = "Bearer ..." }
notify_on = ["*"]  # All events
```

**Implementation Complexity:** Medium
**Estimated Effort:** 2-3 days

---

### 4.4 Session Persistence (Survive Terminal Close)

**Enable:** Campaign continues after terminal closes

**Approach 1: systemd Service (Linux)**

```ini
# /etc/systemd/system/glassworks-campaign.service
[Unit]
Description=Glassworks Campaign Runner
After=network.target

[Service]
Type=simple
User=glassworks
WorkingDirectory=/opt/glassworks
ExecStart=/opt/glassworks/glassware-orchestrator campaign run wave6.toml --detach
Restart=on-failure
RestartSec=30

# Environment
Environment="RUST_LOG=info"
Environment="GLASSWARE_CONFIG=/opt/glassworks/config.toml"

# Logging
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

**Usage:**
```bash
# Start campaign as service
sudo systemctl start glassworks-campaign

# Check status
sudo systemctl status glassworks-campaign

# View logs
journalctl -u glassworks-campaign -f
```

**Approach 2: tmux/screen (Simple)**

```bash
# Start in tmux session
tmux new -s glassworks "glassware-orchestrator campaign run wave6.toml"

# Detach: Ctrl+B, D
# Reattach: tmux attach -t glassworks
```

**Implementation Complexity:** Low (systemd) / None (tmux)
**Estimated Effort:** 0.5 days (systemd unit file)

---

### 4.5 Log Rotation and Archival

**Problem:** Log files grow unbounded during days-long runs

**Solution:** Structured logging with rotation

```toml
[logging]
# Log file configuration
[logging.file]
path = "/var/log/glassworks/campaign.log"
level = "info"

# Rotation settings
[logging.rotation]
# Rotate when file exceeds this size
max_size_mb = 100
# Keep this many rotated files
max_files = 10
# Compress rotated files
compress = true

# Archive settings
[logging.archive]
# Archive rotated files older than
archive_after_days = 7
# Archive location
archive_dir = "/var/log/glassworks/archive"
```

**Implementation:** Use `tracing-appender` with `rolling` feature

```toml
[dependencies]
tracing-appender = "0.2"
tracing-subscriber = { version = "0.3", features = ["rolling"] }
```

**Implementation Complexity:** Low
**Estimated Effort:** 0.5 days

---

## 5. Examples from Other Projects

### 5.1 Folding@Home (Distributed Computing)

**Relevant Features:**
- Work unit checkpointing (survive restarts)
- Progress estimation with refinement
- Resource usage monitoring
- Automatic retry on failure

**Lessons:**
- Checkpoint frequently but not excessively (balance overhead vs. recovery)
- Provide clear progress indicators even for uncertain workloads
- Allow users to set resource limits

---

### 5.2 BOINC (Scientific Computing)

**Relevant Features:**
- Project queue management
- Priority-based scheduling
- Deadline tracking
- Credit/reward system (gamification)

**Lessons:**
- Queue visualization helps users understand wait times
- Deadlines create urgency but can cause stress
- Resource sharing between projects needs fair allocation

---

### 5.3 Torrent Clients (Long-Running Downloads)

**Relevant Features:**
- Peer discovery and connection management
- Rate limiting and adaptation
- Piece-level checkpointing
- ETA calculation with trend analysis

**Lessons:**
- ETA should show range (optimistic/pessimistic) not single value
- Rate limiting should adapt to network conditions
- Checkpoint at natural boundaries (piece completion)

---

### 5.4 CI/CD Pipelines (Long Build Queues)

**Relevant Features:**
- Job queue with priorities
- Resource pools and allocation
- Parallel execution with dependencies
- Notification on completion/failure

**Lessons:**
- Queue visibility reduces anxiety about "stuck" jobs
- Parallel execution needs careful resource management
- Failure notifications should include actionable information

---

## 6. Feature Prioritization

### Priority Matrix

| Feature | Impact | Effort | Priority |
|---------|--------|--------|----------|
| **Adaptive Checkpointing** | High | Medium | P0 |
| **Detached Monitoring** | High | Medium | P0 |
| **Notification System** | High | Medium | P0 |
| **Progress Trends + ETA Refinement** | Medium | Medium | P1 |
| **Resource Monitoring** | Medium | Low | P1 |
| **Anomaly Detection** | Medium | Medium | P1 |
| **Campaign Queue** | Medium | Medium | P2 |
| **Log Rotation** | Low | Low | P2 |
| **Pause/Resume Scheduling** | Low | Medium | P3 |
| **Multi-Campaign Orchestration** | High | High | P3 |
| **Web Dashboard** | Medium | High | P3 |
| **Database Compaction** | Low | Medium | P3 |

### Rationale

**P0 (Immediate):**
- **Adaptive Checkpointing**: Critical for reliability in days-long runs
- **Detached Monitoring**: Essential for operator quality of life
- **Notification System**: Operators need to know about important events

**P1 (Next):**
- **Progress Trends + ETA**: Better visibility into campaign health
- **Resource Monitoring**: Prevent resource exhaustion
- **Anomaly Detection**: Early warning of problems

**P2 (Later):**
- **Campaign Queue**: Useful for batch operations
- **Log Rotation**: Operational hygiene

**P3 (Future):**
- **Multi-Campaign**: Complex, may not be needed initially
- **Web Dashboard**: Nice-to-have, SSH TUI works
- **Database Compaction**: Only needed for very long runs
- **Scheduling**: Specialized use case

---

## 7. Implementation Phases

### Phase 1: Core Reliability (Week 1-2)

**Goal:** Support reliable days-long runs

| Task | Effort | Dependencies |
|------|--------|--------------|
| Adaptive checkpointing | 2-3 days | Existing checkpoint module |
| Retry with circuit breaker | 2 days | Existing retry module |
| Rate limit adaptation | 1 day | Existing rate limiter |
| Detached monitoring | 2-3 days | State manager |
| Session persistence (systemd) | 0.5 days | Detached monitoring |

**Deliverables:**
- Campaigns can run for days without data loss
- Automatic recovery from transient failures
- Operators can detach and reattach

---

### Phase 2: Enhanced Monitoring (Week 3-4)

**Goal:** Better visibility into campaign health

| Task | Effort | Dependencies |
|------|--------|--------------|
| Progress trends | 2 days | Progress tracker |
| ETA refinement | 1-2 days | Progress trends |
| Resource monitoring | 1 day | - |
| Anomaly detection | 3 days | Progress trends, resource monitoring |
| Alert thresholds | 0.5 days | Anomaly detection |
| Notification system | 2-3 days | Alert thresholds |

**Deliverables:**
- Operators can see trends and refined ETAs
- Automatic alerts for anomalies
- Notifications via Slack/email/webhook

---

### Phase 3: Management Features (Week 5-6)

**Goal:** Better campaign management

| Task | Effort | Dependencies |
|------|--------|--------------|
| Campaign queue | 2-3 days | - |
| Resource allocation | 1-2 days | Campaign queue |
| Pause/resume scheduling | 2 days | - |
| Milestone reporting | 1-2 days | Report generator |
| Log rotation | 0.5 days | - |

**Deliverables:**
- Queue management for multiple campaigns
- Scheduled pause/resume
- Automatic milestone reports

---

### Phase 4: Advanced Features (Week 7+)

**Goal:** Advanced capabilities

| Task | Effort | Dependencies |
|------|--------|--------------|
| Multi-campaign orchestration | 5-7 days | Campaign queue, resource allocation |
| Web dashboard | 5-7 days | State manager, event bus |
| Database compaction | 1-2 days | Checkpoint module |

**Deliverables:**
- Run multiple campaigns concurrently
- Web-based monitoring
- Efficient long-term storage

---

## 8. Architecture Changes Needed

### 8.1 New Modules

```
glassware-orchestrator/src/
├── campaign/
│   ├── ... (existing)
│   ├── queue.rs              # NEW: Campaign queue
│   ├── scheduler.rs          # NEW: Pause/resume scheduling
│   ├── notifier.rs           # NEW: Notification system
│   └── milestone.rs          # NEW: Milestone reporting
│
├── monitoring/
│   ├── mod.rs                # NEW: Monitoring module
│   ├── trends.rs             # NEW: Progress trends
│   ├── eta.rs                # NEW: ETA calculation
│   ├── anomaly.rs            # NEW: Anomaly detection
│   └── resources.rs          # NEW: Resource monitoring
│
├── reliability/
│   ├── mod.rs                # NEW: Reliability module
│   ├── adaptive_checkpoint.rs # NEW: Adaptive checkpointing
│   ├── circuit_breaker.rs    # NEW: Circuit breaker
│   └── rate_adaptation.rs    # NEW: Rate limit adaptation
│
└── web/                      # NEW: Web dashboard (Phase 4)
    ├── mod.rs
    ├── server.rs
    └── routes/
```

### 8.2 Dependency Additions

```toml
[dependencies]
# Resource monitoring
sysinfo = "0.30"

# Scheduling
cron = "0.12"

# Email notifications
lettre = "0.11"

# Web dashboard (Phase 4)
axum = "0.7"
tokio-tungstenite = "0.21"  # WebSocket
```

### 8.3 Database Schema Changes

```sql
-- Campaign queue table
CREATE TABLE IF NOT EXISTS campaign_queue (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    priority TEXT NOT NULL,
    config_path TEXT NOT NULL,
    status TEXT NOT NULL,
    submitted_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    submitted_by TEXT
);

-- Resource usage history
CREATE TABLE IF NOT EXISTS resource_samples (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    case_id TEXT NOT NULL,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    memory_mb INTEGER,
    cpu_percent REAL,
    disk_mb INTEGER
);

-- Progress samples for trend analysis
CREATE TABLE IF NOT EXISTS progress_samples (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    case_id TEXT NOT NULL,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    packages_scanned INTEGER,
    packages_flagged INTEGER,
    packages_malicious INTEGER,
    rate_per_hour REAL
);

-- Anomaly log
CREATE TABLE IF NOT EXISTS anomaly_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    case_id TEXT NOT NULL,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    anomaly_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    details TEXT
);
```

---

## 9. Success Criteria

| Criterion | Target | Measurement |
|-----------|--------|-------------|
| **Reliability** | 99% completion rate for 100k+ package campaigns | Campaign logs |
| **Recovery** | <5 minute recovery from transient failures | Time-to-resume |
| **Checkpoint Overhead** | <5% performance impact | Benchmark comparison |
| **ETA Accuracy** | Within 20% of actual completion time | Post-campaign analysis |
| **Anomaly Detection** | Detect 90% of significant slowdowns within 5 minutes | Anomaly log review |
| **Notification Latency** | <1 minute from event to notification | Timestamp comparison |
| **Resource Usage** | <1GB memory for 100k package campaign | Resource monitoring |

---

## 10. Top 5 Recommended Features

### 1. Adaptive Checkpointing

**Rationale:** Foundation for reliable long runs. Fixed-interval checkpointing is inefficient - too frequent wastes resources, too infrequent risks data loss.

**Implementation Complexity:** Medium
**Estimated Effort:** 2-3 days
**Dependencies:** Existing checkpoint module

**Key Benefits:**
- Automatic adjustment based on error rate and scan speed
- Time-based fallback ensures checkpoints even during slow periods
- Wave boundary checkpoints for clean resume points

---

### 2. Detached Monitoring

**Rationale:** Operators cannot stay tied to terminal for days. Must support start-and-detach workflow.

**Implementation Complexity:** Medium
**Estimated Effort:** 2-3 days
**Dependencies:** State manager (already exists)

**Key Benefits:**
- Start campaign, disconnect, reattach later
- Multiple operators can monitor same campaign
- Survives SSH disconnection

---

### 3. Notification System

**Rationale:** Operators need to know about important events without constant monitoring.

**Implementation Complexity:** Medium
**Estimated Effort:** 2-3 days
**Dependencies:** Alert thresholds

**Key Benefits:**
- Slack/email/webhook notifications
- Configurable triggers (malicious found, campaign failed, anomaly detected)
- Reduces need for constant monitoring

---

### 4. Progress Trends + ETA Refinement

**Rationale:** Simple linear ETA is misleading for long runs. Trend analysis provides more accurate predictions.

**Implementation Complexity:** Medium
**Estimated Effort:** 2-3 days
**Dependencies:** Progress tracker

**Key Benefits:**
- Shows if scan rate is increasing/decreasing
- Provides confidence intervals for ETA
- Helps operators plan and identify problems early

---

### 5. Resource Monitoring

**Rationale:** Days-long runs can exhaust resources (memory leaks, disk growth). Early detection prevents crashes.

**Implementation Complexity:** Low-Medium
**Estimated Effort:** 1 day
**Dependencies:** None (uses `sysinfo` crate)

**Key Benefits:**
- Track memory, CPU, disk usage over time
- Alert before resource exhaustion
- Historical data for capacity planning

---

## 11. Suggested Implementation Order

```
Week 1-2: Core Reliability
├── Adaptive Checkpointing (P0)
├── Detached Monitoring (P0)
└── Session Persistence (systemd) (P0)

Week 3-4: Enhanced Monitoring
├── Resource Monitoring (P1)
├── Progress Trends (P1)
├── ETA Refinement (P1)
├── Alert Thresholds (P1)
└── Notification System (P0)

Week 5-6: Management Features
├── Campaign Queue (P2)
├── Log Rotation (P2)
├── Pause/Resume Scheduling (P3)
└── Milestone Reporting (P3)

Week 7+: Advanced Features
├── Multi-Campaign Orchestration (P3)
├── Web Dashboard (P3)
└── Database Compaction (P3)
```

---

## 12. Risk Mitigation

| Risk | Mitigation |
|------|------------|
| **Checkpoint corruption** | Transactional writes, checksums, backup checkpoints |
| **Memory leak** | Resource monitoring with alerts, automatic restart on threshold |
| **Disk exhaustion** | Log rotation, checkpoint compaction, disk usage alerts |
| **Network instability** | Circuit breaker, exponential backoff, connection health monitoring |
| **Alert fatigue** | Configurable thresholds, alert suppression, severity levels |
| **Notification spam** | Rate limiting, batching, quiet hours |

---

## 13. Testing Strategy

### Unit Tests
- Adaptive checkpoint logic
- ETA calculation accuracy
- Anomaly detection thresholds
- Notification channel delivery

### Integration Tests
- Campaign resume after simulated crash
- Multi-day simulated run (accelerated time)
- Resource exhaustion scenarios
- Network failure injection

### Load Tests
- 100k+ package campaign
- Multiple concurrent campaigns
- High notification volume

---

## 14. Conclusion

This design provides a comprehensive roadmap for supporting days-long campaign runs in Glassworks. The phased approach prioritizes reliability and operator experience first, then adds management capabilities, and finally advanced features.

**Key Takeaways:**
1. **Reliability is foundational**: Adaptive checkpointing and retry logic enable long runs
2. **Visibility reduces anxiety**: Progress trends, refined ETA, and notifications keep operators informed
3. **Resource awareness prevents crashes**: Monitoring memory, disk, and CPU avoids unexpected failures
4. **Start simple, iterate**: Begin with P0 features, add complexity based on operational experience

**Next Steps:**
1. Review and approve this design
2. Begin Phase 1 implementation (Adaptive Checkpointing, Detached Monitoring)
3. Test with extended campaign runs (10k+ packages)
4. Iterate based on operational feedback

---

**Document History:**
- v1.0.0 (2026-03-23): Initial design specification
