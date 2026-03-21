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

use crate::detector::{Detector, DetectorMetadata, DetectorTier};
use crate::finding::{DetectionCategory, Finding, Severity};
use crate::ir::FileIR;
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

/// Patterns for locale/timezone detection
static LOCALE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Russian locale strings - must be exact quoted values
        Regex::new(r#"['"]ru-RU['"]"#).unwrap(),
        // Russian/Moscow timezones - must be exact quoted strings
        Regex::new(r#"['"]Europe/Moscow['"]"#).unwrap(),
        // Navigator language check for Russian - must be checking for 'ru' specifically
        Regex::new(r#"navigator\.language\s*===\s*['"]ru['"]"#).unwrap(),
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

    fn tier(&self) -> DetectorTier {
        DetectorTier::Tier3Behavioral
    }

    fn cost(&self) -> u8 {
        3  // Low cost - simple regex matching
    }

    fn signal_strength(&self) -> u8 {
        6  // Medium-high signal - locale checks can be legitimate i18n
    }

    fn prerequisites(&self) -> Vec<&'static str> {
        vec!["glassware", "encrypted_payload"]  // Run after Tier 2
    }

    fn should_short_circuit(&self, findings: &[Finding]) -> bool {
        // Don't run Tier 3 if nothing found by Tier 1-2
        findings.is_empty()
    }

    fn detect(&self, ir: &FileIR) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Collect lines into a vector for single-pass sliding window
        let lines: Vec<&str> = ir.content().lines().collect();

        // Track locale checks and their line numbers
        let mut locale_check_lines: Vec<usize> = Vec::new();

        // Single pass: check for both locale patterns and exit patterns with sliding window
        for (line_num, line) in lines.iter().enumerate() {
            let mut is_locale_check = false;

            // Check for locale patterns in current line
            for pattern in LOCALE_PATTERNS.iter() {
                if pattern.is_match(line) {
                    locale_check_lines.push(line_num);
                    is_locale_check = true;
                    // Don't emit finding yet - wait to see if there's an exit pattern
                }
            }

            // Check for exit patterns in current line
            if EXIT_PATTERN.is_match(line) {
                // Check if this exit is within 5 lines AFTER a locale check (backward lookup)
                for &check_line in &locale_check_lines {
                    if line_num > check_line && line_num <= check_line + 5 {
                        // Found locale check + exit pattern = CRITICAL
                        findings.push(
                            Finding::new(
                                &ir.metadata.path,
                                check_line + 1,
                                1,
                                0,
                                '\0',
                                DetectionCategory::LocaleGeofencing,
                                Severity::Critical,
                                "Active geofencing: locale check followed by early exit",
                                "CRITICAL: This is active geographic targeting. The package exits early on Russian systems to avoid domestic prosecution. Immediate investigation required.",
                            )
                            .with_cwe_id("CWE-506")
                            .with_reference(
                                "https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode",
                            ),
                        );
                    }
                }
            }

            // Sliding window forward: ONLY check if current line is a locale check
            if is_locale_check {
                for offset in 1..=5 {
                    let forward_line_num = line_num + offset;
                    if forward_line_num < lines.len() {
                        let forward_line = lines[forward_line_num];
                        if EXIT_PATTERN.is_match(forward_line) {
                            // Found exit pattern within 5 lines ahead - emit CRITICAL finding
                            findings.push(
                                Finding::new(
                                    &ir.metadata.path,
                                    line_num + 1,
                                    1,
                                    0,
                                    '\0',
                                    DetectionCategory::LocaleGeofencing,
                                    Severity::Critical,
                                    "Active geofencing: locale check followed by early exit",
                                    "CRITICAL: This is active geographic targeting. The package exits early on Russian systems to avoid domestic prosecution. Immediate investigation required.",
                                )
                                .with_cwe_id("CWE-506")
                                .with_reference(
                                    "https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode",
                                ),
                            );
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
            description: "Detects geographic targeting behavior with Russian locale checks and early exit patterns".to_string(),
        }
    }
}

impl LocaleGeofencingDetector {
    /// Backward compatibility method for tests
    pub fn scan(&self, path: &Path, content: &str, _config: &crate::config::UnicodeConfig) -> Vec<Finding> {
        // Build IR and call detect (for backward compatibility)
        let ir = FileIR::build(path, content);
        self.detect(&ir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::UnicodeConfig;

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
        assert_eq!(findings[0].severity, Severity::Critical); // Correctly upgraded for locale+exit pattern
    }

    #[test]
    fn test_detect_timezone_check_with_exit() {
        let detector = LocaleGeofencingDetector::new();
        let content = r#"
            const timezone = Intl.DateTimeFormat().resolvedOptions().timeZone;
            if (timezone === 'Europe/Moscow') {
                process.exit(0);
            }
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(!findings.is_empty());
        assert_eq!(findings[0].category, DetectionCategory::LocaleGeofencing);
        assert_eq!(findings[0].severity, Severity::Critical);  // Both conditions met
    }

    #[test]
    fn test_no_detect_legitimate_i18n() {
        let detector = LocaleGeofencingDetector::new();
        let content = r#"
            // Legitimate i18n library
            import { useTranslation } from 'react-i18next';
            const { t, i18n } = useTranslation();
            i18n.changeLanguage('ru-RU');
            
            // No exit pattern - just normal i18n usage
            console.log('Language changed');
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        // Should NOT detect - no exit pattern following locale check
        assert!(findings.is_empty());
    }

    #[test]
    fn test_no_detect_clean_i18n_file() {
        let detector = LocaleGeofencingDetector::new();
        let content = r#"
            // Clean i18n file with no Russian locale checks
            import i18n from 'i18next';
            i18n.init({ lng: 'en' });
            export default i18n;
        "#;

        let findings = detector.scan(Path::new("test.js"), content, &UnicodeConfig::default());
        assert!(findings.is_empty());
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
