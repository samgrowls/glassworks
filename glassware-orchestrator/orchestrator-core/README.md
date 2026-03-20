# orchestrator-core

Core library for orchestrating glassware security scans across npm and GitHub.

## Overview

`orchestrator-core` is a high-performance Rust library that coordinates security scans across npm packages and GitHub repositories using the `glassware-core` detection engine. It provides parallel scanning, caching, rate limiting, retry logic, and comprehensive output formatting.

## Features

### Core Capabilities

- **Parallel Scanning**: Concurrent downloads and scans with configurable concurrency
- **SQLite Caching**: Persistent cache with configurable TTL (default: 7 days)
- **Rate Limiting**: Configurable rate limits for npm and GitHub APIs
- **Retry Logic**: Exponential backoff with jitter for transient failures
- **Progress Tracking**: Real-time progress updates with ETA calculation
- **GitHub Integration**: Repository search and archive download
- **Checkpoint/Resume**: Save and resume long-running scans
- **Output Formatters**: JSON, JSON Lines, SARIF 2.1.0 output
- **LLM Analysis**: OpenAI-compatible API integration (optional)

### Phase 3 Features

- **Streaming Output**: Stream results as they complete to prevent OOM on large scans
- **Adversarial Testing**: Integrated mutation and fuzzing engines for evasion detection
- **Enhanced Error Handling**: Better error messages with context, categorization, and recovery suggestions
- **Tracing/Logging**: Configurable tracing with multiple formats and output destinations
- **Performance Benchmarks**: Comprehensive benchmarks for scan speed, memory usage, and cache performance

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
orchestrator-core = { path = "../orchestrator-core" }
```

## Quick Start

```rust
use orchestrator_core::{Orchestrator, OrchestratorConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create orchestrator with default config
    let orchestrator = Orchestrator::new().await?;

    // Scan npm packages
    let packages = vec!["express".to_string(), "lodash".to_string()];
    let results = orchestrator.scan_npm_packages(&packages).await;

    // Process results
    for result in results {
        match result {
            Ok(scan_result) => {
                println!("Package: {}", scan_result.package_name);
                println!("Threat score: {:.2}", scan_result.threat_score);
                println!("Findings: {}", scan_result.findings.len());
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    // Export results to SARIF
    let sarif = orchestrator.format_sarif(&results, true)?;
    std::fs::write("results.sarif", &sarif)?;

    Ok(())
}
```

## Configuration

```rust
use orchestrator_core::{OrchestratorConfig, DownloaderConfig, ScannerConfig};
use orchestrator_core::retry::RetryConfigBuilder;

let config = OrchestratorConfig {
    downloader: DownloaderConfig {
        max_retries: 5,
        npm_rate_limit: 5.0,  // 5 requests/sec
        github_rate_limit: 2.0,  // 2 requests/sec
        ..Default::default()
    },
    scanner: ScannerConfig {
        max_concurrent: 20,
        threat_threshold: 7.0,
        ..Default::default()
    },
    cache_ttl_days: 14,  // 2 weeks
    enable_checkpoint: true,
    checkpoint_interval: 10,  // Auto-save every 10 packages
    retry_config: RetryConfigBuilder::new()
        .max_retries(3)
        .build(),
    npm_rate_limit: 5.0,
    github_rate_limit: 2.0,
    ..Default::default()
};

let orchestrator = Orchestrator::with_config(config).await?;
```

## Streaming Output

For large-scale scans, use streaming output to prevent OOM:

```rust
use orchestrator_core::streaming::{StreamingWriter, OutputFormat};
use tokio::io::BufWriter;
use std::fs::File;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create("results.jsonl")?;
    let writer = BufWriter::new(file);
    let mut streaming = StreamingWriter::json_lines(writer);

    // Write results as they complete
    for result in results {
        streaming.write_result(&result).await?;
    }

    streaming.flush().await?;
    Ok(())
}
```

## Adversarial Testing

Test packages for evasion techniques:

```rust
use orchestrator_core::adversarial::AdversarialTester;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tester = AdversarialTester::new()?;
    let report = tester.test_package("/path/to/package").await?;

    println!("Evasion rate: {:.2}%", report.evasion_rate * 100.0);
    println!("Risk level: {:?}", report.risk_level());

    for evasion in &report.high_risk_evasions {
        println!("High-risk evasion: {}", evasion);
    }

    Ok(())
}
```

## Tracing and Logging

Configure tracing for debugging or production:

```rust
use orchestrator_core::tracing::{init_tracing, TracingConfig, TracingFormat, TracingOutput};
use tracing::Level;

// Debug mode
let config = TracingConfig::debug();
init_tracing(&config)?;

// Production mode with JSON output
let config = TracingConfig::json()
    .with_level(Level::INFO)
    .with_output(TracingOutput::File("app.log".to_string()));
init_tracing(&config)?;

tracing::info!("Tracing initialized");
```

## Progress Tracking

```rust
use orchestrator_core::{Orchestrator, ScanProgress};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let orchestrator = Orchestrator::new()
        .await?
        .with_progress_callback(|progress: ScanProgress| {
            println!(
                "Progress: {:.1}% - {} (ETA: {})",
                progress.percentage(),
                progress.status,
                progress.format_eta()
            );
        });

    // ... use orchestrator
    Ok(())
}
```

## Checkpoint/Resume

```rust
use orchestrator_core::Orchestrator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut orchestrator = Orchestrator::new().await?;

    // Load checkpoint if exists
    if orchestrator.load_checkpoint("npm").await? {
        println!("Resuming from checkpoint...");
    }

    // Auto-save every 10 packages
    // Checkpoints stored in .glassware-checkpoints/

    Ok(())
}
```

## Output Formats

### JSON

```rust
let json = orchestrator.format_json(&results, true)?;
std::fs::write("results.json", &json)?;
```

### SARIF (GitHub Advanced Security)

```rust
let sarif = orchestrator.format_sarif(&results, true)?;
std::fs::write("results.sarif", &sarif)?;
```

### JSON Lines (Streaming)

```rust
use orchestrator_core::streaming::{StreamingWriter, OutputFormat};

let file = File::create("results.jsonl")?;
let mut streaming = StreamingWriter::json_lines(BufWriter::new(file));

for result in &results {
    streaming.write_result(result).await?;
}
streaming.flush().await?;
```

## Error Handling

The library provides enhanced error handling with context and recovery suggestions:

```rust
use orchestrator_core::error::{OrchestratorError, ErrorCategory};

match orchestrator.scan_npm_package("pkg").await {
    Ok(result) => { /* success */ }
    Err(e) => {
        eprintln!("Error category: {:?}", e.category());
        eprintln!("Is retryable: {}", e.is_retryable());
        eprintln!("Recovery: {}", e.recovery_suggestion());
        eprintln!("Context: {}", e.context());
    }
}
```

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Orchestrator                             │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  Downloader  │  │   Scanner    │  │    Cacher    │          │
│  │              │  │              │  │              │          │
│  │ - npm API    │  │ - glassware  │  │ - SQLite     │          │
│  │ - GitHub API │  │ - L1/L2/L3   │  │ - 7-day TTL  │          │
│  │ - Rate limit │  │ - Parallel   │  │ - Stats      │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   GitHub     │  │  Progress    │  │ Checkpoint   │          │
│  │  Downloader  │  │   Tracker    │  │   Manager    │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  Formatters  │  │  Streaming   │  │ Adversarial  │          │
│  │ - JSON       │  │ - JSON Lines │  │ - Mutation   │          │
│  │ - SARIF      │  │ - SARIF      │  │ - Fuzzing    │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
```

## Performance

| Metric | Value |
|--------|-------|
| Scan speed | ~50k LOC/sec |
| Memory usage | ~50 MB peak |
| Cache lookup | < 1ms |
| Cache write | < 5ms |
| Concurrent scans | Configurable (default: 10) |

## Testing

```bash
# Run tests
cargo test

# Run with all features
cargo test --features "full,llm"

# Run benchmarks
cargo bench
```

## License

Apache-2.0

## Contributing

See the main [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.
