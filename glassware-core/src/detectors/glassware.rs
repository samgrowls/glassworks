//! Glassware-Specific Detector
//!
//! Specialized detector for Glassware attack patterns including:
//! - Dense runs of VS codepoints (steganographic payloads)
//! - Decoder function patterns (codePointAt + VS constants)
//! - Pipe delimiter patterns (npm variant)
//!
//! Note: This detector requires the `regex` feature for pattern detection.

#[cfg(feature = "regex")]
use regex::Regex;

use crate::config::UnicodeConfig;
use crate::decoder::{decode_vs_stego, find_vs_runs, is_vs_codepoint};
use crate::detector::{Detector, DetectorMetadata, DetectorTier};
use crate::finding::{DetectionCategory, Finding, Severity};
use crate::ir::FileIR;

/// Minimum run length of VS codepoints to consider as stego payload
const DEFAULT_MIN_RUN_LENGTH: usize = 16;

/// Detector for Glassware attack patterns
pub struct GlasswareDetector {
    #[cfg(feature = "regex")]
    decoder_patterns: Vec<Regex>,
    #[cfg(feature = "regex")]
    eval_patterns: Vec<Regex>,
    #[cfg(feature = "regex")]
    encoding_patterns: Vec<Regex>,
    #[cfg(feature = "regex")]
    #[allow(dead_code)]
    config: UnicodeConfig,
    #[cfg(not(feature = "regex"))]
    #[allow(dead_code)]
    config: UnicodeConfig,
    min_run_length: usize,
}

#[cfg(feature = "regex")]
lazy_static::lazy_static! {
    static ref DECODER_PATTERNS: Vec<Regex> = vec![
        // codePointAt with VS constants (GlassWorm signature)
        Regex::new(r"codePointAt\s*\(\s*0x[Ff][Ee]00\s*\)").unwrap(),  // VS1-16
        Regex::new(r"codePointAt\s*\([^)]*0x[Ff][Ee]00").unwrap(),
        Regex::new(r"codePointAt\s*\([^)]*0x[Ee]0100").unwrap(),  // VS17-256
        // String.fromCodePoint with VS (more specific than fromCharCode)
        Regex::new(r"fromCodePoint\s*\([^)]*0x[Ff][Ee]0").unwrap(),
        // Filter patterns specific to invisible chars
        Regex::new(r"\.filter\s*\([^)]*0x200[BCD]").unwrap(),  // ZWSP/ZWNJ/ZWJ
        Regex::new(r"\.filter\s*\([^)]*0x[Ff][Ee]0").unwrap(),  // VS range
    ];

    static ref EVAL_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"\beval\s*\(").unwrap(),
        Regex::new(r"\bFunction\s*\(").unwrap(),
        Regex::new(r"new\s+Function\s*\(").unwrap(),
    ];

    static ref ENCODING_PATTERNS: Vec<Regex> = vec![
        // REMOVED generic patterns - these are NOT GlassWorm-specific
        // atob(), btoa(), Buffer.from are standard JS APIs used everywhere
        // Only flag if combined with invisible chars (handled in detect_glassware_patterns)
    ];

    // Pattern for detecting VS constants in visible code
    static ref VS_CONSTANT_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"0x[Ff][Ee]00").unwrap(),  // 0xFE00
        Regex::new(r"0x[Ee]0100").unwrap(),    // 0xE0100
        Regex::new(r"\\u\{[Ff][Ee]0[0-9A-Fa-f]\}").unwrap(),  // \u{FE0x}
        Regex::new(r"\\u\{[Ee]01[0-9A-Fa-f]{2}\}").unwrap(),  // \u{E01xx}
    ];
}

impl GlasswareDetector {
    pub fn new(config: UnicodeConfig) -> Self {
        Self {
            #[cfg(feature = "regex")]
            decoder_patterns: DECODER_PATTERNS.clone(),
            #[cfg(feature = "regex")]
            eval_patterns: EVAL_PATTERNS.clone(),
            #[cfg(feature = "regex")]
            encoding_patterns: ENCODING_PATTERNS.clone(),
            config,
            min_run_length: DEFAULT_MIN_RUN_LENGTH,
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(UnicodeConfig::default())
    }

    /// Set minimum run length for VS stego detection
    pub fn with_min_run_length(mut self, min_run_length: usize) -> Self {
        self.min_run_length = min_run_length;
        self
    }

    /// Backward compatibility method - wraps the trait interface
    pub fn detect_with_content(&self, content: &str, file_path: &str) -> Vec<Finding> {
        self.detect_impl(content, file_path)
    }

    /// Check if file is build tool output (should be skipped)
    fn is_build_output(file_path: &str, content: &str) -> bool {
        let path_lower = file_path.to_lowercase();
        
        // Check for build output directories
        let build_dirs = [
            "/node_modules/",
            "/dist/",
            "/build/",
            "/out/",
            "/lib/",
            "/generated/",
            "/.next/",
            "/.nuxt/",
            "/.webpack/",
            "/build-cache/",
        ];
        
        // Check for build tool signatures in content
        let build_signatures = [
            "/* webpack",
            "/* babel",
            "/* ts-loader",
            "/* esbuild",
            "/* rollup",
            "/*! For license information",
            "//# sourceMappingURL=",
            "__webpack_require__",
            "__babel_runtime__",
            "/* @preserve",
            "/* @license",
            "Object.defineProperty(exports,",
            "exports.__esModule = true",
        ];
        
        // Check path
        let in_build_dir = build_dirs.iter().any(|d| path_lower.contains(d));
        
        // Check content
        let has_build_signature = build_signatures.iter()
            .any(|s| content.contains(s));
        
        in_build_dir && has_build_signature
    }

    /// Check if build output has evasion or C2 patterns (should still flag)
    fn has_evasion_or_c2(content: &str) -> bool {
        let evasion_patterns = [
            "process.env.CI",
            "process.env.GITHUB_ACTIONS",
            "os.cpus().length",
            "os.totalmem()",
            "process.exit(0)",
        ];
        
        let c2_patterns = [
            "X-Exfil-ID",
            "X-Session-Token",
            "getSignaturesForAddress",
            "innerInstructions",
        ];
        
        evasion_patterns.iter().any(|p| content.contains(p)) ||
        c2_patterns.iter().any(|p| content.contains(p))
    }

    /// Internal implementation of detection logic
    fn detect_impl(&self, content: &str, file_path: &str) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Skip non-JS/TS files (C++, Python, etc.) - these are not in scope for Glassware detection
        let path_lower = file_path.to_lowercase();
        if !path_lower.ends_with(".js") &&
           !path_lower.ends_with(".mjs") &&
           !path_lower.ends_with(".cjs") &&
           !path_lower.ends_with(".ts") &&
           !path_lower.ends_with(".tsx") &&
           !path_lower.ends_with(".jsx") &&
           !path_lower.ends_with(".json") {
            return findings;  // Skip non-JS/TS files
        }

        // Skip build tool output UNLESS it has evasion or C2 patterns
        if Self::is_build_output(file_path, content) {
            if Self::has_evasion_or_c2(content) {
                // Build output with evasion/C2 = still flag (malicious code injected into build)
                // Continue with normal detection
            } else {
                // Pure build output without evasion/C2 = skip
                return findings;
            }
        }

        // 1. Detect dense VS codepoint runs (steganographic payloads)
        findings.extend(self.detect_stego_runs(content, file_path));

        // 2. Detect pipe delimiter pattern (npm variant)
        findings.extend(self.detect_pipe_delimiter(content, file_path));

        // 3. Detect decoder function patterns
        findings.extend(self.detect_decoder_functions(content, file_path));

        // 4. Detect Glassware patterns (original functionality)
        findings.extend(self.detect_glassware_patterns(content, file_path));

        findings
    }

    #[cfg(not(feature = "regex"))]
    pub fn detect(&self, content: &str, file_path: &str) -> Vec<Finding> {
        let mut findings = Vec::new();
        findings.extend(self.detect_stego_runs(content, file_path));
        findings
    }

    /// Detect dense runs of VS codepoints
    fn detect_stego_runs(&self, content: &str, file_path: &str) -> Vec<Finding> {
        let mut findings = Vec::new();
        let runs = find_vs_runs(content, self.min_run_length);

        for (start_offset, end_offset, codepoint_count) in runs {
            // Extract the VS run
            let vs_run = &content[start_offset..end_offset];

            // Calculate line/column for the start of the run
            let (line, column) = self.offset_to_line_col(content, start_offset);

            // Try to decode the payload
            let decoded = decode_vs_stego(vs_run);

            let (description, severity, decoded_payload) = if let Some(payload) = decoded {
                let class_desc = payload.payload_class.description();
                (
                    format!(
                        "Steganographic payload detected: {} VS codepoints decode to {} bytes (entropy: {:.2}, {})",
                        codepoint_count,
                        payload.bytes.len(),
                        payload.entropy,
                        class_desc
                    ),
                    Severity::Critical,
                    Some(payload),
                )
            } else {
                (
                    format!(
                        "Dense run of {} Variation Selector codepoints detected (potential steganographic payload)",
                        codepoint_count
                    ),
                    Severity::High,
                    None,
                )
            };

            let finding = Finding {
                file: file_path.to_string(),
                line,
                column,
                code_point: 0,
                character: String::new(),
                raw_bytes: None,
                category: DetectionCategory::SteganoPayload,
                severity,
                description,
                remediation: "Review this file for hidden steganographic payloads. \
                             The Variation Selector codepoints may encode malicious code \
                             that is decoded and executed at runtime. Use the decoded payload \
                             preview to understand what's hidden."
                    .to_string(),
                cwe_id: Some("CWE-506".to_string()), // Embedded Malicious Code
                references: vec![
                    "https://www.aikido.dev/blog/glassware-returns-unicode-attack-github-npm-vscode".to_string(),
                ],
                context: self.get_context(content, start_offset, end_offset),
                decoded_payload,
                confidence: None,
            };

            findings.push(finding);
        }

        findings
    }

    /// Detect VS codepoints after pipe delimiter (npm variant)
    fn detect_pipe_delimiter(&self, content: &str, file_path: &str) -> Vec<Finding> {
        let mut findings = Vec::new();
        let chars: Vec<(usize, char)> = content.char_indices().collect();

        for i in 0..chars.len() {
            if chars[i].1 == '|' {
                // Check if next char is a VS codepoint
                if i + 1 < chars.len() && is_vs_codepoint(chars[i + 1].1) {
                    // Count how many VS codepoints follow
                    let mut vs_count = 0;
                    let mut j = i + 1;
                    while j < chars.len() && is_vs_codepoint(chars[j].1) {
                        vs_count += 1;
                        j += 1;
                    }

                    if vs_count >= 4 {
                        let (line, column) = self.offset_to_line_col(content, chars[i].0);

                        // Extract the VS run after the pipe
                        let vs_start = chars[i + 1].0;
                        let vs_end = if j < chars.len() {
                            chars[j].0
                        } else {
                            content.len()
                        };
                        let vs_run = &content[vs_start..vs_end];

                        let decoded = decode_vs_stego(vs_run);

                        let finding = Finding {
                            file: file_path.to_string(),
                            line,
                            column,
                            code_point: 0,
                            character: String::new(),
                            raw_bytes: None,
                            category: DetectionCategory::PipeDelimiterStego,
                            severity: Severity::Critical,
                            description: format!(
                                "Pipe delimiter steganography detected: {} VS codepoints after '|' (npm variant)",
                                vs_count
                            ),
                            remediation: "This is the npm GlassWare variant. VS codepoints after \
                                         pipe delimiters encode hidden payloads. Review the decoded content."
                                .to_string(),
                            cwe_id: Some("CWE-506".to_string()),
                            references: vec![
                                "https://www.aikido.dev/blog/glassware-returns-unicode-attack-github-npm-vscode".to_string(),
                            ],
                            context: self.get_context(content, chars[i].0, vs_end),
                            decoded_payload: decoded,
                            confidence: None,
                        };

                        findings.push(finding);
                    }
                }
            }
        }

        findings
    }

    /// Detect decoder function patterns in visible code
    /// 
    /// CRITICAL FIX 2026-03-25: Require invisible chars OR VS-specific decoder
    /// Previously flagged legitimate libraries that use codePointAt for Unicode handling
    #[cfg(feature = "regex")]
    fn detect_decoder_functions(&self, content: &str, file_path: &str) -> Vec<Finding> {
        let mut findings = Vec::new();

        // First check if file has invisible Unicode chars (required for GlassWorm)
        let has_invisible = Self::find_invisible_unicode(content).len() > 0;
        if !has_invisible {
            return findings;  // No invisible chars = NOT GlassWorm
        }

        for (line_num, line) in content.lines().enumerate() {
            // Check for VS constant patterns (indicates decoder code)
            for pattern in VS_CONSTANT_PATTERNS.iter() {
                if let Some(m) = pattern.find(line) {
                    // Also check if codePointAt is nearby
                    let has_codepointat = line.contains("codePointAt");

                    if has_codepointat {
                        let finding = Finding {
                            file: file_path.to_string(),
                            line: line_num + 1,
                            column: m.start() + 1,
                            code_point: 0,
                            character: String::new(),
                            raw_bytes: None,
                            category: DetectionCategory::DecoderFunction,
                            severity: Severity::High,
                            description: "GlassWare-style decoder function detected: codePointAt \
                                         with Variation Selector range constants (0xFE00/0xE0100)"
                                .to_string(),
                            remediation: "This file contains decoder logic for VS steganographic \
                                         payloads. Review the file that contains the encoded payload."
                                .to_string(),
                            cwe_id: Some("CWE-506".to_string()),
                            references: vec![
                                "https://www.aikido.dev/blog/glassware-returns-unicode-attack-github-npm-vscode".to_string(),
                            ],
                            context: self.get_line_context(line),
                            decoded_payload: None,
                            confidence: None,
                        };

                        findings.push(finding);
                        break; // One finding per line
                    }
                }
            }
        }

        findings
    }

    #[cfg(not(feature = "regex"))]
    fn detect_decoder_functions(&self, _content: &str, _file_path: &str) -> Vec<Finding> {
        Vec::new()
    }

    /// Detect GlassWorm steganography patterns
    /// 
    /// CRITICAL FIX 2026-03-25: Require BOTH invisible chars AND decoder patterns
    /// Previously flagged legitimate libraries (firebase, web3, prisma) due to
    /// generic patterns like atob(), fromCharCode() which are standard JS APIs.
    /// 
    /// Real GlassWorm = Invisible Unicode + VS-specific decoder + (optionally) execution
    #[cfg(feature = "regex")]
    fn detect_glassware_patterns(&self, content: &str, file_path: &str) -> Vec<Finding> {
        let mut findings = Vec::new();

        // Skip minified/bundled files (high FP rate)
        let path_lower = file_path.to_lowercase();
        let is_minified = path_lower.contains(".min.")
            || path_lower.contains("/dist/")
            || path_lower.contains("/build/")
            || path_lower.contains("/bundle")
            || path_lower.ends_with(".bundle.js");

        if is_minified {
            return findings;
        }

        // Skip if file appears to be minified
        if self.is_minified_content(content) {
            return findings;
        }

        // STEP 1: Check for invisible Unicode characters (REQUIRED)
        let invisible_chars = Self::find_invisible_unicode(content);
        if invisible_chars.is_empty() {
            return findings;  // No invisible chars = NOT GlassWorm (prevents FPs on firebase, web3, etc.)
        }

        // STEP 2: Check for GlassWorm-specific decoder patterns
        let mut decoder_indicators = Vec::new();
        
        for (line_num, line) in content.lines().enumerate() {
            // VS-specific decoding (GlassWorm signature - HIGH confidence)
            for pattern in &self.decoder_patterns {
                if let Some(m) = pattern.find(line) {
                    // Only flag if it's VS-specific (0xFE00, 0xE0100), not generic fromCharCode
                    if line.contains("0xFE00") || line.contains("0xE0100") || line.contains("0xfe0") {
                        decoder_indicators.push((line_num, m.start(), "vs_decoder", 0.95));
                    }
                    // Skip generic fromCharCode/fromCodePoint without VS context (legitimate usage)
                }
            }

            // Check for filtering invisible chars specifically
            if line.contains(".filter") && (line.contains("0x200") || line.contains("0xFE0")) {
                decoder_indicators.push((line_num, 0, "invisible_filter", 0.90));
            }

            // Check for VS reconstruction
            if line.contains("fromCodePoint") && line.contains("map") &&
               (line.contains("0xFE0") || line.contains("0xE01")) {
                decoder_indicators.push((line_num, 0, "vs_reconstruction", 0.92));
            }
        }

        // STEP 3: Only flag if we have BOTH invisible chars AND decoder
        if decoder_indicators.is_empty() {
            return findings;  // Invisible chars without decoder = likely legitimate i18n data
        }

        // STEP 4: Calculate confidence based on combination
        let invisible_score = (invisible_chars.len() as f64).min(50.0) / 50.0 * 0.40;  // Up to 40%
        let decoder_score: f64 = decoder_indicators.iter().map(|(_, _, _, c)| *c as f64).sum::<f64>() * 0.60;  // Up to 60%
        let confidence = (invisible_score + decoder_score).min(1.0);

        // Only flag if confidence >= 70%
        if confidence >= 0.70 {
            let severity = if confidence >= 0.90 {
                Severity::Critical
            } else if confidence >= 0.80 {
                Severity::High
            } else {
                Severity::Medium
            };

            let finding = Finding {
                file: file_path.to_string(),
                line: decoder_indicators[0].0 + 1,
                column: decoder_indicators[0].1 + 1,
                code_point: 0,
                character: String::new(),
                raw_bytes: None,
                category: DetectionCategory::GlasswarePattern,
                severity,
                description: format!(
                    "GlassWorm steganography detected: {} invisible chars + decoder (confidence: {:.0}%)",
                    invisible_chars.len(),
                    confidence * 100.0
                ),
                remediation: "Review code for hidden payloads. Check for dynamic code execution (eval, Function, dynamic require).".to_string(),
                cwe_id: Some("CWE-956".to_string()),
                references: vec![
                    "https://www.aikido.dev/blog/glassware-returns-unicode-attack-github-npm-vscode".to_string(),
                ],
                context: Some(content.lines().nth(decoder_indicators[0].0).unwrap_or("").to_string()),
                decoded_payload: None,
                confidence: Some(confidence),
            };

            findings.push(finding);
        }

        findings
    }

    /// Find invisible Unicode characters in content
    fn find_invisible_unicode(content: &str) -> Vec<(usize, char)> {
        content.char_indices()
            .filter(|(_, ch)| {
                let cp = *ch as u32;
                // Variation Selectors (GlassWorm primary)
                (cp >= 0xFE00 && cp <= 0xFE0F) ||  // VS1-16
                (cp >= 0xE0100 && cp <= 0xE01EF) || // VS17-256
                // Zero-Width characters
                cp == 0x200B ||  // ZWSP
                cp == 0x200C ||  // ZWNJ
                cp == 0x200D ||  // ZWJ
                cp == 0x2060     // Word Joiner
            })
            .collect()
    }

    #[cfg(not(feature = "regex"))]
    fn detect_glassware_patterns(&self, _content: &str, _file_path: &str) -> Vec<Finding> {
        Vec::new()
    }

    /// Check if content appears to be minified (heuristic)
    #[cfg(feature = "regex")]
    fn is_minified_content(&self, content: &str) -> bool {
        // Check for minification markers
        let lines: Vec<&str> = content.lines().collect();
        
        // If average line length is very long (>200 chars), likely minified
        if !lines.is_empty() {
            let avg_line_len = content.len() / lines.len();
            if avg_line_len > 200 {
                return true;
            }
        }
        
        // Check for very short variable names (a, b, c, etc.) in function definitions
        let short_var_pattern = Regex::new(r"function\s*\([a-z],[a-z]").unwrap();
        if short_var_pattern.is_match(content) {
            return true;
        }
        
        // Check for lack of whitespace (minified)
        let whitespace_ratio = content.chars().filter(|c| c.is_whitespace()).count() as f32 / content.len() as f32;
        if whitespace_ratio < 0.1 {
            return true;
        }
        
        false
    }

    #[cfg(feature = "regex")]
    fn calculate_confidence(indicator_count: usize) -> f32 {
        let base = (indicator_count as f32 * 0.2).min(0.8);
        let count_bonus = match indicator_count {
            2..=3 => 0.05,
            4..=5 => 0.1,
            _ => 0.15,
        };
        (base + count_bonus).min(1.0)
    }

    #[cfg(not(feature = "regex"))]
    fn calculate_confidence(_indicator_count: usize) -> f32 {
        0.0
    }

    #[cfg(feature = "regex")]
    fn get_remediation(confidence: f32) -> String {
        if confidence >= 0.8 {
            "CRITICAL: This code exhibits strong GlassWare attack characteristics.".to_string()
        } else if confidence >= 0.6 {
            "HIGH: This code shows patterns consistent with GlassWare-style attacks.".to_string()
        } else {
            "MEDIUM: Some patterns associated with GlassWare attacks were detected.".to_string()
        }
    }

    /// Convert byte offset to line/column
    fn offset_to_line_col(&self, content: &str, offset: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;

        for (i, ch) in content.char_indices() {
            if i >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        (line, col)
    }

    /// Get context around a byte range
    fn get_context(&self, content: &str, start: usize, end: usize) -> Option<String> {
        // Get surrounding context (up to 100 chars before and after)
        let context_start = start.saturating_sub(100);
        let context_end = (end + 100).min(content.len());

        let prefix = if context_start > 0 { "..." } else { "" };
        let suffix = if context_end < content.len() {
            "..."
        } else {
            ""
        };

        Some(format!(
            "{}{}{}{}",
            prefix,
            &content[context_start..start],
            &content[end..context_end],
            suffix
        ))
    }

    /// Get line context
    fn get_line_context(&self, line: &str) -> Option<String> {
        // Truncate long lines
        if line.len() > 200 {
            Some(format!("{}...", &line[..200]))
        } else {
            Some(line.to_string())
        }
    }
}

impl Detector for GlasswareDetector {
    fn name(&self) -> &str {
        "glassware"
    }

    fn tier(&self) -> DetectorTier {
        DetectorTier::Tier2Secondary
    }

    fn detect(&self, ir: &FileIR) -> Vec<Finding> {
        self.detect_impl(ir.content(), &ir.metadata.path)
    }

    fn cost(&self) -> u8 {
        5  // Medium cost - multiple regex passes + payload decoding
    }

    fn signal_strength(&self) -> u8 {
        7  // High signal - Glassware patterns are specific
    }

    fn prerequisites(&self) -> Vec<&'static str> {
        vec!["invisible_char", "homoglyph"]  // Run after Tier 1
    }

    fn metadata(&self) -> DetectorMetadata {
        DetectorMetadata {
            name: "glassware".to_string(),
            version: "1.0.0".to_string(),
            description: "Detects Glassware attack patterns including steganographic payloads, decoder functions, and pipe delimiters".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::encode_vs_stego;

    #[test]
    #[cfg(feature = "regex")]
    fn test_stego_run_detection() {
        let detector = GlasswareDetector::with_default_config();

        // Create a run of 20 VS codepoints encoding "test"
        let vs_run = encode_vs_stego(b"test payload here!");
        let content = format!("visible code{}more code", vs_run);

        let findings = detector.detect_with_content(&content, "test.js");

        assert!(findings
            .iter()
            .any(|f| f.category == DetectionCategory::SteganoPayload));
    }

    #[test]
    #[cfg(feature = "regex")]
    fn test_pipe_delimiter_detection() {
        let detector = GlasswareDetector::with_default_config();

        // Create pipe delimiter pattern
        let vs_run = encode_vs_stego(b"hidden");
        let content = format!("some|{}data", vs_run);

        let findings = detector.detect_with_content(&content, "test.js");

        assert!(findings
            .iter()
            .any(|f| f.category == DetectionCategory::PipeDelimiterStego));
    }

    #[test]
    #[cfg(feature = "regex")]
    fn test_decoder_function_detection() {
        let detector = GlasswareDetector::with_default_config();

        let content = r#"
            const decode = (chars) => {
                return chars.map(c => String.fromCodePoint(
                    c.codePointAt(0) - 0xFE00
                )).join('');
            };
        "#;

        let findings = detector.detect_with_content(content, "test.js");

        assert!(findings
            .iter()
            .any(|f| f.category == DetectionCategory::DecoderFunction));
    }

    #[test]
    fn test_clean_content() {
        let detector = GlasswareDetector::with_default_config();
        let content = r#"const normal = 'hello world';"#;
        let findings = detector.detect_with_content(content, "test.js");
        assert!(findings.is_empty());
    }

    #[test]
    #[cfg(feature = "regex")]
    fn test_build_output_without_evasion_not_flagged() {
        let detector = GlasswareDetector::with_default_config();
        
        // Webpack build output without evasion patterns
        let content = r#"
            /* webpack bootstrap */
            (function(modules) {
                function __webpack_require__(moduleId) {
                    var module = installedModules[moduleId];
                    return module.exports;
                }
            })([]);
            //# sourceMappingURL=bundle.js.map
        "#;
        
        let findings = detector.detect_with_content(content, "package/dist/bundle.js");
        
        // Should be skipped - pure build output
        assert!(findings.is_empty());
    }

    #[test]
    #[cfg(feature = "regex")]
    fn test_build_output_with_evasion_still_flagged() {
        let detector = GlasswareDetector::with_default_config();
        
        // Build output WITH evasion pattern (should still be flagged)
        let content = r#"
            /* webpack bootstrap */
            (function() {
                if (process.env.CI === 'true') {
                    process.exit(0);
                }
                __webpack_require__(0);
            })();
            //# sourceMappingURL=bundle.js.map
        "#;
        
        let findings = detector.detect_with_content(content, "package/dist/bundle.js");
        
        // Should NOT be skipped - has evasion pattern
        // Note: This test verifies the skip logic is bypassed, but actual findings
        // depend on other detectors (the evasion pattern itself isn't detected by glassware detector)
        // The key is that is_build_output returns true but has_evasion_or_c2 also returns true
        assert!(GlasswareDetector::is_build_output("package/dist/bundle.js", content));
        assert!(GlasswareDetector::has_evasion_or_c2(content));
    }

    #[test]
    #[cfg(feature = "regex")]
    fn test_build_output_with_c2_still_flagged() {
        let detector = GlasswareDetector::with_default_config();
        
        // Build output WITH C2 pattern (should still be flagged)
        let content = r#"
            /* babel runtime helpers */
            exports.__esModule = true;
            const headers = {
                "X-Exfil-ID": sessionId,
                "Content-Type": "application/json"
            };
        "#;
        
        let findings = detector.detect_with_content(content, "package/lib/index.js");
        
        // Verify build output detection and C2 detection
        assert!(GlasswareDetector::is_build_output("package/lib/index.js", content));
        assert!(GlasswareDetector::has_evasion_or_c2(content));
    }

    #[test]
    fn test_build_signature_detection() {
        // Test various build tool signatures
        let webpack_sig = "/* webpack";
        let babel_sig = "/* babel";
        let sourcemap_sig = "//# sourceMappingURL=";
        let webpack_require = "__webpack_require__";
        let babel_runtime = "__babel_runtime__";
        
        assert!(GlasswareDetector::is_build_output("package/dist/app.js", webpack_sig));
        assert!(GlasswareDetector::is_build_output("package/build/main.js", babel_sig));
        assert!(GlasswareDetector::is_build_output("package/lib/index.js", sourcemap_sig));
        assert!(GlasswareDetector::is_build_output("package/dist/bundle.js", webpack_require));
        assert!(GlasswareDetector::is_build_output("package/out/app.js", babel_runtime));
    }

    #[test]
    fn test_build_directory_detection() {
        // Test build directory detection with various signatures
        let sig = "/* webpack";
        
        assert!(GlasswareDetector::is_build_output("package/dist/app.js", sig));
        assert!(GlasswareDetector::is_build_output("package/build/main.js", sig));
        assert!(GlasswareDetector::is_build_output("package/out/app.js", sig));
        assert!(GlasswareDetector::is_build_output("package/lib/index.js", sig));
        assert!(GlasswareDetector::is_build_output("package/generated/code.js", sig));
        assert!(GlasswareDetector::is_build_output("package/.next/static/app.js", sig));
        assert!(GlasswareDetector::is_build_output("package/.nuxt/dist/app.js", sig));
        assert!(GlasswareDetector::is_build_output("package/.webpack/bundle.js", sig));
        assert!(GlasswareDetector::is_build_output("package/build-cache/output.js", sig));
    }

    #[test]
    fn test_non_build_output_not_skipped() {
        // Source files should not be skipped even with some signatures
        let content = "const x = 1; // some comment";
        
        // No build directory = not skipped
        assert!(!GlasswareDetector::is_build_output("package/src/index.js", content));
        assert!(!GlasswareDetector::is_build_output("package/lib/source.ts", content));
        assert!(!GlasswareDetector::is_build_output("package/index.js", content));
    }

    #[test]
    fn test_evasion_patterns() {
        // Test evasion pattern detection
        assert!(GlasswareDetector::has_evasion_or_c2("if (process.env.CI) {}"));
        assert!(GlasswareDetector::has_evasion_or_c2("process.env.GITHUB_ACTIONS"));
        assert!(GlasswareDetector::has_evasion_or_c2("os.cpus().length"));
        assert!(GlasswareDetector::has_evasion_or_c2("os.totalmem()"));
        assert!(GlasswareDetector::has_evasion_or_c2("process.exit(0)"));
    }

    #[test]
    fn test_c2_patterns() {
        // Test C2 pattern detection
        assert!(GlasswareDetector::has_evasion_or_c2("X-Exfil-ID"));
        assert!(GlasswareDetector::has_evasion_or_c2("X-Session-Token"));
        assert!(GlasswareDetector::has_evasion_or_c2("getSignaturesForAddress"));
        assert!(GlasswareDetector::has_evasion_or_c2("innerInstructions"));
    }

    #[test]
    fn test_clean_content_not_flagged_as_build_or_evasion() {
        // Clean content should not trigger build output or evasion detection
        let content = "const normal = 'hello world';";
        
        assert!(!GlasswareDetector::is_build_output("package/src/index.js", content));
        assert!(!GlasswareDetector::has_evasion_or_c2(content));
    }
}
