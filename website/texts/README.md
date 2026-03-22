# GlassWorm Website Texts

**Directory:** `website/texts/`  
**Purpose:** Factual documentation for website content

---

## Files

| File | Purpose | Word Count |
|------|---------|------------|
| `detection-capabilities.md` | Detector overview, threat scoring, known indicators | ~800 |
| `architecture.md` | System architecture, detection flow, data structures | ~1200 |
| `usage-guide.md` | Installation, CLI usage, examples, troubleshooting | ~1500 |

---

## Content Guidelines

### detection-capabilities.md

**Audience:** Security teams, developers evaluating the tool

**Key Points:**
- 22+ detectors across 4 layers
- Threat scoring model (signal stacking)
- Known GlassWorm indicators (wallets, IPs, packages)
- False positive prevention (whitelisting)
- Performance benchmarks

**Tone:** Factual, technical

---

### architecture.md

**Audience:** Developers, integrators

**Key Points:**
- System overview diagram
- Component descriptions (core, CLI, orchestrator, harness)
- Detection flow (Input → Engine → Scoring → Output)
- Data structures (Finding, DetectionCategory, Severity)
- Configuration (env vars, CLI flags)
- Performance characteristics

**Tone:** Technical, reference-style

---

### usage-guide.md

**Audience:** End users

**Key Points:**
- Installation instructions
- CLI commands and options
- Common usage examples
- LLM integration setup
- Output formats (pretty, JSON, SARIF)
- Troubleshooting

**Tone:** Instructional, user-friendly

---

## Update Process

1. **Update content** in respective `.md` file
2. **Update version** at top of file (e.g., `**Version:** v0.11.0+`)
3. **Update "Last Updated"** date
4. **Commit** with descriptive message

Example:
```bash
git add website/texts/detection-capabilities.md
git commit -m "docs: Update detection capabilities for v0.11.0

- Added binary detector documentation (G6-G11)
- Updated threat scoring formula
- Added new GlassWorm C2 wallets/IPs
- Updated false positive whitelist"
```

---

## Related Documentation

**Main Repo (glassworks):**
- `README.md` - Main project overview
- `docs/USER-GUIDE.md` - Extended user documentation
- `docs/WORKFLOW-GUIDE.md` - Workflow documentation

**Archive Repo (glassworks-archive):**
- `development/` - Handoffs, plans, phase reports
- `intelligence/` - Threat intelligence documents
- `harness-reports/` - Scan reports and analysis
- `architecture/` - Historical architecture docs

---

## Contact

- **Repository:** https://github.com/samgrowls/glassworks
- **Issues:** https://github.com/samgrowls/glassworks/issues
- **Threat Intel:** https://codeberg.org/tip-o-deincognito/glassworm-writeup
