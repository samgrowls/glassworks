//! CLI module for glassware-orchestrator.
//!
//! This module provides the command-line interface using clap.

use clap::{Parser, Subcommand, ValueEnum};

/// Glassware Orchestrator - Security scanning for npm and GitHub.
///
/// A high-performance tool for detecting steganographic payloads,
/// invisible Unicode characters, and bidirectional text attacks
/// in npm packages and GitHub repositories.
#[derive(Parser, Debug)]
#[command(name = "glassware-orchestrator")]
#[command(author = "glassware contributors")]
#[command(version = orchestrator_core::VERSION)]
#[command(about = "Security scanning for npm and GitHub", long_about = None)]
pub struct Cli {
    /// Subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,

    /// Output format.
    #[arg(short, long, value_enum, default_value = "pretty")]
    pub format: OutputFormat,

    /// Minimum severity to report.
    #[arg(short, long, value_enum, default_value = "low")]
    pub severity: SeverityLevel,

    /// Verbose output.
    #[arg(short, long, default_value = "false")]
    pub verbose: bool,

    /// Quiet mode (no output except errors).
    #[arg(short, long, default_value = "false")]
    pub quiet: bool,

    /// Disable colored output.
    #[arg(long, default_value = "false")]
    pub no_color: bool,

    /// Cache database path.
    #[arg(long, default_value = ".glassware-orchestrator-cache.db")]
    pub cache_db: String,

    /// Disable caching.
    #[arg(long, default_value = "false")]
    pub no_cache: bool,

    /// Cache TTL in days.
    #[arg(long, default_value = "7")]
    pub cache_ttl: i64,

    /// Maximum concurrent downloads.
    #[arg(long, default_value = "10")]
    pub concurrency: usize,

    /// Maximum retries per request.
    #[arg(long, default_value = "3")]
    pub max_retries: u32,

    /// npm API rate limit (requests/sec).
    #[arg(long, default_value = "2.0")]
    pub npm_rate_limit: f32,

    /// GitHub API rate limit (requests/sec).
    #[arg(long, default_value = "1.0")]
    pub github_rate_limit: f32,

    /// GitHub API token (optional).
    #[arg(long)]
    pub github_token: Option<String>,

    /// Threat score threshold for marking as malicious.
    #[arg(long, default_value = "5.0")]
    pub threat_threshold: f32,

    /// Enable streaming output (JSON Lines format).
    #[arg(long, default_value = "false")]
    pub streaming: bool,

    /// Enable adversarial testing on scanned packages.
    #[arg(long, default_value = "false")]
    pub adversarial: bool,

    /// Log level (trace, debug, info, warn, error).
    #[arg(long, default_value = "info")]
    pub log_level: String,

    /// Log output file (optional, defaults to stdout).
    #[arg(long)]
    pub log_file: Option<String>,

    /// Output file path (optional, defaults to stdout).
    #[arg(short, long)]
    pub output: Option<String>,
}

/// Available commands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scan npm packages.
    ///
    /// Scans one or more npm packages for security issues.
    ///
    /// Examples:
    ///   glassware-orchestrator scan-npm express lodash
    ///   glassware-orchestrator scan-npm @scope/package
    ScanNpm {
        /// npm package names to scan.
        #[arg(required = true)]
        packages: Vec<String>,
    },

    /// Scan GitHub repositories.
    ///
    /// Scans one or more GitHub repositories for security issues.
    ///
    /// Examples:
    ///   glassware-orchestrator scan-github owner/repo
    ///   glassware-orchestrator scan-github owner/repo --ref main
    ScanGithub {
        /// Repository specifications (owner/repo).
        #[arg(required = true)]
        repos: Vec<String>,

        /// Git reference (branch, tag, or commit).
        #[arg(short, long, default_value = "HEAD")]
        r#ref: Option<String>,
    },

    /// Scan packages from a file.
    ///
    /// Reads package specifications from a file (one per line).
    /// Supports both npm packages and GitHub repos.
    ///
    /// File format:
    ///   # Comments start with #
    ///   express
    ///   lodash
    ///   github:owner/repo
    ///
    /// Examples:
    ///   glassware-orchestrator scan-file packages.txt
    ScanFile {
        /// Path to file containing package list.
        #[arg(required = true)]
        file: String,
    },

    /// Resume an interrupted scan.
    ///
    /// Attempts to resume a previously interrupted scan using cache.
    ///
    /// Examples:
    ///   glassware-orchestrator resume npm --packages express lodash
    ///   glassware-orchestrator resume github --repos owner/repo
    Resume {
        /// Source type to resume.
        #[arg(value_enum)]
        source: ResumeSource,

        /// Packages to resume (for npm).
        #[arg(long, required_if_eq("source", "npm"))]
        packages: Option<Vec<String>>,

        /// Repositories to resume (for github).
        #[arg(long, required_if_eq("source", "github"))]
        repos: Option<Vec<String>>,
    },

    /// Show cache statistics.
    CacheStats {
        /// Clear cache after showing stats.
        #[arg(long, default_value = "false")]
        clear: bool,
    },

    /// Clean up expired cache entries.
    CacheCleanup,
}

/// Output format options.
#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    /// Human-readable pretty print.
    Pretty,

    /// JSON format.
    Json,

    /// JSON Lines format (streaming).
    Jsonl,

    /// SARIF format (for GitHub Advanced Security).
    Sarif,
}

/// Severity level options.
#[derive(ValueEnum, Clone, Debug)]
pub enum SeverityLevel {
    /// Informational findings.
    Info,

    /// Low severity findings.
    Low,

    /// Medium severity findings.
    Medium,

    /// High severity findings.
    High,

    /// Critical severity findings.
    Critical,
}

impl SeverityLevel {
    /// Convert to glassware-core Severity.
    pub fn to_core_severity(&self) -> glassware_core::Severity {
        match self {
            SeverityLevel::Info => glassware_core::Severity::Info,
            SeverityLevel::Low => glassware_core::Severity::Low,
            SeverityLevel::Medium => glassware_core::Severity::Medium,
            SeverityLevel::High => glassware_core::Severity::High,
            SeverityLevel::Critical => glassware_core::Severity::Critical,
        }
    }
}

/// Resume source type.
#[derive(ValueEnum, Clone, Debug)]
pub enum ResumeSource {
    /// npm packages.
    Npm,

    /// GitHub repositories.
    Github,
}

impl Cli {
    /// Parse command-line arguments.
    pub fn parse_args() -> Self {
        Cli::parse()
    }

    /// Parse command-line arguments or exit.
    pub fn try_parse_args() -> Result<Self, clap::Error> {
        Cli::try_parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_scan_npm() {
        let cli = Cli::parse_from([
            "glassware-orchestrator",
            "scan-npm",
            "express",
            "lodash",
        ]);

        match cli.command {
            Commands::ScanNpm { packages } => {
                assert_eq!(packages, vec!["express", "lodash"]);
            }
            _ => panic!("Expected ScanNpm command"),
        }
    }

    #[test]
    fn test_cli_parse_scan_github() {
        let cli = Cli::parse_from([
            "glassware-orchestrator",
            "scan-github",
            "owner/repo",
        ]);

        match cli.command {
            Commands::ScanGithub { repos, r#ref } => {
                assert_eq!(repos, vec!["owner/repo"]);
                assert_eq!(r#ref, Some("HEAD".to_string()));
            }
            _ => panic!("Expected ScanGithub command"),
        }
    }

    #[test]
    fn test_cli_parse_scan_file() {
        let cli = Cli::parse_from([
            "glassware-orchestrator",
            "scan-file",
            "packages.txt",
        ]);

        match cli.command {
            Commands::ScanFile { file } => {
                assert_eq!(file, "packages.txt");
            }
            _ => panic!("Expected ScanFile command"),
        }
    }

    #[test]
    fn test_cli_parse_cache_stats() {
        let cli = Cli::parse_from(["glassware-orchestrator", "cache-stats"]);

        match cli.command {
            Commands::CacheStats { clear } => {
                assert!(!clear);
            }
            _ => panic!("Expected CacheStats command"),
        }
    }

    #[test]
    fn test_severity_conversion() {
        assert_eq!(
            SeverityLevel::Critical.to_core_severity(),
            glassware_core::Severity::Critical
        );
        assert_eq!(
            SeverityLevel::High.to_core_severity(),
            glassware_core::Severity::High
        );
        assert_eq!(
            SeverityLevel::Medium.to_core_severity(),
            glassware_core::Severity::Medium
        );
        assert_eq!(
            SeverityLevel::Low.to_core_severity(),
            glassware_core::Severity::Low
        );
        assert_eq!(
            SeverityLevel::Info.to_core_severity(),
            glassware_core::Severity::Info
        );
    }

    #[test]
    fn test_output_format_enum() {
        let formats = [OutputFormat::Pretty, OutputFormat::Json, OutputFormat::Sarif];
        assert_eq!(formats.len(), 3);
    }
}
