// Diagnostic test to check if semantic analysis is working

use glassware_core::engine::ScanEngine;
use glassware_core::ir::FileIR;
use std::path::Path;

fn main() {
    let fixture_path = "glassware-core/tests/fixtures/glassworm/wave5_aes_decrypt_eval.js";
    let content = std::fs::read_to_string(fixture_path)
        .expect("Failed to read fixture");
    
    println!("=== Diagnostic Test ===");
    println!("Scanning: {}", fixture_path);
    println!("Content length: {} bytes\n", content.len());
    
    // Build IR manually to check AST parsing
    let ir = FileIR::build(Path::new(fixture_path), &content);
    
    println!("IR Metadata:");
    println!("  - Language: {:?}", ir.metadata().language);
    println!("  - Is JS/TS: {}", ir.is_js_or_ts());
    println!("  - Is minified: {}", ir.is_minified());
    println!("  - Is bundled: {}", ir.is_bundled());
    
    #[cfg(feature = "semantic")]
    {
        println!("\nAST Parsing:");
        if let Some(ast) = ir.ast() {
            println!("  - AST present: Yes");
            println!("  - AST valid: {}", ast.is_valid());
            if !ast.parse_success {
                println!("  - Parse errors: {:?}", ast.errors);
            }
        } else {
            println!("  - AST present: No");
        }
        
        // Try semantic analysis
        if let Some(analysis) = glassware_core::semantic::build_semantic(&content, Path::new(fixture_path)) {
            println!("\nSemantic Analysis:");
            println!("  - String literals: {}", analysis.string_literals.len());
            println!("  - Call sites: {}", analysis.call_sites.len());
            println!("  - Declarations: {}", analysis.declarations.len());
            println!("  - References: {}", analysis.references.len());
            println!("  - Scopes: {}", analysis.scopes.len());
            
            // Check for high-entropy strings
            println!("\nHigh-entropy strings detected:");
            for lit in &analysis.string_literals {
                if lit.value.len() > 20 {
                    let entropy = calculate_entropy(&lit.value);
                    if entropy > 3.5 {
                        println!("  - '{}' (entropy: {:.2})", &lit.value[..20.min(lit.value.len())], entropy);
                    }
                }
            }
        } else {
            println!("\nSemantic Analysis: Failed to build");
        }
    }
    
    // Run scan engine
    println!("\n=== Scan Engine Results ===");
    let engine = ScanEngine::default_detectors();
    let findings = engine.scan(Path::new(fixture_path), &content);
    
    println!("Total findings: {}", findings.len());
    for finding in &findings {
        println!("  [{:?}] {:?} (line {})", finding.severity, finding.category, finding.line);
    }
    
    // Check for specific categories
    let has_hardcoded_key = findings.iter().any(|f| {
        format!("{:?}", f.category).contains("HardcodedKey")
    });
    
    println!("\n=== Summary ===");
    println!("HardcodedKeyDecryption detected: {}", has_hardcoded_key);
}

// Simple entropy calculation
fn calculate_entropy(s: &str) -> f64 {
    use std::collections::HashMap;
    let mut freq = HashMap::new();
    for c in s.chars() {
        *freq.entry(c).or_insert(0) += 1;
    }
    let len = s.len() as f64;
    -freq.values().map(|&count| {
        let p = count as f64 / len;
        p * p.log2()
    }).sum::<f64>()
}
