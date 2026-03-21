//! Version Scanner - Scan multiple versions of npm packages
//!
//! Provides functionality to:
//! - Query npm registry for package versions
//! - Sample versions based on policy
//! - Scan multiple versions sequentially

use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::downloader::Downloader;
use crate::error::{OrchestratorError, Result};
use crate::scanner::{Scanner, PackageScanResult};

/// npm package metadata response
#[derive(Debug, Clone, Deserialize)]
pub struct NpmPackageMetadata {
    pub name: String,
    #[serde(rename = "dist-tags")]
    pub dist_tags: Option<HashMap<String, String>>,
    pub versions: HashMap<String, NpmVersionInfo>,
    pub time: Option<NpmTimeInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NpmVersionInfo {
    pub name: String,
    pub version: String,
    pub dist: Option<NpmDistInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NpmDistInfo {
    pub tarball: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NpmTimeInfo {
    pub created: Option<String>,
    pub modified: Option<String>,
    #[serde(flatten)]
    pub version_times: HashMap<String, String>,
}

/// Version sampling policy
#[derive(Debug, Clone)]
pub enum VersionPolicy {
    /// Scan last N versions
    LastN(usize),
    /// Scan versions from last N days
    LastDays(u32),
    /// Scan all versions
    All,
    /// Scan specific versions
    Specific(Vec<String>),
}

impl VersionPolicy {
    /// Parse policy from string
    pub fn from_str(s: &str) -> Result<Self> {
        if s == "all" {
            Ok(VersionPolicy::All)
        } else if s.starts_with("last-") {
            let rest = &s[5..];
            if rest.ends_with('d') {
                // Days format: last-180d
                let days: u32 = rest[..rest.len()-1].parse()
                    .map_err(|_| OrchestratorError::validation_error(
                        format!("Invalid days format: {}", rest),
                        Some("version_policy"),
                    ))?;
                Ok(VersionPolicy::LastDays(days))
            } else {
                // Count format: last-10
                let count: usize = rest.parse()
                    .map_err(|_| OrchestratorError::validation_error(
                        format!("Invalid count format: {}", rest),
                        Some("version_policy"),
                    ))?;
                Ok(VersionPolicy::LastN(count))
            }
        } else {
            // Comma-separated versions
            let versions = s.split(',').map(|s| s.trim().to_string()).collect();
            Ok(VersionPolicy::Specific(versions))
        }
    }
}

/// Version scanner
pub struct VersionScanner {
    client: Client,
    downloader: Downloader,
    scanner: Scanner,
}

impl VersionScanner {
    /// Create new version scanner
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            downloader: Downloader::default(),
            scanner: Scanner::default(),
        })
    }
    
    /// Fetch package metadata from npm
    pub async fn fetch_metadata(&self, package: &str) -> Result<NpmPackageMetadata> {
        let url = format!("https://registry.npmjs.org/{}", package);
        
        let response = self.client.get(&url)
            .send()
            .await
            .map_err(|e| OrchestratorError::http(e))?;
        
        if !response.status().is_success() {
            return Err(OrchestratorError::download_failed(
                package,
                format!("npm returned status {}", response.status()),
            ));
        }
        
        let metadata = response.json::<NpmPackageMetadata>()
            .await
            .map_err(|e| OrchestratorError::http_error(e, "Failed to parse npm metadata"))?;
        
        Ok(metadata)
    }
    
    /// Get all versions sorted by publish time
    pub async fn get_versions_sorted(&self, package: &str) -> Result<Vec<String>> {
        let metadata = self.fetch_metadata(package).await?;
        
        // Get version times
        let mut version_times: Vec<(String, String)> = metadata.time
            .as_ref()
            .map(|t| {
                t.version_times.iter()
                    .filter(|(k, _)| *k != "created" && *k != "modified")
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default();
        
        // Sort by time (newest first)
        version_times.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Return just version numbers
        Ok(version_times.into_iter().map(|(v, _)| v).collect())
    }
    
    /// Sample versions based on policy
    pub async fn sample_versions(
        &self,
        package: &str,
        policy: &VersionPolicy
    ) -> Result<Vec<String>> {
        let all_versions = self.get_versions_sorted(package).await?;
        
        match policy {
            VersionPolicy::All => Ok(all_versions),
            VersionPolicy::LastN(n) => {
                Ok(all_versions.into_iter().take(*n).collect())
            }
            VersionPolicy::LastDays(days) => {
                // For now, just take last 10 versions
                // TODO: Filter by actual date
                warn!("LastDays policy not fully implemented, using LastN(10)");
                Ok(all_versions.into_iter().take(10).collect())
            }
            VersionPolicy::Specific(versions) => {
                // Filter to only versions that exist
                let version_set: std::collections::HashSet<_> = 
                    all_versions.into_iter().collect();
                Ok(versions.iter()
                    .filter(|v| version_set.contains(*v))
                    .cloned()
                    .collect())
            }
        }
    }
    
    /// Scan multiple versions of a package
    pub async fn scan_versions(
        &self,
        package: &str,
        versions: &[String],
    ) -> Vec<Result<PackageScanResult>> {
        let mut results = Vec::new();
        
        for (i, version) in versions.iter().enumerate() {
            info!("Scanning {}/{}: {}@{}", i + 1, versions.len(), package, version);
            
            let full_name = format!("{}@{}", package, version);
            match self.downloader.download_npm_package(&full_name).await {
                Ok(downloaded) => {
                    match self.scanner.scan_package(&downloaded).await {
                        Ok(result) => results.push(Ok(result)),
                        Err(e) => {
                            warn!("Scan failed for {}@{}: {}", package, version, e);
                            results.push(Err(e));
                        }
                    }
                }
                Err(e) => {
                    warn!("Download failed for {}@{}: {}", package, version, e);
                    results.push(Err(e));
                }
            }
        }
        
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_version_policy_parse() {
        assert!(matches!(VersionPolicy::from_str("all"), Ok(VersionPolicy::All)));
        assert!(matches!(VersionPolicy::from_str("last-10"), Ok(VersionPolicy::LastN(10))));
        assert!(matches!(VersionPolicy::from_str("last-180d"), Ok(VersionPolicy::LastDays(180))));
        assert!(matches!(VersionPolicy::from_str("1.0.0,2.0.0"), Ok(VersionPolicy::Specific(v)) if v.len() == 2));
    }
    
    #[tokio::test]
    async fn test_fetch_metadata() {
        let scanner = VersionScanner::new().unwrap();
        let metadata = scanner.fetch_metadata("express").await.unwrap();
        assert_eq!(metadata.name, "express");
        assert!(!metadata.versions.is_empty());
    }
    
    #[tokio::test]
    async fn test_get_versions_sorted() {
        let scanner = VersionScanner::new().unwrap();
        let versions = scanner.get_versions_sorted("express").await.unwrap();
        assert!(!versions.is_empty());
        // First version should be latest
        assert!(versions[0].starts_with('5') || versions[0].starts_with('4'));
    }
}
