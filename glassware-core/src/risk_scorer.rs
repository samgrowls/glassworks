//! Risk Scoring Module
//!
//! Implements cumulative risk scoring for findings with contextual multipliers.
//! Individual detectors emit findings with base severity,
//! but packages are only flagged when cumulative risk exceeds threshold.
//!
//! ## Contextual Risk Scoring
//!
//! The risk scoring system applies context-aware multipliers to adjust the base risk score:
//!
//! - **Ecosystem multiplier**: npm (baseline), PyPI (+20%), GitHub (+50%)
//! - **Package type multiplier**: Library (baseline), CLI (+30%), Extension (+50%)
//! - **Novelty multiplier**: New packages (<7 days: +50%, <30 days: +20%)
//! - **Reputation multiplier**: Known publishers (-20% risk)
//! - **File type multiplier**: Minified files (-50% confidence)
//!
//! ## Example
//!
//! ```rust
//! use glassware_core::{Finding, Severity, DetectionCategory};
//! use glassware_core::risk_scorer::{
//!     RiskContext, Ecosystem, PackageType,
//!     calculate_package_risk_with_context, finding_risk_score,
//! };
//!
//! let findings = vec![
//!     Finding::new("test.js", 1, 1, 0, '\0', DetectionCategory::Unknown, Severity::Critical, "", ""),
//! ];
//!
//! let context = RiskContext {
//!     ecosystem: Ecosystem::Npm,
//!     package_type: PackageType::Extension,
//!     is_known_publisher: false,
//!     package_age_days: Some(3),
//!     is_minified: false,
//! };
//!
//! let risk_score = calculate_package_risk_with_context(&findings, &context);
//! assert!(risk_score > 25.0); // Higher due to extension type and new package
//! ```

use crate::finding::{Finding, Severity};

/// Risk score thresholds
pub const RISK_THRESHOLD_LOW: u32 = 10;
pub const RISK_THRESHOLD_MEDIUM: u32 = 25;
pub const RISK_THRESHOLD_HIGH: u32 = 50;
pub const RISK_THRESHOLD_CRITICAL: u32 = 100;

/// Package ecosystem for risk context
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ecosystem {
    /// npm/Node.js ecosystem (baseline risk)
    Npm,
    /// PyPI/Python ecosystem (slightly higher risk)
    PyPI,
    /// GitHub repository (highest risk - direct repo access)
    GitHub,
    /// Unknown ecosystem (baseline risk)
    Unknown,
}

/// Package type for risk context
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageType {
    /// Library package (baseline risk)
    Library,
    /// CLI tool (30% higher risk - executed directly)
    CLI,
    /// IDE extension (50% higher risk - IDE integration)
    Extension,
}

/// Context for risk scoring with multipliers
///
/// This struct provides contextual information that affects risk assessment:
///
/// ## Multipliers
///
/// | Context | Multiplier | Rationale |
/// |---------|------------|-----------|
/// | Ecosystem: npm | 1.0 | Baseline |
/// | Ecosystem: PyPI | 1.2 | Slightly higher risk profile |
/// | Ecosystem: GitHub | 1.5 | Direct repository access |
/// | Type: Library | 1.0 | Baseline |
/// | Type: CLI | 1.3 | Executed directly by users |
/// | Type: Extension | 1.5 | IDE integration = higher blast radius |
/// | Age: <7 days | 1.5 | New packages are riskier |
/// | Age: <30 days | 1.2 | Recently created |
/// | Age: >=30 days | 1.0 | Established package |
/// | Known publisher | 0.8 | 20% lower risk for trusted publishers |
/// | Minified file | 0.5 | Lower confidence in findings |
///
/// ## Example
///
/// ```rust
/// use glassware_core::risk_scorer::{RiskContext, Ecosystem, PackageType};
///
/// let context = RiskContext {
///     ecosystem: Ecosystem::Npm,
///     package_type: PackageType::Extension,
///     is_known_publisher: false,
///     package_age_days: Some(5),
///     is_minified: false,
/// };
///
/// // Total multiplier: 1.0 * 1.5 * 1.5 * 1.0 * 1.0 = 2.25
/// assert_eq!(context.total_multiplier(), 2.25);
/// ```
#[derive(Debug, Clone)]
pub struct RiskContext {
    /// Package ecosystem (npm, pypi, github)
    pub ecosystem: Ecosystem,
    
    /// Package type (library, cli, extension)
    pub package_type: PackageType,
    
    /// Is this a well-known publisher?
    pub is_known_publisher: bool,
    
    /// Package age in days (None = unknown)
    pub package_age_days: Option<u32>,
    
    /// Is file minified/bundled?
    pub is_minified: bool,
}

impl Default for RiskContext {
    /// Default context for backward compatibility
    ///
    /// Defaults to: npm ecosystem, library type, unknown publisher,
    /// unknown age, not minified
    fn default() -> Self {
        Self {
            ecosystem: Ecosystem::Npm,
            package_type: PackageType::Library,
            is_known_publisher: false,
            package_age_days: None,
            is_minified: false,
        }
    }
}

impl RiskContext {
    /// Create a new risk context with default values
    ///
    /// Use builder methods to customize:
    /// ```rust
    /// use glassware_core::risk_scorer::{RiskContext, Ecosystem, PackageType};
    ///
    /// let context = RiskContext::new()
    ///     .with_ecosystem(Ecosystem::GitHub)
    ///     .with_package_type(PackageType::Extension);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the ecosystem
    pub fn with_ecosystem(mut self, ecosystem: Ecosystem) -> Self {
        self.ecosystem = ecosystem;
        self
    }

    /// Set the package type
    pub fn with_package_type(mut self, package_type: PackageType) -> Self {
        self.package_type = package_type;
        self
    }

    /// Set whether this is a known publisher
    pub fn with_known_publisher(mut self, is_known: bool) -> Self {
        self.is_known_publisher = is_known;
        self
    }

    /// Set the package age in days
    pub fn with_package_age(mut self, age_days: u32) -> Self {
        self.package_age_days = Some(age_days);
        self
    }

    /// Set whether the file is minified
    pub fn with_minified(mut self, is_minified: bool) -> Self {
        self.is_minified = is_minified;
        self
    }

    /// Get ecosystem risk multiplier
    ///
    /// | Ecosystem | Multiplier | Rationale |
    /// |-----------|------------|-----------|
    /// | Npm | 1.0 | Baseline |
    /// | PyPI | 1.2 | Slightly higher risk profile |
    /// | GitHub | 1.5 | Direct repository access |
    /// | Unknown | 1.0 | Baseline |
    pub fn ecosystem_multiplier(&self) -> f32 {
        match self.ecosystem {
            Ecosystem::Npm => 1.0,
            Ecosystem::PyPI => 1.2,
            Ecosystem::GitHub => 1.5,
            Ecosystem::Unknown => 1.0,
        }
    }
    
    /// Get package type multiplier
    ///
    /// | Type | Multiplier | Rationale |
    /// |------|------------|-----------|
    /// | Library | 1.0 | Baseline |
    /// | CLI | 1.3 | 30% higher risk - executed directly |
    /// | Extension | 1.5 | 50% higher risk - IDE integration |
    pub fn package_type_multiplier(&self) -> f32 {
        match self.package_type {
            PackageType::Library => 1.0,
            PackageType::CLI => 1.3,
            PackageType::Extension => 1.5,
        }
    }
    
    /// Get novelty multiplier (new packages are riskier)
    ///
    /// | Age | Multiplier | Rationale |
    /// |-----|------------|-----------|
    /// | Unknown | 1.2 | Unknown age = slightly riskier |
    /// | <7 days | 1.5 | < 1 week = 50% higher risk |
    /// | <30 days | 1.2 | < 1 month = 20% higher risk |
    /// | >=30 days | 1.0 | Established = baseline |
    pub fn novelty_multiplier(&self) -> f32 {
        match self.package_age_days {
            None => 1.2,
            Some(age) if age < 7 => 1.5,
            Some(age) if age < 30 => 1.2,
            Some(_) => 1.0,
        }
    }
    
    /// Get publisher reputation multiplier
    ///
    /// | Publisher | Multiplier | Rationale |
    /// |-----------|------------|-----------|
    /// | Known | 0.8 | 20% lower risk for known publishers |
    /// | Unknown | 1.0 | Unknown = baseline |
    pub fn reputation_multiplier(&self) -> f32 {
        if self.is_known_publisher {
            0.8
        } else {
            1.0
        }
    }
    
    /// Get file type multiplier
    ///
    /// | File Type | Multiplier | Rationale |
    /// |-----------|------------|-----------|
    /// | Minified | 0.5 | Lower confidence in findings |
    /// | Source | 1.0 | Source files = baseline |
    pub fn file_type_multiplier(&self) -> f32 {
        if self.is_minified {
            0.5
        } else {
            1.0
        }
    }
    
    /// Calculate total multiplier
    ///
    /// Multiplies all context multipliers together:
    /// ```text
    /// total = ecosystem × package_type × novelty × reputation × file_type
    /// ```
    pub fn total_multiplier(&self) -> f32 {
        self.ecosystem_multiplier()
            * self.package_type_multiplier()
            * self.novelty_multiplier()
            * self.reputation_multiplier()
            * self.file_type_multiplier()
    }
}

/// Calculate risk score for a single finding based on severity
///
/// | Severity | Score |
/// |----------|-------|
/// | Info | 1 |
/// | Low | 3 |
/// | Medium | 8 |
/// | High | 15 |
/// | Critical | 25 |
pub fn finding_risk_score(finding: &Finding) -> u32 {
    match finding.severity {
        Severity::Info => 1,
        Severity::Low => 3,
        Severity::Medium => 8,
        Severity::High => 15,
        Severity::Critical => 25,
    }
}

/// Calculate total risk score for a package with context
///
/// Applies contextual multipliers to the base risk score.
///
/// # Arguments
/// * `findings` - List of findings to score
/// * `context` - Risk context with multipliers
///
/// # Returns
/// Risk score as f32 (base score × total multiplier)
///
/// # Example
///
/// ```rust
/// use glassware_core::{Finding, Severity, DetectionCategory};
/// use glassware_core::risk_scorer::{
///     RiskContext, Ecosystem, PackageType,
///     calculate_package_risk_with_context,
/// };
///
/// let findings = vec![
///     Finding::new("test.js", 1, 1, 0, '\0', DetectionCategory::Unknown, Severity::Critical, "", ""),
///     Finding::new("test.js", 2, 1, 0, '\0', DetectionCategory::Unknown, Severity::High, "", ""),
/// ];
///
/// let context = RiskContext::new()
///     .with_ecosystem(Ecosystem::GitHub)
///     .with_package_type(PackageType::Extension);
///
/// let score = calculate_package_risk_with_context(&findings, &context);
/// // Base: 25 + 15 = 40
/// // Multiplier: 1.5 (GitHub) × 1.5 (Extension) × 1.2 (unknown age) = 2.7
/// // Total: 40 × 2.7 = 108.0
/// assert_eq!(score, 108.0);
/// ```
pub fn calculate_package_risk_with_context(
    findings: &[Finding],
    context: &RiskContext,
) -> f32 {
    let base_score: f32 = findings.iter()
        .map(finding_risk_score)
        .sum::<u32>() as f32;
    
    base_score * context.total_multiplier()
}

/// Calculate total risk score for a package (backward compatible)
///
/// Uses default context (npm, library, unknown publisher, unknown age, not minified).
/// For contextual scoring, use [`calculate_package_risk_with_context`] instead.
///
/// # Arguments
/// * `findings` - List of findings to score
///
/// # Returns
/// Risk score as u32 (rounded down from f32)
///
/// # Example
///
/// ```rust
/// use glassware_core::{Finding, Severity, DetectionCategory};
/// use glassware_core::risk_scorer::calculate_package_risk;
///
/// let findings = vec![
///     Finding::new("test.js", 1, 1, 0, '\0', DetectionCategory::Unknown, Severity::Critical, "", ""),
///     Finding::new("test.js", 2, 1, 0, '\0', DetectionCategory::Unknown, Severity::High, "", ""),
/// ];
///
/// let score = calculate_package_risk(&findings);
/// assert_eq!(score, 40); // 25 + 15
/// ```
pub fn calculate_package_risk(findings: &[Finding]) -> u32 {
    // Backward compatible: use baseline context (all multipliers = 1.0)
    // This maintains the original behavior of summing severity scores
    let context = RiskContext {
        ecosystem: Ecosystem::Npm,
        package_type: PackageType::Library,
        is_known_publisher: false,
        package_age_days: Some(365), // Established package = 1.0 multiplier
        is_minified: false,
    };
    
    calculate_package_risk_with_context(findings, &context) as u32
}

/// Determine overall risk level based on score
pub fn risk_level(score: u32) -> &'static str {
    if score >= RISK_THRESHOLD_CRITICAL {
        "CRITICAL"
    } else if score >= RISK_THRESHOLD_HIGH {
        "HIGH"
    } else if score >= RISK_THRESHOLD_MEDIUM {
        "MEDIUM"
    } else if score >= RISK_THRESHOLD_LOW {
        "LOW"
    } else {
        "MINIMAL"
    }
}

/// Determine overall risk level based on contextual score (f32 version)
pub fn risk_level_f32(score: f32) -> &'static str {
    risk_level(score as u32)
}

/// Check if package should be flagged based on risk score
pub fn should_flag(score: u32) -> bool {
    score >= RISK_THRESHOLD_LOW
}

/// Check if package should be flagged based on contextual score (f32 version)
pub fn should_flag_f32(score: f32) -> bool {
    score >= RISK_THRESHOLD_LOW as f32
}

/// Get recommended action based on risk level
pub fn recommended_action(score: u32) -> &'static str {
    if score >= RISK_THRESHOLD_CRITICAL {
        "IMMEDIATE_INVESTIGATION"
    } else if score >= RISK_THRESHOLD_HIGH {
        "PRIORITY_REVIEW"
    } else if score >= RISK_THRESHOLD_MEDIUM {
        "REVIEW"
    } else if score >= RISK_THRESHOLD_LOW {
        "MONITOR"
    } else {
        "IGNORE"
    }
}

/// Get recommended action based on contextual score (f32 version)
pub fn recommended_action_f32(score: f32) -> &'static str {
    recommended_action(score as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::DetectionCategory;

    // ========================================================================
    // Original Tests (Backward Compatibility)
    // ========================================================================

    #[test]
    fn test_risk_score_calculation() {
        let findings = vec![
            Finding::new("test.js", 1, 1, 0, '\0', DetectionCategory::Unknown, Severity::Critical, "", ""),
            Finding::new("test.js", 2, 1, 0, '\0', DetectionCategory::Unknown, Severity::High, "", ""),
            Finding::new("test.js", 3, 1, 0, '\0', DetectionCategory::Unknown, Severity::Medium, "", ""),
        ];

        let score = calculate_package_risk(&findings);
        assert_eq!(score, 25 + 15 + 8); // Critical + High + Medium
    }

    #[test]
    fn test_risk_levels() {
        assert_eq!(risk_level(5), "MINIMAL");
        assert_eq!(risk_level(15), "LOW");
        assert_eq!(risk_level(30), "MEDIUM");
        assert_eq!(risk_level(60), "HIGH");
        assert_eq!(risk_level(150), "CRITICAL");
    }

    #[test]
    fn test_should_flag() {
        assert!(!should_flag(5));
        assert!(should_flag(15));
        assert!(should_flag(100));
    }

    // ========================================================================
    // RiskContext Tests
    // ========================================================================

    #[test]
    fn test_risk_context_default() {
        let context = RiskContext::default();
        assert_eq!(context.ecosystem, Ecosystem::Npm);
        assert_eq!(context.package_type, PackageType::Library);
        assert!(!context.is_known_publisher);
        assert_eq!(context.package_age_days, None);
        assert!(!context.is_minified);
    }

    #[test]
    fn test_risk_context_builder() {
        let context = RiskContext::new()
            .with_ecosystem(Ecosystem::GitHub)
            .with_package_type(PackageType::Extension)
            .with_known_publisher(true)
            .with_package_age(15)
            .with_minified(false);

        assert_eq!(context.ecosystem, Ecosystem::GitHub);
        assert_eq!(context.package_type, PackageType::Extension);
        assert!(context.is_known_publisher);
        assert_eq!(context.package_age_days, Some(15));
        assert!(!context.is_minified);
    }

    // ========================================================================
    // Multiplier Tests
    // ========================================================================

    #[test]
    fn test_ecosystem_multiplier() {
        let npm_context = RiskContext::new().with_ecosystem(Ecosystem::Npm);
        assert_eq!(npm_context.ecosystem_multiplier(), 1.0);

        let pypi_context = RiskContext::new().with_ecosystem(Ecosystem::PyPI);
        assert_eq!(pypi_context.ecosystem_multiplier(), 1.2);

        let github_context = RiskContext::new().with_ecosystem(Ecosystem::GitHub);
        assert_eq!(github_context.ecosystem_multiplier(), 1.5);

        let unknown_context = RiskContext::new().with_ecosystem(Ecosystem::Unknown);
        assert_eq!(unknown_context.ecosystem_multiplier(), 1.0);
    }

    #[test]
    fn test_package_type_multiplier() {
        let library_context = RiskContext::new().with_package_type(PackageType::Library);
        assert_eq!(library_context.package_type_multiplier(), 1.0);

        let cli_context = RiskContext::new().with_package_type(PackageType::CLI);
        assert_eq!(cli_context.package_type_multiplier(), 1.3);

        let extension_context = RiskContext::new().with_package_type(PackageType::Extension);
        assert_eq!(extension_context.package_type_multiplier(), 1.5);
    }

    #[test]
    fn test_novelty_multiplier() {
        // Unknown age
        let unknown_context = RiskContext::new();
        assert_eq!(unknown_context.novelty_multiplier(), 1.2);

        // < 7 days
        let new_context = RiskContext::new().with_package_age(3);
        assert_eq!(new_context.novelty_multiplier(), 1.5);

        let week_context = RiskContext::new().with_package_age(6);
        assert_eq!(week_context.novelty_multiplier(), 1.5);

        // < 30 days
        let month_context = RiskContext::new().with_package_age(15);
        assert_eq!(month_context.novelty_multiplier(), 1.2);

        let almost_month_context = RiskContext::new().with_package_age(29);
        assert_eq!(almost_month_context.novelty_multiplier(), 1.2);

        // >= 30 days
        let established_context = RiskContext::new().with_package_age(30);
        assert_eq!(established_context.novelty_multiplier(), 1.0);

        let old_context = RiskContext::new().with_package_age(365);
        assert_eq!(old_context.novelty_multiplier(), 1.0);
    }

    #[test]
    fn test_reputation_multiplier() {
        let unknown_publisher = RiskContext::new().with_known_publisher(false);
        assert_eq!(unknown_publisher.reputation_multiplier(), 1.0);

        let known_publisher = RiskContext::new().with_known_publisher(true);
        assert_eq!(known_publisher.reputation_multiplier(), 0.8);
    }

    #[test]
    fn test_file_type_multiplier() {
        let source_context = RiskContext::new().with_minified(false);
        assert_eq!(source_context.file_type_multiplier(), 1.0);

        let minified_context = RiskContext::new().with_minified(true);
        assert_eq!(minified_context.file_type_multiplier(), 0.5);
    }

    // ========================================================================
    // Total Multiplier Tests
    // ========================================================================

    #[test]
    fn test_total_multiplier_baseline() {
        // All baseline multipliers
        let context = RiskContext::default();
        // 1.0 (npm) × 1.0 (library) × 1.2 (unknown age) × 1.0 (unknown publisher) × 1.0 (source)
        assert!((context.total_multiplier() - 1.2).abs() < 0.001);
    }

    #[test]
    fn test_total_multiplier_maximum_risk() {
        // Maximum risk scenario
        let context = RiskContext::new()
            .with_ecosystem(Ecosystem::GitHub)
            .with_package_type(PackageType::Extension)
            .with_package_age(3)
            .with_known_publisher(false)
            .with_minified(false);
        // 1.5 (GitHub) × 1.5 (Extension) × 1.5 (<7 days) × 1.0 (unknown) × 1.0 (source)
        let expected = 1.5 * 1.5 * 1.5 * 1.0 * 1.0;
        assert!((context.total_multiplier() - expected).abs() < 0.001);
    }

    #[test]
    fn test_total_multiplier_minimum_risk() {
        // Minimum risk scenario (known publisher, established package)
        let context = RiskContext::new()
            .with_ecosystem(Ecosystem::Npm)
            .with_package_type(PackageType::Library)
            .with_package_age(365)
            .with_known_publisher(true)
            .with_minified(true);
        // 1.0 (npm) × 1.0 (library) × 1.0 (established) × 0.8 (known) × 0.5 (minified)
        let expected = 1.0 * 1.0 * 1.0 * 0.8 * 0.5;
        assert!((context.total_multiplier() - expected).abs() < 0.001);
    }

    #[test]
    fn test_total_multiplier_vscode_extension_scenario() {
        // Realistic VS Code extension scenario
        let context = RiskContext::new()
            .with_ecosystem(Ecosystem::Npm)
            .with_package_type(PackageType::Extension)
            .with_package_age(5)
            .with_known_publisher(false)
            .with_minified(false);
        // 1.0 (npm) × 1.5 (Extension) × 1.5 (<7 days) × 1.0 (unknown) × 1.0 (source)
        let expected = 1.0 * 1.5 * 1.5 * 1.0 * 1.0;
        assert!((context.total_multiplier() - expected).abs() < 0.001);
    }

    // ========================================================================
    // Contextual Risk Calculation Tests
    // ========================================================================

    #[test]
    fn test_calculate_package_risk_with_context_basic() {
        let findings = vec![
            Finding::new("test.js", 1, 1, 0, '\0', DetectionCategory::Unknown, Severity::Critical, "", ""),
        ];

        let context = RiskContext::default();
        let score = calculate_package_risk_with_context(&findings, &context);
        
        // Base: 25, Multiplier: 1.2 (unknown age default)
        assert!((score - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_package_risk_with_context_github_extension() {
        let findings = vec![
            Finding::new("test.js", 1, 1, 0, '\0', DetectionCategory::Unknown, Severity::Critical, "", ""),
            Finding::new("test.js", 2, 1, 0, '\0', DetectionCategory::Unknown, Severity::High, "", ""),
        ];

        let context = RiskContext::new()
            .with_ecosystem(Ecosystem::GitHub)
            .with_package_type(PackageType::Extension)
            .with_package_age(3);
        
        let score = calculate_package_risk_with_context(&findings, &context);
        
        // Base: 25 + 15 = 40
        // Multiplier: 1.5 (GitHub) × 1.5 (Extension) × 1.5 (<7 days) × 1.0 × 1.0 = 3.375
        // Total: 40 × 3.375 = 135.0
        assert!((score - 135.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_package_risk_with_context_known_publisher() {
        let findings = vec![
            Finding::new("test.js", 1, 1, 0, '\0', DetectionCategory::Unknown, Severity::High, "", ""),
        ];

        let context = RiskContext::new()
            .with_known_publisher(true)
            .with_package_age(100);
        
        let score = calculate_package_risk_with_context(&findings, &context);
        
        // Base: 15
        // Multiplier: 1.0 × 1.0 × 1.0 (established) × 0.8 (known) × 1.0 = 0.8
        // Total: 15 × 0.8 = 12.0
        assert!((score - 12.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_package_risk_with_context_minified() {
        let findings = vec![
            Finding::new("bundle.min.js", 1, 1, 0, '\0', DetectionCategory::Unknown, Severity::Critical, "", ""),
        ];

        let context = RiskContext::new().with_minified(true);
        
        let score = calculate_package_risk_with_context(&findings, &context);
        
        // Base: 25
        // Multiplier: 1.0 × 1.0 × 1.2 (unknown age) × 1.0 × 0.5 (minified) = 0.6
        // Total: 25 × 0.6 = 15.0
        assert!((score - 15.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_package_risk_backward_compatible() {
        let findings = vec![
            Finding::new("test.js", 1, 1, 0, '\0', DetectionCategory::Unknown, Severity::Critical, "", ""),
            Finding::new("test.js", 2, 1, 0, '\0', DetectionCategory::Unknown, Severity::High, "", ""),
        ];

        let score = calculate_package_risk(&findings);
        
        // Backward compatible: raw sum with all multipliers = 1.0
        // Base: 25 + 15 = 40
        assert_eq!(score, 40);
    }

    // ========================================================================
    // Risk Level and Action Tests (f32 versions)
    // ========================================================================

    #[test]
    fn test_risk_level_f32() {
        assert_eq!(risk_level_f32(5.0), "MINIMAL");
        assert_eq!(risk_level_f32(15.5), "LOW");
        assert_eq!(risk_level_f32(30.9), "MEDIUM");
        assert_eq!(risk_level_f32(60.1), "HIGH");
        assert_eq!(risk_level_f32(150.9), "CRITICAL");
    }

    #[test]
    fn test_should_flag_f32() {
        assert!(!should_flag_f32(5.0));
        assert!(should_flag_f32(15.5));
        assert!(should_flag_f32(100.0));
    }

    #[test]
    fn test_recommended_action_f32() {
        assert_eq!(recommended_action_f32(5.0), "IGNORE");
        assert_eq!(recommended_action_f32(15.5), "MONITOR");
        assert_eq!(recommended_action_f32(30.0), "REVIEW");
        assert_eq!(recommended_action_f32(60.0), "PRIORITY_REVIEW");
        assert_eq!(recommended_action_f32(150.0), "IMMEDIATE_INVESTIGATION");
    }

    // ========================================================================
    // Integration Tests
    // ========================================================================

    #[test]
    fn test_end_to_end_npm_library() {
        let findings = vec![
            Finding::new("index.js", 10, 5, 0xFE00, '\u{FE00}', DetectionCategory::InvisibleCharacter, Severity::High, "", ""),
            Finding::new("index.js", 15, 10, 0x202E, '\u{202E}', DetectionCategory::BidirectionalOverride, Severity::Critical, "", ""),
        ];

        let context = RiskContext::new()
            .with_ecosystem(Ecosystem::Npm)
            .with_package_type(PackageType::Library)
            .with_package_age(60)
            .with_known_publisher(false);

        let score = calculate_package_risk_with_context(&findings, &context);
        
        // Base: 15 + 25 = 40
        // Multiplier: 1.0 × 1.0 × 1.0 (established) × 1.0 × 1.0 = 1.0
        // Total: 40.0
        assert!((score - 40.0).abs() < 0.001);
        assert_eq!(risk_level_f32(score), "MEDIUM");
        assert_eq!(recommended_action_f32(score), "REVIEW");
        assert!(should_flag_f32(score));
    }

    #[test]
    fn test_end_to_end_github_extension_high_risk() {
        let findings = vec![
            Finding::new("extension.js", 1, 1, 0, '\0', DetectionCategory::GlasswarePattern, Severity::Critical, "", ""),
        ];

        let context = RiskContext::new()
            .with_ecosystem(Ecosystem::GitHub)
            .with_package_type(PackageType::Extension)
            .with_package_age(2)
            .with_known_publisher(false);

        let score = calculate_package_risk_with_context(&findings, &context);
        
        // Base: 25
        // Multiplier: 1.5 × 1.5 × 1.5 × 1.0 × 1.0 = 3.375
        // Total: 84.375
        assert!((score - 84.375).abs() < 0.001);
        assert_eq!(risk_level_f32(score), "HIGH");
        assert_eq!(recommended_action_f32(score), "PRIORITY_REVIEW");
    }
}
