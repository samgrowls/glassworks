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
use tracing::{debug, error, info};

use glassware_orchestrator::campaign::{
    event_bus::{EventBus, CampaignEvent, CampaignCommand},
    command_channel::{CommandChannel, CommandSender},
    types::{CampaignState, CampaignStatus, WaveStatus, WaveMode, WaveState},
};
use glassware_orchestrator::llm::LlmVerdict;

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

/// Concurrency dialog state.
#[derive(Debug, Clone)]
pub struct ConcurrencyDialog {
    /// Whether the dialog is visible.
    pub visible: bool,
    /// Current concurrency value.
    pub current_value: usize,
    /// Input buffer for new value.
    pub input_buffer: String,
}

impl ConcurrencyDialog {
    fn new(current_concurrency: usize) -> Self {
        Self {
            visible: false,
            current_value: current_concurrency,
            input_buffer: String::new(),
        }
    }

    fn show(&mut self) {
        self.visible = true;
        self.input_buffer.clear();
    }

    fn hide(&mut self) {
        self.visible = false;
        self.input_buffer.clear();
    }

    /// Check if the dialog is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    fn add_digit(&mut self, digit: char) {
        if self.input_buffer.len() < 3 {
            self.input_buffer.push(digit);
        }
    }

    fn backspace(&mut self) {
        self.input_buffer.pop();
    }

    fn confirm(&mut self) -> Option<usize> {
        if let Ok(value) = self.input_buffer.parse::<usize>() {
            if value >= 1 && value <= 100 {
                self.current_value = value;
                self.hide();
                return Some(value);
            }
        }
        None
    }

    fn cancel(&mut self) {
        self.hide();
    }
}

/// Severity level for flagged packages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

impl Severity {
    pub fn from_score(score: f32) -> Self {
        if score >= 9.0 {
            Severity::Critical
        } else if score >= 7.0 {
            Severity::High
        } else if score >= 4.0 {
            Severity::Medium
        } else {
            Severity::Low
        }
    }

    pub fn color(&self) -> ratatui::style::Color {
        match self {
            Severity::Critical => ratatui::style::Color::Red,
            Severity::High => ratatui::style::Color::Yellow,
            Severity::Medium => ratatui::style::Color::Cyan,
            Severity::Low => ratatui::style::Color::White,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Critical => "CRITICAL",
            Severity::High => "HIGH",
            Severity::Medium => "MEDIUM",
            Severity::Low => "LOW",
        }
    }
}

/// Flagged package information.
#[derive(Debug, Clone)]
pub struct FlaggedPackage {
    /// Package name.
    pub name: String,
    /// Package version.
    pub version: String,
    /// Wave ID this package belongs to.
    pub wave_id: String,
    /// Threat score.
    pub threat_score: f32,
    /// Number of findings.
    pub findings_count: usize,
    /// LLM verdict (if analyzed).
    pub llm_verdict: Option<LlmVerdict>,
    /// LLM explanation (if analyzed).
    pub llm_explanation: Option<String>,
}

impl FlaggedPackage {
    pub fn severity(&self) -> Severity {
        Severity::from_score(self.threat_score)
    }
}

/// Package detail view state.
#[derive(Debug, Clone)]
pub struct PackageDetailView {
    /// Whether the detail view is visible.
    pub visible: bool,
    /// Selected package index in the findings list.
    pub selected_index: usize,
    /// Scroll offset for findings list.
    pub scroll_offset: usize,
}

impl PackageDetailView {
    fn new() -> Self {
        Self {
            visible: false,
            selected_index: 0,
            scroll_offset: 0,
        }
    }

    fn show(&mut self, index: usize) {
        self.visible = true;
        self.selected_index = index;
        self.scroll_offset = 0;
    }

    fn hide(&mut self) {
        self.visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

/// Package query dialog state.
#[derive(Debug, Clone)]
pub struct PackageQueryDialog {
    /// Whether the dialog is visible.
    pub visible: bool,
    /// Input buffer for the query.
    pub input_buffer: String,
    /// LLM response (if queried).
    pub response: Option<String>,
    /// Whether a query is in progress.
    pub querying: bool,
}

impl PackageQueryDialog {
    fn new() -> Self {
        Self {
            visible: false,
            input_buffer: String::new(),
            response: None,
            querying: false,
        }
    }

    fn show(&mut self) {
        self.visible = true;
        self.input_buffer.clear();
        self.response = None;
        self.querying = false;
    }

    fn hide(&mut self) {
        self.visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    fn add_char(&mut self, c: char) {
        self.input_buffer.push(c);
    }

    fn backspace(&mut self) {
        self.input_buffer.pop();
    }

    fn get_query(&self) -> String {
        self.input_buffer.clone()
    }

    fn set_querying(&mut self, querying: bool) {
        self.querying = querying;
    }

    fn set_response(&mut self, response: String) {
        self.response = Some(response);
        self.querying = false;
    }
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
    /// Command sender for steering.
    command_sender: Option<CommandSender>,
    /// Case ID for the campaign.
    case_id: String,
    /// Concurrency dialog state.
    concurrency_dialog: ConcurrencyDialog,
    /// Last command feedback message.
    command_feedback: Option<String>,
    /// Default concurrency value.
    default_concurrency: usize,
    /// List of flagged packages for the Findings tab.
    flagged_packages: Vec<FlaggedPackage>,
    /// Selected index in the flagged packages list.
    selected_package_index: usize,
    /// Package detail view state.
    package_detail_view: PackageDetailView,
    /// Package query dialog state.
    package_query_dialog: PackageQueryDialog,
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
        let command_sender = command_channel.create_sender();
        let default_concurrency = 10;

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
            command_sender: Some(command_sender),
            case_id,
            concurrency_dialog: ConcurrencyDialog::new(default_concurrency),
            command_feedback: None,
            default_concurrency,
            flagged_packages: Vec::new(),
            selected_package_index: 0,
            package_detail_view: PackageDetailView::new(),
            package_query_dialog: PackageQueryDialog::new(),
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

        // Add sample flagged packages
        app.flagged_packages = vec![
            FlaggedPackage {
                name: "colors-linux".to_string(),
                version: "1.0.0".to_string(),
                wave_id: "6A".to_string(),
                threat_score: 9.5,
                findings_count: 5,
                llm_verdict: None,
                llm_explanation: None,
            },
            FlaggedPackage {
                name: "ua-parser-js".to_string(),
                version: "0.8.0".to_string(),
                wave_id: "6A".to_string(),
                threat_score: 8.2,
                findings_count: 3,
                llm_verdict: None,
                llm_explanation: None,
            },
        ];

        app
    }

    /// Create a new TUI application with live event subscription.
    ///
    /// # Arguments
    /// * `case_id` - Campaign case ID to monitor
    /// * `event_rx` - Optional broadcast receiver for campaign events
    /// * `command_sender` - Command sender for steering the campaign
    pub fn with_live_subscription(
        case_id: String,
        event_rx: Option<tokio::sync::broadcast::Receiver<CampaignEvent>>,
        command_sender: CommandSender,
    ) -> Self {
        let (tx, rx) = mpsc::channel(512);
        let default_concurrency = 10;

        // If event_rx provided, spawn task to forward events
        if let Some(mut event_rx) = event_rx {
            tokio::spawn(async move {
                while let Ok(event) = event_rx.recv().await {
                    if tx.send(event).await.is_err() {
                        break;
                    }
                }
            });
        }

        Self {
            running: true,
            active_tab: AppTab::Campaign,
            state: None,
            recent_events: Vec::with_capacity(10),
            event_rx: rx,
            command_sender: Some(command_sender),
            case_id,
            concurrency_dialog: ConcurrencyDialog::new(default_concurrency),
            command_feedback: None,
            default_concurrency,
            flagged_packages: Vec::new(),
            selected_package_index: 0,
            package_detail_view: PackageDetailView::new(),
            package_query_dialog: PackageQueryDialog::new(),
        }
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
            // Poll event bus and update state on events
            let mut state_changed = false;
            while let Ok(event) = self.event_rx.try_recv() {
                self.handle_event(event);
                state_changed = true;
            }

            // Clear command feedback after a delay
            if state_changed && self.command_feedback.is_some() {
                // Keep feedback visible for a bit longer when events arrive
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
        // Handle package query dialog input if visible
        if self.package_query_dialog.is_visible() {
            self.handle_package_query_input(key);
            return;
        }

        // Handle package detail view input if visible
        if self.package_detail_view.is_visible() {
            self.handle_package_detail_input(key);
            return;
        }

        // Handle concurrency dialog input if visible
        if self.concurrency_dialog.is_visible() {
            self.handle_concurrency_dialog_input(key);
            return;
        }

        // Handle Ctrl+C specially
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            info!("Ctrl+C pressed - pausing campaign");
            self.send_command(CampaignCommand::Pause { reason: "User requested via Ctrl+C".to_string() });
            return;
        }

        match key.code {
            KeyCode::Char('q') => {
                info!("Quit requested");
                self.running = false;
            }
            KeyCode::Char('p') => {
                info!("Pause requested");
                self.send_command(CampaignCommand::Pause { reason: "User requested via TUI".to_string() });
            }
            KeyCode::Char('x') => {
                info!("Cancel requested");
                self.send_command(CampaignCommand::Cancel { save_checkpoint: true });
            }
            KeyCode::Char('r') => {
                info!("Resume requested");
                self.send_command(CampaignCommand::Resume);
            }
            KeyCode::Char('s') => {
                info!("Skip wave requested");
                self.send_command(CampaignCommand::SkipWave { wave_id: "current".to_string() });
            }
            KeyCode::Char('c') => {
                info!("Concurrency adjustment requested");
                self.show_concurrency_dialog();
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
            KeyCode::Enter => {
                // Open package detail view when in Findings tab
                if self.active_tab == AppTab::Findings && !self.flagged_packages.is_empty() {
                    self.open_package_detail(self.selected_package_index);
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                // Navigate up in findings list
                if self.active_tab == AppTab::Findings && !self.flagged_packages.is_empty() {
                    if self.selected_package_index > 0 {
                        self.selected_package_index -= 1;
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                // Navigate down in findings list
                if self.active_tab == AppTab::Findings && !self.flagged_packages.is_empty() {
                    if self.selected_package_index < self.flagged_packages.len() - 1 {
                        self.selected_package_index += 1;
                    }
                }
            }
            KeyCode::Char('l') => {
                // Run LLM analysis on selected package
                if self.active_tab == AppTab::Findings && !self.flagged_packages.is_empty() {
                    self.run_llm_analysis_on_selected();
                }
            }
            KeyCode::Char('0'..='9') => {
                // Number keys can be used for quick concurrency input when dialog is shown
                // Handled by concurrency dialog
            }
            _ => {}
        }
    }

    /// Handle input for the package detail view.
    fn handle_package_detail_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                // Close detail view
                self.package_detail_view.hide();
            }
            KeyCode::Char('?') => {
                // Open query dialog
                self.package_query_dialog.show();
            }
            KeyCode::Char('l') => {
                // Run LLM analysis on this package
                let idx = self.package_detail_view.selected_index;
                self.run_llm_analysis_on_selected();
                // Keep detail view open to show updated results
            }
            KeyCode::Up | KeyCode::Char('k') => {
                // Scroll up in findings
                if self.package_detail_view.scroll_offset > 0 {
                    self.package_detail_view.scroll_offset -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                // Scroll down in findings
                let pkg = &self.flagged_packages[self.package_detail_view.selected_index];
                let max_scroll = pkg.findings_count.saturating_sub(1);
                if self.package_detail_view.scroll_offset < max_scroll {
                    self.package_detail_view.scroll_offset += 1;
                }
            }
            _ => {}
        }
    }

    /// Handle input for the package query dialog.
    fn handle_package_query_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                // Submit query
                let query = self.package_query_dialog.get_query();
                if !query.is_empty() && !self.package_query_dialog.querying {
                    self.submit_package_query(query);
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                // Close dialog
                self.package_query_dialog.hide();
            }
            KeyCode::Backspace => {
                self.package_query_dialog.backspace();
            }
            KeyCode::Char(c) => {
                self.package_query_dialog.add_char(c);
            }
            _ => {}
        }
    }

    /// Handle input for the concurrency dialog.
    fn handle_concurrency_dialog_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(digit) if digit.is_ascii_digit() => {
                self.concurrency_dialog.add_digit(digit);
            }
            KeyCode::Backspace => {
                self.concurrency_dialog.backspace();
            }
            KeyCode::Enter => {
                if let Some(new_value) = self.concurrency_dialog.confirm() {
                    info!("Concurrency changed to: {}", new_value);
                    self.send_command(CampaignCommand::SetConcurrency { concurrency: new_value });
                }
            }
            KeyCode::Esc => {
                self.concurrency_dialog.cancel();
            }
            _ => {}
        }
    }

    /// Send a command to the campaign executor.
    fn send_command(&mut self, command: CampaignCommand) {
        if let Some(ref sender) = self.command_sender {
            let command_name = command.name().to_string();
            let command_name_for_feedback = command_name.clone();
            
            // Spawn task to send command asynchronously
            let sender_clone = sender.clone();
            tokio::spawn(async move {
                match sender_clone.send(command).await {
                    glassware_orchestrator::campaign::command_channel::CommandResponse::Accepted { .. } => {
                        info!("Command '{}' accepted", command_name);
                    }
                    glassware_orchestrator::campaign::command_channel::CommandResponse::Completed { result, .. } => {
                        info!("Command '{}' completed: {}", command_name, result);
                    }
                    glassware_orchestrator::campaign::command_channel::CommandResponse::Rejected { reason, .. } => {
                        error!("Command '{}' rejected: {}", command_name, reason);
                    }
                }
            });

            // Set feedback message
            self.command_feedback = Some(format!("Command '{}' sent", command_name_for_feedback));
        } else {
            self.command_feedback = Some("Command channel not available".to_string());
        }
    }

    /// Show the concurrency adjustment dialog.
    fn show_concurrency_dialog(&mut self) {
        self.concurrency_dialog.show();
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

    /// Open the package detail view for a specific package.
    fn open_package_detail(&mut self, index: usize) {
        if index < self.flagged_packages.len() {
            self.package_detail_view.show(index);
            info!("Opened detail view for package: {}", self.flagged_packages[index].name);
        }
    }

    /// Run LLM analysis on the selected package.
    fn run_llm_analysis_on_selected(&mut self) {
        let idx = self.selected_package_index;
        if idx >= self.flagged_packages.len() {
            return;
        }

        // Clone necessary data before spawning task to avoid borrow issues
        let package_name = self.flagged_packages[idx].name.clone();
        let package_version = self.flagged_packages[idx].version.clone();
        let threat_score = self.flagged_packages[idx].threat_score;

        // Set a placeholder to indicate analysis is in progress
        self.flagged_packages[idx].llm_explanation = Some("Analyzing...".to_string());

        // Clone for feedback message (will be used after spawn)
        let feedback_name = package_name.clone();
        let feedback_version = package_version.clone();

        // Spawn async task to run LLM analysis
        tokio::spawn(async move {
            // In a real implementation, this would call the LLM analyzer
            // For now, we'll simulate with a delay and mock response
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            let mock_verdict = LlmVerdict {
                is_malicious: threat_score >= 7.0,
                confidence: 0.85,
                explanation: format!(
                    "Based on analysis of {}@{}, the package shows signs of malicious intent. \
                    Detected patterns include suspicious code obfuscation and potential data exfiltration. \
                    Recommend immediate removal and audit of dependent projects.",
                    package_name, package_version
                ),
                recommendations: vec![
                    "Remove this package from dependencies".to_string(),
                    "Audit code that imports this package".to_string(),
                    "Check for unauthorized network connections".to_string(),
                ],
                false_positive_indicators: vec![],
            };

            // Note: In a real implementation, we'd need to send this back via channel
            // For demo purposes, we just log it
            info!("LLM analysis complete for {}@{}: malicious={}", 
                package_name, package_version, mock_verdict.is_malicious);
        });

        // Set feedback message
        self.command_feedback = Some(format!("LLM analysis started for {}@{}", feedback_name, feedback_version));
    }

    /// Submit a package-specific query to the LLM.
    fn submit_package_query(&mut self, query: String) {
        let idx = self.package_detail_view.selected_index;
        if idx >= self.flagged_packages.len() {
            return;
        }

        // Clone necessary data before spawning task
        let package_name = self.flagged_packages[idx].name.clone();
        let package_version = self.flagged_packages[idx].version.clone();
        let threat_score = self.flagged_packages[idx].threat_score;
        let findings_count = self.flagged_packages[idx].findings_count;

        info!("Submitting query for package {}: {}", package_name, query);

        self.package_query_dialog.set_querying(true);

        // Clone for feedback message
        let feedback_name = package_name.clone();
        let feedback_version = package_version.clone();

        // Spawn async task to run the query
        tokio::spawn(async move {
            // In a real implementation, this would call query_package()
            // For now, we'll simulate with a delay and mock response
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            let mock_response = format!(
                "Analysis of {}@{} (threat score: {:.1}, {} findings):\n\n\
                This package exhibits several concerning patterns:\n\
                1. Code obfuscation techniques detected\n\
                2. Potential data exfiltration endpoints\n\
                3. Suspicious runtime behavior\n\n\
                Recommendation: Treat as malicious and remove from your dependency tree.",
                package_name, package_version, threat_score, findings_count
            );

            info!("Query response for {}@{}: {}", package_name, package_version, mock_response);
        });

        self.command_feedback = Some(format!("Query submitted for {}@{}", feedback_name, feedback_version));
    }

    /// Get the flagged packages list.
    pub fn flagged_packages(&self) -> &[FlaggedPackage] {
        &self.flagged_packages
    }

    /// Get the selected package index.
    pub fn selected_package_index(&self) -> usize {
        self.selected_package_index
    }

    /// Get the package detail view state.
    pub fn package_detail_view(&self) -> &PackageDetailView {
        &self.package_detail_view
    }

    /// Get the package query dialog state.
    pub fn package_query_dialog(&self) -> &PackageQueryDialog {
        &self.package_query_dialog
    }

    /// Get the currently selected package.
    pub fn selected_package(&self) -> Option<&FlaggedPackage> {
        self.flagged_packages.get(self.selected_package_index)
    }

    /// Get the package being viewed in detail.
    pub fn detail_package(&self) -> Option<&FlaggedPackage> {
        if let Some(idx) = self.package_detail_view.visible.then(|| self.package_detail_view.selected_index) {
            self.flagged_packages.get(idx)
        } else {
            None
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

    /// Get the concurrency dialog state.
    pub fn concurrency_dialog(&self) -> &ConcurrencyDialog {
        &self.concurrency_dialog
    }

    /// Get the command feedback message.
    pub fn command_feedback(&self) -> Option<&str> {
        self.command_feedback.as_deref()
    }

    /// Get the default concurrency value.
    pub fn default_concurrency(&self) -> usize {
        self.default_concurrency
    }

    /// Clear the command feedback message.
    pub fn clear_command_feedback(&mut self) {
        self.command_feedback = None;
    }

    /// Set the campaign state (for external loading from checkpoint).
    pub fn set_state(&mut self, state: CampaignState) {
        self.state = Some(state);
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
