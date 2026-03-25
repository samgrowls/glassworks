//! Package Context for Reputation-Based Scoring
//!
//! This module provides package reputation and context data used in the scoring system.
//! Popular, well-established packages receive a benefit of the doubt through reputation
//! multipliers that reduce their threat scores.

use serde::{Deserialize, Serialize};

/// Package reputation and context data
///
/// Tracks metadata about a package that influences its reputation score.
/// High-download, old, verified packages get lower multipliers (benefit of doubt).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackageContext {
    /// Package name (e.g., "lodash", "express")
    pub name: String,
    /// Package version (e.g., "4.17.21")
    pub version: String,
    /// Weekly download count from npm
    pub downloads_weekly: u64,
    /// Age of the package in days since first publish
    pub age_days: u64,
    /// Whether the maintainer is verified (npm verified, known identity)
    pub maintainer_verified: bool,
}

impl PackageContext {
    /// Create a new package context with minimal info
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            downloads_weekly: 0,
            age_days: 0,
            maintainer_verified: false,
        }
    }

    /// Create a new package context with full reputation data
    pub fn with_reputation(
        name: String,
        version: String,
        downloads_weekly: u64,
        age_days: u64,
        maintainer_verified: bool,
    ) -> Self {
        Self {
            name,
            version,
            downloads_weekly,
            age_days,
            maintainer_verified,
        }
    }

    /// Calculate reputation multiplier (0.5-1.0)
    ///
    /// Returns a multiplier that reduces threat scores for reputable packages:
    /// - High downloads + old + verified = 0.5 (50% reduction)
    /// - Medium downloads + established = 0.7 (30% reduction)
    /// - Low downloads + some age = 0.85 (15% reduction)
    /// - Unknown/new packages = 1.0 (no adjustment)
    ///
    /// # Examples
    ///
    /// ```
    /// use glassware::package_context::PackageContext;
    ///
    /// // Popular, old, verified package
    /// let lodash = PackageContext::with_reputation(
    ///     "lodash".to_string(),
    ///     "4.17.21".to_string(),
    ///     50_000_000,  // 50M weekly downloads
    ///     3000,        // ~8 years old
    ///     true,        // verified maintainer
    /// );
    /// assert_eq!(lodash.reputation_multiplier(), 0.5);
    ///
    /// // Unknown new package
    /// let new_pkg = PackageContext::new("suspicious-pkg".to_string(), "1.0.0".to_string());
    /// assert_eq!(new_pkg.reputation_multiplier(), 1.0);
    /// ```
    pub fn reputation_multiplier(&self) -> f32 {
        // Tier 1: High downloads + old + verified = maximum benefit of doubt
        // These are battle-tested packages used by millions
        if self.downloads_weekly > 100_000
            && self.age_days > 365
            && self.maintainer_verified
        {
            return 0.5; // Reduce score by 50% for reputable packages
        }

        // Tier 2: High downloads + established track record
        // Popular packages with some history
        if self.downloads_weekly > 10_000 && self.age_days > 180 {
            return 0.7; // Reduce score by 30%
        }

        // Tier 3: Moderate downloads + some age
        // Established but not widely used
        if self.downloads_weekly > 1_000 && self.age_days > 90 {
            return 0.85; // Reduce score by 15%
        }

        // Tier 4: Unknown or new packages
        // No reputation benefit - full score applies
        1.0
    }

    /// Get the reputation tier label
    pub fn reputation_tier(&self) -> &'static str {
        if self.downloads_weekly > 100_000
            && self.age_days > 365
            && self.maintainer_verified
        {
            "tier1_verified"
        } else if self.downloads_weekly > 10_000 && self.age_days > 180 {
            "tier2_popular"
        } else if self.downloads_weekly > 1_000 && self.age_days > 90 {
            "tier3_established"
        } else {
            "tier4_unknown"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reputation_multiplier_tier1() {
        // Popular, old, verified package (e.g., lodash, express, react)
        let pkg = PackageContext::with_reputation(
            "lodash".to_string(),
            "4.17.21".to_string(),
            50_000_000, // 50M weekly downloads
            3000,       // ~8 years old
            true,       // verified maintainer
        );
        assert_eq!(pkg.reputation_multiplier(), 0.5);
        assert_eq!(pkg.reputation_tier(), "tier1_verified");
    }

    #[test]
    fn test_reputation_multiplier_tier2() {
        // Popular but not verified or not as old
        let pkg = PackageContext::with_reputation(
            "some-lib".to_string(),
            "2.0.0".to_string(),
            50_000, // 50K weekly downloads
            400,    // ~1+ year old
            false,  // not verified
        );
        assert_eq!(pkg.reputation_multiplier(), 0.7);
        assert_eq!(pkg.reputation_tier(), "tier2_popular");
    }

    #[test]
    fn test_reputation_multiplier_tier3() {
        // Moderate downloads, some age
        let pkg = PackageContext::with_reputation(
            "small-lib".to_string(),
            "1.0.0".to_string(),
            5_000, // 5K weekly downloads
            200,   // ~6+ months old
            false,
        );
        assert_eq!(pkg.reputation_multiplier(), 0.85);
        assert_eq!(pkg.reputation_tier(), "tier3_established");
    }

    #[test]
    fn test_reputation_multiplier_tier4() {
        // Unknown new package
        let pkg = PackageContext::new("suspicious-pkg".to_string(), "1.0.0".to_string());
        assert_eq!(pkg.reputation_multiplier(), 1.0);
        assert_eq!(pkg.reputation_tier(), "tier4_unknown");
    }

    #[test]
    fn test_reputation_multiplier_edge_cases() {
        // High downloads but new (could be attack campaign)
        let pkg = PackageContext::with_reputation(
            "viral-malware".to_string(),
            "1.0.0".to_string(),
            500_000, // High downloads
            10,      // Only 10 days old
            false,
        );
        assert_eq!(pkg.reputation_multiplier(), 1.0); // No benefit - too new

        // Old but low downloads (abandoned package)
        let pkg = PackageContext::with_reputation(
            "abandoned-pkg".to_string(),
            "0.1.0".to_string(),
            100, // Very low downloads
            1000, // ~3 years old
            false,
        );
        assert_eq!(pkg.reputation_multiplier(), 1.0); // No benefit - too obscure
    }

    #[test]
    fn test_package_context_default() {
        let pkg = PackageContext::default();
        assert_eq!(pkg.name, "");
        assert_eq!(pkg.version, "");
        assert_eq!(pkg.downloads_weekly, 0);
        assert_eq!(pkg.age_days, 0);
        assert!(!pkg.maintainer_verified);
        assert_eq!(pkg.reputation_multiplier(), 1.0);
    }
}
