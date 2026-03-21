//! Browser-Kill Command Detector (E3)
//!
//! Detects commands that kill browser processes, a GlassWorm signature
//! used to terminate Chrome before injecting malicious extensions.
//!
//! ## Detection Logic
//!
//! This detector emits findings when:
//! 1. Windows taskkill commands targeting browsers
//! 2. Unix pkill/killall commands targeting browsers
//! 3. PowerShell Stop-Process commands
//! 4. Force kill flags (/F, -9, -Force)
//!
//! ## Severity
//!
//! High when combined with extension installation patterns
//! Medium when standalone (could be legitimate admin scripts)
//!
//! ## GlassWorm Context
//!
//! From Part 5: GlassWorm uses these commands to kill Chrome before
//! sideloading malicious extensions, bypassing the "Chrome is running"
//! check that would prevent extension installation.

use crate::detector::{Detector, DetectorTier};
use crate::finding::{DetectionCategory, Finding, Severity};
use crate::ir::FileIR;
use std::path::Path;

/// Browser-kill command patterns (E3 enhancement)
const BROWSER_KILL_PATTERNS: &[&str] = &[
    // Windows taskkill
    "taskkill /F /IM chrome.exe",
    "taskkill /F /IM msedge.exe",
    "taskkill /F /IM brave.exe",
    "taskkill /F /IM opera.exe",
    "taskkill /F /IM vivaldi.exe",
    "taskkill /F /IM firefox.exe",
    "taskkill /T /F /IM chrome",  // /T = tree (kill children)
    
    // Unix pkill/killall
    "pkill -9 -f \"Google Chrome\"",
    "pkill -9 -f \"Microsoft Edge\"",
    "pkill -9 chrome",
    "pkill -9 firefox",
    "killall -9 chrome",
    "killall -9 firefox",
    
    // PowerShell
    "Stop-Process -Name chrome -Force",
    "Stop-Process -Name msedge -Force",
    "Stop-Process \"chrome\" -Force",
];

/// Detector for browser-kill commands
pub struct BrowserKillDetector;

impl BrowserKillDetector {
    /// Create a new browser-kill detector
    pub fn new() -> Self {
        Self
    }
}

impl Default for BrowserKillDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for BrowserKillDetector {
    fn name(&self) -> &str {
        "browser_kill"
    }

    fn tier(&self) -> DetectorTier {
        DetectorTier::Tier3Behavioral
    }

    fn cost(&self) -> u8 {
        2  // Low cost - simple string matching
    }

    fn signal_strength(&self) -> u8 {
        8  // High signal - specific to GlassWorm TTP
    }

    fn detect(&self, ir: &FileIR) -> Vec<Finding> {
        let mut findings = Vec::new();
        let content = ir.content();

        for (line_num, original_line) in content.lines().enumerate() {
            let line = original_line.to_lowercase();
            
            // Check for exact browser-kill patterns
            for pattern in BROWSER_KILL_PATTERNS {
                let pattern_lower = pattern.to_lowercase();
                
                if line.contains(&pattern_lower) {
                    findings.push(self.create_finding(
                        Path::new(&ir.metadata.path),
                        line_num + 1,
                        original_line,
                        pattern,
                    ));
                }
            }
        }

        findings
    }
}

impl BrowserKillDetector {
    fn create_finding(
        &self,
        path: &Path,
        line: usize,
        original_line: &str,
        matched_pattern: &str,
    ) -> Finding {
        // Determine severity based on force flags
        let line_lower = original_line.to_lowercase();
        let severity = if line_lower.contains("/f") || 
                       line_lower.contains("-9") || 
                       line_lower.contains("-force") ||
                       line_lower.contains("/t") {
            Severity::High  // Force kill = high confidence
        } else {
            Severity::Medium  // Non-force could be legitimate
        };

        // Extract browser name from matched pattern
        let browser_name = if original_line.to_lowercase().contains("chrome") {
            "Chrome"
        } else if original_line.to_lowercase().contains("edge") || original_line.to_lowercase().contains("msedge") {
            "Edge"
        } else if original_line.to_lowercase().contains("brave") {
            "Brave"
        } else if original_line.to_lowercase().contains("firefox") {
            "Firefox"
        } else if original_line.to_lowercase().contains("opera") {
            "Opera"
        } else if original_line.to_lowercase().contains("vivaldi") {
            "Vivaldi"
        } else {
            "browser"
        };

        Finding::new(
            &path.to_string_lossy(),
            line,
            1,
            0,
            '\0',
            DetectionCategory::Unknown,  // Using Unknown since no Behavioral category
            severity,
            &format!("Browser-kill command detected (targeting {})", browser_name),
            "GlassWorm kills browser processes before sideloading malicious extensions. Review for malicious extension installation patterns.",
        )
        .with_cwe_id("CWE-693")  // Protection Mechanism Failure
        .with_reference("https://github.com/samgrowls/glassworks/blob/main/glassworm-writeup/PART5.md")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::UnicodeConfig;

    #[test]
    fn test_detect_windows_taskkill() {
        let detector = BrowserKillDetector::new();
        let content = r#"
            // Kill Chrome before extension install
            exec("taskkill /F /IM chrome.exe /T");
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::High);
        assert!(findings[0].description.contains("Browser-kill"));
    }

    #[test]
    fn test_detect_unix_pkill() {
        let detector = BrowserKillDetector::new();
        let content = r#"
            const { exec } = require('child_process');
            exec('pkill -9 -f "Google Chrome"');
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::High);
    }

    #[test]
    fn test_detect_powershell() {
        let detector = BrowserKillDetector::new();
        let content = r#"
            Stop-Process -Name chrome -Force
        "#;

        let ir = FileIR::build(Path::new("test.ps1"), content);
        let findings = detector.detect(&ir);
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::High);
    }

    #[test]
    fn test_no_detect_legitimate_process_management() {
        let detector = BrowserKillDetector::new();
        let content = r#"
            // Legitimate process management
            function cleanup() {
                // Close our own app
                process.exit(0);
            }
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_no_detect_non_browser_kill() {
        let detector = BrowserKillDetector::new();
        let content = r#"
            // Kill node process (not a browser)
            exec("taskkill /F /IM node.exe");
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_detect_multiple_browsers() {
        let detector = BrowserKillDetector::new();
        let content = r#"
            // Kill all browsers
            exec("taskkill /F /IM chrome.exe");
            exec("taskkill /F /IM msedge.exe");
            exec("pkill -9 firefox");
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);
        assert_eq!(findings.len(), 3);
    }
}
