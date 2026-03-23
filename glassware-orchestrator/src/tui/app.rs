//! Main TUI application logic.
//!
//! Provides the App struct with the main event loop,
//! keyboard handling, and state management.

use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use tokio::sync::mpsc;
use tracing::{debug, info};

use glassware_orchestrator::campaign::{
    event_bus::{EventBus, CampaignEvent},
    state_manager::StateManager,
    command_channel::CommandChannel,
    types::{CampaignState, CampaignStatus, WaveStatus, WaveMode, WaveState},
};

use super::ui::Ui;

/// Result type for TUI operations.
pub type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Main application tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTab {
    Campaign,
    Findings,
    Logs,
}

/// Main TUI application.
pub struct App {
    /// Whether the app is running.
    running: bool,
    /// Current active tab.
    active_tab: AppTab,
    /// Campaign state snapshot.
    state: Option<CampaignState>,
    /// Recent events for display.
    recent_events: Vec<CampaignEvent>,
    /// Event bus receiver.
    event_rx: mpsc::Receiver<CampaignEvent>,
    /// Command channel for steering.
    command_tx: CommandChannel,
    /// Case ID for the campaign.
    case_id: String,
}

impl App {
    /// Create a new TUI application.
    ///
    /// # Arguments
    /// * `case_id` - Campaign case ID to monitor
    /// * `event_bus` - Event bus for receiving campaign events
    /// * `command_channel` - Command channel for steering the campaign
    pub fn new(
        case_id: String,
        event_bus: EventBus,
        command_channel: CommandChannel,
    ) -> Self {
        let (tx, rx) = mpsc::channel(512);
        let event_bus_clone = event_bus.clone();

        // Spawn task to forward broadcast events to mpsc channel
        tokio::spawn(async move {
            let mut event_rx = event_bus_clone.subscribe();
            while let Ok(event) = event_rx.recv().await {
                if tx.send(event).await.is_err() {
                    break;
                }
            }
        });

        Self {
            running: true,
            active_tab: AppTab::Campaign,
            state: None,
            recent_events: Vec::with_capacity(10),
            event_rx: rx,
            command_tx: command_channel,
            case_id,
        }
    }

    /// Create a new TUI application with hardcoded sample data (for demo).
    pub fn with_sample_data() -> Self {
        let event_bus = EventBus::new(512);
        let command_channel = CommandChannel::new();

        let mut app = Self::new("demo-123".to_string(), event_bus, command_channel);

        // Create sample campaign state
        let mut state = CampaignState::new("demo-123", "Wave 6 Calibration");
        state.status = CampaignStatus::Running;

        // Add sample waves
        let mut wave_6a = WaveState::new("6A", "Known Malicious", WaveMode::Hunt);
        wave_6a.status = WaveStatus::Completed;
        wave_6a.packages_total = 2;
        wave_6a.packages_scanned = 2;
        wave_6a.packages_flagged = 2;
        wave_6a.packages_malicious = 2;
        state.waves.insert("6A".to_string(), wave_6a);

        let mut wave_6b = WaveState::new("6B", "Clean Baseline", WaveMode::Hunt);
        wave_6b.status = WaveStatus::Running;
        wave_6b.packages_total = 5;
        wave_6b.packages_scanned = 3;
        wave_6b.packages_flagged = 0;
        wave_6b.packages_malicious = 0;
        state.waves.insert("6B".to_string(), wave_6b);

        let mut wave_6c = WaveState::new("6C", "React Native", WaveMode::Hunt);
        wave_6c.status = WaveStatus::Pending;
        wave_6c.packages_total = 4;
        wave_6c.packages_scanned = 0;
        wave_6c.packages_flagged = 0;
        wave_6c.packages_malicious = 0;
        state.waves.insert("6C".to_string(), wave_6c);

        state.current_wave = Some("6B".to_string());
        state.recalculate_stats();

        app.state = Some(state);

        // Add sample recent events
        app.recent_events = vec![
            CampaignEvent::PackageScanned {
                package: "axios".to_string(),
                version: "1.6.7".to_string(),
                wave_id: "6B".to_string(),
                threat_score: 2.5,
                findings_count: 0,
            },
            CampaignEvent::PackageScanned {
                package: "chalk".to_string(),
                version: "5.3.0".to_string(),
                wave_id: "6B".to_string(),
                threat_score: 1.2,
                findings_count: 0,
            },
            CampaignEvent::WaveCompleted {
                wave_id: "6A".to_string(),
                packages_scanned: 2,
                packages_flagged: 2,
                packages_malicious: 2,
                duration_seconds: 45,
            },
        ];

        app
    }

    /// Run the TUI application.
    pub async fn run(&mut self) -> AppResult<()> {
        // Setup terminal
        enable_raw_mode()?;
        std::io::stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
        terminal.clear()?;

        // Create UI
        let mut ui = Ui::new();

        info!("TUI started for campaign: {}", self.case_id);

        // Main loop
        while self.running {
            // Draw UI
            terminal.draw(|frame| {
                ui.render(frame, self);
            })?;

            // Handle events with timeout
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key_input(key);
                }
            }

            // Receive events from channel (non-blocking)
            while let Ok(event) = self.event_rx.try_recv() {
                self.handle_event(event);
            }
        }

        // Restore terminal
        disable_raw_mode()?;
        std::io::stdout().execute(LeaveAlternateScreen)?;

        info!("TUI stopped");

        Ok(())
    }

    /// Handle keyboard input.
    fn handle_key_input(&mut self, key: KeyEvent) {
        // Handle Ctrl+C specially
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            info!("Ctrl+C pressed - pausing campaign");
            // In a real implementation, send pause command
            return;
        }

        match key.code {
            KeyCode::Char('q') => {
                info!("Quit requested");
                self.running = false;
            }
            KeyCode::Tab => {
                // Cycle through tabs
                self.active_tab = match self.active_tab {
                    AppTab::Campaign => AppTab::Findings,
                    AppTab::Findings => AppTab::Logs,
                    AppTab::Logs => AppTab::Campaign,
                };
                debug!("Switched to tab: {:?}", self.active_tab);
            }
            KeyCode::Char('p') => {
                info!("Pause requested");
                // In a real implementation, send pause command
            }
            KeyCode::Char('x') => {
                info!("Cancel requested");
                // In a real implementation, send cancel command
            }
            KeyCode::Char('s') => {
                info!("Skip wave requested");
                // In a real implementation, send skip wave command
            }
            KeyCode::Char('c') => {
                info!("Concurrency adjustment requested");
                // In a real implementation, show concurrency dialog
            }
            _ => {}
        }
    }

    /// Handle campaign events.
    fn handle_event(&mut self, event: CampaignEvent) {
        debug!("Received event: {:?}", event);

        // Update recent events
        self.recent_events.push(event.clone());
        if self.recent_events.len() > 10 {
            self.recent_events.remove(0);
        }

        // For demo purposes, update state based on events
        // In a real implementation, this would come from StateManager
        match event {
            CampaignEvent::PackageScanned { wave_id, package, version, .. } => {
                if let Some(state) = &mut self.state {
                    if let Some(wave) = state.waves.get_mut(&wave_id) {
                        wave.packages_scanned += 1;
                        state.recalculate_stats();
                    }
                }
                info!("Package scanned: {}@{} in wave {}", package, version, wave_id);
            }
            CampaignEvent::WaveCompleted { wave_id, packages_scanned, packages_flagged, .. } => {
                if let Some(state) = &mut self.state {
                    if let Some(wave) = state.waves.get_mut(&wave_id) {
                        wave.status = WaveStatus::Completed;
                        wave.packages_scanned = packages_scanned;
                        wave.packages_flagged = packages_flagged;
                    }
                    state.recalculate_stats();
                }
            }
            _ => {}
        }
    }

    /// Get the current active tab.
    pub fn active_tab(&self) -> AppTab {
        self.active_tab
    }

    /// Get the campaign state.
    pub fn state(&self) -> Option<&CampaignState> {
        self.state.as_ref()
    }

    /// Get recent events.
    pub fn recent_events(&self) -> &[CampaignEvent] {
        &self.recent_events
    }

    /// Get the case ID.
    pub fn case_id(&self) -> &str {
        &self.case_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_tab_cycle() {
        let mut tab = AppTab::Campaign;
        tab = match tab {
            AppTab::Campaign => AppTab::Findings,
            AppTab::Findings => AppTab::Logs,
            AppTab::Logs => AppTab::Campaign,
        };
        assert_eq!(tab, AppTab::Findings);
    }

    #[test]
    fn test_app_with_sample_data() {
        let app = App::with_sample_data();
        assert!(app.state.is_some());
        assert_eq!(app.case_id(), "demo-123");
        assert!(!app.recent_events.is_empty());
    }
}
