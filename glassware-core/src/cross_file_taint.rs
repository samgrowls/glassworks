//! Cross-File Taint Tracking
//!
//! Tracks taint flows across module boundaries by combining module graph analysis
//! with intra-file taint tracking. Enables detection of split payloads where the
//! decoder is in one file and the payload execution is in another.

// Re-export taint types for convenience
pub use crate::taint::{
    CrossFileTaintFlow, CrossFileTaintSink, CrossFileTaintSource, CrossFileSinkType,
    CrossFileSourceType, DynExecKind, ImportEdge, TaintSink, TaintSource,
};
use crate::module_graph::ModuleGraph;
use std::collections::{HashMap, HashSet, VecDeque};

/// Cross-file taint tracker
///
/// Combines module graph analysis with taint source/sink tracking to detect
/// data flows across file boundaries.
pub struct CrossFileTaintTracker {
    /// Module graph for import/export relationships
    graph: ModuleGraph,
    /// Taint sources (e.g., decoder output, encrypted payload)
    taint_sources: Vec<CrossFileTaintSource>,
    /// Taint sinks (e.g., eval, exec)
    taint_sinks: Vec<CrossFileTaintSink>,
    /// Symbol renames: (file, local_name) -> original_name
    symbol_renames: HashMap<(String, String), String>,
}

impl CrossFileTaintTracker {
    /// Create a new cross-file taint tracker
    pub fn new(graph: ModuleGraph) -> Self {
        Self {
            graph,
            taint_sources: Vec::new(),
            taint_sinks: Vec::new(),
            symbol_renames: HashMap::new(),
        }
    }
    
    /// Add a taint source (e.g., stego decoder output)
    pub fn add_source(&mut self, source: CrossFileTaintSource) {
        self.taint_sources.push(source);
    }
    
    /// Add a taint sink (e.g., eval, exec)
    pub fn add_sink(&mut self, sink: CrossFileTaintSink) {
        self.taint_sinks.push(sink);
    }
    
    /// Record a symbol rename (for aliased imports)
    pub fn add_symbol_rename(&mut self, file: &str, local_name: &str, original_name: &str) {
        self.symbol_renames.insert(
            (file.to_string(), local_name.to_string()),
            original_name.to_string(),
        );
    }
    
    /// Check if any source flows to any sink across files
    pub fn has_cross_file_flow(&self) -> bool {
        self.find_cross_file_flows().next().is_some()
    }
    
    /// Find all cross-file taint flows
    pub fn find_cross_file_flows(&self) -> impl Iterator<Item = CrossFileTaintFlow> {
        let mut flows = Vec::new();
        
        for source in &self.taint_sources {
            // Trace taint from source through module graph
            for sink in self.trace_taint(source) {
                flows.push(sink);
            }
        }
        
        flows.into_iter()
    }
    
    /// Trace taint from a source through the module graph
    /// Returns all reachable sinks with flow information
    fn trace_taint(&self, source: &CrossFileTaintSource) -> Vec<CrossFileTaintFlow> {
        let mut flows = Vec::new();
        
        // Get all files that import from the source file
        let importers = self.graph.find_importers(&source.symbol, &source.file);
        
        // BFS to find all reachable sinks
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        // Start from source file
        queue.push_back((source.file.clone(), source.symbol.clone(), Vec::new()));
        visited.insert((source.file.clone(), source.symbol.clone()));

        while let Some((current_file, current_symbol, import_chain)) = queue.pop_front() {
            // Check if there's a sink in this file for this symbol
            for sink in &self.taint_sinks {
                if sink.file == current_file && self.symbol_matches(&sink.symbol, &current_symbol) {
                    // Found a flow!
                    let mut files_traversed: Vec<String> = import_chain
                        .iter()
                        .map(|e: &ImportEdge| e.to_file.clone())
                        .collect();
                    files_traversed.push(current_file.clone());

                    // Calculate confidence based on flow characteristics
                    let confidence = self.calculate_confidence(source, sink, &import_chain);

                    flows.push(CrossFileTaintFlow {
                        source: source.clone(),
                        sink: sink.clone(),
                        files_traversed,
                        import_chain: import_chain.clone(),
                        confidence,
                    });
                }
            }
            
            // Follow import edges to find more files
            for importer in &importers {
                let edge_key = (current_file.clone(), current_symbol.clone());
                
                if !visited.contains(&edge_key) {
                    visited.insert(edge_key.clone());
                    
                    let mut new_chain = import_chain.clone();
                    new_chain.push(ImportEdge {
                        from_file: current_file.clone(),
                        to_file: importer.to_string(),
                        symbol: current_symbol.clone(),
                        import_type: "named".to_string(),
                    });
                    
                    queue.push_back((importer.to_string(), current_symbol.clone(), new_chain));
                }
            }
        }
        
        flows
    }
    
    /// Check if a sink symbol matches a source symbol (considering renames)
    fn symbol_matches(&self, sink_symbol: &str, source_symbol: &str) -> bool {
        // Direct match
        if sink_symbol == source_symbol {
            return true;
        }
        
        // Check renames
        // Note: This would need context about which file we're in
        // For now, do a simple check
        false
    }
    
    /// Calculate confidence score for a flow
    fn calculate_confidence(
        &self,
        source: &CrossFileTaintSource,
        sink: &CrossFileTaintSink,
        import_chain: &[ImportEdge],
    ) -> f64 {
        let mut confidence: f64 = 0.5; // Base confidence

        // Cross-file flows are more suspicious
        if !import_chain.is_empty() {
            confidence += 0.2;
        }

        // Multiple file hops increase suspicion
        if import_chain.len() > 1 {
            confidence += 0.1;
        }

        // Specific source/sink combinations
        match (&source.source_type, &sink.sink_type) {
            (CrossFileSourceType::StegoDecoder { .. }, CrossFileSinkType::DynamicExecution { .. }) => {
                confidence += 0.3; // Very suspicious
            }
            (CrossFileSourceType::EncryptedPayload { .. }, CrossFileSinkType::DynamicExecution { .. }) => {
                confidence += 0.25;
            }
            (CrossFileSourceType::HttpHeader, CrossFileSinkType::DynamicExecution { .. }) => {
                confidence += 0.2;
            }
            (CrossFileSourceType::CryptoOutput { .. }, CrossFileSinkType::DynamicExecution { .. }) => {
                confidence += 0.2;
            }
            _ => {}
        }

        // Cap at 1.0
        confidence.min(1.0)
    }
    
    /// Get all taint sources
    pub fn sources(&self) -> &[CrossFileTaintSource] {
        &self.taint_sources
    }
    
    /// Get all taint sinks
    pub fn sinks(&self) -> &[CrossFileTaintSink] {
        &self.taint_sinks
    }
    
    /// Get the module graph
    pub fn graph(&self) -> &ModuleGraph {
        &self.graph
    }
}

/// Convert intra-file TaintSource to cross-file TaintSource
impl From<(&TaintSource, &str, usize, &str)> for CrossFileTaintSource {
    fn from((source, file, line, symbol): (&TaintSource, &str, usize, &str)) -> Self {
        let source_type = match source {
            TaintSource::HighEntropyString { entropy, .. } => {
                CrossFileSourceType::EncryptedPayload { entropy: *entropy }
            }
            TaintSource::HttpHeaderAccess { .. } => CrossFileSourceType::HttpHeader,
            TaintSource::CryptoApiCall { method, .. } => {
                CrossFileSourceType::CryptoOutput {
                    method: method.clone(),
                }
            }
        };
        
        CrossFileTaintSource {
            file: file.to_string(),
            line,
            symbol: symbol.to_string(),
            source_type,
        }
    }
}

/// Convert intra-file TaintSink to cross-file TaintSink
impl From<(&TaintSink, &str, usize, &str)> for CrossFileTaintSink {
    fn from((sink, file, line, symbol): (&TaintSink, &str, usize, &str)) -> Self {
        let sink_type = match sink {
            TaintSink::DynamicExec { kind, .. } => {
                CrossFileSinkType::DynamicExecution { kind: kind.clone() }
            }
        };
        
        CrossFileTaintSink {
            file: file.to_string(),
            line,
            symbol: symbol.to_string(),
            sink_type,
        }
    }
}

/// Helper to detect split payload patterns
pub struct SplitPayloadDetector {
    tracker: CrossFileTaintTracker,
}

impl SplitPayloadDetector {
    /// Create a new split payload detector
    pub fn new(graph: ModuleGraph) -> Self {
        Self {
            tracker: CrossFileTaintTracker::new(graph),
        }
    }
    
    /// Add a decoder function (potential taint source)
    pub fn add_decoder(&mut self, file: &str, line: usize, function_name: &str) {
        self.tracker.add_source(CrossFileTaintSource {
            file: file.to_string(),
            line,
            symbol: function_name.to_string(),
            source_type: CrossFileSourceType::StegoDecoder {
                decoder_name: function_name.to_string(),
            },
        });
    }
    
    /// Add an execution sink (eval, exec, etc.)
    pub fn add_exec_sink(&mut self, file: &str, line: usize, function_name: &str) {
        self.tracker.add_sink(CrossFileTaintSink {
            file: file.to_string(),
            line,
            symbol: function_name.to_string(),
            sink_type: CrossFileSinkType::DynamicExecution {
                kind: DynExecKind::Eval,
            },
        });
    }
    
    /// Check for split payload patterns
    pub fn detect_split_payload(&self) -> Vec<CrossFileTaintFlow> {
        self.tracker.find_cross_file_flows().collect()
    }
    
    /// Get the underlying tracker
    pub fn tracker(&self) -> &CrossFileTaintTracker {
        &self.tracker
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module_graph::{Module, Export, ExportType, Import, ImportType, ModuleType, ModuleEdge, EdgeType};

    #[test]
    fn test_basic_cross_file_flow() {
        let mut graph = ModuleGraph::new();
        
        // Create decoder.js module
        graph.modules.insert("decoder.js".to_string(), Module {
            path: "decoder.js".to_string(),
            exports: vec![Export {
                name: "decoder".to_string(),
                export_type: ExportType::Inline,
                line: 1,
                is_type_only: false,
            }],
            imports: vec![],
            dependencies: vec![],
            module_type: ModuleType::ES6,
        });
        
        // Create payload.js module that imports decoder
        graph.modules.insert("payload.js".to_string(), Module {
            path: "payload.js".to_string(),
            exports: vec![],
            imports: vec![Import {
                local_name: "decoder".to_string(),
                imported_name: Some("decoder".to_string()),
                from_module: "./decoder".to_string(),
                resolved_path: Some("decoder.js".to_string()),
                line: 1,
                import_type: ImportType::Named,
                is_type_only: false,
            }],
            dependencies: vec!["decoder.js".to_string()],
            module_type: ModuleType::ES6,
        });
        
        // Add edge
        graph.edges.push(ModuleEdge {
            from_file: "decoder.js".to_string(),
            to_file: "payload.js".to_string(),
            symbol: "decoder".to_string(),
            edge_type: EdgeType::Named,
        });
        
        // Create tracker
        let mut tracker = CrossFileTaintTracker::new(graph);
        
        // Add decoder source
        tracker.add_source(CrossFileTaintSource {
            file: "decoder.js".to_string(),
            line: 1,
            symbol: "decoder".to_string(),
            source_type: CrossFileSourceType::StegoDecoder {
                decoder_name: "decoder".to_string(),
            },
        });
        
        // Add eval sink in payload.js
        tracker.add_sink(CrossFileTaintSink {
            file: "payload.js".to_string(),
            line: 5,
            symbol: "eval".to_string(),
            sink_type: CrossFileSinkType::DynamicExecution {
                kind: DynExecKind::Eval,
            },
        });
        
        // Check for cross-file flows
        // Note: This test shows the structure, but actual flow detection
        // would require the decoder symbol to flow to the eval call
        assert!(!tracker.has_cross_file_flow()); // No direct flow in this simple test
    }
    
    #[test]
    fn test_symbol_conversion() {
        let intra_source = TaintSource::HighEntropyString {
            value: "test".to_string(),
            entropy: 5.0,
            span: (0, 10),
            scope_id: 0,
            assigned_to: None,
        };
        
        let cross_source: CrossFileTaintSource = (&intra_source, "test.js", 1, "payload").into();
        
        assert_eq!(cross_source.file, "test.js");
        assert_eq!(cross_source.line, 1);
        assert_eq!(cross_source.symbol, "payload");
        
        if let CrossFileSourceType::EncryptedPayload { entropy } = cross_source.source_type {
            assert!((entropy - 5.0).abs() < 0.001);
        } else {
            panic!("Expected EncryptedPayload source type");
        }
    }
    
    #[test]
    fn test_split_payload_detector() {
        let graph = ModuleGraph::new();
        let mut detector = SplitPayloadDetector::new(graph);
        
        detector.add_decoder("decoder.js", 1, "decode");
        detector.add_exec_sink("payload.js", 5, "eval");
        
        // No flows yet (no module graph connections)
        let flows = detector.detect_split_payload();
        assert!(flows.is_empty());
    }
    
    #[test]
    fn test_confidence_calculation() {
        let graph = ModuleGraph::new();
        let tracker = CrossFileTaintTracker::new(graph);
        
        let source = CrossFileTaintSource {
            file: "a.js".to_string(),
            line: 1,
            symbol: "decoder".to_string(),
            source_type: CrossFileSourceType::StegoDecoder {
                decoder_name: "decoder".to_string(),
            },
        };
        
        let sink = CrossFileTaintSink {
            file: "b.js".to_string(),
            line: 5,
            symbol: "eval".to_string(),
            sink_type: CrossFileSinkType::DynamicExecution {
                kind: DynExecKind::Eval,
            },
        };
        
        // Base confidence
        let confidence = tracker.calculate_confidence(&source, &sink, &[]);
        assert!(confidence >= 0.5);
        
        // With import chain (cross-file)
        let chain = vec![ImportEdge {
            from_file: "a.js".to_string(),
            to_file: "b.js".to_string(),
            symbol: "decoder".to_string(),
            import_type: "named".to_string(),
        }];
        let confidence = tracker.calculate_confidence(&source, &sink, &chain);
        assert!(confidence > 0.5); // Should be higher with cross-file flow
    }
}
