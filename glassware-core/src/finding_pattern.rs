//! Deduplicated Finding Pattern for Scoring
//!
//! This module provides a deduplication mechanism for findings that share similar patterns.
//! Instead of counting 383 similar i18n findings as 383 separate issues, they are grouped
//! into a single pattern with count metadata, applying diminishing returns.

use crate::finding::{DetectionCategory, Severity};

/// A deduplicated pattern of similar findings
///
/// Groups findings that share the same category and description pattern,
/// tracking count and average confidence for scoring with diminishing returns.
#[derive(Debug, Clone)]
pub struct FindingPattern {
    /// Detection category of the pattern
    pub category: DetectionCategory,
    /// Base severity of the pattern
    pub severity: Severity,
    /// Maximum severity across all merged findings
    pub max_severity: Severity,
    /// Number of findings merged into this pattern
    pub count: usize,
    /// Average confidence across all merged findings
    pub avg_confidence: f32,
    /// Description of the pattern
    pub description: String,
}

impl FindingPattern {
    /// Create a new finding pattern from a single finding
    pub fn new(
        category: DetectionCategory,
        severity: Severity,
        confidence: f32,
        description: String,
    ) -> Self {
        Self {
            category,
            severity,
            max_severity: severity,
            count: 1,
            avg_confidence: confidence,
            description,
        }
    }

    /// Merge another finding into this pattern
    ///
    /// Updates the count, max severity, and running average confidence.
    pub fn merge(&mut self, other_severity: Severity, other_confidence: f32) {
        self.count += 1;
        self.max_severity = self.max_severity.max(other_severity);
        // Running average: (old_avg * old_count + new_value) / new_count
        self.avg_confidence =
            (self.avg_confidence * (self.count - 1) as f32 + other_confidence) / self.count as f32;
    }

    /// Create a deduplication key from a finding's category and description
    ///
    /// This key is used to group similar findings together.
    pub fn create_key(category: &DetectionCategory, description: &str) -> String {
        format!("{:?}:{}", category, description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finding_pattern_new() {
        let pattern = FindingPattern::new(
            DetectionCategory::InvisibleCharacter,
            Severity::High,
            0.85,
            "Invisible character detected".to_string(),
        );

        assert_eq!(pattern.category, DetectionCategory::InvisibleCharacter);
        assert_eq!(pattern.severity, Severity::High);
        assert_eq!(pattern.max_severity, Severity::High);
        assert_eq!(pattern.count, 1);
        assert_eq!(pattern.avg_confidence, 0.85);
    }

    #[test]
    fn test_finding_pattern_merge() {
        let mut pattern = FindingPattern::new(
            DetectionCategory::InvisibleCharacter,
            Severity::High,
            0.80,
            "Invisible character detected".to_string(),
        );

        // Merge a critical finding with lower confidence
        pattern.merge(Severity::Critical, 0.60);

        assert_eq!(pattern.count, 2);
        assert_eq!(pattern.max_severity, Severity::Critical);
        // Average: (0.80 + 0.60) / 2 = 0.70
        assert!((pattern.avg_confidence - 0.70).abs() < 0.001);

        // Merge another high finding
        pattern.merge(Severity::High, 0.90);

        assert_eq!(pattern.count, 3);
        assert_eq!(pattern.max_severity, Severity::Critical);
        // Average: (0.80 + 0.60 + 0.90) / 3 = 0.767
        assert!((pattern.avg_confidence - 0.767).abs() < 0.001);
    }

    #[test]
    fn test_finding_pattern_create_key() {
        let key1 = FindingPattern::create_key(
            &DetectionCategory::InvisibleCharacter,
            "Invisible character detected",
        );
        let key2 = FindingPattern::create_key(
            &DetectionCategory::InvisibleCharacter,
            "Invisible character detected",
        );
        let key3 = FindingPattern::create_key(
            &DetectionCategory::Homoglyph,
            "Invisible character detected",
        );

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }
}
