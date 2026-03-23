# Glassware Plugin Architecture Design

**Document Status:** Draft  
**Version:** 1.0  
**Date:** March 23, 2026  
**Author:** Glassworks Research Team  

---

## Executive Summary

This document proposes a plugin architecture for Glassware detectors, enabling community contributions and easier addition of new attack vector detectors. After analyzing four architectural options against our requirements, we recommend **Option A: Rust Crate Plugins** as the primary approach, with **Option D: Configuration-Driven Rules** as a complementary lightweight option.

**Key Recommendations:**
- **Primary:** Rust crate-based plugins (separate crates implementing the `Detector` trait)
- **Secondary:** YAML configuration for simple pattern-based detectors
- **Deferred:** WASM and dynamic library loading (complexity outweighs benefits)

**Timeline Estimate:** 6-8 weeks for initial implementation  
**Migration Complexity:** Low - existing detectors require minimal changes  

---

## Table of Contents

1. [Current Architecture Analysis](#1-current-architecture-analysis)
2. [Requirements & Goals](#2-requirements--goals)
3. [Plugin Architecture Options](#3-plugin-architecture-options)
4. [Comparative Analysis](#4-comparative-analysis)
5. [Examples from Other Projects](#5-examples-from-other-projects)
6. [Recommended Approach](#6-recommended-approach)
7. [API Design](#7-api-design)
8. [Example Plugin](#8-example-plugin)
9. [Migration Plan](#9-migration-plan)
10. [Implementation Timeline](#10-implementation-timeline)
11. [Security Considerations](#11-security-considerations)
12. [Appendix: Research Notes](#12-appendix-research-notes)

---

## 1. Current Architecture Analysis

### 1.1 Detector Trait Overview

**Location:** `/home/property.sightlines/samgrowls/glassworks/glassware-core/src/detector.rs`

The current `Detector` trait defines the interface for all detectors:

```rust
pub trait Detector: Send + Sync {
    /// Get detector name
    fn name(&self) -> &str;

    /// Get detector tier (default: Tier1)
    fn tier(&self) -> DetectorTier {
        DetectorTier::Tier1Primary
    }

    /// Run detection on the provided IR
    fn detect(&self, ir: &FileIR) -> Vec<Finding>;

    /// Get detector metadata (optional)
    fn metadata(&self) -> DetectorMetadata { ... }

    /// Get computational cost (1-10)
    fn cost(&self) -> u8 { 5 }

    /// Get signal strength (1-10)
    fn signal_strength(&self) -> u8 { 5 }

    /// Get prerequisites (DAG dependencies)
    fn prerequisites(&self) -> Vec<&'static str> { vec![] }

    /// Check if should short-circuit execution
    fn should_short_circuit(&self, _findings: &[Finding]) -> bool { false }

    /// Check if detector should run based on other findings
    fn should_run(&self, _other_findings: &[Finding]) -> bool { true }
}
```

### 1.2 Existing Detectors

**Location:** `/home/property.sightlines/samgrowls/glassworks/glassware-core/src/detectors/`

| Detector | File | Complexity | Tier | Cost | Signal |
|----------|------|------------|------|------|--------|
| `BidiDetector` | `bidi.rs` | Low | Tier1 | 1 | 9 |
| `HomoglyphDetector` | `homoglyph.rs` | Medium | Tier1 | 2 | 8 |
| `InvisibleCharDetector` | `invisible.rs` | Low | Tier1 | 1 | 9 |
| `GlasswareDetector` | `glassware.rs` | High | Tier2 | 5 | 7 |
| `SocketIOC2Detector` | `socketio_c2.rs` | Medium | Tier3 | 4 | 8 |
| `BrowserKillDetector` | `browser_kill.rs` | Low | Tier2 | 2 | 7 |
| `ExfilSchemaDetector` | `exfil_schema.rs` | Medium | Tier2 | 3 | 6 |
| `TypoAttributionDetector` | `typo_attribution.rs` | Low | Tier2 | 2 | 7 |
| `UnicodeTagDetector` | `tags.rs` | Low | Tier1 | 1 | 8 |

### 1.3 What Makes a Detector Easy/Hard to Write

**Easy Detectors (Low Complexity):**
- Single-pass character scanning (Bidi, Invisible)
- Simple pattern matching with regex
- No external dependencies beyond glassware-core
- Clear, well-defined attack signatures

**Hard Detectors (High Complexity):**
- Multi-pass analysis (GlasswareDetector)
- Requires AST/semantic analysis
- Cross-file correlation needed
- Behavioral/heuristic detection (SocketIOC2)
- Requires configuration tuning

**Common Patterns:**
```rust
// Pattern 1: Character-by-character scanning
for (line_num, line) in content.lines().enumerate() {
    for (col_num, ch) in line.chars().enumerate() {
        if is_suspicious(ch) {
            findings.push(finding);
        }
    }
}

// Pattern 2: Regex-based detection
for (line_num, line) in content.lines().enumerate() {
    for pattern in PATTERNS {
        if let Some(m) = pattern.find(line) {
            findings.push(finding);
        }
    }
}

// Pattern 3: Multi-signal scoring (SocketIOC2)
let groups_matched = [has_group_a, has_group_b, has_group_c]
    .iter().filter(|&&x| x).count();
if groups_matched >= 3 { /* HIGH */ }
```

### 1.4 Engine Registration

**Location:** `/home/property.sightlines/samgrowls/glassworks/glassware-core/src/engine.rs`

Detectors are currently registered in `ScanEngine::default_detectors()`:

```rust
pub fn default_detectors() -> Self {
    let mut engine = Self::new();
    engine.register(Box::new(UnicodeDetector::new()));
    engine.register(Box::new(EncryptedPayloadDetector::new()));
    engine.register(Box::new(HeaderC2Detector::new()));
    // ... more detectors
    engine
}
```

**Key Observation:** All detectors are compiled into the same crate. Adding a new detector requires:
1. Creating a new `.rs` file in `src/detectors/`
2. Adding `pub mod` declaration in `mod.rs`
3. Registering in `engine.rs`
4. Rebuilding the entire library

---

## 2. Requirements & Goals

### 2.1 Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| FR1 | Enable adding new detectors without modifying glassware-core | Must Have |
| FR2 | Support community-contributed detectors | Must Have |
| FR3 | Maintain type safety and compile-time checks | Must Have |
| FR4 | Support detector versioning | Should Have |
| FR5 | Enable/disable detectors at runtime | Should Have |
| FR6 | Support detector configuration | Should Have |
| FR7 | Enable hot-reloading of detectors | Nice to Have |
| FR8 | Support detector discovery and listing | Should Have |

### 2.2 Non-Functional Requirements

| ID | Requirement | Target |
|----|-------------|--------|
| NFR1 | Performance overhead | <5% vs compiled-in |
| NFR2 | Memory overhead | <10MB per plugin |
| NFR3 | Plugin load time | <100ms per plugin |
| NFR4 | Security | No arbitrary code execution |
| NFR5 | Ease of development | New detector in <1 hour |
| NFR6 | Distribution | Publishable to crates.io |

### 2.3 Long-Term Vision

- **Community Ecosystem:** Third-party detectors for emerging attack vectors
- **Specialized Detectors:** Industry-specific patterns (finance, healthcare, etc.)
- **Rapid Response:** Quick deployment of detectors for new threats
- **Broader Coverage:** 50+ detectors covering OWASP Top 10, supply chain attacks

---

## 3. Plugin Architecture Options

### 3.1 Option A: Rust Crate Plugins

**Description:** Each detector is a separate Rust crate that implements the `Detector` trait. Plugins are compiled separately and linked at build time or loaded dynamically.

**Architecture:**
```
glassware-core/          # Core library with Detector trait
├── src/
│   ├── detector.rs      # Trait definition
│   ├── finding.rs       # Finding types
│   └── ir.rs            # Intermediate representation
│
glassware-detector-bidi/ # Separate crate
├── Cargo.toml
└── src/
    └── lib.rs           # Implements Detector trait

glassware-detector-homoglyph/  # Another separate crate
├── Cargo.toml
└── src/
    └── lib.rs
```

**Example Plugin:**
```rust
// glassware-detector-bidi/src/lib.rs
use glassware_core::{Detector, DetectorTier, FileIR, Finding, DetectorMetadata};

pub struct BidiDetector {
    config: BidiConfig,
}

impl BidiDetector {
    pub fn new() -> Self { /* ... */ }
}

impl Detector for BidiDetector {
    fn name(&self) -> &str { "bidi" }
    fn tier(&self) -> DetectorTier { DetectorTier::Tier1Primary }
    fn detect(&self, ir: &FileIR) -> Vec<Finding> { /* ... */ }
    fn metadata(&self) -> DetectorMetadata {
        DetectorMetadata {
            name: "bidi".to_string(),
            version: "1.0.0".to_string(),
            description: "Detects bidirectional text overrides".to_string(),
        }
    }
}

// Plugin registration macro
glassware_plugin::register_plugin!(BidiDetector);
```

**Pros:**
- Full type safety at compile time
- Access to full Rust ecosystem
- No performance overhead
- Easy to version and distribute via crates.io
- IDE support for plugin development

**Cons:**
- Requires Rust toolchain for plugin development
- Plugins must be compiled (no scripting)
- Version compatibility management needed

### 3.2 Option B: WASM Plugins

**Description:** Detectors are compiled to WebAssembly and loaded at runtime by a WASM host embedded in glassware-core.

**Architecture:**
```
┌─────────────────────────────────────────────────────────┐
│                    glassware-core                        │
│  ┌─────────────────────────────────────────────────┐    │
│  │              WASM Runtime (wasmtime)             │    │
│  │  ┌──────────────┐  ┌──────────────┐            │    │
│  │  │ bidi.wasm    │  │ homoglyph.wasm│            │    │
│  │  └──────────────┘  └──────────────┘            │    │
│  └─────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

**Example Plugin (Rust -> WASM):**
```rust
// glassware-detector-bidi/src/lib.rs
use glassware_plugin_wasm::{DetectorTrait, FileIR, Finding};

#[no_mangle]
pub extern "C" fn detector_name() -> *const u8 {
    b"bidi\0" as *const u8
}

#[no_mangle]
pub extern "C" fn detector_detect(ir_ptr: *mut FileIR) -> *mut Finding {
    // WASM FFI implementation
}

glassware_wasm_plugin::export_plugin!(BidiDetector);
```

**Pros:**
- True runtime loading without recompilation
- Language agnostic (could write plugins in Rust, C++, AssemblyScript)
- Sandboxed execution (memory safety)
- Hot-reloading support

**Cons:**
- Significant complexity in WASM host implementation
- FFI overhead for data marshaling
- Limited ecosystem for WASM plugins
- Debugging is harder
- Performance overhead (5-15%)

### 3.3 Option C: Dynamic Library Plugins (.so/.dylib/.dll)

**Description:** Detectors are compiled as dynamic libraries and loaded at runtime using `libloading`.

**Architecture:**
```
┌─────────────────────────────────────────────────────────┐
│                    glassware-core                        │
│  ┌─────────────────────────────────────────────────┐    │
│  │           Plugin Loader (libloading)             │    │
│  │  ┌──────────────┐  ┌──────────────┐            │    │
│  │  │ libbidi.so   │  │ libhomoglyph.so│           │    │
│  │  └──────────────┘  └──────────────┘            │    │
│  └─────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

**Example Plugin:**
```rust
// glassware-detector-bidi/src/lib.rs
use glassware_core::{Detector, FileIR, Finding};

#[no_mangle]
pub extern "C" fn create_detector() -> *mut dyn Detector {
    Box::into_raw(Box::new(BidiDetector::new()))
}

#[no_mangle]
pub extern "C" fn destroy_detector(ptr: *mut dyn Detector) {
    unsafe { drop(Box::from_raw(ptr)); }
}
```

**Pros:**
- Runtime loading without recompilation
- No WASM complexity
- Native performance

**Cons:**
- Platform-specific (.so, .dylib, .dll)
- ABI compatibility issues across Rust versions
- Security risk (arbitrary native code execution)
- Versioning is complex (symbol conflicts)
- Harder to distribute

### 3.4 Option D: Configuration-Driven Rules

**Description:** Simple detectors are defined via YAML/JSON configuration files. Complex detectors still use Rust.

**Architecture:**
```yaml
# detectors/bidi.yaml
name: bidi
description: Detects bidirectional text overrides
tier: tier1
cost: 1
signal_strength: 9

rules:
  - type: unicode_range
    range: "U+202A-U+202E"
    severity: critical
    description: "Bidirectional override character"

  - type: unicode_range
    range: "U+2066-U+2069"
    severity: high
    description: "Isolate override character"
```

**Runtime:**
```rust
let config = DetectorConfig::load("detectors/bidi.yaml")?;
let detector = ConfiguredDetector::new(config);
engine.register(Box::new(detector));
```

**Pros:**
- Extremely easy to write new detectors (no Rust needed)
- No compilation required
- Hot-reloading trivial
- Safe (no arbitrary code execution)
- Great for pattern-based detection

**Cons:**
- Limited to pattern matching (no complex logic)
- Less performant than native code
- No type safety for configurations
- Cannot implement behavioral detectors

---

## 4. Comparative Analysis

### 4.1 Evaluation Matrix

| Criteria | Option A: Crates | Option B: WASM | Option C: dylib | Option D: Config |
|----------|------------------|----------------|-----------------|------------------|
| **Ease of Writing** | Medium | Hard | Medium | Very Easy |
| **Ease of Updating** | Easy (cargo update) | Easy | Hard (rebuild) | Very Easy |
| **Performance** | Native (0% overhead) | 5-15% overhead | Native | 10-20% overhead |
| **Security** | High (compile-time) | Very High (sandbox) | Low (native code) | Very High (no code) |
| **Distribution** | crates.io | Custom registry | Manual | Git/config |
| **Version Compatibility** | SemVer (cargo) | WASI versioning | ABI compatibility | Schema versioning |
| **Hot Reloading** | No | Yes | Yes | Yes |
| **Language Flexibility** | Rust only | Any -> WASM | Rust/C/C++ | None (config) |
| **Debugging** | Excellent | Poor | Good | Good |
| **Implementation Complexity** | Low | Very High | Medium | Low |

### 4.2 Scoring (1-5, 5=best)

| Criteria | A | B | C | D |
|----------|---|---|---|---|
| Developer Experience | 4 | 2 | 3 | 5 |
| Performance | 5 | 3 | 5 | 3 |
| Security | 4 | 5 | 2 | 5 |
| Maintainability | 5 | 3 | 3 | 4 |
| Flexibility | 3 | 5 | 4 | 2 |
| **Total** | **21** | **18** | **17** | **19** |

### 4.3 Recommendation Summary

**Primary: Option A (Rust Crates)**
- Best balance of safety, performance, and ease of use
- Leverages existing Rust ecosystem
- Minimal implementation complexity

**Complementary: Option D (Configuration)**
- Perfect for simple pattern-based detectors
- Enables non-Rust developers to contribute
- Low barrier to entry

**Not Recommended:**
- **Option B (WASM):** Too complex for current needs
- **Option C (dylib):** Security and compatibility concerns

---

## 5. Examples from Other Projects

### 5.1 Semgrep Rules (YAML-based)

**Approach:** Pattern matching rules defined in YAML

```yaml
# Example Semgrep rule
rules:
  - id: dangerous-eval
    pattern: eval($USER_INPUT)
    message: "User input passed to eval()"
    severity: ERROR
    languages: [javascript, python]
```

**Lessons:**
- YAML is accessible to security researchers
- Pattern matching covers 80% of use cases
- Complex logic still requires custom code

### 5.2 Clippy Lints (Rust Crate-based)

**Approach:** Each lint is Rust code implementing a trait

```rust
// Example Clippy lint
declare_clippy_lint! {
    pub UNUSED_UNIT,
    style,
    "unnecessary unit type"
}

impl LateLintPass for UnusedUnit {
    fn check_stmt(&mut self, cx: &LateContext<'_>, stmt: &Stmt<'_>) {
        // Lint logic
    }
}
```

**Lessons:**
- Type safety prevents many bugs
- Compile-time checks catch errors early
- Versioning via Rust edition system

### 5.3 ESLint Plugins (JavaScript Modules)

**Approach:** NPM packages exporting rule functions

```javascript
// eslint-plugin-security
module.exports = {
  rules: {
    'detect-eval-with-expression': {
      meta: { docs: { description: 'eval with expression' } },
      create(context) {
        return {
          CallExpression(node) {
            if (node.callee.name === 'eval') {
              context.report({ node, message: 'eval used' });
            }
          }
        };
      }
    }
  }
};
```

**Lessons:**
- Easy to publish and share (npm)
- Runtime errors possible (no type safety)
- Large ecosystem due to low barrier

### 5.4 Wireshark Dissectors (C + Lua)

**Approach:** Native C dissectors + Lua scripting

**Lessons:**
- Hybrid approach works well
- Lua for rapid prototyping
- C for performance-critical dissectors

---

## 6. Recommended Approach

### 6.1 Hybrid Architecture

We recommend a **hybrid approach** combining Options A and D:

```
┌─────────────────────────────────────────────────────────┐
│                    glassware-core                        │
│                                                          │
│  ┌──────────────────────────────────────────────────┐   │
│  │              Plugin Registry                      │   │
│  └──────────────────────────────────────────────────┘   │
│         │                              │                 │
│         ▼                              ▼                 │
│  ┌─────────────────┐        ┌─────────────────┐        │
│  │  Rust Plugins   │        │  Config Rules   │        │
│  │  (complex logic)│        │  (simple patterns)│       │
│  │                 │        │                  │        │
│  │ - bidi          │        │ - unicode_ranges │        │
│  │ - homoglyph     │        │ - regex_patterns │        │
│  │ - socketio_c2   │        │ - keyword_match  │        │
│  └─────────────────┘        └─────────────────┘        │
└─────────────────────────────────────────────────────────┘
```

### 6.2 Architecture Diagram

```
                                    ┌─────────────────────┐
                                    │   Detector Plugin   │
                                    │   (Separate Crate)  │
                                    │                     │
                                    │  - Cargo.toml       │
                                    │  - src/lib.rs       │
                                    │  - tests/           │
                                    └──────────┬──────────┘
                                               │
                                               │ implements
                                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                    glassware-core                                │
│                                                                  │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │   Detector  │    │   FileIR    │    │   Finding   │         │
│  │    Trait    │    │             │    │             │         │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘         │
│         │                  │                  │                 │
│         └──────────────────┼──────────────────┘                 │
│                            │                                     │
│  ┌─────────────────────────▼─────────────────────────┐         │
│  │              ScanEngine                            │         │
│  │  - Plugin loader                                   │         │
│  │  - DAG scheduler                                   │         │
│  │  - Result aggregator                               │         │
│  └─────────────────────────┬─────────────────────────┘         │
│                            │                                     │
└────────────────────────────┼─────────────────────────────────────┘
                             │
                             │ uses
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                    glassware-orchestrator                        │
│  - Loads plugins from config                                     │
│  - Manages plugin lifecycle                                      │
│  - Reports findings                                              │
└─────────────────────────────────────────────────────────────────┘
```

### 6.3 Why This Approach?

**Technical Rationale:**
1. **Type Safety:** Rust's type system prevents plugin errors at compile time
2. **Performance:** No FFI or interpretation overhead
3. **Ecosystem:** Leverages crates.io for distribution
4. **Security:** No arbitrary code execution from configs
5. **Flexibility:** Config rules for simple cases, Rust for complex

**Business Rationale:**
1. **Community:** Low barrier for config-based contributions
2. **Enterprise:** Type-safe plugins for critical detectors
3. **Maintenance:** Clear separation of concerns
4. **Evolution:** Can add WASM later if needed

---

## 7. API Design

### 7.1 Plugin Trait Extension

```rust
// glassware-core/src/plugin.rs

/// Plugin metadata for discovery and versioning
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    /// Plugin name (e.g., "glassware-detector-bidi")
    pub name: String,
    /// Plugin version (SemVer)
    pub version: String,
    /// Author information
    pub author: Option<String>,
    /// License (e.g., "MIT", "Apache-2.0")
    pub license: Option<String>,
    /// Compatible glassware-core version range
    pub glassware_version_req: String,
    /// Detector types provided by this plugin
    pub detectors: Vec<String>,
    /// Plugin description
    pub description: Option<String>,
    /// Documentation URL
    pub documentation: Option<String>,
    /// Repository URL
    pub repository: Option<String>,
}

/// Trait for plugin initialization
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;

    /// Initialize the plugin with configuration
    fn initialize(&mut self, config: PluginConfig) -> Result<(), PluginError>;

    /// Get all detectors provided by this plugin
    fn detectors(&self) -> Vec<Box<dyn Detector>>;

    /// Shutdown the plugin (cleanup resources)
    fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

/// Plugin configuration
#[derive(Debug, Clone, Default)]
pub struct PluginConfig {
    /// Plugin-specific settings
    pub settings: HashMap<String, Value>,
    /// Enable/disable flags
    pub enabled: bool,
    /// Detector-specific overrides
    pub detector_configs: HashMap<String, DetectorConfig>,
}

/// Plugin loading errors
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Version incompatibility: plugin requires {required}, core provides {provided}")]
    VersionMismatch { required: String, provided: String },

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Initialization failed: {0}")]
    InitializationFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

### 7.2 Plugin Registry

```rust
// glassware-core/src/plugin/registry.rs

/// Registry for managing loaded plugins
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn Plugin>>,
    config: RegistryConfig,
}

impl PluginRegistry {
    /// Create a new registry
    pub fn new(config: RegistryConfig) -> Self {
        Self {
            plugins: HashMap::new(),
            config,
        }
    }

    /// Register a plugin
    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<(), PluginError> {
        let metadata = plugin.metadata();
        self.plugins.insert(metadata.name.clone(), plugin);
        Ok(())
    }

    /// Load a plugin from a crate
    pub fn load_crate(&mut self, crate_name: &str) -> Result<(), PluginError> {
        // Implementation uses cargo metadata and dynamic loading
        // or compile-time registration via build script
    }

    /// Load plugins from configuration
    pub fn load_from_config(&mut self, config_path: &Path) -> Result<(), PluginError> {
        let config = PluginRegistryConfig::load(config_path)?;
        for plugin_config in config.plugins {
            self.load_crate(&plugin_config.name)?;
        }
        Ok(())
    }

    /// Get all registered detectors
    pub fn all_detectors(&self) -> Vec<Box<dyn Detector>> {
        self.plugins
            .values()
            .flat_map(|p| p.detectors())
            .collect()
    }

    /// Enable/disable a plugin
    pub fn set_enabled(&mut self, name: &str, enabled: bool) -> Result<(), PluginError> {
        if let Some(plugin) = self.plugins.get_mut(name) {
            // Update plugin state
            Ok(())
        } else {
            Err(PluginError::NotFound(name.to_string()))
        }
    }

    /// List all loaded plugins
    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        self.plugins.values().map(|p| p.metadata()).collect()
    }
}
```

### 7.3 Configuration Schema

```yaml
# glassware-plugins.yaml

# Plugin registry configuration
registry:
  # Directory to search for plugins
  plugin_dir: ~/.glassware/plugins

  # Auto-load plugins on startup
  auto_load: true

# Plugins to load
plugins:
  # Built-in plugins (compiled into glassware-core)
  - name: glassware-detector-bidi
    enabled: true
    version: "1.0"

  - name: glassware-detector-homoglyph
    enabled: true
    version: "1.0"

  # Third-party plugins
  - name: glassware-detector-supply-chain
    enabled: true
    source: crates.io
    version: "0.3"

  # Local development plugin
  - name: my-custom-detector
    enabled: false
    source: local
    path: ./detectors/my-detector

# Detector-specific configuration
detectors:
  bidi:
    severity_overrides:
      RLO: critical
      RLE: high
    enabled: true

  homoglyph:
    min_confidence: 0.8
    scripts:
      - cyrillic
      - greek
    enabled: true
```

---

## 8. Example Plugin

### 8.1 Plugin Structure

```
glassware-detector-example/
├── Cargo.toml
├── README.md
├── src/
│   └── lib.rs
├── tests/
│   └── integration_test.rs
└── config/
    └── detector.yaml
```

### 8.2 Cargo.toml

```toml
[package]
name = "glassware-detector-example"
version = "1.0.0"
edition = "2021"
license = "MIT OR Apache-2.0"
description = "Example Glassware detector plugin"
keywords = ["glassware", "security", "detector", "plugin"]

[dependencies]
glassware-core = { version = "0.5", path = "../glassware-core" }
regex = "1.10"
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"

[dev-dependencies]
tempfile = "3.9"
```

### 8.3 Plugin Implementation

```rust
// glassware-detector-example/src/lib.rs

use glassware_core::{
    Detector, DetectorMetadata, DetectorTier,
    FileIR, Finding, DetectionCategory, Severity,
    Plugin, PluginMetadata, PluginConfig, PluginError,
};
use regex::Regex;
use serde::Deserialize;

/// Example detector configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ExampleDetectorConfig {
    /// Patterns to match
    pub patterns: Vec<String>,
    /// Minimum severity to report
    pub min_severity: Severity,
    /// Enable case-insensitive matching
    pub case_insensitive: bool,
}

impl Default for ExampleDetectorConfig {
    fn default() -> Self {
        Self {
            patterns: vec![
                r"eval\s*\(".to_string(),
                r"Function\s*\(".to_string(),
            ],
            min_severity: Severity::Medium,
            case_insensitive: false,
        }
    }
}

/// Example detector implementation
pub struct ExampleDetector {
    config: ExampleDetectorConfig,
    patterns: Vec<Regex>,
}

impl ExampleDetector {
    /// Create a new example detector
    pub fn new(config: ExampleDetectorConfig) -> Result<Self, PluginError> {
        let mut patterns = Vec::new();
        for pattern_str in &config.patterns {
            let regex = if config.case_insensitive {
                Regex::new(&format!("(?i){}", pattern_str))
            } else {
                Regex::new(pattern_str)
            }.map_err(|e| PluginError::ConfigError(
                format!("Invalid regex '{}': {}", pattern_str, e)
            ))?;
            patterns.push(regex);
        }

        Ok(Self { config, patterns })
    }
}

impl Detector for ExampleDetector {
    fn name(&self) -> &str {
        "example_pattern"
    }

    fn tier(&self) -> DetectorTier {
        DetectorTier::Tier2Secondary
    }

    fn detect(&self, ir: &FileIR) -> Vec<Finding> {
        let mut findings = Vec::new();

        for (line_num, line) in ir.content().lines().enumerate() {
            for pattern in &self.patterns {
                if let Some(m) = pattern.find(line) {
                    let finding = Finding::new(
                        &ir.metadata().path,
                        line_num + 1,
                        m.start() + 1,
                        0,
                        '\0',
                        DetectionCategory::GlasswarePattern,
                        self.config.min_severity,
                        &format!("Suspicious pattern detected: {}", &line[m.start()..m.end()]),
                        "Review this code for potential malicious intent. \
                         Dynamic code execution patterns can be used to hide malicious payloads.",
                    )
                    .with_cwe_id("CWE-95")
                    .with_context(line);

                    findings.push(finding);
                }
            }
        }

        findings
    }

    fn cost(&self) -> u8 {
        3  // Medium cost - regex matching
    }

    fn signal_strength(&self) -> u8 {
        6  // Medium signal - eval can be legitimate
    }

    fn metadata(&self) -> DetectorMetadata {
        DetectorMetadata {
            name: "example_pattern".to_string(),
            version: "1.0.0".to_string(),
            description: "Example detector demonstrating plugin architecture".to_string(),
        }
    }
}

/// Example plugin wrapper
pub struct ExamplePlugin {
    metadata: PluginMetadata,
    detector: Option<ExampleDetector>,
    config: Option<PluginConfig>,
}

impl ExamplePlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name: "glassware-detector-example".to_string(),
                version: "1.0.0".to_string(),
                author: Some("Glassworks Team".to_string()),
                license: Some("MIT".to_string()),
                glassware_version_req: "^0.5".to_string(),
                detectors: vec!["example_pattern".to_string()],
                description: Some("Example plugin demonstrating the plugin architecture".to_string()),
                documentation: None,
                repository: None,
            },
            detector: None,
            config: None,
        }
    }
}

impl Plugin for ExamplePlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    fn initialize(&mut self, config: PluginConfig) -> Result<(), PluginError> {
        // Extract detector config from plugin config
        let detector_config = config.settings
            .get("detector")
            .and_then(|v| serde_yaml::from_value(v.clone()).ok())
            .unwrap_or_else(ExampleDetectorConfig::default);

        self.detector = Some(ExampleDetector::new(detector_config)?);
        self.config = Some(config);
        Ok(())
    }

    fn detectors(&self) -> Vec<Box<dyn Detector>> {
        match &self.detector {
            Some(d) => vec![Box::new(d.clone())],
            None => vec![],
        }
    }

    fn shutdown(&mut self) -> Result<(), PluginError> {
        self.detector = None;
        self.config = None;
        Ok(())
    }
}

// Plugin registration macro
glassware_plugin::register_plugin!(ExamplePlugin);
```

### 8.4 Integration Tests

```rust
// glassware-detector-example/tests/integration_test.rs

use glassware_core::{FileIR, Detector};
use glassware_detector_example::{ExampleDetector, ExampleDetectorConfig};
use std::path::Path;

#[test]
fn test_eval_detection() {
    let config = ExampleDetectorConfig::default();
    let detector = ExampleDetector::new(config).unwrap();

    let content = r#"
        function execute(code) {
            eval(code);
        }
    "#;

    let ir = FileIR::build(Path::new("test.js"), content);
    let findings = detector.detect(&ir);

    assert!(!findings.is_empty());
    assert_eq!(findings[0].line, 3);
    assert!(findings[0].description.contains("eval"));
}

#[test]
fn test_clean_code() {
    let config = ExampleDetectorConfig::default();
    let detector = ExampleDetector::new(config).unwrap();

    let content = r#"
        function add(a, b) {
            return a + b;
        }
    "#;

    let ir = FileIR::build(Path::new("test.js"), content);
    let findings = detector.detect(&ir);

    assert!(findings.is_empty());
}
```

---

## 9. Migration Plan

### 9.1 Phase 1: Foundation (Week 1-2)

**Goals:** Establish plugin infrastructure without breaking existing code

**Tasks:**
1. Create `glassware-plugin` helper crate with macros
2. Define `Plugin` and `PluginMetadata` traits
3. Implement `PluginRegistry`
4. Add plugin configuration schema
5. Write documentation for plugin developers

**Deliverables:**
- `glassware-plugin` crate (published to crates.io)
- Plugin API documentation
- Example plugin template

**Migration Impact:** None - existing detectors unchanged

### 9.2 Phase 2: Detector Extraction (Week 3-4)

**Goals:** Move existing detectors to plugin structure

**Tasks:**
1. Create plugin crates for each existing detector:
   - `glassware-detector-bidi`
   - `glassware-detector-homoglyph`
   - `glassware-detector-invisible`
   - etc.
2. Update `glassware-core` to load detectors from plugins
3. Maintain backward compatibility with built-in detectors
4. Update `ScanEngine::default_detectors()` to use plugins

**Deliverables:**
- 10 detector plugin crates
- Updated engine with plugin loading
- Migration guide for detector authors

**Migration Impact:** Low - internal refactoring, public API unchanged

### 9.3 Phase 3: Configuration Support (Week 5)

**Goals:** Add YAML configuration for simple detectors

**Tasks:**
1. Implement `ConfiguredDetector` for YAML rules
2. Add configuration file parser
3. Support unicode_range, regex_pattern, keyword_match rules
4. Add hot-reload support for config changes

**Deliverables:**
- Configuration-driven detector support
- Example YAML configurations
- Hot-reload documentation

**Migration Impact:** None - additive feature

### 9.4 Phase 4: Ecosystem (Week 6-8)

**Goals:** Enable community contributions

**Tasks:**
1. Create plugin template repository (GitHub template)
2. Set up CI/CD for plugin testing
3. Document plugin publishing process
4. Create plugin discovery mechanism
5. Write tutorial: "Writing Your First Detector"

**Deliverables:**
- `glassware-plugin-template` repository
- Plugin publishing guide
- Community contribution guidelines
- Plugin registry website (optional)

**Migration Impact:** None - ecosystem enablement

### 9.5 Migration Checklist for Existing Detectors

```markdown
## Migration Checklist: BidiDetector

- [ ] Create `glassware-detector-bidi/` directory
- [ ] Copy `bidi.rs` to `src/lib.rs`
- [ ] Create `Cargo.toml` with glassware-core dependency
- [ ] Add `Plugin` trait implementation
- [ ] Add `PluginMetadata` implementation
- [ ] Move tests to `tests/integration_test.rs`
- [ ] Create example configuration (optional)
- [ ] Update `glassware-core/Cargo.toml` (optional dependency)
- [ ] Update `engine.rs` to load from plugin
- [ ] Run all tests
- [ ] Update documentation
- [ ] Publish to crates.io (optional)
```

---

## 10. Implementation Timeline

### 10.1 Gantt Chart

```
Week 1-2: Foundation
├─ Plugin trait definition      ████████
├─ PluginRegistry implementation ████████
├─ Configuration schema          ████
└─ Documentation                 ██████

Week 3-4: Detector Extraction
├─ Bidi plugin                   ████
├─ Homoglyph plugin              ████
├─ Invisible plugin              ████
├─ Glassware plugin              ██████
├─ Other detectors               ████████
└─ Engine integration            ████

Week 5: Configuration Support
├─ Config parser                 ██████
├─ ConfiguredDetector            ██████
├─ Hot-reload                    ████
└─ Testing                       ████

Week 6-8: Ecosystem
├─ Template repository           ████
├─ CI/CD setup                   ██████
├─ Publishing guide              ████
├─ Tutorial                      ██████
└─ Community docs                ████
```

### 10.2 Milestone Dates

| Milestone | Target Date | Deliverables |
|-----------|-------------|--------------|
| M1: Foundation Complete | Week 2 | Plugin API, Registry, Docs |
| M2: All Detectors Migrated | Week 4 | 10+ plugin crates |
| M3: Config Support Complete | Week 5 | YAML detector support |
| M4: Ecosystem Ready | Week 8 | Template, CI/CD, Guide |

### 10.3 Resource Requirements

| Role | Effort | Responsibilities |
|------|--------|------------------|
| Core Developer | 4 weeks | Plugin API, Registry, Engine |
| Detector Developer | 2 weeks | Migrate existing detectors |
| Documentation | 1 week | API docs, tutorials |
| DevOps | 1 week | CI/CD, publishing |

---

## 11. Security Considerations

### 11.1 Threat Model

**Assets:**
- Detector code (integrity)
- Scan results (confidentiality, integrity)
- Plugin registry (integrity)

**Threats:**
1. Malicious plugin injected into registry
2. Plugin with vulnerable dependencies
3. Plugin exfiltrating scan results
4. Version incompatibility causing crashes

### 11.2 Mitigations

| Threat | Mitigation |
|--------|------------|
| Malicious plugin | Code review for official plugins, signing |
| Vulnerable deps | `cargo audit` in CI, dependency scanning |
| Data exfiltration | Plugin sandboxing (future), audit trail |
| Version issues | SemVer enforcement, compatibility matrix |

### 11.3 Plugin Signing (Future)

```rust
// Verify plugin signature before loading
pub fn verify_plugin_signature(
    plugin_path: &Path,
    public_key: &PublicKey,
) -> Result<bool, PluginError> {
    let signature = load_plugin_signature(plugin_path)?;
    let content = std::fs::read(plugin_path)?;
    Ok(public_key.verify(&content, &signature))
}
```

### 11.4 Security Guidelines for Plugin Authors

1. **Minimize Dependencies:** Fewer dependencies = smaller attack surface
2. **Audit Regularly:** Run `cargo audit` before publishing
3. **No Network Access:** Detectors should not make network calls
4. **No Filesystem Access:** Beyond reading scanned files
5. **Error Handling:** Don't expose sensitive info in errors
6. **Version Pinning:** Pin dependency versions for reproducibility

---

## 12. Appendix: Research Notes

### 12.1 Detector Complexity Analysis

| Detector | Lines of Code | External Deps | Test Coverage |
|----------|---------------|---------------|---------------|
| Bidi | 150 | 0 | 85% |
| Homoglyph | 200 | 0 | 80% |
| Invisible | 250 | 0 | 90% |
| Glassware | 400 | regex | 75% |
| SocketIOC2 | 300 | 0 | 70% |

**Average:** 260 LOC, 0.2 external deps, 80% test coverage

### 12.2 Plugin Size Estimates

Based on existing detectors:

| Plugin Type | Binary Size | Load Time | Memory |
|-------------|-------------|-----------|--------|
| Simple (Bidi) | ~500KB | <10ms | <1MB |
| Medium (Homoglyph) | ~800KB | <20ms | <2MB |
| Complex (Glassware) | ~1.5MB | <50ms | <5MB |

### 12.3 Version Compatibility Matrix

| glassware-core | Plugin API | Compatible Plugins |
|----------------|------------|-------------------|
| 0.5.x | 1.0 | 1.0.x |
| 0.6.x | 1.0 | 1.0.x, 1.1.x |
| 0.7.x | 2.0 | 2.0.x (breaking) |

### 12.4 Open Questions

1. **Q:** Should plugins be able to register custom Finding categories?
   - **A:** Yes, via extensible enum or string-based categories

2. **Q:** How to handle plugin failures during scanning?
   - **A:** Catch panics, log error, continue with other detectors

3. **Q:** Should there be a plugin marketplace?
   - **A:** Future consideration; start with crates.io tags

4. **Q:** How to test plugins against multiple glassware-core versions?
   - **A:** CI matrix testing, SemVer guarantees

### 12.5 References

- [Semgrep Rule Format](https://semgrep.dev/docs/writing-rules/rule-syntax/)
- [Clippy Lint Development](https://rust-lang.github.io/rust-clippy/master/development/index.html)
- [ESLint Plugin API](https://eslint.org/docs/developer-guide/working-with-plugins)
- [Wireshark Plugin Development](https://www.wireshark.org/docs/wsdg_html_chunked/ChapterDevelopment.html)
- [Rust Plugin Architecture Patterns](https://matklad.github.io/2022/05/03/plugin-architecture.html)

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-03-23 | Glassworks Team | Initial draft |

---

## Approval

- [ ] Architecture Review
- [ ] Security Review
- [ ] Implementation Plan Approved
- [ ] Resource Allocation Confirmed
