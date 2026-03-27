//! Context-Aware File Filter
//!
//! Uses AST analysis to classify files as:
//! - Test files (should skip or downweight)
//! - Data files (should skip or downweight)  
//! - Build output (should skip)
//! - Production code (full detection)

#[cfg(feature = "semantic")]
use crate::semantic::SemanticAnalysis;
use std::path::Path;

/// File classification based on AST analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileClassification {
    /// Test file - skip or downweight
    Test,
    /// Data file - skip or downweight
    Data,
    /// Build output - skip
    BuildOutput,
    /// Production code - full detection
    Production,
}

/// Classify a file based on semantic analysis and path
#[cfg(feature = "semantic")]
pub fn classify_file(analysis: &SemanticAnalysis, path: &Path) -> FileClassification {
    // Check path patterns first (fast)
    if is_test_path(path) {
        return FileClassification::Test;
    }

    if is_build_path(path) {
        return FileClassification::BuildOutput;
    }

    if is_data_path(path) {
        return FileClassification::Data;
    }

    // Check AST patterns
    if is_test_file(analysis) {
        return FileClassification::Test;
    }

    if is_data_file(analysis) {
        return FileClassification::Data;
    }

    if is_build_output(analysis) {
        return FileClassification::BuildOutput;
    }

    FileClassification::Production
}

/// Classify a file based on path only (no AST analysis)
pub fn classify_file_by_path(path: &Path) -> FileClassification {
    if is_test_path(path) {
        return FileClassification::Test;
    }

    if is_build_path(path) {
        return FileClassification::BuildOutput;
    }

    if is_data_path(path) {
        return FileClassification::Data;
    }

    FileClassification::Production
}

/// Check if path indicates a test file
pub fn is_test_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();

    // Check for test file patterns
    // Note: We don't check for /fixtures/ because that would flag test fixture files
    // used in our own test suite, which are actually production code samples
    path_str.contains(".test.") ||
    path_str.contains(".spec.") ||
    path_str.contains("/test/") ||
    path_str.contains("/tests/") ||
    path_str.contains("/__tests__/") ||
    path_str.contains("/__mocks__/") ||
    path_str.ends_with("-test.js") ||
    path_str.ends_with("-test.ts") ||
    path_str.ends_with("-spec.js") ||
    path_str.ends_with("-spec.ts") ||
    path_str.starts_with("test/") ||
    path_str.starts_with("tests/")
}

/// Check if path indicates build output
fn is_build_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();

    // Check for build/dist directories
    path_str.contains("/dist/") ||
    path_str.contains("/build/") ||
    path_str.contains("/out/") ||
    path_str.contains("/bundle/") ||
    path_str.contains("/compiled/") ||
    path_str.contains("/generated/") ||
    path_str.contains(".min.js") ||
    path_str.contains(".min.ts") ||
    path_str.contains(".bundle.js") ||
    path_str.contains(".bundle.ts") ||
    path_str.starts_with("dist/") ||
    path_str.starts_with("build/") ||
    path_str.starts_with("out/")
}

/// Check if path indicates a data file
fn is_data_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();

    // Check for data file patterns
    path_str.contains("/data/") ||
    path_str.contains("/locale/") ||
    path_str.contains("/locales/") ||
    path_str.contains("/i18n/") ||
    path_str.contains("/lang/") ||
    path_str.contains("/languages/") ||
    path_str.contains("all-countries") ||
    path_str.contains("countries.json") ||
    path_str.contains("l10n") ||
    path_str.starts_with("data/") ||
    path_str.starts_with("locale/") ||
    path_str.starts_with("i18n/") ||
    (path_str.ends_with(".json") && (
        path_str.contains("/data/") || 
        path_str.contains("/locale/") ||
        path_str.contains("/i18n/") ||
        path_str.starts_with("data/") ||
        path_str.starts_with("locale/") ||
        path_str.starts_with("i18n/")
    ))
}

/// Check if file is a test file using AST analysis
#[cfg(feature = "semantic")]
pub fn is_test_file(analysis: &SemanticAnalysis) -> bool {
    // Look for test framework calls
    let test_calls = [
        "describe", "it", "test", "expect", 
        "beforeEach", "afterEach", "beforeAll", "afterAll",
        "suite", "vitest", "should"
    ];
    
    analysis.call_sites.iter().any(|call| {
        test_calls.contains(&call.callee.as_str())
    })
}

/// Check if file is a data file using AST analysis
#[cfg(feature = "semantic")]
fn is_data_file(analysis: &SemanticAnalysis) -> bool {
    // Data files have:
    // - Many string literals (data)
    // - Very few function calls (no logic)
    // - Many declarations (constants)
    // - NO dynamic execution (eval, Function, etc.)
    // - NO crypto API calls
    
    let string_count = analysis.string_literals.len();
    let call_count = analysis.call_sites.len();
    let decl_count = analysis.declarations.len();
    
    // Check for module exports of data
    let has_data_export = analysis.call_sites.iter().any(|call| {
        call.callee == "module.exports" || call.callee == "exports"
    });
    
    // Check for logic patterns that indicate this is NOT a data file
    let has_logic_patterns = analysis.call_sites.iter().any(|call| {
        // Crypto operations
        call.callee.contains("crypto") ||
        call.callee.contains("encrypt") ||
        call.callee.contains("decrypt") ||
        call.callee.contains("createDecipher") ||
        call.callee.contains("createCipher") ||
        // Dynamic execution
        call.callee == "eval" ||
        call.callee == "Function" ||
        call.callee.contains("exec") ||
        // Network operations  
        call.callee == "fetch" ||
        call.callee.contains("http") ||
        call.callee.contains("axios")
    });
    
    // Heuristic: lots of strings, very few calls, no logic patterns, or explicit data export
    // Require stronger signal: >15 strings AND <3 calls
    (string_count > 15 && call_count < 3 && !has_logic_patterns) ||
    (has_data_export && call_count < 5 && !has_logic_patterns)
}

/// Check if file is build output using AST analysis
#[cfg(feature = "semantic")]
fn is_build_output(analysis: &SemanticAnalysis) -> bool {
    // Check for webpack/rollup/parcel wrapper patterns
    analysis.call_sites.iter().any(|call| {
        call.callee_chain.contains(&"__webpack_require__".to_string()) ||
        call.callee.contains("__webpack_require__") ||
        call.callee_chain.contains(&"__rollup__".to_string()) ||
        call.callee.contains("__parcel__")
    })
}

/// Check if a file should be skipped for detection
#[cfg(feature = "semantic")]
pub fn should_skip_file(analysis: &SemanticAnalysis, path: &Path) -> bool {
    matches!(
        classify_file(analysis, path),
        FileClassification::Test | FileClassification::Data | FileClassification::BuildOutput
    )
}

/// Check if a file should be skipped based on path only
pub fn should_skip_file_by_path(path: &Path) -> bool {
    matches!(
        classify_file_by_path(path),
        FileClassification::Test | FileClassification::Data | FileClassification::BuildOutput
    )
}

#[cfg(test)]
#[cfg(feature = "semantic")]
mod tests {
    use super::*;
    use crate::semantic::build_semantic;

    #[test]
    fn test_classify_test_file_by_path() {
        assert_eq!(
            classify_file_by_path(Path::new("foo.test.js")),
            FileClassification::Test
        );
        assert_eq!(
            classify_file_by_path(Path::new("bar.spec.ts")),
            FileClassification::Test
        );
        assert_eq!(
            classify_file_by_path(Path::new("src/__tests__/foo.js")),
            FileClassification::Test
        );
    }

    #[test]
    fn test_classify_build_output_by_path() {
        assert_eq!(
            classify_file_by_path(Path::new("dist/bundle.js")),
            FileClassification::BuildOutput
        );
        assert_eq!(
            classify_file_by_path(Path::new("build/output.min.js")),
            FileClassification::BuildOutput
        );
    }

    #[test]
    fn test_classify_data_file_by_path() {
        assert_eq!(
            classify_file_by_path(Path::new("data/locales.json")),
            FileClassification::Data
        );
        assert_eq!(
            classify_file_by_path(Path::new("src/i18n/en.json")),
            FileClassification::Data
        );
        assert_eq!(
            classify_file_by_path(Path::new("lib/all-countries.js")),
            FileClassification::Data
        );
    }

    #[test]
    fn test_classify_production_file() {
        assert_eq!(
            classify_file_by_path(Path::new("src/index.js")),
            FileClassification::Production
        );
        assert_eq!(
            classify_file_by_path(Path::new("lib/utils.ts")),
            FileClassification::Production
        );
    }

    #[test]
    fn test_classify_test_file_by_ast() {
        let source = r#"
            describe('test suite', () => {
                it('should work', () => {
                    expect(true).toBe(true);
                });
            });
        "#;
        let analysis = build_semantic(source, Path::new("test.js")).unwrap();
        
        assert!(is_test_file(&analysis));
        assert_eq!(classify_file(&analysis, Path::new("test.js")), FileClassification::Test);
    }

    #[test]
    fn test_classify_data_file_by_ast() {
        let source = r#"
            const data1 = "value1";
            const data2 = "value2";
            const data3 = "value3";
            const data4 = "value4";
            const data5 = "value5";
            const data6 = "value6";
            const data7 = "value7";
            const data8 = "value8";
            const data9 = "value9";
            const data10 = "value10";
            const data11 = "value11";
            const data12 = "value12";
            
            module.exports = { data1, data2 };
        "#;
        let analysis = build_semantic(source, Path::new("data.js")).unwrap();
        
        assert!(is_data_file(&analysis));
        assert_eq!(classify_file(&analysis, Path::new("data.js")), FileClassification::Data);
    }

    #[test]
    fn test_classify_production_file_by_ast() {
        let source = r#"
            function add(a, b) {
                return a + b;
            }
            
            async function fetchData() {
                const response = await fetch('/api/data');
                return response.json();
            }
            
            module.exports = { add, fetchData };
        "#;
        let analysis = build_semantic(source, Path::new("utils.js")).unwrap();
        
        assert!(!is_test_file(&analysis));
        assert!(!is_data_file(&analysis));
        assert_eq!(classify_file(&analysis, Path::new("utils.js")), FileClassification::Production);
    }
}
