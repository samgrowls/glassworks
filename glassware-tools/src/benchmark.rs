//! Benchmark Module
//! 
//! Scans packages and collects benchmark results.

use crate::metrics::BenchmarkResult;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn, error};

/// Benchmark runner
pub struct BenchmarkRunner {
    glassware_binary: PathBuf,
    evidence_dir: PathBuf,
    clean_packages_dir: PathBuf,
    use_subset: bool,
    subset_size: usize,
}

impl BenchmarkRunner {
    /// Create a new benchmark runner
    pub fn new(
        glassware_binary: PathBuf,
        evidence_dir: PathBuf,
        clean_packages_dir: PathBuf,
        use_subset: bool,
        subset_size: usize,
    ) -> Self {
        Self {
            glassware_binary,
            evidence_dir,
            clean_packages_dir,
            use_subset,
            subset_size,
        }
    }
    
    /// Run benchmark and return results
    pub fn run(&self) -> anyhow::Result<BenchmarkResult> {
        info!("Running benchmark...");
        
        let start_time = std::time::Instant::now();
        
        // Scan evidence packages
        let (evidence_total, evidence_detected) = self.scan_evidence()?;
        
        // Scan clean packages
        let (clean_total, clean_flagged) = self.scan_clean()?;
        
        let elapsed = start_time.elapsed();
        let scan_time_seconds = elapsed.as_secs_f64();
        
        // Estimate scan speed (rough estimate based on package count)
        // In production, we'd track actual LOC scanned
        let estimated_loc = (evidence_total + clean_total) as f64 * 5000.0;
        let scan_speed_loc_per_sec = if scan_time_seconds > 0.0 {
            estimated_loc / scan_time_seconds
        } else {
            0.0
        };
        
        let result = BenchmarkResult {
            evidence_total,
            evidence_detected,
            clean_total,
            clean_flagged,
            scan_time_seconds,
            scan_speed_loc_per_sec,
        };
        
        info!(
            "Benchmark complete: FP={:.1}%, Detection={:.1}%, F1={:.3}",
            result.fp_rate() * 100.0,
            result.detection_rate() * 100.0,
            result.f1_score()
        );
        
        Ok(result)
    }
    
    /// Scan evidence packages
    fn scan_evidence(&self) -> anyhow::Result<(usize, usize)> {
        let mut total = 0;
        let mut detected = 0;
        
        // Scan root evidence tarballs
        if self.evidence_dir.exists() {
            for entry in std::fs::read_dir(&self.evidence_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.extension().and_then(|s| s.to_str()) == Some("tgz") {
                    total += 1;
                    if self.is_flagged(&path)? {
                        detected += 1;
                    }
                }
            }
        }
        
        // Scan category subdirectories
        for category_dir in std::fs::read_dir(&self.evidence_dir)? {
            let category_dir = category_dir?.path();
            if !category_dir.is_dir() {
                continue;
            }
            
            for package_dir in std::fs::read_dir(&category_dir)? {
                let package_dir = package_dir?.path();
                if !package_dir.is_dir() {
                    continue;
                }
                
                total += 1;
                
                // Create temporary tarball
                let temp_tarball = std::env::temp_dir().join(format!(
                    "evidence_{}.tgz",
                    package_dir.file_name().unwrap().to_string_lossy()
                ));
                
                let status = Command::new("tar")
                    .args(["-czf", temp_tarball.to_str().unwrap(), "-C", package_dir.to_str().unwrap(), "."])
                    .status()?;
                
                if status.success() {
                    if self.is_flagged(&temp_tarball)? {
                        detected += 1;
                    }
                    let _ = std::fs::remove_file(temp_tarball);
                }
            }
        }
        
        Ok((total, detected))
    }
    
    /// Scan clean packages
    fn scan_clean(&self) -> anyhow::Result<(usize, usize)> {
        let mut total = 0;
        let mut flagged = 0;
        
        if !self.clean_packages_dir.exists() {
            warn!("Clean packages directory does not exist: {:?}", self.clean_packages_dir);
            return Ok((0, 0));
        }
        
        // Get list of packages
        let mut packages: Vec<PathBuf> = std::fs::read_dir(&self.clean_packages_dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("tgz"))
            .collect();
        
        // Use subset if configured
        if self.use_subset && packages.len() > self.subset_size {
            use rand::seq::SliceRandom;
            let mut rng = rand::thread_rng();
            packages.shuffle(&mut rng);
            packages.truncate(self.subset_size);
            info!("Using subset of {} packages (out of {})", packages.len(), self.clean_packages_dir.read_dir()?.count());
        }
        
        // Scan each package
        for package in &packages {
            total += 1;
            
            if self.is_flagged(package)? {
                flagged += 1;
                info!("  FP: {}", package.file_name().unwrap().to_string_lossy());
            }
        }
        
        Ok((total, flagged))
    }
    
    /// Check if a package is flagged as malicious
    fn is_flagged(&self, tarball: &Path) -> anyhow::Result<bool> {
        let output = Command::new(&self.glassware_binary)
            .args(["scan-tarball", tarball.to_str().unwrap()])
            .output();
        
        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                // Check for "malicious" or "flagged" in output
                let is_flagged = stdout.to_lowercase().contains("malicious")
                    || stdout.to_lowercase().contains("flagged")
                    || stderr.to_lowercase().contains("malicious")
                    || stderr.to_lowercase().contains("flagged");
                
                Ok(is_flagged)
            }
            Err(e) => {
                error!("Failed to scan {:?}: {}", tarball, e);
                Ok(false) // Don't count as flagged on error
            }
        }
    }
}
