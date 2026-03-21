//! E4: PhantomRaven Campaign Matcher
//!
//! Evaluates whether findings collectively indicate the PhantomRaven campaign
//! (the umbrella campaign that GlassWorm is part of).
//!
//! ## Architecture
//!
//! Takes `Vec<Finding>` from all detectors and evaluates:
//! - Number of distinct detector hits
//! - Number of different categories represented
//! - Specific high-value indicators (CLSIDs, typo fingerprints, etc.)
//!
//! ## Threshold
//!
//! A campaign match requires:
//! - At least 3 distinct detector hits
//! - From at least 2 different categories (e.g., JS + binary, or JS + host)
//!
//! ## Confidence Levels
//!
//! - **HIGH**: 5+ signals from 3+ categories
//! - **MEDIUM**: 3-4 signals from 2+ categories
//! - **LOW**: Below threshold (not a match)

use crate::finding::{DetectionCategory, Finding, Severity};
use std::collections::{HashMap, HashSet};

/// Campaign match result
#[derive(Debug, Clone)]
pub struct CampaignMatch {
    /// Confidence level (LOW, MEDIUM, HIGH)
    pub confidence: ConfidenceLevel,
    /// Number of distinct signals found
    pub signal_count: usize,
    /// Number of categories represented
    pub category_count: usize,
    /// Which specific indicators were matched
    pub matched_indicators: Vec<String>,
    /// Recommendation for response
    pub recommendation: String,
}

/// Confidence levels for campaign matching
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfidenceLevel {
    /// Below threshold - not a campaign match
    Low,
    /// Meets minimum threshold
    Medium,
    /// Strong campaign indicators
    High,
}

impl ConfidenceLevel {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            ConfidenceLevel::Low => "low",
            ConfidenceLevel::Medium => "medium",
            ConfidenceLevel::High => "high",
        }
    }
}

/// PhantomRaven campaign matcher
pub struct CampaignMatcher {
    /// High-value indicators that strongly suggest PhantomRaven
    high_value_indicators: HashSet<DetectionCategory>,
}

impl CampaignMatcher {
    /// Create a new campaign matcher
    pub fn new() -> Self {
        let mut high_value = HashSet::new();
        // High-value indicators specific to PhantomRaven/GlassWorm
        high_value.insert(DetectionCategory::IElevatorCom);
        high_value.insert(DetectionCategory::JpdAuthor);
        high_value.insert(DetectionCategory::MemexecLoader);
        high_value.insert(DetectionCategory::XorShiftObfuscation);
        high_value.insert(DetectionCategory::SocketIOC2);
        high_value.insert(DetectionCategory::ExfilSchema);
        high_value.insert(DetectionCategory::BlockchainC2);

        Self {
            high_value_indicators: high_value,
        }
    }

    /// Evaluate findings for PhantomRaven campaign indicators
    pub fn evaluate(&self, findings: &[Finding]) -> Option<CampaignMatch> {
        if findings.is_empty() {
            return None;
        }

        // Count signals by category
        let mut category_signals: HashMap<DetectionCategory, usize> = HashMap::new();
        let mut matched_indicators = Vec::new();
        let mut high_value_count = 0;

        for finding in findings {
            // Skip Unknown and low-signal categories
            if finding.category == DetectionCategory::Unknown
                || finding.category == DetectionCategory::InvisibleCharacter
                || finding.category == DetectionCategory::Homoglyph
            {
                continue;
            }

            *category_signals.entry(finding.category.clone()).or_insert(0) += 1;

            // Track high-value indicators
            if self.high_value_indicators.contains(&finding.category) {
                high_value_count += 1;
                matched_indicators.push(format!("{:?}", finding.category));
            }
        }

        let signal_count = category_signals.values().sum();
        let category_count = category_signals.len();

        // Determine if threshold is met
        // Need: 3+ signals from 2+ categories
        if signal_count < 3 || category_count < 2 {
            // Below threshold
            return Some(CampaignMatch {
                confidence: ConfidenceLevel::Low,
                signal_count,
                category_count,
                matched_indicators,
                recommendation: "Insufficient indicators for campaign attribution. Continue monitoring.".to_string(),
            });
        }

        // Determine confidence level
        let (confidence, recommendation) = if signal_count >= 5 && category_count >= 3 {
            (
                ConfidenceLevel::High,
                "Strong PhantomRaven campaign indicators. Immediate incident response recommended.".to_string(),
            )
        } else {
            (
                ConfidenceLevel::Medium,
                "PhantomRaven campaign indicators detected. Investigate and correlate with threat intel.".to_string(),
            )
        };

        Some(CampaignMatch {
            confidence,
            signal_count,
            category_count,
            matched_indicators,
            recommendation,
        })
    }

    /// Check if a specific finding category is a high-value PhantomRaven indicator
    pub fn is_high_value_indicator(&self, category: &DetectionCategory) -> bool {
        self.high_value_indicators.contains(category)
    }

    /// Get list of all high-value indicators
    pub fn get_high_value_indicators(&self) -> Vec<&'static str> {
        self.high_value_indicators
            .iter()
            .map(|c| c.as_str())
            .collect()
    }
}

impl Default for CampaignMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Evaluate findings for PhantomRaven campaign match
pub fn match_campaign(findings: &[Finding]) -> Option<CampaignMatch> {
    let matcher = CampaignMatcher::new();
    matcher.evaluate(findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matcher_creation() {
        let matcher = CampaignMatcher::new();
        assert!(!matcher.high_value_indicators.is_empty());
    }

    #[test]
    fn test_no_findings_no_match() {
        let matcher = CampaignMatcher::new();
        let result = matcher.evaluate(&[]);
        assert!(result.is_none());
    }

    #[test]
    fn test_below_threshold_no_match() {
        let findings = vec![
            Finding::new(
                "test.js",
                1,
                1,
                0,
                '\0',
                DetectionCategory::SocketIOC2,
                Severity::High,
                "Socket.IO C2",
                "test",
            ),
            Finding::new(
                "test.js",
                1,
                1,
                0,
                '\0',
                DetectionCategory::SocketIOC2,
                Severity::High,
                "Socket.IO C2",
                "test",
            ),
        ];

        let matcher = CampaignMatcher::new();
        let result = matcher.evaluate(&findings);

        assert!(result.is_some());
        let match_result = result.unwrap();
        assert_eq!(match_result.confidence, ConfidenceLevel::Low);
        // Only 1 category, so below threshold
    }

    #[test]
    fn test_meets_threshold_medium_confidence() {
        let findings = vec![
            // Category 1: Binary
            Finding::new(
                "test.node",
                1,
                1,
                0,
                '\0',
                DetectionCategory::MemexecLoader,
                Severity::High,
                "memexec",
                "test",
            ),
            Finding::new(
                "test.node",
                1,
                1,
                0,
                '\0',
                DetectionCategory::XorShiftObfuscation,
                Severity::High,
                "xorshift",
                "test",
            ),
            // Category 2: JS
            Finding::new(
                "test.js",
                1,
                1,
                0,
                '\0',
                DetectionCategory::SocketIOC2,
                Severity::High,
                "Socket.IO",
                "test",
            ),
        ];

        let matcher = CampaignMatcher::new();
        let result = matcher.evaluate(&findings);

        assert!(result.is_some());
        let match_result = result.unwrap();
        assert_eq!(match_result.confidence, ConfidenceLevel::Medium);
        assert!(match_result.signal_count >= 3);
        assert!(match_result.category_count >= 2);
    }

    #[test]
    fn test_high_confidence_many_signals() {
        let findings = vec![
            // Binary indicators
            Finding::new(
                "test.node",
                1,
                1,
                0,
                '\0',
                DetectionCategory::MemexecLoader,
                Severity::High,
                "memexec",
                "test",
            ),
            Finding::new(
                "test.node",
                1,
                1,
                0,
                '\0',
                DetectionCategory::XorShiftObfuscation,
                Severity::High,
                "xorshift",
                "test",
            ),
            Finding::new(
                "test.node",
                1,
                1,
                0,
                '\0',
                DetectionCategory::IElevatorCom,
                Severity::Critical,
                "IElevator",
                "test",
            ),
            // JS indicators
            Finding::new(
                "test.js",
                1,
                1,
                0,
                '\0',
                DetectionCategory::SocketIOC2,
                Severity::High,
                "Socket.IO",
                "test",
            ),
            Finding::new(
                "test.js",
                1,
                1,
                0,
                '\0',
                DetectionCategory::ExfilSchema,
                Severity::High,
                "exfil",
                "test",
            ),
            // Blockchain
            Finding::new(
                "blockchain",
                1,
                1,
                0,
                '\0',
                DetectionCategory::BlockchainC2,
                Severity::High,
                "Solana",
                "test",
            ),
        ];

        let matcher = CampaignMatcher::new();
        let result = matcher.evaluate(&findings);

        assert!(result.is_some());
        let match_result = result.unwrap();
        assert_eq!(match_result.confidence, ConfidenceLevel::High);
        assert!(match_result.signal_count >= 5);
        assert!(match_result.category_count >= 3);
    }

    #[test]
    fn test_high_value_indicator_detection() {
        let matcher = CampaignMatcher::new();

        assert!(matcher.is_high_value_indicator(&DetectionCategory::IElevatorCom));
        assert!(matcher.is_high_value_indicator(&DetectionCategory::MemexecLoader));
        assert!(matcher.is_high_value_indicator(&DetectionCategory::SocketIOC2));
        assert!(!matcher.is_high_value_indicator(&DetectionCategory::InvisibleCharacter));
    }

    #[test]
    fn test_single_strong_signal_not_enough() {
        // Even a CRITICAL finding alone shouldn't match
        let findings = vec![Finding::new(
            "test.node",
            1,
            1,
            0,
            '\0',
            DetectionCategory::IElevatorCom,
            Severity::Critical,
            "IElevator CLSID",
            "test",
        )];

        let matcher = CampaignMatcher::new();
        let result = matcher.evaluate(&findings);

        assert!(result.is_some());
        let match_result = result.unwrap();
        assert_eq!(match_result.confidence, ConfidenceLevel::Low);
        // Single signal, even if high-value, doesn't meet threshold
    }
}
