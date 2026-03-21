//! G2: Chrome Preference Tampering Detector
//!
//! Detects Chrome Preferences / Secure Preferences JSON modifications
//! consistent with GlassWorm extension sideloading.
//!
//! ## Intel Source
//!
//! From PART2.md and PART3.md:
//!
//! ### Extension Indicators
//! - Name: "Google Docs Offline" (spoofed)
//! - Version: 1.95.1 (legitimate is 1.85.1)
//! - Sideloaded via w.node (Windows) or m (macOS)
//!
//! ### Preferences Keys
//! - `extensions.<id>.from_webstore` — false for sideloaded
//! - `extensions.<id>.location` — 4 for sideloaded
//! - `extensions.<id>.creation_flags` — 38 for suspicious combo
//! - `browser.enabled_labs_experiments` — may be modified
//!
//! ## Severity Rules (CRITICAL)
//!
//! **`from_webstore: false` ALONE = INFO**
//!
//! Many legitimate extensions (corporate tools, dev extensions, local unpacked
//! extensions) have `from_webstore: false`. This is a WEAK signal by itself.
//!
//! Escalation requires ADDITIONAL corroborating signals:
//! - `from_webstore: false` + known malicious extension pattern → CRITICAL
//! - `from_webstore: false` + suspicious permission combo → MEDIUM
//! - `from_webstore: false` + location=4 + creation_flags=38 → CRITICAL
//! - `from_webstore: false` alone, no other signals → INFO
//!
//! ## Additive Scoring
//!
//! Multiple weak signals combine to raise severity. Single weak signal stays low.

use crate::finding::{DetectionCategory, Finding, Severity};
use std::collections::HashMap;
use std::path::Path;

#[cfg(feature = "serde")]
use serde_json::Value;

/// Known malicious extension patterns (name/version combos from intel)
const MALICIOUS_EXTENSION_PATTERNS: &[(&str, &str)] = &[
    // Spoofed Google Docs Offline
    ("Google Docs Offline", "1.95.1"),
];

/// Suspicious permission combinations
const SUSPICIOUS_PERMISSIONS: &[&str] = &[
    "nativeMessaging",
    "clipboardRead",
    "cookies",
    "declarativeNetRequest",
];

/// Chrome profile paths (platform-specific)
const CHROME_PROFILES: &[&str] = &[
    // Windows
    "%APPDATA%/Google/Chrome/User Data",
    "%LOCALAPPDATA%/Google/Chrome/User Data",
    // macOS
    "~/Library/Application Support/Google/Chrome",
    // Linux
    "~/.config/google-chrome",
];

/// Scanner for Chrome preference tampering
pub struct ChromePrefsScanner {
    /// Findings collected during scan
    findings: Vec<Finding>,
}

impl ChromePrefsScanner {
    /// Create a new Chrome prefs scanner
    pub fn new() -> Self {
        Self {
            findings: Vec::new(),
        }
    }

    /// Scan a Chrome profile directory for tampering
    pub fn scan_profile(&mut self, profile_path: &Path) -> Vec<Finding> {
        self.findings.clear();

        // Check for Preferences file
        let prefs_path = profile_path.join("Preferences");
        if prefs_path.exists() {
            self.scan_preferences_file(&prefs_path);
        }

        // Check for Secure Preferences file
        let secure_prefs_path = profile_path.join("Secure Preferences");
        if secure_prefs_path.exists() {
            self.scan_secure_preferences(&secure_prefs_path);
        }

        self.findings.clone()
    }

    /// Scan Preferences JSON file
    #[cfg(feature = "serde")]
    fn scan_preferences_file(&mut self, path: &Path) {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };

        let prefs: Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => return,
        };

        // Check extensions
        if let Some(extensions) = prefs.get("extensions").and_then(|e| e.as_object()) {
            self.scan_extensions(extensions, path);
        }

        // Check browser.enabled_labs_experiments
        if let Some(labs) = prefs.get("browser").and_then(|b| b.get("enabled_labs_experiments")) {
            if let Some(labs_array) = labs.as_array() {
                if !labs_array.is_empty() {
                    self.add_finding(
                        path,
                        Severity::Info,
                        "Chrome Labs experiments enabled",
                        "Review enabled experiments for suspicious modifications",
                    );
                }
            }
        }
    }

    /// Scan extensions object for suspicious patterns
    #[cfg(feature = "serde")]
    fn scan_extensions(&mut self, extensions: &serde_json::Map<String, Value>, prefs_path: &Path) {
        for (ext_id, ext_data) in extensions {
            if let Some(ext_obj) = ext_data.as_object() {
                let mut signals = Vec::new();
                let mut severity = Severity::Info;

                // Check from_webstore
                let from_webstore = ext_obj
                    .get("from_webstore")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);

                if !from_webstore {
                    signals.push("from_webstore:false");
                    // Alone = INFO (many legitimate reasons)
                }

                // Check location (4 = sideloaded)
                let location = ext_obj
                    .get("location")
                    .and_then(|v| v.as_u64());

                if location == Some(4) {
                    signals.push("location:4(sideloaded)");
                    if !from_webstore {
                        severity = Severity::Medium;
                    }
                }

                // Check creation_flags (38 = suspicious combo)
                let creation_flags = ext_obj
                    .get("creation_flags")
                    .and_then(|v| v.as_u64());

                if creation_flags == Some(38) {
                    signals.push("creation_flags:38");
                    if !from_webstore && location == Some(4) {
                        // All three = GlassWorm signature
                        severity = Severity::Critical;
                    } else if !from_webstore {
                        severity = Severity::Medium;
                    }
                }

                // Check extension name/version for known malicious patterns
                let name = ext_obj.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let version = ext_obj.get("version").and_then(|v| v.as_str()).unwrap_or("");

                for (mal_name, mal_version) in MALICIOUS_EXTENSION_PATTERNS {
                    if name == *mal_name && version == *mal_version {
                        signals.push("known_malicious_pattern");
                        severity = Severity::Critical;
                    }
                }

                // Check permissions
                if let Some(permissions) = ext_obj.get("permissions").and_then(|v| v.as_array()) {
                    let suspicious_count = permissions.iter().filter(|p| {
                        p.as_str().map(|s| {
                            SUSPICIOUS_PERMISSIONS.iter().any(|sp| s.contains(sp))
                        }).unwrap_or(false)
                    }).count();

                    if suspicious_count >= 3 {
                        signals.push("suspicious_permissions");
                        if severity < Severity::Medium {
                            severity = Severity::Medium;
                        }
                    }
                }

                // Check content_verification (missing/tampered = suspicious)
                let has_verification = ext_obj.get("content_verification").is_some();
                if !from_webstore && !has_verification {
                    signals.push("no_content_verification");
                    // Just adds to the signal count, doesn't escalate alone
                }

                // Report finding if any signals detected
                if !signals.is_empty() {
                    self.add_finding(
                        prefs_path,
                        severity,
                        &format!("Extension {} analysis", ext_id),
                        &format!("Signals: {}", signals.join(", ")),
                    );
                }
            }
        }
    }

    /// Scan Secure Preferences file (for tampering detection)
    fn scan_secure_preferences(&mut self, path: &Path) {
        // Secure Preferences is binary/encrypted, but we can check if it exists
        // and if its size is suspicious (modified)
        if let Ok(metadata) = std::fs::metadata(path) {
            let size = metadata.len();
            // Typical Secure Preferences files are 2-10KB
            // Significantly larger or smaller could indicate tampering
            if size < 1024 || size > 100_000 {
                self.add_finding(
                    path,
                    Severity::Info,
                    "Secure Preferences size anomaly",
                    &format!("File size: {} bytes (unusual)", size),
                );
            }
        }
    }

    /// Add a finding to the results
    fn add_finding(&mut self, path: &Path, severity: Severity, category: &str, description: &str) {
        self.findings.push(
            Finding::new(
                &path.to_string_lossy(),
                1,
                1,
                0,
                '\0',
                DetectionCategory::Unknown,
                severity,
                description,
                "Review the extension configuration for signs of tampering or sideloading.",
            )
            .with_cwe_id("CWE-506")
            .with_reference("https://codeberg.org/tip-o-deincognito/glassworm-writeup/raw/branch/main/PART3.md")
            .with_context(category),
        );
    }

    /// Scan preferences JSON content directly (for testing or when file is already loaded)
    #[cfg(feature = "serde")]
    pub fn scan_preferences_content(&mut self, content: &str, path: &Path) -> Vec<Finding> {
        self.findings.clear();

        let prefs: Value = match serde_json::from_str(content) {
            Ok(v) => v,
            Err(_) => return self.findings.clone(),
        };

        if let Some(extensions) = prefs.get("extensions").and_then(|e| e.as_object()) {
            self.scan_extensions(extensions, path);
        }

        self.findings.clone()
    }
}

#[cfg(not(feature = "serde"))]
impl ChromePrefsScanner {
    /// Stub implementation when serde feature is disabled
    pub fn new() -> Self {
        Self {
            findings: Vec::new(),
        }
    }

    pub fn scan_profile(&mut self, _profile_path: &Path) -> Vec<Finding> {
        self.findings.clear();
        self.findings.clone()
    }

    fn add_finding(&mut self, path: &Path, severity: Severity, category: &str, description: &str) {
        self.findings.push(
            Finding::new(
                &path.to_string_lossy(),
                1,
                1,
                0,
                '\0',
                DetectionCategory::Unknown,
                severity,
                description,
                "Review the extension configuration for signs of tampering or sideloading.",
            )
            .with_cwe_id("CWE-506")
            .with_reference("https://codeberg.org/tip-o-deincognito/glassworm-writeup/raw/branch/main/PART3.md")
            .with_context(category),
        );
    }

    pub fn scan_preferences_content(&mut self, _content: &str, _path: &Path) -> Vec<Finding> {
        self.findings.clear();
        self.findings.clone()
    }
}

/// Scan a Chrome profile directory for tampering
pub fn scan_chrome_profile(profile_path: &Path) -> Vec<Finding> {
    let mut scanner = ChromePrefsScanner::new();
    scanner.scan_profile(profile_path)
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scanner_creation() {
        let scanner = ChromePrefsScanner::new();
        assert!(scanner.findings.is_empty());
    }

    #[test]
    fn test_from_webstore_false_alone_is_info() {
        let prefs_content = r#"{
            "extensions": {
                "abcdefghijklmnop": {
                    "name": "Some Extension",
                    "version": "1.0.0",
                    "from_webstore": false
                }
            }
        }"#;

        let mut scanner = ChromePrefsScanner::new();
        let findings = scanner.scan_preferences_content(prefs_content, Path::new("test/Preferences"));

        // Should be INFO only - from_webstore:false alone is weak signal
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::Info);
    }

    #[test]
    fn test_from_webstore_false_with_location_4_is_medium() {
        let prefs_content = r#"{
            "extensions": {
                "abcdefghijklmnop": {
                    "name": "Some Extension",
                    "version": "1.0.0",
                    "from_webstore": false,
                    "location": 4
                }
            }
        }"#;

        let mut scanner = ChromePrefsScanner::new();
        let findings = scanner.scan_preferences_content(prefs_content, Path::new("test/Preferences"));

        // Should be MEDIUM - two signals combined
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::Medium);
    }

    #[test]
    fn test_glassworm_signature_is_critical() {
        // All three: from_webstore:false + location:4 + creation_flags:38
        let prefs_content = r#"{
            "extensions": {
                "abcdefghijklmnop": {
                    "name": "Some Extension",
                    "version": "1.0.0",
                    "from_webstore": false,
                    "location": 4,
                    "creation_flags": 38
                }
            }
        }"#;

        let mut scanner = ChromePrefsScanner::new();
        let findings = scanner.scan_preferences_content(prefs_content, Path::new("test/Preferences"));

        // Should be CRITICAL - GlassWorm signature
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::Critical);
    }

    #[test]
    fn test_known_malicious_pattern_is_critical() {
        let prefs_content = r#"{
            "extensions": {
                "abcdefghijklmnop": {
                    "name": "Google Docs Offline",
                    "version": "1.95.1",
                    "from_webstore": true
                }
            }
        }"#;

        let mut scanner = ChromePrefsScanner::new();
        let findings = scanner.scan_preferences_content(prefs_content, Path::new("test/Preferences"));

        // Should be CRITICAL - known malicious pattern
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::Critical);
    }

    #[test]
    fn test_suspicious_permissions_escalates_severity() {
        let prefs_content = r#"{
            "extensions": {
                "abcdefghijklmnop": {
                    "name": "Some Extension",
                    "version": "1.0.0",
                    "from_webstore": false,
                    "permissions": [
                        "nativeMessaging",
                        "clipboardRead",
                        "cookies",
                        "declarativeNetRequest",
                        "tabs"
                    ]
                }
            }
        }"#;

        let mut scanner = ChromePrefsScanner::new();
        let findings = scanner.scan_preferences_content(prefs_content, Path::new("test/Preferences"));

        // Should be at least MEDIUM - suspicious permissions + from_webstore:false
        assert!(!findings.is_empty());
        assert!(findings[0].severity >= Severity::Medium);
    }

    #[test]
    fn test_clean_profile_no_findings() {
        let prefs_content = r#"{
            "extensions": {
                "abcdefghijklmnop": {
                    "name": "Legitimate Extension",
                    "version": "1.0.0",
                    "from_webstore": true,
                    "location": 1
                }
            }
        }"#;

        let mut scanner = ChromePrefsScanner::new();
        let findings = scanner.scan_preferences_content(prefs_content, Path::new("test/Preferences"));

        // Should have no findings for clean extension
        assert!(findings.is_empty());
    }

    #[test]
    fn test_scan_profile_with_mock_file() {
        let temp = TempDir::new().unwrap();
        let prefs_path = temp.path().join("Preferences");
        
        let prefs_content = r#"{
            "extensions": {
                "abcdefghijklmnop": {
                    "name": "Google Docs Offline",
                    "version": "1.95.1",
                    "from_webstore": false,
                    "location": 4,
                    "creation_flags": 38
                }
            }
        }"#;
        
        fs::write(&prefs_path, prefs_content).unwrap();

        let findings = scan_chrome_profile(temp.path());

        // Should find the malicious extension pattern
        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.severity == Severity::Critical));
    }
}
