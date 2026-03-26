# glassware — GlassWorm Steganography Detection System

**Production-ready Rust-based scanner for detecting GlassWorm steganographic attacks in npm packages and GitHub repositories.**

[![Release](https://img.shields.io/github/v/release/samgrowls/glassworks)](https://github.com/samgrowls/glassworks/releases)
[![License](https://img.shields.io/github/license/samgrowls/glassworks)](LICENSE)

---

## 🎯 What Is GlassWorm?

**GlassWorm** is a supply chain attack technique that hides malicious payloads using invisible Unicode characters (steganography) combined with decoder functions. The attack was first identified in 2024 and has evolved through multiple waves targeting npm packages and GitHub repositories.

**Key Characteristics:**
- Invisible Unicode characters (ZWSP, ZWNJ, Variation Selectors) encode hidden payloads
- Decoder functions extract and execute the hidden code at runtime
- Often combined with blockchain-based C2 (Solana) for command retrieval
- Uses sandbox evasion to avoid detection in CI/CD environments

---

## 🛡️ What glassware Does

**glassware** detects GlassWorm attacks through 13+ specialized detectors organized in tiers:

| Tier | Detectors | Purpose |
|------|-----------|---------|
| **Tier 1** | InvisibleCharacter, Homoglyph, BidirectionalOverride, GlasswarePattern | Primary GlassWorm indicators |
| **Tier 2** | EncryptedPayload, HeaderC2, ExfilSchema | Secondary confirmation |
| **Tier 3** | BlockchainC2, TimeDelaySandboxEvasion, LocaleGeofencing | Behavioral analysis |

### Detection Approach

**Context-Aware Detection:** glassware distinguishes between:
- Build tool patterns vs. runtime evasion
- Telemetry headers vs. C2 communication
- Legitimate SDK usage vs. malicious C2

**No Package Whitelisting:** Detection is based on code patterns, not package popularity.

---

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+
- Optional: LLM API keys (Cerebras, NVIDIA) for AI-powered analysis

### Build

```bash
# Clone repository
git clone https://github.com/samgrowls/glassworks.git
cd glassworks

# Build release binary
cargo build --release

# Verify installation
./target/release/glassware --help
```

### Scan a Single Package

```bash
# Scan npm package
./target/release/glassware scan-npm <package>@<version>

# Example
./target/release/glassware scan-npm express@4.19.2

# Scan with LLM analysis
./target/release/glassware scan-npm <package>@<version> --llm
```

### Scan a Tarball

```bash
# Scan local tarball
./target/release/glassware scan-tarball evidence/package.tgz
```

### Run a Campaign

```bash
# Run campaign from TOML config
./target/release/glassware campaign run campaigns/wave15-validation.toml

# Monitor progress (in another terminal)
./target/release/glassware campaign monitor <case-id>

# Generate report after completion
./target/release/glassware campaign report <case-id>
```

---

## 📖 Documentation

| Document | Purpose |
|----------|---------|
| **[docs/USER-GUIDE.md](docs/USER-GUIDE.md)** | Complete user guide |
| **[docs/DEVELOPER-GUIDE.md](docs/DEVELOPER-GUIDE.md)** | Developer reference |
| **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)** | System architecture |
| **[docs/DETECTORS.md](docs/DETECTORS.md)** | Detector reference |
| **[docs/CAMPAIGN-OPERATOR-GUIDE.md](docs/CAMPAIGN-OPERATOR-GUIDE.md)** | Campaign operations |

---

## 🔬 Detection Capabilities

### Tier 1: Primary Indicators

| Detector | Detects | Severity |
|----------|---------|----------|
| **InvisibleCharacter** | ZWSP, ZWNJ, Variation Selectors | High |
| **Homoglyph** | Confusable Unicode characters | Medium |
| **BidirectionalOverride** | RTL text override attacks | Medium |
| **GlasswarePattern** | Steganography + decoder combination | Critical |

### Tier 2: Secondary Confirmation

| Detector | Detects | Severity |
|----------|---------|----------|
| **EncryptedPayload** | High-entropy blobs + dynamic execution | High |
| **HeaderC2** | HTTP header extraction + decryption + exec | Critical |
| **ExfilSchema** | Data exfiltration patterns | High |

### Tier 3: Behavioral Analysis

| Detector | Detects | Severity |
|----------|---------|----------|
| **BlockchainC2** | Known C2 wallets/IPs, GlassWorm signature | Critical |
| **TimeDelaySandboxEvasion** | CI bypass + time delays | Critical |
| **LocaleGeofencing** | Geographic targeting (Russia skip) | Critical |

---

## 📊 Scoring System

glassware uses a tiered scoring system with category diversity caps:

| Categories Detected | Max Score | Interpretation |
|---------------------|-----------|----------------|
| 1 category | 5.0 | Suspicious |
| 2 categories | 7.0 | Borderline malicious |
| 3 categories | 8.5 | Likely malicious |
| 4+ categories | 10.0+ | Very likely malicious |

**Malicious Threshold:** 7.0 (configurable per campaign)

### GlassWorm Signature Detection

When specific GlassWorm patterns are detected, scores can reach 25.0+:

```toml
[[settings.scoring.conditional_rules]]
name = "glassworm_signature"
condition = "invisible_char.count >= 10 AND blockchain_c2.count >= 1"
action = "final_score = 25.0"
```

---

## ⚙️ Configuration

### Environment Variables

```bash
# Tier 1 LLM (Cerebras - fast triage)
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="csk-..."
export GLASSWARE_LLM_MODEL="llama-3.3-70b"

# Tier 2 LLM (NVIDIA - deep analysis)
export NVIDIA_API_KEY="nvapi-..."

# GitHub (private repos)
export GITHUB_TOKEN="ghp_..."
```

### Campaign Configuration

Campaigns are configured via TOML files:

```toml
[campaign]
name = "My Campaign"
description = "Description"

[settings]
concurrency = 20
rate_limit_npm = 10.0

[settings.scoring]
malicious_threshold = 7.0

[settings.scoring.tier_config]
mode = "tiered"

[[settings.scoring.tiers]]
tier = 1
detectors = ["invisible_char", "glassware_pattern"]
threshold = 0.0

[[waves]]
id = "wave_1"
name = "Evidence Validation"

[[waves.sources]]
type = "packages"
list = ["package@1.0.0"]
```

See [`campaigns/wave15-validation.toml`](campaigns/wave15-validation.toml) for a complete example.

---

## 📈 Performance Metrics

| Metric | Value |
|--------|-------|
| Binary size | ~25MB |
| Scan speed | ~50k LOC/sec |
| npm scan | ~0.5s per package |
| GitHub scan | ~5-20s per repo |
| Memory usage | ~50MB during scan |

---

## 🧪 Evidence Validation

glassware maintains a curated evidence set of confirmed GlassWorm attacks:

| Package | Type | Detection Status |
|---------|------|------------------|
| iflow-mcp-watercrawl-mcp-1.3.4 | Real Attack | ✅ Detected (8.50) |
| glassworm-combo-002 | Synthetic | ✅ Detected (7.00) |
| glassworm-combo-003 | Synthetic | ✅ Detected (7.00) |
| glassworm-combo-004 | Synthetic | ✅ Detected (7.00) |

**Evidence Detection Rate:** 100% (4/4)

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Campaign Executor                         │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐ │
│  │ Wave 1       │────►│ Wave 2       │────►│ Wave 3       │ │
│  │ (evidence)   │     │ (clean)      │     │ (hunt)       │ │
│  └──────────────┘     └──────────────┘     └──────────────┘ │
│                           │                                  │
│                  ┌────────▼────────┐                         │
│                  │ Event Bus       │                         │
│                  │ (pub/sub)       │                         │
│                  └────────┬────────┘                         │
│                           │                                  │
│         ┌─────────────────┼─────────────────┐               │
│         │                 │                 │               │
│   ┌─────▼─────┐   ┌──────▼──────┐   ┌──────▼──────┐        │
│   │ State     │   │ Command     │   │    TUI      │        │
│   │ Manager   │   │ Channel     │   │             │        │
│   └───────────┘   └─────────────┘   └──────────────┘        │
└─────────────────────────────────────────────────────────────┘
```

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for detailed architecture documentation.

---

## 📝 License

MIT License - see [LICENSE](LICENSE) for details.

---

## 🔗 References

- [GlassWorm Writeup](https://codeberg.org/tip-o-deincognito/glassworm-writeup)
- [Aikido Security Blog](https://www.aikido.dev/blog/glassworm-returns-unicode-attack-github-npm-vscode)
- [Sonatype Research](https://www.sonatype.com/blog/glassworm-supply-chain-attack)

---

**Version:** v0.67.0-production-ready
**Last Updated:** 2026-03-26
