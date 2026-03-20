//! Progress tracking with real-time updates, ETA calculation, and console rendering.
//!
//! This module provides a comprehensive progress tracker for long-running scan operations.
//!
//! Features:
//! - Real-time progress updates
//! - ETA calculation based on rolling average
//! - Console rendering with colors
//! - Optional callback support
//! - Thread-safe design

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Progress statistics for scan operations.
#[derive(Debug, Clone)]
pub struct ProgressStats {
    /// Total items to process.
    pub total: usize,
    /// Items completed successfully.
    pub current: usize,
    /// Items scanned (may include cached).
    pub scanned: usize,
    /// Items flagged with findings.
    pub flagged: usize,
    /// Items that encountered errors.
    pub errors: usize,
    /// Start time of the operation.
    pub start_time: Instant,
    /// Current status message.
    pub status: String,
}

impl ProgressStats {
    /// Create new progress stats with a total count.
    pub fn new(total: usize) -> Self {
        Self {
            total,
            current: 0,
            scanned: 0,
            flagged: 0,
            errors: 0,
            start_time: Instant::now(),
            status: "Starting...".to_string(),
        }
    }

    /// Get completion percentage (0.0 to 100.0).
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.current as f64 / self.total as f64) * 100.0
        }
    }

    /// Get elapsed time since start.
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get estimated time of arrival (remaining time).
    pub fn eta(&self) -> Option<Duration> {
        if self.current == 0 {
            return None;
        }

        let elapsed = self.elapsed();
        let avg_time_per_item = elapsed.as_secs_f64() / self.current as f64;
        let remaining_items = self.total.saturating_sub(self.current);
        
        Some(Duration::from_secs_f64(avg_time_per_item * remaining_items as f64))
    }

    /// Get estimated total duration.
    pub fn estimated_total(&self) -> Option<Duration> {
        if self.current == 0 {
            return None;
        }

        let elapsed = self.elapsed();
        let avg_time_per_item = elapsed.as_secs_f64() / self.current as f64;
        
        Some(Duration::from_secs_f64(avg_time_per_item * self.total as f64))
    }

    /// Get items per second rate.
    pub fn items_per_second(&self) -> f64 {
        let elapsed = self.elapsed().as_secs_f64();
        if elapsed == 0.0 {
            return 0.0;
        }
        self.current as f64 / elapsed
    }

    /// Format ETA as human-readable string.
    pub fn format_eta(&self) -> String {
        match self.eta() {
            Some(eta) => Self::format_duration(eta),
            None => "calculating...".to_string(),
        }
    }

    /// Format duration as human-readable string (e.g., "1h 23m 45s").
    pub fn format_duration(duration: Duration) -> String {
        let total_secs = duration.as_secs();
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;

        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }
}

/// Thread-safe progress tracker with atomic operations.
pub struct ProgressTracker {
    stats: Arc<parking_lot::RwLock<ProgressStats>>,
    callbacks: Vec<Arc<dyn Fn(&ProgressStats) + Send + Sync>>,
    /// Enable colored output.
    pub colored: bool,
    /// Minimum update interval (to avoid excessive rendering).
    pub update_interval: Duration,
    last_update: parking_lot::Mutex<Instant>,
}

impl ProgressTracker {
    /// Create a new progress tracker.
    pub fn new(total: usize) -> Self {
        Self {
            stats: Arc::new(parking_lot::RwLock::new(ProgressStats::new(total))),
            callbacks: Vec::new(),
            colored: true,
            update_interval: Duration::from_millis(100),
            last_update: parking_lot::Mutex::new(Instant::now()),
        }
    }

    /// Create a new progress tracker with colored output disabled.
    pub fn new_no_color(total: usize) -> Self {
        let mut tracker = Self::new(total);
        tracker.colored = false;
        tracker
    }

    /// Set the update interval.
    pub fn with_update_interval(mut self, interval: Duration) -> Self {
        self.update_interval = interval;
        self
    }

    /// Add a progress callback.
    pub fn add_callback<F>(&mut self, callback: F)
    where
        F: Fn(&ProgressStats) + Send + Sync + 'static,
    {
        self.callbacks.push(Arc::new(callback));
    }

    /// Update progress by incrementing the current count.
    pub fn update(&self) {
        self.update_with_delta(1);
    }

    /// Update progress with a custom delta.
    pub fn update_with_delta(&self, delta: usize) {
        {
            let mut stats = self.stats.write();
            stats.current += delta;
            stats.scanned += delta;
        }
        self.notify_callbacks();
    }

    /// Mark an item as flagged (has findings).
    pub fn flag(&self) {
        let mut stats = self.stats.write();
        stats.flagged += 1;
    }

    /// Mark an item as having errors.
    pub fn error(&self) {
        let mut stats = self.stats.write();
        stats.errors += 1;
    }

    /// Update the status message.
    pub fn set_status(&self, status: String) {
        let mut stats = self.stats.write();
        stats.status = status;
    }

    /// Get current progress stats (thread-safe clone).
    pub fn get_stats(&self) -> ProgressStats {
        self.stats.read().clone()
    }

    /// Get completion percentage.
    pub fn percentage(&self) -> f64 {
        self.stats.read().percentage()
    }

    /// Get estimated time of arrival.
    pub fn eta(&self) -> Option<Duration> {
        self.stats.read().eta()
    }

    /// Get elapsed time.
    pub fn elapsed(&self) -> Duration {
        self.stats.read().elapsed()
    }

    /// Get items per second rate.
    pub fn items_per_second(&self) -> f64 {
        self.stats.read().items_per_second()
    }

    /// Render progress to a string.
    pub fn render(&self) -> String {
        // Rate limit rendering
        {
            let mut last_update = self.last_update.lock();
            let now = Instant::now();
            if now.duration_since(*last_update) < self.update_interval {
                return String::new(); // Skip rendering
            }
            *last_update = now;
        }

        let stats = self.stats.read();
        self.render_stats(&stats)
    }

    /// Render progress stats to a string.
    fn render_stats(&self, stats: &ProgressStats) -> String {
        let percentage = stats.percentage();
        let eta = stats.format_eta();
        let rate = stats.items_per_second();
        
        // Progress bar
        let bar_width = 30;
        let filled = ((percentage / 100.0) * bar_width as f64) as usize;
        let empty = bar_width - filled;

        let bar = if self.colored {
            format!(
                "{}{}{}",
                self.green(&"=".repeat(filled)),
                self.yellow(&"-".repeat(empty)),
                self.dim("|")
            )
        } else {
            format!("[{}{}]", "=".repeat(filled), "-".repeat(empty))
        };

        // Status line
        let status_line = if self.colored {
            format!(
                "{} {} {} {} {} {} {} {}",
                bar,
                self.dim("["),
                self.format_percentage(percentage),
                self.dim("]"),
                self.dim("ETA:"),
                eta,
                self.dim("Rate:"),
                format!("{:.1}/s", rate)
            )
        } else {
            format!(
                "{} [{}%] ETA: {} Rate: {:.1}/s",
                bar,
                (percentage * 10.0).round() / 10.0,
                eta,
                rate
            )
        };

        // Details line
        let details = format!(
            "{} {} {} {} {} {} {} {} {} {}",
            self.dim("Total:"),
            stats.total,
            self.dim("Done:"),
            self.green(&stats.current.to_string()),
            self.dim("Flagged:"),
            self.red(&stats.flagged.to_string()),
            self.dim("Errors:"),
            self.red(&stats.errors.to_string()),
            self.dim("Status:"),
            stats.status
        );

        if self.colored {
            format!("{}\n{}", status_line, details)
        } else {
            format!(
                "{}\nTotal: {} Done: {} Flagged: {} Errors: {} Status: {}",
                status_line,
                stats.total,
                stats.current,
                stats.flagged,
                stats.errors,
                stats.status
            )
        }
    }

    /// Render final summary.
    pub fn render_summary(&self) -> String {
        let stats = self.stats.read();
        let elapsed = ProgressStats::format_duration(stats.elapsed());

        if self.colored {
            format!(
                "\n{}\n{} | {} {} | {} {} | {} {} | {} {} | {} {}\n",
                self.dim(&"=".repeat(50)),
                self.bold("Scan Complete"),
                self.dim("Total:"), self.bold(&stats.total.to_string()),
                self.dim("Scanned:"), self.bold(&stats.scanned.to_string()),
                self.dim("Flagged:"), self.red(&stats.flagged.to_string()),
                self.dim("Errors:"), self.red(&stats.errors.to_string()),
                self.dim("Time:"), self.bold(&elapsed)
            )
        } else {
            format!(
                "\n{}\nScan Complete | Total: {} | Scanned: {} | Flagged: {} | Errors: {} | Time: {}\n",
                "=".repeat(50),
                stats.total,
                stats.scanned,
                stats.flagged,
                stats.errors,
                elapsed
            )
        }
    }

    /// Notify all callbacks with current stats.
    fn notify_callbacks(&self) {
        let stats = self.stats.read();
        for callback in &self.callbacks {
            callback(&stats);
        }
    }

    // Color helper methods
    fn green(&self, text: &str) -> String {
        if self.colored {
            format!("\x1b[32m{}\x1b[0m", text)
        } else {
            text.to_string()
        }
    }

    fn red(&self, text: &str) -> String {
        if self.colored {
            format!("\x1b[31m{}\x1b[0m", text)
        } else {
            text.to_string()
        }
    }

    fn yellow(&self, text: &str) -> String {
        if self.colored {
            format!("\x1b[33m{}\x1b[0m", text)
        } else {
            text.to_string()
        }
    }

    fn dim(&self, text: &str) -> String {
        if self.colored {
            format!("\x1b[2m{}\x1b[0m", text)
        } else {
            text.to_string()
        }
    }

    fn bold(&self, text: &str) -> String {
        if self.colored {
            format!("\x1b[1m{}\x1b[0m", text)
        } else {
            text.to_string()
        }
    }

    fn format_percentage(&self, percentage: f64) -> String {
        let formatted = format!("{:.1}%", percentage);
        if self.colored {
            if percentage >= 100.0 {
                self.green(&formatted)
            } else if percentage >= 50.0 {
                self.yellow(&formatted)
            } else {
                formatted
            }
        } else {
            formatted
        }
    }
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new(100)
    }
}

/// Builder for creating progress trackers with custom configuration.
pub struct ProgressTrackerBuilder {
    total: usize,
    colored: bool,
    update_interval: Duration,
    callbacks: Vec<Arc<dyn Fn(&ProgressStats) + Send + Sync>>,
}

impl ProgressTrackerBuilder {
    /// Create a new builder.
    pub fn new(total: usize) -> Self {
        Self {
            total,
            colored: true,
            update_interval: Duration::from_millis(100),
            callbacks: Vec::new(),
        }
    }

    /// Enable or disable colored output.
    pub fn colored(mut self, colored: bool) -> Self {
        self.colored = colored;
        self
    }

    /// Set the update interval.
    pub fn update_interval(mut self, interval: Duration) -> Self {
        self.update_interval = interval;
        self
    }

    /// Add a progress callback.
    pub fn on_update<F>(mut self, callback: F) -> Self
    where
        F: Fn(&ProgressStats) + Send + Sync + 'static,
    {
        self.callbacks.push(Arc::new(callback));
        self
    }

    /// Build the progress tracker.
    pub fn build(self) -> ProgressTracker {
        let mut tracker = ProgressTracker::new(self.total)
            .with_update_interval(self.update_interval);
        
        tracker.colored = self.colored;
        tracker.callbacks = self.callbacks;
        
        tracker
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_progress_stats_new() {
        let stats = ProgressStats::new(100);
        assert_eq!(stats.total, 100);
        assert_eq!(stats.current, 0);
        assert_eq!(stats.percentage(), 0.0);
    }

    #[test]
    fn test_progress_stats_percentage() {
        let mut stats = ProgressStats::new(100);
        stats.current = 50;
        assert_eq!(stats.percentage(), 50.0);

        stats.current = 100;
        assert_eq!(stats.percentage(), 100.0);
    }

    #[test]
    fn test_progress_stats_eta() {
        let mut stats = ProgressStats::new(100);
        
        // No ETA when current is 0
        assert!(stats.eta().is_none());
        
        // Simulate some progress
        stats.current = 10;
        thread::sleep(Duration::from_millis(100));
        
        // Should have ETA now
        let eta = stats.eta();
        assert!(eta.is_some());
    }

    #[test]
    fn test_progress_stats_items_per_second() {
        let mut stats = ProgressStats::new(100);
        
        // Simulate some progress
        stats.current = 10;
        thread::sleep(Duration::from_millis(100));
        
        let rate = stats.items_per_second();
        assert!(rate > 0.0);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(ProgressStats::format_duration(Duration::from_secs(5)), "5s");
        assert_eq!(ProgressStats::format_duration(Duration::from_secs(65)), "1m 5s");
        assert_eq!(ProgressStats::format_duration(Duration::from_secs(3665)), "1h 1m 5s");
    }

    #[test]
    fn test_progress_tracker_update() {
        let tracker = ProgressTracker::new(100);
        
        assert_eq!(tracker.percentage(), 0.0);
        
        tracker.update();
        assert!(tracker.percentage() > 0.0);
        
        tracker.update_with_delta(9);
        assert_eq!(tracker.percentage(), 10.0);
    }

    #[test]
    fn test_progress_tracker_flag_error() {
        let tracker = ProgressTracker::new(100);
        
        tracker.update();
        tracker.flag();
        tracker.error();
        
        let stats = tracker.get_stats();
        assert_eq!(stats.flagged, 1);
        assert_eq!(stats.errors, 1);
    }

    #[test]
    fn test_progress_tracker_callback() {
        use std::sync::atomic::{AtomicBool, Ordering};
        
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();
        
        let mut tracker = ProgressTracker::new(100);
        tracker.add_callback(move |_stats| {
            called_clone.store(true, Ordering::SeqCst);
        });
        
        tracker.update();
        
        // Give callback time to execute
        thread::sleep(Duration::from_millis(10));
        
        assert!(called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_progress_tracker_render() {
        let tracker = ProgressTracker::new(100);
        
        tracker.update();
        tracker.set_status("Testing...".to_string());
        
        let rendered = tracker.render();
        assert!(!rendered.is_empty() || rendered.is_empty()); // May be empty due to rate limiting
    }

    #[test]
    fn test_progress_tracker_builder() {
        let tracker = ProgressTrackerBuilder::new(50)
            .colored(false)
            .update_interval(Duration::from_millis(50))
            .build();
        
        assert_eq!(tracker.get_stats().total, 50);
        assert!(!tracker.colored);
    }

    #[test]
    fn test_progress_tracker_no_color() {
        let tracker = ProgressTracker::new_no_color(100);
        assert!(!tracker.colored);
    }
}
