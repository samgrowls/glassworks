# Plugin Architecture Research Summary

**Date:** March 23, 2026  
**Research Duration:** Comprehensive analysis  
**Design Document:** `HANDOFF/FUTURE/PLUGIN-ARCHITECTURE.md`

---

## Executive Summary

After analyzing the current glassware-core detector architecture and evaluating four plugin architecture options, we recommend a **hybrid approach** combining:

1. **Rust Crate Plugins (Primary)** - For complex detectors requiring full Rust capabilities
2. **Configuration-Driven Rules (Complementary)** - For simple pattern-based detectors

**Implementation Estimate:** 6-8 weeks  
**Migration Complexity:** Low - existing detectors require minimal changes

---

## Current Architecture Analysis

### Detector Trait Location
`/home/property.sightlines/samgrowls/glassworks/glassware-core/src/detector.rs`

### Key Findings

**Existing Detectors (10 total):**
| Detector | Complexity | Tier | Cost | Signal |
|----------|------------|------|------|--------|
| BidiDetector | Low | Tier1 | 1 | 9 |
| HomoglyphDetector | Medium | Tier1 | 2 | 8 |
| InvisibleCharDetector | Low | Tier1 | 1 | 9 |
| GlasswareDetector | High | Tier2 | 5 | 7 |
| SocketIOC2Detector | Medium | Tier3 | 4 | 8 |
| BrowserKillDetector | Low | Tier2 | 2 | 7 |
| ExfilSchemaDetector | Medium | Tier2 | 3 | 6 |
| TypoAttributionDetector | Low | Tier2 | 2 | 7 |
| UnicodeTagDetector | Low | Tier1 | 1 | 8 |
| TagsDetector | Low | Tier1 | 1 | 8 |

**What Makes Detectors Easy/Hard to Write:**

**Easy (Low Complexity):**
- Single-pass character scanning
- Simple regex pattern matching
- No external dependencies
- Clear attack signatures

**Hard (High Complexity):**
- Multi-pass analysis
- AST/semantic analysis required
- Cross-file correlation
- Behavioral/heuristic detection
- Configuration tuning

**Current Registration Process:**
1. Create `.rs` file in `src/detectors/`
2. Add `pub mod` in `mod.rs`
3. Register in `engine.rs::default_detectors()`
4. Rebuild entire library

---

## Plugin Architecture Options Analyzed

### Option A: Rust Crate Plugins
**Separate crates implementing the `Detector` trait**

**Pros:**
- Full type safety at compile time
- Access to full Rust ecosystem
- Zero performance overhead
- Easy distribution via crates.io
- Excellent IDE support

**Cons:**
- Requires Rust toolchain
- No hot-reloading
- Version compatibility management

**Best For:** Complex detectors, production use, community distribution

---

### Option B: WASM Plugins
**Runtime-loaded WebAssembly modules**

**Pros:**
- True runtime loading
- Language agnostic
- Sandboxed execution
- Hot-reloading support

**Cons:**
- High implementation complexity
- FFI overhead (5-15% performance)
- Limited WASM plugin ecosystem
- Difficult debugging

**Best For:** Future consideration when hot-reloading is critical

---

### Option C: Dynamic Library Plugins
**.so/.dylib/.dll loading via libloading**

**Pros:**
- Runtime loading
- Native performance
- No WASM complexity

**Cons:**
- Platform-specific binaries
- ABI compatibility issues
- Security risk (arbitrary code execution)
- Complex versioning

**Best For:** Not recommended for glassware

---

### Option D: Configuration-Driven Rules
**YAML/JSON detector definitions**

**Pros:**
- Extremely easy to write (no Rust needed)
- No compilation required
- Hot-reloading trivial
- Very safe (no code execution)

**Cons:**
- Limited to pattern matching
- 10-20% performance overhead
- No type safety
- Cannot implement behavioral detectors

**Best For:** Simple pattern-based detectors, community contributions

---

## Comparative Analysis

### Evaluation Matrix

| Criteria | Crates | WASM | dylib | Config |
|----------|--------|------|-------|--------|
| Ease of Writing | Medium | Hard | Medium | **Very Easy** |
| Ease of Updating | **Easy** | Easy | Hard | **Very Easy** |
| Performance | **Native** | 5-15% overhead | **Native** | 10-20% overhead |
| Security | High | **Very High** | Low | **Very High** |
| Distribution | **crates.io** | Custom | Manual | Git/config |
| Version Compatibility | **SemVer** | WASI | ABI issues | Schema |
| Hot Reloading | No | **Yes** | **Yes** | **Yes** |
| Implementation Complexity | **Low** | Very High | Medium | **Low** |

### Scoring (out of 25)

| Option | Score |
|--------|-------|
| **A: Rust Crates** | **21** |
| D: Configuration | 19 |
| B: WASM | 18 |
| C: dylib | 17 |

---

## Examples from Other Projects

### Semgrep (YAML-based)
- Pattern matching rules in YAML
- Accessible to security researchers
- Complex logic requires custom code

### Clippy (Rust Crate-based)
- Each lint is Rust code
- Type safety prevents bugs
- Compile-time error catching

### ESLint (JavaScript Modules)
- NPM packages exporting rules
- Easy to publish and share
- Runtime errors possible

### Wireshark (C + Lua)
- Hybrid approach
- Lua for rapid prototyping
- C for performance-critical

---

## Recommended Approach

### Hybrid Architecture

```
glassware-core
├── Plugin Registry
│   ├── Rust Plugins (complex logic)
│   │   ├── bidi
│   │   ├── homoglyph
│   │   └── socketio_c2
│   └── Config Rules (simple patterns)
│       ├── unicode_ranges
│       ├── regex_patterns
│       └── keyword_match
```

### Rationale

**Technical:**
1. Type safety prevents plugin errors
2. No FFI or interpretation overhead
3. Leverages crates.io ecosystem
4. Safe configuration for simple cases
5. Flexibility for all use cases

**Business:**
1. Low barrier for config contributions
2. Type-safe plugins for enterprises
3. Clear separation of concerns
4. Can add WASM later if needed

---

## API Design Highlights

### Plugin Trait

```rust
pub trait Plugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;
    fn initialize(&mut self, config: PluginConfig) -> Result<(), PluginError>;
    fn detectors(&self) -> Vec<Box<dyn Detector>>;
    fn shutdown(&mut self) -> Result<(), PluginError> { Ok(()) }
}
```

### Plugin Metadata

```rust
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub license: Option<String>,
    pub glassware_version_req: String,
    pub detectors: Vec<String>,
    pub description: Option<String>,
}
```

### Configuration Schema

```yaml
plugins:
  - name: glassware-detector-bidi
    enabled: true
    version: "1.0"
    
  - name: glassware-detector-supply-chain
    enabled: true
    source: crates.io
    version: "0.3"

detectors:
  bidi:
    severity_overrides:
      RLO: critical
    enabled: true
```

---

## Example Plugin Structure

```
glassware-detector-example/
├── Cargo.toml
├── README.md
├── src/
│   └── lib.rs          # Detector + Plugin implementation
├── tests/
│   └── integration_test.rs
└── config/
    └── detector.yaml
```

### Minimal Plugin Code

```rust
use glassware_core::{Detector, FileIR, Finding, Plugin, PluginMetadata};

pub struct ExampleDetector;

impl Detector for ExampleDetector {
    fn name(&self) -> &str { "example" }
    fn detect(&self, ir: &FileIR) -> Vec<Finding> { /* ... */ }
}

pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn metadata(&self) -> PluginMetadata { /* ... */ }
    fn initialize(&mut self, _config: PluginConfig) -> Result<(), PluginError> { Ok(()) }
    fn detectors(&self) -> Vec<Box<dyn Detector>> { vec![Box::new(ExampleDetector)] }
}

glassware_plugin::register_plugin!(ExamplePlugin);
```

---

## Migration Plan

### Phase 1: Foundation (Week 1-2)
- Create `glassware-plugin` helper crate
- Define Plugin trait and metadata
- Implement PluginRegistry
- Write documentation

### Phase 2: Detector Extraction (Week 3-4)
- Create plugin crates for 10 existing detectors
- Update engine to load from plugins
- Maintain backward compatibility

### Phase 3: Configuration Support (Week 5)
- Implement ConfiguredDetector for YAML rules
- Add configuration parser
- Support hot-reload

### Phase 4: Ecosystem (Week 6-8)
- Create plugin template repository
- Set up CI/CD for plugin testing
- Write tutorials and guides

---

## Implementation Estimate

### Timeline

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| Foundation | 2 weeks | Plugin API, Registry, Docs |
| Detector Extraction | 2 weeks | 10+ plugin crates |
| Configuration | 1 week | YAML detector support |
| Ecosystem | 2 weeks | Template, CI/CD, Guide |
| **Total** | **6-8 weeks** | **Full plugin system** |

### Resource Requirements

| Role | Effort | Responsibilities |
|------|--------|------------------|
| Core Developer | 4 weeks | Plugin API, Registry, Engine |
| Detector Developer | 2 weeks | Migrate detectors |
| Documentation | 1 week | API docs, tutorials |
| DevOps | 1 week | CI/CD, publishing |

---

## Security Considerations

### Threats and Mitigations

| Threat | Mitigation |
|--------|------------|
| Malicious plugin | Code review, signing (future) |
| Vulnerable dependencies | `cargo audit` in CI |
| Data exfiltration | Plugin guidelines, audit trail |
| Version incompatibility | SemVer enforcement |

### Plugin Security Guidelines

1. Minimize dependencies
2. Audit regularly with `cargo audit`
3. No network access
4. No filesystem access beyond scanned files
5. Pin dependency versions

---

## Key Files Referenced

### Core Architecture
- `/home/property.sightlines/samgrowls/glassworks/glassware-core/src/detector.rs` - Detector trait
- `/home/property.sightlines/samgrowls/glassworks/glassware-core/src/engine.rs` - Scan engine
- `/home/property.sightlines/samgrowls/glassworks/glassware-core/src/ir.rs` - Unified IR
- `/home/property.sightlines/samgrowls/glassworks/glassware-core/src/finding.rs` - Finding types

### Existing Detectors
- `/home/property.sightlines/samgrowls/glassworks/glassware-core/src/detectors/bidi.rs`
- `/home/property.sightlines/samgrowls/glassworks/glassware-core/src/detectors/homoglyph.rs`
- `/home/property.sightlines/samgrowls/glassworks/glassware-core/src/detectors/invisible.rs`
- `/home/property.sightlines/samgrowls/glassworks/glassware-core/src/detectors/glassware.rs`
- `/home/property.sightlines/samgrowls/glassworks/glassware-core/src/detectors/socketio_c2.rs`

### Design Document
- `/home/property.sightlines/samgrowls/glassworks/HANDOFF/FUTURE/PLUGIN-ARCHITECTURE.md` - Full design

---

## Next Steps

1. **Review Design Document** - Share with team for feedback
2. **Approve Approach** - Get sign-off on hybrid architecture
3. **Create Tracking Issues** - Break down into GitHub issues
4. **Start Phase 1** - Begin foundation implementation

---

## Questions?

The full design document at `HANDOFF/FUTURE/PLUGIN-ARCHITECTURE.md` includes:
- Complete API specifications
- Full example plugin implementation
- Detailed migration checklist
- Security guidelines
- Version compatibility matrix
- Community contribution guidelines
