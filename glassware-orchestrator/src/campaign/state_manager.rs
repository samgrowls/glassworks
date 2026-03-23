//! State manager for queryable campaign state.
//!
//! Provides thread-safe access to campaign state with
//! automatic event publishing on state changes.

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error};

use crate::campaign::types::{CampaignState, CampaignStatus, WaveState, ActivePackage, CampaignStats, WaveStatus};
use crate::campaign::event_bus::{EventBus, CampaignEvent, CampaignCommand, EventBusExt};
use crate::campaign::command_channel::{CommandResponse, CommandSender};

/// Thread-safe state manager with event publishing.
///
/// The state manager provides:
/// - Atomic state updates via RwLock
/// - Automatic event publishing on changes
/// - Recent event history for quick access
/// - Clone handles for multiple consumers
#[derive(Clone)]
pub struct StateManager {
    state: Arc<RwLock<CampaignState>>,
    event_bus: EventBus,
    recent_events: Arc<RwLock<VecDeque<CampaignEvent>>>,
}

impl StateManager {
    /// Create a new state manager.
    ///
    /// # Arguments
    /// * `case_id` - Unique identifier for this campaign run
    /// * `campaign_name` - Human-readable campaign name
    /// * `event_bus` - Event bus for publishing state change events
    pub fn new(case_id: impl Into<String>, campaign_name: impl Into<String>, event_bus: EventBus) -> Self {
        let state = CampaignState::new(case_id, campaign_name);
        
        Self {
            state: Arc::new(RwLock::new(state)),
            event_bus,
            recent_events: Arc::new(RwLock::new(VecDeque::with_capacity(100))),
        }
    }

    /// Get current state snapshot (for TUI/CLI rendering).
    ///
    /// This acquires a read lock, clones the state, and releases the lock.
    /// The returned snapshot may be slightly stale but won't block writers.
    pub async fn snapshot(&self) -> CampaignState {
        self.state.read().await.clone()
    }

    /// Get a specific field from state without cloning the entire state.
    pub async fn with_status<F, R>(&self, f: F) -> R
    where
        F: FnOnce(CampaignStatus) -> R,
    {
        let state = self.state.read().await;
        f(state.status)
    }

    /// Update state with a closure and publish an event.
    ///
    /// The update function receives a mutable reference to the state
    /// and can modify it atomically. After the update completes,
    /// the event is published to all subscribers.
    ///
    /// # Note
    /// The state lock is released before publishing the event to avoid
    /// blocking other readers during event distribution.
    pub async fn update<F>(&self, update_fn: F, event: CampaignEvent)
    where
        F: FnOnce(&mut CampaignState),
    {
        // Update state under lock
        {
            let mut state = self.state.write().await;
            update_fn(&mut state);
        }  // Release lock before publishing

        // Publish event
        self.event_bus.publish_lifecycle(event.clone());

        // Store in recent events
        self.add_recent_event(event).await;
    }

    /// Update state without publishing an event.
    ///
    /// Use this for internal state updates that don't need to be
    /// broadcast to subscribers (e.g., recalculating derived stats).
    pub async fn update_silent<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut CampaignState),
    {
        let mut state = self.state.write().await;
        update_fn(&mut state);
    }

    /// Update state and publish a wave-specific event.
    pub async fn update_wave<F>(&self, update_fn: F, event: CampaignEvent)
    where
        F: FnOnce(&mut CampaignState),
    {
        {
            let mut state = self.state.write().await;
            update_fn(&mut state);
        }
        
        self.event_bus.publish_wave(event.clone());
        self.add_recent_event(event).await;
    }

    /// Update state and publish a package-specific event.
    pub async fn update_package<F>(&self, update_fn: F, event: CampaignEvent)
    where
        F: FnOnce(&mut CampaignState),
    {
        {
            let mut state = self.state.write().await;
            update_fn(&mut state);
        }
        
        self.event_bus.publish_package(event.clone());
        self.add_recent_event(event).await;
    }

    /// Add an event to the recent events history.
    async fn add_recent_event(&self, event: CampaignEvent) {
        let mut events = self.recent_events.write().await;
        events.push_back(event);
        if events.len() > 100 {
            events.pop_front();
        }
    }

    /// Get recent events (for CLI --live mode).
    pub async fn recent_events(&self) -> Vec<CampaignEvent> {
        self.recent_events.read().await.iter().cloned().collect()
    }

    /// Clear recent events history.
    pub async fn clear_recent_events(&self) {
        self.recent_events.write().await.clear();
    }

    // === Convenience methods for common state updates ===

    /// Set campaign status.
    pub async fn set_status(&self, status: CampaignStatus) {
        self.update_silent(|state| {
            state.status = status;
        }).await;
    }

    /// Set campaign status and publish event.
    pub async fn set_status_with_event(&self, status: CampaignStatus, event: CampaignEvent) {
        self.update(|state| {
            state.status = status;
        }, event).await;
    }

    /// Set current wave being executed.
    pub async fn set_current_wave(&self, wave_id: Option<String>) {
        self.update_silent(|state| {
            state.current_wave = wave_id;
        }).await;
    }

    /// Add a new wave to the campaign state.
    pub async fn add_wave(&self, wave: WaveState) {
        let wave_id = wave.id.clone();
        self.update_silent(|state| {
            state.waves.insert(wave_id.clone(), wave);
        }).await;
    }

    /// Update wave status.
    pub async fn update_wave_status(&self, wave_id: &str, status: WaveStatus) {
        self.update_silent(|state| {
            if let Some(wave) = state.waves.get_mut(wave_id) {
                let old_status = wave.status;
                wave.status = status;
                
                debug!("Wave '{}' status: {:?} -> {:?}", wave_id, old_status, status);
            }
        }).await;
    }

    /// Update wave progress counters.
    pub async fn update_wave_progress(&self, wave_id: &str, scanned: usize, flagged: usize, malicious: usize) {
        self.update_silent(|state| {
            if let Some(wave) = state.waves.get_mut(wave_id) {
                wave.packages_scanned = scanned;
                wave.packages_flagged = flagged;
                wave.packages_malicious = malicious;
            }
            // Recalculate aggregate stats
            state.recalculate_stats();
        }).await;
    }

    /// Mark wave as started.
    pub async fn start_wave(&self, wave_id: &str, packages_total: usize) {
        let wave_name = self.state.read().await
            .waves.get(wave_id)
            .map(|w| w.name.clone())
            .unwrap_or_else(|| wave_id.to_string());

        self.update_wave(|state| {
            if let Some(wave) = state.waves.get_mut(wave_id) {
                wave.start();
                wave.packages_total = packages_total;
            }
            state.current_wave = Some(wave_id.to_string());
            state.recalculate_stats();
        }, CampaignEvent::WaveStarted {
            wave_id: wave_id.to_string(),
            name: wave_name,
            packages_total,
        }).await;
    }

    /// Mark wave as completed.
    pub async fn complete_wave(&self, wave_id: &str) {
        let (packages_scanned, packages_flagged, packages_malicious) = self.state.read().await
            .waves.get(wave_id)
            .map(|w| (w.packages_scanned, w.packages_flagged, w.packages_malicious))
            .unwrap_or((0, 0, 0));

        self.update_wave(|state| {
            if let Some(wave) = state.waves.get_mut(wave_id) {
                wave.complete();
            }
            state.current_wave = None;
            state.recalculate_stats();
        }, CampaignEvent::WaveCompleted {
            wave_id: wave_id.to_string(),
            packages_scanned,
            packages_flagged,
            packages_malicious,
            duration_seconds: 0,  // Would need to track start time
        }).await;
    }

    /// Mark wave as failed.
    pub async fn fail_wave(&self, wave_id: &str, error: String) {
        let error_event = error.clone();
        self.update_wave(|state| {
            if let Some(wave) = state.waves.get_mut(wave_id) {
                wave.fail(error);
            }
            state.current_wave = None;
        }, CampaignEvent::WaveFailed {
            wave_id: wave_id.to_string(),
            error: error_event,
        }).await;
    }

    /// Set the active package being processed.
    pub async fn set_active_package(&self, package: ActivePackage) {
        self.update_silent(|state| {
            state.active_package = Some(package);
        }).await;
    }

    /// Clear the active package (processing complete).
    pub async fn clear_active_package(&self) {
        self.update_silent(|state| {
            state.active_package = None;
        }).await;
    }

    /// Update campaign statistics.
    pub async fn update_stats(&self, f: impl FnOnce(&mut CampaignStats)) {
        self.update_silent(|state| {
            f(&mut state.stats);
        }).await;
    }

    /// Increment the LLM analyses counter.
    pub async fn increment_llm_analyses(&self) {
        self.update_silent(|state| {
            state.stats.llm_analyses_run += 1;
        }).await;
    }

    /// Increment the evidence collected counter.
    pub async fn increment_evidence(&self) {
        self.update_silent(|state| {
            state.stats.evidence_collected += 1;
        }).await;
    }

    /// Set the config hash.
    pub async fn set_config_hash(&self, hash: String) {
        self.update_silent(|state| {
            state.config_hash = hash;
        }).await;
    }

    /// Mark campaign as completed.
    pub async fn complete_campaign(&self, total_scanned: usize, total_flagged: usize, total_malicious: usize, duration_seconds: u64) {
        let case_id = self.state.read().await.case_id.clone();
        
        self.update(|state| {
            state.status = CampaignStatus::Completed;
            state.completed_at = Some(chrono::Utc::now());
        }, CampaignEvent::CampaignCompleted {
            case_id,
            total_scanned,
            total_flagged,
            total_malicious,
            duration_seconds,
        }).await;
    }

    /// Mark campaign as failed.
    pub async fn fail_campaign(&self, error: String) {
        let case_id = self.state.read().await.case_id.clone();
        
        self.update(|state| {
            state.status = CampaignStatus::Failed;
            state.completed_at = Some(chrono::Utc::now());
        }, CampaignEvent::CampaignFailed {
            case_id,
            error,
        }).await;
    }

    /// Mark campaign as cancelled.
    pub async fn cancel_campaign(&self) {
        let case_id = self.state.read().await.case_id.clone();
        
        self.update(|state| {
            state.status = CampaignStatus::Cancelled;
            state.completed_at = Some(chrono::Utc::now());
        }, CampaignEvent::CampaignCancelled {
            case_id,
        }).await;
    }

    /// Check if campaign can accept commands.
    pub async fn accepts_commands(&self) -> bool {
        let status = self.with_status(|s| s).await;
        status.accepts_commands()
    }

    /// Get the current case ID.
    pub async fn case_id(&self) -> String {
        self.state.read().await.case_id.clone()
    }

    /// Get the current campaign name.
    pub async fn campaign_name(&self) -> String {
        self.state.read().await.campaign_name.clone()
    }
}

/// Handle for sending commands to a running campaign.
pub struct CommandHandle {
    command_tx: tokio::sync::mpsc::Sender<crate::campaign::command_channel::CommandMessage>,
}

impl CommandHandle {
    /// Create a new command handle.
    pub fn new(
        command_tx: tokio::sync::mpsc::Sender<crate::campaign::command_channel::CommandMessage>,
    ) -> Self {
        Self { command_tx }
    }

    /// Send a command and wait for response.
    pub async fn send(&self, command: CampaignCommand) -> CommandResponse {
        let (response_tx, mut response_rx) = tokio::sync::mpsc::channel(1);
        
        if self.command_tx.send((command.clone(), response_tx)).await.is_err() {
            return CommandResponse::rejected(
                command.clone(),
                "Command channel closed".to_string(),
            );
        }

        response_rx.recv().await.unwrap_or(CommandResponse::rejected(
            command,
            "No response received".to_string(),
        ))
    }

    /// Send a pause command.
    pub async fn pause(&self, reason: impl Into<String>) -> CommandResponse {
        self.send(CampaignCommand::Pause { reason: reason.into() }).await
    }

    /// Send a resume command.
    pub async fn resume(&self) -> CommandResponse {
        self.send(CampaignCommand::Resume).await
    }

    /// Send a cancel command.
    pub async fn cancel(&self, save_checkpoint: bool) -> CommandResponse {
        self.send(CampaignCommand::Cancel { save_checkpoint }).await
    }

    /// Send a skip wave command.
    pub async fn skip_wave(&self, wave_id: impl Into<String>) -> CommandResponse {
        self.send(CampaignCommand::SkipWave { wave_id: wave_id.into() }).await
    }

    /// Send a set concurrency command.
    pub async fn set_concurrency(&self, concurrency: usize) -> CommandResponse {
        self.send(CampaignCommand::SetConcurrency { concurrency }).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_state_manager_snapshot() {
        let event_bus = EventBus::new(16);
        let manager = StateManager::new("test-123", "Test Campaign", event_bus);

        let state = manager.snapshot().await;
        assert_eq!(state.case_id, "test-123");
        assert_eq!(state.campaign_name, "Test Campaign");
        assert_eq!(state.status, CampaignStatus::Initializing);
    }

    #[tokio::test]
    async fn test_state_manager_update() {
        let event_bus = EventBus::new(16);
        let manager = StateManager::new("test-456", "Test", event_bus.clone());

        let mut rx = event_bus.subscribe();

        manager.update(
            |state| {
                state.status = CampaignStatus::Running;
            },
            CampaignEvent::CampaignStarted {
                case_id: "test-456".to_string(),
                campaign_name: "Test".to_string(),
                config_hash: "abc".to_string(),
            },
        ).await;

        // Verify state updated
        let state = manager.snapshot().await;
        assert_eq!(state.status, CampaignStatus::Running);

        // Verify event published
        let event = rx.recv().await.unwrap();
        match event {
            CampaignEvent::CampaignStarted { case_id, .. } => {
                assert_eq!(case_id, "test-456");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[tokio::test]
    async fn test_state_manager_wave_operations() {
        let event_bus = EventBus::new(16);
        let manager = StateManager::new("test-789", "Test", event_bus);

        // Add a wave
        let wave = WaveState::new("wave1", "Wave 1", crate::campaign::types::WaveMode::Hunt);
        manager.add_wave(wave).await;

        // Start the wave
        manager.start_wave("wave1", 100).await;

        // Verify wave started
        let state = manager.snapshot().await;
        let wave = state.get_wave("wave1").unwrap();
        assert_eq!(wave.status, WaveStatus::Running);
        assert_eq!(wave.packages_total, 100);
        assert_eq!(state.current_wave, Some("wave1".to_string()));
    }

    #[tokio::test]
    async fn test_state_manager_recent_events() {
        let event_bus = EventBus::new(16);
        let manager = StateManager::new("test", "Test", event_bus);

        // Generate some events
        for i in 0..50 {
            manager.update(
                |_| {},
                CampaignEvent::WaveProgress {
                    wave_id: "wave1".to_string(),
                    scanned: i,
                    flagged: 0,
                    malicious: 0,
                },
            ).await;
        }

        let events = manager.recent_events().await;
        assert_eq!(events.len(), 50);
    }
}
