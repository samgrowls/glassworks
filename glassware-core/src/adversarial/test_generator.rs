//! Test Generator for Adversarial Testing
//!
//! Collects successful evasions and generates test cases for CI/CD integration.

use super::mutation::MaliciousPayload;

/// Severity levels for evasion test cases
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EvasionSeverity {
    /// Evades all detectors - critical blind spot
    Critical,
    /// Evades Tier 1-2 detectors (regex and semantic)
    High,
    /// Evades some Tier 3 detectors (LLM/review layer)
    Medium,
    /// Evades single detector - minor blind spot
    Low,
}

impl EvasionSeverity {
    /// Get the string representation of the severity level
    pub fn as_str(&self) -> &'static str {
        match self {
            EvasionSeverity::Critical => "critical",
            EvasionSeverity::High => "high",
            EvasionSeverity::Medium => "medium",
            EvasionSeverity::Low => "low",
        }
    }

    /// Calculate evasion severity based on number of detectors evaded
    pub fn from_detectors_evaded(total_detectors: usize, evaded_detectors: usize) -> Self {
        if total_detectors == 0 {
            return EvasionSeverity::Low;
        }

        let evasion_ratio = evaded_detectors as f32 / total_detectors as f32;

        if evasion_ratio == 1.0 {
            EvasionSeverity::Critical
        } else if evasion_ratio >= 0.75 {
            EvasionSeverity::High
        } else if evasion_ratio >= 0.5 {
            EvasionSeverity::Medium
        } else {
            EvasionSeverity::Low
        }
    }
}

impl std::fmt::Display for EvasionSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A test case for adversarial testing
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EvasionTestCase {
    /// Unique test case identifier
    pub name: String,
    /// Description of the evasion technique
    pub description: String,
    /// Original malicious payload before mutation
    pub original_payload: MaliciousPayload,
    /// Mutated payload that evades detection
    pub evasion_payload: MaliciousPayload,
    /// Description of the mutation applied
    pub mutation_applied: String,
    /// List of detector names that were evaded
    pub detectors_evaded: Vec<String>,
    /// Severity of the evasion
    pub severity: EvasionSeverity,
}

impl EvasionTestCase {
    /// Create a new evasion test case
    pub fn new(
        name: String,
        description: String,
        original_payload: MaliciousPayload,
        evasion_payload: MaliciousPayload,
        mutation_applied: String,
        detectors_evaded: Vec<String>,
        severity: EvasionSeverity,
    ) -> Self {
        Self {
            name,
            description,
            original_payload,
            evasion_payload,
            mutation_applied,
            detectors_evaded,
            severity,
        }
    }

    /// Create a test case from mutation results
    pub fn from_mutation(
        original: &MaliciousPayload,
        mutated: &MaliciousPayload,
        mutation_type: &str,
        detectors_evaded: Vec<String>,
        total_detectors: usize,
    ) -> Self {
        let severity = EvasionSeverity::from_detectors_evaded(total_detectors, detectors_evaded.len());

        let name = format!(
            "evasion_{}_{}",
            mutation_type,
            original.attack_type.to_lowercase().replace(' ', "_")
        );

        let description = format!(
            "Evasion test: {} mutation on {} attack pattern",
            mutation_type, original.attack_type
        );

        Self {
            name,
            description,
            original_payload: original.clone(),
            evasion_payload: mutated.clone(),
            mutation_applied: mutation_type.to_string(),
            detectors_evaded,
            severity,
        }
    }

    /// Get the number of detectors evaded
    pub fn detectors_evaded_count(&self) -> usize {
        self.detectors_evaded.len()
    }

    /// Check if this is a critical evasion (all detectors evaded)
    pub fn is_critical(&self) -> bool {
        self.severity == EvasionSeverity::Critical
    }

    /// Generate test code snippet for this test case
    pub fn generate_test_snippet(&self) -> String {
        let escaped_content = self.evasion_payload.content.replace("r###", "r####");
        
        format!(
            "#[test]\nfn test_{}() {{\n    let content = r###\"{}\"###;\n    let findings = scan(content, \"test.js\");\n    \n    // This test documents a known evasion technique\n    // Detectors evaded: {:?}\n    // Severity: {:?}\n    // \n    // TODO: Improve detection to catch this evasion\n    assert!(findings.is_empty(), \"Known evasion should not be detected until fixed\");\n}}",
            self.name,
            escaped_content,
            self.detectors_evaded,
            self.severity
        )
    }
}

/// Statistics for test generation
#[derive(Debug, Default)]
pub struct TestGeneratorStats {
    /// Total test cases generated
    pub total_cases: usize,
    /// Critical severity cases
    pub critical_cases: usize,
    /// High severity cases
    pub high_cases: usize,
    /// Medium severity cases
    pub medium_cases: usize,
    /// Low severity cases
    pub low_cases: usize,
    /// Unique mutation types covered
    pub mutation_types: Vec<String>,
    /// Unique attack types covered
    pub attack_types: Vec<String>,
}

impl TestGeneratorStats {
    /// Get the total number of detectors evaded across all test cases
    pub fn total_detectors_evaded(&self) -> usize {
        self.critical_cases * 4 + self.high_cases * 3 + self.medium_cases * 2 + self.low_cases
    }
}

/// Test Generator - Collects successful evasions and generates test cases
pub struct TestGenerator {
    /// Collection of generated test cases
    test_cases: Vec<EvasionTestCase>,
    /// Statistics about generated tests
    stats: TestGeneratorStats,
    /// Total number of detectors in the system (for severity calculation)
    total_detectors: usize,
}

impl TestGenerator {
    /// Create a new test generator
    pub fn new(total_detectors: usize) -> Self {
        Self {
            test_cases: Vec::new(),
            stats: TestGeneratorStats::default(),
            total_detectors,
        }
    }

    /// Add a test case from a successful evasion
    pub fn add_evasion(
        &mut self,
        original: &MaliciousPayload,
        mutated: &MaliciousPayload,
        mutation_type: &str,
        detectors_evaded: Vec<String>,
    ) {
        let test_case = EvasionTestCase::from_mutation(
            original,
            mutated,
            mutation_type,
            detectors_evaded,
            self.total_detectors,
        );

        self.update_stats(&test_case);
        self.test_cases.push(test_case);
    }

    /// Add a custom test case
    pub fn add_test_case(&mut self, test_case: EvasionTestCase) {
        self.update_stats(&test_case);
        self.test_cases.push(test_case);
    }

    /// Update statistics based on a new test case
    fn update_stats(&mut self, test_case: &EvasionTestCase) {
        match test_case.severity {
            EvasionSeverity::Critical => self.stats.critical_cases += 1,
            EvasionSeverity::High => self.stats.high_cases += 1,
            EvasionSeverity::Medium => self.stats.medium_cases += 1,
            EvasionSeverity::Low => self.stats.low_cases += 1,
        }

        self.stats.total_cases += 1;

        if !self.stats.mutation_types.contains(&test_case.mutation_applied) {
            self.stats.mutation_types.push(test_case.mutation_applied.clone());
        }

        if !self.stats.attack_types.contains(&test_case.original_payload.attack_type) {
            self.stats.attack_types.push(test_case.original_payload.attack_type.clone());
        }
    }

    /// Get all generated test cases
    pub fn test_cases(&self) -> &[EvasionTestCase] {
        &self.test_cases
    }

    /// Get test cases by severity
    pub fn test_cases_by_severity(&self, severity: EvasionSeverity) -> Vec<&EvasionTestCase> {
        self.test_cases
            .iter()
            .filter(|tc| tc.severity == severity)
            .collect()
    }

    /// Get test cases by mutation type
    pub fn test_cases_by_mutation(&self, mutation_type: &str) -> Vec<&EvasionTestCase> {
        self.test_cases
            .iter()
            .filter(|tc| tc.mutation_applied == mutation_type)
            .collect()
    }

    /// Get test cases by attack type
    pub fn test_cases_by_attack(&self, attack_type: &str) -> Vec<&EvasionTestCase> {
        self.test_cases
            .iter()
            .filter(|tc| tc.original_payload.attack_type == attack_type)
            .collect()
    }

    /// Get statistics about generated tests
    pub fn stats(&self) -> &TestGeneratorStats {
        &self.stats
    }

    /// Calculate overall evasion rate
    pub fn evasion_rate(&self) -> f32 {
        if self.test_cases.is_empty() {
            return 0.0;
        }

        let critical_weight = 1.0;
        let high_weight = 0.75;
        let medium_weight = 0.5;
        let low_weight = 0.25;

        let weighted_sum = self.stats.critical_cases as f32 * critical_weight
            + self.stats.high_cases as f32 * high_weight
            + self.stats.medium_cases as f32 * medium_weight
            + self.stats.low_cases as f32 * low_weight;

        weighted_sum / self.stats.total_cases as f32
    }

    /// Generate a test report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Test Generator Report\n");
        report.push_str("=======================\n\n");

        report.push_str(&format!("Total test cases: {}\n", self.stats.total_cases));
        report.push_str(&format!("Critical: {}\n", self.stats.critical_cases));
        report.push_str(&format!("High: {}\n", self.stats.high_cases));
        report.push_str(&format!("Medium: {}\n", self.stats.medium_cases));
        report.push_str(&format!("Low: {}\n", self.stats.low_cases));
        report.push_str(&format!("\nWeighted evasion rate: {:.1}%\n", self.evasion_rate() * 100.0));

        report.push_str(&format!(
            "\nMutation types covered: {}\n",
            self.stats.mutation_types.join(", ")
        ));
        report.push_str(&format!(
            "Attack types covered: {}\n",
            self.stats.attack_types.join(", ")
        ));

        if !self.test_cases.is_empty() {
            report.push_str("\n--- Test Cases ---\n");
            for tc in &self.test_cases {
                report.push_str(&format!(
                    "\n[{}] {}\n",
                    tc.severity.as_str().to_uppercase(),
                    tc.name
                ));
                report.push_str(&format!("  Description: {}\n", tc.description));
                report.push_str(&format!("  Mutation: {}\n", tc.mutation_applied));
                report.push_str(&format!(
                    "  Detectors evaded: {:?}\n",
                    tc.detectors_evaded
                ));
            }
        }

        report
    }

    /// Generate Rust test code for all test cases
    pub fn generate_test_code(&self) -> String {
        let mut code = String::new();
        code.push_str("//! Auto-generated adversarial test cases\n\n");
        code.push_str("use glassware_core::scan;\n\n");

        for tc in &self.test_cases {
            code.push_str(&tc.generate_test_snippet());
            code.push_str("\n\n");
        }

        code
    }

    /// Export test cases to JSON format
    #[cfg(feature = "serde")]
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.test_cases)
    }

    /// Clear all test cases
    pub fn clear(&mut self) {
        self.test_cases.clear();
        self.stats = TestGeneratorStats::default();
    }
}

impl Default for TestGenerator {
    fn default() -> Self {
        Self::new(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_payload(content: &str, attack_type: &str) -> MaliciousPayload {
        MaliciousPayload::new(
            content.to_string(),
            "test.js".to_string(),
            vec![],
            attack_type.to_string(),
        )
    }

    #[test]
    fn test_evasion_severity_from_detectors() {
        assert_eq!(
            EvasionSeverity::from_detectors_evaded(4, 4),
            EvasionSeverity::Critical
        );
        assert_eq!(
            EvasionSeverity::from_detectors_evaded(4, 3),
            EvasionSeverity::High
        );
        assert_eq!(
            EvasionSeverity::from_detectors_evaded(4, 2),
            EvasionSeverity::Medium
        );
        assert_eq!(
            EvasionSeverity::from_detectors_evaded(4, 1),
            EvasionSeverity::Low
        );
    }

    #[test]
    fn test_evasion_severity_display() {
        assert_eq!(format!("{}", EvasionSeverity::Critical), "critical");
        assert_eq!(format!("{}", EvasionSeverity::High), "high");
        assert_eq!(format!("{}", EvasionSeverity::Medium), "medium");
        assert_eq!(format!("{}", EvasionSeverity::Low), "low");
    }

    #[test]
    fn test_evasion_test_case_creation() {
        let original = create_test_payload("const test = 1;", "invisible_char");
        let mutated = create_test_payload("const test\u{FE0F} = 1;", "invisible_char");

        let test_case = EvasionTestCase::from_mutation(
            &original,
            &mutated,
            "unicode_substitution",
            vec!["InvisibleCharDetector".to_string()],
            4,
        );

        assert!(test_case.name.starts_with("evasion_unicode_substitution"));
        assert_eq!(test_case.mutation_applied, "unicode_substitution");
        assert_eq!(test_case.detectors_evaded.len(), 1);
        assert_eq!(test_case.severity, EvasionSeverity::Low);
    }

    #[test]
    fn test_evasion_test_case_critical() {
        let original = create_test_payload("const test = 1;", "invisible_char");
        let mutated = create_test_payload("const test\u{FE0F} = 1;", "invisible_char");

        let test_case = EvasionTestCase::from_mutation(
            &original,
            &mutated,
            "unicode_substitution",
            vec![
                "InvisibleCharDetector".to_string(),
                "GlasswareDetector".to_string(),
                "SemanticDetector".to_string(),
                "LLMAnalyzer".to_string(),
            ],
            4,
        );

        assert!(test_case.is_critical());
        assert_eq!(test_case.severity, EvasionSeverity::Critical);
    }

    #[test]
    fn test_test_generator_creation() {
        let generator = TestGenerator::new(4);
        assert_eq!(generator.test_cases().len(), 0);
        assert_eq!(generator.stats().total_cases, 0);
    }

    #[test]
    fn test_test_generator_add_evasion() {
        let mut generator = TestGenerator::new(4);
        let original = create_test_payload("const test = 1;", "invisible_char");
        let mutated = create_test_payload("const test\u{FE0F} = 1;", "invisible_char");

        generator.add_evasion(
            &original,
            &mutated,
            "unicode_substitution",
            vec!["InvisibleCharDetector".to_string()],
        );

        assert_eq!(generator.test_cases().len(), 1);
        assert_eq!(generator.stats().total_cases, 1);
        assert_eq!(generator.stats().low_cases, 1);
    }

    #[test]
    fn test_test_generator_filtering() {
        let mut generator = TestGenerator::new(4);

        let original1 = create_test_payload("const test = 1;", "invisible_char");
        let mutated1 = create_test_payload("const test\u{FE0F} = 1;", "invisible_char");
        generator.add_evasion(
            &original1,
            &mutated1,
            "unicode_substitution",
            vec!["InvisibleCharDetector".to_string()],
        );

        let original2 = create_test_payload("const test = 1;", "stegano");
        let mutated2 = create_test_payload("const test\u{FE0F}\u{FE0E} = 1;", "stegano");
        generator.add_evasion(
            &original2,
            &mutated2,
            "unicode_substitution",
            vec![
                "InvisibleCharDetector".to_string(),
                "GlasswareDetector".to_string(),
                "SemanticDetector".to_string(),
                "LLMAnalyzer".to_string(),
            ],
        );

        let low_tests = generator.test_cases_by_severity(EvasionSeverity::Low);
        let critical_tests = generator.test_cases_by_severity(EvasionSeverity::Critical);

        assert_eq!(low_tests.len(), 1);
        assert_eq!(critical_tests.len(), 1);

        let unicode_tests = generator.test_cases_by_mutation("unicode_substitution");
        assert_eq!(unicode_tests.len(), 2);
    }

    #[test]
    fn test_test_generator_evasion_rate() {
        let mut generator = TestGenerator::new(4);

        let original1 = create_test_payload("const test = 1;", "invisible_char");
        let mutated1 = create_test_payload("const test\u{FE0F} = 1;", "invisible_char");
        generator.add_evasion(
            &original1,
            &mutated1,
            "unicode_substitution",
            vec![
                "InvisibleCharDetector".to_string(),
                "GlasswareDetector".to_string(),
                "SemanticDetector".to_string(),
                "LLMAnalyzer".to_string(),
            ],
        );

        let original2 = create_test_payload("const test = 1;", "invisible_char");
        let mutated2 = create_test_payload("const test\u{FE0E} = 1;", "invisible_char");
        generator.add_evasion(
            &original2,
            &mutated2,
            "unicode_substitution",
            vec!["InvisibleCharDetector".to_string()],
        );

        let rate = generator.evasion_rate();
        assert!((rate - 0.625).abs() < 0.01);
    }

    #[test]
    fn test_test_generator_report() {
        let mut generator = TestGenerator::new(4);
        let original = create_test_payload("const test = 1;", "invisible_char");
        let mutated = create_test_payload("const test\u{FE0F} = 1;", "invisible_char");

        generator.add_evasion(
            &original,
            &mutated,
            "unicode_substitution",
            vec!["InvisibleCharDetector".to_string()],
        );

        let report = generator.generate_report();
        assert!(report.contains("Total test cases: 1"));
        assert!(report.contains("unicode_substitution"));
        assert!(report.contains("invisible_char"));
    }

    #[test]
    fn test_test_generator_test_snippet() {
        let original = create_test_payload("const test = 1;", "invisible_char");
        let mutated = create_test_payload("const test\u{FE0F} = 1;", "invisible_char");

        let test_case = EvasionTestCase::from_mutation(
            &original,
            &mutated,
            "unicode_substitution",
            vec!["InvisibleCharDetector".to_string()],
            4,
        );

        let snippet = test_case.generate_test_snippet();
        assert!(snippet.contains("fn test_evasion_unicode_substitution"));
        assert!(snippet.contains("InvisibleCharDetector"));
        assert!(snippet.contains("assert!"));
    }

    #[test]
    fn test_test_generator_clear() {
        let mut generator = TestGenerator::new(4);
        let original = create_test_payload("const test = 1;", "invisible_char");
        let mutated = create_test_payload("const test\u{FE0F} = 1;", "invisible_char");

        generator.add_evasion(
            &original,
            &mutated,
            "unicode_substitution",
            vec!["InvisibleCharDetector".to_string()],
        );

        assert_eq!(generator.test_cases().len(), 1);

        generator.clear();

        assert_eq!(generator.test_cases().len(), 0);
        assert_eq!(generator.stats().total_cases, 0);
    }

    #[test]
    fn test_test_generator_stats() {
        let mut generator = TestGenerator::new(4);

        for i in 0..4 {
            let original = create_test_payload("const test = 1;", "invisible_char");
            let mutated = create_test_payload(&format!("const test{} = 1;", i), "invisible_char");
            let detectors_evaded = vec!["Detector".to_string(); (i + 1) as usize];

            generator.add_evasion(&original, &mutated, "unicode_substitution", detectors_evaded);
        }

        let stats = generator.stats();
        assert_eq!(stats.total_cases, 4);
        assert_eq!(stats.critical_cases, 1);
        assert_eq!(stats.high_cases, 1);
        assert_eq!(stats.medium_cases, 1);
        assert_eq!(stats.low_cases, 1);
    }
}
