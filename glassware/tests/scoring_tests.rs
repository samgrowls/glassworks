//! Scoring System Tests for Phase A.5 Redesign
//!
//! Tests for the new scoring system with:
//! 1. Deduplication (383 similar findings = 1 pattern)
//! 2. LLM confidence impact
//! 3. Reputation multipliers
//! 4. Raised malicious threshold (8.0)
//! 5. Evidence detection maintenance

use glassware::scoring::{ScoringEngine, ScoringConfig};
use glassware::package_context::PackageContext;
use glassware::llm::LlmVerdict;
use glassware_core::{Finding, Severity, DetectionCategory};

/// Test that 383 i18n findings score < 5.0 (deduplication working)
#[test]
fn test_deduplication_many_similar_findings_low_score() {
    let config = ScoringConfig::default();
    let ctx = PackageContext::new("i18n-pkg".to_string(), "1.0.0".to_string());
    let engine = ScoringEngine::new(config, ctx);

    // Simulate 383 similar i18n findings (false positives from locale data)
    let findings: Vec<Finding> = (0..383)
        .map(|i| Finding {
            file: format!("locale/data{}.json", i % 10),
            line: (i % 100) + 1,
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

    // With deduplication, 383 similar findings should score < 5.0
    // (single category cap + diminishing returns)
    assert!(
        score < 5.0,
        "383 similar i18n findings should score < 5.0 with deduplication, got: {}",
        score
    );
    println!("383 similar findings scored: {:.2} (should be < 5.0)", score);
}

/// Test that LLM confidence 0.10 reduces score by ~70%
#[test]
fn test_llm_low_confidence_reduces_score() {
    let config = ScoringConfig::default();
    let ctx = PackageContext::new("test-pkg".to_string(), "1.0.0".to_string());
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

    // LLM verdict with very low confidence (0.10)
    let llm_verdict = LlmVerdict {
        is_malicious: false,
        glassworm_match: false,
        matched_glassworm_stages: vec![],
        confidence: 0.10,
        explanation: "Likely false positive - common pattern in i18n libraries".to_string(),
        recommendations: vec!["Review context before flagging".to_string()],
        false_positive_indicators: vec![
            "Found in locale data file".to_string(),
            "Pattern matches legitimate i18n usage".to_string(),
        ],
    };

    let score_with_llm = engine.calculate_score(&findings, Some(&llm_verdict));

    // LLM confidence 0.10 should give multiplier of 0.3 + (0.10 * 0.7) = 0.37
    // This reduces score by ~63%
    let expected_multiplier = 0.3 + (0.10 * 0.7);
    let expected_score = score_without_llm * expected_multiplier;

    println!("Score without LLM: {:.2}", score_without_llm);
    println!("Score with LLM (0.10 confidence): {:.2}", score_with_llm);
    println!("Expected score: {:.2}", expected_score);

    assert!(
        (score_with_llm - expected_score).abs() < 0.1,
        "LLM should reduce score according to confidence multiplier"
    );

    // Verify significant reduction (at least 50%)
    assert!(
        score_with_llm < score_without_llm * 0.6,
        "LLM low confidence should reduce score significantly"
    );
}

/// Test that LLM high confidence (0.90) keeps score mostly unchanged
#[test]
fn test_llm_high_confidence_minimal_reduction() {
    let config = ScoringConfig::default();
    let ctx = PackageContext::new("test-pkg".to_string(), "1.0.0".to_string());
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

    // LLM verdict with high confidence (0.90)
    let llm_verdict = LlmVerdict {
        is_malicious: true,
        glassworm_match: true,
        matched_glassworm_stages: vec![1, 2, 3],
        confidence: 0.90,
        explanation: "Likely malicious - matches GlassWorm pattern".to_string(),
        recommendations: vec!["Remove package immediately".to_string()],
        false_positive_indicators: vec![],
    };

    let score_with_llm = engine.calculate_score(&findings, Some(&llm_verdict));

    // LLM confidence 0.90 should give multiplier of 0.3 + (0.90 * 0.7) = 0.93
    // This is only a 7% reduction
    let expected_multiplier = 0.3 + (0.90 * 0.7);

    println!("Score without LLM: {:.2}", score_without_llm);
    println!("Score with LLM (0.90 confidence): {:.2}", score_with_llm);
    println!("Expected multiplier: {:.2}", expected_multiplier);

    // High confidence should keep score mostly unchanged (< 15% reduction)
    assert!(
        score_with_llm > score_without_llm * 0.85,
        "LLM high confidence should keep score mostly unchanged"
    );
}

/// Test that popular packages get reputation benefit
#[test]
fn test_reputation_multiplier_popular_package() {
    let config = ScoringConfig::default();
    
    // Popular package: lodash (50M downloads, 8 years old, verified)
    let ctx_popular = PackageContext::with_reputation(
        "lodash".to_string(),
        "4.17.21".to_string(),
        50_000_000, // 50M weekly downloads
        3000,       // ~8 years old
        true,       // verified maintainer
    );
    let engine_popular = ScoringEngine::new(config.clone(), ctx_popular.clone());

    // Unknown package: new suspicious package
    let ctx_unknown = PackageContext::new("suspicious-new-pkg".to_string(), "1.0.0".to_string());
    let engine_unknown = ScoringEngine::new(config, ctx_unknown);

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

    let score_popular = engine_popular.calculate_score(&findings, None);
    let score_unknown = engine_unknown.calculate_score(&findings, None);

    println!("Popular package (lodash) score: {:.2}", score_popular);
    println!("Unknown package score: {:.2}", score_unknown);

    // Popular package should get 0.5 multiplier
    assert_eq!(ctx_popular.reputation_multiplier(), 0.5);
    
    // Popular package should score lower than unknown package
    assert!(
        score_popular < score_unknown,
        "Popular package should score lower due to reputation benefit"
    );

    // The difference should be approximately 50% (0.5 multiplier)
    let ratio = score_popular / score_unknown;
    assert!(
        (ratio - 0.5).abs() < 0.1,
        "Popular package should score approximately 50% of unknown package"
    );
}

/// Test that medium popularity packages get appropriate benefit
#[test]
fn test_reputation_multiplier_medium_package() {
    let config = ScoringConfig::default();
    
    // Medium package: 50K downloads, 1 year old, not verified
    let ctx_medium = PackageContext::with_reputation(
        "medium-lib".to_string(),
        "2.0.0".to_string(),
        50_000,  // 50K weekly downloads
        400,     // ~1+ year old
        false,   // not verified
    );
    let engine = ScoringEngine::new(config, ctx_medium.clone());

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

    // Medium package should get 0.7 multiplier
    assert_eq!(ctx_medium.reputation_multiplier(), 0.7);
    assert_eq!(ctx_medium.reputation_tier(), "tier2_popular");
    
    println!("Medium package score: {:.2}", score);
}

/// Test category caps work correctly
#[test]
fn test_category_caps_single_category() {
    let config = ScoringConfig::default();
    let ctx = PackageContext::new("test".to_string(), "1.0.0".to_string());
    let engine = ScoringEngine::new(config, ctx);

    // Multiple findings in single category
    let findings: Vec<Finding> = (0..10)
        .map(|i| Finding {
            file: format!("file{}.js", i),
            line: 1,
            column: 1,
            code_point: 0xFE00,
            character: "\u{FE00}".to_string(),
            raw_bytes: None,
            category: DetectionCategory::InvisibleCharacter,
            severity: Severity::Critical,
            description: "Invisible character".to_string(),
            remediation: "Remove".to_string(),
            cwe_id: None,
            references: vec![],
            context: None,
            decoded_payload: None,
            confidence: Some(0.9),
        })
        .collect();

    let score = engine.calculate_score(&findings, None);

    // Single category should be capped at 5.0
    assert!(
        score <= 5.0,
        "Single category should be capped at 5.0, got: {}",
        score
    );
}

#[test]
fn test_category_caps_two_categories() {
    let config = ScoringConfig::default();
    let ctx = PackageContext::new("test".to_string(), "1.0.0".to_string());
    let engine = ScoringEngine::new(config, ctx);

    // Findings in two categories
    let findings = vec![
        Finding {
            file: "test.js".to_string(),
            line: 1,
            column: 1,
            code_point: 0xFE00,
            character: "\u{FE00}".to_string(),
            raw_bytes: None,
            category: DetectionCategory::InvisibleCharacter,
            severity: Severity::Critical,
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
            severity: Severity::Critical,
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
fn test_category_caps_three_categories() {
    let config = ScoringConfig::default();
    let ctx = PackageContext::new("test".to_string(), "1.0.0".to_string());
    let engine = ScoringEngine::new(config, ctx);

    // Findings in three categories
    let findings = vec![
        Finding {
            file: "test.js".to_string(),
            line: 1,
            column: 1,
            code_point: 0xFE00,
            character: "\u{FE00}".to_string(),
            raw_bytes: None,
            category: DetectionCategory::InvisibleCharacter,
            severity: Severity::Critical,
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
            severity: Severity::Critical,
            description: "Bidi override".to_string(),
            remediation: "Remove".to_string(),
            cwe_id: None,
            references: vec![],
            context: None,
            decoded_payload: None,
            confidence: Some(0.9),
        },
        Finding {
            file: "test.js".to_string(),
            line: 3,
            column: 1,
            code_point: 0,
            character: "".to_string(),
            raw_bytes: None,
            category: DetectionCategory::Homoglyph,
            severity: Severity::Critical,
            description: "Homoglyph detected".to_string(),
            remediation: "Remove".to_string(),
            cwe_id: None,
            references: vec![],
            context: None,
            decoded_payload: None,
            confidence: Some(0.9),
        },
    ];

    let score = engine.calculate_score(&findings, None);

    // Three categories should be capped at 8.5
    assert!(
        score <= 8.5,
        "Three categories should be capped at 8.5, got: {}",
        score
    );
}

/// Test exceptions (known C2) work
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

/// Test exceptions (GlassWorm C2 polling) work
#[test]
fn test_exception_glassworm_c2_polling() {
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
        description: "GlassWorm C2 polling detected (getSignaturesForAddress + setInterval)".to_string(),
        remediation: "Remove malicious code".to_string(),
        cwe_id: None,
        references: vec![],
        context: None,
        decoded_payload: None,
        confidence: Some(0.95),
    }];

    let score = engine.calculate_score(&findings, None);

    // GlassWorm C2 polling should score at least 9.0
    assert!(
        score >= 9.0,
        "GlassWorm C2 polling should score at least 9.0, got: {}",
        score
    );
}

/// Test exceptions (steganography with decoder) work
#[test]
fn test_exception_steganography_decoder() {
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
        severity: Severity::Critical,
        description: "GlassWorm steganography decoder detected".to_string(),
        remediation: "Remove malicious code".to_string(),
        cwe_id: None,
        references: vec![],
        context: None,
        decoded_payload: None,
        confidence: Some(0.95),
    }];

    let score = engine.calculate_score(&findings, None);

    // Steganography with decoder should score at least 8.5
    assert!(
        score >= 8.5,
        "Steganography with decoder should score at least 8.5, got: {}",
        score
    );
}

/// Test malicious threshold is 8.0
#[test]
fn test_malicious_threshold_is_8() {
    let config = ScoringConfig::default();
    assert_eq!(config.malicious_threshold, 8.0);
    assert_eq!(config.suspicious_threshold, 4.0);
}
