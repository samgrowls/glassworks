

# 🔍  Code Review — glassworks (v0.5.0)



### 1. Detector Execution IS Linear (Not DAG-Based)

From `engine.rs`:
```rust
// Run regex-based detectors on all files
for detector in &self.detectors {
    findings.extend(detector.detect(&ctx));
}
```

**Issue:** Detectors run sequentially, not prioritized by cost/signal.

**Impact:** On large packages, cheap detectors don't short-circuit expensive ones.

**Fix:** Add `prerequisites()` and `cost()` to `Detector` trait, build execution DAG.

---

### 2. No Unified IR Layer

Each detector parses independently:
- `UnicodeDetector` → uses `UnicodeScanner`
- `RddDetector` → parses JSON directly
- `LocaleGeofencingDetector` → regex on lines
- Semantic detectors → OXC AST

**Issue:** Redundant parsing, inconsistent semantics.

**Fix:** Introduce `FileIR` struct that all detectors consume:
```rust
struct FileIR {
    content: String,
    lines: Vec<String>,
    ast: Option<Ast>,  // For JS/TS
    json: Option<Value>, // For package.json
    unicode_map: UnicodeAnalysis,
}
```

---

### 3. Risk Scoring Missing Context Multipliers

Current scoring:
```rust
pub fn calculate_package_risk(findings: &[Finding]) -> u32 {
    findings.iter().map(finding_risk_score).sum()
}
```

**Missing:**
- Package type multiplier (CLI vs library)
- Ecosystem multiplier (npm vs PyPI)
- Publisher reputation factor
- File type adjustment (minified vs source)

**Fix:** Add contextual scoring:
```rust
score = base_score * ecosystem_multiplier * novelty_multiplier
```

---

### 4. Python Harness + Rust Core Split IS a Problem

From `optimized_scanner.py`:
```python
GLASSWARE = "/home/.../glassware-scanner"
scan_result = subprocess.run([GLASSWARE, "--format", "json", str(pkg_dir)])
```

**Issue:** 
- Serialization overhead (JSON in/out)
- Duplicate logic (caching in both Python and Rust)
- Hard to scale (process-per-package)

**Fix:** Move orchestration to Rust with Tokio, keep Python for experimentation only.

---

### 5. No Adversarial Testing Framework

You detect attacks but don't test detector robustness.

**Missing:**
- Mutation testing (can attackers evade?)
- Detector fuzzing
- Polymorphic payload simulation

**Fix:** Add `harness/adversarial/` with evasion test cases.

---

## 📊 Detector Coverage (Actual)

| Tier | Detector | File Type | Status |
|------|----------|-----------|--------|
| T1 | InvisibleChar | All | ✅ |
| T1 | Homoglyph | All | ✅ |
| T1 | Bidi | All | ✅ |
| T2 | Glassware | Source | ✅ |
| T2 | EncryptedPayload | Source | ✅ |
| T2 | HeaderC2 | Source | ✅ |
| T3 | LocaleGeofencing | JS/TS | ✅ |
| T3 | TimeDelay | JS/TS | ✅ |
| T3 | BlockchainC2 | JS/TS | ✅ |
| T3 | Rdd | package.json | ✅ |
| T3 | JpdAuthor | package.json | ✅ |
| T3 | ForceMemo | Python | ✅ |
| L2 | GW005-GW008 Semantic | JS/TS | ✅ |
| L3 | LLM Analyzer | Flagged | ✅ |

**17 detectors total** — more than the previous review acknowledged.

---

## 🔧 Specific Code Issues

### 1. `engine.rs` — Clone Overhead

```rust
// Line ~400
let findings_clone = sorted_findings.clone();
cache.set(path_str, content, findings_clone, file_size);
```

**Issue:** Unnecessary clone before cache set.

**Fix:** Use `Arc< Finding >` or move semantics.

---

### 2. `rdd_detector.rs` — Hardcoded Line Numbers

```rust
findings.push(Finding::new(&path.to_string_lossy(), 1, 1, 0, '0', ...));
```

**Issue:** All findings report line 1, column 1 — loses precision.

**Fix:** Parse JSON with line tracking (use `serde_spanned` or similar).

---

### 3. `locale_detector.rs` — Two-Pass Inefficiency

```rust
// First pass: find locale checks
for (line_num, line) in ctx.content.lines().enumerate() { ... }

// Second pass: check for exit patterns
for (line_num, line) in ctx.content.lines().enumerate() { ... }
```

**Issue:** Iterates content twice.

**Fix:** Single-pass with sliding window.

---

### 4. `scanner.rs` — File Size Check Race Condition

```rust
let file_size = std::fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);
let is_large_file = file_size > 500_000;
```

**Issue:** File could change between metadata check and read.

**Fix:** Check size after reading content.

---

### 5. `finding.rs` — Missing `Eq` for Dedup Key

```rust
pub struct Finding {
    pub file: String,
    pub line: usize,
    // ...
}
```

Dedup uses `(file, line, column, category)` but `Finding` doesn't derive `Eq`/`Hash` properly for all fields.

---

## 🎯 Priority Fixes (Before v0.5.0 Release)

| Priority | Issue | Effort | Impact |
|----------|-------|--------|--------|
| P0 | RDD detector line numbers | 1h | High (forensics) |
| P0 | Locale detector single-pass | 2h | Medium (perf) |
| P1 | Detector DAG execution | 8h | High (scale) |
| P1 | Unified IR layer | 16h | High (consistency) |
| P2 | Contextual risk scoring | 4h | Medium (accuracy) |
| P2 | Rust-native orchestrator | 24h | High (scale) |
| P3 | Adversarial test framework | 16h | Medium (robustness) |

---

