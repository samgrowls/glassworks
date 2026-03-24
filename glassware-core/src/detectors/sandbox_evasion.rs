//! Sandbox Evasion Detector (GlassWorm-specific)
//!
//! Detects GlassWorm-specific sandbox evasion techniques including
//! CI environment detection and VM characteristic checks.
//!
//! ## Detection Logic
//!
//! This detector emits findings when:
//! 1. CI + VM detection combination (CRITICAL - GlassWorm signature)
//! 2. CPU count checks (< 2 CPUs)
//! 3. Memory checks (< 2GB RAM)
//! 4. Silent exit (process.exit(0)) when sandbox detected
//!
//! ## GlassWorm Evasion Strategy
//!
//! GlassWorm employs multi-layer evasion:
//! 1. CI Detection: Checks for CI/CD environments (GitHub Actions, Travis, etc.)
//! 2. VM Detection: Checks for low-resource VMs used in automated analysis
//! 3. Conditional Execution: Only runs payload in production environments
//! 4. Silent Exit: Exits with code 0 when sandbox detected (no alert)
//!
//! ## Severity
//!
//! Critical: CI + VM detection combination
//! Critical: Silent exit with sandbox detection
//! High: CPU count check for sandbox detection
//! High: Memory check for sandbox detection

use crate::detector::{Detector, DetectorMetadata, DetectorTier};
use crate::finding::{DetectionCategory, Finding, Severity};
use crate::ir::FileIR;
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

/// Patterns for CI environment detection
const CI_PATTERNS: &[&str] = &[
    "process.env.CI",
    "process.env.GITHUB_ACTIONS",
    "process.env.TRAVIS",
    "process.env.JENKINS",
    "process.env.CIRCLECI",
    "process.env.GITLAB_CI",
    "process.env.BUILDKITE",
    "process.env.TEAMCITY",
    "env.CI",
    "env.GITHUB_ACTIONS",
    "isCI",
    "is_ci",
];

/// Patterns for VM/sandbox detection
const VM_PATTERNS: &[&str] = &[
    "os.cpus()",
    "os.totalmem()",
    "require('os').cpus()",
    "require('os').totalmem()",
    "cpus().length",
    "totalmem()",
    "os.cpus().length",
    "os.freemem()",
    "require('os').freemem()",
];

/// Patterns for silent exit
const SILENT_EXIT_PATTERNS: &[&str] = &[
    "process.exit(0)",
    "process.exit(0x0)",
    "process.exit(0x00)",
    "exit(0)",
];

/// Compiled regex patterns
static CI_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"process\.env\.(CI|GITHUB_ACTIONS|TRAVIS|JENKINS|CIRCLECI|GITLAB_CI|BUILDKITE|TEAMCITY)|\bisCI\b|\bis_ci\b").unwrap()
});

static VM_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"os\.(cpus|totalmem|freemem)\s*\(\s*\)|require\s*\(\s*["']os["']\s*\)\s*\.\s*(cpus|totalmem|freemem)\s*\(\s*\)|cpus\s*\(\s*\)\s*\.\s*length|totalmem\s*\(\s*\)|freemem\s*\(\s*\)"#).unwrap()
});

static EXIT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"process\.exit\s*\(\s*0\s*\)|process\.exit\s*\(\s*0x0+\s*\)|exit\s*\(\s*0\s*\)").unwrap()
});

/// Detector for GlassWorm sandbox evasion
pub struct SandboxEvasionDetector;

impl SandboxEvasionDetector {
    /// Create a new sandbox evasion detector
    pub fn new() -> Self {
        Self
    }
}

impl Default for SandboxEvasionDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for SandboxEvasionDetector {
    fn name(&self) -> &str {
        "sandbox_evasion"
    }

    fn tier(&self) -> DetectorTier {
        DetectorTier::Tier3Behavioral
    }

    fn cost(&self) -> u8 {
        3  // Low cost - string matching
    }

    fn signal_strength(&self) -> u8 {
        9  // High signal - CI+VM combination is strong GlassWorm indicator
    }

    fn prerequisites(&self) -> Vec<&'static str> {
        vec!["invisible_char", "glassware"]  // Run after Tier 1-2
    }

    fn should_run(&self, findings: &[Finding]) -> bool {
        // Don't run Tier 3 if nothing found by Tier 1-2
        !findings.is_empty()
    }

    fn detect(&self, ir: &FileIR) -> Vec<Finding> {
        let mut findings = Vec::new();
        let content = ir.content();
        let path = &ir.metadata.path;

        // Track matches
        let mut ci_check_lines: Vec<usize> = Vec::new();
        let mut vm_check_lines: Vec<usize> = Vec::new();
        let mut exit_lines: Vec<usize> = Vec::new();
        let mut cpu_check_lines: Vec<usize> = Vec::new();
        let mut memory_check_lines: Vec<usize> = Vec::new();

        // First pass: categorize all matches
        for (line_num, line) in content.lines().enumerate() {
            // CI detection
            if CI_REGEX.is_match(line) {
                ci_check_lines.push(line_num);
            }

            // VM detection
            if VM_REGEX.is_match(line) {
                vm_check_lines.push(line_num);

                // Check for specific CPU count check
                if line.contains("cpus()") && (line.contains("< 2") || line.contains("<= 1") || line.contains("<2")) {
                    cpu_check_lines.push(line_num);
                }

                // Check for specific memory check (< 2GB)
                if line.contains("totalmem()") && (line.contains("2 * 1024 * 1024 * 1024") || line.contains("2147483648") || line.contains("2GB")) {
                    memory_check_lines.push(line_num);
                }
            }

            // Silent exit
            if EXIT_REGEX.is_match(line) {
                exit_lines.push(line_num);
            }
        }

        // Second pass: generate findings with context

        // CRITICAL: CI + VM detection combination (GlassWorm signature)
        let has_ci = !ci_check_lines.is_empty();
        let has_vm = !vm_check_lines.is_empty();

        if has_ci && has_vm {
            let first_ci_line = ci_check_lines.first().unwrap_or(&0) + 1;
            let first_vm_line = vm_check_lines.first().unwrap_or(&0) + 1;
            let report_line = first_ci_line.min(first_vm_line);

            findings.push(
                Finding::new(
                    path,
                    report_line,
                    1,
                    0,
                    '\0',
                    DetectionCategory::TimeDelaySandboxEvasion,
                    Severity::Critical,
                    "GlassWorm evasion: CI + VM detection combination",
                    "CRITICAL: This code checks for both CI environments AND VM characteristics. \
                     This is the GlassWorm sandbox evasion signature - it only executes payloads \
                     in production environments while evading automated analysis.",
                )
                .with_cwe_id("CWE-506")
                .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
                .with_confidence(0.92)
                .with_context(&format!(
                    "CI checks: {}, VM checks: {}",
                    ci_check_lines.len(),
                    vm_check_lines.len()
                )),
            );
        }

        // CRITICAL: Silent exit when sandbox detected
        for &exit_line in &exit_lines {
            // Check if exit is near CI/VM check (within 15 lines)
            let near_ci = ci_check_lines.iter().any(|&l| (l as i32 - exit_line as i32).abs() <= 15);
            let near_vm = vm_check_lines.iter().any(|&l| (l as i32 - exit_line as i32).abs() <= 15);

            if near_ci || near_vm {
                findings.push(
                    Finding::new(
                        path,
                        exit_line + 1,
                        1,
                        0,
                        '\0',
                        DetectionCategory::TimeDelaySandboxEvasion,
                        Severity::Critical,
                        "Silent exit when sandbox detected",
                        "CRITICAL: Code exits silently (code 0) when sandbox/CI is detected. \
                         This prevents analysis tools from detecting malicious behavior while \
                         making the exit appear normal.",
                    )
                    .with_cwe_id("CWE-506")
                    .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
                    .with_confidence(0.90),
                );
            }
        }

        // HIGH: CPU count check for sandbox detection
        for &cpu_line in &cpu_check_lines {
            // Only flag if not already part of CI+VM combination
            let already_flagged = has_ci && has_vm;

            if !already_flagged {
                findings.push(
                    Finding::new(
                        path,
                        cpu_line + 1,
                        1,
                        0,
                        '\0',
                        DetectionCategory::TimeDelaySandboxEvasion,
                        Severity::High,
                        "CPU count check for sandbox detection",
                        "Code checks for low CPU count (< 2). Automated sandboxes often have \
                         limited CPU resources. This is a common evasion technique.",
                    )
                    .with_cwe_id("CWE-506")
                    .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
                    .with_confidence(0.82),
                );
            }
        }

        // HIGH: Memory check for sandbox detection
        for &mem_line in &memory_check_lines {
            // Only flag if not already part of CI+VM combination
            let already_flagged = has_ci && has_vm;

            if !already_flagged {
                findings.push(
                    Finding::new(
                        path,
                        mem_line + 1,
                        1,
                        0,
                        '\0',
                        DetectionCategory::TimeDelaySandboxEvasion,
                        Severity::High,
                        "Memory check for sandbox detection",
                        "Code checks for low memory (< 2GB). Automated sandboxes often have \
                         limited RAM. This is a common evasion technique.",
                    )
                    .with_cwe_id("CWE-506")
                    .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
                    .with_confidence(0.82),
                );
            }
        }

        // MEDIUM: CI detection alone (could be legitimate)
        if has_ci && !has_vm {
            let first_line = ci_check_lines.first().unwrap_or(&0) + 1;

            // Check if CI check is combined with conditional execution
            let has_conditional = ci_check_lines.iter().any(|&line| {
                content
                    .lines()
                    .nth(line)
                    .map(|l| l.contains("if") || l.contains("!") || l.contains("||") || l.contains("&&"))
                    .unwrap_or(false)
            });

            if has_conditional {
                findings.push(
                    Finding::new(
                        path,
                        first_line,
                        1,
                        0,
                        '\0',
                        DetectionCategory::TimeDelaySandboxEvasion,
                        Severity::Medium,
                        "CI environment check with conditional execution",
                        "Code conditionally executes based on CI environment. \
                         This could be legitimate CI/CD logic or sandbox evasion. \
                         Review the conditional logic for malicious behavior.",
                    )
                    .with_cwe_id("CWE-506")
                    .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
                    .with_confidence(0.60),
                );
            }
        }

        // MEDIUM: VM detection alone (could be legitimate)
        if has_vm && !has_ci {
            let first_line = vm_check_lines.first().unwrap_or(&0) + 1;

            findings.push(
                Finding::new(
                    path,
                    first_line,
                    1,
                    0,
                    '\0',
                    DetectionCategory::TimeDelaySandboxEvasion,
                    Severity::Medium,
                    "VM characteristic detection",
                    "Code checks VM characteristics (CPU, memory). \
                     This could be legitimate performance optimization or sandbox evasion. \
                     Review the context for malicious intent.",
                )
                .with_cwe_id("CWE-506")
                .with_reference("https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode")
                .with_confidence(0.55),
            );
        }

        findings
    }

    fn metadata(&self) -> DetectorMetadata {
        DetectorMetadata {
            name: "sandbox_evasion".to_string(),
            version: "1.0.0".to_string(),
            description: "Detects GlassWorm sandbox evasion including CI/CD detection, VM checks, and silent exits".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::UnicodeConfig;

    #[test]
    fn test_detect_ci_vm_combination() {
        let detector = SandboxEvasionDetector::new();

        // GlassWorm signature: CI + VM detection
        let content = r#"
            const os = require('os');
            
            function isSandbox() {
                // Check CI environment
                const isCI = process.env.CI === 'true' || process.env.GITHUB_ACTIONS === 'true';
                
                // Check VM characteristics
                const cpuCount = os.cpus().length;
                const totalMem = os.totalmem();
                const isVM = cpuCount < 2 || totalMem < 2 * 1024 * 1024 * 1024;
                
                return isCI || isVM;
            }
            
            if (!isSandbox()) {
                executePayload();
            }
        "#;

        let ir = FileIR::build(Path::new("evasion.js"), content);
        let findings = detector.detect(&ir);

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.severity == Severity::Critical));
        assert!(findings.iter().any(|f| f.description.contains("CI + VM detection")));
        assert!(findings.iter().any(|f| f.confidence.unwrap_or(0.0) >= 0.90));
    }

    #[test]
    fn test_detect_silent_exit() {
        let detector = SandboxEvasionDetector::new();

        let content = r#"
            const os = require('os');
            
            if (process.env.CI === 'true') {
                // Exit silently in CI
                process.exit(0);
            }
            
            // Only run in production
            maliciousCode();
        "#;

        let ir = FileIR::build(Path::new("evasion.js"), content);
        let findings = detector.detect(&ir);

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.severity == Severity::Critical));
        assert!(findings.iter().any(|f| f.description.contains("Silent exit")));
    }

    #[test]
    fn test_detect_cpu_check() {
        let detector = SandboxEvasionDetector::new();

        let content = r#"
            const os = require('os');
            
            // Check if running in sandbox (low CPU count)
            if (os.cpus().length < 2) {
                console.log('Sandbox detected');
                process.exit(0);
            }
        "#;

        let ir = FileIR::build(Path::new("evasion.js"), content);
        let findings = detector.detect(&ir);

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.description.contains("CPU count")));
    }

    #[test]
    fn test_detect_memory_check() {
        let detector = SandboxEvasionDetector::new();

        let content = r#"
            const os = require('os');
            
            // Check if running in sandbox (low memory)
            if (os.totalmem() < 2 * 1024 * 1024 * 1024) {
                console.log('Sandbox detected (low RAM)');
                process.exit(0);
            }
        "#;

        let ir = FileIR::build(Path::new("evasion.js"), content);
        let findings = detector.detect(&ir);

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.description.contains("Memory check")));
    }

    #[test]
    fn test_no_detect_legitimate_ci_check() {
        let detector = SandboxEvasionDetector::new();

        // Legitimate CI check without VM detection
        let content = r#"
            // Skip tests in CI
            if (process.env.CI === 'true') {
                console.log('Running in CI, skipping optional tests');
                return;
            }
            
            // Normal test execution
            runTests();
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);

        // Should not have Critical findings (no VM detection)
        assert!(!findings.iter().any(|f| f.severity == Severity::Critical));
    }

    #[test]
    fn test_no_detect_legitimate_os_usage() {
        let detector = SandboxEvasionDetector::new();

        // Legitimate OS module usage without evasion
        let content = r#"
            const os = require('os');
            
            // Log system info for debugging
            console.log('Platform:', os.platform());
            console.log('CPUs:', os.cpus().length);
            console.log('Memory:', os.totalmem());
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);

        // Should not have Critical findings (no conditional logic)
        assert!(!findings.iter().any(|f| f.severity == Severity::Critical));
    }
}
