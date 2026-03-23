//! UI rendering for the GlassWorm TUI.
//!
//! Provides rendering functions for the main screen layout,
//! progress bars, wave lists, and event logs.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Gauge, List, ListItem, Clear, Tabs},
};

use super::app::{App, AppTab, ConcurrencyDialog, FlaggedPackage, Severity, PackageDetailView, PackageQueryDialog};
use glassware::campaign::{
    types::{CampaignStatus, WaveStatus},
    event_bus::CampaignEvent,
};

/// Main UI renderer.
pub struct Ui {
    /// Title of the application.
    title: String,
}

impl Ui {
    /// Create a new UI renderer.
    pub fn new() -> Self {
        Self {
            title: "GlassWorm Campaign Monitor".to_string(),
        }
    }

    /// Render the UI.
    pub fn render(&mut self, frame: &mut Frame, app: &App) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title bar
                Constraint::Length(3),  // Progress bar
                Constraint::Min(10),    // Main content (waves/tabs)
                Constraint::Length(7),  // Recent events
                Constraint::Length(3),  // Help bar with command feedback
            ])
            .split(frame.size());

        // Render title bar
        self.render_title_bar(frame, app, chunks[0]);

        // Render progress bar
        self.render_progress_bar(frame, app, chunks[1]);

        // Render main content based on active tab
        match app.active_tab() {
            AppTab::Campaign => self.render_campaign_tab(frame, app, chunks[2]),
            AppTab::Findings => self.render_findings_tab(frame, app, chunks[2]),
            AppTab::Logs => self.render_logs_tab(frame, app, chunks[2]),
        }

        // Render recent events
        self.render_event_log(frame, app, chunks[3]);

        // Render help bar with command feedback
        self.render_help_bar(frame, app, chunks[4]);

        // Render package detail view if visible
        if app.package_detail_view().is_visible() {
            self.render_package_detail_view(frame, app);
        }

        // Render package query dialog if visible
        if app.package_query_dialog().is_visible() {
            self.render_package_query_dialog(frame, app);
        }

        // Render concurrency dialog if visible
        if app.concurrency_dialog().is_visible() {
            self.render_concurrency_dialog(frame, app.concurrency_dialog());
        }
    }

    /// Render the title bar.
    fn render_title_bar(&self, frame: &mut Frame, app: &App, area: Rect) {
        let status = app.state()
            .map(|s| s.status)
            .unwrap_or(CampaignStatus::Initializing);

        let status_text = match status {
            CampaignStatus::Initializing => "Initializing",
            CampaignStatus::Running => "Running",
            CampaignStatus::Paused => "Paused",
            CampaignStatus::Completed => "Completed",
            CampaignStatus::Failed => "Failed",
            CampaignStatus::Cancelled => "Cancelled",
        };

        let campaign_name = app.state()
            .map(|s| s.campaign_name.as_str())
            .unwrap_or("Unknown Campaign");

        let status_style = match status {
            CampaignStatus::Running => Style::default().fg(Color::Green),
            CampaignStatus::Paused => Style::default().fg(Color::Yellow),
            CampaignStatus::Completed => Style::default().fg(Color::Blue),
            CampaignStatus::Failed => Style::default().fg(Color::Red),
            CampaignStatus::Cancelled => Style::default().fg(Color::Red),
            CampaignStatus::Initializing => Style::default().fg(Color::Gray),
        };

        let title = format!(" Campaign: {} ", campaign_name);
        let status = format!(" [{}] ", status_text);

        let padding_len = area.width.saturating_sub((title.len() + status.len()) as u16);
        let title_block = Paragraph::new(Line::from(vec![
            Span::styled(title, Style::default().bold()),
            Span::raw(" ".repeat(padding_len as usize)),
            Span::styled(status, status_style),
        ]))
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(self.title.as_str())
            .title_style(Style::default().bold().fg(Color::Cyan)));

        frame.render_widget(title_block, area);
    }

    /// Render the progress bar.
    fn render_progress_bar(&self, frame: &mut Frame, app: &App, area: Rect) {
        let (progress, label) = if let Some(state) = app.state() {
            let total: usize = state.waves.values().map(|w| w.packages_total).sum();
            let scanned: usize = state.waves.values().map(|w| w.packages_scanned).sum();

            let ratio = if total > 0 {
                scanned as f32 / total as f32
            } else {
                0.0
            };

            let percentage = (ratio * 100.0) as u32;

            // Calculate ETA (simplified - based on average scan time)
            let elapsed = chrono::Utc::now().signed_duration_since(state.started_at).num_seconds() as u64;

            let eta = if scanned > 0 && total > scanned {
                let avg_time = elapsed as f32 / scanned as f32;
                let remaining = total - scanned;
                let eta_seconds = (avg_time * remaining as f32) as u64;
                format!("{}m {}s", eta_seconds / 60, eta_seconds % 60)
            } else {
                "N/A".to_string()
            };

            (ratio, format!("Progress: {}%  ETA: {}", percentage, eta))
        } else {
            (0.0, "Progress: Waiting for data...".to_string())
        };

        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Blue))
            .percent((progress * 100.0) as u16)
            .label(label);

        frame.render_widget(gauge, area);
    }

    /// Render the campaign tab (main view).
    fn render_campaign_tab(&self, frame: &mut Frame, app: &App, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Tabs
                Constraint::Min(0),     // Wave list
            ])
            .split(area);

        // Render tabs
        let titles = vec!["Campaign", "Findings", "Logs"];
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("Tabs"))
            .select(app.active_tab() as usize)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow).bold());

        frame.render_widget(tabs, chunks[0]);

        // Render wave list
        self.render_wave_list(frame, app, chunks[1]);
    }

    /// Render the findings tab.
    fn render_findings_tab(&self, frame: &mut Frame, app: &App, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Tabs
                Constraint::Min(0),     // Package list
            ])
            .split(area);

        // Render tabs
        let titles = vec!["Campaign", "Findings", "Logs"];
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("Tabs"))
            .select(app.active_tab() as usize)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow).bold());

        frame.render_widget(tabs, chunks[0]);

        // Render package list
        self.render_package_list(frame, app, chunks[1]);
    }

    /// Render the logs tab.
    fn render_logs_tab(&self, frame: &mut Frame, _app: &App, area: Rect) {
        let logs = Paragraph::new("Logs view (not yet implemented)")
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Logs"))
            .style(Style::default().fg(Color::Gray));

        frame.render_widget(logs, area);
    }

    /// Render the package list in the Findings tab.
    fn render_package_list(&self, frame: &mut Frame, app: &App, area: Rect) {
        let packages: Vec<ListItem> = app.flagged_packages()
            .iter()
            .map(|pkg| {
                let severity = pkg.severity();
                let icon = match severity {
                    Severity::Critical => "🔴",
                    Severity::High => "🟠",
                    Severity::Medium => "🟡",
                    Severity::Low => "🟢",
                };

                let llm_status = if pkg.llm_verdict.is_some() {
                    " [LLM ✓]"
                } else if pkg.llm_explanation.as_ref().map(|s| s == "Analyzing...").unwrap_or(false) {
                    " [LLM ...]"
                } else {
                    ""
                };

                let content = format!("{} {}@{}  Score: {:.1}  {} findings{}",
                    icon,
                    pkg.name,
                    pkg.version,
                    pkg.threat_score,
                    pkg.findings_count,
                    llm_status);

                ListItem::new(content)
            })
            .collect();

        let package_list = if packages.is_empty() {
            List::new(vec![ListItem::new("No flagged packages found")])
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("Flagged Packages (Press Enter for details, 'l' for LLM analysis)"))
                .style(Style::default().fg(Color::Gray))
        } else {
            List::new(packages)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Flagged Packages ({}) - [Enter] Details  [l] LLM Analysis  [↑/↓] Navigate", 
                        app.flagged_packages().len())))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().bg(Color::DarkGray).bold())
        };

        frame.render_widget(package_list, area);
    }

    /// Render the wave list.
    fn render_wave_list(&self, frame: &mut Frame, app: &App, area: Rect) {
        let waves: Vec<ListItem> = if let Some(state) = app.state() {
            state.waves.values()
                .map(|wave| {
                    let icon = match wave.status {
                        WaveStatus::Completed => "✅",
                        WaveStatus::Running => "🟡",
                        WaveStatus::Pending => "⏳",
                        WaveStatus::Failed => "❌",
                        WaveStatus::Skipped => "⏭️",
                    };

                    let status_text = match wave.status {
                        WaveStatus::Completed => format!("{}/{} scanned, {} flagged",
                            wave.packages_scanned, wave.packages_total, wave.packages_flagged),
                        WaveStatus::Running => format!("{}/{} scanned, {} flagged",
                            wave.packages_scanned, wave.packages_total, wave.packages_flagged),
                        WaveStatus::Pending => format!("{}/{} scanned, {} flagged",
                            wave.packages_scanned, wave.packages_total, wave.packages_flagged),
                        WaveStatus::Failed => format!("Failed: {}", wave.error_message.as_deref().unwrap_or("unknown error")),
                        WaveStatus::Skipped => "Skipped".to_string(),
                    };

                    let content = format!("{} {}: {}  {}",
                        icon,
                        wave.id,
                        wave.name,
                        status_text);

                    ListItem::new(content)
                })
                .collect()
        } else {
            vec![ListItem::new("No waves available")]
        };

        let wave_list = List::new(waves)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Waves"))
            .style(Style::default().fg(Color::White));

        frame.render_widget(wave_list, area);
    }

    /// Render the recent events log.
    fn render_event_log(&self, frame: &mut Frame, app: &App, area: Rect) {
        let events: Vec<ListItem> = app.recent_events()
            .iter()
            .rev()  // Show most recent first
            .take(5)
            .map(|event| {
                let timestamp = chrono::Local::now().format("[%H:%M:%S]").to_string();
                let text = match event {
                    CampaignEvent::PackageScanned { package, version, wave_id, .. } => {
                        format!("Wave {}: Package scanned: {}@{}", wave_id, package, version)
                    }
                    CampaignEvent::PackageFlagged { package, version, wave_id, findings_count, .. } => {
                        format!("Wave {}: Package flagged: {}@{} ({} findings)", wave_id, package, version, findings_count)
                    }
                    CampaignEvent::PackageMalicious { package, version, wave_id, .. } => {
                        format!("Wave {}: MALICIOUS: {}@{}", wave_id, package, version)
                    }
                    CampaignEvent::WaveCompleted { wave_id, packages_scanned, packages_flagged, .. } => {
                        format!("Wave {}: Complete ({} scanned, {} flagged)", wave_id, packages_scanned, packages_flagged)
                    }
                    CampaignEvent::WaveStarted { name, packages_total, .. } => {
                        format!("Wave {}: Started ({} packages)", name, packages_total)
                    }
                    CampaignEvent::CampaignStarted { campaign_name, .. } => {
                        format!("Campaign started: {}", campaign_name)
                    }
                    CampaignEvent::CampaignCompleted { total_scanned, total_malicious, .. } => {
                        format!("Campaign completed: {} scanned, {} malicious", total_scanned, total_malicious)
                    }
                    _ => format!("{:?}", event),
                };

                ListItem::new(format!("{} {}", timestamp, text))
            })
            .collect();

        if events.is_empty() {
            let empty = Paragraph::new("No recent events")
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("Recent Events (last 5)"))
                .style(Style::default().fg(Color::Gray));
            frame.render_widget(empty, area);
        } else {
            let event_log = List::new(events)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title("Recent Events (last 5)"))
                .style(Style::default().fg(Color::White));

            frame.render_widget(event_log, area);
        }
    }

    /// Render the help bar.
    fn render_help_bar(&self, frame: &mut Frame, app: &App, area: Rect) {
        // Build help text with command feedback
        let help_text = if let Some(feedback) = app.command_feedback() {
            format!("  {}  |  [p] Pause  [x] Cancel  [r] Resume  [s] Skip  [c] Concurrency  [q] Quit  [Tab] Switch Tab  ", feedback)
        } else {
            "  [p] Pause  [x] Cancel  [r] Resume  [s] Skip  [c] Concurrency  [q] Quit  [Tab] Switch Tab  ".to_string()
        };

        let help_style = if app.command_feedback().is_some() {
            Style::default().fg(Color::Yellow).bold()
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let help = Paragraph::new(help_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)))
            .style(help_style);

        frame.render_widget(help, area);
    }

    /// Render the concurrency adjustment dialog.
    fn render_concurrency_dialog(&self, frame: &mut Frame, dialog: &ConcurrencyDialog) {
        // Create centered dialog area
        let area = centered_rect(50, 30, frame.size());

        // Clear the area behind the dialog
        frame.render_widget(Clear, area);

        // Build dialog content
        let mut lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Adjust Concurrency",
                Style::default().bold().fg(Color::Cyan),
            )),
            Line::from(""),
            Line::from(format!(
                "Current: {} concurrent operations",
                dialog.current_value
            )),
            Line::from(""),
            Line::from("Enter new value (1-100):"),
            Line::from(""),
        ];

        // Show input buffer with cursor
        let input_display = if dialog.input_buffer.is_empty() {
            Span::styled("_", Style::default().fg(Color::Gray))
        } else {
            Span::styled(
                format!("{}_", dialog.input_buffer),
                Style::default().fg(Color::Yellow).bold(),
            )
        };
        lines.push(Line::from(input_display));

        lines.extend(vec![
            Line::from(""),
            Line::from(Span::styled(
                "Press Enter to confirm, Esc to cancel",
                Style::default().fg(Color::Gray),
            )),
            Line::from(""),
        ]);

        let dialog_content = Paragraph::new(lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Concurrency Settings "))
            .style(Style::default().bg(Color::Black));

        frame.render_widget(dialog_content, area);
    }

    /// Render the package detail view.
    fn render_package_detail_view(&self, frame: &mut Frame, app: &App) {
        // Create centered dialog area (larger for detail view)
        let area = centered_rect(70, 70, frame.size());

        // Clear the area behind the dialog
        frame.render_widget(Clear, area);

        // Get the package being viewed
        let pkg = match app.detail_package() {
            Some(p) => p,
            None => return,
        };

        let severity = pkg.severity();

        // Build dialog content
        let mut lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Package Details",
                Style::default().bold().fg(Color::Cyan),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("{}@{}", pkg.name, pkg.version),
                Style::default().bold().fg(Color::White),
            )),
            Line::from(""),
            Line::from(format!("Wave: {}", pkg.wave_id)),
            Line::from(format!("Threat Score: {:.1}", pkg.threat_score)),
            Line::from(Span::styled(
                format!("Severity: {}", severity.as_str()),
                Style::default().fg(severity.color()),
            )),
            Line::from(format!("Findings: {}", pkg.findings_count)),
            Line::from(""),
        ];

        // Add LLM verdict section
        if let Some(ref explanation) = pkg.llm_explanation {
            lines.push(Line::from(Span::styled(
                "LLM Analysis:",
                Style::default().bold().fg(Color::Green),
            )));
            lines.push(Line::from(""));
            
            // Wrap text to fit width
            let max_width = (area.width - 4) as usize;
            for line in explanation.chars().collect::<Vec<_>>().chunks(max_width) {
                lines.push(Line::from(String::from_iter(line.iter())));
            }
            lines.push(Line::from(""));
        } else {
            lines.push(Line::from(Span::styled(
                "LLM Analysis: Not yet analyzed",
                Style::default().fg(Color::Gray),
            )));
            lines.push(Line::from(""));
        }

        // Add help text
        lines.extend(vec![
            Line::from(""),
            Line::from(Span::styled(
                "[l] Run LLM Analysis  [?] Ask Question  [q/Esc] Close  [↑/↓] Scroll",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
        ]);

        let dialog_content = Paragraph::new(lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(severity.color()))
                .title(format!(" {} - Threat Score: {:.1} ", pkg.name, pkg.threat_score)))
            .style(Style::default().bg(Color::Black));

        frame.render_widget(dialog_content, area);
    }

    /// Render the package query dialog.
    fn render_package_query_dialog(&self, frame: &mut Frame, app: &App) {
        // Create centered dialog area
        let area = centered_rect(60, 40, frame.size());

        // Clear the area behind the dialog
        frame.render_widget(Clear, area);

        let dialog = app.package_query_dialog();

        // Get the package being queried
        let pkg = match app.detail_package() {
            Some(p) => p,
            None => return,
        };

        // Build dialog content
        let mut lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("Ask about {}@{}", pkg.name, pkg.version),
                Style::default().bold().fg(Color::Cyan),
            )),
            Line::from(""),
            Line::from("Enter your question:"),
            Line::from(""),
        ];

        // Show input buffer with cursor
        let input_display = if dialog.input_buffer.is_empty() {
            Span::styled("_", Style::default().fg(Color::Gray))
        } else {
            Span::styled(
                format!("{}_", dialog.input_buffer),
                Style::default().fg(Color::Yellow).bold(),
            )
        };
        lines.push(Line::from(input_display));

        lines.push(Line::from(""));

        // Show querying status or response
        if dialog.querying {
            lines.push(Line::from(Span::styled(
                "Querying LLM...",
                Style::default().fg(Color::Yellow),
            )));
        } else if let Some(ref response) = dialog.response {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Response:",
                Style::default().bold().fg(Color::Green),
            )));
            lines.push(Line::from(""));
            
            // Wrap response text
            let max_width = (area.width - 4) as usize;
            for line in response.chars().collect::<Vec<_>>().chunks(max_width) {
                lines.push(Line::from(String::from_iter(line.iter())));
            }
        }

        lines.extend(vec![
            Line::from(""),
            Line::from(Span::styled(
                "Press Enter to submit, Esc to close",
                Style::default().fg(Color::Gray),
            )),
            Line::from(""),
        ]);

        let dialog_content = Paragraph::new(lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Package Query "))
            .style(Style::default().bg(Color::Black));

        frame.render_widget(dialog_content, area);
    }
}

/// Helper to create a centered rectangle.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

impl Default for Ui {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_creation() {
        let ui = Ui::new();
        assert_eq!(ui.title, "GlassWorm Campaign Monitor");
    }
}
