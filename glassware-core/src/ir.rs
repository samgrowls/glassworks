//! Unified Intermediate Representation (IR) Layer
//!
//! This module provides a unified representation of file content that is parsed once
//! and consumed by all detectors, eliminating redundant parsing work.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      FileIR Builder                          │
//! │  - Reads raw content once                                    │
//! │  - Parses JSON (if package.json)                             │
//! │  - Parses AST (if JS/TS)                                     │
//! │  - Analyzes Unicode (all files)                              │
//! │  - Extracts metadata                                         │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                        FileIR                                │
//! │  - content: String                                           │
//! │  - lines: Vec<String>                                        │
//! │  - json: Option<Value>                                       │
//! │  - ast: Option<JavaScriptAST>                                │
//! │  - unicode: UnicodeAnalysis                                  │
//! │  - metadata: FileMetadata                                    │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!              ┌───────────────┼───────────────┐
//!              ▼               ▼               ▼
//!     ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
//!     │  L1: Regex  │  │ L2: Semantic│  │ L3: LLM     │
//!     │  Detectors  │  │  Detectors  │  │  Review     │
//!     └─────────────┘  └─────────────┘  └─────────────┘
//! ```
//!
//! ## Benefits
//!
//! - **Performance**: Parse once, use many times (20-30% improvement expected)
//! - **Consistency**: All detectors see the same parsed representation
//! - **Modularity**: Detectors focus on detection logic, not parsing
//! - **Extensibility**: Easy to add new analysis layers to the IR

#[cfg(feature = "semantic")]
use oxc_allocator::Allocator;
#[cfg(feature = "semantic")]
use oxc_ast::ast::Program;
#[cfg(feature = "semantic")]
use oxc_parser::Parser;
#[cfg(feature = "semantic")]
use oxc_span::SourceType;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde_json")]
use serde_json::Value;
use std::path::Path;
use std::sync::Arc;

/// Unicode analysis results for a file
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UnicodeAnalysis {
    /// Whether the file contains invisible characters
    pub has_invisible: bool,
    /// Whether the file contains bidirectional overrides
    pub has_bidi: bool,
    /// Positions of invisible characters (byte offset, char)
    pub invisible_positions: Vec<(usize, char)>,
    /// Positions of bidirectional characters (byte offset, char)
    pub bidi_positions: Vec<(usize, char)>,
    /// Whether the file contains homoglyphs
    pub has_homoglyphs: bool,
    /// Whether the file contains Unicode tag characters
    pub has_unicode_tags: bool,
}

impl UnicodeAnalysis {
    /// Analyze Unicode characteristics of content
    pub fn analyze(content: &str) -> Self {
        let mut invisible_positions = Vec::new();
        let mut bidi_positions = Vec::new();
        let mut has_homoglyphs = false;
        let mut has_unicode_tags = false;

        for (idx, ch) in content.char_indices() {
            let code_point = ch as u32;
            
            // Check for invisible characters (variation selectors, zero-width, etc.)
            if is_in_invisible_range_simple(code_point) {
                invisible_positions.push((idx, ch));
            }

            // Check for bidirectional characters
            if is_bidi_char(ch) {
                bidi_positions.push((idx, ch));
            }

            // Check for Unicode tag characters (U+E0001–U+E007F)
            if matches!(ch, '\u{E0001}'..='\u{E007F}') {
                has_unicode_tags = true;
            }

            // Check for potential homoglyphs (Cyrillic/Greek in Latin context)
            // This is a simplified check - full homoglyph detection is more complex
            if is_potential_homoglyph(ch) {
                has_homoglyphs = true;
            }
        }

        let has_invisible = !invisible_positions.is_empty();
        let has_bidi = !bidi_positions.is_empty();

        Self {
            has_invisible,
            has_bidi,
            invisible_positions,
            bidi_positions,
            has_homoglyphs,
            has_unicode_tags,
        }
    }

    /// Check if analysis found any suspicious Unicode patterns
    pub fn is_suspicious(&self) -> bool {
        self.has_invisible || self.has_bidi || self.has_homoglyphs || self.has_unicode_tags
    }
}

/// Check if a character is a potential homoglyph
fn is_potential_homoglyph(ch: char) -> bool {
    // Cyrillic characters that look like Latin
    const CYRILLIC_HOMOGlyphS: &[char] = &[
        '\u{0430}', // а (Cyrillic a)
        '\u{0435}', // е (Cyrillic e)
        '\u{043E}', // о (Cyrillic o)
        '\u{0440}', // р (Cyrillic p)
        '\u{0441}', // с (Cyrillic c)
        '\u{0443}', // у (Cyrillic y)
        '\u{0445}', // х (Cyrillic x)
        '\u{0456}', // і (Cyrillic i)
    ];

    // Greek characters that look like Latin
    const GREEK_HOMOGlyphS: &[char] = &[
        '\u{03B1}', // α (Greek alpha)
        '\u{03B5}', // ε (Greek epsilon)
        '\u{03BF}', // ο (Greek omicron)
        '\u{03C1}', // ρ (Greek rho)
        '\u{03C4}', // τ (Greek tau)
        '\u{03C5}', // υ (Greek upsilon)
        '\u{03C7}', // χ (Greek chi)
    ];

    CYRILLIC_HOMOGlyphS.contains(&ch) || GREEK_HOMOGlyphS.contains(&ch)
}

/// Simple check for invisible character ranges
fn is_in_invisible_range_simple(code_point: u32) -> bool {
    matches!(code_point,
        0xFE00..=0xFE0F |  // Variation Selectors
        0xE0100..=0xE01EF |  // Variation Selectors Supplement
        0x200B..=0x200F |  // Zero-width space, joiner, non-joiner
        0x2060..=0x206F |  // Word joiner, invisible operators
        0xE0000..=0xE007F  // Tags
    )
}

/// Check for bidirectional characters
fn is_bidi_char(ch: char) -> bool {
    matches!(ch,
        '\u{202A}'..='\u{202E}' |  // LRE, RLE, LRO, RLO, PDF
        '\u{2066}'..='\u{2069}' |  // LRI, RLI, FSI, PDI
        '\u{200E}' | '\u{200F}' |  // LRM, RLM
        '\u{061C}'  // ALM
    )
}

/// Check if path is a JS or TS file
fn is_js_or_ts_path(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    matches!(ext, "js" | "mjs" | "cjs" | "jsx" | "ts" | "mts" | "cts" | "tsx")
}

/// JavaScript/TypeScript AST representation
#[cfg(feature = "semantic")]
pub struct JavaScriptAST {
    /// Allocator for AST nodes
    pub allocator: Allocator,
    /// Parsed program
    pub program: Program<'static>,
    /// Source type (script, module, TypeScript, etc.)
    pub source_type: SourceType,
    /// Whether parsing succeeded
    pub parse_success: bool,
    /// Parsing errors (if any)
    pub errors: Vec<String>,
}

#[cfg(feature = "semantic")]
impl JavaScriptAST {
    /// Parse JavaScript/TypeScript source code into an AST
    pub fn parse(source_code: &str, path: &Path) -> Option<Self> {
        // Determine source type from file extension
        let source_type = determine_source_type(path);

        // Create allocator for AST
        let allocator = Allocator::default();

        // Parse the source code
        let ret = Parser::new(&allocator, source_code, source_type).parse();

        // Check for parsing errors
        let errors: Vec<String> = ret.errors.iter().map(|e| e.message.to_string()).collect();
        let parse_success = errors.is_empty();

        // Safety: We need to extend the lifetime of the program to 'static
        // This is safe because the allocator owns the memory and lives as long as the AST
        let program = unsafe {
            std::mem::transmute::<Program<'_>, Program<'static>>(ret.program)
        };

        Some(Self {
            allocator,
            program,
            source_type,
            parse_success,
            errors,
        })
    }

    /// Check if the AST is valid for analysis
    pub fn is_valid(&self) -> bool {
        self.parse_success
    }
}

/// Determine OXC source type from file path
#[cfg(feature = "semantic")]
fn determine_source_type(path: &Path) -> SourceType {
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    match extension {
        "ts" => SourceType::from_path(path).unwrap_or(SourceType::default()),
        "tsx" => SourceType::from_path(path).unwrap_or(SourceType::tsx()),
        "mts" => SourceType::from_path(path).unwrap_or(SourceType::tsx()),
        "cts" => SourceType::from_path(path).unwrap_or(SourceType::tsx()),
        "js" | "mjs" | "cjs" | "jsx" => SourceType::from_path(path).unwrap_or(SourceType::default()),
        _ => SourceType::default(),
    }
}

/// File metadata
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FileMetadata {
    /// File path
    pub path: String,
    /// File size in bytes
    pub size: u64,
    /// File extension (lowercase, without dot)
    pub extension: String,
    /// Whether the file appears to be minified
    pub is_minified: bool,
    /// Whether the file appears to be bundled
    pub is_bundled: bool,
    /// Detected language (JS, TS, JSON, etc.)
    pub language: Language,
    /// Raw binary data (for .node files and other binaries)
    #[cfg_attr(feature = "serde", serde(skip_serializing, default))]
    pub data: Vec<u8>,
}

/// Programming language detection
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Language {
    /// JavaScript
    JavaScript,
    /// TypeScript
    TypeScript,
    /// TypeScript JSX
    TypeScriptReact,
    /// JSON
    Json,
    /// Python
    Python,
    /// Rust
    Rust,
    /// Go
    Go,
    /// Unknown/other
    Unknown,
}

impl Language {
    /// Detect language from file extension
    pub fn from_extension(extension: &str) -> Self {
        match extension {
            "js" | "mjs" | "cjs" | "jsx" => Language::JavaScript,
            "ts" | "mts" | "cts" => Language::TypeScript,
            "tsx" => Language::TypeScriptReact,
            "json" => Language::Json,
            "py" => Language::Python,
            "rs" => Language::Rust,
            "go" => Language::Go,
            _ => Language::Unknown,
        }
    }

    /// Check if this language supports semantic analysis
    pub fn supports_semantic_analysis(&self) -> bool {
        matches!(
            self,
            Language::JavaScript | Language::TypeScript | Language::TypeScriptReact
        )
    }
}

impl FileMetadata {
    /// Extract metadata from file path and content
    pub fn new(path: &Path, content: &str) -> Self {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let size = content.len() as u64;
        let language = Language::from_extension(&extension);

        // Detect minified code
        let is_minified = detect_minified(content);

        // Detect bundled code
        let is_bundled = detect_bundled(path, content);

        Self {
            path: path.to_string_lossy().to_string(),
            size,
            extension,
            is_minified,
            is_bundled,
            language,
            data: Vec::new(),
        }
    }

    /// Create metadata with binary data (for .node files)
    pub fn with_data(path: &Path, content: &str, data: Vec<u8>) -> Self {
        let mut metadata = Self::new(path, content);
        metadata.size = data.len() as u64;
        metadata.data = data;
        metadata
    }

    /// Check if this file is a package.json
    pub fn is_package_json(&self) -> bool {
        self.extension == "json" && self.path.ends_with("package.json")
    }

    /// Check if this file is JavaScript or TypeScript
    pub fn is_js_or_ts(&self) -> bool {
        self.language.supports_semantic_analysis()
    }
}

/// Detect if content appears to be minified
fn detect_minified(content: &str) -> bool {
    // Heuristics for minified code:
    // 1. Very long lines (> 500 chars on average)
    // 2. Few newlines relative to content size
    // 3. No whitespace after certain tokens

    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return false;
    }

    // Check average line length
    let avg_line_length = content.len() / lines.len();
    if avg_line_length > 500 {
        return true;
    }

    // Check for single-line code
    if lines.len() == 1 && content.len() > 1000 {
        return true;
    }

    // Check for minification patterns
    content.contains("function(e,t,n){") ||
    content.contains("function(t,e,n){") ||
    content.contains("(function(e){") ||
    content.contains("!function(e){") ||
    content.contains("var r={}") ||
    content.contains("var n={}")
}

/// Detect if content appears to be bundled code
fn detect_bundled(path: &Path, content: &str) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();

    // Check path patterns
    let is_bundled_path = path_str.contains("/dist/")
        || path_str.contains("/build/")
        || path_str.contains("/bin/")
        || path_str.ends_with(".mjs")
        || path_str.ends_with(".cjs")
        || path_str.contains(".min.")
        || path_str.contains(".bundle.")
        || path_str.contains(".umd.");

    if is_bundled_path {
        return true;
    }

    // Check content patterns
    content.contains("/* webpack") ||
    content.contains("/* rollup") ||
    content.contains("/* parcel") ||
    content.contains("/*! For license information") ||
    content.contains("__webpack_require__") ||
    content.contains("__ROLLUP__")
}

/// Unified Intermediate Representation for a file
///
/// This struct contains all parsed representations of a file,
/// built once and consumed by multiple detectors.
#[derive(Clone)]
pub struct FileIR {
    /// Raw file content
    pub content: Arc<String>,

    /// Lines (for line-based detectors)
    pub lines: Vec<String>,

    /// Parsed JSON (for package.json, etc.)
    pub json: Option<Arc<serde_json::Value>>,

    /// Parsed AST (for JS/TS files)
    pub ast: Option<Arc<JavaScriptAST>>,

    /// Unicode analysis (for invisible char detection)
    pub unicode: Arc<UnicodeAnalysis>,

    /// File metadata
    pub metadata: FileMetadata,
}

impl FileIR {
    /// Build IR from file path and content
    ///
    /// This is the main entry point for creating IR. It performs:
    /// 1. Line splitting
    /// 2. JSON parsing (if applicable)
    /// 3. AST parsing (if JS/TS)
    /// 4. Unicode analysis
    /// 5. Metadata extraction
    ///
    /// # Arguments
    /// * `path` - Path to the file
    /// * `content` - File content as string
    ///
    /// # Returns
    /// A fully constructed FileIR instance
    pub fn build(path: &Path, content: &str) -> Self {
        // Split into lines (preserving original content)
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        // Parse JSON if applicable
        #[cfg(feature = "serde_json")]
        let json = if path.file_name().map_or(false, |n| n == "package.json") {
            serde_json::from_str(content).ok().map(Arc::new)
        } else {
            None
        };

        #[cfg(not(feature = "serde_json"))]
        let json: Option<Arc<Value>> = None;

        // Parse JS/TS AST if applicable (check extension first)
        #[cfg(feature = "semantic")]
        let ast = if is_js_or_ts_path(path) {
            JavaScriptAST::parse(content, path).map(Arc::new)
        } else {
            None
        };

        #[cfg(not(feature = "semantic"))]
        let ast: Option<Arc<JavaScriptAST>> = None;

        // Analyze Unicode (always)
        let unicode = Arc::new(UnicodeAnalysis::analyze(content));

        // Extract metadata
        let metadata = FileMetadata::new(path, content);

        Self {
            content: Arc::new(content.to_string()),
            lines,
            json,
            ast,
            unicode,
            metadata,
        }
    }

    /// Build IR from file path, content, and binary data
    ///
    /// This is used for .node files and other binary formats.
    ///
    /// # Arguments
    /// * `path` - Path to the file
    /// * `content` - File content as string (may be empty for pure binaries)
    /// * `data` - Raw binary data
    ///
    /// # Returns
    /// A fully constructed FileIR instance with binary data
    pub fn build_with_data(path: &Path, content: &str, data: Vec<u8>) -> Self {
        // Split into lines (preserving original content)
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        // No JSON for binary files
        #[cfg(feature = "serde_json")]
        let json: Option<Arc<Value>> = None;

        #[cfg(not(feature = "serde_json"))]
        let json: Option<Arc<Value>> = None;

        // No AST for binary files
        #[cfg(feature = "semantic")]
        let ast: Option<Arc<JavaScriptAST>> = None;

        #[cfg(not(feature = "semantic"))]
        let ast: Option<Arc<JavaScriptAST>> = None;

        // Analyze Unicode (always)
        let unicode = Arc::new(UnicodeAnalysis::analyze(content));

        // Extract metadata with binary data
        let metadata = FileMetadata::with_data(path, content, data);

        Self {
            content: Arc::new(content.to_string()),
            lines,
            json,
            ast,
            unicode,
            metadata,
        }
    }

    /// Get the raw content as a string
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Get a specific line by index (0-based)
    pub fn line(&self, index: usize) -> Option<&str> {
        self.lines.get(index).map(|s| s.as_str())
    }

    /// Get the number of lines
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Get JSON value if available
    #[cfg(feature = "serde_json")]
    pub fn json(&self) -> Option<&Value> {
        self.json.as_ref().map(|v| v.as_ref())
    }

    /// Get AST if available
    #[cfg(feature = "semantic")]
    pub fn ast(&self) -> Option<&JavaScriptAST> {
        self.ast.as_ref().map(|a| a.as_ref())
    }

    /// Get Unicode analysis
    pub fn unicode(&self) -> &UnicodeAnalysis {
        &self.unicode
    }

    /// Get binary data if available (for .node files)
    pub fn data(&self) -> &[u8] {
        &self.metadata.data
    }

    /// Check if this file has binary data
    pub fn has_binary_data(&self) -> bool {
        !self.metadata.data.is_empty()
    }

    /// Get file metadata
    pub fn metadata(&self) -> &FileMetadata {
        &self.metadata
    }

    /// Check if file is minified
    pub fn is_minified(&self) -> bool {
        self.metadata.is_minified
    }

    /// Check if file is bundled
    pub fn is_bundled(&self) -> bool {
        self.metadata.is_bundled
    }

    /// Check if file is package.json
    pub fn is_package_json(&self) -> bool {
        self.metadata.is_package_json()
    }

    /// Check if file is JS/TS
    pub fn is_js_or_ts(&self) -> bool {
        self.metadata.is_js_or_ts()
    }
}

impl std::fmt::Debug for FileIR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileIR")
            .field("content_length", &self.content.len())
            .field("line_count", &self.lines.len())
            .field("has_json", &self.json.is_some())
            .field("has_ast", &self.ast.is_some())
            .field("unicode", &self.unicode)
            .field("metadata", &self.metadata)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fileir_build_basic() {
        let content = "const x = 1;\nconst y = 2;\n";
        let path = Path::new("test.js");

        let ir = FileIR::build(path, content);

        assert_eq!(ir.content(), content);
        assert_eq!(ir.line_count(), 2);
        assert_eq!(ir.line(0), Some("const x = 1;"));
        assert_eq!(ir.line(1), Some("const y = 2;"));
        assert!(ir.is_js_or_ts());
        assert!(!ir.is_minified());
    }

    #[test]
    fn test_fileir_unicode_analysis() {
        let content = "const secret\u{FE00}Key = 'value';";
        let path = Path::new("test.js");

        let ir = FileIR::build(path, content);

        assert!(ir.unicode.has_invisible);
        assert!(!ir.unicode.invisible_positions.is_empty());
        assert!(ir.unicode.is_suspicious());
    }

    #[test]
    fn test_fileir_unicode_bidi() {
        let content = "const file = \"test\u{202E}exe\";";
        let path = Path::new("test.js");

        let ir = FileIR::build(path, content);

        assert!(ir.unicode.has_bidi);
        assert!(!ir.unicode.bidi_positions.is_empty());
    }

    #[test]
    fn test_fileir_clean_content() {
        let content = "const normal = 'hello world';";
        let path = Path::new("test.js");

        let ir = FileIR::build(path, content);

        assert!(!ir.unicode.is_suspicious());
    }

    #[test]
    #[cfg(feature = "serde_json")]
    fn test_fileir_json_parsing() {
        let content = r#"{
            "name": "test-package",
            "version": "1.0.0",
            "dependencies": {
                "lodash": "^4.17.21"
            }
        }"#;
        let path = Path::new("package.json");

        let ir = FileIR::build(path, content);

        assert!(ir.json.is_some());
        assert!(ir.is_package_json());

        let json = ir.json().unwrap();
        assert_eq!(json["name"], "test-package");
        assert_eq!(json["version"], "1.0.0");
    }

    #[test]
    #[cfg(feature = "serde_json")]
    fn test_fileir_non_json() {
        let content = "const x = 1;";
        let path = Path::new("test.js");

        let ir = FileIR::build(path, content);

        assert!(ir.json.is_none());
    }

    #[test]
    fn test_fileir_metadata() {
        let content = "const x = 1;";
        let path = Path::new("test.ts");

        let ir = FileIR::build(path, content);

        assert_eq!(ir.metadata.extension, "ts");
        assert_eq!(ir.metadata.language, Language::TypeScript);
        assert_eq!(ir.metadata.size, content.len() as u64);
    }

    #[test]
    fn test_fileir_minified_detection() {
        let content = "function(e,t,n){var r={};var o=!1;";
        let path = Path::new("bundle.js");

        let ir = FileIR::build(path, content);

        assert!(ir.is_minified());
    }

    #[test]
    fn test_fileir_bundled_detection() {
        let content = "/* webpack */ const x = 1;";
        let path = Path::new("dist/bundle.js");

        let ir = FileIR::build(path, content);

        assert!(ir.is_bundled());
    }

    #[test]
    fn test_language_from_extension() {
        assert_eq!(Language::from_extension("js"), Language::JavaScript);
        assert_eq!(Language::from_extension("mjs"), Language::JavaScript);
        assert_eq!(Language::from_extension("cjs"), Language::JavaScript);
        assert_eq!(Language::from_extension("jsx"), Language::JavaScript);
        assert_eq!(Language::from_extension("ts"), Language::TypeScript);
        assert_eq!(Language::from_extension("mts"), Language::TypeScript);
        assert_eq!(Language::from_extension("cts"), Language::TypeScript);
        assert_eq!(Language::from_extension("tsx"), Language::TypeScriptReact);
        assert_eq!(Language::from_extension("json"), Language::Json);
        assert_eq!(Language::from_extension("py"), Language::Python);
        assert_eq!(Language::from_extension("rs"), Language::Rust);
        assert_eq!(Language::from_extension("go"), Language::Go);
        assert_eq!(Language::from_extension("unknown"), Language::Unknown);
    }

    #[test]
    fn test_language_supports_semantic_analysis() {
        assert!(Language::JavaScript.supports_semantic_analysis());
        assert!(Language::TypeScript.supports_semantic_analysis());
        assert!(Language::TypeScriptReact.supports_semantic_analysis());
        assert!(!Language::Json.supports_semantic_analysis());
        assert!(!Language::Python.supports_semantic_analysis());
        assert!(!Language::Rust.supports_semantic_analysis());
        assert!(!Language::Go.supports_semantic_analysis());
        assert!(!Language::Unknown.supports_semantic_analysis());
    }

    #[test]
    fn test_unicode_analysis_homoglyphs() {
        // Cyrillic 'а' (U+0430) looks like Latin 'a'
        let content = "const pаssword = 'secret';";
        let path = Path::new("test.js");

        let ir = FileIR::build(path, content);

        assert!(ir.unicode.has_homoglyphs);
    }

    #[test]
    fn test_unicode_analysis_unicode_tags() {
        let content = "const tag\u{E0001}ged = 'value';";
        let path = Path::new("test.js");

        let ir = FileIR::build(path, content);

        assert!(ir.unicode.has_unicode_tags);
    }
}
