//! Download Benchmark Packages
//! 
//! Downloads clean packages for benchmark testing.

use anyhow::Result;
use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::Command;
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(name = "download-benchmarks")]
#[command(about = "Download benchmark packages for autoresearch")]
struct Args {
    /// Output directory
    #[arg(short, long, default_value = "benchmarks/clean-packages")]
    output: String,
    
    /// Package list file
    #[arg(short, long, default_value = "benchmarks/clean-packages/packages.txt")]
    list: String,
}

fn main() -> Result<()> {
    // Initialize logging
    let _subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();
    
    let args = Args::parse();
    
    println!("=== Downloading Benchmark Packages ===");
    println!("Output directory: {}", args.output);
    println!("Package list: {}", args.list);
    println!("");
    
    // Create output directory
    std::fs::create_dir_all(&args.output)?;
    
    // Read package list
    let file = File::open(&args.list)?;
    let reader = BufReader::new(file);
    
    let mut total = 0;
    let mut success = 0;
    let mut failed = 0;
    
    for line in reader.lines() {
        let package = line?;
        
        // Skip comments and empty lines
        let package = package.trim();
        if package.is_empty() || package.starts_with('#') {
            continue;
        }
        
        total += 1;
        print!("[{}] Downloading: {} ", total, package);
        
        // Download package
        let output = Command::new("npm")
            .args(["pack", package, "--pack-destination", &args.output])
            .output();
        
        match output {
            Ok(output) => {
                if output.status.success() {
                    println!("✓");
                    success += 1;
                } else {
                    println!("✗ Failed");
                    failed += 1;
                    tracing::warn!("Failed to download: {}", package);
                }
            }
            Err(e) => {
                println!("✗ Error: {}", e);
                failed += 1;
                tracing::warn!("Error downloading {}: {}", package, e);
            }
        }
    }
    
    println!("");
    println!("=== Download Summary ===");
    println!("Total: {}", total);
    println!("Success: {}", success);
    println!("Failed: {}", failed);
    println!("");
    
    // Show downloaded files
    if success > 0 {
        let output_path = PathBuf::from(&args.output);
        if let Ok(entries) = std::fs::read_dir(&output_path) {
            let count = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("tgz"))
                .count();
            println!("Downloaded {} packages", count);
        }
    }
    
    if failed > 0 {
        println!("");
        println!("WARNING: {} packages failed to download", failed);
        println!("This is OK if you have at least 50 packages for testing");
    }
    
    Ok(())
}
