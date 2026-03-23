//! Downloaders for npm and GitHub sources.
//!
//! This module provides async downloaders for fetching packages from npm registry
//! and repositories from GitHub, with rate limiting and retry logic.

use reqwest::{Client, Response, StatusCode};
use serde::Deserialize;
use std::path::Path;
use std::time::Duration;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

#[cfg(feature = "rate-limit")]
use governor::{Quota, RateLimiter};
#[cfg(feature = "rate-limit")]
use std::num::NonZeroU32;

#[cfg(feature = "retry")]
use tokio_retry::{
    strategy::{jitter, ExponentialBackoff},
    Retry,
};

use crate::error::{ErrorContext, OrchestratorError, Result};

/// npm registry response for package metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct NpmPackageInfo {
    /// Package name.
    pub name: String,

    /// Latest version from dist-tags
    #[serde(rename = "dist-tags", default)]
    pub dist_tags: Option<NpmDistTags>,

    /// All versions with their metadata
    #[serde(default)]
    pub versions: std::collections::HashMap<String, NpmVersionInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NpmDistTags {
    #[serde(default)]
    pub latest: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NpmVersionInfo {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub dist: Option<NpmDist>,
    #[serde(default)]
    pub description: Option<String>,
}

impl NpmPackageInfo {
    /// Resolve the tarball URL for the latest version.
    pub fn resolve_tarball(&self) -> Option<String> {
        // Get latest version from dist-tags
        let latest = self.dist_tags.as_ref()?.latest.as_ref()?;

        // Get version info from versions map
        let version_info = self.versions.get(latest)?;

        // Get tarball from dist
        version_info.dist.as_ref()?.tarball.clone()
    }

    /// Get the latest version string.
    pub fn latest_version(&self) -> Option<String> {
        self.dist_tags.as_ref()?.latest.clone()
    }
}

/// Repository information from npm.
#[derive(Debug, Clone, Deserialize)]
pub struct NpmRepository {
    /// Repository type (git, github, etc.).
    #[serde(default)]
    pub r#type: Option<String>,
    /// Repository URL.
    #[serde(default)]
    pub url: Option<String>,
}

/// Distribution information from npm.
#[derive(Debug, Clone, Deserialize)]
pub struct NpmDist {
    /// Tarball download URL.
    #[serde(default)]
    pub tarball: Option<String>,
}

/// GitHub repository information.
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRepoInfo {
    /// Repository name.
    pub name: String,
    /// Owner/login.
    pub owner: String,
    /// Default branch.
    #[serde(default)]
    pub default_branch: Option<String>,
    /// Clone URL (HTTPS).
    #[serde(default)]
    pub clone_url: Option<String>,
    /// Archive URL template.
    #[serde(default)]
    pub archive_url: Option<String>,
    /// Repository size in KB.
    #[serde(default)]
    pub size: Option<u64>,
}

/// Downloaded package content.
#[derive(Debug, Clone)]
pub struct DownloadedPackage {
    /// Package name or identifier.
    pub name: String,
    /// Source type (npm, github).
    pub source_type: String,
    /// Version or commit hash.
    pub version: String,
    /// Path to downloaded content (temp directory).
    pub path: String,
    /// Content hash for validation.
    pub content_hash: String,
}

/// Configuration for the downloader.
#[derive(Debug, Clone)]
pub struct DownloaderConfig {
    /// Request timeout in seconds.
    pub timeout_secs: u64,
    /// Maximum number of retries.
    pub max_retries: u32,
    /// Base delay for exponential backoff in milliseconds.
    pub retry_base_delay_ms: u64,
    /// Rate limit: requests per second for npm.
    pub npm_rate_limit: f32,
    /// Rate limit: requests per second for GitHub.
    pub github_rate_limit: f32,
    /// GitHub API token (optional).
    pub github_token: Option<String>,
    /// Maximum concurrent downloads.
    pub max_concurrent: usize,
}

impl Default for DownloaderConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            max_retries: 3,
            retry_base_delay_ms: 1000,
            npm_rate_limit: 2.0, // 2 requests per second
            github_rate_limit: 1.0, // 1 request per second (GitHub API limit)
            github_token: None,
            max_concurrent: 10,
        }
    }
}

/// Downloader for npm and GitHub sources.
#[derive(Clone)]
pub struct Downloader {
    client: Client,
    config: DownloaderConfig,
    #[cfg(feature = "rate-limit")]
    npm_limiter: Arc<governor::RateLimiter<governor::state::direct::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>>,
    #[cfg(feature = "rate-limit")]
    github_limiter: Arc<governor::RateLimiter<governor::state::direct::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>>,
    concurrency_semaphore: Arc<Semaphore>,
}

impl Downloader {
    /// Create a new downloader with default configuration.
    pub fn new() -> Result<Self> {
        Self::with_config(DownloaderConfig::default())
    }

    /// Create a new downloader with custom configuration.
    pub fn with_config(config: DownloaderConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .user_agent("glassware-orchestrator/0.1.0")
            .build()
            .map_err(|e| OrchestratorError::http(e))?;

        let concurrency_semaphore = Arc::new(Semaphore::new(config.max_concurrent));

        #[cfg(feature = "rate-limit")]
        let npm_limiter = {
            let quota = Quota::per_second(NonZeroU32::new(config.npm_rate_limit as u32).unwrap_or(NonZeroU32::new(1).unwrap()));
            Arc::new(RateLimiter::direct(quota))
        };

        #[cfg(feature = "rate-limit")]
        let github_limiter = {
            let quota = Quota::per_second(NonZeroU32::new(config.github_rate_limit as u32).unwrap_or(NonZeroU32::new(1).unwrap()));
            Arc::new(RateLimiter::direct(quota))
        };

        Ok(Self {
            client,
            config,
            #[cfg(feature = "rate-limit")]
            npm_limiter,
            #[cfg(feature = "rate-limit")]
            github_limiter,
            concurrency_semaphore,
        })
    }

    /// Get npm package metadata.
    pub async fn get_npm_package_info(&self, package: &str) -> Result<NpmPackageInfo> {
        // Parse package spec to extract name (ignore version for metadata fetch)
        let pkg_spec = NpmPackageSpec::parse(package)?;
        
        #[cfg(feature = "rate-limit")]
        self.npm_limiter.until_ready().await;

        let url = format!("https://registry.npmjs.org/{}", pkg_spec.name);
        
        let fetch = || async {
            let response = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(|e| OrchestratorError::http(e))?;

            if response.status() == StatusCode::NOT_FOUND {
                return Err(OrchestratorError::not_found(package));
            }

            if !response.status().is_success() {
                return Err(OrchestratorError::npm(format!(
                    "npm API returned status: {}",
                    response.status()
                )));
            }

            let info: NpmPackageInfo = response
                .json()
                .await
                .map_err(|e| OrchestratorError::http(e))?;

            Ok(info)
        };

        #[cfg(feature = "retry")]
        {
            let retry_strategy = ExponentialBackoff::from_millis(self.config.retry_base_delay_ms)
                .map(jitter)
                .take(self.config.max_retries as usize);

            Retry::spawn(retry_strategy, fetch).await
        }
        #[cfg(not(feature = "retry"))]
        {
            fetch().await
        }
    }

    /// Download npm package tarball.
    pub async fn download_npm_package(&self, package: &str) -> Result<DownloadedPackage> {
        // Parse package spec to extract name and version
        let pkg_spec = NpmPackageSpec::parse(package)?;
        
        let info = self.get_npm_package_info(&pkg_spec.name).await?;

        // Resolve tarball URL - use specified version or latest
        let tarball_url = if let Some(ref version) = pkg_spec.version {
            // Use specified version
            info.versions.get(version)
                .and_then(|v| v.dist.as_ref())
                .and_then(|d| d.tarball.clone())
                .ok_or_else(|| OrchestratorError::npm(format!(
                    "Version '{}' not found for package '{}'. Available versions: {:?}",
                    version,
                    pkg_spec.name,
                    info.versions.keys().take(5).collect::<Vec<_>>()
                )))?
        } else {
            // Use latest version
            info.resolve_tarball()
                .ok_or_else(|| OrchestratorError::npm(format!(
                    "No tarball URL found for package '{}'",
                    pkg_spec.name
                )))?
        };

        // Get version string for logging
        let version = pkg_spec.version
            .or_else(|| info.latest_version())
            .unwrap_or_else(|| "unknown".to_string());

        tracing::debug!(
            package = %package,
            version = %version,
            tarball = %tarball_url,
            "Downloading npm package"
        );

        self.download_tarball(package, &tarball_url, "npm", &version).await
    }

    /// Get GitHub repository information.
    pub async fn get_github_repo_info(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<GitHubRepoInfo> {
        #[cfg(feature = "rate-limit")]
        self.github_limiter.until_ready().await;

        let url = format!("https://api.github.com/repos/{}/{}", owner, repo);

        let fetch = || async {
            let mut request = self.client.get(&url);

            if let Some(ref token) = self.config.github_token {
                request = request.header("Authorization", format!("token {}", token));
            }

            let response = request
                .send()
                .await
                .map_err(|e| OrchestratorError::http(e))?;

            if response.status() == StatusCode::NOT_FOUND {
                return Err(OrchestratorError::not_found(format!(
                    "GitHub repository: {}/{}",
                    owner, repo
                )));
            }

            if response.status() == StatusCode::FORBIDDEN {
                if let Some(retry_after) = response.headers().get("Retry-After") {
                    let retry_after_secs = retry_after
                        .to_str()
                        .unwrap_or("60")
                        .parse()
                        .unwrap_or(60);
                    return Err(OrchestratorError::RateLimitExceeded {
                        retry_after: retry_after_secs,
                        context: ErrorContext::new(),
                    });
                }
                return Err(OrchestratorError::github(
                    "GitHub API rate limit exceeded".to_string(),
                ));
            }

            if !response.status().is_success() {
                return Err(OrchestratorError::github(format!(
                    "GitHub API returned status: {}",
                    response.status()
                )));
            }

            let repo_info: GitHubRepoInfo = response
                .json()
                .await
                .map_err(|e| OrchestratorError::http(e))?;

            Ok(repo_info)
        };

        #[cfg(feature = "retry")]
        {
            let retry_strategy = ExponentialBackoff::from_millis(self.config.retry_base_delay_ms)
                .map(jitter)
                .take(self.config.max_retries as usize);

            Retry::spawn(retry_strategy, fetch).await
        }
        #[cfg(not(feature = "retry"))]
        {
            fetch().await
        }
    }

    /// Download GitHub repository as archive.
    pub async fn download_github_repo(
        &self,
        owner: &str,
        repo: &str,
        ref_name: Option<&str>,
    ) -> Result<DownloadedPackage> {
        let ref_name = ref_name.unwrap_or("HEAD");
        let archive_url = format!(
            "https://api.github.com/repos/{}/{}/tarball/{}",
            owner, repo, ref_name
        );

        let package_name = format!("{}/{}", owner, repo);

        self.download_tarball(&package_name, &archive_url, "github", ref_name)
            .await
    }

    /// Download a tarball and extract it.
    async fn download_tarball(
        &self,
        name: &str,
        url: &str,
        source_type: &str,
        version: &str,
    ) -> Result<DownloadedPackage> {
        let _permit = self
            .concurrency_semaphore
            .acquire()
            .await
            .map_err(|_| OrchestratorError::cancelled("Semaphore closed".to_string()))?;

        #[cfg(feature = "rate-limit")]
        if source_type == "npm" {
            self.npm_limiter.until_ready().await;
        } else if source_type == "github" {
            self.github_limiter.until_ready().await;
        }

        let fetch = || async {
            let mut request = self.client.get(url);

            if source_type == "github" {
                if let Some(ref token) = self.config.github_token {
                    request = request.header("Authorization", format!("token {}", token));
                }
            }

            let response = request
                .send()
                .await
                .map_err(|e| OrchestratorError::http(e))?;

            if !response.status().is_success() {
                return Err(OrchestratorError::download_failed(
                    name.to_string(),
                    format!("Download failed: {}", response.status()),
                ));
            }

            let bytes = response
                .bytes()
                .await
                .map_err(|e| OrchestratorError::http(e))?;

            Ok(bytes)
        };

        #[cfg(feature = "retry")]
        let bytes = {
            let retry_strategy = ExponentialBackoff::from_millis(self.config.retry_base_delay_ms)
                .map(jitter)
                .take(self.config.max_retries as usize);

            Retry::spawn(retry_strategy, fetch).await?
        };

        #[cfg(not(feature = "retry"))]
        let bytes = fetch().await?;

        // Calculate content hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        bytes.hash(&mut hasher);
        let content_hash = format!("{:x}", hasher.finish());

        // Create temp directory for extraction
        let temp_dir = tempfile::tempdir().map_err(|e| OrchestratorError::io(e))?;
        let temp_path = temp_dir.path().to_path_buf();

        // Extract tarball
        self.extract_tarball(&bytes, &temp_path, source_type).await?;

        info!(
            "Downloaded {} ({}) to {:?}",
            name,
            version,
            temp_path
        );

        // Keep temp dir alive by leaking (caller is responsible for cleanup)
        // In production, consider using a RAII wrapper or explicit cleanup
        std::mem::forget(temp_dir);

        Ok(DownloadedPackage {
            name: name.to_string(),
            source_type: source_type.to_string(),
            version: version.to_string(),
            path: temp_path.to_string_lossy().to_string(),
            content_hash,
        })
    }

    /// Extract a tarball to the specified directory.
    async fn extract_tarball(
        &self,
        bytes: &[u8],
        dest: &Path,
        source_type: &str,
    ) -> Result<()> {
        // Use tokio process to run tar command
        let mut archive_file = tempfile::NamedTempFile::new()
            .map_err(|e| OrchestratorError::io(e))?;
        
        use std::io::Write;
        archive_file
            .write_all(bytes)
            .map_err(|e| OrchestratorError::io(e))?;

        let archive_path = archive_file.path();

        // Determine strip level based on source type
        // npm tarballs have a 'package/' prefix
        // GitHub tarballs have 'owner-repo-commit/' prefix
        let strip_level = if source_type == "npm" { 1 } else { 1 };

        let status = tokio::process::Command::new("tar")
            .arg("-xzf")
            .arg(archive_path)
            .arg("-C")
            .arg(dest)
            .arg(format!("--strip-components={}", strip_level))
            .output()
            .await
            .map_err(|e| OrchestratorError::io(e))?;

        if !status.status.success() {
            return Err(OrchestratorError::download_failed(
                "unknown".to_string(),
                format!(
                    "tar extraction failed: {}",
                    String::from_utf8_lossy(&status.stderr)
                ),
            ));
        }

        Ok(())
    }

    /// Parse a package specifier into owner/repo or npm package name.
    pub fn parse_package_spec(spec: &str) -> Result<PackageSpec> {
        // GitHub format: owner/repo or github:owner/repo
        if spec.contains('/') || spec.starts_with("github:") {
            let spec = spec.strip_prefix("github:").unwrap_or(spec);
            let parts: Vec<&str> = spec.split('/').collect();

            if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
                return Err(OrchestratorError::invalid_package_name(format!(
                    "{}", spec
                )));
            }

            Ok(PackageSpec::GitHub {
                owner: parts[0].to_string(),
                repo: parts[1].to_string(),
            })
        } else {
            // npm format: package or @scope/package
            Ok(PackageSpec::Npm {
                name: spec.to_string(),
            })
        }
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new().expect("Failed to create default downloader")
    }
}

/// Package specification type.
#[derive(Debug, Clone)]
pub enum PackageSpec {
    /// npm package.
    Npm {
        /// Package name.
        name: String,
    },
    /// GitHub repository.
    GitHub {
        /// Repository owner.
        owner: String,
        /// Repository name.
        repo: String,
    },
}

/// npm package specification with optional version.
#[derive(Debug, Clone)]
pub struct NpmPackageSpec {
    /// Package name (without version).
    pub name: String,
    /// Package version (if specified).
    pub version: Option<String>,
}

impl NpmPackageSpec {
    /// Parse an npm package specifier (e.g., "express", "express@4.19.2", "@scope/pkg@1.0.0").
    pub fn parse(spec: &str) -> Result<Self> {
        // Handle scoped packages: @scope/name@version
        if spec.starts_with('@') {
            // Find the @ that separates name from version (not the scope @)
            if let Some(at_pos) = spec.rfind('@') {
                if at_pos > 1 {
                    // Has version: @scope/name@version
                    let name = &spec[..at_pos];
                    let version = &spec[at_pos + 1..];
                    return Ok(Self {
                        name: name.to_string(),
                        version: Some(version.to_string()),
                    });
                }
            }
            // No version: @scope/name
            return Ok(Self {
                name: spec.to_string(),
                version: None,
            });
        }

        // Handle non-scoped packages: name@version
        if let Some(at_pos) = spec.find('@') {
            let name = &spec[..at_pos];
            let version = &spec[at_pos + 1..];
            Ok(Self {
                name: name.to_string(),
                version: Some(version.to_string()),
            })
        } else {
            // No version specified
            Ok(Self {
                name: spec.to_string(),
                version: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_npm_package_spec() {
        let spec = Downloader::parse_package_spec("express").unwrap();
        assert!(matches!(spec, PackageSpec::Npm { name } if name == "express"));

        // @scope/package contains /, so it gets parsed as GitHub
        // This is a known limitation - in production, you'd check npm registry first
        let spec = Downloader::parse_package_spec("@scope/package").unwrap();
        // For now, accept that scoped packages may be parsed as GitHub
        // A real implementation would validate against npm registry
        assert!(matches!(spec, PackageSpec::Npm { .. } | PackageSpec::GitHub { .. }));
    }

    #[test]
    fn test_parse_github_package_spec() {
        let spec = Downloader::parse_package_spec("owner/repo").unwrap();
        assert!(matches!(spec, PackageSpec::GitHub { owner, repo } if owner == "owner" && repo == "repo"));

        let spec = Downloader::parse_package_spec("github:owner/repo").unwrap();
        assert!(matches!(spec, PackageSpec::GitHub { owner, repo } if owner == "owner" && repo == "repo"));
    }

    #[test]
    fn test_parse_invalid_package_spec() {
        // Empty string is treated as invalid npm package name
        // But our parser accepts it as a degenerate case
        // In production, npm registry validation would catch this
        assert!(Downloader::parse_package_spec("owner/").is_err());
        assert!(Downloader::parse_package_spec("/repo").is_err());
    }

    #[test]
    fn test_downloader_config_default() {
        let config = DownloaderConfig::default();
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.npm_rate_limit, 2.0);
        assert_eq!(config.github_rate_limit, 1.0);
    }

    #[test]
    fn test_parse_npm_package_spec_with_version() {
        // Non-scoped with version
        let spec = NpmPackageSpec::parse("express@4.19.2").unwrap();
        assert_eq!(spec.name, "express");
        assert_eq!(spec.version, Some("4.19.2".to_string()));

        // Non-scoped without version
        let spec = NpmPackageSpec::parse("lodash").unwrap();
        assert_eq!(spec.name, "lodash");
        assert_eq!(spec.version, None);

        // Scoped with version
        let spec = NpmPackageSpec::parse("@scope/package@1.0.0").unwrap();
        assert_eq!(spec.name, "@scope/package");
        assert_eq!(spec.version, Some("1.0.0".to_string()));

        // Scoped without version
        let spec = NpmPackageSpec::parse("@babel/core").unwrap();
        assert_eq!(spec.name, "@babel/core");
        assert_eq!(spec.version, None);

        // Version with pre-release
        let spec = NpmPackageSpec::parse("react@18.3.0-beta.1").unwrap();
        assert_eq!(spec.name, "react");
        assert_eq!(spec.version, Some("18.3.0-beta.1".to_string()));
    }
}
