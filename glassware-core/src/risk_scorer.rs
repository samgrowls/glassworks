//! Risk Scoring Module
//!
//! Implements cumulative risk scoring for findings.
//! Individual detectors emit findings with base severity,
//! but packages are only flagged when cumulative risk exceeds threshold.

use crate::finding::{Finding, Severity};

/// Risk score thresholds
pub const RISK_THRESHOLD_LOW: u32 = 10;
pub const RISK_THRESHOLD_MEDIUM: u32 = 25;
pub const RISK_THRESHOLD_HIGH: u32 = 50;
pub const RISK_THRESHOLD_CRITICAL: u32 = 100;

/// Calculate risk score for a single finding based on severity
pub fn finding_risk_score(finding: &Finding) -> u32 {
    match finding.severity {
        Severity::Info => 1,
        Severity::Low => 3,
        Severity::Medium => 8,
        Severity::High => 15,
        Severity::Critical => 25,
    }
}

/// Calculate total risk score for a package
pub fn calculate_package_risk(findings: &[Finding]) -> u32 {
    findings.iter().map(finding_risk_score).sum()
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

/// Check if package should be flagged based on risk score
pub fn should_flag(score: u32) -> bool {
    score >= RISK_THRESHOLD_LOW
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::DetectionCategory;

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
}
