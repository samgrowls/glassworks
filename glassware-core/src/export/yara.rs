//! E5: YARA Rule Export
//!
//! Generates YARA rules from detection patterns that are string/byte-based.
//!
//! ## Exportable Detectors
//!
//! Good candidates for YARA export:
//! - G3: Typo fingerprints (exact strings)
//! - G7: IElevator CLSIDs (exact strings)
//! - G9: memexec typo (exact string)
//! - G4: Exfil schema keys (string patterns)
//! - G11: PDB paths (string patterns)
//!
//! NOT suitable for YARA:
//! - G6: XorShift (heuristic-based, entropy analysis)
//! - G8: APC injection (import table analysis)
//! - Behavioral detectors requiring semantic analysis
//!
//! ## YARA Rule Format
//!
//! ```yara
//! rule GlassWorm_TypoFingerprints {
//!     meta:
//!         author = "glassware-core"
//!         description = "GlassWorm typo fingerprints"
//!         severity = "high"
//!         reference = "https://codeberg.org/tip-o-deincognito/glassworm-writeup"
//!         date = "2026-03-21"
//!     strings:
//!         $typo1 = "LoadLibararyFail"
//!         $typo2 = "Invlaid"
//!         $typo3 = "NtAllocVmErr"
//!     condition:
//!         any of them
//! }
//! ```

use crate::finding::Severity;
use std::fmt::Write;

/// A single YARA rule
#[derive(Debug, Clone)]
pub struct YaraRule {
    /// Rule name (e.g., "GlassWorm_TypoFingerprints")
    pub name: String,
    /// Rule description
    pub description: String,
    /// YARA string patterns
    pub strings: Vec<YaraString>,
    /// Rule condition
    pub condition: String,
    /// Metadata (author, severity, etc.)
    pub metadata: YaraMetadata,
}

/// A YARA string pattern
#[derive(Debug, Clone)]
pub struct YaraString {
    /// Identifier (e.g., "$typo1")
    pub identifier: String,
    /// String value or hex pattern
    pub value: String,
    /// Whether this is a hex pattern
    pub is_hex: bool,
}

/// YARA rule metadata
#[derive(Debug, Clone)]
pub struct YaraMetadata {
    /// Author
    pub author: String,
    /// Description
    pub description: String,
    /// Severity level
    pub severity: String,
    /// Reference URL
    pub reference: String,
    /// Date created
    pub date: String,
}

impl YaraMetadata {
    /// Create metadata with defaults
    pub fn new(detector_name: &str, severity: Severity) -> Self {
        Self {
            author: "glassware-core".to_string(),
            description: format!("GlassWorm {}", detector_name),
            severity: severity.as_str().to_string(),
            reference: "https://codeberg.org/tip-o-deincognito/glassworm-writeup".to_string(),
            date: "2026-03-21".to_string(),
        }
    }
}

/// YARA rule exporter
pub struct YaraExporter {
    /// Rules to export
    rules: Vec<YaraRule>,
}

impl YaraExporter {
    /// Create a new YARA exporter
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }

    /// Add a rule to export
    pub fn add_rule(&mut self, rule: YaraRule) {
        self.rules.push(rule);
    }

    /// Generate all YARA rules as a single string
    pub fn generate(&self) -> String {
        let mut output = String::new();

        for (i, rule) in self.rules.iter().enumerate() {
            if i > 0 {
                output.push_str("\n\n");
            }
            output.push_str(&self.format_rule(rule));
        }

        output
    }

    /// Format a single YARA rule
    fn format_rule(&self, rule: &YaraRule) -> String {
        let mut output = String::new();

        // Rule header
        writeln!(output, "rule {} {{", rule.name).unwrap();

        // Meta section
        writeln!(output, "    meta:").unwrap();
        writeln!(output, "        author = \"{}\"", rule.metadata.author).unwrap();
        writeln!(output, "        description = \"{}\"", rule.metadata.description).unwrap();
        writeln!(output, "        severity = \"{}\"", rule.metadata.severity).unwrap();
        writeln!(output, "        reference = \"{}\"", rule.metadata.reference).unwrap();
        writeln!(output, "        date = \"{}\"", rule.metadata.date).unwrap();

        // Strings section
        writeln!(output, "    strings:").unwrap();
        for yara_str in &rule.strings {
            if yara_str.is_hex {
                writeln!(output, "        {} = {}", yara_str.identifier, yara_str.value).unwrap();
            } else {
                writeln!(output, "        {} = \"{}\"", yara_str.identifier, yara_str.value).unwrap();
            }
        }

        // Condition section
        writeln!(output, "    condition:").unwrap();
        writeln!(output, "        {}", rule.condition).unwrap();

        // Rule footer
        output.push('}');

        output
    }

    /// Export typo fingerprint rules (G3, G9)
    pub fn export_typo_fingerprints(&mut self) {
        let rule = YaraRule {
            name: "GlassWorm_TypoFingerprints".to_string(),
            description: "GlassWorm typo fingerprints from source code".to_string(),
            strings: vec![
                YaraString {
                    identifier: "$typo_memexec".to_string(),
                    value: "LoadLibararyFail".to_string(),
                    is_hex: false,
                },
                YaraString {
                    identifier: "$typo_invlaid".to_string(),
                    value: "Invlaid".to_string(),
                    is_hex: false,
                },
                YaraString {
                    identifier: "$typo_ntalloc".to_string(),
                    value: "NtAllocVmErr".to_string(),
                    is_hex: false,
                },
            ],
            condition: "any of them".to_string(),
            metadata: YaraMetadata::new("Typo Fingerprints", Severity::High),
        };
        self.add_rule(rule);
    }

    /// Export IElevator CLSID rules (G7)
    pub fn export_ielevator_clsids(&mut self) {
        let rule = YaraRule {
            name: "GlassWorm_IElevator_CLSIDs".to_string(),
            description: "IElevator COM CLSIDs for App-Bound key extraction".to_string(),
            strings: vec![
                YaraString {
                    identifier: "$clsid_chrome".to_string(),
                    value: "{708860E0-F641-4611-8895-7D867DD3675B}".to_string(),
                    is_hex: false,
                },
                YaraString {
                    identifier: "$clsid_edge".to_string(),
                    value: "{576B31AF-6369-4B6B-8560-E4B203A97A8B}".to_string(),
                    is_hex: false,
                },
                YaraString {
                    identifier: "$clsid_brave".to_string(),
                    value: "{1FCBE96C-1697-43AF-9140-2897C7C69767}".to_string(),
                    is_hex: false,
                },
            ],
            condition: "any of them".to_string(),
            metadata: YaraMetadata::new("IElevator CLSIDs", Severity::Critical),
        };
        self.add_rule(rule);
    }

    /// Export exfil schema key rules (G4)
    pub fn export_exfil_schema(&mut self) {
        let rule = YaraRule {
            name: "GlassWorm_Exfil_Schema".to_string(),
            description: "GlassWorm exfil JSON schema keys".to_string(),
            strings: vec![
                // High-signal keys
                YaraString {
                    identifier: "$exfil_sync_oauth".to_string(),
                    value: "sync_oauth_token".to_string(),
                    is_hex: false,
                },
                YaraString {
                    identifier: "$exfil_wallet".to_string(),
                    value: "walletCount".to_string(),
                    is_hex: false,
                },
                YaraString {
                    identifier: "$exfil_credit".to_string(),
                    value: "creditCardCount".to_string(),
                    is_hex: false,
                },
                // Medium-signal keys
                YaraString {
                    identifier: "$exfil_master_key".to_string(),
                    value: "master_key".to_string(),
                    is_hex: false,
                },
                YaraString {
                    identifier: "$exfil_session".to_string(),
                    value: "session_token".to_string(),
                    is_hex: false,
                },
            ],
            condition: "any of them".to_string(),
            metadata: YaraMetadata::new("Exfil Schema", Severity::High),
        };
        self.add_rule(rule);
    }

    /// Export PDB path rules (G11)
    pub fn export_pdb_paths(&mut self) {
        let rule = YaraRule {
            name: "GlassWorm_PDB_Paths".to_string(),
            description: "GlassWorm build PDB paths for attribution".to_string(),
            strings: vec![
                YaraString {
                    identifier: "$pdb_work".to_string(),
                    value: "N:\\\\work\\\\chrome_current".to_string(),
                    is_hex: false,
                },
                YaraString {
                    identifier: "$pdb_dump".to_string(),
                    value: "DumpBrowserSecrets".to_string(),
                    is_hex: false,
                },
                YaraString {
                    identifier: "$pdb_sideloader".to_string(),
                    value: "ext_sideloader.pdb".to_string(),
                    is_hex: false,
                },
                YaraString {
                    identifier: "$pdb_admin".to_string(),
                    value: "C:\\\\Users\\\\Administrator\\\\.cargo".to_string(),
                    is_hex: false,
                },
            ],
            condition: "any of them".to_string(),
            metadata: YaraMetadata::new("PDB Paths", Severity::Medium),
        };
        self.add_rule(rule);
    }

    /// Export all exportable rules
    pub fn export_all(&mut self) {
        self.export_typo_fingerprints();
        self.export_ielevator_clsids();
        self.export_exfil_schema();
        self.export_pdb_paths();
    }

    /// Get the number of rules
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

impl Default for YaraExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Export all YARA rules
pub fn export_yara_rules() -> String {
    let mut exporter = YaraExporter::new();
    exporter.export_all();
    exporter.generate()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exporter_creation() {
        let exporter = YaraExporter::new();
        assert_eq!(exporter.rule_count(), 0);
    }

    #[test]
    fn test_export_typo_fingerprints() {
        let mut exporter = YaraExporter::new();
        exporter.export_typo_fingerprints();

        assert_eq!(exporter.rule_count(), 1);
        let output = exporter.generate();

        // Check required sections
        assert!(output.contains("rule GlassWorm_TypoFingerprints"));
        assert!(output.contains("meta:"));
        assert!(output.contains("strings:"));
        assert!(output.contains("condition:"));
        assert!(output.contains("LoadLibararyFail"));
        assert!(output.contains("Invlaid"));
        assert!(output.contains("NtAllocVmErr"));
    }

    #[test]
    fn test_export_ielevator_clsids() {
        let mut exporter = YaraExporter::new();
        exporter.export_ielevator_clsids();

        assert_eq!(exporter.rule_count(), 1);
        let output = exporter.generate();

        assert!(output.contains("rule GlassWorm_IElevator_CLSIDs"));
        assert!(output.contains("{708860E0-F641-4611-8895-7D867DD3675B}"));
        assert!(output.contains("{576B31AF-6369-4B6B-8560-E4B203A97A8B}"));
        assert!(output.contains("{1FCBE96C-1697-43AF-9140-2897C7C69767}"));
    }

    #[test]
    fn test_export_all_rules() {
        let mut exporter = YaraExporter::new();
        exporter.export_all();

        assert_eq!(exporter.rule_count(), 4);
        let output = exporter.generate();

        // Check all rules are present
        assert!(output.contains("GlassWorm_TypoFingerprints"));
        assert!(output.contains("GlassWorm_IElevator_CLSIDs"));
        assert!(output.contains("GlassWorm_Exfil_Schema"));
        assert!(output.contains("GlassWorm_PDB_Paths"));
    }

    #[test]
    fn test_yara_syntax_validity() {
        let mut exporter = YaraExporter::new();
        exporter.export_all();
        let output = exporter.generate();

        // Basic syntax validation
        // Each rule should have matching braces
        let open_braces = output.matches('{').count();
        let close_braces = output.matches('}').count();
        assert_eq!(open_braces, close_braces);

        // Each rule should have required sections
        let rule_count = output.matches("rule ").count();
        let meta_count = output.matches("meta:").count();
        let strings_count = output.matches("strings:").count();
        let condition_count = output.matches("condition:").count();

        assert_eq!(rule_count, meta_count);
        assert_eq!(rule_count, strings_count);
        assert_eq!(rule_count, condition_count);
    }

    #[test]
    fn test_metadata_included() {
        let mut exporter = YaraExporter::new();
        exporter.export_typo_fingerprints();
        let output = exporter.generate();

        assert!(output.contains("author = \"glassware-core\""));
        assert!(output.contains("severity = \"high\""));
        assert!(output.contains("reference = \"https://codeberg.org/tip-o-deincognito/glassworm-writeup\""));
    }
}
