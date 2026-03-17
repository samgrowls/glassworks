//! Unicode Finding Types
//!
//! This module defines the data structures for representing Unicode attack findings.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt;

pub use crate::decoder::{DecodedPayload, PayloadClass};

/// Severity levels for Unicode findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
            Severity::Critical => "critical",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "critical" => Severity::Critical,
            "high" => Severity::High,
            "medium" => Severity::Medium,
            _ => Severity::Low,
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Category of Unicode attack
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum UnicodeCategory {
    InvisibleCharacter,
    Homoglyph,
    BidirectionalOverride,
    UnicodeTag,
    NormalizationAttack,
    GlasswormPattern,
    EmojiObfuscation,
    /// Dense run of VS codepoints encoding hidden data
    SteganoPayload,
    /// Visible code matching GlassWorm decoder pattern
    DecoderFunction,
    /// VS codepoints after pipe delimiter (npm variant)
    PipeDelimiterStego,
    Unknown,
}

impl UnicodeCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            UnicodeCategory::InvisibleCharacter => "invisible_character",
            UnicodeCategory::Homoglyph => "homoglyph",
            UnicodeCategory::BidirectionalOverride => "bidirectional_override",
            UnicodeCategory::UnicodeTag => "unicode_tag",
            UnicodeCategory::NormalizationAttack => "normalization_attack",
            UnicodeCategory::GlasswormPattern => "glassworm_pattern",
            UnicodeCategory::EmojiObfuscation => "emoji_obfuscation",
            UnicodeCategory::SteganoPayload => "stegano_payload",
            UnicodeCategory::DecoderFunction => "decoder_function",
            UnicodeCategory::PipeDelimiterStego => "pipe_delimiter_stego",
            UnicodeCategory::Unknown => "unknown",
        }
    }
}

/// Represents a source location in a file
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub byte_offset: Option<usize>,
}

impl SourceLocation {
    pub fn new(file: &str, line: usize, column: usize) -> Self {
        Self {
            file: file.to_string(),
            line,
            column,
            byte_offset: None,
        }
    }
}

/// A Unicode security finding
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UnicodeFinding {
    /// File path where the finding was detected
    pub file: String,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Unicode code point value
    pub code_point: u32,
    /// The character itself (may be empty for invisible chars)
    pub character: String,
    /// Raw bytes of the invisible sequence (for decoded payload display)
    pub raw_bytes: Option<String>,
    /// Category of the attack
    pub category: UnicodeCategory,
    /// Severity level
    pub severity: Severity,
    /// Human-readable description
    pub description: String,
    /// Remediation guidance
    pub remediation: String,
    /// CWE ID if applicable (e.g., "CWE-172")
    pub cwe_id: Option<String>,
    /// References to research/advisories
    pub references: Vec<String>,
    /// Optional snippet of surrounding context
    pub context: Option<String>,
    /// Decoded steganographic payload (if applicable)
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub decoded_payload: Option<DecodedPayload>,
}

impl UnicodeFinding {
    pub fn new(
        file: &str,
        line: usize,
        column: usize,
        code_point: u32,
        character: char,
        category: UnicodeCategory,
        severity: Severity,
        description: &str,
        remediation: &str,
    ) -> Self {
        Self {
            file: file.to_string(),
            line,
            column,
            code_point,
            character: character.to_string(),
            raw_bytes: None,
            category,
            severity,
            description: description.to_string(),
            remediation: remediation.to_string(),
            cwe_id: None,
            references: Vec::new(),
            context: None,
            decoded_payload: None,
        }
    }

    pub fn with_cwe_id(mut self, cwe_id: &str) -> Self {
        self.cwe_id = Some(cwe_id.to_string());
        self
    }

    pub fn with_reference(mut self, url: &str) -> Self {
        self.references.push(url.to_string());
        self
    }

    pub fn with_context(mut self, context: &str) -> Self {
        self.context = Some(context.to_string());
        self
    }

    pub fn with_raw_bytes(mut self, bytes: &str) -> Self {
        self.raw_bytes = Some(bytes.to_string());
        self
    }

    pub fn with_decoded_payload(mut self, payload: DecodedPayload) -> Self {
        self.decoded_payload = Some(payload);
        self
    }

    pub fn location(&self) -> SourceLocation {
        SourceLocation::new(&self.file, self.line, self.column)
    }
}

impl fmt::Display for UnicodeFinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}:{}:{} - U+{:04X} ({}) - {}",
            self.severity.as_str().to_uppercase(),
            self.file,
            self.line,
            self.column,
            self.code_point,
            self.category.as_str(),
            self.description
        )
    }
}
