//! Command channel for steering campaign execution.
//!
//! Allows external components (TUI, CLI) to send commands
//! to the running campaign executor.

use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, info, warn};

use crate::campaign::event_bus::CampaignCommand;

/// Response to a campaign command.
#[derive(Debug, Clone)]
pub enum CommandResponse {
    /// Command accepted and will be executed.
    Accepted {
        command: CampaignCommand,
    },
    /// Command rejected (invalid for current state).
    Rejected {
        command: CampaignCommand,
        reason: String,
    },
    /// Command completed successfully.
    Completed {
        command: CampaignCommand,
        result: String,
    },
}

impl CommandResponse {
    /// Create an accepted response.
    pub fn accepted(command: CampaignCommand) -> Self {
        Self::Accepted { command }
    }

    /// Create a rejected response.
    pub fn rejected(command: CampaignCommand, reason: impl Into<String>) -> Self {
        Self::Rejected {
            command,
            reason: reason.into(),
        }
    }

    /// Create a completed response.
    pub fn completed(command: CampaignCommand, result: impl Into<String>) -> Self {
        Self::Completed {
            command,
            result: result.into(),
        }
    }

    /// Check if the command was accepted.
    pub fn is_accepted(&self) -> bool {
        matches!(self, CommandResponse::Accepted { .. } | CommandResponse::Completed { .. })
    }

    /// Check if the command was rejected.
    pub fn is_rejected(&self) -> bool {
        matches!(self, CommandResponse::Rejected { .. })
    }

    /// Get the rejection reason if rejected.
    pub fn rejection_reason(&self) -> Option<&str> {
        match self {
            CommandResponse::Rejected { reason, .. } => Some(reason),
            _ => None,
        }
    }
}

pub type CommandMessage = (CampaignCommand, mpsc::Sender<CommandResponse>);

/// Command channel for steering campaigns.
///
/// # Architecture
///
/// ```text
/// ┌─────────────┐     ┌─────────────┐     ┌──────────────┐
/// │    TUI      │     │    CLI      │     │   Other     │
/// │  (ratatui)  │     │  (commands) │     │  Components │
/// └──────┬──────┘     └──────┬──────┘     └──────┬───────┘
///        │                   │                    │
///        └───────────────────┼────────────────────┘
///                            │
///                   ┌────────▼────────┐
///                   │  CommandSender  │
///                   │  (cloneable)    │
///                   └────────┬────────┘
///                            │
///                   ┌────────▼────────┐
///                   │ CommandChannel  │
///                   │  (mpsc queue)   │
///                   └────────┬────────┘
///                            │
///                   ┌────────▼────────┐
///                   │ CampaignExecutor│
///                   │  (polls cmds)   │
///                   └─────────────────┘
/// ```
pub struct CommandChannel {
    sender: mpsc::Sender<CommandMessage>,
    receiver: Arc<Mutex<mpsc::Receiver<CommandMessage>>>,
}

impl CommandChannel {
    /// Create a new command channel.
    ///
    /// The channel has a buffer size of 100 commands. If the executor
    /// doesn't process commands fast enough, senders will block when
    /// the buffer is full.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            sender: tx,
            receiver: Arc::new(Mutex::new(rx)),
        }
    }

    /// Create a new command channel with custom buffer size.
    pub fn with_capacity(capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(capacity);
        Self {
            sender: tx,
            receiver: Arc::new(Mutex::new(rx)),
        }
    }

    /// Get the sender handle (for TUI/CLI to send commands).
    ///
    /// The sender can be cloned and passed to multiple components.
    pub fn sender(&self) -> mpsc::Sender<CommandMessage> {
        self.sender.clone()
    }

    /// Create a command sender handle for external use.
    pub fn create_sender(&self) -> CommandSender {
        CommandSender::new(self.sender())
    }

    /// Receive a command (called by executor).
    ///
    /// Returns `None` if the channel is closed (all senders dropped).
    pub async fn recv(&self) -> Option<CommandMessage> {
        self.receiver.lock().await.recv().await
    }

    /// Try to receive a command without waiting.
    ///
    /// Returns `None` if no command is available or if the lock is contested.
    pub fn try_recv(&self) -> Option<CommandMessage> {
        if let Ok(mut guard) = self.receiver.try_lock() {
            guard.try_recv().ok()
        } else {
            None
        }
    }

    /// Check if there are pending commands.
    pub fn has_pending(&self) -> bool {
        if let Ok(guard) = self.receiver.try_lock() {
            !guard.is_empty()
        } else {
            false
        }
    }

    /// Get the number of pending commands.
    pub fn pending_count(&self) -> usize {
        if let Ok(guard) = self.receiver.try_lock() {
            guard.len()
        } else {
            0
        }
    }

    /// Send a command and wait for response (for CLI commands).
    ///
    /// This is a convenience method for sending commands from external
    /// components. The executor uses `recv()` directly.
    pub async fn send(&self, command: CampaignCommand) -> CommandResponse {
        let (response_tx, mut response_rx) = mpsc::channel(1);
        
        if self.sender.send((command.clone(), response_tx)).await.is_err() {
            return CommandResponse::rejected(
                command,
                "Command channel closed (executor may have stopped)",
            );
        }

        response_rx.recv().await.unwrap_or(CommandResponse::rejected(
            command,
            "No response received (executor may have crashed)",
        ))
    }
}

impl Default for CommandChannel {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle for sending commands to a running campaign.
///
/// This is a lightweight, cloneable handle that can be passed
/// to TUI, CLI, or other components that need to send commands.
#[derive(Clone)]
pub struct CommandSender {
    command_tx: mpsc::Sender<CommandMessage>,
}

impl CommandSender {
    /// Create a new command sender.
    pub fn new(command_tx: mpsc::Sender<CommandMessage>) -> Self {
        Self { command_tx }
    }

    /// Send a command and wait for response.
    pub async fn send(&self, command: CampaignCommand) -> CommandResponse {
        let (response_tx, mut response_rx) = mpsc::channel(1);

        if self.command_tx.send((command.clone(), response_tx)).await.is_err() {
            return CommandResponse::rejected(
                command,
                "Command channel closed",
            );
        }

        response_rx.recv().await.unwrap_or(CommandResponse::rejected(
            command,
            "No response received",
        ))
    }

    /// Send a pause command.
    pub async fn pause(&self, reason: impl Into<String>) -> CommandResponse {
        let reason = reason.into();
        debug!("Sending pause command: {}", reason);
        self.send(CampaignCommand::Pause { reason }).await
    }

    /// Send a resume command.
    pub async fn resume(&self) -> CommandResponse {
        debug!("Sending resume command");
        self.send(CampaignCommand::Resume).await
    }

    /// Send a cancel command.
    pub async fn cancel(&self, save_checkpoint: bool) -> CommandResponse {
        debug!("Sending cancel command (save_checkpoint={})", save_checkpoint);
        self.send(CampaignCommand::Cancel { save_checkpoint }).await
    }

    /// Send a skip wave command.
    pub async fn skip_wave(&self, wave_id: impl Into<String>) -> CommandResponse {
        let wave_id = wave_id.into();
        debug!("Sending skip_wave command for: {}", wave_id);
        self.send(CampaignCommand::SkipWave { wave_id }).await
    }

    /// Send a retry wave command.
    pub async fn retry_wave(&self, wave_id: impl Into<String>) -> CommandResponse {
        let wave_id = wave_id.into();
        debug!("Sending retry_wave command for: {}", wave_id);
        self.send(CampaignCommand::RetryWave { wave_id }).await
    }

    /// Send a set concurrency command.
    pub async fn set_concurrency(&self, concurrency: usize) -> CommandResponse {
        debug!("Sending set_concurrency command: {}", concurrency);
        self.send(CampaignCommand::SetConcurrency { concurrency }).await
    }

    /// Send a set rate limit command.
    pub async fn set_rate_limit(&self, rate_limit: f32) -> CommandResponse {
        debug!("Sending set_rate_limit command: {}", rate_limit);
        self.send(CampaignCommand::SetRateLimit { rate_limit }).await
    }

    /// Send a toggle LLM command.
    pub async fn toggle_llm(&self, enabled: bool) -> CommandResponse {
        debug!("Sending toggle_llm command: {}", enabled);
        self.send(CampaignCommand::ToggleLlm { enabled }).await
    }

    /// Send a dump state command.
    pub async fn dump_state(&self) -> CommandResponse {
        debug!("Sending dump_state command");
        self.send(CampaignCommand::DumpState).await
    }

    /// Send a collect metrics command.
    pub async fn collect_metrics(&self) -> CommandResponse {
        debug!("Sending collect_metrics command");
        self.send(CampaignCommand::CollectMetrics).await
    }
}

/// Validator for checking if commands are valid for the current campaign state.
pub struct CommandValidator {
    is_running: bool,
    is_paused: bool,
    current_wave: Option<String>,
    completed_waves: Vec<String>,
}

impl CommandValidator {
    /// Create a new command validator.
    pub fn new(
        is_running: bool,
        is_paused: bool,
        current_wave: Option<String>,
        completed_waves: Vec<String>,
    ) -> Self {
        Self {
            is_running,
            is_paused,
            current_wave,
            completed_waves,
        }
    }

    /// Validate a command and return an error message if invalid.
    pub fn validate(&self, command: &CampaignCommand) -> Result<(), String> {
        match command {
            CampaignCommand::Pause { .. } => {
                if !self.is_running {
                    return Err("Can only pause a running campaign".to_string());
                }
                if self.is_paused {
                    return Err("Campaign is already paused".to_string());
                }
            }
            CampaignCommand::Resume => {
                if !self.is_paused {
                    return Err("Can only resume a paused campaign".to_string());
                }
            }
            CampaignCommand::Cancel { .. } => {
                // Always allowed
            }
            CampaignCommand::SkipWave { wave_id } => {
                if !self.is_running && !self.is_paused {
                    return Err("Can only skip waves in a running/paused campaign".to_string());
                }
                if Some(wave_id.as_str()) == self.current_wave.as_deref() {
                    return Err("Cannot skip the currently executing wave".to_string());
                }
            }
            CampaignCommand::RetryWave { wave_id } => {
                if !self.completed_waves.contains(wave_id) {
                    return Err(format!("Can only retry completed/failed waves (wave '{}' not in completed list)", wave_id));
                }
            }
            CampaignCommand::SetConcurrency { concurrency } => {
                if *concurrency == 0 {
                    return Err("Concurrency must be at least 1".to_string());
                }
            }
            CampaignCommand::SetRateLimit { rate_limit } => {
                if *rate_limit <= 0.0 {
                    return Err("Rate limit must be positive".to_string());
                }
            }
            CampaignCommand::ToggleLlm { .. }
            | CampaignCommand::DumpState
            | CampaignCommand::CollectMetrics => {
                // Always allowed
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_command_channel_send_recv() {
        let channel = CommandChannel::new();
        let sender = channel.create_sender();

        // Spawn a task to receive the command
        let recv_handle = tokio::spawn(async move {
            channel.recv().await
        });

        // Send a command
        let send_handle = tokio::spawn(async move {
            sender.pause("testing").await
        });

        // Wait for receive
        let received = recv_handle.await.unwrap();
        assert!(received.is_some());

        // Wait for send response
        let response = send_handle.await.unwrap();
        assert!(response.is_accepted());
    }

    #[tokio::test]
    async fn test_command_sender_methods() {
        let channel = CommandChannel::new();
        let sender = channel.create_sender();

        // Test various command methods
        let response = sender.resume().await;
        assert!(response.is_accepted());

        let response = sender.skip_wave("wave1").await;
        assert!(response.is_accepted());

        let response = sender.set_concurrency(20).await;
        assert!(response.is_accepted());
    }

    #[test]
    fn test_command_validator() {
        // Running campaign
        let validator = CommandValidator::new(
            true,   // is_running
            false,  // is_paused
            Some("wave1".to_string()),
            vec!["wave0".to_string()],
        );

        // Pause should be valid
        assert!(validator.validate(&CampaignCommand::Pause { reason: "test".to_string() }).is_ok());
        
        // Resume should be invalid (not paused)
        assert!(validator.validate(&CampaignCommand::Resume).is_err());
        
        // Skip current wave should be invalid
        assert!(validator.validate(&CampaignCommand::SkipWave { wave_id: "wave1".to_string() }).is_err());
        
        // Skip other wave should be valid
        assert!(validator.validate(&CampaignCommand::SkipWave { wave_id: "wave2".to_string() }).is_ok());
        
        // Retry completed wave should be valid
        assert!(validator.validate(&CampaignCommand::RetryWave { wave_id: "wave0".to_string() }).is_ok());
        
        // Retry non-completed wave should be invalid
        assert!(validator.validate(&CampaignCommand::RetryWave { wave_id: "wave2".to_string() }).is_err());
    }

    #[test]
    fn test_command_response_helpers() {
        let cmd = CampaignCommand::Pause { reason: "test".to_string() };
        
        let accepted = CommandResponse::accepted(cmd.clone());
        assert!(accepted.is_accepted());
        assert!(!accepted.is_rejected());
        
        let rejected = CommandResponse::rejected(cmd.clone(), "test reason");
        assert!(!rejected.is_accepted());
        assert!(rejected.is_rejected());
        assert_eq!(rejected.rejection_reason(), Some("test reason"));
        
        let completed = CommandResponse::completed(cmd.clone(), "done");
        assert!(completed.is_accepted());
        assert!(!completed.is_rejected());
    }

    #[tokio::test]
    async fn test_command_channel_capacity() {
        let channel = CommandChannel::with_capacity(5);
        
        // Fill the channel
        for i in 0..5 {
            let _ = channel.sender.send((
                CampaignCommand::Pause { reason: format!("test {}", i) },
                tokio::sync::mpsc::channel(1).0,
            )).await;
        }
        
        assert_eq!(channel.pending_count(), 5);
        assert!(channel.has_pending());
    }
}
