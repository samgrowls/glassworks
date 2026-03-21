//! Blockchain C2 Module
//!
//! This module provides blockchain-based C2 detection capabilities,
//! specifically for Solana memo-based command and control.
//!
//! ## Components
//!
//! - **G10**: Solana memo C2 parser

pub mod solana;

pub use solana::{parse_memo, SolanaMemoParser, MemoCommand};
