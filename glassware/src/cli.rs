//! CLI module for glassware.

use clap::{Parser, Subcommand, ValueEnum};

/// Glassware - Unified GlassWare attack detection and campaign orchestration.
#[derive(Parser, Debug)]
#[command(name = "glassware")]
#[command(author = "glassware contributors")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Unified GlassWare attack detection and campaign orchestration", long_about = None)]
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

    /// Enable LLM analysis (Tier 1 - Cerebras fast triage).
    #[arg(long, global = true)]
    pub llm: bool,

    /// Enable deep LLM analysis (Tier 2 - NVIDIA with model fallback).
    /// Uses NVIDIA_API_KEY environment variable.
    #[arg(long, global = true)]
    pub deep_llm: bool,

    /// Log level (trace, debug, info, warn, error).
    #[arg(long, default_value = "info")]
    pub log_level: String,

    /// Log output file (optional, defaults to stdout).
    #[arg(long)]
    pub log_file: Option<String>,

    /// Output file path (optional, defaults to stdout).
    #[arg(short, long)]
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

/// Campaign subcommands
#[derive(Subcommand, Debug, Clone)]
pub enum CampaignCommands {
    /// Run a campaign from a configuration file.
    Run {
        /// Path to campaign configuration file (TOML).
        #[arg(required = true)]
        config: String,

        /// Override concurrency setting.
        #[arg(long)]
        concurrency: Option<usize>,

        /// Override rate limit (requests/second).
        #[arg(long)]
        rate_limit: Option<f32>,

        /// Enable Tier 1 LLM (Cerebras) during scan.
        #[arg(long)]
        llm: bool,

        /// Enable Tier 2 LLM (NVIDIA) for flagged packages.
        #[arg(long)]
        llm_deep: bool,
    },

    /// Resume an interrupted campaign.
    Resume {
        /// Campaign case ID to resume.
        #[arg(required = true)]
        case_id: String,
    },

    /// Show campaign status and progress.
    Status {
        /// Campaign case ID.
        #[arg(required = true)]
        case_id: String,

        /// Show live updating status.
        #[arg(long)]
        live: bool,
    },

    /// Send a command to a running campaign.
    Command {
        /// Campaign case ID.
        #[arg(required = true)]
        case_id: String,

        /// Command to send.
        #[arg(required = true)]
        command: CampaignCommandArg,

        /// Command argument (for some commands).
        #[arg()]
        argument: Option<String>,
    },

    /// List recent campaigns.
    List {
        /// Maximum number of campaigns to show.
        #[arg(long, default_value = "10")]
        limit: usize,

        /// Filter by status.
        #[arg(long, value_enum)]
        status: Option<CampaignStatusFilter>,
    },

    /// Generate a report for a completed campaign.
    Report {
        /// Campaign case ID.
        #[arg(required = true)]
        case_id: String,

        /// Output format.
        #[arg(long, value_enum, default_value = "markdown")]
        format: ReportFormat,

        /// Output file path (defaults to stdout).
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Query a campaign with a natural language question.
    Query {
        /// Campaign case ID.
        #[arg(required = true)]
        case_id: String,

        /// Question to ask about the campaign.
        #[arg(required = true)]
        question: String,
    },

    /// Launch TUI for monitoring a campaign.
    Monitor {
        /// Campaign case ID to monitor.
        #[arg(required = true)]
        case_id: String,
    },

    /// Launch TUI demo with sample data.
    Demo,
}

/// Campaign command types
#[derive(clap::ValueEnum, Debug, Clone)]
pub enum CampaignCommandArg {
    /// Pause campaign execution
    Pause,
    /// Resume paused campaign
    Resume,
    /// Cancel campaign
    Cancel,
    /// Skip current wave
    SkipWave,
    /// Set concurrency level
    SetConcurrency,
    /// Set rate limit
    SetRateLimit,
}

/// Campaign status filter
#[derive(clap::ValueEnum, Debug, Clone)]
pub enum CampaignStatusFilter {
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Report output format
#[derive(clap::ValueEnum, Debug, Clone)]
pub enum ReportFormat {
    Markdown,
    Json,
    Sarif,
}

/// Available commands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run a campaign from configuration.
    #[command(subcommand)]
    Campaign(CampaignCommands),

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
