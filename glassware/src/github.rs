//! GitHub API integration for repository downloading and searching.
//!
//! This module provides GitHub API integration with support for:
//! - Repository search
//! - Archive download and extraction
//! - Rate limiting (1 req/s without token, higher with token)
//! - Optional authentication token

use reqwest::{Client, StatusCode, header::{HeaderMap, HeaderValue}};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{debug, info, warn};

#[cfg(feature = "rate-limit")]
use governor::{Quota, RateLimiter};
#[cfg(feature = "rate-limit")]
use std::num::NonZeroU32;
#[cfg(feature = "rate-limit")]
use std::sync::Arc;

#[cfg(feature = "retry")]
use tokio_retry::{
    strategy::{jitter, ExponentialBackoff},
    Retry,
};

use crate::error::{OrchestratorError, Result};

/// GitHub search response.
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubSearchResponse {
    /// Total number of matching repositories.
    pub total_count: u32,
    /// Whether the search results are incomplete.
    #[serde(default)]
    pub incomplete_results: bool,
    /// Matching repositories.
    pub items: Vec<GitHubRepoSearchResult>,
}

/// Repository search result.
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRepoSearchResult {
    /// Full repository name (owner/repo).
    pub full_name: String,
    /// Repository description.
    #[serde(default)]
    pub description: Option<String>,
    /// Number of stars.
    #[serde(default)]
    pub stargazers_count: u32,
    /// Number of forks.
    #[serde(default)]
    pub forks_count: u32,
    /// Primary programming language.
    #[serde(default)]
    pub language: Option<String>,
    /// Repository URL.
    pub html_url: String,
    /// Whether the repository is archived.
    #[serde(default)]
    pub archived: bool,
    /// Default branch.
    #[serde(default)]
    pub default_branch: String,
}

/// GitHub repository archive information.
#[derive(Debug, Clone)]
pub struct GitHubArchiveInfo {
    /// Repository owner.
    pub owner: String,
    /// Repository name.
    pub repo: String,
    /// Reference (branch, tag, or commit).
    pub ref_name: String,
    /// Archive download URL.
    pub archive_url: String,
    /// Archive format (tarball or zipball).
    pub format: String,
}

/// Configuration for GitHub downloader.
#[derive(Debug, Clone)]
pub struct GitHubDownloaderConfig {
    /// GitHub API token (optional).
    pub token: Option<String>,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
    /// Maximum number of retries.
    pub max_retries: u32,
    /// Base delay for exponential backoff in milliseconds.
    pub retry_base_delay_ms: u64,
    /// Rate limit: requests per second (1 without token, 5000/hour with token).
    pub rate_limit: f32,
}

impl Default for GitHubDownloaderConfig {
    fn default() -> Self {
        Self {
            token: None,
            timeout_secs: 30,
            max_retries: 3,
            retry_base_delay_ms: 1000,
            rate_limit: 1.0, // Conservative default without token
        }
    }
}

/// GitHub repository downloader with API integration.
pub struct GitHubDownloader {
    client: Client,
    config: GitHubDownloaderConfig,
    #[cfg(feature = "rate-limit")]
    limiter: Arc<RateLimiter<governor::state::direct::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>>,
}

impl GitHubDownloader {
    /// Create a new GitHub downloader with default configuration.
    pub fn new() -> Result<Self> {
        Self::with_config(GitHubDownloaderConfig::default())
    }

    /// Create a new GitHub downloader with custom configuration.
    pub fn with_config(config: GitHubDownloaderConfig) -> Result<Self> {
        let mut client_builder = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .user_agent("glassware-orchestrator/0.1.0");

        // Add higher rate limit if token is provided
        let rate_limit = if config.token.is_some() {
            5000.0 / 3600.0 // ~1.39 req/s for authenticated users (5000/hour)
        } else {
            config.rate_limit // 1 req/s for unauthenticated
        };

        if let Some(ref token) = config.token {
            let mut headers = HeaderMap::new();
            headers.insert("Authorization", HeaderValue::from_str(&format!("token {}", token)).unwrap());
            client_builder = client_builder.default_headers(headers);
        }

        let client = client_builder
            .build()
            .map_err(|e| OrchestratorError::http(e))?;

        #[cfg(feature = "rate-limit")]
        let limiter = {
            let quota = Quota::per_second(NonZeroU32::new(rate_limit as u32).unwrap_or(NonZeroU32::new(1).unwrap()));
            Arc::new(RateLimiter::direct(quota))
        };

        Ok(Self {
            client,
            config,
            #[cfg(feature = "rate-limit")]
            limiter,
        })
    }

    /// Download a GitHub repository as an archive.
    pub async fn download_repo(&self, repo: &str) -> Result<PathBuf> {
        // Parse repo string (owner/repo or full URL)
        let (owner, repo_name) = self.parse_repo_identifier(repo)?;
        
        // Get default branch
        let repo_info = self.get_repo_info(&owner, &repo_name).await?;
        let ref_name = repo_info.default_branch
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("HEAD");

        // Download archive
        self.download_archive(&owner, &repo_name, ref_name, "tarball").await
    }

    /// Download a specific reference (branch, tag, or commit).
    pub async fn download_ref(&self, owner: &str, repo: &str, ref_name: &str) -> Result<PathBuf> {
        self.download_archive(owner, repo, ref_name, "tarball").await
    }

    /// Search GitHub repositories.
    pub async fn search_repos(&self, query: &str, max_results: usize) -> Result<Vec<String>> {
        #[cfg(feature = "rate-limit")]
        self.limiter.until_ready().await;

        let url = format!(
            "https://api.github.com/search/repositories?q={}&per_page={}",
            urlencoding::encode(query),
            std::cmp::min(max_results, 100) // GitHub API max per page
        );

        let fetch = || async {
            let response = self
                .client
                .get(&url)
                .send()
                .await
                .map_err(|e| OrchestratorError::http(e))?;

            if response.status() == StatusCode::FORBIDDEN {
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

            let search_response: GitHubSearchResponse = response
                .json()
                .await
                .map_err(|e| OrchestratorError::http(e))?;

            Ok(search_response)
        };

        let search_response = {
            #[cfg(feature = "retry")]
            {
                let retry_strategy = ExponentialBackoff::from_millis(self.config.retry_base_delay_ms)
                    .map(jitter)
                    .take(self.config.max_retries as usize);

                Retry::spawn(retry_strategy, fetch).await?
            }
            #[cfg(not(feature = "retry"))]
            {
                fetch().await?
            }
        };

        // Collect repository names up to max_results
        let repos: Vec<String> = search_response
            .items
            .into_iter()
            .take(max_results)
            .map(|item| item.full_name)
            .collect();

        info!("Found {} repositories for query: {}", repos.len(), query);

        Ok(repos)
    }

    /// Get repository information.
    pub async fn get_repo_info(&self, owner: &str, repo: &str) -> Result<crate::downloader::GitHubRepoInfo> {
        #[cfg(feature = "rate-limit")]
        self.limiter.until_ready().await;

        let url = format!("https://api.github.com/repos/{}/{}", owner, repo);

        let fetch = || async {
            let response = self
                .client
                .get(&url)
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

            let repo_info: crate::downloader::GitHubRepoInfo = response
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

    /// Download a repository archive.
    async fn download_archive(
        &self,
        owner: &str,
        repo: &str,
        ref_name: &str,
        format: &str,
    ) -> Result<PathBuf> {
        #[cfg(feature = "rate-limit")]
        self.limiter.until_ready().await;

        let archive_url = format!(
            "https://api.github.com/repos/{}/{}/{}/{}",
            owner, repo, format, ref_name
        );

        debug!("Downloading archive from: {}", archive_url);

        let fetch = || async {
            let response = self
                .client
                .get(&archive_url)
                .send()
                .await
                .map_err(|e| OrchestratorError::http(e))?;

            if response.status() == StatusCode::NOT_FOUND {
                return Err(OrchestratorError::not_found(format!(
                    "GitHub archive: {}/{}@{}",
                    owner, repo, ref_name
                )));
            }

            if response.status() == StatusCode::FORBIDDEN {
                return Err(OrchestratorError::github(
                    "GitHub API rate limit exceeded".to_string(),
                ));
            }

            if !response.status().is_success() {
                return Err(OrchestratorError::download_failed(
                    format!("{}/{}", owner, repo),
                    format!("Download failed: {}", response.status()),
                ));
            }

            let bytes = response
                .bytes()
                .await
                .map_err(|e| OrchestratorError::http(e))?;

            Ok(bytes)
        };

        let bytes = {
            #[cfg(feature = "retry")]
            {
                let retry_strategy = ExponentialBackoff::from_millis(self.config.retry_base_delay_ms)
                    .map(jitter)
                    .take(self.config.max_retries as usize);

                Retry::spawn(retry_strategy, fetch).await?
            }
            #[cfg(not(feature = "retry"))]
            {
                fetch().await?
            }
        };

        // Create temp directory for extraction
        let temp_dir = tempfile::tempdir().map_err(|e| OrchestratorError::io(e))?;
        let temp_path = temp_dir.path().to_path_buf();

        // Extract archive
        self.extract_archive(&bytes, &temp_path, format).await?;

        info!(
            "Downloaded GitHub repository {}/{}@{} to {:?}",
            owner, repo, ref_name, temp_path
        );

        // Keep temp dir alive by leaking (caller is responsible for cleanup)
        std::mem::forget(temp_dir);

        Ok(temp_path)
    }

    /// Extract a tarball or zipball archive.
    async fn extract_archive(&self, bytes: &[u8], dest: &Path, format: &str) -> Result<()> {
        let mut archive_file = tempfile::NamedTempFile::new()
            .map_err(|e| OrchestratorError::io(e))?;

        use std::io::Write;
        archive_file
            .write_all(bytes)
            .map_err(|e| OrchestratorError::io(e))?;

        let archive_path = archive_file.path();

        // GitHub archives have 'owner-repo-commit/' prefix, strip it
        let strip_level = 1;

        let status = if format == "tarball" {
            tokio::process::Command::new("tar")
                .arg("-xzf")
                .arg(archive_path)
                .arg("-C")
                .arg(dest)
                .arg(format!("--strip-components={}", strip_level))
                .output()
                .await
                .map_err(|e| OrchestratorError::io(e))?
        } else {
            // For zipball, use unzip
            tokio::process::Command::new("unzip")
                .arg("-q")
                .arg(archive_path)
                .arg("-d")
                .arg(dest)
                .output()
                .await
                .map_err(|e| OrchestratorError::io(e))?
        };

        if !status.status.success() {
            return Err(OrchestratorError::download_failed(
                "unknown".to_string(),
                format!(
                    "Archive extraction failed: {}",
                    String::from_utf8_lossy(&status.stderr)
                ),
            ));
        }

        Ok(())
    }

    /// Parse a repository identifier into owner and repo name.
    fn parse_repo_identifier(&self, repo: &str) -> Result<(String, String)> {
        // Handle full URL
        if repo.starts_with("https://github.com/") {
            let parts: Vec<&str> = repo
                .strip_prefix("https://github.com/")
                .unwrap()
                .trim_end_matches('/')
                .split('/')
                .collect();

            if parts.len() < 2 {
                return Err(OrchestratorError::invalid_package_name(format!(
                    "{}", repo
                )));
            }

            return Ok((parts[0].to_string(), parts[1].to_string()));
        }

        // Handle owner/repo format
        let parts: Vec<&str> = repo.split('/').collect();
        if parts.len() != 2 {
            return Err(OrchestratorError::invalid_package_name(format!(
                "{}", repo
            )));
        }

        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    /// Get the current rate limit status.
    pub async fn get_rate_limit_status(&self) -> Result<GitHubRateLimit> {
        let url = "https://api.github.com/rate_limit";

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| OrchestratorError::http(e))?;

        if !response.status().is_success() {
            return Err(OrchestratorError::github(format!(
                "Failed to get rate limit: {}",
                response.status()
            )));
        }

        let rate_limit: GitHubRateLimitResponse = response
            .json()
            .await
            .map_err(|e| OrchestratorError::http(e))?;

        Ok(GitHubRateLimit {
            core: rate_limit.resources.core,
            search: rate_limit.resources.search,
            graphql: rate_limit.resources.graphql,
        })
    }
}

impl Default for GitHubDownloader {
    fn default() -> Self {
        Self::new().expect("Failed to create default GitHub downloader")
    }
}

/// GitHub rate limit information.
#[derive(Debug, Clone)]
pub struct GitHubRateLimit {
    /// Core API rate limit.
    pub core: RateLimitInfo,
    /// Search API rate limit.
    pub search: RateLimitInfo,
    /// GraphQL API rate limit.
    pub graphql: RateLimitInfo,
}

/// Rate limit information for a specific API.
#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitInfo {
    /// Remaining requests.
    pub remaining: u32,
    /// Limit reset time (Unix timestamp).
    pub reset: u64,
    /// Maximum requests allowed.
    pub limit: u32,
}

/// GitHub rate limit API response.
#[derive(Debug, Clone, Deserialize)]
struct GitHubRateLimitResponse {
    resources: GitHubRateLimitResources,
}

#[derive(Debug, Clone, Deserialize)]
struct GitHubRateLimitResources {
    core: RateLimitInfo,
    search: RateLimitInfo,
    graphql: RateLimitInfo,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_repo_identifier_url() {
        let downloader = GitHubDownloader::new().unwrap();
        
        let (owner, repo) = downloader
            .parse_repo_identifier("https://github.com/owner/repo")
            .unwrap();
        
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_repo_identifier_short() {
        let downloader = GitHubDownloader::new().unwrap();
        
        let (owner, repo) = downloader
            .parse_repo_identifier("owner/repo")
            .unwrap();
        
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
    }

    #[test]
    fn test_parse_repo_identifier_invalid() {
        let downloader = GitHubDownloader::new().unwrap();
        
        assert!(downloader.parse_repo_identifier("invalid").is_err());
        assert!(downloader.parse_repo_identifier("owner/repo/extra").is_err());
    }

    #[test]
    fn test_github_downloader_config_default() {
        let config = GitHubDownloaderConfig::default();
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.max_retries, 3);
        assert!(config.token.is_none());
    }
}
