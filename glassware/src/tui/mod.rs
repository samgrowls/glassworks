//! TUI module for GlassWorm Campaign System
//!
//! Provides a terminal user interface for monitoring campaign progress
//! using ratatui and crossterm.

pub mod app;
pub mod ui;

pub use app::{App, AppResult};
pub use ui::Ui;
