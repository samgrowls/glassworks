//! Event bus for campaign events (pub/sub pattern).
//!
//! Uses tokio's broadcast channel for efficient event distribution
//! to multiple subscribers (TUI, CLI, logging, etc.).

use tokio::sync::broadcast;
use tracing::debug;

use crate::campaign::types::{CaseId, CampaignStatus, WaveStatus};
use crate::llm::LlmVerdict;

/// Campaign events published to all subscribers.
#[derive(Debug, Clone)]
pub enum CampaignEvent {
    // === Campaign Lifecycle ===
    /// Campaign execution started.
    CampaignStarted {
        case_id: String,
        campaign_name: String,
        config_hash: String,
    },
    /// Campaign paused by user command.
    CampaignPaused {
        case_id: String,
        reason: String,
    },
    /// Campaign resumed after pause.
    CampaignResumed {
        case_id: String,
    },
    /// Campaign completed successfully.
    CampaignCompleted {
        case_id: String,
        total_scanned: usize,
        total_flagged: usize,
        total_malicious: usize,
        duration_seconds: u64,
    },
    /// Campaign failed with an error.
    CampaignFailed {
        case_id: String,
        error: String,
    },
    /// Campaign cancelled by user.
    CampaignCancelled {
        case_id: String,
    },

    // === Wave Lifecycle ===
    /// Wave execution started.
    WaveStarted {
        wave_id: String,
        name: String,
        packages_total: usize,
    },
    /// Wave progress update.
    WaveProgress {
        wave_id: String,
        scanned: usize,
        flagged: usize,
        malicious: usize,
    },
    /// Wave completed successfully.
    WaveCompleted {
        wave_id: String,
        packages_scanned: usize,
        packages_flagged: usize,
        packages_malicious: usize,
        duration_seconds: u64,
    },
    /// Wave failed with an error.
    WaveFailed {
        wave_id: String,
        error: String,
    },
    /// Wave skipped by user command.
    WaveSkipped {
        wave_id: String,
        reason: String,
    },

    // === Package-Level Events ===
    /// Package scan completed.
    PackageScanned {
        package: String,
        version: String,
        wave_id: String,
        threat_score: f32,
        findings_count: usize,
    },
    /// Package flagged with findings.
    PackageFlagged {
        package: String,
        version: String,
        wave_id: String,
        findings_count: usize,
        threat_score: f32,
    },
    /// Package marked as malicious.
    PackageMalicious {
        package: String,
        version: String,
        wave_id: String,
        threat_score: f32,
        llm_verdict: Option<String>,
    },

    // === LLM Events ===
    /// LLM analysis started.
    LlmAnalysisStarted {
        package: String,
        tier: u8,
        model: String,
    },
    /// LLM analysis completed.
    LlmAnalysisCompleted {
        package: String,
        verdict: LlmVerdict,
        model: String,
        duration_ms: u64,
    },

    // === System Events ===
    /// Rate limit wait triggered.
    RateLimitWait {
        target: String,
        wait_ms: u64,
    },
    /// Checkpoint saved to disk.
    CheckpointSaved {
        path: String,
        case_id: String,
    },
    /// Evidence collected for a package.
    EvidenceCollected {
        package: String,
        version: String,
        path: String,
    },

    // === Command Echo ===
    /// Command received from user.
    CommandReceived {
        command: CampaignCommand,
    },
    /// Command executed successfully.
    CommandExecuted {
        command: CampaignCommand,
        result: String,
    },
    /// Command rejected (invalid for current state).
    CommandRejected {
        command: CampaignCommand,
        reason: String,
    },
}

/// Commands that can steer campaign execution.
#[derive(Debug, Clone)]
pub enum CampaignCommand {
    // === Execution Control ===
    /// Pause campaign execution.
    Pause { reason: String },
    /// Resume paused campaign.
    Resume,
    /// Cancel campaign execution.
    Cancel { save_checkpoint: bool },

    // === Wave Control ===
    /// Skip a specific wave.
    SkipWave { wave_id: String },
    /// Retry a failed wave.
    RetryWave { wave_id: String },

    // === Runtime Adjustments ===
    /// Adjust concurrency level.
    SetConcurrency { concurrency: usize },
    /// Adjust rate limit.
    SetRateLimit { rate_limit: f32 },
    /// Toggle LLM analysis.
    ToggleLlm { enabled: bool },

    // === Diagnostics ===
    /// Dump current state to file.
    DumpState,
    /// Force metrics collection.
    CollectMetrics,
}

impl CampaignCommand {
    /// Get a human-readable name for the command.
    pub fn name(&self) -> &'static str {
        match self {
            CampaignCommand::Pause { .. } => "pause",
            CampaignCommand::Resume => "resume",
            CampaignCommand::Cancel { .. } => "cancel",
            CampaignCommand::SkipWave { .. } => "skip_wave",
            CampaignCommand::RetryWave { .. } => "retry_wave",
            CampaignCommand::SetConcurrency { .. } => "set_concurrency",
            CampaignCommand::SetRateLimit { .. } => "set_rate_limit",
            CampaignCommand::ToggleLlm { .. } => "toggle_llm",
            CampaignCommand::DumpState => "dump_state",
            CampaignCommand::CollectMetrics => "collect_metrics",
        }
    }
}

/// Event bus for campaign events.
///
/// Uses a broadcast channel to distribute events to multiple subscribers.
/// Slow subscribers may miss events if the buffer fills up.
pub struct EventBus {
    sender: broadcast::Sender<CampaignEvent>,
}

impl EventBus {
    /// Create a new event bus with the specified buffer size.
    ///
    /// # Arguments
    /// * `buffer_size` - Maximum number of events to buffer. Slow subscribers
    ///   will miss events if they fall behind by more than this amount.
    ///
    /// # Recommended buffer sizes
    /// - CLI monitoring: 64 events
    /// - TUI monitoring: 256 events
    /// - Full logging: 512+ events
    pub fn new(buffer_size: usize) -> Self {
        let (sender, _) = broadcast::channel(buffer_size);
        Self { sender }
    }

    /// Publish an event to all subscribers.
    ///
    /// This is non-blocking - the event is copied to the channel buffer
    /// and subscribers receive it asynchronously. If the buffer is full,
    /// the oldest events are dropped to make room.
    pub fn publish(&self, event: CampaignEvent) {
        let event_name = event_type_name(&event);
        
        match self.sender.send(event.clone()) {
            Ok(subscriber_count) => {
                debug!("Event '{}' published to {} subscribers", event_name, subscriber_count);
            }
            Err(broadcast::error::SendError(_)) => {
                // No active subscribers - event is dropped
                debug!("Event '{}' dropped (no subscribers)", event_name);
            }
        }
    }

    /// Subscribe to receive events.
    ///
    /// Returns a receiver that will receive all future events published
    /// to this bus. The receiver starts receiving from the point of
    /// subscription (no replay of past events).
    pub fn subscribe(&self) -> broadcast::Receiver<CampaignEvent> {
        self.sender.subscribe()
    }

    /// Get the number of active subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }

    /// Create a sender clone for passing to other components.
    pub fn clone_sender(&self) -> broadcast::Sender<CampaignEvent> {
        self.sender.clone()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(512)  // Default buffer: 512 events
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

/// Helper to get event type name for logging.
fn event_type_name(event: &CampaignEvent) -> &'static str {
    match event {
        CampaignEvent::CampaignStarted { .. } => "CampaignStarted",
        CampaignEvent::CampaignPaused { .. } => "CampaignPaused",
        CampaignEvent::CampaignResumed { .. } => "CampaignResumed",
        CampaignEvent::CampaignCompleted { .. } => "CampaignCompleted",
        CampaignEvent::CampaignFailed { .. } => "CampaignFailed",
        CampaignEvent::CampaignCancelled { .. } => "CampaignCancelled",
        CampaignEvent::WaveStarted { .. } => "WaveStarted",
        CampaignEvent::WaveProgress { .. } => "WaveProgress",
        CampaignEvent::WaveCompleted { .. } => "WaveCompleted",
        CampaignEvent::WaveFailed { .. } => "WaveFailed",
        CampaignEvent::WaveSkipped { .. } => "WaveSkipped",
        CampaignEvent::PackageScanned { .. } => "PackageScanned",
        CampaignEvent::PackageFlagged { .. } => "PackageFlagged",
        CampaignEvent::PackageMalicious { .. } => "PackageMalicious",
        CampaignEvent::LlmAnalysisStarted { .. } => "LlmAnalysisStarted",
        CampaignEvent::LlmAnalysisCompleted { .. } => "LlmAnalysisCompleted",
        CampaignEvent::RateLimitWait { .. } => "RateLimitWait",
        CampaignEvent::CheckpointSaved { .. } => "CheckpointSaved",
        CampaignEvent::EvidenceCollected { .. } => "EvidenceCollected",
        CampaignEvent::CommandReceived { .. } => "CommandReceived",
        CampaignEvent::CommandExecuted { .. } => "CommandExecuted",
        CampaignEvent::CommandRejected { .. } => "CommandRejected",
    }
}

/// Extension trait for convenient event publishing.
pub trait EventBusExt {
    /// Publish a campaign lifecycle event.
    fn publish_lifecycle(&self, event: CampaignEvent);
    
    /// Publish a wave lifecycle event.
    fn publish_wave(&self, event: CampaignEvent);
    
    /// Publish a package scan event.
    fn publish_package(&self, event: CampaignEvent);
}

impl EventBusExt for EventBus {
    fn publish_lifecycle(&self, event: CampaignEvent) {
        #[cfg(debug_assertions)]
        match &event {
            CampaignEvent::CampaignStarted { case_id, campaign_name, .. } => {
                tracing::info!("🚀 Campaign '{}' started (case: {})", campaign_name, case_id);
            }
            CampaignEvent::CampaignCompleted { case_id, total_scanned, total_malicious, .. } => {
                tracing::info!("✅ Campaign completed: {} scanned, {} malicious", total_scanned, total_malicious);
            }
            CampaignEvent::CampaignFailed { case_id, error } => {
                tracing::error!("❌ Campaign failed (case: {}): {}", case_id, error);
            }
            _ => {}
        }
        self.publish(event);
    }

    fn publish_wave(&self, event: CampaignEvent) {
        #[cfg(debug_assertions)]
        match &event {
            CampaignEvent::WaveStarted { name, packages_total, .. } => {
                tracing::info!("📦 Wave '{}' started ({} packages)", name, packages_total);
            }
            CampaignEvent::WaveCompleted { wave_id, packages_scanned, packages_flagged, .. } => {
                tracing::debug!("✅ Wave '{}' completed: {} scanned, {} flagged", wave_id, packages_scanned, packages_flagged);
            }
            CampaignEvent::WaveFailed { wave_id, error } => {
                tracing::error!("❌ Wave '{}' failed: {}", wave_id, error);
            }
            _ => {}
        }
        self.publish(event);
    }

    fn publish_package(&self, event: CampaignEvent) {
        #[cfg(debug_assertions)]
        match &event {
            CampaignEvent::PackageMalicious { package, version, threat_score, .. } => {
                tracing::warn!("🚨 MALICIOUS: {}@{} (score: {})", package, version, threat_score);
            }
            CampaignEvent::PackageFlagged { package, findings_count, .. } => {
                tracing::debug!("⚠️  Flagged: {} ({} findings)", package, findings_count);
            }
            _ => {}
        }
        self.publish(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();

        bus.publish(CampaignEvent::CampaignStarted {
            case_id: "test-123".to_string(),
            campaign_name: "Test Campaign".to_string(),
            config_hash: "abc123".to_string(),
        });

        let event = rx.recv().await.unwrap();
        match event {
            CampaignEvent::CampaignStarted { case_id, .. } => {
                assert_eq!(case_id, "test-123");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = EventBus::new(16);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        bus.publish(CampaignEvent::CampaignStarted {
            case_id: "test-456".to_string(),
            campaign_name: "Test".to_string(),
            config_hash: "xyz".to_string(),
        });

        // Both subscribers should receive the event
        let event1 = rx1.recv().await.unwrap();
        let event2 = rx2.recv().await.unwrap();

        match (event1, event2) {
            (CampaignEvent::CampaignStarted { case_id: id1, .. },
             CampaignEvent::CampaignStarted { case_id: id2, .. }) => {
                assert_eq!(id1, id2);
            }
            _ => panic!("Wrong event types"),
        }
    }

    #[tokio::test]
    async fn test_subscriber_count() {
        let bus = EventBus::new(16);
        assert_eq!(bus.subscriber_count(), 0);

        let _rx1 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 1);

        let _rx2 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 2);

        drop(_rx1);
        assert_eq!(bus.subscriber_count(), 1);
    }

    #[tokio::test]
    async fn test_command_clone() {
        let cmd = CampaignCommand::Pause { reason: "testing".to_string() };
        let cmd_clone = cmd.clone();
        
        assert_eq!(cmd.name(), cmd_clone.name());
    }
}
