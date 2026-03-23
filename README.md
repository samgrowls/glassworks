# glassware — Autonomous GlassWare Detection System

**Production-ready Rust-based campaign orchestration for detecting GlassWare steganographic attacks in npm packages and GitHub repositories.**

[![Release](https://img.shields.io/github/v/release/samgrowls/glassworks)](https://github.com/samgrowls/glassworks/releases)
[![License](https://img.shields.io/github/license/samgrowls/glassworks)](LICENSE)

---

## 🎯 What It Does

**glassware** detects steganographic payloads, invisible Unicode characters, bidirectional text attacks, and behavioral evasion patterns in source code and repositories.

### Key Features

- ✅ **Campaign Orchestration** - Run large-scale scanning campaigns (100k+ packages)
- ✅ **Checkpoint/Resume** - Reliable interruption recovery
- ✅ **Interactive TUI** - Live monitoring with command palette
- ✅ **LLM-Powered Analysis** - Natural language queries about findings
- ✅ **Markdown Reports** - Professional stakeholder reports
- ✅ **13+ Detectors** - Unicode, behavioral, and semantic analysis

---

## 🚀 Quick Start

### Install

```bash
# Build from source
cargo build --release

# Binary location
./target/release/glassware-orchestrator --help
```

### Run Your First Campaign

```bash
# Run Wave 6 calibration campaign
./target/release/glassware-orchestrator campaign run campaigns/wave6.toml

# Monitor in TUI (in another terminal)
./target/release/glassware-orchestrator campaign monitor <case-id>

# Generate report after completion
./target/release/glassware-orchestrator campaign report <case-id>
```

### TUI Demo

```bash
# Launch TUI demo with sample data
./target/release/glassware-orchestrator campaign demo
```

**Keyboard shortcuts:**
- `q` - Quit
- `Tab` - Switch tabs
- `p` - Pause/Resume
- `x` - Cancel
- `c` - Adjust concurrency
- `Enter` - Drill down into package
- `l` - Run LLM analysis
- `?` - Ask question about package

---

## 📖 Documentation

| Document | Purpose |
|----------|---------|
| **[HANDOFF/README.md](HANDOFF/README.md)** | **Developer handoff & getting started** |
| [docs/CAMPAIGN-USER-GUIDE.md](docs/CAMPAIGN-USER-GUIDE.md) | Complete user guide |
| [HANDOFF/FINAL-SESSION-SUMMARY.md](HANDOFF/FINAL-SESSION-SUMMARY.md) | Session summary |
| [HANDOFF/FUTURE/ROADMAP-2026.md](HANDOFF/FUTURE/ROADMAP-2026.md) | Strategic roadmap |

---

## 📁 Project Structure

```
glassworks/
├── glassware-core/              # Detection engine (library)
├── glassware-orchestrator/      # Campaign orchestrator (binary)
├── glassware-cli/               # Simple scanner (binary)
├── campaigns/                   # Campaign configurations
│   └── wave6.toml              # Calibration campaign
├── docs/                        # User documentation
├── design/                      # Architecture docs
├── HANDOFF/                     # Developer documentation
└── harness/                     # Python tools (legacy)
```

---

## 🎮 Commands Reference

### Campaign Management

```bash
# Run campaign
glassware-orchestrator campaign run campaigns/wave6.toml

# Resume interrupted campaign
glassware-orchestrator campaign resume <case-id>

# List campaigns
glassware-orchestrator campaign list

# Show status
glassware-orchestrator campaign status <case-id>

# TUI monitoring
glassware-orchestrator campaign demo              # Demo mode
glassware-orchestrator campaign monitor <case-id> # Live monitoring
```

### Analysis & Reporting

```bash
# Generate markdown report
glassware-orchestrator campaign report <case-id>

# Ask LLM questions
glassware-orchestrator campaign query <case-id> "Why was express flagged?"

# Send commands
glassware-orchestrator campaign command <case-id> pause
```

---

## 🔬 Detection Capabilities

### L1 Detectors (Primary)
- Invisible character detection
- Homoglyph/confusable character detection
- Bidirectional text override detection
- Unicode tag character detection

### L2 Detectors (Secondary)
- GlassWare pattern detection
- Encrypted payload detection
- RDD (URL dependency) detection
- JPD author signature detection

### L3 Detectors (Behavioral)
- Locale geofencing detection
- Time delay sandbox evasion
- Blockchain C2 detection

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Campaign Executor                         │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐ │
│  │ Wave 6A      │────►│ Wave 6B      │────►│ Wave 6C      │ │
│  │ (validate)   │     │ (validate)   │     │ (hunt)       │ │
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

---

## 🤝 Contributing

### For Users

1. **Report bugs** via GitHub Issues
2. **Request features** via GitHub Issues
3. **Share findings** and detection patterns

### For Developers

1. **Read HANDOFF/README.md** for developer onboarding
2. **Review ROADMAP-2026.md** for strategic direction
3. **Start with good first issues**

### Building

```bash
# Debug build
cargo build -p glassware-orchestrator

# Release build (optimized)
cargo build -p glassware-orchestrator --release

# Run tests
cargo test --features "full,llm"
```

---

## 📊 Performance

| Metric | Value |
|--------|-------|
| Binary size | ~25MB |
| Scan speed | ~50k LOC/sec |
| npm scan | ~0.5s per package |
| GitHub scan | ~5-20s per repo |
| Memory usage | ~50MB during scan |

---

## 🔐 Security

**This tool is for defensive security research only.**

- Use responsibly
- Respect rate limits
- Don't scan without permission
- Report findings responsibly

---

## 📄 License

MIT License - see [LICENSE](LICENSE)

---

## 🙏 Acknowledgments

- Koi Security - GlassWorm research
- Aikido Security - Campaign analysis
- Endor Labs - Threat intelligence
- Socket.dev - Real-time detection

---

## 📬 Contact

- **GitHub:** https://github.com/samgrowls/glassworks
- **Issues:** https://github.com/samgrowls/glassworks/issues
- **Documentation:** See `HANDOFF/` and `docs/` directories

---

**Last updated:** March 23, 2026
**Version:** 0.15.0
**Status:** Production ready
