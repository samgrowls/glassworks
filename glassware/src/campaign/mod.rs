//! Campaign System for GlassWorm Scanner
//!
//! This module provides a robust campaign orchestration system for running
//! large-scale security scanning campaigns with features like:
//!
//! - **Wave-based execution**: Organize scans into logical waves with dependencies
//! - **Event-driven architecture**: Real-time event publishing for TUI/CLI monitoring
//! - **Queryable state**: Thread-safe state access for progress reporting
//! - **Command steering**: Pause, resume, skip waves, adjust concurrency at runtime
//! - **Checkpoint/resume**: Survive interruptions and resume from last checkpoint
//! - **Evidence collection**: Court-admissible evidence with chain of custody
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      UI Layer (Pluggable)                        │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
//! │  │ CLI Progress│  │   TUI       │  │   Headless (CI/CD)    │  │
//! │  │ (indicatif) │  │ (ratatui)   │  │   (JSON output only)  │  │
//! │  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
//! └─────────────────────────────────────────────────────────────────┘
//!                            │
//!                   Event Bus (pub/sub)
//!                            │
//! ┌──────────────────────────▼────────────────────────────────────┐
//! │                      Core Engine                               │
//! │  ┌──────────────┐               │               ┌──────────┐  │
//! │  │   Campaign   │◄──────────────┼──────────────►│ Command  │  │
//! │  │   Executor   │               │               │ Channel  │  │
//! │  └──────────────┘               │               └──────────┘  │
//! │         │              ┌────────▼────────┐                    │
//! │         │              │  State Manager  │                    │
//! │         │              │  (queryable)    │                    │
//! │         │              └─────────────────┘                    │
//! └───────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use glassware::campaign::{
//!     CampaignConfig, CampaignExecutor, EventBus, StateManager, CommandChannel,
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create event bus
//!     let event_bus = EventBus::new(512);
//!
//!     // Create state manager
//!     let state = StateManager::new("case-123", "Wave 6 Hunt", event_bus.clone());
//!
//!     // Create command channel
//!     let commands = CommandChannel::new();
//!
//!     // Load campaign config
//!     let config = CampaignConfig::from_file("campaigns/wave6.toml")?;
//!
//!     // Create executor and run
//!     let executor = CampaignExecutor::new(config, state, event_bus, commands).await;
//!     executor.run().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Modules
//!
//! - [`types`]: Core types (CaseId, CampaignStatus, WaveState, etc.)
//! - [`event_bus`]: Event pub/sub system for real-time monitoring
//! - [`state_manager`]: Thread-safe state with event publishing
//! - [`command_channel`]: Command steering for runtime control
//! - [`config`]: Campaign configuration (TOML parsing)
//! - [`executor`]: Campaign execution engine
//! - [`wave`]: Wave execution logic

pub mod types;
pub mod event_bus;
pub mod state_manager;
pub mod command_channel;
pub mod config;
pub mod wave;
pub mod executor;
pub mod checkpoint;
pub mod report;
pub mod query;

// Re-export commonly used types for convenience
pub use types::{
    CaseId,
    Priority,
    CampaignStatus,
    WaveMode,
    WaveStatus,
    PackageStage,
    SortOrder,
    GitHubSort,
    WaveState,
    CampaignStats,
    ActivePackage,
    CampaignState,
};

pub use event_bus::{
    EventBus,
    CampaignEvent,
    CampaignCommand,
    EventBusExt,
};

pub use state_manager::{
    StateManager,
    CommandHandle,
};

pub use command_channel::{
    CommandChannel,
    CommandSender,
    CommandValidator,
    CommandResponse,
    CommandMessage,
};

// Re-export key types for convenience
pub use config::{
    CampaignConfig,
    CampaignMetadata,
    CampaignSettings,
    WaveConfig,
    WaveSource,
};

pub use wave::{
    WaveExecutor,
    WaveResult,
    PackageSpec,
};

pub use executor::{
    CampaignExecutor,
    CampaignResult,
};

pub use checkpoint::{CheckpointManager, CampaignCheckpoint};

pub use report::{ReportGenerator, ReportContext, ReportError, ConfigSummary};

/// Campaign system version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Check if the campaign system is available.
///
/// Returns `true` if all required dependencies are available.
pub fn is_available() -> bool {
    // All core dependencies are part of std/tokio, so always available
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_available() {
        assert!(is_available());
    }

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[tokio::test]
    async fn test_campaign_module_integration() {
        // Create core components
        let event_bus = EventBus::new(64);
        let state = StateManager::new("test-case", "Test Campaign", event_bus.clone());
        let commands = CommandChannel::new();
        
        // Verify state initialized correctly
        let snapshot = state.snapshot().await;
        assert_eq!(snapshot.case_id, "test-case");
        assert_eq!(snapshot.campaign_name, "Test Campaign");
        assert_eq!(snapshot.status, CampaignStatus::Initializing);
        
        // Verify command channel works
        let sender = commands.create_sender();
        let response = sender.resume().await;
        assert!(response.is_accepted());
        
        // Verify event bus works
        let mut rx = event_bus.subscribe();
        event_bus.publish(CampaignEvent::CampaignStarted {
            case_id: "test".to_string(),
            campaign_name: "Test".to_string(),
            config_hash: "abc".to_string(),
        });
        
        let event = rx.recv().await.unwrap();
        match event {
            CampaignEvent::CampaignStarted { case_id, .. } => {
                assert_eq!(case_id, "test");
            }
            _ => panic!("Wrong event type"),
        }
    }
}
