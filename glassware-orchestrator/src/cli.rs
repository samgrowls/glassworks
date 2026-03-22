//! CLI module for glassware-orchestrator.

use clap::{Parser, Subcommand, ValueEnum};

/// Glassware Orchestrator - Security scanning for npm and GitHub.
#[derive(Parser, Debug)]
#[command(name = "glassware-orchestrator")]
#[command(author = "glassware contributors")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Security scanning for npm and GitHub", long_about = None)]
pub struct Cli {
    /// Subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,

    /// Output format.
    #[arg(short, long, value_enum, default_value = "pretty", global = true)]
    pub format: OutputFormat,

    /// Minimum severity to report.
    #[arg(short, long, value_enum, default_value = "low")]
    pub severity: SeverityLevel,

    /// Maximum concurrent operations.
    #[arg(short, long, default_value = "10")]
    pub concurrency: usize,

    /// Maximum retries per operation.
    #[arg(long, default_value = "3")]
    pub max_retries: u32,

    /// npm registry rate limit (requests/second).
    #[arg(long, default_value = "10")]
    pub npm_rate_limit: u32,

    /// GitHub API rate limit (requests/second).
    #[arg(long, default_value = "5")]
    pub github_rate_limit: u32,

    /// GitHub token for private repositories.
    #[arg(long)]
    pub github_token: Option<String>,

    /// Enable streaming output (JSON Lines).
    #[arg(long)]
    pub streaming: bool,

    /// Enable adversarial testing.
    #[arg(long)]
    pub adversarial: bool,

    /// Enable LLM analysis.
    #[arg(long)]
    pub llm: bool,

    /// Log level (trace, debug, info, warn, error).
    #[arg(long, default_value = "info")]
    pub log_level: String,

    /// Log output file (optional, defaults to stdout).
    #[arg(long)]
    pub log_file: Option<String>,

    /// Output file path (optional, defaults to stdout).
    #[arg(short, long, global = true)]
    pub output: Option<String>,

    /// Quiet mode (minimal output).
    #[arg(long)]
    pub quiet: bool,

    /// Verbose mode (detailed output, sets log level to debug).
    #[arg(short, long)]
    pub verbose: bool,

    /// Disable colored output.
    #[arg(long)]
    pub no_color: bool,

    /// Checkpoint directory for resume support.
    #[arg(long, default_value = ".glassware-checkpoints")]
    pub checkpoint_dir: String,

    /// Threat score threshold for marking packages as malicious.
    #[arg(long, default_value = "7.0")]
    pub threat_threshold: f32,

    /// Cache database path.
    #[arg(long, default_value = ".glassware-orchestrator-cache.db")]
    pub cache_db: String,

    /// Cache TTL in days.
    #[arg(long, default_value = "7")]
    pub cache_ttl: u64,

    /// Disable caching.
    #[arg(long)]
    pub no_cache: bool,
}

/// Configuration subcommands
#[derive(Subcommand, Debug, Clone)]
pub enum ConfigCommands {
    /// Initialize default configuration file
    Init,

    /// Show current configuration
    Show,

    /// Edit configuration file
    Edit,

    /// Validate configuration syntax
    Validate,

    /// Reset configuration to defaults
    Reset,
}

/// Available commands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scan npm packages.
    ScanNpm {
        /// Package names to scan.
        #[arg(required = true)]
        packages: Vec<String>,

        /// Version policy: last-10, last-180d, all, or comma-separated versions
        #[arg(long)]
        versions: Option<String>,
    },

    /// Scan GitHub repositories.
    ScanGithub {
        /// Repository specifications (owner/repo).
        #[arg(required = true)]
        repos: Vec<String>,

        /// Git reference (branch, tag, or commit).
        #[arg(short, long)]
        r#ref: Option<String>,
    },

    /// Search GitHub repositories.
    SearchGithub {
        /// Search query.
        #[arg(required = true)]
        query: String,

        /// Maximum results to return.
        #[arg(long, default_value = "50")]
        max_results: usize,

        /// Output file to save results (optional).
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Scan packages from a file.
    ScanFile {
        /// Path to file containing package list.
        #[arg(required = true)]
        file: String,
    },

    /// Resume an interrupted scan.
    Resume {
        /// Source type to resume.
        #[arg(value_enum)]
        source: ResumeSource,

        /// Package names to resume (for npm).
        #[arg(long)]
        packages: Option<Vec<String>>,

        /// Repository specs to resume (for GitHub).
        #[arg(long)]
        repos: Option<Vec<String>>,
    },

    /// Show cache statistics.
    CacheStats {
        /// Clear cache after showing stats.
        #[arg(long, default_value = "false")]
        clear: bool,
    },

    /// Sample packages from npm by category.
    SamplePackages {
        /// Categories to sample from (ai-ml, native-build, install-scripts, etc.)
        #[arg(long, required = true)]
        category: Vec<String>,

        /// Number of samples per category
        #[arg(long, default_value = "50")]
        samples: usize,

        /// Output file to save package list
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Clean up expired cache entries.
    CacheCleanup,

    /// Scan tarball files directly.
    ScanTarball {
        /// Paths to .tgz/.tar.gz files to scan.
        #[arg(required = true)]
        files: Vec<String>,
    },

    /// List scan history.
    ScanList {
        /// Filter by status.
        #[arg(long, value_enum)]
        status: Option<ScanStatusFilter>,

        /// Limit number of results.
        #[arg(long, default_value = "20")]
        limit: usize,
    },

    /// Show scan details.
    ScanShow {
        /// Scan ID.
        #[arg(required = true)]
        id: String,
    },

    /// Cancel a running scan.
    ScanCancel {
        /// Scan ID.
        #[arg(required = true)]
        id: String,
    },

    /// Configuration management.
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
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
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl SeverityLevel {
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

/// Scan status filter.
#[derive(ValueEnum, Clone, Debug)]
pub enum ScanStatusFilter {
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl Cli {
    /// Parse command-line arguments.
    pub fn parse_args() -> Self {
        Cli::parse()
    }

    /// Parse command-line arguments or exit.
    #[allow(dead_code)]
    pub fn try_parse_args() -> Result<Self, clap::Error> {
        Cli::try_parse()
    }
}
