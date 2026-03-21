//! Glassware Orchestrator CLI
//!
//! Command-line interface for orchestrating security scans across npm and GitHub.

mod cli;

use anyhow::Result;
use cli::{Cli, Commands, OutputFormat, ResumeSource};
use glassware_orchestrator::{
    Orchestrator, OrchestratorConfig, DownloaderConfig, ScannerConfig,
    PackageScanResult, ScanSummary,
    streaming::StreamingWriter,
    adversarial::AdversarialTester,
    scan_registry::{ScanRegistry, ScanStatus},
    cli_validator,
};
use glassware_core::Severity;
use tracing::{error, info, warn, Level};
use tokio::io::BufWriter;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse_args();

    // Initialize tracing with config
    let log_level = match cli.log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
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
        Commands::ScanNpm { ref packages, ref versions } => {
            cmd_scan_npm(&cli, packages.clone(), versions.clone()).await?;
        }
        Commands::ScanGithub { ref repos, ref r#ref } => {
            cmd_scan_github(&cli, repos.clone(), r#ref.as_deref()).await?;
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
    }

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
    let config = OrchestratorConfig {
        downloader: DownloaderConfig {
            max_retries: cli.max_retries,
            npm_rate_limit: cli.npm_rate_limit as f32,
            github_rate_limit: cli.github_rate_limit as f32,
            github_token: cli.github_token.clone(),
            max_concurrent: cli.concurrency,
            ..Default::default()
        },
        scanner: ScannerConfig {
            max_concurrent: cli.concurrency,
            min_severity: cli.severity.to_core_severity(),
            threat_threshold: cli.threat_threshold,
            ..Default::default()
        },
        cache_db_path: if cli.no_cache {
            None
        } else {
            Some(cli.cache_db.clone())
        },
        cache_ttl_days: cli.cache_ttl as i64,
        enable_cache: !cli.no_cache,
        github_token: cli.github_token.clone(),
        enable_checkpoint: true,
        checkpoint_dir: Some(cli.checkpoint_dir.clone()),
        checkpoint_interval: 10,
        retry_config: glassware_orchestrator::retry::RetryConfig::default(),
        npm_rate_limit: cli.npm_rate_limit as f32,
        github_rate_limit: cli.github_rate_limit as f32,
        #[cfg(feature = "llm")]
        enable_llm: cli.llm,
        #[cfg(feature = "llm")]
        llm_config: if cli.llm {
            glassware_orchestrator::llm::LlmAnalyzerConfig::from_env()
        } else {
            None
        },
    };

    let orchestrator = Orchestrator::with_config(config).await?;

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
