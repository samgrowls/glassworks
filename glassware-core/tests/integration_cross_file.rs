//! Integration Tests for Cross-File Taint Tracking
//!
//! Tests the module graph construction and cross-file taint flow detection
//! using real fixture files.

#[cfg(feature = "semantic")]
mod cross_file_tests {
    use glassware_core::module_graph::{ModuleGraph, ModuleType, ImportType};
    use glassware_core::cross_file_taint::{CrossFileTaintTracker, CrossFileTaintSource, CrossFileSourceType, CrossFileTaintSink, CrossFileSinkType};
    use glassware_core::taint::DynExecKind;
    use std::path::Path;
    use std::fs;

    const FIXTURES_DIR: &str = "tests/fixtures/cross_file";

    #[test]
    fn test_module_graph_construction_es6() {
        let mut graph = ModuleGraph::new();
        
        let decoder_path = Path::new(FIXTURES_DIR).join("decoder.js");
        let content = fs::read_to_string(&decoder_path).expect("Failed to read decoder.js");
        
        graph.add_file(&decoder_path.to_string_lossy(), &content);
        
        let module = graph.get_module(&decoder_path.to_string_lossy()).expect("Module not found");
        
        // Should detect ES6 module type
        assert_eq!(module.module_type, ModuleType::ES6);
        
        // Should have exports
        assert!(!module.exports.is_empty());
        assert!(module.exports.iter().any(|e| e.name == "decoder"));
        assert!(module.exports.iter().any(|e| e.name == "bytesToString"));
        assert!(module.exports.iter().any(|e| e.name == "decodePayload"));
        
        // Should have no imports (decoder is a leaf module)
        assert!(module.imports.is_empty());
    }

    #[test]
    fn test_module_graph_construction_with_imports() {
        let mut graph = ModuleGraph::new();
        
        // Add both modules
        let decoder_path = Path::new(FIXTURES_DIR).join("decoder.js");
        let payload_path = Path::new(FIXTURES_DIR).join("payload.js");
        
        let decoder_content = fs::read_to_string(&decoder_path).expect("Failed to read decoder.js");
        let payload_content = fs::read_to_string(&payload_path).expect("Failed to read payload.js");
        
        graph.add_file(&decoder_path.to_string_lossy(), &decoder_content);
        graph.add_file(&payload_path.to_string_lossy(), &payload_content);
        
        // Check payload module
        let payload_module = graph.get_module(&payload_path.to_string_lossy()).expect("Module not found");
        
        // Should have imports
        assert!(!payload_module.imports.is_empty());
        assert!(payload_module.imports.iter().any(|i| i.local_name == "decoder"));
        assert!(payload_module.imports.iter().any(|i| i.local_name == "decodePayload"));
        
        // Check import types
        let decoder_import = payload_module.imports.iter().find(|i| i.local_name == "decoder").unwrap();
        assert_eq!(decoder_import.import_type, ImportType::Named);
    }

    #[test]
    fn test_module_graph_commonjs() {
        let mut graph = ModuleGraph::new();
        
        let utils_path = Path::new(FIXTURES_DIR).join("utils.js");
        let content = fs::read_to_string(&utils_path).expect("Failed to read utils.js");
        
        graph.add_file(&utils_path.to_string_lossy(), &content);
        
        let module = graph.get_module(&utils_path.to_string_lossy()).expect("Module not found");
        
        // Should detect CommonJS module type
        assert_eq!(module.module_type, ModuleType::CommonJS);
        
        // Should have require imports
        assert!(module.imports.iter().any(|i| i.import_type == ImportType::Require));
    }

    #[test]
    fn test_module_graph_typescript() {
        let mut graph = ModuleGraph::new();
        
        let encoder_path = Path::new(FIXTURES_DIR).join("encoder.ts");
        let content = fs::read_to_string(&encoder_path).expect("Failed to read encoder.ts");
        
        graph.add_file(&encoder_path.to_string_lossy(), &content);
        
        let module = graph.get_module(&encoder_path.to_string_lossy()).expect("Module not found");
        
        // Should detect TypeScript module type
        assert_eq!(module.module_type, ModuleType::TypeScript);
        
        // Should have exports
        assert!(module.exports.iter().any(|e| e.name == "encodeToVS"));
        assert!(module.exports.iter().any(|e| e.name == "createStegoPayload"));
    }

    #[test]
    fn test_find_importers() {
        let mut graph = ModuleGraph::new();
        
        let decoder_path = Path::new(FIXTURES_DIR).join("decoder.js");
        let payload_path = Path::new(FIXTURES_DIR).join("payload.js");
        
        let decoder_content = fs::read_to_string(&decoder_path).expect("Failed to read decoder.js");
        let payload_content = fs::read_to_string(&payload_path).expect("Failed to read payload.js");
        
        graph.add_file(&decoder_path.to_string_lossy(), &decoder_content);
        graph.add_file(&payload_path.to_string_lossy(), &payload_content);
        
        // Find files that import 'decoder' from decoder.js
        let importers = graph.find_importers("decoder", &decoder_path.to_string_lossy());
        
        // payload.js should import decoder
        assert!(!importers.is_empty());
        assert!(importers.iter().any(|p| p.contains("payload.js")));
    }

    #[test]
    fn test_dependency_chain() {
        let mut graph = ModuleGraph::new();

        let decoder_path = Path::new(FIXTURES_DIR).join("decoder.js");
        let payload_path = Path::new(FIXTURES_DIR).join("payload.js");

        let decoder_content = fs::read_to_string(&decoder_path).expect("Failed to read decoder.js");
        let payload_content = fs::read_to_string(&payload_path).expect("Failed to read payload.js");

        let decoder_path_str = decoder_path.to_string_lossy().to_string();
        let payload_path_str = payload_path.to_string_lossy().to_string();

        graph.add_file(&decoder_path_str, &decoder_content);
        graph.add_file(&payload_path_str, &payload_content);

        // Get dependency chain from payload.js
        let chain = graph.get_dependency_chain(&payload_path_str);

        // Chain should include both payload.js and decoder.js
        assert!(chain.iter().any(|p| p.contains("payload.js")));
        assert!(chain.iter().any(|p| p.contains("decoder.js")));
    }

    #[test]
    fn test_cross_file_taint_tracker_basic() {
        let mut graph = ModuleGraph::new();
        
        let decoder_path = Path::new(FIXTURES_DIR).join("decoder.js");
        let payload_path = Path::new(FIXTURES_DIR).join("payload.js");
        
        let decoder_content = fs::read_to_string(&decoder_path).expect("Failed to read decoder.js");
        let payload_content = fs::read_to_string(&payload_path).expect("Failed to read payload.js");
        
        graph.add_file(&decoder_path.to_string_lossy(), &decoder_content);
        graph.add_file(&payload_path.to_string_lossy(), &payload_content);
        
        let mut tracker = CrossFileTaintTracker::new(graph);
        
        // Add decoder as a taint source
        tracker.add_source(CrossFileTaintSource {
            file: decoder_path.to_string_lossy().to_string(),
            line: 8,
            symbol: "decoder".to_string(),
            source_type: CrossFileSourceType::StegoDecoder {
                decoder_name: "decoder".to_string(),
            },
        });
        
        // Add eval as a taint sink in payload.js
        tracker.add_sink(CrossFileTaintSink {
            file: payload_path.to_string_lossy().to_string(),
            line: 12,
            symbol: "eval".to_string(),
            sink_type: CrossFileSinkType::DynamicExecution {
                kind: DynExecKind::Eval,
            },
        });
        
        // The tracker should be set up correctly
        assert_eq!(tracker.sources().len(), 1);
        assert_eq!(tracker.sinks().len(), 1);
    }

    #[test]
    fn test_split_payload_detector() {
        use glassware_core::cross_file_taint::SplitPayloadDetector;
        
        let graph = ModuleGraph::new();
        let mut detector = SplitPayloadDetector::new(graph);
        
        // Register decoder in one file
        detector.add_decoder("decoder.js", 8, "decoder");
        
        // Register eval sink in another file
        detector.add_exec_sink("payload.js", 12, "eval");
        
        // Without module graph connections, no flows will be detected
        // This test verifies the detector structure works
        let flows = detector.detect_split_payload();
        assert!(flows.is_empty()); // No graph connections
    }

    #[test]
    fn test_has_path() {
        let mut graph = ModuleGraph::new();
        
        // Manually create modules and edges
        graph.modules.insert("a.js".to_string(), glassware_core::module_graph::Module {
            path: "a.js".to_string(),
            exports: vec![],
            imports: vec![],
            dependencies: vec![],
            module_type: ModuleType::ES6,
        });
        graph.modules.insert("b.js".to_string(), glassware_core::module_graph::Module {
            path: "b.js".to_string(),
            exports: vec![],
            imports: vec![],
            dependencies: vec![],
            module_type: ModuleType::ES6,
        });
        
        graph.edges.push(glassware_core::module_graph::ModuleEdge {
            from_file: "a.js".to_string(),
            to_file: "b.js".to_string(),
            symbol: "foo".to_string(),
            edge_type: glassware_core::module_graph::EdgeType::Named,
        });
        
        // Should have path from a.js to b.js
        assert!(graph.has_path("a.js", "b.js"));
        
        // Should not have path in reverse direction
        assert!(!graph.has_path("b.js", "a.js"));
    }
}
