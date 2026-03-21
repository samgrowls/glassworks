//! E2: Host Indicator Enrichment
//!
//! Cross-references JS/binary findings with host artifacts to add context.
//! This is a **post-processing layer**, not a detector.
//!
//! ## Purpose
//!
//! Takes findings from JS/binary scans and host scan results, then:
//! - Adds cross-reference notes when correlated findings exist
//! - Boosts confidence scores for corroborated detections
//! - Does NOT change severity - only adds metadata to `Finding.context`
//!
//! ## Enrichment Examples
//!
//! - Binary finding (G9 memexec) + matching `.node` file in filesystem (G1)
//!   → add cross-reference note, boost confidence
//! - JS exfil schema (G4) + Chrome extension with matching permissions (G2)
//!   → add correlation note
//! - Socket.IO C2 (G5) + Solana wallet C2 (G10)
//!   → note multi-channel C2 infrastructure

use crate::finding::{DetectionCategory, Finding};
use std::collections::HashSet;

/// Context for enrichment operations
pub struct EnrichmentContext {
    /// Filesystem scan findings (from G1)
    pub filesystem_findings: Vec<Finding>,
    /// Chrome prefs findings (from G2)
    pub chrome_findings: Vec<Finding>,
    /// Binary scan findings (from G6-G9, G11)
    pub binary_findings: Vec<Finding>,
    /// JS scan findings (from other detectors)
    pub js_findings: Vec<Finding>,
}

impl EnrichmentContext {
    /// Create a new enrichment context
    pub fn new() -> Self {
        Self {
            filesystem_findings: Vec::new(),
            chrome_findings: Vec::new(),
            binary_findings: Vec::new(),
            js_findings: Vec::new(),
        }
    }

    /// Set filesystem findings
    pub fn with_filesystem(mut self, findings: Vec<Finding>) -> Self {
        self.filesystem_findings = findings;
        self
    }

    /// Set Chrome findings
    pub fn with_chrome(mut self, findings: Vec<Finding>) -> Self {
        self.chrome_findings = findings;
        self
    }

    /// Set binary findings
    pub fn with_binary(mut self, findings: Vec<Finding>) -> Self {
        self.binary_findings = findings;
        self
    }

    /// Set JS findings
    pub fn with_js(mut self, findings: Vec<Finding>) -> Self {
        self.js_findings = findings;
        self
    }
}

impl Default for EnrichmentContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Enrich findings with cross-reference context
pub fn enrich_findings(findings: Vec<Finding>, context: &EnrichmentContext) -> Vec<Finding> {
    let mut enriched = Vec::new();

    for mut finding in findings {
        let mut correlations = Vec::new();

        // Check for filesystem correlation
        if has_filesystem_correlation(&finding, &context.filesystem_findings) {
            correlations.push("filesystem artifact found");
        }

        // Check for Chrome correlation
        if has_chrome_correlation(&finding, &context.chrome_findings) {
            correlations.push("Chrome extension correlation");
        }

        // Check for binary correlation
        if has_binary_correlation(&finding, &context.binary_findings) {
            correlations.push("binary payload correlation");
        }

        // Check for JS/binary correlation
        if has_js_binary_correlation(&finding, &context.js_findings, &context.binary_findings) {
            correlations.push("JS+binary multi-layer detection");
        }

        // Add correlation context if any found
        if !correlations.is_empty() {
            let existing_context = finding.context.clone().unwrap_or_default();
            let correlation_str = correlations.join(", ");
            finding.context = Some(format!(
                "{} | Correlations: {}",
                existing_context, correlation_str
            ));

            // Boost confidence if not already high
            if let Some(conf) = finding.confidence {
                if conf < 0.9 {
                    finding.confidence = Some((conf + 0.1).min(0.95));
                }
            } else {
                finding.confidence = Some(0.75);
            }
        }

        enriched.push(finding);
    }

    enriched
}

/// Check if a finding has a filesystem correlation
fn has_filesystem_correlation(finding: &Finding, fs_findings: &[Finding]) -> bool {
    // Look for matching file paths or related artifacts
    for fs_finding in fs_findings {
        // Same file path
        if finding.file == fs_finding.file {
            return true;
        }

        // Binary finding + .node file in filesystem
        if is_binary_category(&finding.category) && fs_finding.file.ends_with(".node") {
            return true;
        }

        // Exfil schema + Chrome extension
        if finding.category == DetectionCategory::ExfilSchema
            && fs_finding.context.as_ref().map_or(false, |c| c.contains("extension"))
        {
            return true;
        }
    }

    false
}

/// Check if a finding has a Chrome correlation
fn has_chrome_correlation(finding: &Finding, chrome_findings: &[Finding]) -> bool {
    for chrome_finding in chrome_findings {
        // Critical Chrome finding + any other finding
        if chrome_finding.severity == crate::finding::Severity::Critical {
            return true;
        }

        // Exfil schema + Chrome extension with suspicious permissions
        if finding.category == DetectionCategory::ExfilSchema
            && chrome_finding.context.as_ref().map_or(false, |c| {
                c.contains("permissions") || c.contains("nativeMessaging")
            })
        {
            return true;
        }
    }

    false
}

/// Check if a finding has a binary correlation
fn has_binary_correlation(finding: &Finding, binary_findings: &[Finding]) -> bool {
    for bin_finding in binary_findings {
        // Same file path
        if finding.file == bin_finding.file {
            return true;
        }

        // Multiple binary findings on same file
        if is_binary_category(&finding.category)
            && is_binary_category(&bin_finding.category)
            && finding.file == bin_finding.file
        {
            return true;
        }
    }

    false
}

/// Check for JS + binary multi-layer correlation
fn has_js_binary_correlation(
    finding: &Finding,
    js_findings: &[Finding],
    binary_findings: &[Finding],
) -> bool {
    let has_js = js_findings.iter().any(|f| f.file == finding.file);
    let has_binary = binary_findings.iter().any(|f| f.file == finding.file);

    has_js && has_binary
}

/// Check if a category is binary-related
fn is_binary_category(category: &DetectionCategory) -> bool {
    matches!(
        category,
        DetectionCategory::XorShiftObfuscation
            | DetectionCategory::IElevatorCom
            | DetectionCategory::ApcInjection
            | DetectionCategory::MemexecLoader
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::Severity;

    #[test]
    fn test_enrichment_context_creation() {
        let ctx = EnrichmentContext::new();
        assert!(ctx.filesystem_findings.is_empty());
        assert!(ctx.chrome_findings.is_empty());
        assert!(ctx.binary_findings.is_empty());
        assert!(ctx.js_findings.is_empty());
    }

    #[test]
    fn test_enrichment_adds_correlation_context() {
        let finding = Finding::new(
            "test.node",
            1,
            1,
            0,
            '\0',
            DetectionCategory::MemexecLoader,
            Severity::High,
            "memexec loader detected",
            "Review for fileless execution",
        );

        let fs_finding = Finding::new(
            "test.node",
            1,
            1,
            0,
            '\0',
            DetectionCategory::Unknown,
            Severity::High,
            "payload file found",
            "Filesystem artifact",
        );

        let ctx = EnrichmentContext::new().with_filesystem(vec![fs_finding]);
        let enriched = enrich_findings(vec![finding], &ctx);

        assert_eq!(enriched.len(), 1);
        assert!(enriched[0].context.as_ref().unwrap().contains("Correlations:"));
        assert!(enriched[0].context.as_ref().unwrap().contains("filesystem artifact"));
    }

    #[test]
    fn test_enrichment_boosts_confidence() {
        let finding = Finding::new(
            "test.node",
            1,
            1,
            0,
            '\0',
            DetectionCategory::MemexecLoader,
            Severity::High,
            "memexec loader detected",
            "Review for fileless execution",
        )
        .with_confidence(0.7);

        let fs_finding = Finding::new(
            "test.node",
            1,
            1,
            0,
            '\0',
            DetectionCategory::Unknown,
            Severity::High,
            "payload file found",
            "Filesystem artifact",
        );

        let ctx = EnrichmentContext::new().with_filesystem(vec![fs_finding]);
        let enriched = enrich_findings(vec![finding], &ctx);

        assert_eq!(enriched.len(), 1);
        assert!(enriched[0].confidence.unwrap_or(0.0) > 0.7);
    }

    #[test]
    fn test_enrichment_no_correlation_no_change() {
        let finding = Finding::new(
            "test.js",
            1,
            1,
            0,
            '\0',
            DetectionCategory::InvisibleCharacter,
            Severity::High,
            "invisible char detected",
            "Review for steganography",
        )
        .with_confidence(0.8);

        let ctx = EnrichmentContext::new();
        let enriched = enrich_findings(vec![finding], &ctx);

        assert_eq!(enriched.len(), 1);
        // No correlation context added
        assert!(enriched[0].context.is_none() || !enriched[0].context.as_ref().unwrap().contains("Correlations:"));
        // Confidence unchanged
        assert_eq!(enriched[0].confidence, Some(0.8));
    }

    #[test]
    fn test_chrome_correlation_with_critical_finding() {
        let finding = Finding::new(
            "package.json",
            1,
            1,
            0,
            '\0',
            DetectionCategory::ExfilSchema,
            Severity::High,
            "exfil schema detected",
            "Review for data exfiltration",
        );

        let chrome_finding = Finding::new(
            "Chrome/Preferences",
            1,
            1,
            0,
            '\0',
            DetectionCategory::Unknown,
            Severity::Critical,
            "GlassWorm extension signature",
            "location=4 + creation_flags=38 + from_webstore=false",
        );

        let ctx = EnrichmentContext::new().with_chrome(vec![chrome_finding]);
        let enriched = enrich_findings(vec![finding], &ctx);

        assert_eq!(enriched.len(), 1);
        assert!(enriched[0].context.as_ref().unwrap().contains("Correlations:"));
        assert!(enriched[0].context.as_ref().unwrap().contains("Chrome extension"));
    }

    #[test]
    fn test_js_binary_multi_layer_correlation() {
        let finding = Finding::new(
            "index.js",
            1,
            1,
            0,
            '\0',
            DetectionCategory::SocketIOC2,
            Severity::High,
            "Socket.IO C2 detected",
            "Review for C2 communication",
        );

        let js_finding = Finding::new(
            "index.js",
            1,
            1,
            0,
            '\0',
            DetectionCategory::GlasswarePattern,
            Severity::Medium,
            "GlassWare pattern",
            "Decoder function found",
        );

        let binary_finding = Finding::new(
            "index.js",
            1,
            1,
            0,
            '\0',
            DetectionCategory::MemexecLoader,
            Severity::High,
            "memexec loader",
            "Fileless execution",
        );

        let ctx = EnrichmentContext::new()
            .with_js(vec![js_finding])
            .with_binary(vec![binary_finding]);

        let enriched = enrich_findings(vec![finding], &ctx);

        assert_eq!(enriched.len(), 1);
        assert!(enriched[0].context.as_ref().unwrap().contains("Correlations:"));
        assert!(enriched[0].context.as_ref().unwrap().contains("multi-layer"));
    }

    #[test]
    fn test_enrichment_is_noop_without_correlations() {
        let finding = Finding::new(
            "clean.js",
            1,
            1,
            0,
            '\0',
            DetectionCategory::InvisibleCharacter,
            Severity::Low,
            "invisible char",
            "Review",
        );

        let ctx = EnrichmentContext::new();
        let enriched = enrich_findings(vec![finding.clone()], &ctx);

        assert_eq!(enriched.len(), 1);
        // Should be essentially unchanged
        assert_eq!(enriched[0].file, finding.file);
        assert_eq!(enriched[0].severity, finding.severity);
    }
}
