# glassware — Production Handoff

**Last updated:** 2026-03-20 00:30 UTC  
**Version:** v0.4.0 (planned)  
**Status:** Threat intelligence system with attack correlation  

---

## 🚀 Quick Start

**Scan npm packages:**
```bash
cd harness
python3 optimized_scanner.py packages.txt -w 10 -e data/evidence/scan -o results.json
```

**Scan GitHub repos:**
```bash
cd harness
python3 github_scanner.py --queries "mcp" "vscode" --max-repos 500
```

**Scan with attack correlation:**
```bash
./glassware-scanner src/ --attack-graph --campaign-intelligence
```

---

## 📊 Current Status

### Detection Coverage

| Campaign | Detectors | Coverage |
|----------|-----------|----------|
| **GlassWorm Core** | 17 detectors (3 tiers) + attack graphs | ✅ 100% |
| **PhantomRaven** | RDD + JPD + campaign tracking | ✅ 100% |
| **ForceMemo** | Python detector + campaign tracking | ✅ 100% |
| **Chrome RAT** | Blockchain C2 + campaign tracking | ✅ 100% |
| **React Native** | Encrypted payload + attack chains | ✅ 100% |

### NEW in v0.5.0: Cross-File Analysis

**Multi-File Taint Tracking:**
- Module graph construction (ES6, CommonJS, TypeScript)
- Cross-file taint propagation
- Split payload detection (decoder in file A, exec in file B)
- Import chain tracking
- Confidence scoring for deliberate obfuscation

**Module Systems Supported:**
- ES6 modules (`import/export`)
- CommonJS (`require/module.exports`)
- TypeScript (`import type/export type`)
- Dynamic imports (`import()`)

### NEW in v0.4.0: Threat Intelligence

**Attack Graph Engine:**
- Correlates findings into attack chains
- 6 attack chain types (GlassWareStego, EncryptedExec, HeaderC2Chain, etc.)
- Threat score: 0.0-10.0

**Campaign Intelligence:**
- Tracks infrastructure reuse (domains, wallets, authors)
- Clusters packages into campaigns (GlassWorm, PhantomRaven, ForceMemo, etc.)
- Code similarity clustering (MinHash-based)

### Tiered Detection (v0.3.1)

**Tier 1 (Primary):** Always run - invisible chars, homoglyphs, bidi (<1% FP)  
**Tier 2 (Secondary):** Run if Tier 1 finds OR file not minified - glassware patterns, encrypted payload  
**Tier 3 (Behavioral):** Run only if Tier 1+2 find - locale, time delay, blockchain C2  

**Result:** 90% FP reduction on minified/bundled code

### Active Scans

| Scan | Target | Status | ETA |
|------|--------|--------|-----|
| **High-Impact** | 630 packages | ✅ Complete | Done |
| **GitHub Mixed** | 900 repos | ✅ Complete | Done |

### Recent Results

| Scan | Packages | Flagged | Malicious | FP Rate |
|------|----------|---------|-----------|---------|
| High-impact | 630 | 10 | 1 (@iflow-mcp) | ~1% (after tiering) |
| GitHub Mixed | 848 | 0 | 0 | 0% |

---

## 🎯 Detector Registry

### Tier 1: Primary Detectors (Always Run)

| ID | Detector | Purpose | FP Rate |
|----|----------|---------|---------|
| GW001 | InvisibleCharDetector | Zero-width chars, variation selectors | <0.1% |
| GW002 | HomoglyphDetector | Mixed-script identifiers | ~0.5% |
| GW003 | BidiDetector | Bidirectional text overrides | <0.1% |
| GW004 | UnicodeTagDetector | Unicode tag characters | ~0.1% |

### Tier 2: Secondary Detectors (Skip Minified Files)

| ID | Detector | Purpose | FP Rate | Skip Conditions |
|----|----------|---------|---------|-----------------|
| GW005 | GlasswareDetector | Stego decoder patterns | ~15% → ~2% | Minified, bundled |
| GW006 | EncryptedPayloadDetector | High-entropy + exec | ~10% → ~1% | /lib/, /dist/, bundled |
| GW007 | HeaderC2Detector | HTTP header C2 | ~5% → ~1% | Minified files |

### Tier 3: Behavioral Detectors (Run Only if Tier 1+2 Find)

| ID | Detector | Purpose | FP Rate (standalone) | FP Rate (tiered) |
|----|----------|---------|---------------------|------------------|
| GW008 | LocaleGeofencingDetector | Russian locale checks | ~50% | ~5% |
| GW009 | TimeDelayDetector | Sandbox evasion delays | ~80% | ~10% |
| GW010 | BlockchainC2Detector | Solana/Google Calendar C2 | ~30% | ~3% |
| GW011 | ForceMemoDetector | Python repo injection | ~20% | ~2% |
| GW012 | RddDetector | URL dependencies (PhantomRaven) | ~10% | ~1% |
| GW013 | JpdAuthorDetector | "JPD" author signature | ~5% | ~0.5% |

### L2: Semantic Detectors (JS/TS Only)

| ID | Detector | Purpose |
|----|----------|---------|
| L2-GW005 | Gw005SemanticDetector | Stego → exec flow |
| L2-GW006 | Gw006SemanticDetector | Hardcoded key → exec |
| L2-GW007 | Gw007SemanticDetector | RC4 cipher → exec |
| L2-GW008 | Gw008SemanticDetector | Header C2 → decrypt → exec |

### L3: LLM Review

| Component | Purpose |
|-----------|---------|
| OpenAiCompatibleAnalyzer | Intent-level reasoning |

---

## 📁 File Organization

```
glassworks/
├── HANDOFF.md                    ← 🟢 You are here
├── README.md                     ← Project overview
├── RELEASE.md                    ← Release notes (v0.3.1)
├── TODO.md                       ← Current priorities
├── DOCUMENTATION-CATALOG.md      ← All docs catalogued
├── INTEL.md                      ← Current intelligence
├── INTEL-REVIEW-EVASION-TECHNIQUES.md ← Evasion patterns
│
├── docs/
│   ├── WORKFLOW-GUIDE.md         ← Complete scan/analyze/improve workflow
│   └── archive/                  ← Historical documents
│
├── harness/
│   ├── github_scanner.py         ← GitHub repo scanner
│   ├── optimized_scanner.py      ← npm package scanner
│   ├── diverse_sampling.py       ← Category sampling
│   ├── batch_llm_analyzer.py     ← LLM analysis
│   ├── database.py               ← SQLite cache
│   ├── reporter.py               ← Report generation
│   ├── glassware-scanner         ← Binary (current)
│   └── reports/                  ← Scan reports
│       ├── TIERED-DETECTOR-ARCHITECTURE.md
│       ├── PHASE1-IMPLEMENTATION-REPORT.md
│       ├── PHASE2-IMPLEMENTATION-REPORT.md
│       ├── REAL-WORLD-VALIDATION-REPORT.md
│       └── *.md (scan reports)
│
├── glassware-core/
│   └── src/
│       ├── detector.rs           ← Unified Detector trait with tiers ⭐ NEW
│       ├── minified.rs           ← Minified code detection ⭐ NEW
│       ├── cache.rs              ← Incremental scanning (10x speedup) ⭐ NEW
│       ├── rdd_detector.rs       ← RDD detection
│       ├── jpd_author_detector.rs ← JPD signature
│       ├── forcememo_detector.rs ← Python injection
│       ├── locale_detector.rs    ← Locale geofencing
│       ├── time_delay_detector.rs ← Time delays
│       ├── blockchain_c2_detector.rs ← Blockchain C2
│       └── ...
│
└── llm-analyzer/
    └── analyzer.py               ← LLM analysis
```

---

## 🔧 Configuration

### Environment Variables

```bash
# LLM API (required for LLM analysis)
export NVIDIA_API_KEY="nvapi-..."

# GitHub API (optional, for higher rate limits)
export GITHUB_TOKEN="ghp_..."

# Evidence directories
export GLASSWARE_EVIDENCE_DIR="data/evidence/custom"
```

### .env File

```bash
cp .env.example .env
# Edit with your credentials
```

### CLI Flags (NEW in v0.3.1)

```bash
# Default: tiered scanning enabled
glassware src/

# Disable tiered scanning (run all detectors)
glassware --no-tiered src/

# Analyze bundled code (include Tier 2+ on minified files)
glassware --analyze-bundled src/

# With caching (10x re-scan speedup)
glassware --cache-file .glassware-cache.json src/
```

---

## 📋 Workflows

### Workflow 1: npm Package Scan

```bash
cd harness

# 1. Sample packages
python3 diverse_sampling.py \
  --categories ai-ml native-build install-scripts \
  --samples-per-keyword 20 \
  --output packages.txt

# 2. Scan packages
python3 optimized_scanner.py \
  packages.txt \
  -w 10 \
  -e data/evidence/scan \
  -o results.json

# 3. Analyze flagged
cat results.json | jq -r '.flagged_packages[].package' > flagged.txt
python3 batch_llm_analyzer.py flagged.txt -w 2
```

### Workflow 2: GitHub Repo Scan

```bash
cd harness

# 1. Scan repositories
python3 github_scanner.py \
  --queries "mcp" "vscode" "cursor" \
  --repos-per-query 100 \
  --max-repos 500 \
  --output github-results.json

# 2. Review results
cat github-results.json | jq '{scanned, flagged, errors}'
```

### Workflow 3: High-Security Scan (Analyze Bundled Code)

```bash
cd harness

# Scan including bundled/minified files
python3 optimized_scanner.py \
  packages.txt \
  -w 10 \
  -e data/evidence/scan-high-security \
  -o results.json \
  -- --analyze-bundled  # Pass to binary
```

---

## 📈 Performance Metrics

| Metric | v0.1.0 | v0.3.0 | v0.3.1 |
|--------|--------|--------|--------|
| **Initial scan (524 files)** | ~5s | ~2.4s | ~1.8s |
| **Re-scan (cached)** | ~5s | ~0.5s | ~0.5s |
| **Minified files** | ~5s | ~2.4s | ~0.5s |
| **FP rate (prettier)** | 28 findings | 28 findings | 0 findings |
| **FP rate (webpack)** | 3 findings | 3 findings | 0 findings |

**Improvements:**
- v0.3.0: 2x faster (parallel), 10x re-scan (caching)
- v0.3.1: 90% FP reduction (tiered detection)

---

## 🐛 Troubleshooting

### npm 429 Rate Limit

```bash
# Increase delay
python3 diverse_sampling.py --delay-between-keywords 2.0

# Reduce retries
python3 diverse_sampling.py --npm-retries 2
```

### GitHub 403 Rate Limit

```bash
# Add token
export GITHUB_TOKEN="ghp_..."

# Or wait 60s (automatic backoff)
```

### High False Positive Rate

```bash
# Check if scanning minified files
glassware src/  # Should skip /dist/, /lib/, etc.

# If you need to scan bundled code
glassware --analyze-bundled src/

# Or disable tiered detection entirely
glassware --no-tiered src/
```

### Cache Not Working

```bash
# Verify cache file exists
ls -la .glassware-cache.json

# Check cache stats
glassware --cache-file .glassware-cache.json src/
# Look for "Cache: X hits, Y misses" in output
```

---

## 📞 Support

### Documentation

| Document | Purpose |
|----------|---------|
| `HANDOFF.md` | Current status & quick start |
| `README.md` | Project overview |
| `RELEASE.md` | Release notes |
| `docs/WORKFLOW-GUIDE.md` | Complete scan/analyze/improve workflow |
| `harness/reports/TIERED-DETECTOR-ARCHITECTURE.md` | Tiered detection details |
| `DOCUMENTATION-CATALOG.md` | All documents catalogued |

### Reports

All scan reports in `harness/reports/`

---

## ✅ Current Priorities (TODO.md)

1. **Monitor high-impact scan results** - Review flagged packages
2. **Tune tier thresholds** - Based on real-world data
3. **Expand scanning** - 5k+ repos with tiered detection
4. **Prepare disclosure** - If malicious found (@iflow-mcp confirmed)

---

## 🏷️ Version History

| Version | Date | Key Changes |
|---------|------|-------------|
| **v0.3.1** | 2026-03-19 | Tiered detection, minified code skip, 90% FP reduction |
| **v0.3.0** | 2026-03-19 | Parallel scanning, caching, SARIF compliance, deduplication |
| **v0.2.0** | 2026-03-19 | File size limits, error tracking, HashSet optimization |
| **v0.1.0** | 2026-03-18 | Initial release |

---

**Status:** ✅ All systems operational  
**Last scan:** High-impact (630 packages, 1 malicious confirmed)  
**Next action:** Review tier thresholds, prepare disclosure  

---

**End of Handoff**

### NEW in v0.7.0: Contextual Risk Scoring

**Risk multipliers based on context:**
- Ecosystem: npm (1.0x), PyPI (1.2x), GitHub (1.5x)
- Package type: Library (1.0x), CLI (1.3x), Extension (1.5x)
- Novelty: <7 days (1.5x), <30 days (1.2x), established (1.0x)
- Reputation: Known publisher (0.8x), unknown (1.0x)
- File type: Minified (0.5x), source (1.0x)

**Example:**
```rust
let context = RiskContext::new()
    .with_ecosystem(Ecosystem::GitHub)
    .with_package_type(PackageType::Extension)
    .with_package_age(2);  // 2 days old = 1.5x novelty multiplier

let score = calculate_package_risk_with_context(&findings, &context);
// GitHub extension, 2 days old: 1.5×1.5×1.5 = 3.375x multiplier
```

