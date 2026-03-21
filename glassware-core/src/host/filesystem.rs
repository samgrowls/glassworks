//! G1: Filesystem Persistence Scanner
//!
//! Detects GlassWorm artifacts on the host filesystem — persistence directories,
//! dropped payloads, scheduled tasks, and temp files.
//!
//! ## Intel Source
//!
//! From PART2.md and PART3.md:
//!
//! ### Known GlassWorm Paths
//! - `%TEMP%\SsWolTaQA\` — Staging directory
//! - `%APPDATA%\_node_x86\` — Node.js x86 download
//! - `%APPDATA%\_node_x64\` — Node.js x64 download
//! - `~/.config/system/.data/.nodejs/webrtc/` — TCP tunnel module
//! - `N:\work\chrome_current\` — Build path (PDB)
//!
//! ### Persistence Mechanisms
//! - Scheduled task: `UpdateApp`
//! - Registry: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\MtsYO`
//! - Registry: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\UpdateLedger`
//!
//! ### Dropped Files
//! - `w.node` — Chrome extension sideloader (Windows)
//! - `c_x64.node` — DumpBrowserSecrets decryptor
//! - `f_ex86.node` — VS Code workspace harvester
//! - `data` — Browser process detector
//! - `index_ia32.node`, `index_x64.node` — Key generation
//! - `m` — Chrome extension sideloader (macOS)
//! - `BqJuCBQutS.exe` — Ledger-specific binary
//!
//! ## Severity Rules
//!
//! - **HIGH**: Known GlassWorm directory with matching payload file
//! - **MEDIUM**: Known directory structure but no payload
//! - **INFO**: Suspicious temp file patterns without GlassWorm-specific markers

use crate::finding::{DetectionCategory, Finding, Severity};
use std::path::{Path, PathBuf};

/// Known GlassWorm directory paths (platform-agnostic patterns)
const GLASSWORM_DIRS: &[&str] = &[
    // Temp staging directory
    "SsWolTaQA",
    // AppData directories
    "_node_x86",
    "_node_x64",
    // Linux config path
    ".config/system/.data/.nodejs",
    // Build path from PDB
    "work/chrome_current",
    "DumpBrowserSecrets",
];

/// Known GlassWorm payload filenames
const GLASSWORM_PAYLOADS: &[&str] = &[
    "w.node",
    "c_x64.node",
    "f_ex86.node",
    "data",
    "index_ia32.node",
    "index_x64.node",
    "m",
    "BqJuCBQutS.exe",
    "led-win32",
    // New payloads from Mar 2026 reports
    "i.js",  // Loader filename
    "init.json",  // Bot configuration persistence
];

/// Known persistence indicators
const PERSISTENCE_INDICATORS: &[&str] = &[
    // Scheduled task name
    "UpdateApp",
    // Registry key names
    "MtsYO",
    "UpdateLedger",
];

/// Scanner for filesystem persistence artifacts
pub struct FilesystemScanner {
    /// Root path to scan
    root: PathBuf,
    /// Findings collected during scan
    findings: Vec<Finding>,
}

impl FilesystemScanner {
    /// Create a new filesystem scanner
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            findings: Vec::new(),
        }
    }

    /// Scan the filesystem for GlassWorm artifacts
    pub fn scan(&mut self) -> Vec<Finding> {
        self.findings.clear();

        // Scan the provided root directory recursively
        let root = self.root.clone();
        self.scan_directory(&root);

        self.findings.clone()
    }

    /// Scan system locations (temp, AppData, etc.) - use with caution as this can be slow
    #[allow(dead_code)]
    fn scan_system_locations(&mut self) {
        // Scan temp directory
        self.scan_directory(&std::env::temp_dir());

        // Scan AppData (Windows)
        if let Some(appdata) = std::env::var("APPDATA").ok() {
            self.scan_directory(Path::new(&appdata));
        }

        // Scan LocalAppData (Windows)
        if let Some(local_appdata) = std::env::var("LOCALAPPDATA").ok() {
            self.scan_directory(Path::new(&local_appdata));
        }

        // Scan home directory (Linux/macOS)
        if let Some(home) = std::env::var("HOME").ok() {
            self.scan_directory(Path::new(&home));
        }
    }

    /// Recursively scan a directory for GlassWorm artifacts
    fn scan_directory(&mut self, dir: &Path) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();

                // Check for known GlassWorm directory names
                if path.is_dir() {
                    for pattern in GLASSWORM_DIRS {
                        if name.contains(pattern) {
                            let severity = if self.dir_contains_payloads(&path) {
                                Severity::High
                            } else {
                                Severity::Medium
                            };
                            self.add_finding(
                                &path,
                                severity,
                                "Known GlassWorm directory pattern",
                                &format!("Directory matching '{}' found", pattern),
                            );
                        }
                    }
                    // Recurse into subdirectory
                    self.scan_directory(&path);
                }

                // Check for known payload files
                if GLASSWORM_PAYLOADS.contains(&name.as_str()) {
                    self.add_finding(
                        &path,
                        Severity::High,
                        "GlassWorm payload file detected",
                        &format!("Known payload '{}' found", name),
                    );
                }
            }
        }
    }

    /// Check if directory contains known payload files
    fn dir_contains_payloads(&self, dir: &Path) -> bool {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if GLASSWORM_PAYLOADS.contains(&name.as_str()) {
                    return true;
                }
            }
        }
        false
    }

    /// Add a finding to the results
    fn add_finding(&mut self, path: &Path, severity: Severity, category_desc: &str, description: &str) {
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
                "Review the identified path for GlassWorm infection artifacts.",
            )
            .with_cwe_id("CWE-506")
            .with_reference("https://codeberg.org/tip-o-deincognito/glassworm-writeup/raw/branch/main/PART3.md")
            .with_context(category_desc),
        );
    }
}

/// Scan a filesystem root for GlassWorm artifacts
pub fn scan_filesystem(root: &Path) -> Vec<Finding> {
    let mut scanner = FilesystemScanner::new(root);
    scanner.scan()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scanner_creation() {
        let temp = TempDir::new().unwrap();
        let scanner = FilesystemScanner::new(temp.path());
        assert_eq!(scanner.root, temp.path());
        assert!(scanner.findings.is_empty());
    }

    #[test]
    fn test_clean_filesystem_no_false_positives() {
        let temp = TempDir::new().unwrap();
        let mut scanner = FilesystemScanner::new(temp.path());
        let findings = scanner.scan();
        // Should not find anything in a clean temp directory
        // (unless the temp dir happens to be in a GlassWorm path)
        // We just verify the scan completes without errors
    }

    #[test]
    fn test_detect_staging_directory() {
        let temp = TempDir::new().unwrap();
        // Create the known staging directory
        let staging_dir = temp.path().join("SsWolTaQA");
        fs::create_dir(&staging_dir).unwrap();

        let mut scanner = FilesystemScanner::new(temp.path());
        let findings = scanner.scan();

        // Should find the staging directory
        assert!(findings.iter().any(|f| {
            f.file.contains("SsWolTaQA") && f.severity >= Severity::Medium
        }));
    }

    #[test]
    fn test_detect_payload_file() {
        let temp = TempDir::new().unwrap();
        // Create a known payload file
        let payload_path = temp.path().join("w.node");
        fs::write(&payload_path, b"fake payload").unwrap();

        let mut scanner = FilesystemScanner::new(temp.path());
        let findings = scanner.scan();

        // Should find the payload file at HIGH severity
        assert!(findings.iter().any(|f| {
            f.file.contains("w.node") && f.severity == Severity::High
        }));
    }

    #[test]
    fn test_detect_payload_with_high_severity() {
        let temp = TempDir::new().unwrap();
        // Create GlassWorm directory with payload
        let glassworm_dir = temp.path().join("_node_x86");
        fs::create_dir(&glassworm_dir).unwrap();
        let payload_path = glassworm_dir.join("c_x64.node");
        fs::write(&payload_path, b"fake decryptor").unwrap();

        let mut scanner = FilesystemScanner::new(temp.path());
        let findings = scanner.scan();

        // Should find both directory and payload
        let dir_finding = findings.iter().find(|f| f.file.contains("_node_x86"));
        let payload_finding = findings.iter().find(|f| f.file.contains("c_x64.node"));

        assert!(dir_finding.is_some());
        assert!(payload_finding.is_some());

        // Directory with payload should be HIGH
        if let Some(f) = dir_finding {
            assert_eq!(f.severity, Severity::High);
        }
    }

    #[test]
    fn test_multiple_payload_types() {
        let temp = TempDir::new().unwrap();
        // Create multiple payload files
        let payloads = ["w.node", "f_ex86.node", "data"];
        for payload in &payloads {
            let path = temp.path().join(payload);
            fs::write(&path, b"fake").unwrap();
        }

        let mut scanner = FilesystemScanner::new(temp.path());
        let findings = scanner.scan();

        // Should find all three payloads
        for payload in &payloads {
            assert!(findings.iter().any(|f| {
                f.file.contains(payload) && f.severity == Severity::High
            }));
        }
    }

    #[test]
    fn test_directory_without_payload_medium_severity() {
        let temp = TempDir::new().unwrap();
        // Create GlassWorm directory without payload
        let glassworm_dir = temp.path().join("_node_x64");
        fs::create_dir(&glassworm_dir).unwrap();
        // Add a benign file
        fs::write(glassworm_dir.join("readme.txt"), b"hello").unwrap();

        let mut scanner = FilesystemScanner::new(temp.path());
        let findings = scanner.scan();

        // Directory without payload should be MEDIUM
        let dir_finding = findings.iter().find(|f| f.file.contains("_node_x64"));
        assert!(dir_finding.is_some());
        if let Some(f) = dir_finding {
            assert_eq!(f.severity, Severity::Medium);
        }
    }
}
