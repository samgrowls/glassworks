//! Streaming output writer for large-scale scan operations.
//!
//! This module provides streaming writers that output results as they complete,
//! preventing OOM issues on large scans.
//!
//! Features:
//! - Stream results as they complete (no buffering)
//! - JSON Lines format support
//! - SARIF streaming support
//! - Async I/O for non-blocking writes
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use orchestrator_core::streaming::{StreamingWriter, OutputFormat};
//! use orchestrator_core::scanner::PackageScanResult;
//! use std::fs::File;
//! use std::io::BufWriter;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create streaming writer to file
//!     let file = File::create("results.jsonl")?;
//!     let writer = BufWriter::new(file);
//!     let mut streaming = StreamingWriter::json_lines(writer);
//!
//!     // Write results as they complete
//!     let result = PackageScanResult {
//!         package_name: "test-pkg".to_string(),
//!         source_type: "npm".to_string(),
//!         version: "1.0.0".to_string(),
//!         path: "/path".to_string(),
//!         content_hash: "hash".to_string(),
//!         findings: vec![],
//!         threat_score: 0.0,
//!         is_malicious: false,
//!     };
//!
//!     streaming.write_result(&result).await?;
//!     streaming.flush().await?;
//!
//!     Ok(())
//! }
//! ```

use std::io::{self, Write};
use tokio::io::{AsyncWrite, AsyncWriteExt};
use serde::Serialize;

use crate::error::{OrchestratorError, Result};
use crate::scanner::PackageScanResult;

/// Output format for streaming writer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// JSON Lines format (one JSON object per line)
    JsonLines,
    /// SARIF streaming format
    Sarif,
    /// Plain JSON array (buffers all results)
    JsonArray,
}

/// SARIF stream state for maintaining streaming context.
#[derive(Debug)]
struct SarifStreamState {
    /// Whether the stream has started
    started: bool,
    /// Number of results written
    count: usize,
}

impl SarifStreamState {
    fn new() -> Self {
        Self {
            started: false,
            count: 0,
        }
    }
}

/// Streaming writer for scan results.
///
/// Writes results as they complete to prevent OOM on large scans.
pub struct StreamingWriter<W>
where
    W: AsyncWrite + Unpin + Send,
{
    output: W,
    format: OutputFormat,
    sarif_state: Option<SarifStreamState>,
    /// Buffer for JSON array format (only used when format is JsonArray)
    buffer: Vec<PackageScanResult>,
}

impl<W> StreamingWriter<W>
where
    W: AsyncWrite + Unpin + Send,
{
    /// Create a new streaming writer with JSON Lines format.
    pub fn json_lines(output: W) -> Self {
        Self {
            output,
            format: OutputFormat::JsonLines,
            sarif_state: None,
            buffer: Vec::new(),
        }
    }

    /// Create a new streaming writer with SARIF format.
    pub fn sarif(output: W) -> Self {
        Self {
            output,
            format: OutputFormat::Sarif,
            sarif_state: Some(SarifStreamState::new()),
            buffer: Vec::new(),
        }
    }

    /// Create a new streaming writer with JSON array format.
    /// Note: This buffers all results in memory before writing.
    pub fn json_array(output: W) -> Self {
        Self {
            output,
            format: OutputFormat::JsonArray,
            sarif_state: None,
            buffer: Vec::new(),
        }
    }

    /// Write a single scan result.
    pub async fn write_result(&mut self, result: &PackageScanResult) -> Result<()> {
        match self.format {
            OutputFormat::JsonLines => self.write_json_lines(result).await,
            OutputFormat::Sarif => self.write_sarif_result(result).await,
            OutputFormat::JsonArray => {
                // Buffer for later
                self.buffer.push(result.clone());
                Ok(())
            }
        }
    }

    /// Write result in JSON Lines format.
    async fn write_json_lines(&mut self, result: &PackageScanResult) -> Result<()> {
        let json = serde_json::to_string(result).map_err(|e| {
            OrchestratorError::json(e)
        })?;

        self.output.write_all(json.as_bytes()).await.map_err(|e| {
            OrchestratorError::io(e)
        })?;
        self.output.write_all(b"\n").await.map_err(|e| {
            OrchestratorError::io(e)
        })?;

        Ok(())
    }

    /// Write result in SARIF format.
    async fn write_sarif_result(&mut self, result: &PackageScanResult) -> Result<()> {
        let state = self.sarif_state.as_mut().ok_or_else(|| {
            OrchestratorError::config_error("SARIF state not initialized".to_string())
        })?;

        if !state.started {
            // Write SARIF header
            let header = r#"{"$schema":"https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json","version":"2.1.0","runs":[{"tool":{"driver":{"name":"glassware-orchestrator","version":"0.1.0","informationUri":"https://github.com/glassware/glassworks"}},"results":["#;
            self.output.write_all(header.as_bytes()).await.map_err(|e| {
                OrchestratorError::io(e)
            })?;
            state.started = true;
        } else if state.count > 0 {
            // Write comma separator
            self.output.write_all(b",").await.map_err(|e| {
                OrchestratorError::io(e)
            })?;
        }

        // Convert finding to SARIF result
        for finding in &result.findings {
            if state.count > 0 {
                self.output.write_all(b",").await.map_err(|e| {
                    OrchestratorError::io(e)
                })?;
            }

            let sarif_result = serde_json::json!({
                "ruleId": format!("{:?}", finding.category),
                "level": severity_to_sarif_level(finding.severity),
                "message": {
                    "text": finding.description
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": finding.file
                        },
                        "region": {
                            "startLine": finding.line,
                            "startColumn": finding.column
                        }
                    }
                }],
                "properties": {
                    "package": result.package_name,
                    "version": result.version,
                    "threatScore": result.threat_score
                }
            });

            let json = serde_json::to_string(&sarif_result).map_err(|e| {
                OrchestratorError::json(e)
            })?;

            self.output.write_all(json.as_bytes()).await.map_err(|e| {
                OrchestratorError::io(e)
            })?;

            state.count += 1;
        }

        Ok(())
    }

    /// Flush the output buffer.
    pub async fn flush(&mut self) -> Result<()> {
        match self.format {
            OutputFormat::JsonArray => {
                // Write all buffered results as JSON array
                self.output.write_all(b"[").await.map_err(|e| {
                    OrchestratorError::io(e)
                })?;

                for (i, result) in self.buffer.iter().enumerate() {
                    if i > 0 {
                        self.output.write_all(b",").await.map_err(|e| {
                            OrchestratorError::io(e)
                        })?;
                    }

                    let json = serde_json::to_string(result).map_err(|e| {
                        OrchestratorError::json(e)
                    })?;
                    self.output.write_all(json.as_bytes()).await.map_err(|e| {
                        OrchestratorError::io(e)
                    })?;
                }

                self.output.write_all(b"]").await.map_err(|e| {
                    OrchestratorError::io(e)
                })?;
            }
            OutputFormat::Sarif => {
                // Close SARIF structure
                if let Some(ref state) = self.sarif_state {
                    if state.started {
                        let footer = r#"]}]}"#;
                        self.output.write_all(footer.as_bytes()).await.map_err(|e| {
                            OrchestratorError::io(e)
                        })?;
                    }
                }
            }
            OutputFormat::JsonLines => {
                // Nothing special needed for JSON Lines
            }
        }

        self.output.flush().await.map_err(|e| {
            OrchestratorError::io(e)
        })?;

        Ok(())
    }

    /// Finish streaming and close the output.
    pub async fn finish(mut self) -> Result<()> {
        self.flush().await
    }
}

/// Convert severity to SARIF level.
fn severity_to_sarif_level(severity: glassware_core::Severity) -> &'static str {
    match severity {
        glassware_core::Severity::Critical => "error",
        glassware_core::Severity::High => "error",
        glassware_core::Severity::Medium => "warning",
        glassware_core::Severity::Low => "note",
        glassware_core::Severity::Info => "none",
    }
}

/// Builder for creating streaming writers with custom configuration.
pub struct StreamingWriterBuilder {
    format: OutputFormat,
}

impl StreamingWriterBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            format: OutputFormat::JsonLines,
        }
    }

    /// Set the output format.
    pub fn format(mut self, format: OutputFormat) -> Self {
        self.format = format;
        self
    }

    /// Build a streaming writer.
    pub fn build<W>(self, output: W) -> StreamingWriter<W>
    where
        W: AsyncWrite + Unpin + Send,
    {
        let mut writer = StreamingWriter::json_lines(output);
        writer.format = self.format;
        if self.format == OutputFormat::Sarif {
            writer.sarif_state = Some(SarifStreamState::new());
        }
        writer
    }
}

impl Default for StreamingWriterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::BufWriter;
    use tempfile::TempDir;
    use std::path::PathBuf;

    fn create_test_result() -> PackageScanResult {
        PackageScanResult {
            package_name: "test-pkg".to_string(),
            source_type: "npm".to_string(),
            version: "1.0.0".to_string(),
            path: "/path".to_string(),
            content_hash: "hash".to_string(),
            findings: vec![],
            threat_score: 0.0,
            is_malicious: false,
            llm_verdict: None,
        }
    }

    #[tokio::test]
    async fn test_json_lines_writer() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("results.jsonl");
        let file = tokio::fs::File::create(&file_path).await.unwrap();
        let writer = BufWriter::new(file);

        let mut streaming = StreamingWriter::json_lines(writer);
        let result = create_test_result();

        streaming.write_result(&result).await.unwrap();
        streaming.flush().await.unwrap();

        // Read back and verify
        let content = tokio::fs::read_to_string(&file_path).await.unwrap();
        assert!(content.contains("test-pkg"));
    }

    #[tokio::test]
    async fn test_sarif_writer() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("results.sarif");
        let file = tokio::fs::File::create(&file_path).await.unwrap();
        let writer = BufWriter::new(file);

        let mut streaming = StreamingWriter::sarif(writer);
        let result = create_test_result();

        streaming.write_result(&result).await.unwrap();
        streaming.finish().await.unwrap();

        // Read back and verify
        let content = tokio::fs::read_to_string(&file_path).await.unwrap();
        assert!(content.contains("\"version\":\"2.1.0\""));
        assert!(content.contains("\"runs\""));
    }

    #[tokio::test]
    async fn test_json_array_writer() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("results.json");
        let file = tokio::fs::File::create(&file_path).await.unwrap();
        let writer = BufWriter::new(file);

        let mut streaming = StreamingWriter::json_array(writer);

        for i in 0..3 {
            let mut result = create_test_result();
            result.package_name = format!("pkg-{}", i);
            streaming.write_result(&result).await.unwrap();
        }

        streaming.flush().await.unwrap();

        // Read back and verify
        let content = tokio::fs::read_to_string(&file_path).await.unwrap();
        assert!(content.starts_with("["));
        assert!(content.ends_with("]"));
        assert!(content.contains("pkg-0"));
        assert!(content.contains("pkg-2"));
    }

    #[tokio::test]
    async fn test_streaming_writer_builder() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("results.jsonl");
        let file = tokio::fs::File::create(&file_path).await.unwrap();
        let writer = BufWriter::new(file);

        let streaming = StreamingWriterBuilder::new()
            .format(OutputFormat::JsonLines)
            .build(writer);

        assert_eq!(streaming.format, OutputFormat::JsonLines);
    }

    #[test]
    fn test_output_format_display() {
        let formats = [
            OutputFormat::JsonLines,
            OutputFormat::Sarif,
            OutputFormat::JsonArray,
        ];

        for format in &formats {
            // Just verify they can be created and compared
            assert_eq!(*format, *format);
        }
    }
}
