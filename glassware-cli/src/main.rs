//! GlassWare - Unicode Attack Scanner CLI
//!
//! Detects invisible Unicode attacks in source code including:
//! - Steganographic payloads using Variation Selectors
//! - Bidirectional text overrides (Trojan Source)
//! - Homoglyph attacks
//! - GlassWare decoder patterns

use clap::{Parser, ValueEnum};
use colored::Colorize;
use glassware_core::{
    DecodedPayload, DetectionCategory, Finding, PayloadClass, ScanConfig, ScanEngine, Severity,
};
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use rayon::prelude::*;
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Maximum file size to scan (5MB)
const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024;

/// GlassWare - Detect invisible Unicode attacks in source code
#[derive(Parser, Debug)]
#[command(name = "glassware")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Files or directories to scan
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "pretty")]
    format: OutputFormat,

    /// Minimum severity to report
    #[arg(short, long, value_enum, default_value = "low")]
    severity: SeverityLevel,

    /// Suppress output, only set exit code
    #[arg(short, long, default_value = "false")]
    quiet: bool,

    /// Disable colored output
    #[arg(long, default_value = "false")]
    no_color: bool,

    /// File extensions to include (comma-separated)
    #[arg(
        long,
        default_value = "js,mjs,cjs,ts,tsx,jsx,py,rs,go,java,rb,php,sh,bash,zsh,yml,yaml,toml,json,xml,md,txt"
    )]
    extensions: String,

    /// Directories to exclude (comma-separated)
    #[arg(
        long,
        default_value = ".git,node_modules,target,__pycache__,.venv,vendor"
    )]
    exclude: String,

    /// Run LLM analysis on flagged files (requires GLASSWARE_LLM_BASE_URL and
    /// GLASSWARE_LLM_API_KEY environment variables, or a .env file)
    #[cfg(feature = "llm")]
    #[arg(long, default_value = "false")]
    llm: bool,

    /// Enable incremental scanning with caching
    #[arg(long, default_value = ".glassware-cache.json")]
    cache_file: PathBuf,

    /// Cache TTL in days (default: 7)
    #[arg(long, default_value = "7")]
    cache_ttl: u64,

    /// Disable caching
    #[arg(long, default_value = "false")]
    no_cache: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Pretty,
    Json,
    Sarif,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq, PartialOrd)]
enum SeverityLevel {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl SeverityLevel {
    fn matches(&self, severity: &Severity) -> bool {
        let finding_level = match severity {
            Severity::Low | Severity::Info => SeverityLevel::Low,
            Severity::Medium => SeverityLevel::Medium,
            Severity::High => SeverityLevel::High,
            Severity::Critical => SeverityLevel::Critical,
        };
        finding_level >= *self
    }
}

#[derive(Debug, Serialize)]
struct JsonOutput {
    version: String,
    findings: Vec<JsonFinding>,
    summary: JsonSummary,
}

#[derive(Debug, Serialize)]
struct JsonFinding {
    file: String,
    line: usize,
    column: usize,
    severity: String,
    category: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    decoded: Option<JsonDecodedPayload>,
}

#[derive(Debug, Serialize)]
struct JsonDecodedPayload {
    byte_count: usize,
    entropy: f64,
    payload_class: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    preview_hex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    preview_text: Option<String>,
}

#[derive(Debug, Serialize)]
struct JsonSummary {
    files_scanned: usize,
    findings_count: usize,
    duration_ms: u64,
}

fn main() {
    // DEPRECATION WARNING
    eprintln!("{}", "⚠️  DEPRECATION WARNING".yellow().bold());
    eprintln!("{}", "The 'glassware' binary is deprecated and will be removed in v1.0.0.".yellow());
    eprintln!("{}", "Please use 'glassware scan-file' instead:".yellow());
    eprintln!("{}", "  glassware scan-file /path/to/code --format json".yellow());
    eprintln!();

    let args = Args::parse();

    // Disable colors if requested
    if args.no_color {
        colored::control::set_override(false);
    }

    // Collect files to scan
    let files = collect_files(&args);

    if files.is_empty() {
        eprintln!("No files to scan");
        std::process::exit(2);
    }

    // Create ScanConfig from CLI args
    let extensions: Vec<String> = args.extensions.split(',').map(|s| s.to_string()).collect();
    let exclude_patterns: Vec<String> = args.exclude.split(',').map(|s| s.to_string()).collect();
    
    let min_severity = match args.severity {
        SeverityLevel::Critical => Severity::Critical,
        SeverityLevel::High => Severity::High,
        SeverityLevel::Medium => Severity::Medium,
        SeverityLevel::Low => Severity::Low,
        SeverityLevel::Info => Severity::Info,
    };

    let scan_config = ScanConfig {
        extensions,
        exclude_patterns,
        max_file_size: MAX_FILE_SIZE,
        enable_parallel: true,
        parallel_workers: 10,
        enable_dedup: true,
        min_severity,
        #[cfg(feature = "llm")]
        enable_llm: args.llm,
    };

    // Create scan engine with config
    let mut engine_builder = ScanEngine::default_detectors_with_config(scan_config.clone());

    // Enable caching if not disabled
    if !args.no_cache {
        engine_builder = engine_builder.with_cache(args.cache_file.clone(), args.cache_ttl);
    }

    #[cfg(feature = "llm")]
    let engine = Arc::new(engine_builder.with_llm(args.llm));
    #[cfg(not(feature = "llm"))]
    let engine = Arc::new(engine_builder);

    let start = Instant::now();
    
    // Thread-safe collections for findings and errors
    let all_findings = Arc::new(Mutex::new(Vec::new()));
    #[cfg(feature = "llm")]
    let all_llm_verdicts = Arc::new(Mutex::new(Vec::new()));
    
    // Track scan errors with thread-safe counters
    let files_scanned = Arc::new(Mutex::new(0usize));
    let files_skipped = Arc::new(Mutex::new(0usize));
    let read_errors = Arc::new(Mutex::new(Vec::new()));

    // Scan each file in parallel
    files.par_iter().for_each(|file| {
        // Check file size
        if let Ok(metadata) = fs::metadata(file) {
            if metadata.len() > MAX_FILE_SIZE {
                let mut skipped = files_skipped.lock().unwrap();
                *skipped += 1;
                if !args.quiet {
                    eprintln!(
                        "{}: Skipping {} ({}MB, exceeds {}MB limit)",
                        "⊘".yellow(),
                        file.display(),
                        metadata.len() / 1024 / 1024,
                        MAX_FILE_SIZE / 1024 / 1024
                    );
                }
                return;
            }
        }

        // Read file content
        match fs::read_to_string(file) {
            Ok(content) => {
                let mut scanned = files_scanned.lock().unwrap();
                *scanned += 1;

                #[cfg(feature = "llm")]
                {
                    let result = engine.scan_with_llm(file, &content);
                    let mut findings = all_findings.lock().unwrap();
                    findings.extend(result.findings);
                    let mut llm_verdicts = all_llm_verdicts.lock().unwrap();
                    llm_verdicts.extend(result.llm_verdicts);
                }
                #[cfg(not(feature = "llm"))]
                {
                    let result = engine.scan_with_stats(file, &content);
                    let mut findings = all_findings.lock().unwrap();
                    findings.extend(result.findings);
                }
            }
            Err(e) => {
                let mut errors = read_errors.lock().unwrap();
                errors.push((file.clone(), e.to_string()));
                if !args.quiet {
                    eprintln!(
                        "{}: Failed to read {}: {}",
                        "✗".red(),
                        file.display(),
                        e
                    );
                }
            }
        }
    });

    // Extract results from Arc<Mutex<>>
    let all_findings = Arc::try_unwrap(all_findings)
        .unwrap()
        .into_inner()
        .unwrap();
    
    #[cfg(feature = "llm")]
    let all_llm_verdicts = Arc::try_unwrap(all_llm_verdicts)
        .unwrap()
        .into_inner()
        .unwrap();
    
    let files_scanned = Arc::try_unwrap(files_scanned)
        .unwrap()
        .into_inner()
        .unwrap();
    
    let files_skipped = Arc::try_unwrap(files_skipped)
        .unwrap()
        .into_inner()
        .unwrap();
    
    let read_errors = Arc::try_unwrap(read_errors)
        .unwrap()
        .into_inner()
        .unwrap();

    let duration = start.elapsed();

    // Save cache to disk (if enabled)
    if !args.no_cache {
        if let Err(e) = engine.save_cache() {
            eprintln!("Warning: Failed to save cache: {}", e);
        }
    }

    // Get cache statistics (if enabled)
    let cache_stats = engine.cache_stats();

    // Filter by severity
    let filtered_findings: Vec<_> = all_findings
        .into_iter()
        .filter(|f| args.severity.matches(&f.severity))
        .collect();

    // Output results
    let has_findings = !filtered_findings.is_empty();

    if !args.quiet {
        match args.format {
            OutputFormat::Pretty => {
                #[cfg(feature = "llm")]
                {
                    let filtered_llm_verdicts: Vec<_> = all_llm_verdicts
                        .iter()
                        .filter(|v| files.iter().any(|f| f == &v.file_path))
                        .cloned()
                        .collect();
                    output_pretty_with_llm(
                        &filtered_findings,
                        &files,
                        duration,
                        &filtered_llm_verdicts,
                        files_scanned,
                        files_skipped,
                        &read_errors,
                        cache_stats.as_ref(),
                    );
                }
                #[cfg(not(feature = "llm"))]
                {
                    output_pretty(&filtered_findings, &files, duration, files_scanned, files_skipped, &read_errors, cache_stats.as_ref());
                }
            }
            OutputFormat::Json => output_json(&filtered_findings, &files, duration, files_scanned, files_skipped, &read_errors),
            OutputFormat::Sarif => output_sarif(&filtered_findings, &files, duration, files_scanned, files_skipped, &read_errors),
        }
    }

    // Exit code
    if has_findings {
        std::process::exit(1);
    } else {
        std::process::exit(0);
    }
}

/// Collect files to scan from the provided paths
fn collect_files(args: &Args) -> Vec<PathBuf> {
    let extensions: HashSet<&str> = args.extensions.split(',').collect();
    let exclude_dirs: Vec<&str> = args.exclude.split(',').collect();
    
    // Build overrides for file extensions using OverrideBuilder
    let mut override_builder = OverrideBuilder::new(".");
    for ext in &extensions {
        let _ = override_builder.add(&format!("**/*.{}", ext));
    }
    
    // Add exclude patterns (supports glob patterns like **/node_modules, **/dist, etc.)
    // Note: OverrideBuilder uses gitignore semantics where ! means exclude
    for exclude in &exclude_dirs {
        let _ = override_builder.add(&format!("!{}", exclude));
        let _ = override_builder.add(&format!("!**/{}", exclude));
    }
    
    let overrides = override_builder.build().unwrap();
    
    let mut files = Vec::new();
    
    for path in &args.paths {
        if path.is_file() {
            // For individual files, check extension directly
            if should_scan_file(path, &extensions) {
                files.push(path.clone());
            }
        } else if path.is_dir() {
            let walker = WalkBuilder::new(path)
                .standard_filters(true)  // Respects .gitignore and other standard ignores
                .hidden(false)  // Don't skip hidden files
                .overrides(overrides.clone())
                .build();
            
            for result in walker {
                if let Ok(entry) = result {
                    if entry.file_type().map_or(false, |ft| ft.is_file()) {
                        files.push(entry.path().to_path_buf());
                    }
                }
            }
        }
    }
    
    files
}

/// Check if a file should be scanned based on extension
fn should_scan_file(path: &Path, extensions: &HashSet<&str>) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        extensions.contains(ext)
    } else {
        false
    }
}

/// Output results in pretty format
#[cfg(feature = "llm")]
fn output_pretty_with_llm(
    findings: &[Finding],
    _files: &[PathBuf],
    duration: std::time::Duration,
    llm_verdicts: &[glassware_core::llm::LlmFileResult],
    files_scanned: usize,
    files_skipped: usize,
    read_errors: &[(PathBuf, String)],
    cache_stats: Option<&glassware_core::CacheStats>,
) {
    if findings.is_empty() && read_errors.is_empty() {
        println!("{}", "✓ No Unicode attacks detected".green().bold());
        println!(
            "Scanned {} files in {:.2}s",
            files_scanned,
            duration.as_secs_f64()
        );
        if files_skipped > 0 {
            println!(
                "{} {} files skipped (size >5MB)",
                "⊘".yellow(),
                files_skipped
            );
        }
        // Display cache stats
        if let Some(stats) = cache_stats {
            println!();
            println!("{}", "━".repeat(60).dimmed());
            println!("{}", "Cache Statistics".bold().cyan());
            println!("{}", "━".repeat(60).dimmed());
            println!("  Hits:       {} ({:.1}%)", stats.hits, stats.hit_rate());
            println!("  Misses:     {}", stats.misses);
            println!("  Expired:    {}", stats.expired);
            println!("  Loaded:     {}", stats.loaded);
        }
        return;
    }

    // Group findings by severity
    let critical: Vec<_> = findings
        .iter()
        .filter(|f| f.severity == Severity::Critical)
        .collect();
    let high: Vec<_> = findings
        .iter()
        .filter(|f| f.severity == Severity::High)
        .collect();
    let medium: Vec<_> = findings
        .iter()
        .filter(|f| f.severity == Severity::Medium)
        .collect();
    let low: Vec<_> = findings
        .iter()
        .filter(|f| f.severity == Severity::Low || f.severity == Severity::Info)
        .collect();

    for finding in &critical {
        print_finding(finding, "CRITICAL", "red");
    }

    for finding in &high {
        print_finding(finding, "HIGH", "yellow");
    }

    for finding in &medium {
        print_finding(finding, "MEDIUM", "blue");
    }

    for finding in &low {
        print_finding(finding, "LOW", "cyan");
    }

    // Display LLM verdicts
    if !llm_verdicts.is_empty() {
        println!();
        println!("{}", "━".repeat(60).dimmed());
        println!("{}", "LLM Analysis".bold().magenta());
        println!("{}", "━".repeat(60).dimmed());

        for verdict_result in llm_verdicts {
            println!();
            println!(
                "{}",
                format!("File: {}", verdict_result.file_path.display()).dimmed()
            );
            let verdict = &verdict_result.verdict;
            let verdict_str = if verdict.is_malicious {
                "MALICIOUS".red().bold()
            } else {
                "BENIGN".green().bold()
            };
            println!(
                "  Verdict:    {} (confidence: {:.2})",
                verdict_str, verdict.confidence
            );
            if let Some(ref sev) = verdict.reclassified_severity {
                println!("  Severity:   {} (reclassified)", sev);
            }
            println!("  Reasoning:  {}", wrap_text(&verdict.reasoning, 72));
        }
    }

    // Summary
    println!("{}", "━".repeat(60).dimmed());
    println!(
        "{} in {} files ({} critical, {} high, {} medium, {} low)",
        format!("{} findings", findings.len()).bold(),
        files_scanned,
        critical.len().to_string().red(),
        high.len().to_string().yellow(),
        medium.len().to_string().blue(),
        low.len().to_string().cyan()
    );
    println!(
        "Scanned {} files in {:.2}s",
        files_scanned,
        duration.as_secs_f64()
    );
    if files_skipped > 0 {
        println!(
            "{} {} files skipped (size >5MB)",
            "⊘".yellow(),
            files_skipped
        );
    }
    if !read_errors.is_empty() {
        println!(
            "{} {} read errors",
            "✗".red(),
            read_errors.len()
        );
    }

    // Display cache stats
    if let Some(stats) = cache_stats {
        println!();
        println!("{}", "━".repeat(60).dimmed());
        println!("{}", "Cache Statistics".bold().cyan());
        println!("{}", "━".repeat(60).dimmed());
        println!("  Hits:       {} ({:.1}%)", stats.hits, stats.hit_rate());
        println!("  Misses:     {}", stats.misses);
        if stats.expired > 0 {
            println!("  Expired:    {}", stats.expired);
        }
        if stats.loaded > 0 {
            println!("  Loaded:     {}", stats.loaded);
        }
    }
}

/// Output results in pretty format (without LLM)
#[cfg(not(feature = "llm"))]
fn output_pretty(
    findings: &[Finding],
    _files: &[PathBuf],
    duration: std::time::Duration,
    files_scanned: usize,
    files_skipped: usize,
    read_errors: &[(PathBuf, String)],
    cache_stats: Option<&glassware_core::CacheStats>,
) {
    if findings.is_empty() && read_errors.is_empty() {
        println!("{}", "✓ No Unicode attacks detected".green().bold());
        println!(
            "Scanned {} files in {:.2}s",
            files_scanned,
            duration.as_secs_f64()
        );
        if files_skipped > 0 {
            println!("{} {} files skipped (size >5MB)", "⊘".yellow(), files_skipped);
        }
        // Display cache stats
        if let Some(stats) = cache_stats {
            println!();
            println!("{}", "━".repeat(60).dimmed());
            println!("{}", "Cache Statistics".bold().cyan());
            println!("{}", "━".repeat(60).dimmed());
            println!("  Hits:       {} ({:.1}%)", stats.hits, stats.hit_rate());
            println!("  Misses:     {}", stats.misses);
            println!("  Expired:    {}", stats.expired);
            println!("  Loaded:     {}", stats.loaded);
        }
        return;
    }

    // Group findings by severity
    let critical: Vec<_> = findings
        .iter()
        .filter(|f| f.severity == Severity::Critical)
        .collect();
    let high: Vec<_> = findings
        .iter()
        .filter(|f| f.severity == Severity::High)
        .collect();
    let medium: Vec<_> = findings
        .iter()
        .filter(|f| f.severity == Severity::Medium)
        .collect();
    let low: Vec<_> = findings
        .iter()
        .filter(|f| f.severity == Severity::Low || f.severity == Severity::Info)
        .collect();

    for finding in &critical {
        print_finding(finding, "CRITICAL", "red");
    }

    for finding in &high {
        print_finding(finding, "HIGH", "yellow");
    }

    for finding in &medium {
        print_finding(finding, "MEDIUM", "blue");
    }

    for finding in &low {
        print_finding(finding, "LOW", "cyan");
    }

    // Summary
    println!("{}", "━".repeat(60).dimmed());
    println!(
        "{} in {} files ({} critical, {} high, {} medium, {} low)",
        format!("{} findings", findings.len()).bold(),
        files.len(),
        critical.len().to_string().red(),
        high.len().to_string().yellow(),
        medium.len().to_string().blue(),
        low.len().to_string().cyan()
    );
    println!(
        "Scanned {} files in {:.2}s",
        files.len(),
        duration.as_secs_f64()
    );
}

/// Wrap text to a maximum width
#[cfg(feature = "llm")]
fn wrap_text(text: &str, max_width: usize) -> String {
    let mut result = String::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.len() + word.len() + 1 > max_width {
            if !result.is_empty() {
                result.push('\n');
                result.push_str(&" ".repeat(14));
            }
            result.push_str(&current_line);
            current_line.clear();
        }
        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
    }

    if !current_line.is_empty() {
        if !result.is_empty() {
            result.push('\n');
            result.push_str(&" ".repeat(14));
        }
        result.push_str(&current_line);
    }

    result
}

/// Print a single finding in pretty format
fn print_finding(finding: &Finding, level: &str, color: &str) {
    let level_colored = match color {
        "red" => format!("⚠ {}", level).red().bold(),
        "yellow" => format!("⚠ {}", level).yellow().bold(),
        "blue" => format!("⚠ {}", level).blue().bold(),
        "cyan" => format!("⚠ {}", level).cyan().bold(),
        _ => level.to_string().into(),
    };

    println!();
    println!("{}", level_colored);
    println!("  {}", format!("File: {}", finding.file).dimmed());
    println!("  Line: {}", finding.line);
    println!(
        "  Type: {}",
        finding.category.as_str().replace('_', " ").bold()
    );
    println!("  {}", finding.description);

    // Print decoded payload if available
    if let Some(payload) = &finding.decoded_payload {
        print_decoded_payload(payload);
    }

    println!("  {}", finding.remediation.dimmed());
    println!("{}", "---".dimmed());
}

/// Print decoded payload information
fn print_decoded_payload(payload: &DecodedPayload) {
    println!(
        "  {}",
        format!(
            "Hidden: {} invisible codepoints → {} bytes decoded",
            payload.codepoint_count,
            payload.bytes.len()
        )
        .bold()
    );
    println!("  Entropy: {:.2} bits/byte", payload.entropy);
    println!("  Classification: {}", payload.payload_class.description());

    match &payload.payload_class {
        PayloadClass::PlaintextCode => {
            if let Some(text) = payload.text_preview(512) {
                println!();
                println!(
                    "  {}",
                    "┌─ Decoded payload ─────────────────────────────────┐".dimmed()
                );
                for line in text.lines().take(15) {
                    println!("  {} {}", "│".dimmed(), line);
                }
                if text.lines().count() > 15 {
                    println!(
                        "  {} {}",
                        "│".dimmed(),
                        format!("(... {} bytes total)", payload.bytes.len()).dimmed()
                    );
                }
                println!(
                    "  {}",
                    "└────────────────────────────────────────────────────┘".dimmed()
                );
            }
        }
        PayloadClass::EncryptedOrCompressed | PayloadClass::SuspiciousData => {
            println!(
                "  {} {}",
                "Payload preview (first 64 bytes, hex):".dimmed(),
                payload.hex_preview(32)
            );
        }
        PayloadClass::TooSmall => {}
    }
}

/// Output results in JSON format
fn output_json(findings: &[Finding], _files: &[PathBuf], duration: std::time::Duration, files_scanned: usize, _files_skipped: usize, _read_errors: &[(PathBuf, String)]) {
    let json_findings: Vec<JsonFinding> = findings
        .iter()
        .map(|f| JsonFinding {
            file: f.file.clone(),
            line: f.line,
            column: f.column,
            severity: f.severity.as_str().to_string(),
            category: f.category.as_str().to_string(),
            message: f.description.clone(),
            decoded: f.decoded_payload.as_ref().map(|p| JsonDecodedPayload {
                byte_count: p.bytes.len(),
                entropy: p.entropy,
                payload_class: p.payload_class.as_str().to_string(),
                preview_hex: Some(p.hex_preview(32)),
                preview_text: p.text_preview(128),
            }),
        })
        .collect();

    let output = JsonOutput {
        version: env!("CARGO_PKG_VERSION").to_string(),
        findings: json_findings,
        summary: JsonSummary {
            files_scanned,
            findings_count: findings.len(),
            duration_ms: duration.as_millis() as u64,
        },
    };

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

/// Output results in SARIF format
fn output_sarif(findings: &[Finding], _files: &[PathBuf], _duration: std::time::Duration, _files_scanned: usize, _files_skipped: usize, _read_errors: &[(PathBuf, String)]) {
    let sarif = SarifOutput {
        schema: "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json".to_string(),
        version: "2.1.0".to_string(),
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: "glassware".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    information_uri: "https://github.com/PropertySightlines/glassware".to_string(),
                    rules: vec![
                        SarifRule {
                            id: "GW001".to_string(),
                            name: "SteganoPayload".to_string(),
                            short_description: SarifMessage {
                                text: "Steganographic payload using Unicode Variation Selectors".to_string(),
                            },
                            default_configuration: SarifConfig { level: "error".to_string() },
                        },
                        SarifRule {
                            id: "GW002".to_string(),
                            name: "DecoderFunction".to_string(),
                            short_description: SarifMessage {
                                text: "GlassWare-style decoder function pattern".to_string(),
                            },
                            default_configuration: SarifConfig { level: "warning".to_string() },
                        },
                        SarifRule {
                            id: "GW003".to_string(),
                            name: "InvisibleCharacters".to_string(),
                            short_description: SarifMessage {
                                text: "Invisible Unicode characters in source code".to_string(),
                            },
                            default_configuration: SarifConfig { level: "warning".to_string() },
                        },
                        SarifRule {
                            id: "GW004".to_string(),
                            name: "BidiOverride".to_string(),
                            short_description: SarifMessage {
                                text: "Bidirectional text override (Trojan Source)".to_string(),
                            },
                            default_configuration: SarifConfig { level: "error".to_string() },
                        },
                        // GW005: EncryptedPayload
                        SarifRule {
                            id: "GW005".to_string(),
                            name: "EncryptedPayload".to_string(),
                            short_description: SarifMessage {
                                text: "High-entropy encrypted payload combined with dynamic code execution".to_string(),
                            },
                            default_configuration: SarifConfig { level: "error".to_string() },
                        },
                        // GW006: HardcodedKeyDecryption
                        SarifRule {
                            id: "GW006".to_string(),
                            name: "HardcodedKeyDecryption".to_string(),
                            short_description: SarifMessage {
                                text: "Cryptographic decryption with hardcoded key leading to code execution".to_string(),
                            },
                            default_configuration: SarifConfig { level: "error".to_string() },
                        },
                        // GW007: Rc4Pattern
                        SarifRule {
                            id: "GW007".to_string(),
                            name: "Rc4Pattern".to_string(),
                            short_description: SarifMessage {
                                text: "Hand-rolled RC4-like cipher implementation with dynamic execution".to_string(),
                            },
                            default_configuration: SarifConfig { level: "warning".to_string() },
                        },
                        // GW008: HeaderC2
                        SarifRule {
                            id: "GW008".to_string(),
                            name: "HeaderC2".to_string(),
                            short_description: SarifMessage {
                                text: "HTTP header extraction used for C2 communication and payload decryption".to_string(),
                            },
                            default_configuration: SarifConfig { level: "error".to_string() },
                        },
                    ],
                },
            },
            results: findings.iter().map(|f| SarifResult {
                rule_id: match f.category {
                    DetectionCategory::SteganoPayload => "GW001",
                    DetectionCategory::DecoderFunction => "GW002",
                    DetectionCategory::InvisibleCharacter => "GW003",
                    DetectionCategory::BidirectionalOverride => "GW004",
                    DetectionCategory::EncryptedPayload => "GW005",
                    DetectionCategory::HardcodedKeyDecryption => "GW006",
                    DetectionCategory::Rc4Pattern => "GW007",
                    DetectionCategory::HeaderC2 => "GW008",
                    _ => "GW003",
                }.to_string(),
                level: severity_to_sarif_level(&f.severity),
                message: SarifMessage {
                    text: f.description.clone(),
                },
                locations: vec![SarifLocation {
                    physical_location: SarifPhysicalLocation {
                        artifact_location: SarifArtifactLocation {
                            uri: f.file.clone(),
                        },
                        region: Some(SarifRegion {
                            start_line: f.line as u32,
                            start_column: Some(f.column as u32),
                            end_line: None,
                            end_column: None,
                            snippet: f.context.as_ref().map(|c| SarifSnippet { text: c.clone() }),
                        }),
                    },
                }],
            }).collect(),
        }],
    };

    println!("{}", serde_json::to_string_pretty(&sarif).unwrap());
}

fn severity_to_sarif_level(severity: &Severity) -> String {
    match severity {
        Severity::Critical | Severity::High => "error".to_string(),
        Severity::Medium => "warning".to_string(),
        Severity::Low | Severity::Info => "note".to_string(),
    }
}

#[derive(Debug, Serialize)]
struct SarifOutput {
    #[serde(rename = "$schema")]
    schema: String,
    version: String,
    runs: Vec<SarifRun>,
}

#[derive(Debug, Serialize)]
struct SarifRun {
    tool: SarifTool,
    results: Vec<SarifResult>,
}

#[derive(Debug, Serialize)]
struct SarifTool {
    driver: SarifDriver,
}

#[derive(Debug, Serialize)]
struct SarifDriver {
    name: String,
    version: String,
    #[serde(rename = "informationUri")]
    information_uri: String,
    rules: Vec<SarifRule>,
}

#[derive(Debug, Serialize)]
struct SarifRule {
    id: String,
    name: String,
    #[serde(rename = "shortDescription")]
    short_description: SarifMessage,
    #[serde(rename = "defaultConfiguration")]
    default_configuration: SarifConfig,
}

#[derive(Debug, Serialize)]
struct SarifConfig {
    level: String,
}

#[derive(Debug, Serialize)]
struct SarifMessage {
    text: String,
}

#[derive(Debug, Serialize)]
struct SarifResult {
    #[serde(rename = "ruleId")]
    rule_id: String,
    level: String,
    message: SarifMessage,
    locations: Vec<SarifLocation>,
}

#[derive(Debug, Serialize)]
struct SarifLocation {
    #[serde(rename = "physicalLocation")]
    physical_location: SarifPhysicalLocation,
}

#[derive(Debug, Serialize)]
struct SarifPhysicalLocation {
    #[serde(rename = "artifactLocation")]
    artifact_location: SarifArtifactLocation,
    #[serde(rename = "region", skip_serializing_if = "Option::is_none")]
    region: Option<SarifRegion>,
}

#[derive(Debug, Serialize)]
struct SarifArtifactLocation {
    uri: String,
}

#[derive(Debug, Serialize)]
struct SarifRegion {
    #[serde(rename = "startLine")]
    start_line: u32,
    #[serde(rename = "startColumn", skip_serializing_if = "Option::is_none")]
    start_column: Option<u32>,
    #[serde(rename = "endLine", skip_serializing_if = "Option::is_none")]
    end_line: Option<u32>,
    #[serde(rename = "endColumn", skip_serializing_if = "Option::is_none")]
    end_column: Option<u32>,
    #[serde(rename = "snippet", skip_serializing_if = "Option::is_none")]
    snippet: Option<SarifSnippet>,
}

#[derive(Debug, Serialize)]
struct SarifSnippet {
    text: String,
}
