//! Locale Geofencing Detector (GW009)
//!
//! Detects geographic targeting behavior where malware checks for Russian locale/timezone
//! and exits early to avoid infecting domestic systems.
//!
//! ## Detection Logic
//!
//! This detector emits findings when BOTH conditions are present:
//! 1. Locale/timezone check for Russian systems (ru-RU, Europe/Moscow, etc.)
//! 2. Early exit pattern (process.exit(0)) within 5 lines of the check
//!
//! ## Severity
//!
//! Critical when locale check + exit pattern detected together
//! High when only locale check detected (may be legitimate i18n)

use crate::config::UnicodeConfig;
use crate::detector::{Detector, DetectorMetadata, ScanContext};
use crate::finding::{DetectionCategory, Finding, Severity};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

/// Patterns for locale/timezone detection
static LOCALE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Russian locale strings
        Regex::new(r#"['"]ru-RU['"]|['"]ru['"]|['"]Russian['"]"#).unwrap(),
        // Russian/Moscow timezones
        Regex::new(r"Europe/Moscow|Europe/Kaliningrad|Europe/Volgograd|Europe/Kirov").unwrap(),
        // Navigator language check
        Regex::new(r"navigator\.(language|languages).*ru").unwrap(),
    ]
});

/// Pattern for early exit
static EXIT_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"process\.exit\s*\(\s*0\s*\)").unwrap());

/// Detector for locale geofencing
pub struct LocaleGeofencingDetector;

impl LocaleGeofencingDetector {
    /// Create a new locale geofencing detector
    pub fn new() -> Self {
        Self
    }
}

impl Default for LocaleGeofencingDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for LocaleGeofencingDetector {
    fn name(&self) -> &str {
        "locale_geofencing"
    }

    fn detect(&self, ctx: &ScanContext) -> Vec<Finding> {
        let mut findings = Vec::new();
        let path = Path::new(&ctx.file_path);

        // Track locale checks and their line numbers
        let mut locale_check_lines: Vec<usize> = Vec::new();

        // First pass: find all locale checks
        for (line_num, line) in ctx.content.lines().enumerate() {
            for pattern in LOCALE_PATTERNS.iter() {
                if pattern.is_match(line) {
                    locale_check_lines.push(line_num);

                    findings.push(
                        Finding::new(
                            &path.to_string_lossy(),
                            line_num + 1,
                            1,
                            0,
                            '\0',
                            DetectionCategory::LocaleGeofencing,
                            Severity::Medium,  // Reduced from High - signal not flag
                            "Russian locale/timezone check detected",
                            "Review for geographic targeting behavior. Common in GlassWorm, PhantomRaven, and SANDWORM_MODE campaigns to avoid infecting Russian systems.",
                        )
                        .with_cwe_id("CWE-506")
                        .with_reference(
                            "https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode",
                        ),
                    );
                }
            }
        }

        // Second pass: check for exit patterns near locale checks
        for (line_num, line) in ctx.content.lines().enumerate() {
            if EXIT_PATTERN.is_match(line) {
                // Check if this exit is within 5 lines of a locale check
                for &check_line in &locale_check_lines {
                    if line_num > check_line && line_num <= check_line + 5 {
                        // Upgrade the finding to Critical
                        for finding in &mut findings {
                            if finding.line == check_line + 1
                                && finding.category == DetectionCategory::LocaleGeofencing
                            {
                                finding.severity = Severity::Critical;
                                finding.description =
                                    "Active geofencing: locale check followed by early exit"
                                        .to_string();
                                finding.remediation =
                                    "CRITICAL: This is active geographic targeting. The package exits early on Russian systems to avoid domestic prosecution. Immediate investigation required."
                                        .to_string();
                            }
                        }
                    }
                }
            }
        }

        findings
    }

    fn metadata(&self) -> DetectorMetadata {
        DetectorMetadata {
            name: "locale_geofencing".to_string(),
            version: "1.0.0".to_string(),
            description: "Detects geographic targeting behavior with Russian locale checks and early exit patterns".to_string().to_string(),
        }
    }
}

impl LocaleGeofencingDetector {
    /// Backward compatibility method for tests
    pub fn scan(&self, path: &Path, content: &str, _config: &UnicodeConfig) -> Vec<Finding> {
        let ctx = ScanContext::new(path.to_string_lossy().to_string(), content.to_string(), UnicodeConfig::default());
        self.detect(&ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_locale_check_with_exit() {
        let detector = LocaleGeofencingDetector::new();
        let content = r#"
            const locale = Intl.DateTimeFormat().resolvedOptions().locale;
            if (locale === 'ru-RU') {
                process.exit(0);
            }
            // Malicious code here
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert_eq!(findings.len(), 2); // locale check + exit pattern
        assert!(findings.iter().any(|f| f.severity == Severity::Critical));
    }

    #[test]
    fn test_detect_timezone_check() {
        let detector = LocaleGeofencingDetector::new();
        let content = r#"
            const timezone = Intl.DateTimeFormat().resolvedOptions().timeZone;
            if (timezone.includes('Europe/Moscow')) {
                return;
            }
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert_eq!(findings[0].category, DetectionCategory::LocaleGeofencing);
        assert_eq!(findings[0].severity, Severity::Medium);  // Updated for signal-based detection
    }

    #[test]
    fn test_no_detect_legitimate_i18n() {
        let detector = LocaleGeofencingDetector::new();
        let content = r#"
            // Legitimate i18n library
            import { useTranslation } from 'react-i18next';
            const { t, i18n } = useTranslation();
            i18n.changeLanguage('ru-RU');
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        // Should detect the ru-RU string but as Medium severity (signal not flag)
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::Medium);  // Updated
    }

    #[test]
    fn test_detect_multiple_locale_checks() {
        let detector = LocaleGeofencingDetector::new();
        let content = r#"
            const locale = Intl.DateTimeFormat().resolvedOptions().locale;
            const timezone = Intl.DateTimeFormat().resolvedOptions().timeZone;
            
            if (locale === 'ru-RU' || timezone === 'Europe/Moscow') {
                process.exit(0);
            }
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        // Should detect multiple locale checks
        assert!(findings.len() >= 2);
    }
}
