//! Glassware Orchestrator CLI
//!
//! Command-line interface for orchestrating security scans across npm and GitHub.

mod cli;

use anyhow::Result;
use cli::{Cli, Commands, OutputFormat, ResumeSource, SeverityLevel};
use orchestrator_core::{
    Orchestrator, OrchestratorConfig, DownloaderConfig, ScannerConfig,
    PackageScanResult, ScanSummary,
    streaming::{StreamingWriter, OutputFormat as StreamOutputFormat},
    adversarial::AdversarialTester,
};
use glassware_core::Severity;
use std::io::{self, Write};
use std::fs::File;
use std::path::Path;
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;
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

    let tracing_config = orchestrator_core::tracing::TracingConfig {
        level: log_level,
        format: if cli.quiet {
            orchestrator_core::tracing::TracingFormat::Minimal
        } else if cli.verbose {
            orchestrator_core::tracing::TracingFormat::Pretty
        } else {
            orchestrator_core::tracing::TracingFormat::Compact
        },
        output: if let Some(ref log_file) = cli.log_file {
            orchestrator_core::tracing::TracingOutput::File(log_file.clone())
        } else {
            orchestrator_core::tracing::TracingOutput::Stdout
        },
        with_ansi: !cli.no_color && !cli.quiet,
        ..Default::default()
    };

    if let Err(e) = orchestrator_core::tracing::init_tracing(&tracing_config) {
        eprintln!("Warning: Failed to initialize tracing: {}", e);
    }

    info!("Glassware Orchestrator v{}", orchestrator_core::VERSION);

    // Run command
    match cli.command {
        Commands::ScanNpm { ref packages } => {
            cmd_scan_npm(&cli, packages.clone()).await?;
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
    }

    Ok(())
}

/// Scan npm packages command.
async fn cmd_scan_npm(cli: &Cli, packages: Vec<String>) -> Result<()> {
    if packages.is_empty() {
        error!("No packages specified");
        return Ok(());
    }

    info!("Scanning {} npm packages", packages.len());

    let orchestrator = create_orchestrator(cli).await?;

    if cli.streaming {
        // Use streaming output
        cmd_scan_npm_streaming(cli, &orchestrator, &packages).await?;
    } else {
        // Use buffered output
        let results = orchestrator.scan_npm_packages(&packages).await;

        // Run adversarial testing if enabled
        if cli.adversarial {
            run_adversarial_tests(&results).await?;
        }

        print_results(cli, &results)?;
    }

    Ok(())
}

/// Scan npm packages with streaming output.
async fn cmd_scan_npm_streaming(
    cli: &Cli,
    orchestrator: &Orchestrator,
    packages: &[String],
) -> Result<()> {
    // Determine output destination
    let output: Box<dyn Write + Send> = if let Some(ref output_path) = cli.output {
        Box::new(File::create(output_path)?)
    } else {
        Box::new(io::stdout())
    };

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
            OutputFormat::Json => {
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

/// Cache cleanup command.
async fn cmd_cache_cleanup(cli: &Cli) -> Result<()> {
    let orchestrator = create_orchestrator(cli).await?;

    let removed = orchestrator.cleanup_cache().await?;

    if !cli.quiet {
        println!("Cleaned up {} expired cache entries", removed);
    }

    Ok(())
}

/// Create orchestrator from CLI options.
async fn create_orchestrator(cli: &Cli) -> Result<Orchestrator> {
    let config = OrchestratorConfig {
        downloader: DownloaderConfig {
            max_retries: cli.max_retries,
            npm_rate_limit: cli.npm_rate_limit,
            github_rate_limit: cli.github_rate_limit,
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
        cache_ttl_days: cli.cache_ttl,
        enable_cache: !cli.no_cache,
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
fn print_results(cli: &Cli, results: &[orchestrator_core::Result<PackageScanResult>]) -> Result<()> {
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
    errors: &[&orchestrator_core::error::OrchestratorError],
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
                    "version": orchestrator_core::VERSION,
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
    results: &[orchestrator_core::Result<PackageScanResult>],
) -> Result<()> {
    info!("Running adversarial tests on scanned packages...");

    let tester = AdversarialTester::new()?;
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
) -> Result<Option<orchestrator_core::adversarial::AdversarialReport>> {
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
