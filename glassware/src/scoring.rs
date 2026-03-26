//! Scoring Engine with Deduplication, LLM Feedback, Reputation, and Tiered Execution
//!
//! This module implements the Phase A.5 scoring system redesign:
//! 1. Deduplicates findings by pattern (383 similar findings = 1 pattern)
//! 2. Weights findings by quality (LLM confidence impact)
//! 3. Applies reputation multipliers (popular packages get benefit of doubt)
//! 4. Raises malicious threshold to 8.0
//! 5. Maintains 100% evidence detection via exceptions
//! 6. Supports tiered detector execution (Phase 2 modular scoring)

use glassware_core::{Finding, Severity, DetectionCategory, FindingPattern};
use std::collections::{HashMap, HashSet};

pub use crate::scoring_config::{ScoringConfig, TierMode, TierDefinition, DetectorWeights, ConditionalRule};
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
    /// A threat score between 0.0 and 10.0 (or higher with tiered mode)
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

        // Check if tiered mode is enabled
        if self.config.tier_config.mode == TierMode::Tiered && !self.config.tier_config.tiers.is_empty() {
            return self.calculate_score_tiered(findings, llm_verdict);
        }

        // Independent mode (original behavior)
        self.calculate_score_independent(findings, llm_verdict)
    }

    /// Calculate score in independent mode (original behavior)
    fn calculate_score_independent(
        &self,
        findings: &[Finding],
        llm_verdict: Option<&LlmVerdict>,
    ) -> f32 {
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

    /// Calculate score in tiered mode (new behavior)
    ///
    /// Tiers execute sequentially, with each tier's threshold gating the next tier.
    /// This allows for conditional scoring where certain detectors only contribute
    /// if other detectors have already flagged the package as suspicious.
    fn calculate_score_tiered(
        &self,
        findings: &[Finding],
        llm_verdict: Option<&LlmVerdict>,
    ) -> f32 {
        let mut tier_sum = 0.0;
        
        // Execute tiers in order
        for tier_def in &self.config.tier_config.tiers {
            // Check if threshold met from previous tiers
            if tier_sum < tier_def.threshold {
                continue;  // Skip this tier
            }
            
            // Run detectors in this tier
            let tier_score = self.run_tier_detectors(tier_def, findings);
            tier_sum += tier_score * tier_def.weight_multiplier;
        }
        
        // Apply conditional rules
        tier_sum = self.apply_conditional_rules(tier_sum, findings);
        
        // Apply LLM multiplier if provided
        if let Some(llm) = llm_verdict {
            tier_sum *= self.calculate_llm_multiplier(llm);
        }
        
        // Apply reputation multiplier
        tier_sum *= self.package_context.reputation_multiplier();
        
        // Apply exceptions (known C2, steganography, etc.)
        tier_sum = self.apply_exceptions(tier_sum, findings);
        
        // In tiered mode, allow scores above 10.0 for confirmed attacks
        // GlassWorm signature can reach 30.0+
        tier_sum.max(0.0)
    }
    
    /// Run detectors for a specific tier
    fn run_tier_detectors(&self, tier_def: &TierDefinition, findings: &[Finding]) -> f32 {
        let mut tier_score = 0.0;
        
        // Group findings by category for this tier
        let tier_findings: Vec<&Finding> = findings.iter()
            .filter(|f| self.detector_matches_tier(&f.category, tier_def))
            .collect();
        
        if tier_findings.is_empty() {
            return 0.0;
        }
        
        // Deduplicate findings within this tier (convert Vec<&Finding> to Vec<Finding>)
        let tier_findings_owned: Vec<Finding> = tier_findings.iter().map(|f| (*f).clone()).collect();
        let patterns = self.deduplicate_findings(&tier_findings_owned);
        
        // Calculate score for this tier
        for pattern in &patterns {
            let pattern_score = self.calculate_pattern_score(pattern);
            
            // Apply detector-specific weight
            let weight = self.config.weights.get_detector_weight(&pattern.category);
            tier_score += pattern_score * weight;
        }
        
        tier_score
    }
    
    /// Check if a detector category matches a tier's detector list
    fn detector_matches_tier(&self, category: &DetectionCategory, tier_def: &TierDefinition) -> bool {
        let category_name = self.category_to_detector_name(category);
        tier_def.detectors.iter().any(|d| d == &category_name)
    }
    
    /// Convert DetectionCategory to detector name string
    fn category_to_detector_name(&self, category: &DetectionCategory) -> String {
        match category {
            DetectionCategory::InvisibleCharacter => "invisible_char".to_string(),
            DetectionCategory::Homoglyph => "homoglyph".to_string(),
            DetectionCategory::BidirectionalOverride => "bidirectional_override".to_string(),
            DetectionCategory::UnicodeTag => "unicode_tag".to_string(),
            DetectionCategory::NormalizationAttack => "normalization_attack".to_string(),
            DetectionCategory::GlasswarePattern => "glassware_pattern".to_string(),
            DetectionCategory::EmojiObfuscation => "emoji_obfuscation".to_string(),
            DetectionCategory::SteganoPayload => "stegano_payload".to_string(),
            DetectionCategory::DecoderFunction => "decoder_function".to_string(),
            DetectionCategory::PipeDelimiterStego => "pipe_delimiter_stego".to_string(),
            DetectionCategory::EncryptedPayload => "encrypted_payload".to_string(),
            DetectionCategory::HeaderC2 => "header_c2".to_string(),
            DetectionCategory::HardcodedKeyDecryption => "hardcoded_key_decryption".to_string(),
            DetectionCategory::Rc4Pattern => "rc4_pattern".to_string(),
            DetectionCategory::LocaleGeofencing => "locale_geofencing".to_string(),
            DetectionCategory::TimeDelaySandboxEvasion => "time_delay_sandbox_evasion".to_string(),
            DetectionCategory::BlockchainC2 => "blockchain_c2".to_string(),
            DetectionCategory::RddAttack => "rdd_attack".to_string(),
            DetectionCategory::ForceMemoPython => "forcememo_python".to_string(),
            DetectionCategory::JpdAuthor => "jpd_author".to_string(),
            DetectionCategory::XorShiftObfuscation => "xor_shift_obfuscation".to_string(),
            DetectionCategory::IElevatorCom => "ielevator_com".to_string(),
            DetectionCategory::ApcInjection => "apc_injection".to_string(),
            DetectionCategory::MemexecLoader => "memexec_loader".to_string(),
            DetectionCategory::ExfilSchema => "exfil_schema".to_string(),
            DetectionCategory::SocketIOC2 => "socketio_c2".to_string(),
            DetectionCategory::Unknown => "unknown".to_string(),
        }
    }
    
    /// Apply conditional rules to adjust score
    fn apply_conditional_rules(&self, current_score: f32, findings: &[Finding]) -> f32 {
        let mut final_score = current_score;
        
        // Build detector score map
        let mut detector_scores: HashMap<String, f32> = HashMap::new();
        let mut detector_counts: HashMap<String, u32> = HashMap::new();
        
        for finding in findings {
            let name = self.category_to_detector_name(&finding.category);
            *detector_scores.entry(name.clone()).or_insert(0.0) += 1.0;
            *detector_counts.entry(name).or_insert(0) += 1;
        }
        
        // Evaluate each rule
        for rule in &self.config.conditional_rules {
            if self.evaluate_condition(rule, &detector_scores, &detector_counts, final_score) {
                final_score = self.apply_rule_action(rule, final_score);
            }
        }
        
        final_score
    }
    
    /// Evaluate a conditional rule's condition
    fn evaluate_condition(
        &self,
        rule: &ConditionalRule,
        detector_scores: &HashMap<String, f32>,
        detector_counts: &HashMap<String, u32>,
        final_score: f32,
    ) -> bool {
        // Simple condition parser
        // Supports: AND, OR, comparisons
        let condition = &rule.condition;
        
        // Handle AND
        if condition.contains(" AND ") {
            let parts: Vec<&str> = condition.split(" AND ").collect();
            return parts.iter().all(|part| {
                self.evaluate_simple_condition(part.trim(), detector_scores, detector_counts, final_score)
            });
        }
        
        // Handle OR
        if condition.contains(" OR ") {
            let parts: Vec<&str> = condition.split(" OR ").collect();
            return parts.iter().any(|part| {
                self.evaluate_simple_condition(part.trim(), detector_scores, detector_counts, final_score)
            });
        }
        
        // Simple condition
        self.evaluate_simple_condition(condition, detector_scores, detector_counts, final_score)
    }
    
    /// Evaluate a simple condition (no AND/OR)
    fn evaluate_simple_condition(
        &self,
        condition: &str,
        detector_scores: &HashMap<String, f32>,
        detector_counts: &HashMap<String, u32>,
        final_score: f32,
    ) -> bool {
        // Parse: <field> <operator> <value>
        let parts: Vec<&str> = condition.split_whitespace().collect();
        if parts.len() != 3 {
            return false;
        }
        
        let field = parts[0];
        let op = parts[1];
        let value_str = parts[2];
        
        // Get field value
        let field_value = self.get_field_value(field, detector_scores, detector_counts, final_score);
        let compare_value: f32 = value_str.parse().unwrap_or(0.0);
        
        // Evaluate operator
        match op {
            "==" => (field_value - compare_value).abs() < 0.001,
            "!=" => (field_value - compare_value).abs() >= 0.001,
            "<" => field_value < compare_value,
            ">" => field_value > compare_value,
            "<=" => field_value <= compare_value,
            ">=" => field_value >= compare_value,
            _ => false,
        }
    }
    
    /// Get value for a field reference
    fn get_field_value(
        &self,
        field: &str,
        detector_scores: &HashMap<String, f32>,
        detector_counts: &HashMap<String, u32>,
        final_score: f32,
    ) -> f32 {
        if field == "final_score" {
            return final_score;
        }
        
        // Parse: <detector>.<field>
        let parts: Vec<&str> = field.split('.').collect();
        if parts.len() != 2 {
            return 0.0;
        }
        
        let detector = parts[0];
        let field_type = parts[1];
        
        match field_type {
            "score" => *detector_scores.get(detector).unwrap_or(&0.0),
            "count" => *detector_counts.get(detector).unwrap_or(&0) as f32,
            "detected" => {
                let count = detector_counts.get(detector).unwrap_or(&0);
                if *count > 0 { 1.0 } else { 0.0 }
            },
            _ => 0.0,
        }
    }
    
    /// Apply a rule's action to the score
    fn apply_rule_action(&self, rule: &ConditionalRule, current_score: f32) -> f32 {
        let action = &rule.action;
        
        // Parse action: <target> <op> <value>
        // Supported: final_score = <value>, final_score += <value>, final_score *= <multiplier>
        if action.starts_with("final_score") {
            if action.contains(" = ") {
                let parts: Vec<&str> = action.split(" = ").collect();
                if parts.len() == 2 {
                    if let Ok(value) = parts[1].parse::<f32>() {
                        return value;
                    }
                }
            } else if action.contains(" += ") {
                let parts: Vec<&str> = action.split(" += ").collect();
                if parts.len() == 2 {
                    if let Ok(value) = parts[1].parse::<f32>() {
                        return current_score + value;
                    }
                }
            } else if action.contains(" *= ") {
                let parts: Vec<&str> = action.split(" *= ").collect();
                if parts.len() == 2 {
                    if let Ok(multiplier) = parts[1].parse::<f32>() {
                        return current_score * multiplier;
                    }
                }
            }
        }
        
        // For now, weight adjustments are handled differently
        // This is a simplified implementation
        current_score
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
    ///
    /// EXCEPTION: If patterns include Critical severity with high count (>100),
    /// the cap is relaxed to allow legitimate high-volume detections.
    fn apply_category_caps(
        &self,
        score: f32,
        patterns: &[FindingPattern],
    ) -> f32 {
        let categories: HashSet<_> = patterns.iter().map(|p| &p.category).collect();
        let category_count = categories.len();

        // Check for high-volume critical findings (exception to caps)
        let has_high_volume_critical = patterns.iter().any(|p| {
            p.max_severity == Severity::Critical && p.count > 100
        });

        if has_high_volume_critical {
            // Relax caps for high-volume critical findings
            // This catches real attacks like aifabrix-miso-client (9000+ encrypted payload findings)
            return score.min(10.0);
        }

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
