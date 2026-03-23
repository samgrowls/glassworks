//! Campaign execution engine.
//!
//! This module provides the main campaign executor that:
//! - Schedules waves based on dependencies (DAG execution)
//! - Manages parallel wave execution
//! - Handles command interrupts (pause, resume, cancel, skip)
//! - Coordinates checkpoint/resume

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use crate::campaign::command_channel::{CommandChannel, CommandResponse};
use crate::campaign::config::CampaignConfig;
use crate::campaign::event_bus::{EventBus, CampaignEvent, CampaignCommand, EventBusExt};
use crate::campaign::state_manager::StateManager;
use crate::campaign::wave::{WaveExecutor, WaveResult};
use crate::campaign::types::{CampaignStatus, WaveStatus};
use crate::campaign::checkpoint::CheckpointManager;

/// Campaign execution result.
#[derive(Debug, Clone)]
pub struct CampaignResult {
    pub case_id: String,
    pub campaign_name: String,
    pub status: CampaignStatus,
    pub total_scanned: usize,
    pub total_flagged: usize,
    pub total_malicious: usize,
    pub duration: Duration,
    pub wave_results: Vec<WaveResult>,
}

/// Campaign executor for running complete campaigns.
pub struct CampaignExecutor {
    config: CampaignConfig,
    state: StateManager,
    event_bus: EventBus,
    command_channel: CommandChannel,
    concurrency: usize,
    checkpoint_mgr: Option<CheckpointManager>,
    case_id: String,
    skip_waves: HashSet<String>,
}

impl CampaignExecutor {
    /// Create a new campaign executor.
    ///
    /// # Arguments
    /// * `config` - Campaign configuration
    /// * `state` - State manager for tracking progress
    /// * `event_bus` - Event bus for publishing events
    /// * `command_channel` - Command channel for receiving steering commands
    pub async fn new(
        config: CampaignConfig,
        state: StateManager,
        event_bus: EventBus,
        command_channel: CommandChannel,
    ) -> Self {
        let concurrency = config.settings.concurrency;
        let case_id = state.case_id().await;
        let checkpoint_db = std::path::Path::new(".glassware-checkpoints.db");
        let checkpoint_mgr = CheckpointManager::new(checkpoint_db).ok();

        Self {
            config,
            state,
            event_bus,
            command_channel,
            concurrency,
            checkpoint_mgr,
            case_id,
            skip_waves: HashSet::new(),
        }
    }

    /// Create a campaign executor for resuming from checkpoint.
    ///
    /// # Arguments
    /// * `config` - Campaign configuration
    /// * `state` - State manager for tracking progress
    /// * `event_bus` - Event bus for publishing events
    /// * `command_channel` - Command channel for receiving steering commands
    /// * `skip_waves` - List of wave IDs to skip (already completed)
    pub async fn with_skip_waves(
        config: CampaignConfig,
        state: StateManager,
        event_bus: EventBus,
        command_channel: CommandChannel,
        skip_waves: Vec<String>,
    ) -> Self {
        let concurrency = config.settings.concurrency;
        let case_id = state.case_id().await;
        let checkpoint_db = std::path::Path::new(".glassware-checkpoints.db");
        let checkpoint_mgr = CheckpointManager::new(checkpoint_db).ok();

        Self {
            config,
            state,
            event_bus,
            command_channel,
            concurrency,
            checkpoint_mgr,
            case_id,
            skip_waves: skip_waves.into_iter().collect(),
        }
    }

    /// Resume a campaign from checkpoint.
    ///
    /// # Arguments
    /// * `case_id` - Campaign case ID to resume
    /// * `checkpoint_mgr` - Checkpoint manager for loading checkpoint
    ///
    /// # Returns
    /// * `Ok(CampaignResult)` - Campaign completed successfully
    /// * `Err(CampaignError)` - Campaign failed
    pub async fn resume(
        case_id: &str,
        checkpoint_mgr: &CheckpointManager,
    ) -> Result<CampaignResult, CampaignError> {
        use crate::campaign::config::CampaignConfig;
        use std::path::Path;

        // Load checkpoint
        let checkpoint = checkpoint_mgr.load_checkpoint(case_id)
            .map_err(|e| CampaignError::ConfigError(format!("Failed to load checkpoint: {}", e)))?
            .ok_or_else(|| CampaignError::ConfigError(format!("Campaign not found: {}", case_id)))?;

        info!("Resuming campaign '{}' from checkpoint", case_id);
        info!("Completed waves: {:?}", checkpoint.completed_waves);

        // Reload config
        let config: CampaignConfig = serde_json::from_str(&checkpoint.config_json)
            .map_err(|e| CampaignError::ConfigError(format!("Failed to parse campaign config: {}", e)))?;

        // Create campaign components
        let event_bus = EventBus::new(512);
        let state = StateManager::new(&checkpoint.case_id, &checkpoint.campaign_name, event_bus.clone());
        let command_channel = CommandChannel::new();

        // Create executor with skip list
        let executor = Self::with_skip_waves(
            config,
            state,
            event_bus.clone(),
            command_channel,
            checkpoint.completed_waves.clone(),
        ).await;

        // Run campaign (will skip completed waves)
        info!("🚀 Resuming campaign execution...");
        executor.run().await
    }

    /// Run the campaign.
    pub async fn run(&self) -> Result<CampaignResult, CampaignError> {
        let start_time = Instant::now();
        let campaign_name = self.state.campaign_name().await;

        // Log skipped waves (from checkpoint)
        if !self.skip_waves.is_empty() {
            info!("Skipping {} completed wave(s): {:?}", self.skip_waves.len(), self.skip_waves);
            for wave_id in &self.skip_waves {
                info!("  Skipping wave {} (completed)", wave_id);
            }
        }

        info!("🚀 Starting campaign '{}' (case: {})", campaign_name, self.case_id);

        // Publish campaign started event
        self.event_bus.publish_lifecycle(CampaignEvent::CampaignStarted {
            case_id: self.case_id.clone(),
            campaign_name: campaign_name.clone(),
            config_hash: self.state.snapshot().await.config_hash,
        });

        // Update state
        self.state.set_status(CampaignStatus::Running).await;

        // Build execution plan (DAG scheduling)
        let stages = self.build_execution_plan();
        let total_stages = stages.len();

        info!("Campaign has {} execution stages", total_stages);

        // Execute stages
        let mut all_wave_results = Vec::new();
        let mut current_stage = 0;

        for (stage_index, stage) in stages.iter().enumerate() {
            current_stage = stage_index;

            // Filter out skipped waves from this stage
            let active_wave_ids: Vec<_> = stage.wave_ids
                .iter()
                .filter(|wave_id| !self.skip_waves.contains(*wave_id))
                .collect();

            if active_wave_ids.is_empty() {
                info!("Stage {}: All waves already completed, skipping", stage_index + 1);
                continue;
            }

            info!("Executing stage {}/{} with {} waves ({} skipped)", 
                stage_index + 1, total_stages, active_wave_ids.len(), 
                stage.wave_ids.len() - active_wave_ids.len());

            // Check for commands before starting stage
            if let Some(result) = self.check_commands().await {
                match result {
                    CommandAction::Pause => {
                        info!("Campaign paused at stage {}", stage_index);
                        self.wait_for_resume().await?;
                    }
                    CommandAction::Cancel => {
                        info!("Campaign cancelled at stage {}", stage_index);
                        return self.cancel_campaign(start_time, all_wave_results).await;
                    }
                    CommandAction::SkipStage => {
                        info!("Skipping stage {}", stage_index);
                        continue;
                    }
                    CommandAction::Continue => {}
                }
            }

            info!("Starting stage {} execution (parallel={})", stage_index + 1, stage.parallel);

            // Execute waves in this stage (parallel or sequential)
            let stage_results = if stage.parallel {
                info!("Executing {} waves in parallel", active_wave_ids.len());
                self.execute_stage_parallel(&active_wave_ids).await
            } else {
                info!("Executing {} waves sequentially", active_wave_ids.len());
                self.execute_stage_sequential(&active_wave_ids).await
            };

            info!("Stage {} complete", stage_index + 1);

            // Collect results
            for result in stage_results {
                match result {
                    Ok(wave_result) => {
                        all_wave_results.push(wave_result);
                    }
                    Err(e) => {
                        error!("Wave failed: {}", e);
                        // Continue with other waves
                    }
                }
            }
        }

        // Calculate final statistics
        let total_scanned = all_wave_results.iter().map(|r| r.packages_scanned).sum();
        let total_flagged = all_wave_results.iter().map(|r| r.packages_flagged).sum();
        let total_malicious = all_wave_results.iter().map(|r| r.packages_malicious).sum();
        let duration = start_time.elapsed();

        // Mark campaign as completed
        self.state.complete_campaign(total_scanned, total_flagged, total_malicious, duration.as_secs()).await;

        info!(
            "✅ Campaign completed: {} scanned, {} flagged, {} malicious in {:?}",
            total_scanned, total_flagged, total_malicious, duration
        );

        Ok(CampaignResult {
            case_id: self.case_id.clone(),
            campaign_name,
            status: CampaignStatus::Completed,
            total_scanned,
            total_flagged,
            total_malicious,
            duration,
            wave_results: all_wave_results,
        })
    }

    /// Build execution plan from wave dependencies.
    ///
    /// Returns stages where waves in each stage can run in parallel.
    fn build_execution_plan(&self) -> Vec<ExecutionStage> {
        let mut stages = Vec::new();
        let mut completed = HashSet::new();
        let mut remaining: HashSet<_> = self.config.waves.iter().map(|w| w.id.clone()).collect();

        while !remaining.is_empty() {
            // Find all waves that can run now (all dependencies met)
            let ready: Vec<_> = remaining.iter()
                .filter(|wave_id| {
                    let wave = self.config.get_wave(wave_id).unwrap();
                    wave.depends_on.iter().all(|dep| completed.contains(dep))
                })
                .cloned()
                .collect();

            if ready.is_empty() {
                error!("Circular dependency detected in campaign!");
                break;
            }

            stages.push(ExecutionStage {
                wave_ids: ready.clone(),
                parallel: true, // Waves in same stage have no dependencies on each other
            });

            for wave_id in &ready {
                completed.insert(wave_id.clone());
                remaining.remove(wave_id);
            }
        }

        stages
    }

    /// Execute a stage with waves running in parallel.
    async fn execute_stage_parallel(&self, wave_ids: &[&String]) -> Vec<Result<WaveResult, CampaignError>> {
        info!("Starting parallel execution of {} waves", wave_ids.len());

        let mut handles = Vec::new();

        for wave_id in wave_ids {
            info!("Spawning wave {} in parallel", wave_id);

            let config = self.config.get_wave(wave_id).unwrap().clone();
            let state = self.state.clone();
            let event_bus = self.event_bus.clone();
            let concurrency = self.concurrency;
            let wave_id_clone = (*wave_id).clone();

            let handle = tokio::spawn(async move {
                let wave_id_for_log = config.id.clone();
                info!("Wave {} executor starting...", wave_id_for_log);
                let executor = WaveExecutor::new(config, state, event_bus, concurrency);
                info!("Wave {} executing...", wave_id_for_log);
                executor.execute().await.map_err(CampaignError::WaveError)
            });

            handles.push((wave_id_clone, handle));
        }

        info!("Waiting for {} parallel waves to complete", handles.len());

        // Wait for all waves in stage to complete
        let mut results = Vec::new();
        for (wave_id, handle) in handles.into_iter() {
            info!("Waiting for wave {} handle...", wave_id);
            match handle.await {
                Ok(result) => {
                    info!("Wave {} handle complete", wave_id);
                    // Save checkpoint after wave completes
                    if let Some(ref mgr) = self.checkpoint_mgr {
                        if let Err(e) = mgr.add_completed_wave(&self.case_id, &wave_id) {
                            warn!("Failed to save checkpoint for wave {}: {}", wave_id, e);
                        } else {
                            debug!("Saved checkpoint for completed wave: {}", wave_id);
                        }
                    }
                    results.push(result);
                }
                Err(e) => {
                    error!("Wave {} handle failed: {}", wave_id, e);
                    results.push(Err(CampaignError::TaskError(e.to_string())));
                }
            }
        }

        info!("All parallel waves complete");
        results
    }

    /// Execute a stage with waves running sequentially.
    async fn execute_stage_sequential(&self, wave_ids: &[&String]) -> Vec<Result<WaveResult, CampaignError>> {
        let mut results = Vec::new();

        for wave_id in wave_ids {
            // Check for commands between waves
            if let Some(result) = self.check_commands().await {
                match result {
                    CommandAction::Pause => {
                        self.wait_for_resume().await.ok();
                    }
                    CommandAction::Cancel => {
                        results.push(Err(CampaignError::Cancelled));
                        break;
                    }
                    CommandAction::SkipStage => {
                        continue;
                    }
                    CommandAction::Continue => {}
                }
            }

            let config = self.config.get_wave(wave_id).unwrap().clone();
            let executor = WaveExecutor::new(
                config,
                self.state.clone(),
                self.event_bus.clone(),
                self.concurrency,
            );

            let result = executor.execute().await.map_err(CampaignError::WaveError);
            
            // Save checkpoint after wave completes (if successful)
            if result.is_ok() {
                if let Some(ref mgr) = self.checkpoint_mgr {
                    if let Err(e) = mgr.add_completed_wave(&self.case_id, wave_id) {
                        warn!("Failed to save checkpoint for wave {}: {}", wave_id, e);
                    } else {
                        debug!("Saved checkpoint for completed wave: {}", wave_id);
                    }
                }
            }
            
            results.push(result);
        }

        results
    }

    /// Check for pending commands.
    async fn check_commands(&self) -> Option<CommandAction> {
        debug!("Checking for pending commands...");
        
        // Use try_recv to avoid blocking when no commands pending
        if let Some((command, response_tx)) = self.command_channel.try_recv() {
            debug!("Received command: {:?}", command);
            let action = self.handle_command(&command).await;
            
            // Send response
            let response = match &action {
                CommandAction::Continue => CommandResponse::accepted(command),
                CommandAction::Pause => CommandResponse::accepted(command),
                CommandAction::Cancel => CommandResponse::accepted(command),
                CommandAction::SkipStage => CommandResponse::completed(command, "Stage skipped"),
            };
            
            let _ = response_tx.send(response).await;
            
            return Some(action);
        }
        
        debug!("No pending commands");
        None
    }

    /// Handle a command.
    async fn handle_command(&self, command: &CampaignCommand) -> CommandAction {
        match command {
            CampaignCommand::Pause { reason } => {
                info!("Pause command received: {}", reason);
                self.state.set_status(CampaignStatus::Paused).await;
                self.event_bus.publish_lifecycle(CampaignEvent::CampaignPaused {
                    case_id: self.state.case_id().await,
                    reason: reason.clone(),
                });
                CommandAction::Pause
            }
            CampaignCommand::Resume => {
                info!("Resume command received");
                CommandAction::Continue
            }
            CampaignCommand::Cancel { save_checkpoint } => {
                info!("Cancel command received (save_checkpoint={})", save_checkpoint);
                CommandAction::Cancel
            }
            CampaignCommand::SkipWave { wave_id } => {
                info!("Skip wave command received: {}", wave_id);
                CommandAction::SkipStage
            }
            CampaignCommand::RetryWave { wave_id } => {
                info!("Retry wave command received: {}", wave_id);
                // TODO: Implement retry logic
                CommandAction::Continue
            }
            CampaignCommand::SetConcurrency { concurrency } => {
                info!("Set concurrency command: {}", concurrency);
                // Concurrency is read at stage start, will apply to next stage
                CommandAction::Continue
            }
            CampaignCommand::SetRateLimit { rate_limit } => {
                info!("Set rate limit command: {}", rate_limit);
                CommandAction::Continue
            }
            CampaignCommand::ToggleLlm { enabled } => {
                info!("Toggle LLM command: {}", enabled);
                CommandAction::Continue
            }
            CampaignCommand::DumpState => {
                info!("Dump state command received");
                // TODO: Dump state to file
                CommandAction::Continue
            }
            CampaignCommand::CollectMetrics => {
                info!("Collect metrics command received");
                CommandAction::Continue
            }
        }
    }

    /// Wait for resume command.
    async fn wait_for_resume(&self) -> Result<(), CampaignError> {
        loop {
            sleep(Duration::from_millis(500)).await;
            
            if let Some((command, response_tx)) = self.command_channel.recv().await {
                if matches!(command, CampaignCommand::Resume) {
                    let _ = response_tx.send(CommandResponse::accepted(command)).await;
                    self.state.set_status(CampaignStatus::Running).await;
                    self.event_bus.publish_lifecycle(CampaignEvent::CampaignResumed {
                        case_id: self.state.case_id().await,
                    });
                    return Ok(());
                } else if matches!(command, CampaignCommand::Cancel { .. }) {
                    let _ = response_tx.send(CommandResponse::accepted(command)).await;
                    return Err(CampaignError::Cancelled);
                } else {
                    // Handle other commands
                    let action = self.handle_command(&command).await;
                    let response = CommandResponse::accepted(command);
                    let _ = response_tx.send(response).await;
                    
                    if matches!(action, CommandAction::Cancel) {
                        return Err(CampaignError::Cancelled);
                    }
                }
            }
        }
    }

    /// Cancel the campaign.
    async fn cancel_campaign(
        &self,
        start_time: Instant,
        wave_results: Vec<WaveResult>,
    ) -> Result<CampaignResult, CampaignError> {
        self.state.cancel_campaign().await;

        let total_scanned = wave_results.iter().map(|r| r.packages_scanned).sum();
        let total_flagged = wave_results.iter().map(|r| r.packages_flagged).sum();
        let total_malicious = wave_results.iter().map(|r| r.packages_malicious).sum();
        let duration = start_time.elapsed();

        Ok(CampaignResult {
            case_id: self.case_id.clone(),
            campaign_name: self.state.campaign_name().await,
            status: CampaignStatus::Cancelled,
            total_scanned,
            total_flagged,
            total_malicious,
            duration,
            wave_results,
        })
    }
}

/// Execution stage for DAG scheduling.
#[derive(Debug, Clone)]
struct ExecutionStage {
    /// Waves that can run in this stage.
    wave_ids: Vec<String>,
    /// Whether waves in this stage can run in parallel.
    parallel: bool,
}

/// Command action result.
#[derive(Debug, Clone)]
enum CommandAction {
    Continue,
    Pause,
    Cancel,
    SkipStage,
}

/// Campaign execution errors.
#[derive(Debug, thiserror::Error)]
pub enum CampaignError {
    #[error("Wave error: {0}")]
    WaveError(#[from] crate::campaign::wave::WaveError),

    #[error("Task error: {0}")]
    TaskError(String),

    #[error("Campaign cancelled")]
    Cancelled,

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::campaign::config::{CampaignMetadata, CampaignSettings, WaveConfig, WaveSource};
    use crate::campaign::types::WaveMode;
    use crate::campaign::command_channel::CommandChannel;

    fn create_test_campaign() -> CampaignConfig {
        CampaignConfig {
            campaign: CampaignMetadata {
                name: "Test Campaign".to_string(),
                description: String::new(),
                created_by: "test".to_string(),
                priority: Default::default(),
                tags: Vec::new(),
            },
            settings: CampaignSettings::default(),
            waves: vec![
                WaveConfig {
                    id: "wave1".to_string(),
                    name: "Wave 1".to_string(),
                    description: String::new(),
                    depends_on: Vec::new(),
                    mode: WaveMode::Hunt,
                    sources: vec![WaveSource::Packages { list: vec!["express".to_string()] }],
                    whitelist: Vec::new(),
                    expectations: None,
                    reporting: None,
                },
                WaveConfig {
                    id: "wave2".to_string(),
                    name: "Wave 2".to_string(),
                    description: String::new(),
                    depends_on: vec!["wave1".to_string()],
                    mode: WaveMode::Hunt,
                    sources: vec![WaveSource::Packages { list: vec!["lodash".to_string()] }],
                    whitelist: Vec::new(),
                    expectations: None,
                    reporting: None,
                },
            ],
        }
    }

    #[tokio::test]
    async fn test_build_execution_plan() {
        let config = create_test_campaign();
        let event_bus = EventBus::new(16);
        let state = StateManager::new("test", "Test", event_bus.clone());
        let commands = CommandChannel::new();

        let executor = CampaignExecutor::new(config, state, event_bus, commands).await;
        let stages = executor.build_execution_plan();

        assert_eq!(stages.len(), 2);
        assert_eq!(stages[0].wave_ids, vec!["wave1"]);
        assert_eq!(stages[1].wave_ids, vec!["wave2"]);
        assert!(stages[0].parallel);
        assert!(stages[1].parallel);
    }

    #[tokio::test]
    async fn test_build_execution_plan_parallel() {
        // Two waves with no dependencies - should run in parallel
        let config = CampaignConfig {
            campaign: CampaignMetadata {
                name: "Test".to_string(),
                description: String::new(),
                created_by: "test".to_string(),
                priority: Default::default(),
                tags: Vec::new(),
            },
            settings: CampaignSettings::default(),
            waves: vec![
                WaveConfig {
                    id: "wave1".to_string(),
                    name: "Wave 1".to_string(),
                    description: String::new(),
                    depends_on: Vec::new(),
                    mode: WaveMode::Hunt,
                    sources: vec![WaveSource::Packages { list: vec!["a".to_string()] }],
                    whitelist: Vec::new(),
                    expectations: None,
                    reporting: None,
                },
                WaveConfig {
                    id: "wave2".to_string(),
                    name: "Wave 2".to_string(),
                    description: String::new(),
                    depends_on: Vec::new(),
                    mode: WaveMode::Hunt,
                    sources: vec![WaveSource::Packages { list: vec!["b".to_string()] }],
                    whitelist: Vec::new(),
                    expectations: None,
                    reporting: None,
                },
            ],
        };

        let event_bus = EventBus::new(16);
        let state = StateManager::new("test", "Test", event_bus.clone());
        let commands = CommandChannel::new();

        let executor = CampaignExecutor::new(config, state, event_bus, commands).await;
        let stages = executor.build_execution_plan();

        assert_eq!(stages.len(), 1); // Both waves in same stage
        assert_eq!(stages[0].wave_ids.len(), 2);
        assert!(stages[0].parallel);
    }
}
