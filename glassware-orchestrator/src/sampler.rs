//! Package Sampling Module
//!
//! Samples packages from npm registry based on categories and keywords.
//!
//! ## Usage
//!
//! ```bash
//! # Sample 100 packages from AI/ML category
//! glassware-orchestrator sample-packages --category ai-ml --samples 100
//!
//! # Sample from multiple categories
//! glassware-orchestrator sample-packages --category ai-ml --category native-build --samples 50
//!
//! # Save to file
//! glassware-orchestrator sample-packages --category ai-ml --samples 100 --output packages.txt
//! ```

use reqwest::Client;
use serde::Deserialize;
use std::collections::HashSet;
use std::io::{BufWriter, Write};
use std::path::Path;
use tracing::{debug, info, warn};

use crate::error::{OrchestratorError, Result};

/// npm search API endpoint
const NPM_SEARCH_URL: &str = "https://registry.npmjs.org/-/v1/search";

/// Package categories with associated keywords
pub struct PackageCategory {
    pub name: &'static str,
    pub keywords: &'static [&'static str],
}

/// Predefined package categories
pub const CATEGORIES: &[PackageCategory] = &[
    PackageCategory {
        name: "ai-ml",
        keywords: &["ai", "llm", "langchain", "openai", "anthropic", "huggingface", "agent"],
    },
    PackageCategory {
        name: "native-build",
        keywords: &["node-gyp", "bindings", "prebuild", "nan", "node-addon-api", "neon"],
    },
    PackageCategory {
        name: "install-scripts",
        keywords: &["preinstall", "postinstall", "install", "husky", "patch-package"],
    },
    PackageCategory {
        name: "web-frameworks",
        keywords: &["react", "vue", "angular", "svelte", "next", "nuxt", "gatsby"],
    },
    PackageCategory {
        name: "backend",
        keywords: &["express", "fastify", "koa", "hapi", "nest", "adonis"],
    },
    PackageCategory {
        name: "database",
        keywords: &["mongoose", "sequelize", "prisma", "typeorm", "knex", "pg"],
    },
    PackageCategory {
        name: "devtools",
        keywords: &["eslint", "prettier", "typescript", "babel", "webpack", "vite"],
    },
    PackageCategory {
        name: "testing",
        keywords: &["jest", "mocha", "vitest", "cypress", "playwright"],
    },
    PackageCategory {
        name: "utils",
        keywords: &["lodash", "async", "moment", "dayjs", "axios", "got"],
    },
    PackageCategory {
        name: "crypto",
        keywords: &["bcrypt", "jsonwebtoken", "jose", "node-forge", "crypto-js"],
    },
];

/// npm search response
#[derive(Debug, Clone, Deserialize)]
pub struct NpmSearchResponse {
    pub objects: Vec<NpmSearchObject>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NpmSearchObject {
    pub package: NpmSearchPackage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NpmSearchPackage {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

/// Package sampler
pub struct PackageSampler {
    client: Client,
}

impl PackageSampler {
    /// Create new sampler
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: Client::new(),
        })
    }

    /// Sample packages from categories
    pub async fn sample(
        &self,
        categories: &[&str],
        samples_per_category: usize,
    ) -> Result<Vec<String>> {
        let mut packages = HashSet::new();

        for category_name in categories {
            // Find category definition
            let category = CATEGORIES
                .iter()
                .find(|c| c.name == *category_name)
                .ok_or_else(|| {
                    OrchestratorError::validation_error(
                        format!("Unknown category: {}. Available: ai-ml, native-build, install-scripts, web-frameworks, backend, database, devtools, testing, utils, crypto", category_name),
                        Some("category"),
                    )
                })?;

            info!(
                "Sampling {} packages from category '{}' (keywords: {:?})",
                samples_per_category,
                category.name,
                category.keywords
            );

            // Search for each keyword
            for keyword in category.keywords {
                if packages.len() >= samples_per_category {
                    break;
                }

                debug!("Searching npm for keyword: {}", keyword);
                let results = self.search_npm(keyword, samples_per_category * 2).await?;

                for pkg in results {
                    if packages.len() >= samples_per_category {
                        break;
                    }
                    packages.insert(pkg.name);
                }
            }

            info!(
                "Sampled {} packages from category '{}'",
                packages.len(),
                category.name
            );
        }

        let result: Vec<String> = packages.into_iter().collect();
        info!("Total unique packages sampled: {}", result.len());
        Ok(result)
    }

    /// Search npm for packages
    async fn search_npm(&self, keyword: &str, size: usize) -> Result<Vec<NpmSearchPackage>> {
        let url = NPM_SEARCH_URL;
        let mut packages = Vec::new();

        let response = self
            .client
            .get(url)
            .query(&[("text", keyword), ("size", &size.to_string())])
            .send()
            .await
            .map_err(|e| OrchestratorError::http(e))?;

        if !response.status().is_success() {
            warn!("npm search returned status {} for keyword '{}'", response.status(), keyword);
            return Ok(packages);
        }

        let search_result: NpmSearchResponse = response
            .json()
            .await
            .map_err(|e| OrchestratorError::http(e))?;

        for obj in search_result.objects {
            packages.push(obj.package);
        }

        debug!("Found {} packages for keyword '{}'", packages.len(), keyword);
        Ok(packages)
    }

    /// Save package list to file
    pub fn save_to_file(&self, packages: &[String], output_path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                OrchestratorError::io_error(e, format!("Failed to create directory: {:?}", parent))
            })?;
        }

        let file = std::fs::File::create(output_path).map_err(|e| {
            OrchestratorError::io_error(e, format!("Failed to create file: {:?}", output_path))
        })?;

        let mut writer = BufWriter::new(file);
        for package in packages {
            writeln!(writer, "{}", package).map_err(|e| {
                OrchestratorError::io_error(e, "Failed to write package name".to_string())
            })?;
        }
        writer.flush().map_err(|e| {
            OrchestratorError::io_error(e, "Failed to flush output".to_string())
        })?;

        info!("Saved {} packages to {:?}", packages.len(), output_path);
        Ok(())
    }
}

impl Default for PackageSampler {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categories_defined() {
        assert!(!CATEGORIES.is_empty());
        assert!(CATEGORIES.iter().any(|c| c.name == "ai-ml"));
        assert!(CATEGORIES.iter().any(|c| c.name == "native-build"));
    }

    #[tokio::test]
    async fn test_sampler_creation() {
        let sampler = PackageSampler::new();
        assert!(sampler.is_ok());
    }
}
