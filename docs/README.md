# Glassworks Documentation

**Version:** 0.25.0
**Last Updated:** March 23, 2026

---

## Quick Start

| Document | Purpose |
|----------|---------|
| [User Guide](user-guide/USER-GUIDE.md) | Getting started with glassware |
| [Campaign Quickstart](user-guide/CAMPAIGN-QUICKSTART.md) | Run your first campaign |
| [Workflow Guide](user-guide/WORKFLOW-GUIDE.md) | Typical scanning workflows |

---

## Documentation Structure

### User Guide (`user-guide/`)
- **USER-GUIDE.md** - Main user documentation
- **CAMPAIGN-USER-GUIDE.md** - Campaign system user guide
- **WORKFLOW-GUIDE.md** - Scanning workflows
- **WAVE-WORKFLOW-GUIDE.md** - Wave-based campaign workflows
- **CAMPAIGN-QUICKSTART.md** - Quick start for campaigns

### Developer Guide (`developer-guide/`)
- **CONFIG-SYSTEM-DESIGN.md** - Configuration system architecture

### Campaign Documentation (`campaigns/`)
- **CAMPAIGN-RAMPUP-PLAN.md** - Campaign scaling strategy
- **WAVE9-PLANNING.md** - Wave 9 (500+ packages) planning

### Evidence Documentation (`evidence/`)
- **EVIDENCE-DETECTION-STATUS.md** - Evidence detection testing status
- **EVIDENCE-SCAN-AND-LONGRUNNING-DESIGN.md** - Long-running campaign design
- **WAVE8-RESULTS-ANALYSIS.md** - Wave 8 results analysis

### Binary Consolidation (`binaryconsolidation/`)
- Documentation from binary consolidation project

---

## Current Status

### ✅ Production Ready Features
- Whitelist integration (prevents false positives)
- LLM Tier 1 + Tier 2 analysis
- Evidence collection
- Campaign orchestration
- TUI monitoring
- Report generation (Markdown, JSON, SARIF)
- Checkpoint/resume

### 🟡 Active Scans
- Wave 9: 481 packages (running)

### 📋 Next Steps
1. Organize remaining documentation
2. Add test coverage
3. Update README with new features
4. Prepare for 5000+ package scans

---

## Key Documents

### For Users
1. Start with [USER-GUIDE.md](user-guide/USER-GUIDE.md)
2. Run [CAMPAIGN-QUICKSTART.md](user-guide/CAMPAIGN-QUICKSTART.md)
3. Reference [CAMPAIGN-USER-GUIDE.md](user-guide/CAMPAIGN-USER-GUIDE.md) for campaigns

### For Developers
1. Read [CONFIG-SYSTEM-DESIGN.md](developer-guide/CONFIG-SYSTEM-DESIGN.md)
2. Review [CAMPAIGN-RAMPUP-PLAN.md](campaigns/CAMPAIGN-RAMPUP-PLAN.md)
3. Check [EVIDENCE-DETECTION-STATUS.md](evidence/EVIDENCE-DETECTION-STATUS.md) for detection status

### For Security Researchers
1. Review [EVIDENCE-DETECTION-STATUS.md](evidence/EVIDENCE-DETECTION-STATUS.md)
2. Check [WAVE8-RESULTS-ANALYSIS.md](evidence/WAVE8-RESULTS-ANALYSIS.md)
3. Reference [WAVE9-PLANNING.md](campaigns/WAVE9-PLANNING.md) for large-scale scanning

---

## Version History

| Version | Date | Key Changes |
|---------|------|-------------|
| v0.25.0 | Mar 23, 2026 | Evidence detection fixed, whitelist working |
| v0.24.0 | Mar 23, 2026 | Wave 10 complete (611 packages) |
| v0.23.0 | Mar 23, 2026 | Wave 9 complete (481 packages) |
| v0.22.0 | Mar 23, 2026 | Wave 8 complete (68 packages) |
| v0.21.0 | Mar 23, 2026 | Whitelist fixes, evidence detection |
| v0.20.0 | Mar 23, 2026 | Tests fixed, real campaign run |

---

**Last Updated:** March 23, 2026
**Maintained by:** Glassworks Security Team
