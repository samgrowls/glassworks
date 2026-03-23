//! Glassware Orchestrator CLI
//!
//! Command-line interface for orchestrating security scans across npm and GitHub.

mod cli;
mod tui;

use anyhow::Result;
use cli::{Cli, CampaignCommands, Commands, ConfigCommands, OutputFormat, ResumeSource};
use glassware_orchestrator::{
    Orchestrator, OrchestratorConfig, DownloaderConfig, ScannerConfig,
    PackageScanResult, ScanSummary,
    streaming::StreamingWriter,
    adversarial::AdversarialTester,
    scan_registry::{ScanRegistry, ScanStatus},
    cli_validator, config::GlasswareConfig,
    campaign::{CampaignResult, ReportGenerator, ConfigSummary, CheckpointManager, CampaignCheckpoint},
};
use glassware_core::Severity;
use tracing::{error, info, warn, Level};
use tokio::io::BufWriter;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse_args();

    // Initialize tracing with config
    let log_level = if cli.verbose {
        Level::DEBUG  // Verbose mode overrides log level
    } else {
        match cli.log_level.to_lowercase().as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        }
    };

    let tracing_config = glassware_orchestrator::tracing::TracingConfig {
        level: log_level,
        format: if cli.quiet {
            glassware_orchestrator::tracing::TracingFormat::Minimal
        } else if cli.verbose {
            glassware_orchestrator::tracing::TracingFormat::Pretty
        } else {
            glassware_orchestrator::tracing::TracingFormat::Compact
        },
        output: if let Some(ref log_file) = cli.log_file {
            glassware_orchestrator::tracing::TracingOutput::File(log_file.clone().into())
        } else {
            glassware_orchestrator::tracing::TracingOutput::Stdout
        },
        with_ansi: !cli.no_color && !cli.quiet,
        ..Default::default()
    };

    if let Err(e) = glassware_orchestrator::tracing::init_tracing(&tracing_config) {
        eprintln!("Warning: Failed to initialize tracing: {}", e);
    }

    info!("Glassware Orchestrator v{}", glassware_orchestrator::VERSION);

    // Validate CLI flags
    if let Err(e) = cli_validator::validate_cli(cli.llm, cli.no_cache, &cli.cache_db, cli.concurrency) {
        eprintln!("Error: Invalid flag combination\n");
        for error in &e.errors {
            eprintln!("  × {}", error);
        }
        if !e.warnings.is_empty() {
            eprintln!("\nWarnings:");
            for warning in &e.warnings {
                eprintln!("  ⚠ {}", warning);
            }
        }
        return Err(anyhow::anyhow!("CLI validation failed"));
    }

    // Run command
    match cli.command {
        Commands::Campaign(ref campaign_cmd) => {
            cmd_campaign(&cli, campaign_cmd).await?;
        }
        Commands::ScanNpm { ref packages, ref versions } => {
            cmd_scan_npm(&cli, packages.clone(), versions.clone()).await?;
        }
        Commands::ScanGithub { ref repos, ref r#ref } => {
            cmd_scan_github(&cli, repos.clone(), r#ref.as_deref()).await?;
        }
        Commands::SearchGithub { ref query, ref max_results, ref output } => {
            cmd_search_github(&cli, query.clone(), *max_results, output.as_deref()).await?;
        }
        Commands::ScanFile { ref file } => {
            cmd_scan_file(&cli, file).await?;
        }
        Commands::Resume { ref source, ref packages, ref repos } => {
            cmd_resume(&cli, source.clone(), packages.clone(), repos.clone()).await?;
        }
        Commands::CacheStats { clear } => {
            cmd_cache_stats(&cli, clear).await?;
        }
        Commands::CacheCleanup => {
            cmd_cache_cleanup(&cli).await?;
        }
        Commands::SamplePackages { ref category, ref samples, ref output } => {
            cmd_sample_packages(&cli, category.clone(), *samples, output.as_deref()).await?;
        }
        Commands::ScanList { ref status, ref limit } => {
            cmd_scan_list(&cli, status.clone(), *limit).await?;
        }
        Commands::ScanShow { ref id } => {
            cmd_scan_show(&cli, id).await?;
        }
        Commands::ScanCancel { ref id } => {
            cmd_scan_cancel(&cli, id).await?;
        }
        Commands::ScanTarball { ref files } => {
            cmd_scan_tarball(&cli, files.clone()).await?;
        }
        Commands::Config { ref command } => {
            cmd_config(&cli, command).await?;
        }
    }

    Ok(())
}

/// Scan tarball files command.
async fn cmd_scan_tarball(cli: &Cli, files: Vec<String>) -> Result<()> {
    if files.is_empty() {
        error!("No tarball files specified");
        return Ok(());
    }

    info!("Scanning {} tarball file(s)", files.len());

    // Create scanner with config
    let config = GlasswareConfig::load().unwrap_or_default();
    let mut scanner = glassware_orchestrator::Scanner::with_config(config.into());
    
    // Enable LLM if requested
    if cli.llm {
        info!("LLM analysis enabled");
        scanner = scanner.with_llm();
    }

    let mut total_findings = 0;
    let mut total_malicious = 0;
    let mut results = Vec::new();

    for file_path in &files {
        info!("Scanning tarball: {}", file_path);

        // Check file exists
        let path = std::path::Path::new(file_path);
        if !path.exists() {
            error!("File not found: {}", file_path);
            continue;
        }

        // Extract and scan tarball
        match scanner.scan_tarball(file_path).await {
            Ok(result) => {
                total_findings += result.findings.len();
                if result.is_malicious {
                    total_malicious += 1;
                }
                results.push(result);
            }
            Err(e) => {
                error!("Failed to scan {}: {}", file_path, e);
            }
        }
    }

    // Print summary
    print_scan_summary(&results);

    // Write output if requested
    if let Some(ref output_path) = cli.output {
        write_output(cli, &results, output_path)?;
    }

    // Exit with error code if malicious packages found
    if total_malicious > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// Configuration management command.
async fn cmd_config(_cli: &Cli, command: &ConfigCommands) -> Result<()> {
    match command {
        ConfigCommands::Init => {
            // Create default config file
            let config = GlasswareConfig::default();

            if let Some(config_path) = GlasswareConfig::user_config_file() {
                if config_path.exists() {
                    eprintln!("Configuration file already exists: {:?}", config_path);
                    eprintln!("Use 'glassware-orchestrator config reset' to reset to defaults");
                    return Ok(());
                }

                config.save_user_config()
                    .map_err(|e| anyhow::anyhow!("Failed to save config: {}", e))?;
                println!("Created default configuration: {:?}", config_path);
                println!("Edit this file to customize GlassWorm behavior");
            } else {
                return Err(anyhow::anyhow!("Could not determine config directory"));
            }
        }

        ConfigCommands::Show => {
            // Load and display current config
            let config = GlasswareConfig::load()
                .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;

            println!("Current GlassWorm Configuration:");
            println!("================================");
            println!();
            println!("[scoring]");
            println!("malicious_threshold = {}", config.scoring.malicious_threshold);
            println!("suspicious_threshold = {}", config.scoring.suspicious_threshold);
            println!("category_weight = {}", config.scoring.category_weight);
            println!("critical_weight = {}", config.scoring.critical_weight);
            println!("high_weight = {}", config.scoring.high_weight);
            println!();
            println!("[performance]");
            println!("concurrency = {}", config.performance.concurrency);
            println!("npm_rate_limit = {}", config.performance.npm_rate_limit);
            println!("github_rate_limit = {}", config.performance.github_rate_limit);
            println!("cache_enabled = {}", config.performance.cache_enabled);
            println!("cache_ttl_days = {}", config.performance.cache_ttl_days);
            println!();
            println!("[output]");
            println!("format = {}", config.output.format);
            println!("min_severity = {}", config.output.min_severity);
            println!("color = {}", config.output.color);
            println!();

            // Show config file locations
            if let Some(user_path) = GlasswareConfig::user_config_file() {
                println!("User config: {:?}", user_path);
                if user_path.exists() {
                    println!("  ✓ exists");
                } else {
                    println!("  ✗ not found (using defaults)");
                }
            }

            if let Some(project_path) = GlasswareConfig::project_config_file() {
                println!("Project config: {:?}", project_path);
                if project_path.exists() {
                    println!("  ✓ exists");
                } else {
                    println!("  ✗ not found");
                }
            }
        }

        ConfigCommands::Edit => {
            // Open config file in editor
            let config_path = GlasswareConfig::user_config_file()
                .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

            if !config_path.exists() {
                println!("Configuration file does not exist. Creating default config...");
                GlasswareConfig::default().save_user_config()
                    .map_err(|e| anyhow::anyhow!("Failed to save config: {}", e))?;
            }

            // Try to open in editor
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
            let status = std::process::Command::new(editor)
                .arg(&config_path)
                .status()?;

            if !status.success() {
                return Err(anyhow::anyhow!("Editor exited with error"));
            }
        }

        ConfigCommands::Validate => {
            // Validate config syntax
            let config_path = GlasswareConfig::user_config_file()
                .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

            if !config_path.exists() {
                println!("Configuration file does not exist");
                return Ok(());
            }

            match GlasswareConfig::from_file(&config_path) {
                Ok(config) => {
                    match config.validate() {
                        Ok(()) => {
                            println!("✓ Configuration is valid");
                            println!("  File: {:?}", config_path);
                            println!("  Malicious threshold: {}", config.scoring.malicious_threshold);
                            println!("  Suspicious threshold: {}", config.scoring.suspicious_threshold);
                        }
                        Err(e) => {
                            eprintln!("✗ Configuration validation failed: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("✗ Configuration syntax error: {}", e);
                    std::process::exit(1);
                }
            }
        }

        ConfigCommands::Reset => {
            // Reset config to defaults
            let config_path = GlasswareConfig::user_config_file()
                .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

            if !config_path.exists() {
                println!("Configuration file does not exist");
                return Ok(());
            }

            print!("Are you sure you want to reset configuration to defaults? [y/N] ");
            std::io::stdout().flush().unwrap();

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();

            if input.trim().to_lowercase() == "y" {
                std::fs::remove_file(&config_path)?;
                println!("Configuration reset to defaults");
            } else {
                println!("Reset cancelled");
            }
        }
    }

    Ok(())
}

/// Print scan summary to console.
fn print_scan_summary(results: &[PackageScanResult]) {
    let total_findings: usize = results.iter().map(|r| r.findings.len()).sum();
    let malicious_count = results.iter().filter(|r| r.is_malicious).count();
    let avg_score: f32 = if results.is_empty() {
        0.0
    } else {
        results.iter().map(|r| r.threat_score).sum::<f32>() / results.len() as f32
    };

    println!("\n============================================================");
    println!("SCAN SUMMARY");
    println!("============================================================");
    println!("Total packages scanned: {}", results.len());
    println!("Malicious packages: {}", malicious_count);
    println!("Total findings: {}", total_findings);
    println!("Average threat score: {:.2}", avg_score);
    println!("============================================================");

    if malicious_count > 0 {
        println!("\n⚠️  MALICIOUS PACKAGES DETECTED:");
        for result in results.iter().filter(|r| r.is_malicious) {
            println!("  - {} ({}) [threat score: {:.2}]", 
                result.package_name, result.version, result.threat_score);
        }
        println!("\n❌ Malicious packages detected ({} findings below threshold)", total_findings);
    } else {
        println!("\n✅ No malicious packages detected ({} findings below threshold)", total_findings);
    }
}

/// Write scan results to output file.
fn write_output(cli: &Cli, results: &[PackageScanResult], output_path: &str) -> Result<()> {
    use std::io::Write;
    
    let output = match cli.format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(&serde_json::json!({
                "results": results,
                "summary": {
                    "total_packages": results.len(),
                    "malicious_packages": results.iter().filter(|r| r.is_malicious).count(),
                    "total_findings": results.iter().map(|r| r.findings.len()).sum::<usize>(),
                    "average_threat_score": if results.is_empty() { 0.0 } else {
                        results.iter().map(|r| r.threat_score).sum::<f32>() / results.len() as f32
                    }
                }
            }))?
        }
        _ => {
            // Default to text format
            let mut output = String::new();
            for result in results {
                output.push_str(&format!("{}@{} - Threat Score: {:.2}\n", 
                    result.package_name, result.version, result.threat_score));
                if result.is_malicious {
                    output.push_str("  ⚠️  MALICIOUS\n");
                }
            }
            output
        }
    };

    let mut file = std::fs::File::create(output_path)?;
    file.write_all(output.as_bytes())?;
    
    info!("Results written to {}", output_path);
    Ok(())
}

/// Scan npm packages command.
async fn cmd_scan_npm(cli: &Cli, packages: Vec<String>, versions: Option<String>) -> Result<()> {
    if packages.is_empty() {
        error!("No packages specified");
        return Ok(());
    }

    // Register scan
    let mut registry = ScanRegistry::new(None)?;
    let scan_id = registry.start_scan("scan-npm", &packages, versions.as_deref());
    info!("Started scan: {}", scan_id);

    // If version scanning requested
    if let Some(version_policy) = versions {
        info!("Scanning multiple versions with policy: {}", version_policy);
        
        let policy = glassware_orchestrator::version_scanner::VersionPolicy::from_str(&version_policy)?;
        let version_scanner = glassware_orchestrator::version_scanner::VersionScanner::new()?;
        
        let mut total_findings = 0;
        let mut total_malicious = 0;
        
        for package in &packages {
            info!("Fetching versions for: {}", package);
            
            // Sample versions
            let sampled_versions = version_scanner.sample_versions(package, &policy).await?;
            info!("Found {} versions to scan", sampled_versions.len());
            
            // Scan each version
            let results = version_scanner.scan_versions(package, &sampled_versions).await;
            
            // Process results
            for (version, result) in sampled_versions.iter().zip(results.iter()) {
                match result {
                    Ok(scan_result) => {
                        total_findings += scan_result.findings.len();
                        if scan_result.is_malicious {
                            total_malicious += 1;
                        }
                        info!("  {}@{}: {} findings, threat score: {:.2}",
                            package, version, scan_result.findings.len(), scan_result.threat_score);
                    }
                    Err(e) => {
                        warn!("  {}@{}: Error - {}", package, version, e);
                    }
                }
            }
        }
        
        // Complete scan registration
        registry.complete_scan(&scan_id, total_findings, total_malicious)?;
        
        // Print summary
        println!("\n============================================================");
        println!("VERSION SCAN SUMMARY");
        println!("============================================================");
        println!("Packages scanned: {}", packages.len());
        println!("Total findings: {}", total_findings);
        println!("Malicious versions: {}", total_malicious);
        println!("============================================================");
        
        if total_malicious > 0 {
            println!("\n🚨 Malicious versions detected!");
        } else if total_findings > 0 {
            println!("\n⚠️  Findings detected (review recommended)");
        } else {
            println!("\n✅ No security issues detected");
        }
    } else {
        // Regular single-version scan
        info!("Scanning {} npm packages", packages.len());

        let orchestrator = create_orchestrator(cli).await?;

        let results = if cli.streaming {
            cmd_scan_npm_streaming(cli, &orchestrator, &packages).await?;
            vec![]
        } else {
            let results = orchestrator.scan_npm_packages(&packages).await;

            if cli.adversarial {
                run_adversarial_tests(&results).await?;
            }

            print_results(cli, &results)?;
            results
        };

        // Complete scan registration
        if !cli.streaming {
            let findings = results.iter().filter_map(|r| r.as_ref().ok()).map(|r| r.findings.len()).sum();
            let malicious = results.iter().filter_map(|r| r.as_ref().ok()).filter(|r| r.is_malicious).count();
            registry.complete_scan(&scan_id, findings, malicious)?;
        } else {
            registry.complete_scan(&scan_id, 0, 0)?;
        }
    }

    Ok(())
}

/// Scan npm packages with streaming output.
async fn cmd_scan_npm_streaming(
    cli: &Cli,
    orchestrator: &Orchestrator,
    packages: &[String],
) -> Result<()> {
    let async_writer = tokio::io::stdout();
    let mut streaming = StreamingWriter::json_lines(BufWriter::new(async_writer));

    for (i, package) in packages.iter().enumerate() {
        info!("Scanning package {}/{}: {}", i + 1, packages.len(), package);

        // Scan single package
        let result = orchestrator.scan_npm_package(package).await;

        match result {
            Ok(scan_result) => {
                streaming.write_result(&scan_result).await?;

                // Run adversarial testing if enabled
                if cli.adversarial {
                    if let Some(path) = get_package_path(&scan_result) {
                        run_adversarial_test_on_file(&path).await?;
                    }
                }
            }
            Err(e) => {
                warn!("Failed to scan package {}: {}", package, e);
            }
        }
    }

    streaming.flush().await?;

    Ok(())
}

/// Get path from scan result for adversarial testing.
fn get_package_path(result: &PackageScanResult) -> Option<String> {
    Some(result.path.clone())
}

/// Scan GitHub repositories command.
async fn cmd_scan_github(cli: &Cli, repos: Vec<String>, ref_name: Option<&str>) -> Result<()> {
    if repos.is_empty() {
        error!("No repositories specified");
        return Ok(());
    }

    info!("Scanning {} GitHub repositories", repos.len());

    // Parse repo specifications
    let mut repo_pairs = Vec::new();
    for repo_spec in &repos {
        let parts: Vec<&str> = repo_spec.split('/').collect();
        if parts.len() != 2 {
            error!("Invalid repository specification: {}", repo_spec);
            continue;
        }
        repo_pairs.push((parts[0].to_string(), parts[1].to_string()));
    }

    let orchestrator = create_orchestrator(cli).await?;

    let results = orchestrator.scan_github_repos(&repo_pairs, ref_name).await;

    print_results(cli, &results)?;

    Ok(())
}

/// Search GitHub repositories command.
async fn cmd_search_github(
    cli: &Cli,
    query: String,
    max_results: usize,
    output: Option<&str>,
) -> Result<()> {
    use std::fs::File;
    use std::io::Write;

    info!("Searching GitHub for: '{}' (max {} results)", query, max_results);

    let orchestrator = create_orchestrator(cli).await?;

    // Search GitHub
    let repos = orchestrator.search_github_repos(&query, max_results).await?;

    info!("Found {} repositories", repos.len());

    // Output results
    if let Some(output_path) = output {
        let mut file = File::create(output_path)?;
        for repo in &repos {
            writeln!(file, "{}", repo)?;
        }
        println!("Saved {} repositories to {}", repos.len(), output_path);
    } else {
        println!("# Found {} repositories for '{}'", repos.len(), query);
        for repo in &repos {
            println!("{}", repo);
        }
    }

    Ok(())
}

/// Scan file list command.
async fn cmd_scan_file(cli: &Cli, file_path: &String) -> Result<()> {
    info!("Scanning packages from file: {}", file_path);

    let orchestrator = create_orchestrator(cli).await?;

    let results = orchestrator.scan_file_list(file_path).await?;

    // Flatten results
    let flat_results: Vec<_> = results.into_iter().collect();

    print_results(cli, &flat_results)?;

    Ok(())
}

/// Resume scan command.
async fn cmd_resume(
    cli: &Cli,
    source: ResumeSource,
    packages: Option<Vec<String>>,
    repos: Option<Vec<String>>,
) -> Result<()> {
    info!("Resuming scan from cache");

    let orchestrator = create_orchestrator(cli).await?;

    match source {
        ResumeSource::Npm => {
            if let Some(packages) = packages {
                let results = orchestrator.scan_npm_packages(&packages).await;
                print_results(cli, &results)?;
            }
        }
        ResumeSource::Github => {
            if let Some(repos) = repos {
                let mut repo_pairs = Vec::new();
                for repo_spec in &repos {
                    let parts: Vec<&str> = repo_spec.split('/').collect();
                    if parts.len() != 2 {
                        error!("Invalid repository specification: {}", repo_spec);
                        continue;
                    }
                    repo_pairs.push((parts[0].to_string(), parts[1].to_string()));
                }
                let results = orchestrator.scan_github_repos(&repo_pairs, None).await;
                print_results(cli, &results)?;
            }
        }
    }

    Ok(())
}

/// Cache stats command.
async fn cmd_cache_stats(cli: &Cli, clear: bool) -> Result<()> {
    let orchestrator = create_orchestrator(cli).await?;

    if let Some(stats) = orchestrator.cache_stats().await {
        match cli.format {
            OutputFormat::Pretty => {
                println!("{}", stats);
            }
            OutputFormat::Json | OutputFormat::Jsonl => {
                let json = serde_json::to_string_pretty(&serde_json::json!({
                    "total_entries": stats.total_entries,
                    "expired_entries": stats.expired_entries,
                    "npm_entries": stats.npm_entries,
                    "github_entries": stats.github_entries,
                    "file_entries": stats.file_entries,
                }))?;
                println!("{}", json);
            }
            OutputFormat::Sarif => {
                // SARIF not meaningful for cache stats, use JSON
                let json = serde_json::to_string_pretty(&serde_json::json!({
                    "total_entries": stats.total_entries,
                    "expired_entries": stats.expired_entries,
                    "npm_entries": stats.npm_entries,
                    "github_entries": stats.github_entries,
                    "file_entries": stats.file_entries,
                }))?;
                println!("{}", json);
            }
        }

        if clear {
            orchestrator.clear_cache().await?;
            info!("Cache cleared");
        }
    } else {
        error!("Cache is not enabled");
    }

    Ok(())
}

/// Sample packages command.
async fn cmd_sample_packages(
    _cli: &Cli,
    categories: Vec<String>,
    samples: usize,
    output: Option<&str>,
) -> Result<()> {
    use glassware_orchestrator::sampler::PackageSampler;
    use std::path::Path;

    info!("Sampling {} packages per category from {:?}", samples, categories);

    let sampler = PackageSampler::new()?;
    let category_refs: Vec<&str> = categories.iter().map(|s| s.as_str()).collect();
    let packages: Vec<String> = sampler.sample(&category_refs, samples).await?;

    // Save to file if specified
    if let Some(output_path) = output {
        sampler.save_to_file(&packages, Path::new(output_path))?;
        println!("Saved {} packages to {}", packages.len(), output_path);
    } else {
        // Print to stdout
        println!("# Sampled {} packages from categories: {}", packages.len(), categories.join(", "));
        for package in &packages {
            println!("{}", package);
        }
    }

    Ok(())
}

/// Cache cleanup command.
async fn cmd_cache_cleanup(cli: &Cli) -> Result<()> {
    let orchestrator = create_orchestrator(cli).await?;

    let removed = orchestrator.cleanup_cache().await?;

    if !cli.quiet {
        println!("Cleaned up {} expired cache entries", removed);
    }

    Ok(())
}

/// Scan list command.
async fn cmd_scan_list(cli: &Cli, status: Option<cli::ScanStatusFilter>, limit: usize) -> Result<()> {
    let registry = ScanRegistry::new(None)?;
    
    let status_filter = match status {
        Some(cli::ScanStatusFilter::Running) => Some(ScanStatus::Running),
        Some(cli::ScanStatusFilter::Completed) => Some(ScanStatus::Completed),
        Some(cli::ScanStatusFilter::Failed) => Some(ScanStatus::Failed),
        Some(cli::ScanStatusFilter::Cancelled) => Some(ScanStatus::Cancelled),
        None => None,
    };
    
    let scans = registry.list_scans(status_filter);
    
    match cli.format {
        OutputFormat::Pretty => {
            if scans.is_empty() {
                println!("No scans found");
                return Ok(());
            }
            
            println!("{:<40} {:<12} {:<8} {:<20}", "ID", "Status", "Findings", "Command");
            println!("{}", "-".repeat(90));
            
            for scan in scans.iter().take(limit) {
                println!("{:<40} {:<12} {:<8} {:<20}", 
                    &scan.id[..8], 
                    format!("{:?}", scan.status),
                    scan.findings_count,
                    &scan.command[..20.min(scan.command.len())]
                );
            }
        }
        OutputFormat::Json => {
            let json_scans: Vec<_> = scans.iter().take(limit).map(|s| {
                serde_json::json!({
                    "id": s.id,
                    "status": format!("{:?}", s.status),
                    "command": s.command,
                    "packages": s.packages,
                    "findings_count": s.findings_count,
                    "malicious_count": s.malicious_count,
                    "started_at": s.started_at,
                    "completed_at": s.completed_at,
                })
            }).collect();
            
            println!("{}", serde_json::to_string_pretty(&json_scans)?);
        }
        _ => {
            return Err(anyhow::anyhow!("Only pretty and JSON formats supported for scan list"));
        }
    }
    
    Ok(())
}

/// Scan show command.
async fn cmd_scan_show(cli: &Cli, id: &str) -> Result<()> {
    let registry = ScanRegistry::new(None)?;
    
    if let Some(scan) = registry.get_scan(id) {
        match cli.format {
            OutputFormat::Pretty => {
                println!("Scan ID: {}", scan.id);
                println!("Status: {:?}", scan.status);
                println!("Command: {}", scan.command);
                println!("Started: {}", scan.started_at);
                if let Some(completed) = scan.completed_at {
                    println!("Completed: {}", completed);
                }
                println!("Packages: {}", scan.packages.join(", "));
                if let Some(policy) = &scan.version_policy {
                    println!("Version Policy: {}", policy);
                }
                println!("Findings: {}", scan.findings_count);
                println!("Malicious: {}", scan.malicious_count);
                if let Some(error) = &scan.error {
                    println!("Error: {}", error);
                }
            }
            OutputFormat::Json => {
                let json = serde_json::json!({
                    "id": scan.id,
                    "status": format!("{:?}", scan.status),
                    "command": scan.command,
                    "packages": scan.packages,
                    "version_policy": scan.version_policy,
                    "findings_count": scan.findings_count,
                    "malicious_count": scan.malicious_count,
                    "started_at": scan.started_at,
                    "completed_at": scan.completed_at,
                    "error": scan.error,
                });
                println!("{}", serde_json::to_string_pretty(&json)?);
            }
            _ => {
                return Err(anyhow::anyhow!("Only pretty and JSON formats supported"));
            }
        }
    } else {
        return Err(anyhow::anyhow!("Scan not found: {}", id));
    }
    
    Ok(())
}

/// Scan cancel command.
async fn cmd_scan_cancel(cli: &Cli, id: &str) -> Result<()> {
    let mut registry = ScanRegistry::new(None)?;
    
    if let Some(scan) = registry.get_scan(id) {
        if scan.status != ScanStatus::Running {
            return Err(anyhow::anyhow!("Scan is not running: {}", id));
        }
        
        registry.cancel_scan(id)?;
        
        if !cli.quiet {
            println!("Cancelled scan: {}", id);
        }
    } else {
        return Err(anyhow::anyhow!("Scan not found: {}", id));
    }
    
    Ok(())
}

/// Create orchestrator from CLI options.
async fn create_orchestrator(cli: &Cli) -> Result<Orchestrator> {
    // Load configuration from file
    let glassware_config = GlasswareConfig::load()
        .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;

    // Load GitHub token from CLI or environment
    let github_token = cli.github_token.clone()
        .or_else(|| std::env::var("GITHUB_TOKEN").ok());

    // Use CLI overrides if provided, otherwise use config file values
    let concurrency = cli.concurrency;
    let npm_rate_limit = cli.npm_rate_limit as f32;
    let github_rate_limit = cli.github_rate_limit as f32;
    let threat_threshold = cli.threat_threshold;
    let min_severity = cli.severity.to_core_severity();

    let orchestrator_config = OrchestratorConfig {
        downloader: DownloaderConfig {
            max_retries: cli.max_retries,
            npm_rate_limit,
            github_rate_limit,
            github_token: github_token.clone(),
            max_concurrent: concurrency,
            ..Default::default()
        },
        scanner: ScannerConfig {
            max_concurrent: concurrency,
            min_severity,
            threat_threshold,
            enable_semantic: true,
            enable_llm: cli.llm,
            extensions: vec![
                "js".to_string(), "mjs".to_string(), "cjs".to_string(),
                "ts".to_string(), "tsx".to_string(), "jsx".to_string(),
                "py".to_string(), "json".to_string(),
            ],
            exclude_dirs: vec![
                "node_modules".to_string(), ".git".to_string(),
                "dist".to_string(), "build".to_string(),
            ],
            glassware_config: glassware_config.clone(),
        },
        cache_db_path: if cli.no_cache {
            None
        } else {
            Some(cli.cache_db.clone())
        },
        cache_ttl_days: cli.cache_ttl as i64,
        enable_cache: !cli.no_cache,
        github_token: github_token.clone(),
        enable_checkpoint: true,
        checkpoint_dir: Some(cli.checkpoint_dir.clone()),
        checkpoint_interval: 10,
        retry_config: glassware_orchestrator::retry::RetryConfig::default(),
        npm_rate_limit,
        github_rate_limit,
        #[cfg(feature = "llm")]
        enable_llm: cli.llm || cli.deep_llm,
        #[cfg(feature = "llm")]
        llm_config: if cli.deep_llm {
            // Tier 2: NVIDIA deep analysis with model fallback
            info!("Using NVIDIA deep analysis (Tier 2)");
            glassware_orchestrator::llm::LlmAnalyzerConfig::nvidia_deep_analysis()
        } else if cli.llm {
            // Tier 1: Cerebras fast triage (from environment)
            info!("Using Cerebras fast triage (Tier 1)");
            glassware_orchestrator::llm::LlmAnalyzerConfig::from_env()
        } else {
            None
        },
    };

    let orchestrator = Orchestrator::with_config(orchestrator_config).await?;

    // Set up progress callback if not quiet
    if !cli.quiet {
        let orchestrator = orchestrator.with_progress_callback(move |progress| {
            eprintln!(
                "[{:.1}%] {}",
                progress.percentage(),
                progress.status
            );
        });
        Ok(orchestrator)
    } else {
        Ok(orchestrator)
    }
}

/// Print scan results.
fn print_results(cli: &Cli, results: &[glassware_orchestrator::Result<PackageScanResult>]) -> Result<()> {
    if cli.quiet {
        // Quiet mode: only print summary
        let success_count = results.iter().filter(|r| r.is_ok()).count();
        let error_count = results.iter().filter(|r| r.is_err()).count();
        eprintln!("Completed: {} succeeded, {} failed", success_count, error_count);
        return Ok(());
    }

    let mut successful_results = Vec::new();
    let mut errors = Vec::new();

    for result in results {
        match result {
            Ok(scan_result) => successful_results.push(scan_result),
            Err(e) => errors.push(e),
        }
    }

    // Print summary
    let summary = ScanSummary::from_results(&successful_results.iter().map(|r| (*r).clone()).collect::<Vec<_>>());

    match cli.format {
        OutputFormat::Pretty => {
            print_pretty_summary(&summary);

            if !errors.is_empty() {
                println!("\nErrors:");
                for error in &errors {
                    println!("  - {}", error);
                }
            }

            // Print malicious packages
            let malicious: Vec<_> = successful_results
                .iter()
                .filter(|r| r.is_malicious)
                .collect();

            if !malicious.is_empty() {
                println!("\n🚨 Malicious Packages Detected:");
                for result in &malicious {
                    println!(
                        "  - {} ({}) [threat score: {:.2}]",
                        result.package_name, result.version, result.threat_score
                    );

                    // Print findings
                    for finding in &result.findings {
                        println!(
                            "    [{}] {}:{} - {}",
                            finding.severity,
                            finding.file,
                            finding.line,
                            finding.description
                        );
                    }
                }
            } else if successful_results.iter().all(|r| !r.is_malicious) && summary.total_findings > 0 {
                println!("\n✅ No malicious packages detected ({} findings below threshold)", summary.total_findings);
            } else if summary.total_findings == 0 {
                println!("\n✅ No security issues detected");
            }
        }

        OutputFormat::Json => {
            let output = serde_json::json!({
                "summary": {
                    "total_packages": summary.total_packages,
                    "malicious_packages": summary.malicious_packages,
                    "total_findings": summary.total_findings,
                    "average_threat_score": summary.average_threat_score,
                    "findings_by_severity": summary.findings_by_severity
                        .iter()
                        .map(|(k, v)| (format!("{:?}", k), *v))
                        .collect::<std::collections::HashMap<_, _>>(),
                    "findings_by_category": summary.findings_by_category
                        .iter()
                        .map(|(k, v)| (format!("{:?}", k), *v))
                        .collect::<std::collections::HashMap<_, _>>(),
                },
                "results": successful_results.iter().map(|r| {
                    serde_json::json!({
                        "package_name": r.package_name,
                        "version": r.version,
                        "source_type": r.source_type,
                        "threat_score": r.threat_score,
                        "is_malicious": r.is_malicious,
                        "findings": r.findings.iter().map(|f| {
                            serde_json::json!({
                                "severity": format!("{:?}", f.severity),
                                "category": format!("{:?}", f.category),
                                "file_path": f.file,
                                "line": f.line,
                                "column": f.column,
                                "description": f.description,
                            })
                        }).collect::<Vec<_>>(),
                    })
                }).collect::<Vec<_>>(),
                "errors": errors.iter().map(|e| e.to_string()).collect::<Vec<_>>(),
            });

            println!("{}", serde_json::to_string_pretty(&output)?);
        }

        OutputFormat::Jsonl => {
            // Output each result as a JSON line
            for result in successful_results {
                let json = serde_json::json!({
                    "package_name": result.package_name,
                    "version": result.version,
                    "source_type": result.source_type,
                    "threat_score": result.threat_score,
                    "is_malicious": result.is_malicious,
                    "findings": result.findings.iter().map(|f| {
                        serde_json::json!({
                            "severity": format!("{:?}", f.severity),
                            "category": format!("{:?}", f.category),
                            "file_path": f.file,
                            "line": f.line,
                            "column": f.column,
                            "description": f.description,
                        })
                    }).collect::<Vec<_>>(),
                });
                println!("{}", serde_json::to_string(&json)?);
            }
            // Output errors as JSON lines
            for error in &errors {
                let json = serde_json::json!({
                    "error": error.to_string()
                });
                eprintln!("{}", serde_json::to_string(&json)?);
            }
        }

        OutputFormat::Sarif => {
            // Convert to SARIF format
            let sarif = build_sarif(&successful_results, &errors)?;
            println!("{}", serde_json::to_string_pretty(&sarif)?);
        }
    }

    Ok(())
}

/// Print pretty summary.
fn print_pretty_summary(summary: &ScanSummary) {
    println!("\n{}", "=".repeat(60));
    println!("SCAN SUMMARY");
    println!("{}", "=".repeat(60));
    println!("Total packages scanned: {}", summary.total_packages);
    println!("Malicious packages: {}", summary.malicious_packages);
    println!("Total findings: {}", summary.total_findings);
    println!("Average threat score: {:.2}", summary.average_threat_score);

    if !summary.findings_by_severity.is_empty() {
        println!("\nFindings by severity:");
        let severities = [
            ("Critical", "Critical"),
            ("High", "High"),
            ("Medium", "Medium"),
            ("Low", "Low"),
            ("Info", "Info"),
        ];
        for (key, label) in &severities {
            if let Some(&count) = summary.findings_by_severity.get(*key) {
                println!("  {}: {}", label, count);
            }
        }
    }

    if !summary.findings_by_category.is_empty() {
        println!("\nFindings by category:");
        for (category, count) in &summary.findings_by_category {
            println!("  {:?}: {}", category, count);
        }
    }

    println!("{}", "=".repeat(60));
}

/// Build SARIF output.
fn build_sarif(
    results: &[&PackageScanResult],
    errors: &[&glassware_orchestrator::error::OrchestratorError],
) -> Result<serde_json::Value> {
    let mut sarif_results = Vec::new();

    for result in results {
        for finding in &result.findings {
            sarif_results.push(serde_json::json!({
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
                    "threatScore": result.threat_score,
                    "severity": format!("{:?}", finding.severity)
                }
            }));
        }
    }

    Ok(serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "glassware-orchestrator",
                    "version": glassware_orchestrator::VERSION,
                    "informationUri": "https://github.com/glassware/glassworks",
                    "rules": [
                        {
                            "id": "InvisibleCharacter",
                            "name": "InvisibleCharacter",
                            "shortDescription": {
                                "text": "Detects invisible Unicode characters"
                            },
                            "defaultConfiguration": {
                                "level": "warning"
                            }
                        },
                        {
                            "id": "Homoglyph",
                            "name": "Homoglyph",
                            "shortDescription": {
                                "text": "Detects confusable characters"
                            },
                            "defaultConfiguration": {
                                "level": "warning"
                            }
                        },
                        {
                            "id": "BidirectionalOverride",
                            "name": "BidirectionalOverride",
                            "shortDescription": {
                                "text": "Detects bidirectional text overrides"
                            },
                            "defaultConfiguration": {
                                "level": "error"
                            }
                        }
                    ]
                }
            },
            "results": sarif_results,
            "invocations": [{
                "executionSuccessful": true,
                "toolExecutionNotifications": errors.iter().map(|e| {
                    serde_json::json!({
                        "level": "error",
                        "message": {
                            "text": e.to_string()
                        }
                    })
                }).collect::<Vec<_>>()
            }]
        }]
    }))
}

/// Convert severity to SARIF level.
fn severity_to_sarif_level(severity: Severity) -> &'static str {
    match severity {
        Severity::Critical => "error",
        Severity::High => "error",
        Severity::Medium => "warning",
        Severity::Low => "note",
        Severity::Info => "none",
    }
}

/// Run adversarial tests on all scanned results.
async fn run_adversarial_tests(
    results: &[glassware_orchestrator::Result<PackageScanResult>],
) -> Result<()> {
    info!("Running adversarial tests on scanned packages...");

    let _tester = AdversarialTester::new()?;
    let mut high_risk_count = 0;

    for result in results {
        if let Ok(scan_result) = result {
            if let Some(report) = run_adversarial_test_on_file(&scan_result.path).await? {
                if report.is_high_risk() {
                    high_risk_count += 1;
                    warn!(
                        "🚨 Package {} flagged as high-risk (evasion rate: {:.2}%)",
                        scan_result.package_name,
                        report.evasion_rate * 100.0
                    );
                }
            }
        }
    }

    if high_risk_count > 0 {
        warn!("{} packages flagged as high-risk by adversarial testing", high_risk_count);
    } else {
        info!("✅ All packages passed adversarial testing");
    }

    Ok(())
}

/// Run adversarial test on a single file.
async fn run_adversarial_test_on_file(
    path: &str,
) -> Result<Option<glassware_orchestrator::adversarial::AdversarialReport>> {
    use std::path::Path as StdPath;

    let path_obj = StdPath::new(path);

    if !path_obj.exists() {
        warn!("Path does not exist for adversarial testing: {}", path);
        return Ok(None);
    }

    let tester = AdversarialTester::new()?;
    let report = tester.test_package(path).await?;

    if report.is_high_risk() {
        for evasion in &report.high_risk_evasions {
            warn!("  High-risk evasion: {}", evasion);
        }
    }

    Ok(Some(report))
}

/// Campaign command handler.
async fn cmd_campaign(cli: &Cli, campaign_cmd: &cli::CampaignCommands) -> Result<()> {
    use cli::CampaignCommands;
    use glassware_orchestrator::campaign::{
        CampaignConfig, CampaignExecutor, EventBus, StateManager, CommandChannel,
    };

    match campaign_cmd {
        CampaignCommands::Run { config, concurrency, rate_limit, llm, llm_deep } => {
            cmd_campaign_run(cli, config, concurrency, rate_limit, *llm, *llm_deep).await?;
        }
        CampaignCommands::Resume { case_id } => {
            cmd_campaign_resume(cli, case_id).await?;
        }
        CampaignCommands::Status { case_id, live } => {
            cmd_campaign_status(cli, case_id, *live).await?;
        }
        CampaignCommands::Command { case_id, command, argument } => {
            cmd_campaign_command(cli, case_id, command, argument.as_deref()).await?;
        }
        CampaignCommands::List { limit, status } => {
            cmd_campaign_list(cli, *limit, status.clone()).await?;
        }
        CampaignCommands::Report { case_id, format, output } => {
            cmd_campaign_report(cli, case_id, format, output.as_deref()).await?;
        }
        CampaignCommands::Monitor { case_id } => {
            cmd_campaign_monitor(case_id).await?;
        }
        CampaignCommands::Demo => {
            cmd_tui_demo().await?;
        }
    }

    Ok(())
}

/// Run a campaign from configuration.
async fn cmd_campaign_run(
    cli: &Cli,
    config_path: &str,
    concurrency: &Option<usize>,
    rate_limit: &Option<f32>,
    _llm: bool,
    _llm_deep: bool,
) -> Result<()> {
    use glassware_orchestrator::campaign::{CampaignConfig, CampaignExecutor, EventBus, StateManager, CommandChannel};
    use std::path::Path;

    info!("Loading campaign configuration: {}", config_path);

    // Load and validate configuration
    let config = CampaignConfig::from_file(Path::new(config_path))
        .map_err(|e| anyhow::anyhow!("Failed to load campaign config: {}", e))?;

    info!("Loaded campaign '{}' with {} waves", config.campaign.name, config.waves.len());

    // Apply CLI overrides
    let mut config = config;
    if let Some(c) = concurrency {
        config.settings.concurrency = *c;
    }
    if let Some(r) = rate_limit {
        config.settings.rate_limit_npm = *r;
        config.settings.rate_limit_github = *r;
    }

    // Create campaign components
    let case_id = format!("{}-{}", config.campaign.name.to_lowercase().replace(' ', "-"), chrono::Utc::now().format("%Y%m%d-%H%M%S"));
    let event_bus = EventBus::new(512);
    let state = StateManager::new(&case_id, &config.campaign.name, event_bus.clone());
    let command_channel = CommandChannel::new();

    // Set config hash
    let config_json = serde_json::to_string(&config).unwrap_or_default();
    let config_hash = format!("{:x}", md5::compute(config_json));
    state.set_config_hash(config_hash).await;

    // Create executor
    let executor = CampaignExecutor::new(config, state, event_bus.clone(), command_channel).await;

    // Run campaign
    info!("🚀 Starting campaign execution...");
    
    match executor.run().await {
        Ok(result) => {
            println!("\n{}", "=".repeat(60));
            println!("CAMPAIGN COMPLETE");
            println!("{}", "=".repeat(60));
            println!("Case ID: {}", result.case_id);
            println!("Status: {:?}", result.status);
            println!("Duration: {:?}", result.duration);
            println!("Packages scanned: {}", result.total_scanned);
            println!("Packages flagged: {}", result.total_flagged);
            println!("Malicious packages: {}", result.total_malicious);
            println!("{}", "=".repeat(60));

            if result.total_malicious > 0 {
                eprintln!("\n🚨 Malicious packages detected!");
                std::process::exit(1);
            }
        }
        Err(e) => {
            error!("Campaign failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Resume a campaign.
async fn cmd_campaign_resume(_cli: &Cli, case_id: &str) -> Result<()> {
    use glassware_orchestrator::campaign::{
        CampaignConfig, CampaignExecutor, EventBus, StateManager, CommandChannel, CheckpointManager,
    };
    use std::path::Path;

    let checkpoint_db = Path::new(".glassware-checkpoints.db");
    let checkpoint_mgr = CheckpointManager::new(checkpoint_db)
        .map_err(|e| anyhow::anyhow!("Failed to open checkpoint database: {}", e))?;

    // Load checkpoint
    let checkpoint = checkpoint_mgr.load_checkpoint(case_id)
        .map_err(|e| anyhow::anyhow!("Failed to load checkpoint: {}", e))?
        .ok_or_else(|| anyhow::anyhow!("Campaign not found: {}", case_id))?;

    info!("Resuming campaign '{}' from checkpoint", case_id);
    info!("Completed waves: {:?}", checkpoint.completed_waves);
    info!("Current wave: {:?}", checkpoint.current_wave);

    // Log which waves will be skipped
    if !checkpoint.completed_waves.is_empty() {
        for wave_id in &checkpoint.completed_waves {
            info!("Skipping wave {} (completed)", wave_id);
        }
    }

    // Reload config
    let config: CampaignConfig = serde_json::from_str(&checkpoint.config_json)
        .map_err(|e| anyhow::anyhow!("Failed to parse campaign config: {}", e))?;

    // Create campaign components
    let event_bus = EventBus::new(512);
    let state = StateManager::new(&checkpoint.case_id, &checkpoint.campaign_name, event_bus.clone());
    let command_channel = CommandChannel::new();

    // Restore state from checkpoint
    state.set_config_hash(checkpoint.config_json.clone()).await;

    // Create executor with skip list for completed waves
    let executor = CampaignExecutor::with_skip_waves(
        config,
        state,
        event_bus.clone(),
        command_channel,
        checkpoint.completed_waves.clone(),
    ).await;

    // Run campaign (will skip completed waves)
    info!("🚀 Resuming campaign execution...");

    match executor.run().await {
        Ok(result) => {
            println!("\n{}", "=".repeat(60));
            println!("CAMPAIGN COMPLETE (Resumed)");
            println!("{}", "=".repeat(60));
            println!("Case ID: {}", result.case_id);
            println!("Status: {:?}", result.status);
            println!("Duration: {:?}", result.duration);
            println!("Packages scanned: {}", result.total_scanned);
            println!("Packages flagged: {}", result.total_flagged);
            println!("Malicious packages: {}", result.total_malicious);
            println!("{}", "=".repeat(60));

            if result.total_malicious > 0 {
                eprintln!("\n🚨 Malicious packages detected!");
                std::process::exit(1);
            }
        }
        Err(e) => {
            error!("Campaign failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Show campaign status.
async fn cmd_campaign_status(cli: &Cli, case_id: &str, _live: bool) -> Result<()> {
    use glassware_orchestrator::campaign::StateManager;
    
    // For now, show scan registry info
    let registry = glassware_orchestrator::scan_registry::ScanRegistry::new(None)?;
    
    if let Some(scan) = registry.get_scan(case_id) {
        match cli.format {
            cli::OutputFormat::Pretty => {
                println!("Campaign: {}", case_id);
                println!("Status: {:?}", scan.status);
                println!("Command: {}", scan.command);
                println!("Findings: {}", scan.findings_count);
                println!("Malicious: {}", scan.malicious_count);
                println!("Started: {}", scan.started_at);
                if let Some(completed) = scan.completed_at {
                    println!("Completed: {}", completed);
                }
            }
            cli::OutputFormat::Json => {
                let json = serde_json::json!({
                    "case_id": case_id,
                    "status": format!("{:?}", scan.status),
                    "command": scan.command,
                    "findings_count": scan.findings_count,
                    "malicious_count": scan.malicious_count,
                    "started_at": scan.started_at,
                    "completed_at": scan.completed_at,
                });
                println!("{}", serde_json::to_string_pretty(&json)?);
            }
            _ => anyhow::bail!("Only pretty and JSON formats supported"),
        }
    } else {
        anyhow::bail!("Campaign not found: {}", case_id);
    }

    Ok(())
}

/// Send command to running campaign.
async fn cmd_campaign_command(_cli: &Cli, _case_id: &str, _command: &cli::CampaignCommandArg, _argument: Option<&str>) -> Result<()> {
    // Live command steering requires a running campaign handle
    // This will be implemented with the TUI in Phase 3
    // For now, provide helpful message
    
    eprintln!("Live command steering requires a running campaign session.");
    eprintln!();
    eprintln!("Commands will be available in Phase 3 with the TUI:");
    eprintln!("  - Pause/Resume campaign execution");
    eprintln!("  - Cancel with checkpoint");
    eprintln!("  - Skip waves");
    eprintln!("  - Adjust concurrency/rate limits");
    eprintln!();
    eprintln!("For now, use Ctrl+C to interrupt and 'campaign resume' to continue.");
    
    Ok(())
}

/// List recent campaigns.
async fn cmd_campaign_list(cli: &Cli, limit: usize, _status: Option<cli::CampaignStatusFilter>) -> Result<()> {
    let registry = glassware_orchestrator::scan_registry::ScanRegistry::new(None)?;

    let scans = registry.list_scans(None);
    let scans: Vec<_> = scans.into_iter().take(limit).collect();

    match cli.format {
        cli::OutputFormat::Pretty => {
            if scans.is_empty() {
                println!("No campaigns found");
                return Ok(());
            }

            println!("{:<40} {:<12} {:<8} {:<8} {:<20}", "Case ID", "Status", "Findings", "Malicious", "Started");
            println!("{}", "-".repeat(90));

            for scan in &scans {
                println!("{:<40} {:<12} {:<8} {:<8} {:<20}",
                    &scan.id[..8.min(scan.id.len())],
                    format!("{:?}", scan.status),
                    scan.findings_count,
                    scan.malicious_count,
                    scan.started_at
                );
            }
        }
        cli::OutputFormat::Json => {
            let json_scans: Vec<_> = scans.iter().map(|s| {
                serde_json::json!({
                    "case_id": s.id,
                    "status": format!("{:?}", s.status),
                    "findings_count": s.findings_count,
                    "malicious_count": s.malicious_count,
                    "started_at": s.started_at,
                })
            }).collect();

            println!("{}", serde_json::to_string_pretty(&json_scans)?);
        }
        _ => anyhow::bail!("Only pretty and JSON formats supported"),
    }

    Ok(())
}

/// Generate campaign report.
async fn cmd_campaign_report(cli: &Cli, case_id: &str, format: &cli::ReportFormat, output: Option<&str>) -> Result<()> {
    use std::path::Path;

    info!("Generating campaign report for case: {}", case_id);

    // Only markdown format is supported for campaign reports
    if !matches!(format, cli::ReportFormat::Markdown) {
        anyhow::bail!("Campaign reports only support Markdown format. Use --format markdown");
    }

    // Load campaign checkpoint to get results
    let checkpoint_db = Path::new(".glassware-checkpoints.db");
    let checkpoint_mgr = CheckpointManager::new(checkpoint_db)
        .map_err(|e| anyhow::anyhow!("Failed to open checkpoint database: {}", e))?;

    // Load checkpoint
    let checkpoint = checkpoint_mgr.load_checkpoint(case_id)
        .map_err(|e| anyhow::anyhow!("Failed to load checkpoint: {}", e))?
        .ok_or_else(|| anyhow::anyhow!("Campaign not found: {}. Run 'campaign list' to see available campaigns.", case_id))?;

    info!("Loaded checkpoint for campaign: {} (status: {})", checkpoint.case_id, checkpoint.status);

    // Reconstruct CampaignResult from checkpoint
    // Note: For full wave results, we'd need to store them in checkpoint
    // For now, we create a minimal result from available data
    let campaign_result = reconstruct_campaign_result(&checkpoint)?;

    // Load config to get settings
    let config: glassware_orchestrator::campaign::CampaignConfig = 
        serde_json::from_str(&checkpoint.config_json)
            .map_err(|e| anyhow::anyhow!("Failed to parse campaign config: {}", e))?;

    // Create config summary
    let config_summary = ConfigSummary {
        concurrency: config.settings.concurrency,
        rate_limit: config.settings.rate_limit_npm as u32,
        llm_enabled: config.settings.llm.tier1_enabled,
        threat_threshold: 7.0, // Default threshold
    };

    // Create report generator
    let generator = ReportGenerator::new()
        .map_err(|e| anyhow::anyhow!("Failed to create report generator: {}", e))?;

    // Generate report
    let report = generator.generate_report(&campaign_result, config_summary)
        .map_err(|e| anyhow::anyhow!("Failed to generate report: {}", e))?;

    // Output report
    match output {
        Some(output_path) => {
            // Save to file
            std::fs::write(output_path, &report)
                .map_err(|e| anyhow::anyhow!("Failed to write report to {}: {}", output_path, e))?;
            println!("Report saved to: {}", output_path);
        }
        None => {
            // Default: save to reports/<case-id>/report.md
            let default_path = format!("reports/{}/report.md", case_id);
            std::fs::create_dir_all(format!("reports/{}", case_id))
                .map_err(|e| anyhow::anyhow!("Failed to create reports directory: {}", e))?;
            std::fs::write(&default_path, &report)
                .map_err(|e| anyhow::anyhow!("Failed to write report: {}", e))?;
            println!("Report saved to: {}", default_path);
            
            // Also print to stdout if not quiet
            if !cli.quiet {
                println!("\n{}", report);
            }
        }
    }

    Ok(())
}

/// Reconstruct CampaignResult from checkpoint data.
/// 
/// Note: This is a simplified reconstruction. For full wave results with
/// detailed findings, the checkpoint would need to store additional data.
fn reconstruct_campaign_result(checkpoint: &CampaignCheckpoint) -> Result<CampaignResult, anyhow::Error> {
    use glassware_orchestrator::campaign::{CampaignStatus, WaveResult};
    use std::time::Duration;

    // Parse status string back to enum
    let status = match checkpoint.status.as_str() {
        "completed" => CampaignStatus::Completed,
        "failed" => CampaignStatus::Failed,
        "cancelled" => CampaignStatus::Cancelled,
        "paused" => CampaignStatus::Paused,
        "running" => CampaignStatus::Running,
        _ => CampaignStatus::Initializing,
    };

    // Create wave results from completed waves
    // Note: This is minimal - full implementation would store per-wave stats
    let wave_results: Vec<WaveResult> = checkpoint.completed_waves.iter().map(|wave_id: &String| {
        WaveResult {
            wave_id: wave_id.clone(),
            packages_scanned: 0, // Would need to be stored in checkpoint
            packages_flagged: 0,
            packages_malicious: 0,
        }
    }).collect();

    // Calculate totals from wave results
    let total_scanned: usize = wave_results.iter().map(|w| w.packages_scanned).sum();
    let total_flagged: usize = wave_results.iter().map(|w| w.packages_flagged).sum();
    let total_malicious: usize = wave_results.iter().map(|w| w.packages_malicious).sum();

    // Calculate duration from timestamps
    let duration = Duration::from_secs(0); // Would need start/end times in checkpoint

    Ok(CampaignResult {
        case_id: checkpoint.case_id.clone(),
        campaign_name: checkpoint.campaign_name.clone(),
        status,
        total_scanned,
        total_flagged,
        total_malicious,
        duration,
        wave_results,
    })
}

/// Launch TUI for monitoring a campaign.
async fn cmd_campaign_monitor(case_id: &str) -> Result<()> {
    use glassware_orchestrator::campaign::{EventBus, CommandChannel};
    use tui::app::App;

    info!("Launching TUI monitor for campaign: {}", case_id);

    // Create event bus and command channel
    let event_bus = EventBus::new(512);
    let command_channel = CommandChannel::new();

    // Create and run TUI app
    let mut app = App::new(case_id.to_string(), event_bus, command_channel);

    // Run the TUI
    app.run().await.map_err(|e| anyhow::anyhow!("TUI error: {}", e))?;

    Ok(())
}

/// Launch TUI demo with sample data.
async fn cmd_tui_demo() -> Result<()> {
    use tui::app::App;

    info!("Launching TUI demo with sample data");

    // Create TUI app with sample data
    let mut app = App::with_sample_data();

    // Run the TUI
    app.run().await.map_err(|e| anyhow::anyhow!("TUI error: {}", e))?;

    Ok(())
}
