//! Campaign Intelligence Integration Tests
//!
//! Tests demonstrating campaign detection across multiple packages.

use glassware_core::{
    campaign::{AnalyzedPackage, CampaignIntelligence, CampaignType, Infrastructure},
    finding::{DetectionCategory, Finding, Severity},
};

#[test]
fn test_glassworm_campaign_detection() {
    // Simulate GlassWorm campaign packages with shared infrastructure
    let mut intel = CampaignIntelligence::new();

    // Package 1: GlassWorm with blockchain C2
    let mut pkg1 = AnalyzedPackage::new("malicious-pkg1".to_string(), "code1".to_string(), 1000);
    pkg1.info.domains.push("evil-c2.com".to_string());
    pkg1.info.wallets.push("BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC".to_string());
    pkg1.info.authors.push("attacker1".to_string());
    pkg1.info.findings.push(Finding::new(
        "index.js", 1, 1, 0xFE00, '\u{FE00}',
        DetectionCategory::SteganoPayload, Severity::Critical,
        "Steganographic payload", "Remove it"
    ));
    pkg1.info.findings.push(Finding::new(
        "index.js", 2, 1, 0, '\0',
        DetectionCategory::BlockchainC2, Severity::Critical,
        "Solana RPC detected", "Investigate"
    ));

    // Package 2: Same campaign, shared infrastructure
    let mut pkg2 = AnalyzedPackage::new("malicious-pkg2".to_string(), "code1".to_string(), 2000);
    pkg2.info.domains.push("evil-c2.com".to_string()); // Shared domain
    pkg2.info.wallets.push("BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC".to_string()); // Shared wallet
    pkg2.info.authors.push("attacker1".to_string()); // Shared author
    pkg2.info.findings.push(Finding::new(
        "index.js", 1, 1, 0xFE00, '\u{FE00}',
        DetectionCategory::InvisibleCharacter, Severity::Critical,
        "Invisible character", "Remove it"
    ));

    // Package 3: Same campaign, similar code
    let mut pkg3 = AnalyzedPackage::new("malicious-pkg3".to_string(), "code1".to_string(), 3000);
    pkg3.info.domains.push("evil-c2.com".to_string()); // Shared domain
    pkg3.info.authors.push("attacker1".to_string()); // Shared author
    pkg3.info.findings.push(Finding::new(
        "index.js", 1, 1, 0xFE00, '\u{FE00}',
        DetectionCategory::DecoderFunction, Severity::High,
        "Decoder function", "Remove it"
    ));

    intel.add_package(pkg1);
    intel.add_package(pkg2);
    intel.add_package(pkg3);

    // Should detect a GlassWorm campaign
    let campaigns = intel.get_campaigns();
    assert!(!campaigns.is_empty(), "Should detect at least one campaign");

    let campaign = &campaigns[0];
    assert_eq!(campaign.classification, CampaignType::GlassWorm);
    assert_eq!(campaign.packages.len(), 3);
    assert!(campaign.confidence > 0.5, "Confidence should be high for shared infrastructure");

    // Verify infrastructure tracking
    assert!(campaign.infrastructure.domains.contains(&"evil-c2.com".to_string()));
    assert!(campaign.infrastructure.wallets.contains(&"BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC".to_string()));
    assert!(campaign.infrastructure.authors.contains(&"attacker1".to_string()));
}

#[test]
fn test_phantom_raven_campaign_detection() {
    // Simulate PhantomRaven campaign with RDD attacks
    let mut intel = CampaignIntelligence::new();

    let mut pkg1 = AnalyzedPackage::new("rdd-pkg1".to_string(), "rdd_code".to_string(), 1000);
    pkg1.info.domains.push("packages.storeartifact.com".to_string());
    pkg1.info.authors.push("jpd_author".to_string());
    pkg1.info.findings.push(Finding::new(
        "index.js", 1, 1, 0, '\0',
        DetectionCategory::RddAttack, Severity::High,
        "Remote Dynamic Dependency", "Investigate"
    ));

    let mut pkg2 = AnalyzedPackage::new("rdd-pkg2".to_string(), "rdd_code".to_string(), 2000);
    pkg2.info.domains.push("packages.storeartifact.com".to_string()); // Shared domain
    pkg2.info.authors.push("jpd_author".to_string()); // Shared author
    pkg2.info.findings.push(Finding::new(
        "index.js", 1, 1, 0, '\0',
        DetectionCategory::RddAttack, Severity::High,
        "Remote Dynamic Dependency", "Investigate"
    ));

    intel.add_package(pkg1);
    intel.add_package(pkg2);

    let campaigns = intel.get_campaigns();
    assert!(!campaigns.is_empty());

    let campaign = &campaigns[0];
    assert_eq!(campaign.classification, CampaignType::PhantomRaven);
}

#[test]
fn test_sandworm_mode_detection() {
    // Simulate SANDWORM_MODE campaign with time delays and MCP injection
    let mut intel = CampaignIntelligence::new();

    let mut pkg1 = AnalyzedPackage::new("sandworm-pkg1".to_string(), "delay_code".to_string(), 1000);
    pkg1.info.domains.push("mcp-server.evil.com".to_string());
    pkg1.info.findings.push(Finding::new(
        "index.js", 1, 1, 0, '\0',
        DetectionCategory::TimeDelaySandboxEvasion, Severity::Critical,
        "48-hour time delay", "Investigate"
    ));

    let mut pkg2 = AnalyzedPackage::new("sandworm-pkg2".to_string(), "delay_code".to_string(), 2000);
    pkg2.info.domains.push("mcp-server.evil.com".to_string()); // Shared MCP domain
    pkg2.info.findings.push(Finding::new(
        "index.js", 1, 1, 0, '\0',
        DetectionCategory::TimeDelaySandboxEvasion, Severity::Critical,
        "Time-delay sandbox evasion", "Investigate"
    ));

    intel.add_package(pkg1);
    intel.add_package(pkg2);

    let campaigns = intel.get_campaigns();
    assert!(!campaigns.is_empty());

    let campaign = &campaigns[0];
    assert_eq!(campaign.classification, CampaignType::SandwormMode);
}

#[test]
fn test_infrastructure_reuse_stats() {
    let mut intel = CampaignIntelligence::new();

    // Create packages with varying infrastructure reuse
    let mut pkg1 = AnalyzedPackage::new("pkg1".to_string(), "code".to_string(), 1000);
    pkg1.info.domains.push("domain1.com".to_string());
    pkg1.info.domains.push("domain2.com".to_string());
    pkg1.info.wallets.push("wallet1".to_string());
    pkg1.info.authors.push("author1".to_string());

    let mut pkg2 = AnalyzedPackage::new("pkg2".to_string(), "code".to_string(), 2000);
    pkg2.info.domains.push("domain1.com".to_string()); // Shared with pkg1
    pkg2.info.wallets.push("wallet2".to_string());
    pkg2.info.authors.push("author1".to_string()); // Shared with pkg1

    let mut pkg3 = AnalyzedPackage::new("pkg3".to_string(), "code".to_string(), 3000);
    pkg3.info.domains.push("domain3.com".to_string());
    pkg3.info.wallets.push("wallet1".to_string()); // Shared with pkg1
    pkg3.info.authors.push("author2".to_string());

    intel.add_package(pkg1);
    intel.add_package(pkg2);
    intel.add_package(pkg3);

    let stats = intel.get_infrastructure_stats();

    assert_eq!(stats.unique_domains, 3);
    assert_eq!(stats.unique_wallets, 2);
    assert_eq!(stats.unique_authors, 2);
    assert!(stats.reused_domains >= 1, "domain1.com is reused");
    assert!(stats.reused_wallets >= 1, "wallet1 is reused");
    assert!(stats.reused_authors >= 1, "author1 is reused");
}

#[test]
fn test_code_similarity_clustering() {
    let mut intel = CampaignIntelligence::new();

    // Packages with identical code
    let pkg1 = AnalyzedPackage::new("similar1".to_string(), "function test() { return 42; }".to_string(), 1000);
    let pkg2 = AnalyzedPackage::new("similar2".to_string(), "function test() { return 42; }".to_string(), 2000);
    let pkg3 = AnalyzedPackage::new("different".to_string(), "completely different code here".to_string(), 3000);

    intel.add_package(pkg1);
    intel.add_package(pkg2);
    intel.add_package(pkg3);

    // Check similarity
    let sim = intel.get_code_similarity("similar1", "similar2");
    assert!(sim >= 0.9, "Identical code should have high similarity");

    let sim = intel.get_code_similarity("similar1", "different");
    assert!(sim < 0.5, "Different code should have low similarity");
}

#[test]
fn test_related_packages_detection() {
    let mut tracker = glassware_core::InfrastructureTracker::new();

    let mut infra1 = Infrastructure::new();
    infra1.add_domain("evil.com".to_string());
    infra1.add_author("attacker".to_string());

    let mut infra2 = Infrastructure::new();
    infra2.add_domain("evil.com".to_string()); // Shared
    infra2.add_wallet("wallet1".to_string());

    let mut infra3 = Infrastructure::new();
    infra3.add_wallet("wallet1".to_string()); // Shared with pkg2
    infra3.add_author("attacker".to_string()); // Shared with pkg1

    tracker.track("pkg1", &infra1);
    tracker.track("pkg2", &infra2);
    tracker.track("pkg3", &infra3);

    // pkg1 should be related to pkg2 (shared domain) and pkg3 (shared author)
    let related = tracker.find_related("pkg1");
    assert!(related.contains(&"pkg2".to_string()));
    assert!(related.contains(&"pkg3".to_string()));
}

#[test]
fn test_campaign_intelligence_clear() {
    let mut intel = CampaignIntelligence::new();

    let pkg = AnalyzedPackage::new("pkg1".to_string(), "code".to_string(), 1000);
    intel.add_package(pkg);

    assert_eq!(intel.get_packages().len(), 1);

    intel.clear();

    assert_eq!(intel.get_packages().len(), 0);
    assert!(intel.get_campaigns().is_empty());
}
