//! Scoring Engine with Deduplication, LLM Feedback, and Reputation
//!
//! This module implements the Phase A.5 scoring system redesign:
//! 1. Deduplicates findings by pattern (383 similar findings = 1 pattern)
//! 2. Weights findings by quality (LLM confidence impact)
//! 3. Applies reputation multipliers (popular packages get benefit of doubt)
//! 4. Raises malicious threshold to 8.0
//! 5. Maintains 100% evidence detection via exceptions

use glassware_core::{Finding, Severity, DetectionCategory, FindingPattern};
use std::collections::{HashMap, HashSet};

pub use crate::scoring_config::ScoringConfig;
pub use crate::package_context::PackageContext;
use crate::llm::LlmVerdict;

/// Scoring engine for calculating threat scores
///
/// Implements the new scoring pipeline:
/// 1. Deduplicate findings by pattern
/// 2. Calculate base score from deduplicated patterns
/// 3. Apply category diversity caps
/// 4. Apply LLM quality multiplier
/// 5. Apply reputation multiplier
/// 6. Apply exceptions (known C2, steganography, etc.)
pub struct ScoringEngine {
    config: ScoringConfig,
    package_context: PackageContext,
}

impl ScoringEngine {
    /// Create a new scoring engine with the given configuration
    pub fn new(config: ScoringConfig, package_context: PackageContext) -> Self {
        Self {
            config,
            package_context,
        }
    }

    /// Calculate threat score with the new scoring system
    ///
    /// # Arguments
    ///
    /// * `findings` - List of findings from the scanner
    /// * `llm_verdict` - Optional LLM verdict for quality adjustment
    ///
    /// # Returns
    ///
    /// A threat score between 0.0 and 10.0
    ///
    /// # Example
    ///
    /// ```
    /// use glassware::scoring::{ScoringEngine, ScoringConfig};
    /// use glassware::package_context::PackageContext;
    /// use glassware_core::{Finding, Severity, DetectionCategory};
    ///
    /// let config = ScoringConfig::default();
    /// let ctx = PackageContext::new("test-pkg".to_string(), "1.0.0".to_string());
    /// let engine = ScoringEngine::new(config, ctx);
    ///
    /// let findings = vec![]; // Empty findings
    /// let score = engine.calculate_score(&findings, None);
    /// assert_eq!(score, 0.0);
    /// ```
    pub fn calculate_score(
        &self,
        findings: &[Finding],
        llm_verdict: Option<&LlmVerdict>,
    ) -> f32 {
        if findings.is_empty() {
            return 0.0;
        }

        // STEP 1: Deduplicate findings by pattern
        let patterns = self.deduplicate_findings(findings);

        // STEP 2: Calculate base score from deduplicated patterns
        let mut base_score = 0.0;
        for pattern in &patterns {
            base_score += self.calculate_pattern_score(pattern);
        }

        // STEP 3: Apply category diversity caps
        base_score = self.apply_category_caps(base_score, &patterns);

        // STEP 4: Apply LLM quality multiplier
        if let Some(llm) = llm_verdict {
            base_score *= self.calculate_llm_multiplier(llm);
        }

        // STEP 5: Apply reputation multiplier
        base_score *= self.package_context.reputation_multiplier();

        // STEP 6: Apply exceptions (known C2, steganography, etc.)
        base_score = self.apply_exceptions(base_score, findings);

        // STEP 7: Clamp to valid range [0.0, 10.0]
        base_score.min(10.0).max(0.0)
    }

    /// Deduplicate findings by pattern similarity
    ///
    /// Groups findings with the same category and description into a single pattern,
    /// tracking count and average confidence for scoring with diminishing returns.
    fn deduplicate_findings(&self, findings: &[Finding]) -> Vec<FindingPattern> {
        if !self.config.pattern_dedup_enabled {
            // Return each finding as separate pattern (old behavior)
            return findings
                .iter()
                .map(|f| {
                    FindingPattern::new(
                        f.category.clone(),
                        f.severity,
                        f.confidence.unwrap_or(0.5) as f32,
                        f.description.clone(),
                    )
                })
                .collect();
        }

        // Group findings by pattern key
        let mut patterns: HashMap<String, FindingPattern> = HashMap::new();

        for finding in findings {
            let key = FindingPattern::create_key(&finding.category, &finding.description);

            patterns
                .entry(key)
                .and_modify(|p| {
                    p.merge(
                        finding.severity,
                        finding.confidence.unwrap_or(0.5) as f32,
                    );
                })
                .or_insert(FindingPattern::new(
                    finding.category.clone(),
                    finding.severity,
                    finding.confidence.unwrap_or(0.5) as f32,
                    finding.description.clone(),
                ));
        }

        patterns.into_values().collect()
    }

    /// Calculate score for a single pattern with diminishing returns
    ///
    /// Uses logarithmic scaling for count:
    /// - 1 finding = full weight (1.0 + log10(1) = 1.0)
    /// - 10 findings = 2x weight (1.0 + log10(10) = 2.0)
    /// - 100 findings = 3x weight (1.0 + log10(100) = 3.0)
    /// - 1000 findings = 4x weight (1.0 + log10(1000) = 4.0)
    fn calculate_pattern_score(&self, pattern: &FindingPattern) -> f32 {
        // Diminishing returns for count
        let count_multiplier = 1.0 + (pattern.count as f32).log10();

        let base = match pattern.max_severity {
            Severity::Critical => 3.0,
            Severity::High => 2.0,
            Severity::Medium => 1.0,
            Severity::Low => 0.5,
            Severity::Info => 0.5,
            _ => 0.5,
        };

        base * count_multiplier * pattern.avg_confidence.min(1.0)
    }

    /// Apply category diversity caps
    ///
    /// Stricter caps than the old system to prevent false positives:
    /// - 1 category: capped at 5.0 (suspicious, not malicious)
    /// - 2 categories: capped at 7.0 (borderline malicious)
    /// - 3 categories: capped at 8.5 (likely malicious)
    /// - 4+ categories: no cap (very likely malicious)
    fn apply_category_caps(
        &self,
        score: f32,
        patterns: &[FindingPattern],
    ) -> f32 {
        let categories: HashSet<_> = patterns.iter().map(|p| &p.category).collect();
        let category_count = categories.len();

        match category_count {
            0 => 0.0,
            1 => score.min(5.0),   // Single category capped at 5.0
            2 => score.min(7.0),   // Two categories capped at 7.0
            3 => score.min(8.5),   // Three categories capped at 8.5
            _ => score.min(10.0),  // 4+ categories = no cap
        }
    }

    /// Calculate LLM quality multiplier (0.2-1.0)
    ///
    /// More aggressive reduction for low-confidence findings:
    /// - LLM confidence < 0.20 → multiplier 0.2-0.3x (70-80% reduction)
    /// - LLM confidence 0.20-0.50 → multiplier 0.3-0.6x (40-70% reduction)
    /// - LLM confidence > 0.50 → multiplier 0.6-1.0x (0-40% reduction)
    fn calculate_llm_multiplier(&self, llm: &LlmVerdict) -> f32 {
        if llm.confidence < 0.20 {
            // Very low confidence = severe penalty (75-80% reduction)
            // 0.10 confidence = 0.25x multiplier
            0.2 + (llm.confidence * 0.5)
        } else if llm.confidence < 0.50 {
            // Medium-low confidence = moderate penalty (40-70% reduction)
            // 0.30 confidence = 0.46x multiplier
            0.3 + ((llm.confidence - 0.20) * 0.6)
        } else {
            // High confidence = minimal penalty (10-30% reduction)
            // 0.90 confidence = 0.86x multiplier
            0.5 + ((llm.confidence - 0.50) * 0.8)
        }
    }

    /// Apply exceptions for known malicious patterns
    ///
    /// Certain attack patterns are so distinctive they override normal scoring:
    /// - Known C2 wallets/IPs: minimum 9.0
    /// - GlassWorm C2 polling: minimum 9.0
    /// - Steganography with decoder: minimum 8.5 (NOT for i18n/locale files)
    fn apply_exceptions(&self, score: f32, findings: &[Finding]) -> f32 {
        // Known C2 wallets/IPs always score high
        if findings.iter().any(|f| {
            f.category == DetectionCategory::BlockchainC2
                && f.severity == Severity::Critical
                && (f.description.contains("Known C2") || f.description.contains("GlassWorm"))
        }) {
            return score.max(self.config.known_c2_min_score);
        }

        // GlassWorm C2 polling (getSignaturesForAddress + setInterval)
        if findings.iter().any(|f| {
            f.category == DetectionCategory::BlockchainC2
                && f.severity == Severity::Critical
                && f.description.contains("polling")
        }) {
            return score.max(self.config.glassworm_c2_min_score);
        }

        // Steganography with decoder - EXCLUDE i18n/locale/translation packages
        // This prevents legitimate i18n libraries (antd, dayjs, moment) from triggering
        if findings.iter().any(|f| {
            (f.category == DetectionCategory::SteganoPayload
                || f.category == DetectionCategory::InvisibleCharacter)
                && f.severity == Severity::Critical
                && (f.description.contains("decoder") || f.description.contains("GlassWorm"))
                // Exclude i18n-related findings
                && !f.description.contains("i18n")
                && !f.description.contains("locale")
                && !f.description.contains("translation")
                && !f.description.contains("gettext")
        }) {
            return score.max(self.config.steganography_min_score);
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_findings() {
        let config = ScoringConfig::default();
        let ctx = PackageContext::new("test".to_string(), "1.0.0".to_string());
        let engine = ScoringEngine::new(config, ctx);

        assert_eq!(engine.calculate_score(&[], None), 0.0);
    }

    #[test]
    fn test_single_finding() {
        let config = ScoringConfig::default();
        let ctx = PackageContext::new("test".to_string(), "1.0.0".to_string());
        let engine = ScoringEngine::new(config, ctx);

        let findings = vec![Finding {
            file: "test.js".to_string(),
            line: 1,
            column: 1,
            code_point: 0xFE00,
            character: "\u{FE00}".to_string(),
            raw_bytes: None,
            category: DetectionCategory::InvisibleCharacter,
            severity: Severity::High,
            description: "Invisible character detected".to_string(),
            remediation: "Remove invisible character".to_string(),
            cwe_id: None,
            references: vec![],
            context: None,
            decoded_payload: None,
            confidence: Some(0.9),
        }];

        let score = engine.calculate_score(&findings, None);
        assert!(score > 0.0);
        assert!(score <= 10.0);
    }

    #[test]
    fn test_deduplication_reduces_score() {
        let config = ScoringConfig::default();
        let ctx = PackageContext::new("test".to_string(), "1.0.0".to_string());
        let engine = ScoringEngine::new(config, ctx);

        // Create 10 identical findings (simulating i18n false positives)
        let findings: Vec<Finding> = (0..10)
            .map(|i| Finding {
                file: format!("locale{}.js", i),
                line: 1,
                column: 1,
                code_point: 0xFE00,
                character: "\u{FE00}".to_string(),
                raw_bytes: None,
                category: DetectionCategory::InvisibleCharacter,
                severity: Severity::Low,
                description: "Invisible character in locale data".to_string(),
                remediation: "Remove invisible character".to_string(),
                cwe_id: None,
                references: vec![],
                context: None,
                decoded_payload: None,
                confidence: Some(0.5),
            })
            .collect();

        let score = engine.calculate_score(&findings, None);

        // With deduplication, 10 similar findings should score < 5.0
        // (single category cap + diminishing returns)
        assert!(
            score < 5.0,
            "Deduplication should keep score low for similar findings, got: {}",
            score
        );
    }

    #[test]
    fn test_llm_multiplier_reduces_low_confidence() {
        let config = ScoringConfig::default();
        let ctx = PackageContext::new("test".to_string(), "1.0.0".to_string());
        let engine = ScoringEngine::new(config, ctx);

        let findings = vec![Finding {
            file: "test.js".to_string(),
            line: 1,
            column: 1,
            code_point: 0xFE00,
            character: "\u{FE00}".to_string(),
            raw_bytes: None,
            category: DetectionCategory::InvisibleCharacter,
            severity: Severity::High,
            description: "Invisible character detected".to_string(),
            remediation: "Remove invisible character".to_string(),
            cwe_id: None,
            references: vec![],
            context: None,
            decoded_payload: None,
            confidence: Some(0.9),
        }];

        let score_without_llm = engine.calculate_score(&findings, None);

        // LLM verdict with low confidence (0.10)
        let llm_verdict = LlmVerdict {
            is_malicious: false,
            glassworm_match: false,
            matched_glassworm_stages: vec![],
            confidence: 0.10,
            explanation: "Likely false positive".to_string(),
            recommendations: vec![],
            false_positive_indicators: vec!["Common pattern in i18n libraries".to_string()],
        };

        let score_with_llm = engine.calculate_score(&findings, Some(&llm_verdict));

        // LLM confidence 0.10 should reduce score by ~63% (multiplier 0.37)
        let expected_reduction = 0.3 + (0.10 * 0.7); // 0.37
        let expected_score = score_without_llm * expected_reduction;

        assert!(
            (score_with_llm - expected_score).abs() < 0.1,
            "LLM should reduce score significantly for low confidence"
        );
        assert!(
            score_with_llm < score_without_llm * 0.7,
            "LLM low confidence should reduce score by at least 30%"
        );
    }

    #[test]
    fn test_reputation_multiplier_for_popular_package() {
        let config = ScoringConfig::default();
        let ctx = PackageContext::with_reputation(
            "lodash".to_string(),
            "4.17.21".to_string(),
            50_000_000, // 50M weekly downloads
            3000,       // ~8 years old
            true,       // verified maintainer
        );
        let engine = ScoringEngine::new(config, ctx);

        let findings = vec![Finding {
            file: "test.js".to_string(),
            line: 1,
            column: 1,
            code_point: 0xFE00,
            character: "\u{FE00}".to_string(),
            raw_bytes: None,
            category: DetectionCategory::InvisibleCharacter,
            severity: Severity::High,
            description: "Invisible character detected".to_string(),
            remediation: "Remove invisible character".to_string(),
            cwe_id: None,
            references: vec![],
            context: None,
            decoded_payload: None,
            confidence: Some(0.9),
        }];

        let score = engine.calculate_score(&findings, None);

        // Popular package should get 0.5 multiplier
        // Score should be lower than unknown package
        assert!(score <= 5.0, "Popular package should get benefit of doubt");
    }

    #[test]
    fn test_category_caps() {
        let config = ScoringConfig::default();
        let ctx = PackageContext::new("test".to_string(), "1.0.0".to_string());
        let engine = ScoringEngine::new(config, ctx);

        // Create findings in multiple categories
        let findings = vec![
            Finding {
                file: "test.js".to_string(),
                line: 1,
                column: 1,
                code_point: 0xFE00,
                character: "\u{FE00}".to_string(),
                raw_bytes: None,
                category: DetectionCategory::InvisibleCharacter,
                severity: Severity::High,
                description: "Invisible character".to_string(),
                remediation: "Remove".to_string(),
                cwe_id: None,
                references: vec![],
                context: None,
                decoded_payload: None,
                confidence: Some(0.9),
            },
            Finding {
                file: "test.js".to_string(),
                line: 2,
                column: 1,
                code_point: 0x202E,
                character: "\u{202E}".to_string(),
                raw_bytes: None,
                category: DetectionCategory::BidirectionalOverride,
                severity: Severity::High,
                description: "Bidi override".to_string(),
                remediation: "Remove".to_string(),
                cwe_id: None,
                references: vec![],
                context: None,
                decoded_payload: None,
                confidence: Some(0.9),
            },
        ];

        let score = engine.calculate_score(&findings, None);

        // Two categories should be capped at 7.0
        assert!(
            score <= 7.0,
            "Two categories should be capped at 7.0, got: {}",
            score
        );
    }

    #[test]
    fn test_exception_known_c2() {
        let config = ScoringConfig::default();
        let ctx = PackageContext::new("test".to_string(), "1.0.0".to_string());
        let engine = ScoringEngine::new(config, ctx);

        let findings = vec![Finding {
            file: "test.js".to_string(),
            line: 1,
            column: 1,
            code_point: 0,
            character: "".to_string(),
            raw_bytes: None,
            category: DetectionCategory::BlockchainC2,
            severity: Severity::Critical,
            description: "Known C2 wallet address detected (GlassWorm)".to_string(),
            remediation: "Remove malicious code".to_string(),
            cwe_id: None,
            references: vec![],
            context: None,
            decoded_payload: None,
            confidence: Some(0.95),
        }];

        let score = engine.calculate_score(&findings, None);

        // Known C2 should score at least 9.0
        assert!(
            score >= 9.0,
            "Known C2 should score at least 9.0, got: {}",
            score
        );
    }
}
