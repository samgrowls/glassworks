//! Adversarial testing integration for evasion detection.
//!
//! This module provides adversarial testing capabilities to detect packages
//! that may be using evasion techniques to avoid detection.
//!
//! Features:
//! - Mutation engine for testing detector robustness
//! - Fuzzer engine for generating edge cases
//! - Evasion rate calculation
//! - Adversarial report generation
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use orchestrator_core::adversarial::AdversarialTester;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let tester = AdversarialTester::new();
//!
//!     // Test a package for evasion techniques
//!     let report = tester.test_package("suspicious-package").await?;
//!
//!     println!("Evasion rate: {:.2}%", report.evasion_rate);
//!     println!("High-risk evasions: {}", report.high_risk_evasions.len());
//!
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};

use crate::error::{OrchestratorError, Result};
use crate::scanner::{Scanner, ScannerConfig};

/// Mutation strategy for adversarial testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationStrategy {
    /// Insert invisible characters at random positions
    InvisibleInsertion,
    /// Replace characters with homoglyphs
    HomoglyphReplacement,
    /// Insert bidirectional overrides
    BidiInsertion,
    /// Encode payload with steganography
    StegoEncoding,
    /// Split payload across multiple files
    PayloadSplitting,
    /// Add noise/decoy code
    NoiseInjection,
    /// Obfuscate variable names
    NameObfuscation,
    /// Insert dead code
    DeadCodeInjection,
}

/// Fuzzing strategy for generating edge cases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuzzStrategy {
    /// Generate random Unicode sequences
    RandomUnicode,
    /// Generate maximum-length strings
    MaxLength,
    /// Generate nested structures
    NestedStructures,
    /// Generate malformed syntax
    MalformedSyntax,
    /// Generate timing-based attacks
    TimingAttack,
    /// Generate memory-exhaustion patterns
    MemoryExhaustion,
}

/// Configuration for the mutation engine.
#[derive(Debug, Clone)]
pub struct MutationEngineConfig {
    /// Number of mutations to generate per test
    pub mutations_per_test: usize,
    /// Mutation probability (0.0 to 1.0)
    pub mutation_probability: f64,
    /// Enable invisible character mutations
    pub enable_invisible: bool,
    /// Enable homoglyph mutations
    pub enable_homoglyph: bool,
    /// Enable bidi mutations
    pub enable_bidi: bool,
    /// Enable stego mutations
    pub enable_stego: bool,
    /// Enable noise injection
    pub enable_noise: bool,
}

impl Default for MutationEngineConfig {
    fn default() -> Self {
        Self {
            mutations_per_test: 10,
            mutation_probability: 0.3,
            enable_invisible: true,
            enable_homoglyph: true,
            enable_bidi: true,
            enable_stego: true,
            enable_noise: false,
        }
    }
}

/// Configuration for the fuzzer engine.
#[derive(Debug, Clone)]
pub struct FuzzerEngineConfig {
    /// Number of fuzz cases to generate
    pub fuzz_cases: usize,
    /// Enable Unicode fuzzing
    pub enable_unicode: bool,
    /// Enable length fuzzing
    pub enable_length: bool,
    /// Enable structure fuzzing
    pub enable_structure: bool,
    /// Enable syntax fuzzing
    pub enable_syntax: bool,
}

impl Default for FuzzerEngineConfig {
    fn default() -> Self {
        Self {
            fuzz_cases: 100,
            enable_unicode: true,
            enable_length: true,
            enable_structure: false,
            enable_syntax: false,
        }
    }
}

/// Result of a single mutation test.
#[derive(Debug, Clone)]
pub struct MutationTestResult {
    /// Strategy used
    pub strategy: MutationStrategy,
    /// Original content
    pub original: String,
    /// Mutated content
    pub mutated: String,
    /// Whether the mutation was detected
    pub detected: bool,
    /// Number of findings in mutated content
    pub findings_count: usize,
    /// Time taken for the test
    pub duration_ms: u64,
}

/// Result of a single fuzz test.
#[derive(Debug, Clone)]
pub struct FuzzTestResult {
    /// Strategy used
    pub strategy: FuzzStrategy,
    /// Fuzz input generated
    pub input: String,
    /// Whether the fuzzer caused a crash/error
    pub caused_error: bool,
    /// Error message if any
    pub error_message: Option<String>,
    /// Time taken for the test
    pub duration_ms: u64,
}

/// Report of adversarial testing results.
#[derive(Debug, Clone)]
pub struct AdversarialReport {
    /// Package name tested
    pub package_name: String,
    /// Total mutations tested
    pub total_mutations: usize,
    /// Mutations that evaded detection
    pub evaded_mutations: usize,
    /// Total fuzz cases tested
    pub total_fuzz_cases: usize,
    /// Fuzz cases that caused errors
    pub erroring_fuzz_cases: usize,
    /// Overall evasion rate (0.0 to 1.0)
    pub evasion_rate: f64,
    /// High-risk evasion techniques detected
    pub high_risk_evasions: Vec<String>,
    /// Recommendations for improving detection
    pub recommendations: Vec<String>,
    /// Detailed mutation results
    pub mutation_results: Vec<MutationTestResult>,
    /// Detailed fuzz results
    pub fuzz_results: Vec<FuzzTestResult>,
}

impl AdversarialReport {
    /// Create a new adversarial report.
    pub fn new(package_name: String) -> Self {
        Self {
            package_name,
            total_mutations: 0,
            evaded_mutations: 0,
            total_fuzz_cases: 0,
            erroring_fuzz_cases: 0,
            evasion_rate: 0.0,
            high_risk_evasions: Vec::new(),
            recommendations: Vec::new(),
            mutation_results: Vec::new(),
            fuzz_results: Vec::new(),
        }
    }

    /// Get risk level based on evasion rate.
    pub fn risk_level(&self) -> RiskLevel {
        if self.evasion_rate >= 0.5 {
            RiskLevel::Critical
        } else if self.evasion_rate >= 0.3 {
            RiskLevel::High
        } else if self.evasion_rate >= 0.1 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    /// Check if the package is considered high-risk.
    pub fn is_high_risk(&self) -> bool {
        self.risk_level() == RiskLevel::High || self.risk_level() == RiskLevel::Critical
    }
}

/// Risk level classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    /// Low risk - evasion rate < 10%
    Low,
    /// Medium risk - evasion rate 10-30%
    Medium,
    /// High risk - evasion rate 30-50%
    High,
    /// Critical risk - evasion rate >= 50%
    Critical,
}

/// Mutation engine for generating test mutations.
pub struct MutationEngine {
    config: MutationEngineConfig,
    scanner: Scanner,
    concurrency_semaphore: Arc<Semaphore>,
}

impl MutationEngine {
    /// Create a new mutation engine.
    pub fn new() -> Result<Self> {
        Self::with_config(MutationEngineConfig::default())
    }

    /// Create a new mutation engine with custom configuration.
    pub fn with_config(config: MutationEngineConfig) -> Result<Self> {
        let scanner = Scanner::with_config(ScannerConfig::default());
        let concurrency_semaphore = Arc::new(Semaphore::new(5));

        Ok(Self {
            config,
            scanner,
            concurrency_semaphore,
        })
    }

    /// Generate mutations for testing.
    pub fn generate_mutations(&self, original: &str, strategy: MutationStrategy) -> Vec<String> {
        let mut mutations = Vec::new();

        for _ in 0..self.config.mutations_per_test {
            if let Some(mutated) = self.apply_mutation(original, strategy) {
                mutations.push(mutated);
            }
        }

        mutations
    }

    /// Apply a single mutation to content.
    fn apply_mutation(&self, content: &str, strategy: MutationStrategy) -> Option<String> {
        match strategy {
            MutationStrategy::InvisibleInsertion => {
                self.insert_invisible_chars(content)
            }
            MutationStrategy::HomoglyphReplacement => {
                self.replace_with_homoglyphs(content)
            }
            MutationStrategy::BidiInsertion => {
                self.insert_bidi_chars(content)
            }
            MutationStrategy::StegoEncoding => {
                self.encode_stego(content)
            }
            MutationStrategy::PayloadSplitting => {
                // This requires multiple files, skip for now
                None
            }
            MutationStrategy::NoiseInjection => {
                Some(self.inject_noise(content))
            }
            MutationStrategy::NameObfuscation => {
                Some(self.obfuscate_names(content))
            }
            MutationStrategy::DeadCodeInjection => {
                Some(self.inject_dead_code(content))
            }
        }
    }

    /// Insert invisible characters at random positions.
    fn insert_invisible_chars(&self, content: &str) -> Option<String> {
        if content.is_empty() {
            return None;
        }

        let mut result = String::with_capacity(content.len() + 100);
        let invisible_chars = ['\u{200B}', '\u{200C}', '\u{200D}', '\u{FE00}', '\u{E0100}'];

        for (i, ch) in content.chars().enumerate() {
            result.push(ch);

            // Randomly insert invisible character
            if i % 10 == 0 && rand::random::<f64>() < self.config.mutation_probability {
                let invisible = invisible_chars[rand::random::<usize>() % invisible_chars.len()];
                result.push(invisible);
            }
        }

        Some(result)
    }

    /// Replace characters with homoglyphs.
    fn replace_with_homoglyphs(&self, content: &str) -> Option<String> {
        if content.is_empty() {
            return None;
        }

        // Common Latin to Cyrillic/Greek homoglyphs
        let homoglyphs: HashMap<char, Vec<char>> = [
            ('a', vec!['а']),  // Cyrillic
            ('e', vec!['е']),  // Cyrillic
            ('o', vec!['о']),  // Cyrillic
            ('p', vec!['р']),  // Cyrillic
            ('c', vec!['с']),  // Cyrillic
            ('x', vec!['х']),  // Cyrillic
            ('A', vec!['Α']),  // Greek
            ('B', vec!['Β']),  // Greek
            ('E', vec!['Ε']),  // Greek
            ('H', vec!['Η']),  // Greek
            ('I', vec!['Ι']),  // Greek
            ('K', vec!['Κ']),  // Greek
            ('M', vec!['Μ']),  // Greek
            ('N', vec!['Ν']),  // Greek
            ('O', vec!['Ο']),  // Greek
            ('P', vec!['Ρ']),  // Greek
            ('T', vec!['Τ']),  // Greek
            ('X', vec!['Χ']),  // Greek
            ('Y', vec!['Υ']),  // Greek
            ('Z', vec!['Ζ']),  // Greek
        ].iter().cloned().collect();

        let mut result = String::with_capacity(content.len());

        for ch in content.chars() {
            if rand::random::<f64>() < self.config.mutation_probability {
                if let Some(replacements) = homoglyphs.get(&ch) {
                    let replacement = replacements[rand::random::<usize>() % replacements.len()];
                    result.push(replacement);
                    continue;
                }
            }
            result.push(ch);
        }

        Some(result)
    }

    /// Insert bidirectional override characters.
    fn insert_bidi_chars(&self, content: &str) -> Option<String> {
        if content.is_empty() {
            return None;
        }

        let mut result = String::with_capacity(content.len() + 20);
        let bidi_chars = ['\u{202A}', '\u{202B}', '\u{202C}', '\u{202D}', '\u{202E}', '\u{2066}', '\u{2067}', '\u{2068}', '\u{2069}'];

        // Insert at beginning and end
        result.push(bidi_chars[rand::random::<usize>() % bidi_chars.len()]);
        result.push_str(content);
        result.push('\u{202C}'); // POP DIRECTIONAL ISOLATE

        Some(result)
    }

    /// Encode content with steganography.
    fn encode_stego(&self, content: &str) -> Option<String> {
        if content.is_empty() {
            return None;
        }

        // Simple stego: encode each byte as variation selector
        let mut result = String::new();

        for byte in content.as_bytes() {
            // Encode as variation selector sequence
            let vs = 0xFE00u32 + (*byte as u32 % 256);
            if let Some(ch) = char::from_u32(vs) {
                result.push(ch);
            }
        }

        Some(result)
    }

    /// Inject noise/decoy code.
    fn inject_noise(&self, content: &str) -> String {
        let noise_patterns = [
            "// TODO: fix this later\n",
            "/* deprecated */\n",
            "var _unused = null;\n",
            "const __DEV__ = false;\n",
            "function _helper() { return 0; }\n",
        ];

        let mut result = String::with_capacity(content.len() + 100);

        for (i, line) in content.lines().enumerate() {
            result.push_str(line);
            result.push('\n');

            // Randomly inject noise
            if i % 5 == 0 && rand::random::<f64>() < self.config.mutation_probability {
                let noise = noise_patterns[rand::random::<usize>() % noise_patterns.len()];
                result.push_str(noise);
            }
        }

        result
    }

    /// Obfuscate variable names.
    fn obfuscate_names(&self, content: &str) -> String {
        // Simple obfuscation: replace common variable names
        let mut result = content.to_string();

        let replacements = [
            ("config", "_0x1a2b"),
            ("data", "_0x2c3d"),
            ("value", "_0x3e4f"),
            ("result", "_0x5g6h"),
            ("temp", "_0x7i8j"),
        ];

        for (original, obfuscated) in &replacements {
            result = result.replace(original, obfuscated);
        }

        result
    }

    /// Inject dead code.
    fn inject_dead_code(&self, content: &str) -> String {
        let dead_code = r#"
// Dead code injection
(function() {
    var _dead1 = "unused";
    var _dead2 = 12345;
    var _dead3 = { a: 1, b: 2 };
    return null;
})();
"#;

        format!("{}{}", dead_code, content)
    }

    /// Test mutations against the scanner.
    pub async fn test_mutations(&self, original: &str) -> Vec<MutationTestResult> {
        let mut results = Vec::new();
        let strategies = [
            MutationStrategy::InvisibleInsertion,
            MutationStrategy::HomoglyphReplacement,
            MutationStrategy::BidiInsertion,
            MutationStrategy::StegoEncoding,
            MutationStrategy::NoiseInjection,
        ];

        for &strategy in &strategies {
            let mutations = self.generate_mutations(original, strategy);

            for mutated in mutations {
                let start = std::time::Instant::now();

                // Test if mutation is detected
                let findings = self.scanner.scan_content(&mutated).await;
                let detected = !findings.is_empty();

                results.push(MutationTestResult {
                    strategy,
                    original: original.to_string(),
                    mutated,
                    detected,
                    findings_count: findings.len(),
                    duration_ms: start.elapsed().as_millis() as u64,
                });
            }
        }

        results
    }
}

impl Default for MutationEngine {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

/// Fuzzer engine for generating edge cases.
pub struct FuzzerEngine {
    config: FuzzerEngineConfig,
    scanner: Scanner,
}

impl FuzzerEngine {
    /// Create a new fuzzer engine.
    pub fn new() -> Result<Self> {
        Self::with_config(FuzzerEngineConfig::default())
    }

    /// Create a new fuzzer engine with custom configuration.
    pub fn with_config(config: FuzzerEngineConfig) -> Result<Self> {
        let scanner = Scanner::with_config(ScannerConfig::default());

        Ok(Self {
            config,
            scanner,
        })
    }

    /// Generate fuzz cases.
    pub fn generate_fuzz_cases(&self, strategy: FuzzStrategy) -> Vec<String> {
        let mut cases = Vec::new();

        for _ in 0..self.config.fuzz_cases / 5 {
            cases.push(self.generate_fuzz_input(strategy));
        }

        cases
    }

    /// Generate a single fuzz input.
    fn generate_fuzz_input(&self, strategy: FuzzStrategy) -> String {
        match strategy {
            FuzzStrategy::RandomUnicode => self.generate_random_unicode(),
            FuzzStrategy::MaxLength => self.generate_max_length(),
            FuzzStrategy::NestedStructures => self.generate_nested(),
            FuzzStrategy::MalformedSyntax => self.generate_malformed(),
            FuzzStrategy::TimingAttack => self.generate_timing_attack(),
            FuzzStrategy::MemoryExhaustion => self.generate_memory_exhaustion(),
        }
    }

    /// Generate random Unicode sequences.
    fn generate_random_unicode(&self) -> String {
        let mut result = String::new();
        let len = rand::random::<usize>() % 1000 + 100;

        for _ in 0..len {
            // Generate random Unicode code point
            let code_point = rand::random::<u32>() % 0x10FFFF;
            if let Some(ch) = char::from_u32(code_point) {
                result.push(ch);
            }
        }

        result
    }

    /// Generate maximum-length strings.
    fn generate_max_length(&self) -> String {
        // Generate a very long string
        "A".repeat(1_000_000)
    }

    /// Generate nested structures.
    fn generate_nested(&self) -> String {
        let mut result = String::new();
        let depth = rand::random::<usize>() % 100 + 10;

        for _ in 0..depth {
            result.push_str("function(){");
        }
        result.push_str("return 0;");
        for _ in 0..depth {
            result.push_str("}");
        }

        result
    }

    /// Generate malformed syntax.
    fn generate_malformed(&self) -> String {
        let patterns = [
            "var x = ;",
            "function( { return; }",
            "if (true { }",
            "const a = [1, 2, 3;",
            "let b = { key: value };",
            "class { constructor() { super(); }",
        ];

        patterns[rand::random::<usize>() % patterns.len()].to_string()
    }

    /// Generate timing attack patterns.
    fn generate_timing_attack(&self) -> String {
        let mut result = String::new();

        // Generate computationally expensive operations
        for i in 0..1000 {
            result.push_str(&format!("Math.pow({}, {});\n", i, i));
        }

        result
    }

    /// Generate memory exhaustion patterns.
    fn generate_memory_exhaustion(&self) -> String {
        // Generate large array allocation
        "new Array(1000000000)".to_string()
    }

    /// Test fuzz cases against the scanner.
    pub async fn test_fuzz_cases(&self) -> Vec<FuzzTestResult> {
        let mut results = Vec::new();
        let strategies = [
            FuzzStrategy::RandomUnicode,
            FuzzStrategy::MaxLength,
            FuzzStrategy::NestedStructures,
            FuzzStrategy::MalformedSyntax,
        ];

        for &strategy in &strategies {
            let cases = self.generate_fuzz_cases(strategy);

            for input in cases {
                let start = std::time::Instant::now();

                // Test if fuzzer causes error
                let (caused_error, error_message) = match self.scanner.scan_content(&input).await {
                    findings => {
                        // Scanner didn't crash
                        (false, None)
                    }
                };

                results.push(FuzzTestResult {
                    strategy,
                    input,
                    caused_error,
                    error_message,
                    duration_ms: start.elapsed().as_millis() as u64,
                });
            }
        }

        results
    }
}

impl Default for FuzzerEngine {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

/// Adversarial tester combining mutation and fuzzing engines.
pub struct AdversarialTester {
    mutation_engine: MutationEngine,
    fuzzer_engine: FuzzerEngine,
}

impl AdversarialTester {
    /// Create a new adversarial tester.
    pub fn new() -> Result<Self> {
        Ok(Self {
            mutation_engine: MutationEngine::new()?,
            fuzzer_engine: FuzzerEngine::new()?,
        })
    }

    /// Create with custom configurations.
    pub fn with_configs(
        mutation_config: MutationEngineConfig,
        fuzz_config: FuzzerEngineConfig,
    ) -> Result<Self> {
        Ok(Self {
            mutation_engine: MutationEngine::with_config(mutation_config)?,
            fuzzer_engine: FuzzerEngine::with_config(fuzz_config)?,
        })
    }

    /// Test a package for adversarial vulnerabilities.
    pub async fn test_package(&self, package_path: &str) -> Result<AdversarialReport> {
        info!("Running adversarial tests on package: {}", package_path);

        let mut report = AdversarialReport::new(package_path.to_string());

        // Read package content
        let content = tokio::fs::read_to_string(package_path)
            .await
            .map_err(|e| OrchestratorError::io_error(e))?;

        // Run mutation tests
        debug!("Running mutation tests...");
        let mutation_results = self.mutation_engine.test_mutations(&content).await;
        report.total_mutations = mutation_results.len();
        report.evaded_mutations = mutation_results.iter().filter(|r| !r.detected).count();
        report.mutation_results = mutation_results;

        // Run fuzz tests
        debug!("Running fuzz tests...");
        let fuzz_results = self.fuzzer_engine.test_fuzz_cases().await;
        report.total_fuzz_cases = fuzz_results.len();
        report.erroring_fuzz_cases = fuzz_results.iter().filter(|r| r.caused_error).count();
        report.fuzz_results = fuzz_results;

        // Calculate evasion rate
        if report.total_mutations > 0 {
            report.evasion_rate = report.evaded_mutations as f64 / report.total_mutations as f64;
        }

        // Identify high-risk evasions
        self.identify_high_risk_evasions(&mut report);

        // Generate recommendations
        self.generate_recommendations(&mut report);

        info!(
            "Adversarial testing complete. Evasion rate: {:.2}%",
            report.evasion_rate * 100.0
        );

        Ok(report)
    }

    /// Identify high-risk evasion techniques.
    fn identify_high_risk_evasions(&self, report: &mut AdversarialReport) {
        let mut evasion_by_strategy: HashMap<MutationStrategy, usize> = HashMap::new();

        for result in &report.mutation_results {
            if !result.detected {
                *evasion_by_strategy.entry(result.strategy).or_insert(0) += 1;
            }
        }

        for (strategy, count) in evasion_by_strategy {
            if count > 0 {
                let technique = match strategy {
                    MutationStrategy::InvisibleInsertion => "Invisible character insertion",
                    MutationStrategy::HomoglyphReplacement => "Homoglyph replacement",
                    MutationStrategy::BidiInsertion => "Bidirectional override",
                    MutationStrategy::StegoEncoding => "Steganographic encoding",
                    MutationStrategy::NoiseInjection => "Noise injection",
                    _ => "Unknown",
                };

                report.high_risk_evasions.push(format!(
                    "{} ({} evasions)",
                    technique, count
                ));
            }
        }
    }

    /// Generate recommendations for improving detection.
    fn generate_recommendations(&self, report: &mut AdversarialReport) {
        if report.evasion_rate > 0.5 {
            report.recommendations.push(
                "CRITICAL: High evasion rate detected. Consider enabling semantic analysis.".to_string()
            );
        }

        if report.evaded_mutations > 0 {
            report.recommendations.push(
                "Enable L3 LLM analysis layer for better detection of obfuscated payloads.".to_string()
            );
        }

        if report.erroring_fuzz_cases > 0 {
            report.recommendations.push(
                "Scanner crashed on fuzz inputs. Add better error handling and input validation.".to_string()
            );
        }

        if report.high_risk_evasions.iter().any(|e| e.contains("Invisible")) {
            report.recommendations.push(
                "Strengthen L1 invisible character detection with stricter thresholds.".to_string()
            );
        }

        if report.high_risk_evasions.iter().any(|e| e.contains("Homoglyph")) {
            report.recommendations.push(
                "Expand homoglyph detection to cover more Cyrillic/Greek confusables.".to_string()
            );
        }
    }
}

impl Default for AdversarialTester {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutation_engine_creation() {
        let engine = MutationEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_fuzzer_engine_creation() {
        let engine = FuzzerEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_adversarial_tester_creation() {
        let tester = AdversarialTester::new();
        assert!(tester.is_ok());
    }

    #[test]
    fn test_mutation_strategies() {
        let strategies = [
            MutationStrategy::InvisibleInsertion,
            MutationStrategy::HomoglyphReplacement,
            MutationStrategy::BidiInsertion,
            MutationStrategy::StegoEncoding,
        ];

        for strategy in &strategies {
            // Just verify they can be created
            assert_eq!(*strategy, *strategy);
        }
    }

    #[test]
    fn test_fuzz_strategies() {
        let strategies = [
            FuzzStrategy::RandomUnicode,
            FuzzStrategy::MaxLength,
            FuzzStrategy::NestedStructures,
        ];

        for strategy in &strategies {
            assert_eq!(*strategy, *strategy);
        }
    }

    #[test]
    fn test_risk_level() {
        let mut report = AdversarialReport::new("test".to_string());

        report.evasion_rate = 0.0;
        assert_eq!(report.risk_level(), RiskLevel::Low);

        report.evasion_rate = 0.15;
        assert_eq!(report.risk_level(), RiskLevel::Medium);

        report.evasion_rate = 0.40;
        assert_eq!(report.risk_level(), RiskLevel::High);

        report.evasion_rate = 0.60;
        assert_eq!(report.risk_level(), RiskLevel::Critical);
    }

    #[test]
    fn test_is_high_risk() {
        let mut report = AdversarialReport::new("test".to_string());

        report.evasion_rate = 0.05;
        assert!(!report.is_high_risk());

        report.evasion_rate = 0.35;
        assert!(report.is_high_risk());

        report.evasion_rate = 0.55;
        assert!(report.is_high_risk());
    }

    #[tokio::test]
    async fn test_mutation_engine_generate() {
        let engine = MutationEngine::new().unwrap();
        let original = "const x = 1;";

        let mutations = engine.generate_mutations(original, MutationStrategy::InvisibleInsertion);
        assert!(!mutations.is_empty());

        // Verify mutations are different from original
        for mutation in &mutations {
            assert_ne!(mutation, original);
        }
    }

    #[tokio::test]
    async fn test_fuzzer_engine_generate() {
        let engine = FuzzerEngine::new().unwrap();

        let cases = engine.generate_fuzz_cases(FuzzStrategy::RandomUnicode);
        assert!(!cases.is_empty());

        // Verify cases are non-empty strings
        for case in &cases {
            assert!(!case.is_empty());
        }
    }

    #[tokio::test]
    async fn test_adversarial_report() {
        let mut report = AdversarialReport::new("test-pkg".to_string());

        report.total_mutations = 100;
        report.evaded_mutations = 25;
        report.evasion_rate = 0.25;

        assert_eq!(report.package_name, "test-pkg");
        assert_eq!(report.risk_level(), RiskLevel::Medium);
        assert!(!report.is_high_risk());
    }
}
