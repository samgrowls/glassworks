# Glassworks Future Roadmap

**Version:** 1.0
**Date:** March 23, 2026
**Status:** Strategic Planning Document

---

## Executive Summary

This document synthesizes research findings and provides a strategic roadmap for glassworks evolution over the next 6-12 months.

**Current State (v0.14.0):**
- ✅ Production-ready campaign orchestration
- ✅ Checkpoint/resume for reliability
- ✅ Markdown reports + LLM query
- ✅ TUI with live monitoring and command palette
- ✅ Package drill-down with LLM analysis

**Strategic Focus Areas:**
1. **Plugin Architecture** - Enable community detectors, broader attack coverage
2. **Long-Running Campaigns** - Support days-long, 100k+ package scans
3. **Auto-Research & Tuning** - Automated detector optimization

---

## Strategic Initiative 1: Plugin Architecture

### Vision

Transform glassworks from a monolithic detector toolkit into an extensible platform where:
- Security researchers can easily contribute new detectors
- Organizations can develop proprietary detectors
- Community shares detectors via crates.io
- Multiple attack vectors covered (Unicode, supply chain, secrets, malware)

### Recommended Approach: Hybrid Architecture

**Primary: Rust Crate Plugins**
- Separate crates implementing `Detector` trait
- Full type safety, zero overhead
- Distribution via crates.io

**Complementary: Configuration-Driven Rules**
- YAML definitions for simple pattern detectors
- No Rust knowledge required
- Hot-reloading support

### Implementation Plan

| Phase | Duration | Deliverables | Resources |
|-------|----------|--------------|-----------|
| **Phase 1: Foundation** | 2 weeks | Plugin API, Registry, Docs | 1 Core Dev |
| **Phase 2: Detector Extraction** | 2 weeks | Migrate 10 existing detectors | 1 Detector Dev |
| **Phase 3: Configuration Support** | 1 week | YAML rules, hot-reload | 1 Core Dev |
| **Phase 4: Ecosystem** | 2-3 weeks | Template repo, CI/CD, tutorials | 1 DevOps |
| **Total** | **6-8 weeks** | Full plugin system | **4 FTE-weeks** |

### Expected Outcomes

**Short-term (3 months):**
- 10+ community-contributed detectors
- Detector development time reduced from days to hours
- Coverage expanded to 5+ attack vector categories

**Long-term (12 months):**
- 50+ detectors in ecosystem
- Community-maintained detector registry
- Enterprise plugin marketplace

### Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Plugin security vulnerabilities | High | Security audit process, signing |
| API breaking changes | Medium | SemVer, compatibility layer |
| Fragmentation | Low | Curated registry, quality standards |

---

## Strategic Initiative 2: Long-Running Campaigns

### Vision

Support reliable, monitorable, manageable campaigns that run for days/weeks scanning 100k-1M+ packages with minimal operator intervention.

### Top 5 Features (Prioritized)

| # | Feature | Rationale | Effort |
|---|---------|-----------|--------|
| **1** | **Adaptive Checkpointing** | Fixed-interval inefficient for days-long runs. Adjusts based on error rate, scan speed, time elapsed. | 2-3 days |
| **2** | **Detached Monitoring** | Operators can't stay tied to terminal. Start-and-detach workflow with re-attach capability. | 2-3 days |
| **3** | **Notification System** | Critical events need immediate attention. Slack, email, webhook support. | 2-3 days |
| **4** | **Progress Trends + ETA Refinement** | Linear ETA misleading. Trend analysis with confidence intervals. | 2-3 days |
| **5** | **Resource Monitoring** | Detect memory leaks, disk growth early. Prevent crashes. | 1 day |

### Implementation Plan

| Phase | Duration | Features | Resources |
|-------|----------|----------|-----------|
| **Phase 1: Core Reliability** | 2 weeks | Adaptive checkpoint, detached monitoring, session persistence | 1 Core Dev |
| **Phase 2: Enhanced Monitoring** | 2 weeks | Resource monitoring, progress trends, ETA refinement, alerts | 1 Core Dev |
| **Phase 3: Management** | 2 weeks | Campaign queue, log rotation, pause/resume scheduling | 1 Core Dev |
| **Phase 4: Advanced** | 2-3 weeks | Multi-campaign orchestration, web dashboard | 1 Full-stack Dev |
| **Total** | **8-9 weeks** | Production-ready long runs | **4 FTE-weeks** |

### Expected Outcomes

**Short-term (3 months):**
- 99% completion rate for 100k+ package campaigns
- <5 minute recovery from transient failures
- <5% checkpoint overhead

**Long-term (12 months):**
- Support for 1M+ package campaigns
- Distributed scanning across multiple nodes
- Web-based monitoring dashboard

### Success Metrics

| Metric | Target |
|--------|--------|
| Reliability | 99% completion rate |
| Recovery | <5 minute recovery time |
| Checkpoint Overhead | <5% performance impact |
| ETA Accuracy | Within 20% of actual |
| Anomaly Detection | 90% within 5 minutes |
| Notification Latency | <1 minute |
| Memory Usage | <1GB for 100k packages |

---

## Strategic Initiative 3: Auto-Research & Detector Tuning

### Vision

Automate detector optimization and threat pattern discovery using:
- LLM-assisted tuning suggestions
- Statistical validation (MAD-based confidence scoring)
- Genetic algorithms for parameter optimization
- Feedback loop from findings to detector refinement

### Research Findings

**Referenced Projects:**
- **YPi (rawwerks/ypi):** Recursive self-delegation, budget controls, file-based context
- **PI Auto-Research (davebcn87/pi-autoresearch):** Experiment loop, MAD confidence scoring, session persistence

**Key Insights:**
- MAD-based confidence scoring highly applicable for validating tuning changes
- Experiment loop (edit → commit → run → log → keep/revert → repeat) perfect for detector tuning
- Budget controls prevent runaway tuning sessions

### Viable Approaches (Ranked)

| Approach | Viability | Effort | Timeline |
|----------|-----------|--------|----------|
| **LLM-Assisted Tuning** | ⭐⭐⭐ High | 2-3 weeks | Immediate (Q2 2026) |
| **Enhanced Manual Tuning** | ⭐⭐⭐ High | 1-2 weeks | Immediate (Q2 2026) |
| **Genetic Algorithms** | ⭐⭐ Medium-High | 3-4 weeks | Short-term (Q3 2026) |
| **Supervised Learning** | ⭐⭐ Medium | 8-12 weeks | Long-term (Q4 2026+) |
| **Reinforcement Learning** | ⭐ Low | 12-16 weeks | Research only |

### Implementation Plan

| Phase | Duration | Deliverables | Resources |
|-------|----------|--------------|-----------|
| **Phase 1: Foundation** | 4 weeks | FP/FN labeling, MAD scoring, LLM prompts, validation pipeline | 1 ML Engineer |
| **Phase 2: Automation** | 4 weeks | Genetic algorithm framework, fitness function, overnight tuning | 1 ML Engineer |
| **Phase 3: Scale** | 4 weeks | Dataset collection, human labeling workflow, simple ML baseline | 1 Data Engineer |
| **Total** | **12 weeks** | `glassware-tune` CLI, automated tuning | **3 FTE-months** |

### Data Requirements

**Current State:**
- ~500-1,000 packages scanned
- ~10-20 known malicious (calibration waves)
- ~50-100 known clean packages

**Requirements by Approach:**
| Approach | Dataset Size | Label Quality |
|----------|-------------|---------------|
| LLM-assisted | 100+ samples | 90%+ accuracy |
| Genetic algorithms | 1,000+ samples | 95%+ accuracy |
| Supervised learning | 10,000+ samples | 99%+ accuracy |

**Gap Analysis:** Current data sufficient for LLM-assisted tuning, borderline for genetic algorithms, insufficient for supervised learning.

### Expected Outcomes

**Short-term (3 months):**
- `glassware-tune` CLI for manual tuning with LLM assistance
- MAD-based confidence scoring for all tuning changes
- 20% improvement in detection accuracy through tuning

**Long-term (12 months):**
- Automated overnight tuning campaigns
- Self-improving detectors
- Community-shared tuning configurations

---

## Cross-Initiative Dependencies

```
Plugin Architecture ─┬─► Better Detectors ─┬─► Auto-Research
                     │                     │
                     │                     ▼
                     │              More Training Data
                     │                     │
                     ▼                     ▼
Long-Running Campaigns ─────► Larger Datasets ──► Better Tuning
```

**Key Insight:** These initiatives reinforce each other:
- Plugin architecture → More detectors → More data for tuning
- Long-running campaigns → Larger datasets → Better ML models
- Auto-research → Better detectors → More effective scanning

---

## Resource Requirements

### Total Investment (6-12 months)

| Initiative | Duration | FTE Required | Total FTE-Months |
|------------|----------|--------------|------------------|
| Plugin Architecture | 6-8 weeks | 2-3 | 4 |
| Long-Running Campaigns | 8-9 weeks | 2 | 4 |
| Auto-Research | 12 weeks | 2-3 | 6 |
| **Total** | **6-12 months** | **4-6 peak** | **14 FTE-months** |

### Recommended Team Composition

- **1 Core Developer** - Plugin API, campaign reliability
- **1 Detector Developer** - Detector extraction, migration
- **1 ML Engineer** - Auto-research, tuning algorithms
- **1 DevOps Engineer** - CI/CD, publishing, monitoring
- **1 Technical Writer** - Documentation, tutorials

---

## Risk Assessment

### Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Plugin API design flaws | Medium | High | Iterative design, community feedback |
| ML model overfitting | Medium | Medium | Cross-validation, holdout testing |
| Checkpoint database corruption | Low | High | Transactional writes, backups |
| Performance regression | Medium | Medium | Benchmark suite, CI checks |

### Business Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Community adoption slow | Medium | Medium | Outreach, tutorials, templates |
| Enterprise features delayed | Low | Low | Prioritize enterprise needs |
| Competitor releases similar | Low | Medium | Focus on differentiation (Unicode expertise) |

---

## Success Criteria (12-Month Horizon)

### Plugin Architecture
- [ ] 50+ community detectors available
- [ ] Detector development time <4 hours
- [ ] 5+ organizations using custom plugins

### Long-Running Campaigns
- [ ] 99% completion rate for 100k+ package campaigns
- [ ] Support for 1M+ package campaigns
- [ ] Web dashboard with real-time monitoring

### Auto-Research
- [ ] 20% improvement in detection accuracy
- [ ] Automated overnight tuning campaigns
- [ ] Community-shared tuning configurations

### Overall
- [ ] 10x increase in active users
- [ ] 5+ enterprise deployments
- [ ] Recognized as leading npm security scanner

---

## Immediate Next Steps (Next 2 Weeks)

1. **Review this roadmap** with team/stakeholders
2. **Prioritize initiatives** based on resources and goals
3. **Create GitHub issues** for Phase 1 of each initiative
4. **Assign owners** for each initiative
5. **Set up project tracking** (GitHub Projects, Jira, etc.)

---

## Appendix: Research Sources

### Plugin Architecture
- Semgrep rules (YAML-based)
- Clippy lints (Rust crate-based)
- ESLint plugins (JavaScript modules)
- Wireshark dissectors (C + Lua)

### Long-Running Campaigns
- Folding@Home (distributed computing)
- BOINC (scientific computing)
- Torrent clients (long-running downloads)
- CI/CD pipelines (long build queues)

### Auto-Research
- YPi (https://github.com/rawwerks/ypi)
- PI Auto-Research (https://github.com/davebcn87/pi-autoresearch)
- AutoML research papers
- Genetic algorithm optimization literature

---

**This roadmap provides a strategic framework for glassworks evolution. Regular review and adjustment recommended based on community feedback and changing requirements.**
