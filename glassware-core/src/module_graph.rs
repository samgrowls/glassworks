//! Module Graph Construction
//!
//! Tracks imports/exports between files in a package to enable cross-file taint tracking.
//! Supports ES6 modules, CommonJS, and TypeScript import/export syntax.

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use regex::Regex;
use once_cell::sync::Lazy;

/// Module graph tracking imports/exports across files
#[derive(Debug, Clone, Default)]
pub struct ModuleGraph {
    /// file_path -> Module info
    pub modules: HashMap<String, Module>,

    /// Import/export edges between modules
    pub edges: Vec<ModuleEdge>,

    /// Resolved import paths (importer -> resolved file)
    resolved_imports: HashMap<String, Vec<String>>,
}

/// A module (file) in the graph
#[derive(Debug, Clone)]
pub struct Module {
    /// Absolute or relative path to the file
    pub path: String,
    /// Exported symbols (names)
    pub exports: Vec<Export>,
    /// Imported symbols
    pub imports: Vec<Import>,
    /// Other files this module depends on (resolved paths)
    pub dependencies: Vec<String>,
    /// Module type detected (ES6, CommonJS, TypeScript, Mixed)
    pub module_type: ModuleType,
}

/// Type of module system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleType {
    /// ES6 modules (import/export)
    ES6,
    /// CommonJS (require/module.exports)
    CommonJS,
    /// TypeScript with type-only imports
    TypeScript,
    /// Mixed module systems
    Mixed,
    /// Unknown or no module syntax detected
    Unknown,
}

/// An exported symbol
#[derive(Debug, Clone)]
pub struct Export {
    /// Symbol name (or "default" for default exports)
    pub name: String,
    /// Whether this is a named or default export
    pub export_type: ExportType,
    /// Line number where export is declared (1-indexed)
    pub line: usize,
    /// Whether this is a type-only export (TypeScript)
    pub is_type_only: bool,
}

/// Type of export
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportType {
    /// Named export: export { foo }
    Named,
    /// Default export: export default
    Default,
    /// Re-export: export { foo } from './bar'
    ReExport,
    /// Star re-export: export * from './bar'
    StarReExport,
    /// Inline export: export const foo = ...
    Inline,
}

/// An imported symbol
#[derive(Debug, Clone)]
pub struct Import {
    /// Local symbol name (may differ from exported name due to aliasing)
    pub local_name: String,
    /// Original exported name (for named imports with aliases)
    pub imported_name: Option<String>,
    /// Module specifier (e.g., "./decoder", "lodash")
    pub from_module: String,
    /// Resolved file path (if resolvable within the package)
    pub resolved_path: Option<String>,
    /// Line number where import is declared (1-indexed)
    pub line: usize,
    /// Type of import
    pub import_type: ImportType,
    /// Whether this is a type-only import (TypeScript)
    pub is_type_only: bool,
}

/// Type of import
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportType {
    /// Named import: import { foo } from './bar'
    Named,
    /// Default import: import foo from './bar'
    Default,
    /// Namespace import: import * as foo from './bar'
    Namespace,
    /// Side-effect import: import './bar'
    SideEffect,
    /// Dynamic import: import('./bar')
    Dynamic,
    /// Require: require('./bar')
    Require,
}

/// An edge in the module graph
#[derive(Debug, Clone)]
pub struct ModuleEdge {
    /// Source file (exporter)
    pub from_file: String,
    /// Target file (importer)
    pub to_file: String,
    /// Symbol being imported/exported
    pub symbol: String,
    /// Type of import/export
    pub edge_type: EdgeType,
}

/// Type of module edge
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
    /// Named import/export
    Named,
    /// Default import/export
    Default,
    /// Namespace import (import * as)
    Namespace,
    /// Star re-export (export *)
    Star,
}

/// Regex patterns for module parsing
static ES6_IMPORT_NAMED: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"import\s*\{([^}]+)\}\s*from\s*['"]([^'"]+)['"]"#).unwrap()
});

static ES6_IMPORT_DEFAULT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"import\s+(\w+)\s+from\s*['"]([^'"]+)['"]"#).unwrap()
});

static ES6_IMPORT_NAMESPACE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"import\s*\*\s*as\s+(\w+)\s+from\s*['"]([^'"]+)['"]"#).unwrap()
});

static ES6_IMPORT_SIDE_EFFECT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"import\s*['"]([^'"]+)['"]"#).unwrap()
});

static ES6_EXPORT_NAMED: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"export\s*\{([^}]+)\}"#).unwrap()
});

static ES6_EXPORT_DEFAULT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"export\s+default\b"#).unwrap()
});

static ES6_EXPORT_INLINE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"export\s+(const|let|var|function|class|async\s+function)\s+(\w+)"#).unwrap()
});

static ES6_EXPORT_FROM: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"export\s*\{([^}]+)\}\s*from\s*['"]([^'"]+)['"]"#).unwrap()
});

static ES6_EXPORT_STAR: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"export\s*\*\s*from\s*['"]([^'"]+)['"]"#).unwrap()
});

static TS_TYPE_IMPORT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"import\s+type\s+\{([^}]+)\}\s+from\s*['"]([^'"]+)['"]"#).unwrap()
});

static TS_TYPE_EXPORT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"export\s+type\s+\{([^}]+)\}"#).unwrap()
});

static COMMONJS_REQUIRE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?:const|let|var)\s+(\w+)\s*=\s*require\s*\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap()
});

static COMMONJS_REQUIRE_DESTRUCTURE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?:const|let|var)\s*\{([^}]+)\}\s*=\s*require\s*\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap()
});

static COMMONJS_EXPORTS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"module\.exports\s*=\s*(\w+)"#).unwrap()
});

static COMMONJS_EXPORTS_OBJECT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"module\.exports\s*\.\s*(\w+)"#).unwrap()
});

static COMMONJS_EXPORTS_ASSIGNMENT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"exports\s*\.\s*(\w+)\s*="#).unwrap()
});

impl ModuleGraph {
    /// Create a new empty module graph
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            edges: Vec::new(),
            resolved_imports: HashMap::new(),
        }
    }
    
    /// Add a file to the module graph
    /// 
    /// Parses the file content to extract imports and exports.
    pub fn add_file(&mut self, path: &str, content: &str) {
        let path = path.to_string();
        let module = self.parse_module(&path, content);
        
        // Store dependencies for resolution
        let dependencies: Vec<String> = module.imports
            .iter()
            .filter_map(|imp| imp.resolved_path.clone())
            .collect();
        
        self.modules.insert(path.clone(), module);
        
        // Update resolved imports map
        self.resolved_imports.insert(path.clone(), dependencies);
        
        // Build edges
        self.build_edges(&path);
    }
    
    /// Parse a file to extract module information
    fn parse_module(&self, path: &str, content: &str) -> Module {
        let mut exports = Vec::new();
        let mut imports = Vec::new();
        let mut module_type = ModuleType::Unknown;

        // Detect module type
        let has_es6 = ES6_IMPORT_NAMED.is_match(content)
            || ES6_IMPORT_DEFAULT.is_match(content)
            || ES6_EXPORT_NAMED.is_match(content)
            || ES6_EXPORT_DEFAULT.is_match(content);

        let has_commonjs = COMMONJS_REQUIRE.is_match(content)
            || COMMONJS_EXPORTS.is_match(content)
            || COMMONJS_EXPORTS_OBJECT.is_match(content);

        let has_typescript = TS_TYPE_IMPORT.is_match(content)
            || TS_TYPE_EXPORT.is_match(content)
            || path.ends_with(".ts")
            || path.ends_with(".tsx");

        // TypeScript files with ES6-style imports (including type imports)
        let has_ts_es6 = has_typescript && (has_es6 || TS_TYPE_IMPORT.is_match(content) || TS_TYPE_EXPORT.is_match(content));

        module_type = if has_ts_es6 {
            ModuleType::TypeScript
        } else if has_es6 && has_commonjs {
            ModuleType::Mixed
        } else if has_es6 {
            ModuleType::ES6
        } else if has_commonjs {
            ModuleType::CommonJS
        } else {
            ModuleType::Unknown
        };
        
        // Parse ES6 imports
        self.parse_es6_imports(content, &mut imports);
        
        // Parse CommonJS requires
        self.parse_commonjs_requires(content, &mut imports);
        
        // Parse ES6 exports
        self.parse_es6_exports(content, &mut exports);
        
        // Parse CommonJS exports
        self.parse_commonjs_exports(content, &mut exports);
        
        Module {
            path: path.to_string(),
            exports,
            imports,
            dependencies: Vec::new(), // Will be populated after resolution
            module_type,
        }
    }
    
    /// Parse ES6 import statements
    fn parse_es6_imports(&self, content: &str, imports: &mut Vec<Import>) {
        let lines: Vec<&str> = content.lines().collect();
        
        // Named imports: import { foo, bar as baz } from './module'
        for cap in ES6_IMPORT_NAMED.captures_iter(content) {
            if let (Some(names_str), Some(from_module)) = (cap.get(1), cap.get(2)) {
                let line = self.find_line_number(&lines, cap.get(0).unwrap().start());
                
                // Parse individual imports (may have aliases)
                for name_part in names_str.as_str().split(',') {
                    let name_part = name_part.trim();
                    let (imported_name, local_name) = if let Some(parts) = name_part.split_once(" as ") {
                        (parts.0.trim().to_string(), parts.1.trim().to_string())
                    } else {
                        (name_part.to_string(), name_part.to_string())
                    };
                    
                    imports.push(Import {
                        local_name,
                        imported_name: Some(imported_name),
                        from_module: from_module.as_str().to_string(),
                        resolved_path: None, // Will be resolved later
                        line,
                        import_type: ImportType::Named,
                        is_type_only: false,
                    });
                }
            }
        }
        
        // Default imports: import foo from './module'
        for cap in ES6_IMPORT_DEFAULT.captures_iter(content) {
            if let (Some(local_name), Some(from_module)) = (cap.get(1), cap.get(2)) {
                let line = self.find_line_number(&lines, cap.get(0).unwrap().start());
                
                imports.push(Import {
                    local_name: local_name.as_str().to_string(),
                    imported_name: None,
                    from_module: from_module.as_str().to_string(),
                    resolved_path: None,
                    line,
                    import_type: ImportType::Default,
                    is_type_only: false,
                });
            }
        }
        
        // Namespace imports: import * as foo from './module'
        for cap in ES6_IMPORT_NAMESPACE.captures_iter(content) {
            if let (Some(local_name), Some(from_module)) = (cap.get(1), cap.get(2)) {
                let line = self.find_line_number(&lines, cap.get(0).unwrap().start());
                
                imports.push(Import {
                    local_name: local_name.as_str().to_string(),
                    imported_name: None,
                    from_module: from_module.as_str().to_string(),
                    resolved_path: None,
                    line,
                    import_type: ImportType::Namespace,
                    is_type_only: false,
                });
            }
        }
        
        // TypeScript type imports: import type { Foo } from './module'
        for cap in TS_TYPE_IMPORT.captures_iter(content) {
            if let (Some(names_str), Some(from_module)) = (cap.get(1), cap.get(2)) {
                let line = self.find_line_number(&lines, cap.get(0).unwrap().start());
                
                for name_part in names_str.as_str().split(',') {
                    let name = name_part.trim();
                    imports.push(Import {
                        local_name: name.to_string(),
                        imported_name: Some(name.to_string()),
                        from_module: from_module.as_str().to_string(),
                        resolved_path: None,
                        line,
                        import_type: ImportType::Named,
                        is_type_only: true,
                    });
                }
            }
        }
    }
    
    /// Parse CommonJS require statements
    fn parse_commonjs_requires(&self, content: &str, imports: &mut Vec<Import>) {
        let lines: Vec<&str> = content.lines().collect();
        
        // Simple require: const foo = require('./module')
        for cap in COMMONJS_REQUIRE.captures_iter(content) {
            if let (Some(local_name), Some(from_module)) = (cap.get(1), cap.get(2)) {
                let line = self.find_line_number(&lines, cap.get(0).unwrap().start());
                
                imports.push(Import {
                    local_name: local_name.as_str().to_string(),
                    imported_name: None,
                    from_module: from_module.as_str().to_string(),
                    resolved_path: None,
                    line,
                    import_type: ImportType::Require,
                    is_type_only: false,
                });
            }
        }
        
        // Destructured require: const { foo, bar } = require('./module')
        for cap in COMMONJS_REQUIRE_DESTRUCTURE.captures_iter(content) {
            if let (Some(names_str), Some(from_module)) = (cap.get(1), cap.get(2)) {
                let line = self.find_line_number(&lines, cap.get(0).unwrap().start());
                
                for name_part in names_str.as_str().split(',') {
                    let name = name_part.trim();
                    imports.push(Import {
                        local_name: name.to_string(),
                        imported_name: Some(name.to_string()),
                        from_module: from_module.as_str().to_string(),
                        resolved_path: None,
                        line,
                        import_type: ImportType::Require,
                        is_type_only: false,
                    });
                }
            }
        }
    }
    
    /// Parse ES6 export statements
    fn parse_es6_exports(&self, content: &str, exports: &mut Vec<Export>) {
        let lines: Vec<&str> = content.lines().collect();
        
        // Named exports: export { foo, bar }
        for cap in ES6_EXPORT_NAMED.captures_iter(content) {
            if let Some(names_str) = cap.get(1) {
                let line = self.find_line_number(&lines, cap.get(0).unwrap().start());
                
                for name_part in names_str.as_str().split(',') {
                    let name = name_part.trim();
                    exports.push(Export {
                        name: name.to_string(),
                        export_type: ExportType::Named,
                        line,
                        is_type_only: false,
                    });
                }
            }
        }
        
        // Default export: export default
        for cap in ES6_EXPORT_DEFAULT.captures_iter(content) {
            let line = self.find_line_number(&lines, cap.get(0).unwrap().start());
            
            exports.push(Export {
                name: "default".to_string(),
                export_type: ExportType::Default,
                line,
                is_type_only: false,
            });
        }
        
        // Inline exports: export const foo = ..., export function bar() {}
        for cap in ES6_EXPORT_INLINE.captures_iter(content) {
            if let Some(name) = cap.get(2) {
                let line = self.find_line_number(&lines, cap.get(0).unwrap().start());
                
                exports.push(Export {
                    name: name.as_str().to_string(),
                    export_type: ExportType::Inline,
                    line,
                    is_type_only: false,
                });
            }
        }
        
        // TypeScript type exports: export type { Foo }
        for cap in TS_TYPE_EXPORT.captures_iter(content) {
            if let Some(names_str) = cap.get(1) {
                let line = self.find_line_number(&lines, cap.get(0).unwrap().start());
                
                for name_part in names_str.as_str().split(',') {
                    let name = name_part.trim();
                    exports.push(Export {
                        name: name.to_string(),
                        export_type: ExportType::Named,
                        line,
                        is_type_only: true,
                    });
                }
            }
        }
    }
    
    /// Parse CommonJS export statements
    fn parse_commonjs_exports(&self, content: &str, exports: &mut Vec<Export>) {
        let lines: Vec<&str> = content.lines().collect();
        
        // module.exports = foo
        for cap in COMMONJS_EXPORTS.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                let line = self.find_line_number(&lines, cap.get(0).unwrap().start());
                
                exports.push(Export {
                    name: name.as_str().to_string(),
                    export_type: ExportType::Default,
                    line,
                    is_type_only: false,
                });
            }
        }
        
        // module.exports.foo = ...
        for cap in COMMONJS_EXPORTS_OBJECT.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                let line = self.find_line_number(&lines, cap.get(0).unwrap().start());
                
                exports.push(Export {
                    name: name.as_str().to_string(),
                    export_type: ExportType::Named,
                    line,
                    is_type_only: false,
                });
            }
        }
        
        // exports.foo = ...
        for cap in COMMONJS_EXPORTS_ASSIGNMENT.captures_iter(content) {
            if let Some(name) = cap.get(1) {
                let line = self.find_line_number(&lines, cap.get(0).unwrap().start());
                
                exports.push(Export {
                    name: name.as_str().to_string(),
                    export_type: ExportType::Named,
                    line,
                    is_type_only: false,
                });
            }
        }
    }
    
    /// Find line number for a byte offset
    fn find_line_number(&self, lines: &[&str], offset: usize) -> usize {
        let mut current_offset = 0;
        for (i, line) in lines.iter().enumerate() {
            if current_offset + line.len() >= offset {
                return i + 1; // 1-indexed
            }
            current_offset += line.len() + 1; // +1 for newline
        }
        1
    }
    
    /// Build edges from imports
    fn build_edges(&mut self, path: &str) {
        let module = match self.modules.get(path) {
            Some(m) => m.clone(),
            None => return,
        };
        
        for import in &module.imports {
            // Skip external modules (not starting with . or ..)
            if !import.from_module.starts_with('.') {
                continue;
            }
            
            // Create edges for each import
            let edge_type = match import.import_type {
                ImportType::Named | ImportType::Require => EdgeType::Named,
                ImportType::Default => EdgeType::Default,
                ImportType::Namespace => EdgeType::Namespace,
                ImportType::SideEffect | ImportType::Dynamic => continue,
            };
            
            // For named imports, create edge for each symbol
            if let Some(ref imported_name) = import.imported_name {
                self.edges.push(ModuleEdge {
                    from_file: import.resolved_path.clone().unwrap_or_default(),
                    to_file: path.to_string(),
                    symbol: imported_name.clone(),
                    edge_type,
                });
            } else {
                // Default or namespace import
                self.edges.push(ModuleEdge {
                    from_file: import.resolved_path.clone().unwrap_or_default(),
                    to_file: path.to_string(),
                    symbol: import.local_name.clone(),
                    edge_type,
                });
            }
        }
    }
    
    /// Resolve import paths within a package
    /// 
    /// # Arguments
    /// * `base_dir` - Base directory of the package
    /// * `import_path` - Import specifier (e.g., "./decoder")
    /// * `importer_file` - File making the import
    pub fn resolve_import(&self, base_dir: &Path, import_path: &str, importer_file: &str) -> Option<String> {
        // Skip external modules
        if !import_path.starts_with('.') && !import_path.starts_with('/') {
            return None;
        }
        
        let importer_path = Path::new(importer_file);
        let importer_dir = importer_path.parent()?;
        
        // Build candidate paths
        let mut candidates = Vec::new();
        
        // Direct path
        let direct = importer_dir.join(import_path);
        candidates.push(direct.clone());
        
        // Try .js extension
        candidates.push(direct.with_extension("js"));
        
        // Try .ts extension
        candidates.push(direct.with_extension("ts"));
        
        // Try .tsx extension
        candidates.push(direct.with_extension("tsx"));
        
        // Try .mjs extension
        candidates.push(direct.with_extension("mjs"));
        
        // Try .cjs extension
        candidates.push(direct.with_extension("cjs"));
        
        // Try index files
        candidates.push(importer_dir.join(import_path).join("index.js"));
        candidates.push(importer_dir.join(import_path).join("index.ts"));
        
        // Check which candidate exists
        for candidate in &candidates {
            let abs_path = if candidate.is_absolute() {
                candidate.clone()
            } else {
                base_dir.join(candidate)
            };
            
            if abs_path.exists() {
                return Some(abs_path.to_string_lossy().to_string());
            }
        }
        
        None
    }
    
    /// Get all files that import a specific symbol from a file
    pub fn find_importers(&self, symbol: &str, from_file: &str) -> Vec<&str> {
        self.edges
            .iter()
            .filter(|edge| edge.symbol == symbol && edge.from_file == from_file)
            .map(|edge| edge.to_file.as_str())
            .collect()
    }

    /// Get dependency chain starting from a file (BFS)
    pub fn get_dependency_chain<'a>(&'a self, start_file: &'a str) -> Vec<&'a str> {
        let mut visited: HashSet<&str> = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        queue.push_back(start_file);
        visited.insert(start_file);

        while let Some(current) = queue.pop_front() {
            result.push(current);

            // Find all files that current imports from
            for edge in &self.edges {
                if edge.to_file == current && !visited.contains(edge.from_file.as_str()) {
                    visited.insert(&edge.from_file);
                    queue.push_back(&edge.from_file);
                }
            }
        }

        result
    }
    
    /// Get all modules in the graph
    pub fn modules(&self) -> impl Iterator<Item = (&String, &Module)> {
        self.modules.iter()
    }
    
    /// Get a specific module
    pub fn get_module(&self, path: &str) -> Option<&Module> {
        self.modules.get(path)
    }
    
    /// Get all edges in the graph
    pub fn edges(&self) -> &[ModuleEdge] {
        &self.edges
    }
    
    /// Check if there's a path from source to target
    pub fn has_path(&self, source: &str, target: &str) -> bool {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        queue.push_back(source);
        visited.insert(source);
        
        while let Some(current) = queue.pop_front() {
            if current == target {
                return true;
            }
            
            for edge in &self.edges {
                if edge.from_file == current && !visited.contains(&edge.to_file.as_str()) {
                    visited.insert(&edge.to_file);
                    queue.push_back(&edge.to_file);
                }
            }
        }
        
        false
    }

    /// Get the shortest path from source to target
    pub fn find_path<'a>(&'a self, source: &'a str, target: &'a str) -> Option<Vec<&'a str>> {
        let mut visited: HashSet<&str> = HashSet::new();
        let mut parent: HashMap<&str, Option<&str>> = HashMap::new();
        let mut queue = VecDeque::new();

        queue.push_back(source);
        visited.insert(source);
        parent.insert(source, None);

        while let Some(current) = queue.pop_front() {
            if current == target {
                // Reconstruct path
                let mut path = Vec::new();
                let mut node = Some(current);
                while let Some(n) = node {
                    path.push(n);
                    node = *parent.get(n).unwrap_or(&None);
                }
                path.reverse();
                return Some(path);
            }

            for edge in &self.edges {
                if edge.from_file == current && !visited.contains(edge.to_file.as_str()) {
                    visited.insert(&edge.to_file);
                    parent.insert(&edge.to_file, Some(current));
                    queue.push_back(&edge.to_file);
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_es6_named_import() {
        let content = r#"import { foo, bar as baz } from './module';"#;
        let mut graph = ModuleGraph::new();
        let module = graph.parse_module("test.js", content);
        
        assert_eq!(module.imports.len(), 2);
        assert_eq!(module.imports[0].local_name, "foo");
        assert_eq!(module.imports[0].imported_name, Some("foo".to_string()));
        assert_eq!(module.imports[1].local_name, "baz");
        assert_eq!(module.imports[1].imported_name, Some("bar".to_string()));
    }
    
    #[test]
    fn test_parse_es6_default_import() {
        let content = r#"import decoder from './decoder';"#;
        let mut graph = ModuleGraph::new();
        let module = graph.parse_module("test.js", content);
        
        assert_eq!(module.imports.len(), 1);
        assert_eq!(module.imports[0].local_name, "decoder");
        assert_eq!(module.imports[0].import_type, ImportType::Default);
    }
    
    #[test]
    fn test_parse_es6_namespace_import() {
        let content = r#"import * as utils from './utils';"#;
        let mut graph = ModuleGraph::new();
        let module = graph.parse_module("test.js", content);
        
        assert_eq!(module.imports.len(), 1);
        assert_eq!(module.imports[0].local_name, "utils");
        assert_eq!(module.imports[0].import_type, ImportType::Namespace);
    }
    
    #[test]
    fn test_parse_es6_named_export() {
        let content = r#"export { foo, bar };"#;
        let mut graph = ModuleGraph::new();
        let module = graph.parse_module("test.js", content);
        
        assert_eq!(module.exports.len(), 2);
        assert_eq!(module.exports[0].name, "foo");
        assert_eq!(module.exports[0].export_type, ExportType::Named);
    }
    
    #[test]
    fn test_parse_es6_default_export() {
        let content = r#"export default function decode() {}"#;
        let mut graph = ModuleGraph::new();
        let module = graph.parse_module("test.js", content);
        
        assert!(module.exports.iter().any(|e| e.name == "default"));
    }
    
    #[test]
    fn test_parse_es6_inline_export() {
        let content = r#"export const decoder = (s) => s;"#;
        let mut graph = ModuleGraph::new();
        let module = graph.parse_module("test.js", content);
        
        assert_eq!(module.exports.len(), 1);
        assert_eq!(module.exports[0].name, "decoder");
        assert_eq!(module.exports[0].export_type, ExportType::Inline);
    }
    
    #[test]
    fn test_parse_commonjs_require() {
        let content = r#"const decoder = require('./decoder');"#;
        let mut graph = ModuleGraph::new();
        let module = graph.parse_module("test.js", content);
        
        assert_eq!(module.imports.len(), 1);
        assert_eq!(module.imports[0].local_name, "decoder");
        assert_eq!(module.imports[0].import_type, ImportType::Require);
    }
    
    #[test]
    fn test_parse_commonjs_destructure_require() {
        let content = r#"const { decode, encode } = require('./utils');"#;
        let mut graph = ModuleGraph::new();
        let module = graph.parse_module("test.js", content);
        
        assert_eq!(module.imports.len(), 2);
        assert_eq!(module.imports[0].local_name, "decode");
        assert_eq!(module.imports[1].local_name, "encode");
    }
    
    #[test]
    fn test_parse_commonjs_exports() {
        // Test module.exports = identifier
        let content = r#"module.exports = decoder;"#;
        let mut graph = ModuleGraph::new();
        let module = graph.parse_module("test.js", content);

        assert!(!module.exports.is_empty());
        assert!(module.exports.iter().any(|e| e.name == "decoder"));
    }
    
    #[test]
    fn test_module_type_detection() {
        let mut graph = ModuleGraph::new();
        
        let es6_content = r#"import { foo } from './bar'; export { foo };"#;
        let module = graph.parse_module("test.js", es6_content);
        assert_eq!(module.module_type, ModuleType::ES6);
        
        let cjs_content = r#"const foo = require('./bar'); module.exports = foo;"#;
        let module = graph.parse_module("test.js", cjs_content);
        assert_eq!(module.module_type, ModuleType::CommonJS);
        
        let ts_content = r#"import type { Foo } from './foo'; export type { Foo };"#;
        let module = graph.parse_module("test.ts", ts_content);
        assert_eq!(module.module_type, ModuleType::TypeScript);
    }
    
    #[test]
    fn test_dependency_chain() {
        let mut graph = ModuleGraph::new();
        
        // Manually create modules and edges for testing
        graph.modules.insert("a.js".to_string(), Module {
            path: "a.js".to_string(),
            exports: vec![],
            imports: vec![],
            dependencies: vec![],
            module_type: ModuleType::ES6,
        });
        graph.modules.insert("b.js".to_string(), Module {
            path: "b.js".to_string(),
            exports: vec![],
            imports: vec![],
            dependencies: vec![],
            module_type: ModuleType::ES6,
        });
        graph.modules.insert("c.js".to_string(), Module {
            path: "c.js".to_string(),
            exports: vec![],
            imports: vec![],
            dependencies: vec![],
            module_type: ModuleType::ES6,
        });
        
        graph.edges.push(ModuleEdge {
            from_file: "a.js".to_string(),
            to_file: "b.js".to_string(),
            symbol: "foo".to_string(),
            edge_type: EdgeType::Named,
        });
        graph.edges.push(ModuleEdge {
            from_file: "b.js".to_string(),
            to_file: "c.js".to_string(),
            symbol: "bar".to_string(),
            edge_type: EdgeType::Named,
        });
        
        let chain = graph.get_dependency_chain("c.js");
        assert!(chain.contains(&"c.js"));
        assert!(chain.contains(&"b.js"));
        assert!(chain.contains(&"a.js"));
    }
    
    #[test]
    fn test_find_importers() {
        let mut graph = ModuleGraph::new();
        
        graph.modules.insert("decoder.js".to_string(), Module {
            path: "decoder.js".to_string(),
            exports: vec![],
            imports: vec![],
            dependencies: vec![],
            module_type: ModuleType::ES6,
        });
        graph.modules.insert("payload.js".to_string(), Module {
            path: "payload.js".to_string(),
            exports: vec![],
            imports: vec![],
            dependencies: vec![],
            module_type: ModuleType::ES6,
        });
        
        graph.edges.push(ModuleEdge {
            from_file: "decoder.js".to_string(),
            to_file: "payload.js".to_string(),
            symbol: "decoder".to_string(),
            edge_type: EdgeType::Named,
        });
        
        let importers = graph.find_importers("decoder", "decoder.js");
        assert_eq!(importers.len(), 1);
        assert_eq!(importers[0], "payload.js");
    }
}
