//! Campaign Intelligence Module
//!
//! This module provides campaign tracking and clustering capabilities for identifying
//! coordinated attack campaigns across multiple packages.
//!
//! ## Overview
//!
//! Campaign intelligence tracks:
//! - **Infrastructure reuse**: Shared domains, wallets, authors across packages
//! - **Code similarity**: Clustering packages by code patterns using MinHash
//! - **Temporal patterns**: Time-based clustering of package releases
//! - **Campaign classification**: Identifying known campaign types (GlassWorm, PhantomRaven, etc.)
//!
//! ## Example
//!
//! ```rust
//! use glassware_core::campaign::{CampaignIntelligence, AnalyzedPackage, PackageInfo};
//!
//! let mut intel = CampaignIntelligence::new();
//!
//! // Add analyzed packages
//! intel.add_package(pkg1);
//! intel.add_package(pkg2);
//!
//! // Get detected campaigns
//! for campaign in intel.get_campaigns() {
//!     println!("Campaign: {}", campaign.id);
//!     println!("Packages: {:?}", campaign.packages);
//!     println!("Confidence: {:.2}", campaign.confidence);
//! }
//! ```

use crate::finding::Finding;
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

/// A campaign tracks related malicious packages
#[derive(Debug, Clone)]
pub struct Campaign {
    /// Campaign identifier (e.g., "GlassWorm-Wave5")
    pub id: String,

    /// Packages in this campaign
    pub packages: Vec<PackageInfo>,

    /// Shared infrastructure (domains, wallets, authors)
    pub infrastructure: Infrastructure,

    /// Time range of activity
    pub time_range: TimeRange,

    /// Confidence this is a coordinated campaign (0.0-1.0)
    pub confidence: f32,

    /// Campaign classification
    pub classification: CampaignType,
}

impl Campaign {
    /// Create a new campaign
    pub fn new(
        id: String,
        packages: Vec<PackageInfo>,
        infrastructure: Infrastructure,
        classification: CampaignType,
    ) -> Self {
        let time_range = TimeRange::from_packages(&packages);
        let confidence = Self::calculate_confidence(&packages, &infrastructure);

        Self {
            id,
            packages,
            infrastructure,
            time_range,
            confidence,
            classification,
        }
    }

    /// Calculate confidence based on shared infrastructure and package count
    fn calculate_confidence(packages: &[PackageInfo], infrastructure: &Infrastructure) -> f32 {
        let mut confidence = 0.0;

        // Base confidence from package count
        confidence += (packages.len() as f32 * 0.1).min(0.3);

        // Confidence from shared domains
        if !infrastructure.domains.is_empty() {
            confidence += 0.25;
        }

        // Confidence from shared wallets
        if !infrastructure.wallets.is_empty() {
            confidence += 0.25;
        }

        // Confidence from shared authors
        if !infrastructure.authors.is_empty() {
            confidence += 0.15;
        }

        // Confidence from code clusters
        if !infrastructure.code_clusters.is_empty() {
            confidence += 0.15;
        }

        confidence.min(1.0)
    }

    /// Get all unique package names in this campaign
    pub fn package_names(&self) -> Vec<&str> {
        self.packages.iter().map(|p| p.name.as_str()).collect()
    }

    /// Check if a package belongs to this campaign
    pub fn contains_package(&self, package_name: &str) -> bool {
        self.packages.iter().any(|p| p.name == package_name)
    }
}

/// Infrastructure shared across packages in a campaign
#[derive(Debug, Clone, Default)]
pub struct Infrastructure {
    /// Shared domains/C2 servers
    pub domains: Vec<String>,

    /// Shared blockchain wallets
    pub wallets: Vec<String>,

    /// Shared author identities
    pub authors: Vec<String>,

    /// Code similarity clusters
    pub code_clusters: Vec<CodeCluster>,
}

impl Infrastructure {
    /// Create new infrastructure tracking
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a domain to the infrastructure
    pub fn add_domain(&mut self, domain: String) {
        if !self.domains.contains(&domain) {
            self.domains.push(domain);
        }
    }

    /// Add a wallet to the infrastructure
    pub fn add_wallet(&mut self, wallet: String) {
        if !self.wallets.contains(&wallet) {
            self.wallets.push(wallet);
        }
    }

    /// Add an author to the infrastructure
    pub fn add_author(&mut self, author: String) {
        if !self.authors.contains(&author) {
            self.authors.push(author);
        }
    }

    /// Add a code cluster
    pub fn add_code_cluster(&mut self, cluster: CodeCluster) {
        self.code_clusters.push(cluster);
    }

    /// Check if infrastructure is empty
    pub fn is_empty(&self) -> bool {
        self.domains.is_empty()
            && self.wallets.is_empty()
            && self.authors.is_empty()
            && self.code_clusters.is_empty()
    }

    /// Get total count of infrastructure items
    pub fn item_count(&self) -> usize {
        self.domains.len() + self.wallets.len() + self.authors.len() + self.code_clusters.len()
    }
}

/// A code similarity cluster groups packages with similar code patterns
#[derive(Debug, Clone)]
pub struct CodeCluster {
    /// Cluster identifier
    pub id: String,

    /// Package names in this cluster
    pub packages: Vec<String>,

    /// Similarity threshold used for clustering (0.0-1.0)
    pub threshold: f32,

    /// Representative signature (MinHash hash)
    pub signature: Vec<u64>,
}

impl CodeCluster {
    /// Create a new code cluster
    pub fn new(id: String, packages: Vec<String>, threshold: f32, signature: Vec<u64>) -> Self {
        Self {
            id,
            packages,
            threshold,
            signature,
        }
    }

    /// Check if a package belongs to this cluster
    pub fn contains(&self, package_name: &str) -> bool {
        self.packages.iter().any(|p| p == package_name)
    }

    /// Get the size of this cluster
    pub fn size(&self) -> usize {
        self.packages.len()
    }
}

/// Time range of campaign activity
#[derive(Debug, Clone)]
pub struct TimeRange {
    /// Earliest package publish time (Unix timestamp in seconds)
    pub start: u64,

    /// Latest package publish time (Unix timestamp in seconds)
    pub end: u64,
}

impl TimeRange {
    /// Create a new time range
    pub fn new(start: u64, end: u64) -> Self {
        Self { start, end }
    }

    /// Create time range from packages
    pub fn from_packages(packages: &[PackageInfo]) -> Self {
        if packages.is_empty() {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            return Self::new(now, now);
        }

        let start = packages.iter().map(|p| p.publish_time).min().unwrap_or(0);
        let end = packages.iter().map(|p| p.publish_time).max().unwrap_or(0);

        Self::new(start, end)
    }

    /// Get duration in seconds
    pub fn duration_secs(&self) -> u64 {
        self.end.saturating_sub(self.start)
    }

    /// Get duration in days
    pub fn duration_days(&self) -> f64 {
        self.duration_secs() as f64 / 86400.0
    }

    /// Check if a timestamp falls within this range
    pub fn contains(&self, timestamp: u64) -> bool {
        timestamp >= self.start && timestamp <= self.end
    }
}

/// Campaign type classification
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum CampaignType {
    /// Unicode steganography campaign (GlassWorm)
    GlassWorm,

    /// Remote dependency delivery (PhantomRaven)
    PhantomRaven,

    /// Python repo injection (ForceMemo)
    ForceMemo,

    /// Chrome extension RAT
    ChromeRAT,

    /// Self-propagating worm (Shai-Hulud)
    ShaiHulud,

    /// Sandbox evasion with time-gates (SANDWORM_MODE)
    SandwormMode,

    /// Unknown/new campaign
    Unknown(String),
}

impl CampaignType {
    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            CampaignType::GlassWorm => {
                "Unicode steganography with decoder functions and blockchain C2"
            }
            CampaignType::PhantomRaven => {
                "Remote Dynamic Dependencies (RDD) with URL-based delivery"
            }
            CampaignType::ForceMemo => {
                "Python repository injection via forcememo package"
            }
            CampaignType::ChromeRAT => {
                "Chrome extension remote access trojan"
            }
            CampaignType::ShaiHulud => {
                "Self-propagating worm via npm token theft"
            }
            CampaignType::SandwormMode => {
                "Sandbox evasion with time-gates and MCP injection"
            }
            CampaignType::Unknown(_name) => "Unknown or new campaign type",
        }
    }

    /// Get typical severity for this campaign type
    pub fn typical_severity(&self) -> crate::finding::Severity {
        use crate::finding::Severity;

        match self {
            CampaignType::GlassWorm => Severity::Critical,
            CampaignType::PhantomRaven => Severity::High,
            CampaignType::ForceMemo => Severity::High,
            CampaignType::ChromeRAT => Severity::Critical,
            CampaignType::ShaiHulud => Severity::Critical,
            CampaignType::SandwormMode => Severity::Critical,
            CampaignType::Unknown(_) => Severity::Medium,
        }
    }

    /// Try to classify from indicators
    pub fn from_indicators(
        has_unicode_stego: bool,
        has_blockchain_c2: bool,
        has_rdd: bool,
        has_forcememo: bool,
        has_time_delay: bool,
        has_mcp_injection: bool,
    ) -> Self {
        if has_unicode_stego && has_blockchain_c2 {
            return CampaignType::GlassWorm;
        }

        if has_rdd {
            return CampaignType::PhantomRaven;
        }

        if has_forcememo {
            return CampaignType::ForceMemo;
        }

        if has_time_delay && has_mcp_injection {
            return CampaignType::SandwormMode;
        }

        if has_unicode_stego && !has_blockchain_c2 {
            // Could be Shai-Hulud or generic
            return CampaignType::ShaiHulud;
        }

        CampaignType::Unknown("unclassified".to_string())
    }
}

/// Information about an analyzed package
#[derive(Debug, Clone)]
pub struct PackageInfo {
    /// Package name
    pub name: String,

    /// Package version
    pub version: Option<String>,

    /// Package author(s)
    pub authors: Vec<String>,

    /// Publish time (Unix timestamp in seconds)
    pub publish_time: u64,

    /// Domains found in package
    pub domains: Vec<String>,

    /// Blockchain wallets found in package
    pub wallets: Vec<String>,

    /// Code signature (MinHash hash)
    pub code_signature: Vec<u64>,

    /// Findings from scanning
    pub findings: Vec<Finding>,

    /// File paths in package
    pub files: Vec<String>,
}

impl PackageInfo {
    /// Create a new package info
    pub fn new(name: String, publish_time: u64) -> Self {
        Self {
            name,
            version: None,
            authors: Vec::new(),
            publish_time,
            domains: Vec::new(),
            wallets: Vec::new(),
            code_signature: Vec::new(),
            findings: Vec::new(),
            files: Vec::new(),
        }
    }

    /// Set package version
    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    /// Set package authors
    pub fn with_authors(mut self, authors: Vec<String>) -> Self {
        self.authors = authors;
        self
    }

    /// Add a domain
    pub fn add_domain(mut self, domain: String) -> Self {
        self.domains.push(domain);
        self
    }

    /// Add a wallet
    pub fn add_wallet(mut self, wallet: String) -> Self {
        self.wallets.push(wallet);
        self
    }

    /// Set code signature
    pub fn with_code_signature(mut self, signature: Vec<u64>) -> Self {
        self.code_signature = signature;
        self
    }

    /// Set findings
    pub fn with_findings(mut self, findings: Vec<Finding>) -> Self {
        self.findings = findings;
        self
    }

    /// Add a file
    pub fn add_file(mut self, file: String) -> Self {
        self.files.push(file);
        self
    }
}

/// An analyzed package with full scan results
#[derive(Debug, Clone)]
pub struct AnalyzedPackage {
    /// Package information
    pub info: PackageInfo,

    /// Raw source code (for similarity analysis)
    pub source_code: String,

    /// Scan timestamp (Unix timestamp in seconds)
    pub scan_time: u64,
}

impl AnalyzedPackage {
    /// Create a new analyzed package
    pub fn new(name: String, source_code: String, publish_time: u64) -> Self {
        let scan_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            info: PackageInfo::new(name, publish_time),
            source_code,
            scan_time,
        }
    }

    /// Set package info
    pub fn with_info(mut self, info: PackageInfo) -> Self {
        self.info = info;
        self
    }
}

/// Campaign intelligence tracker
///
/// Tracks packages, detects campaigns from shared infrastructure and code similarity
pub struct CampaignIntelligence {
    /// Analyzed packages
    packages: Vec<AnalyzedPackage>,

    /// Detected campaigns
    campaigns: Vec<Campaign>,

    /// Infrastructure tracker
    infra_tracker: InfrastructureTracker,

    /// Code similarity analyzer
    code_similarity: CodeSimilarity,

    /// Campaign counter for ID generation
    campaign_counter: u32,
}

impl CampaignIntelligence {
    /// Create a new campaign intelligence tracker
    pub fn new() -> Self {
        Self {
            packages: Vec::new(),
            campaigns: Vec::new(),
            infra_tracker: InfrastructureTracker::new(),
            code_similarity: CodeSimilarity::new(),
            campaign_counter: 0,
        }
    }

    /// Add an analyzed package
    pub fn add_package(&mut self, pkg: AnalyzedPackage) {
        // Extract infrastructure from package
        let infra = self.extract_infrastructure(&pkg);

        // Track infrastructure usage
        self.infra_tracker.track(&pkg.info.name, &infra);

        // Add to code similarity analyzer
        self.code_similarity
            .add_package(&pkg.info.name, &pkg.source_code);

        // Add to packages list
        self.packages.push(pkg);

        // Detect campaigns
        self.detect_campaigns();
    }

    /// Extract infrastructure from a package
    fn extract_infrastructure(&self, pkg: &AnalyzedPackage) -> Infrastructure {
        let mut infra = Infrastructure::new();

        // Add authors
        for author in &pkg.info.authors {
            infra.add_author(author.clone());
        }

        // Add domains
        for domain in &pkg.info.domains {
            infra.add_domain(domain.clone());
        }

        // Add wallets
        for wallet in &pkg.info.wallets {
            infra.add_wallet(wallet.clone());
        }

        infra
    }

    /// Detect campaigns from package clusters
    fn detect_campaigns(&mut self) {
        self.campaigns.clear();

        // Cluster by shared infrastructure
        let infra_clusters = self.infra_tracker.get_clusters();

        // Cluster by code similarity
        let code_clusters = self.code_similarity.cluster(0.7);

        // Build campaigns from infrastructure clusters
        for (_idx, infra_cluster) in infra_clusters.iter().enumerate() {
            let packages: Vec<PackageInfo> = infra_cluster
                .packages
                .iter()
                .filter_map(|name| {
                    self.packages
                        .iter()
                        .find(|p| p.info.name == *name)
                        .map(|p| p.info.clone())
                })
                .collect();

            if packages.len() >= 2 {
                // Get shared infrastructure
                let mut infra = Infrastructure::new();
                for pkg in &packages {
                    for domain in &pkg.domains {
                        infra.add_domain(domain.clone());
                    }
                    for wallet in &pkg.wallets {
                        infra.add_wallet(wallet.clone());
                    }
                    for author in &pkg.authors {
                        infra.add_author(author.clone());
                    }
                }

                // Add code clusters for these packages
                for code_cluster in &code_clusters {
                    let matching_pkgs: Vec<String> = code_cluster
                        .packages
                        .iter()
                        .filter(|name| infra_cluster.packages.contains(name))
                        .cloned()
                        .collect();

                    if matching_pkgs.len() >= 2 {
                        infra.add_code_cluster(CodeCluster::new(
                            format!("code_{}", code_cluster.id),
                            matching_pkgs,
                            code_cluster.threshold,
                            code_cluster.signature.clone(),
                        ));
                    }
                }

                // Classify campaign type
                let classification = self.classify_campaign(&packages);

                // Create campaign
                let campaign = Campaign::new(
                    self.generate_campaign_id(&classification),
                    packages,
                    infra,
                    classification,
                );

                self.campaigns.push(campaign);
            }
        }

        // Also create campaigns from pure code similarity clusters (no shared infra)
        for code_cluster in &code_clusters {
            if code_cluster.size() >= 3 {
                // Check if this cluster is already covered by infrastructure clusters
                let already_covered = self.campaigns.iter().any(|c| {
                    c.infrastructure
                        .code_clusters
                        .iter()
                        .any(|cc| cc.id.starts_with(&format!("code_{}", code_cluster.id)))
                });

                if !already_covered {
                    let packages: Vec<PackageInfo> = code_cluster
                        .packages
                        .iter()
                        .filter_map(|name| {
                            self.packages
                                .iter()
                                .find(|p| p.info.name == *name)
                                .map(|p| p.info.clone())
                        })
                        .collect();

                    if packages.len() >= 3 {
                        let mut infra = Infrastructure::new();
                        infra.add_code_cluster(CodeCluster::new(
                            code_cluster.id.clone(),
                            code_cluster.packages.clone(),
                            code_cluster.threshold,
                            code_cluster.signature.clone(),
                        ));

                        let classification = self.classify_campaign(&packages);

                        let campaign = Campaign::new(
                            self.generate_campaign_id(&classification),
                            packages,
                            infra,
                            classification,
                        );

                        self.campaigns.push(campaign);
                    }
                }
            }
        }
    }

    /// Classify campaign type based on package indicators
    fn classify_campaign(&self, packages: &[PackageInfo]) -> CampaignType {
        let mut has_unicode_stego = false;
        let mut has_blockchain_c2 = false;
        let mut has_rdd = false;
        let mut has_forcememo = false;
        let mut has_time_delay = false;
        let mut has_mcp_injection = false;

        for pkg in packages {
            for finding in &pkg.findings {
                use crate::finding::DetectionCategory;

                match finding.category {
                    DetectionCategory::SteganoPayload
                    | DetectionCategory::InvisibleCharacter
                    | DetectionCategory::DecoderFunction => {
                        has_unicode_stego = true;
                    }
                    DetectionCategory::BlockchainC2 => {
                        has_blockchain_c2 = true;
                    }
                    DetectionCategory::RddAttack => {
                        has_rdd = true;
                    }
                    DetectionCategory::ForceMemoPython => {
                        has_forcememo = true;
                    }
                    DetectionCategory::TimeDelaySandboxEvasion => {
                        has_time_delay = true;
                    }
                    _ => {}
                }
            }

            // Check for MCP injection in domains/files
            for domain in &pkg.domains {
                if domain.contains("mcp") || domain.contains("claude") {
                    has_mcp_injection = true;
                }
            }
        }

        CampaignType::from_indicators(
            has_unicode_stego,
            has_blockchain_c2,
            has_rdd,
            has_forcememo,
            has_time_delay,
            has_mcp_injection,
        )
    }

    /// Generate a campaign ID
    fn generate_campaign_id(&mut self, campaign_type: &CampaignType) -> String {
        self.campaign_counter += 1;

        let type_prefix = match campaign_type {
            CampaignType::GlassWorm => "GlassWorm",
            CampaignType::PhantomRaven => "PhantomRaven",
            CampaignType::ForceMemo => "ForceMemo",
            CampaignType::ChromeRAT => "ChromeRAT",
            CampaignType::ShaiHulud => "ShaiHulud",
            CampaignType::SandwormMode => "SANDWORM",
            CampaignType::Unknown(_) => "Unknown",
        };

        format!("{}-Wave{}", type_prefix, self.campaign_counter)
    }

    /// Get detected campaigns
    pub fn get_campaigns(&self) -> &[Campaign] {
        &self.campaigns
    }

    /// Check if package belongs to known campaign
    pub fn get_package_campaign(&self, package_name: &str) -> Option<&Campaign> {
        self.campaigns
            .iter()
            .find(|c| c.contains_package(package_name))
    }

    /// Get all packages
    pub fn get_packages(&self) -> &[AnalyzedPackage] {
        &self.packages
    }

    /// Get infrastructure statistics
    pub fn get_infrastructure_stats(&self) -> InfrastructureStats {
        self.infra_tracker.get_reuse_stats()
    }

    /// Get code similarity between two packages
    pub fn get_code_similarity(&self, pkg1: &str, pkg2: &str) -> f32 {
        self.code_similarity.similarity(pkg1, pkg2)
    }

    /// Find packages related to a given package
    pub fn find_related_packages(&self, package_name: &str) -> Vec<String> {
        self.infra_tracker.find_related(package_name)
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.packages.clear();
        self.campaigns.clear();
        self.infra_tracker = InfrastructureTracker::new();
        self.code_similarity = CodeSimilarity::new();
        self.campaign_counter = 0;
    }
}

impl Default for CampaignIntelligence {
    fn default() -> Self {
        Self::new()
    }
}

/// Infrastructure reuse statistics
#[derive(Debug, Clone, Default)]
pub struct InfrastructureStats {
    /// Total packages tracked
    pub total_packages: usize,

    /// Unique domains found
    pub unique_domains: usize,

    /// Unique wallets found
    pub unique_wallets: usize,

    /// Unique authors found
    pub unique_authors: usize,

    /// Domains used by multiple packages
    pub reused_domains: usize,

    /// Wallets used by multiple packages
    pub reused_wallets: usize,

    /// Authors with multiple packages
    pub reused_authors: usize,

    /// Average packages per domain
    pub avg_packages_per_domain: f32,

    /// Average packages per wallet
    pub avg_packages_per_wallet: f32,
}

/// Infrastructure tracker - tracks infrastructure usage across packages
pub struct InfrastructureTracker {
    /// Domain → packages mapping
    domain_index: HashMap<String, HashSet<String>>,

    /// Wallet → packages mapping
    wallet_index: HashMap<String, HashSet<String>>,

    /// Author → packages mapping
    author_index: HashMap<String, HashSet<String>>,
}

impl InfrastructureTracker {
    /// Create a new infrastructure tracker
    pub fn new() -> Self {
        Self {
            domain_index: HashMap::new(),
            wallet_index: HashMap::new(),
            author_index: HashMap::new(),
        }
    }

    /// Track infrastructure usage for a package
    pub fn track(&mut self, package: &str, infrastructure: &Infrastructure) {
        // Update domain index
        for domain in &infrastructure.domains {
            self.domain_index
                .entry(domain.clone())
                .or_default()
                .insert(package.to_string());
        }

        // Update wallet index
        for wallet in &infrastructure.wallets {
            self.wallet_index
                .entry(wallet.clone())
                .or_default()
                .insert(package.to_string());
        }

        // Update author index
        for author in &infrastructure.authors {
            self.author_index
                .entry(author.clone())
                .or_default()
                .insert(package.to_string());
        }
    }

    /// Find packages sharing infrastructure with the given package
    pub fn find_related(&self, package: &str) -> Vec<String> {
        let mut related = HashSet::new();

        // Find packages sharing domains
        for (_, pkgs) in &self.domain_index {
            if pkgs.contains(package) {
                related.extend(pkgs.iter().filter(|p| *p != package).cloned());
            }
        }

        // Find packages sharing wallets
        for (_, pkgs) in &self.wallet_index {
            if pkgs.contains(package) {
                related.extend(pkgs.iter().filter(|p| *p != package).cloned());
            }
        }

        // Find packages sharing authors
        for (_, pkgs) in &self.author_index {
            if pkgs.contains(package) {
                related.extend(pkgs.iter().filter(|p| *p != package).cloned());
            }
        }

        related.into_iter().collect()
    }

    /// Get infrastructure reuse statistics
    pub fn get_reuse_stats(&self) -> InfrastructureStats {
        let total_packages = self
            .domain_index
            .values()
            .chain(self.wallet_index.values())
            .chain(self.author_index.values())
            .flatten()
            .collect::<HashSet<_>>()
            .len();

        let reused_domains = self
            .domain_index
            .values()
            .filter(|pkgs| pkgs.len() > 1)
            .count();

        let reused_wallets = self
            .wallet_index
            .values()
            .filter(|pkgs| pkgs.len() > 1)
            .count();

        let reused_authors = self
            .author_index
            .values()
            .filter(|pkgs| pkgs.len() > 1)
            .count();

        let total_domain_usage: usize = self.domain_index.values().map(|v| v.len()).sum();
        let total_wallet_usage: usize = self.wallet_index.values().map(|v| v.len()).sum();

        InfrastructureStats {
            total_packages,
            unique_domains: self.domain_index.len(),
            unique_wallets: self.wallet_index.len(),
            unique_authors: self.author_index.len(),
            reused_domains,
            reused_wallets,
            reused_authors,
            avg_packages_per_domain: if self.domain_index.is_empty() {
                0.0
            } else {
                total_domain_usage as f32 / self.domain_index.len() as f32
            },
            avg_packages_per_wallet: if self.wallet_index.is_empty() {
                0.0
            } else {
                total_wallet_usage as f32 / self.wallet_index.len() as f32
            },
        }
    }

    /// Get clusters of packages sharing infrastructure
    pub fn get_clusters(&self) -> Vec<InfrastructureCluster> {
        let mut clusters: Vec<InfrastructureCluster> = Vec::new();
        let _assigned: HashSet<String> = HashSet::new();

        // Cluster by shared domains
        for (domain, pkgs) in &self.domain_index {
            if pkgs.len() > 1 {
                let cluster_pkgs: HashSet<String> = pkgs.iter().cloned().collect();

                // Merge with existing cluster if there's overlap
                let mut merged = false;
                for cluster in &mut clusters {
                    if cluster.packages.iter().any(|p| cluster_pkgs.contains(p)) {
                        cluster.packages.extend(cluster_pkgs.iter().cloned());
                        cluster
                            .shared_infra
                            .insert(format!("domain:{}", domain));
                        merged = true;
                        break;
                    }
                }

                if !merged {
                    let mut cluster = InfrastructureCluster::new();
                    cluster.packages = cluster_pkgs.into_iter().collect();
                    cluster.shared_infra.insert(format!("domain:{}", domain));
                    clusters.push(cluster);
                }
            }
        }

        // Cluster by shared wallets
        for (wallet, pkgs) in &self.wallet_index {
            if pkgs.len() > 1 {
                let cluster_pkgs: HashSet<String> = pkgs.iter().cloned().collect();

                // Merge with existing cluster if there's overlap
                let mut merged = false;
                for cluster in &mut clusters {
                    if cluster.packages.iter().any(|p| cluster_pkgs.contains(p)) {
                        cluster.packages.extend(cluster_pkgs.iter().cloned());
                        cluster
                            .shared_infra
                            .insert(format!("wallet:{}", wallet));
                        merged = true;
                        break;
                    }
                }

                if !merged {
                    let mut cluster = InfrastructureCluster::new();
                    cluster.packages = cluster_pkgs.into_iter().collect();
                    cluster.shared_infra.insert(format!("wallet:{}", wallet));
                    clusters.push(cluster);
                }
            }
        }

        // Cluster by shared authors
        for (author, pkgs) in &self.author_index {
            if pkgs.len() > 1 {
                let cluster_pkgs: HashSet<String> = pkgs.iter().cloned().collect();

                // Merge with existing cluster if there's overlap
                let mut merged = false;
                for cluster in &mut clusters {
                    if cluster.packages.iter().any(|p| cluster_pkgs.contains(p)) {
                        cluster.packages.extend(cluster_pkgs.iter().cloned());
                        cluster
                            .shared_infra
                            .insert(format!("author:{}", author));
                        merged = true;
                        break;
                    }
                }

                if !merged {
                    let mut cluster = InfrastructureCluster::new();
                    cluster.packages = cluster_pkgs.into_iter().collect();
                    cluster.shared_infra.insert(format!("author:{}", author));
                    clusters.push(cluster);
                }
            }
        }

        // Deduplicate package lists in clusters
        for cluster in &mut clusters {
            cluster.packages.sort();
            cluster.packages.dedup();
        }

        clusters
    }

    /// Get all domains tracked
    pub fn get_domains(&self) -> Vec<&String> {
        self.domain_index.keys().collect()
    }

    /// Get all wallets tracked
    pub fn get_wallets(&self) -> Vec<&String> {
        self.wallet_index.keys().collect()
    }

    /// Get all authors tracked
    pub fn get_authors(&self) -> Vec<&String> {
        self.author_index.keys().collect()
    }

    /// Get packages for a domain
    pub fn get_packages_for_domain(&self, domain: &str) -> Vec<&String> {
        self.domain_index
            .get(domain)
            .map(|pkgs| pkgs.iter().collect())
            .unwrap_or_default()
    }

    /// Get packages for a wallet
    pub fn get_packages_for_wallet(&self, wallet: &str) -> Vec<&String> {
        self.wallet_index
            .get(wallet)
            .map(|pkgs| pkgs.iter().collect())
            .unwrap_or_default()
    }

    /// Get packages for an author
    pub fn get_packages_for_author(&self, author: &str) -> Vec<&String> {
        self.author_index
            .get(author)
            .map(|pkgs| pkgs.iter().collect())
            .unwrap_or_default()
    }
}

impl Default for InfrastructureTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// A cluster of packages sharing infrastructure
#[derive(Debug, Clone, Default)]
pub struct InfrastructureCluster {
    /// Package names in this cluster
    pub packages: Vec<String>,

    /// Shared infrastructure items (e.g., "domain:example.com", "wallet:abc123")
    pub shared_infra: HashSet<String>,
}

impl InfrastructureCluster {
    /// Create a new infrastructure cluster
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the size of this cluster
    pub fn size(&self) -> usize {
        self.packages.len()
    }

    /// Check if a package belongs to this cluster
    pub fn contains(&self, package_name: &str) -> bool {
        self.packages.iter().any(|p| p == package_name)
    }
}

/// Code similarity analyzer using MinHash
pub struct CodeSimilarity {
    /// Package name → MinHash signature mapping
    signatures: HashMap<String, Vec<u64>>,

    /// Number of hash functions in MinHash
    num_hashes: usize,

    /// N-gram size for shingling
    ngram_size: usize,
}

impl CodeSimilarity {
    /// Create a new code similarity analyzer
    pub fn new() -> Self {
        Self {
            signatures: HashMap::new(),
            num_hashes: 128,
            ngram_size: 3,
        }
    }

    /// Create with custom parameters
    pub fn with_params(num_hashes: usize, ngram_size: usize) -> Self {
        Self {
            signatures: HashMap::new(),
            num_hashes,
            ngram_size,
        }
    }

    /// Add a package for similarity analysis
    pub fn add_package(&mut self, name: &str, source_code: &str) {
        let signature = self.compute_minhash(source_code);
        self.signatures.insert(name.to_string(), signature);
    }

    /// Compute MinHash signature for source code
    fn compute_minhash(&self, source_code: &str) -> Vec<u64> {
        // Generate n-grams (shingles)
        let shingles: Vec<String> = self.generate_shingles(source_code);

        // Compute MinHash signature
        let mut signature = vec![u64::MAX; self.num_hashes];

        for shingle in &shingles {
            // Compute multiple hash values for MinHash
            for i in 0..self.num_hashes {
                let h = self.hash_with_seed(shingle, i as u64);
                signature[i] = signature[i].min(h);
            }
        }

        signature
    }

    /// Generate n-grams from source code
    fn generate_shingles(&self, code: &str) -> Vec<String> {
        // Normalize whitespace
        let normalized: String = code
            .chars()
            .map(|c| if c.is_whitespace() { ' ' } else { c })
            .collect();

        // Generate n-grams
        let chars: Vec<char> = normalized.chars().collect();
        let mut shingles = Vec::new();

        if chars.len() >= self.ngram_size {
            for i in 0..=chars.len() - self.ngram_size {
                let shingle: String = chars[i..i + self.ngram_size].iter().collect();
                shingles.push(shingle);
            }
        }

        shingles
    }

    /// Hash a shingle
    fn hash_shingle(&self, shingle: &str) -> u64 {
        // Simple hash function (FNV-1a)
        const FNV_OFFSET: u64 = 14695981039346656037;
        const FNV_PRIME: u64 = 1099511628211;

        let mut hash = FNV_OFFSET;
        for byte in shingle.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash
    }

    /// Hash with seed for multiple hash functions
    fn hash_with_seed(&self, shingle: &str, seed: u64) -> u64 {
        // Combine shingle hash with seed
        let base_hash = self.hash_shingle(shingle);
        base_hash.wrapping_add(seed.wrapping_mul(0x9E3779B97F4A7C15))
    }

    /// Compute similarity between two packages
    pub fn similarity(&self, pkg1: &str, pkg2: &str) -> f32 {
        match (self.signatures.get(pkg1), self.signatures.get(pkg2)) {
            (Some(sig1), Some(sig2)) => {
                // Jaccard similarity on MinHash signatures
                let matches = sig1
                    .iter()
                    .zip(sig2.iter())
                    .filter(|(a, b)| a == b)
                    .count() as f32;
                matches / self.num_hashes as f32
            }
            _ => 0.0,
        }
    }

    /// Cluster packages by code similarity
    pub fn cluster(&self, threshold: f32) -> Vec<CodeCluster> {
        let mut clusters = Vec::new();
        let mut assigned = HashSet::new();

        let package_names: Vec<&String> = self.signatures.keys().collect();

        for (i, pkg1) in package_names.iter().enumerate() {
            if assigned.contains(*pkg1) {
                continue;
            }

            let mut cluster_pkgs = vec![(*pkg1).clone()];

            for pkg2 in package_names.iter().skip(i + 1) {
                if assigned.contains(*pkg2) {
                    continue;
                }

                let sim = self.similarity(pkg1, pkg2);
                if sim >= threshold {
                    cluster_pkgs.push((*pkg2).clone());
                }
            }

            if cluster_pkgs.len() > 1 {
                // Get representative signature (use first package)
                let signature = self.signatures.get(*pkg1).cloned().unwrap_or_default();

                for pkg in &cluster_pkgs {
                    assigned.insert(pkg.clone());
                }

                clusters.push(CodeCluster::new(
                    format!("cluster_{}", clusters.len()),
                    cluster_pkgs,
                    threshold,
                    signature,
                ));
            }
        }

        clusters
    }

    /// Remove a package
    pub fn remove_package(&mut self, name: &str) {
        self.signatures.remove(name);
    }

    /// Clear all packages
    pub fn clear(&mut self) {
        self.signatures.clear();
    }

    /// Get all tracked package names
    pub fn get_package_names(&self) -> Vec<&String> {
        self.signatures.keys().collect()
    }
}

impl Default for CodeSimilarity {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_campaign_creation() {
        let packages = vec![
            PackageInfo::new("pkg1".to_string(), 1000),
            PackageInfo::new("pkg2".to_string(), 2000),
        ];

        let mut infra = Infrastructure::new();
        infra.add_domain("evil.com".to_string());
        infra.add_wallet("wallet123".to_string());

        let campaign = Campaign::new(
            "Test-Wave1".to_string(),
            packages.clone(),
            infra.clone(),
            CampaignType::GlassWorm,
        );

        assert_eq!(campaign.id, "Test-Wave1");
        assert_eq!(campaign.packages.len(), 2);
        assert_eq!(campaign.infrastructure.domains.len(), 1);
        assert_eq!(campaign.infrastructure.wallets.len(), 1);
        assert!(campaign.confidence > 0.0);
    }

    #[test]
    fn test_infrastructure_tracking() {
        let mut tracker = InfrastructureTracker::new();

        let mut infra1 = Infrastructure::new();
        infra1.add_domain("evil.com".to_string());
        infra1.add_author("attacker".to_string());

        let mut infra2 = Infrastructure::new();
        infra2.add_domain("evil.com".to_string());
        infra2.add_wallet("wallet123".to_string());

        tracker.track("pkg1", &infra1);
        tracker.track("pkg2", &infra2);

        // pkg1 and pkg2 should be related (shared domain)
        let related = tracker.find_related("pkg1");
        assert!(related.contains(&"pkg2".to_string()));

        // Stats should show reused domain
        let stats = tracker.get_reuse_stats();
        assert_eq!(stats.reused_domains, 1);
        assert_eq!(stats.unique_domains, 1);
    }

    #[test]
    fn test_code_similarity() {
        let mut similarity = CodeSimilarity::new();

        // Add similar code
        let code1 = "function test() { return 42; }";
        let code2 = "function test() { return 42; }";
        let code3 = "function different() { console.log('hi'); }";

        similarity.add_package("pkg1", code1);
        similarity.add_package("pkg2", code2);
        similarity.add_package("pkg3", code3);

        // Identical code should have high similarity
        let sim = similarity.similarity("pkg1", "pkg2");
        assert!(sim >= 0.9);

        // Different code should have low similarity
        let sim = similarity.similarity("pkg1", "pkg3");
        assert!(sim < 0.5);
    }

    #[test]
    fn test_code_clustering() {
        let mut similarity = CodeSimilarity::new();

        // Add similar packages
        similarity.add_package("pkg1", "function test() { return 42; }");
        similarity.add_package("pkg2", "function test() { return 42; }");
        similarity.add_package("pkg3", "function test() { return 42; }");
        similarity.add_package("pkg4", "completely different code here");

        let clusters = similarity.cluster(0.7);

        // Should have at least one cluster with pkg1, pkg2, pkg3
        assert!(!clusters.is_empty());
        let main_cluster = clusters.iter().find(|c| c.size() >= 3).unwrap();
        assert!(main_cluster.contains("pkg1"));
        assert!(main_cluster.contains("pkg2"));
        assert!(main_cluster.contains("pkg3"));
        assert!(!main_cluster.contains("pkg4"));
    }

    #[test]
    fn test_campaign_intelligence() {
        let mut intel = CampaignIntelligence::new();

        // Add packages with shared infrastructure
        let mut pkg1 = AnalyzedPackage::new("pkg1".to_string(), "code1".to_string(), 1000);
        pkg1.info.domains.push("evil.com".to_string());
        pkg1.info.authors.push("attacker".to_string());

        let mut pkg2 = AnalyzedPackage::new("pkg2".to_string(), "code1".to_string(), 2000);
        pkg2.info.domains.push("evil.com".to_string());
        pkg2.info.authors.push("attacker".to_string());

        intel.add_package(pkg1);
        intel.add_package(pkg2);

        // Should detect a campaign
        let campaigns = intel.get_campaigns();
        assert!(!campaigns.is_empty());

        let campaign = &campaigns[0];
        assert_eq!(campaign.packages.len(), 2);
        assert!(campaign.confidence > 0.5);
    }

    #[test]
    fn test_campaign_type_classification() {
        // Test GlassWorm classification
        let campaign_type = CampaignType::from_indicators(
            true,  // has_unicode_stego
            true,  // has_blockchain_c2
            false, // has_rdd
            false, // has_forcememo
            false, // has_time_delay
            false, // has_mcp_injection
        );
        assert_eq!(campaign_type, CampaignType::GlassWorm);

        // Test PhantomRaven classification
        let campaign_type = CampaignType::from_indicators(
            false, // has_unicode_stego
            false, // has_blockchain_c2
            true,  // has_rdd
            false, // has_forcememo
            false, // has_time_delay
            false, // has_mcp_injection
        );
        assert_eq!(campaign_type, CampaignType::PhantomRaven);

        // Test SandwormMode classification
        let campaign_type = CampaignType::from_indicators(
            false, // has_unicode_stego
            false, // has_blockchain_c2
            false, // has_rdd
            false, // has_forcememo
            true,  // has_time_delay
            true,  // has_mcp_injection
        );
        assert_eq!(campaign_type, CampaignType::SandwormMode);
    }

    #[test]
    fn test_time_range() {
        let packages = vec![
            PackageInfo::new("pkg1".to_string(), 1000),
            PackageInfo::new("pkg2".to_string(), 5000),
            PackageInfo::new("pkg3".to_string(), 3000),
        ];

        let time_range = TimeRange::from_packages(&packages);
        assert_eq!(time_range.start, 1000);
        assert_eq!(time_range.end, 5000);
        assert_eq!(time_range.duration_secs(), 4000);
    }

    #[test]
    fn test_infrastructure_cluster() {
        let mut tracker = InfrastructureTracker::new();

        // Create overlapping infrastructure
        let mut infra1 = Infrastructure::new();
        infra1.add_domain("domain1.com".to_string());
        infra1.add_author("author1".to_string());

        let mut infra2 = Infrastructure::new();
        infra2.add_domain("domain1.com".to_string());
        infra2.add_domain("domain2.com".to_string());

        let mut infra3 = Infrastructure::new();
        infra3.add_domain("domain2.com".to_string());
        infra3.add_wallet("wallet1".to_string());

        tracker.track("pkg1", &infra1);
        tracker.track("pkg2", &infra2);
        tracker.track("pkg3", &infra3);

        let clusters = tracker.get_clusters();

        // All three should be in the same cluster due to overlapping domains
        assert!(!clusters.is_empty());
    }
}
