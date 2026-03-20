//! Minified Code Detection
//!
//! Utilities for detecting minified, bundled, or generated code.
//! These files should be skipped by certain detectors to avoid false positives.

use std::path::Path;

/// Check if a file appears to be minified or bundled code
pub fn is_minified_file(path: &Path, content: &str) -> bool {
    let path_str = path.to_string_lossy();
    
    // Check file path patterns first (fast)
    if is_bundled_path(&path_str) {
        return true;
    }
    
    // Check content heuristics
    is_minified_content(content)
}

/// Check if file path suggests bundled/minified code
fn is_bundled_path(path: &str) -> bool {
    // Directory patterns
    let bundled_dirs = [
        "/dist/", "/build/", "/lib/", "/bin/", "/out/",
        "/bundle/", "/compiled/", "/generated/", "/.next/",
        "/.nuxt/", "/.output/", "/umd/", "/esm/", "/cjs/",
    ];
    
    for dir in &bundled_dirs {
        if path.contains(dir) {
            return true;
        }
    }
    
    // File patterns
    let bundled_patterns = [
        ".min.", ".bundle.", ".umd.", ".esm.", ".cjs.",
        ".webpack.", ".rollup.", ".babel.", ".swc.",
    ];
    
    for pattern in &bundled_patterns {
        if path.contains(pattern) {
            return true;
        }
    }
    
    // Known bundler output signatures in path
    let bundler_signatures = [
        "webpack", "rollup", "babel", "swc", "esbuild",
        "vite", "parcel", "browserify", "gulp", "grunt",
    ];
    
    for sig in &bundler_signatures {
        if path.to_lowercase().contains(sig) {
            return true;
        }
    }
    
    false
}

/// Check if content appears to be minified
fn is_minified_content(content: &str) -> bool {
    let lines: Vec<&str> = content.lines().collect();
    
    // Empty file check
    if lines.is_empty() || content.is_empty() {
        return false;
    }
    
    // Heuristic 1: Very long average line length (>200 chars)
    let avg_line_length = content.len() / lines.len().max(1);
    if avg_line_length > 200 {
        return true;
    }
    
    // Heuristic 2: Few newlines relative to file size
    // Minified code often has <1 newline per 500 bytes
    if content.len() > 10000 && lines.len() < content.len() / 500 {
        return true;
    }
    
    // Heuristic 3: Bundle signatures in content
    let bundle_signatures = [
        "webpackJsonp", "__webpack_require__", "webpackChunk",
        "rollupChunk", "__rollup__", "babelHelpers",
        "esbuild", "parcelRequire", "browserify",
    ];
    
    for sig in &bundle_signatures {
        if content.contains(sig) {
            return true;
        }
    }
    
    // Heuristic 4: High ratio of non-ASCII to total (minified often has unicode issues)
    // Skip this for now as it's expensive
    
    false
}

/// Check if content contains bundler signatures
pub fn has_bundler_signature(content: &str) -> bool {
    let signatures = [
        "webpack", "rollup", "babel", "esbuild", "vite",
        "parcel", "browserify", "gulp", "grunt", "swc",
    ];
    
    signatures.iter().any(|sig| content.to_lowercase().contains(sig))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_dist_directory() {
        assert!(is_bundled_path("/project/dist/bundle.js"));
        assert!(is_bundled_path("/project/build/index.js"));
        assert!(is_bundled_path("/project/lib/utils.js"));
    }

    #[test]
    fn test_detect_min_file() {
        assert!(is_bundled_path("/project/jquery.min.js"));
        assert!(is_bundled_path("/project/react.production.min.js"));
    }

    #[test]
    fn test_detect_bundler_in_path() {
        assert!(is_bundled_path("/project/webpack.config.js"));
        assert!(is_bundled_path("/project/.next/bundle.js"));
    }

    #[test]
    fn test_detect_minified_content() {
        let minified = "var a=1,b=2;function c(d){return d+1}console.log(a,b,c(5));".repeat(100);
        assert!(is_minified_content(&minified));
    }

    #[test]
    fn test_detect_normal_content() {
        let normal = r#"
            function add(a, b) {
                return a + b;
            }

            console.log(add(1, 2));
        "#.repeat(10);
        assert!(!is_minified_content(&normal));
    }

    #[test]
    fn test_bundler_signatures() {
        assert!(has_bundler_signature("webpackJsonp([1,2,3])"));
        assert!(has_bundler_signature("__webpack_require__(123)"));
        assert!(has_bundler_signature("babelHelpers.defineProperty"));
    }
}
