# GlassWorm Detection Capabilities

**Version:** v0.11.0+  
**Last Updated:** 2026-03-21

---

## Overview

GlassWorm is a supply chain attack detection tool focused on identifying the GlassWorm campaign and related threats. It uses a three-layer detection system with 22+ detectors across multiple categories.

---

## Detection Layers

### Layer 1: Regex Detectors (All Files)

Fast pattern matching using regular expressions. Runs on all files.

| ID | Detector | Severity | Description |
|----|----------|----------|-------------|
| GW001 | InvisibleChar | Critical | Zero-width characters, variation selectors |
| GW002 | Homoglyph | Medium-High | Mixed-script identifiers (Cyrillic/Greek) |
| GW003 | BidiOverride | Critical | Trojan Source bidirectional overrides |
| GW004 | UnicodeTag | High | Unicode tag characters (U+E0001–U+E007F) |
| GW005 | GlasswarePattern | High | Stego decoder patterns |
| GW006 | EncryptedPayload | High | High-entropy blob + dynamic execution |
| GW007 | HeaderC2 | Critical | HTTP header extraction + decrypt → exec |
| GW008 | LocaleGeofencing | Medium | Russian locale checks |
| GW009 | TimeDelay | Low | Sandbox evasion delays |
| GW010 | BlockchainC2 | High | Solana/Google Calendar C2 |
| GW011 | RddDetector | High | URL dependencies (PhantomRaven) |
| GW012 | ForceMemo | Critical | Python repo injection |
| GW013 | JpdAuthor | Critical | "JPD" author signature |

### Layer 2: Semantic Detectors (JS/TS Only)

Requires OXC parser. Analyzes JavaScript/TypeScript AST for flow-based detection.

| ID | Detector | Severity | Description |
|----|----------|----------|-------------|
| L2-GW005 | Gw005Semantic | High | Stego → exec flow |
| L2-GW006 | Gw006Semantic | High | Hardcoded key → exec |
| L2-GW007 | Gw007Semantic | High | RC4 cipher → exec |
| L2-GW008 | Gw008Semantic | Critical | Header C2 → decrypt → exec |

### Layer 3: Binary Detectors (.node Files)

Uses goblin parser for PE/ELF/Mach-O analysis.

| ID | Detector | Severity | Description |
|----|----------|----------|-------------|
| G6 | XorShift128 | High | XorShift obfuscation (heuristic) |
| G7 | IElevator CLSID | Critical | COM interface for App-Bound bypass |
| G8 | APC Injection | High | Process injection imports |
| G9 | memexec | High | Fileless PE loader |
| G11 | .node Metadata | Info/Medium | PDB paths, build attribution |

### Layer 4: Host Forensics

Filesystem and browser profile scanning.

| ID | Detector | Severity | Description |
|----|----------|----------|-------------|
| G1 | Filesystem Persistence | High | Known GlassWorm paths |
| G2 | Chrome Prefs Tampering | Critical | Sideloaded extension detection |

---

## Threat Scoring Model

### Signal Stacking Formula

```
score = (categories_present × 2.0) + (critical_hits × 3.0) + (high_hits × 1.5)
```

### Categories

| Category | Indicators |
|----------|------------|
| **Obfuscation** | Invisible chars, homoglyphs, bidi |
| **Evasion** | Locale bypass, time delay |
| **C2** | Known wallets, known IPs, blockchain polling |
| **Execution** | eval patterns, encrypted payload |
| **Persistence** | Preinstall scripts, file writes |

### Thresholds

| Score | Classification | Action |
|-------|---------------|--------|
| 0-3 | Clean | No action |
| 3-6 | Suspicious | Flag for review |
| 6-10 | Likely malicious | Quarantine |
| 10+ | Confirmed malicious | Block + report |

---

## Known Indicators

### GlassWorm C2 Wallets

- `BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC`
- `28PKnu7RzizxBzFPoLp69HLXp9bJL3JFtT2s5QzHsEA2`
- `DSRUBTziADDHSik7WQvSMjvwCHFsbsThrbbjWMoJPUiW`
- `6YGcuyFRJKZtcaYCCFba9fScNUvPkGXodXE1mJiSzqDJ`

### GlassWorm C2 IPs

- `45.32.150.251`, `45.32.151.157`, `70.34.242.255`
- `217.69.3.152`, `217.69.3.51`, `217.69.11.99`, `217.69.0.159`
- `104.238.191.54`, `108.61.208.161`, `45.150.34.158`

### Malicious Packages (Confirmed)

- `react-native-country-select@0.3.91`
- `react-native-international-phone-number@0.11.8`

---

## False Positive Prevention

### Whitelisted Packages

The following package types have reduced scoring to prevent false positives:

| Package Type | Examples | Whitelisted Signals |
|--------------|----------|---------------------|
| **Date/Locale libraries** | moment, date-fns, globalize | Invisible chars in locale files |
| **Code formatters** | prettier, eslint, babel | eval patterns (string processing) |
| **Build tools** | webpack, vite, rollup | Time delays (watch mode) |
| **Crypto libraries** | ethers, web3, viem, wagmi | Blockchain API calls |
| **Compilers** | typescript | Locale checks in tests |

### Detection Logic

- **Single category findings** → Low score (likely legitimate)
- **Multiple category findings** → High score (likely malicious)
- **Known C2 wallets/IPs** → Always flagged (Critical)

---

## Performance Benchmarks

| Metric | Value |
|--------|-------|
| Scan speed | ~50k LOC/sec |
| npm scan | ~0.5s per package |
| GitHub scan | ~5-20s per repo |
| Rust orchestrator | 0.15 pkg/sec |
| Python harness | 0.08 pkg/sec |

---

## Usage Examples

### CLI Scanning

```bash
# Scan directory
glassware /path/to/project

# Scan npm packages
glassware-orchestrator scan-npm express lodash axios

# Scan tarball files
glassware-orchestrator scan-tarball package-1.0.0.tgz

# Scan GitHub repo
glassware-orchestrator scan-github owner/repo

# With LLM analysis
glassware-orchestrator --llm scan-npm suspicious-pkg
```

### Python Harness

```bash
cd harness

# Run wave campaign
python3 -m core.orchestrator run-wave --wave 0

# With LLM analysis
python3 -m core.orchestrator run-wave --wave 0 --llm
```

---

## References

- [GlassWorm Writeup](https://codeberg.org/tip-o-deincognito/glassworm-writeup)
- [Sonatype Report](https://www.sonatype.com/resources/blog/glassworm-supply-chain-attack)
- [Aikido Security Report](https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode)
