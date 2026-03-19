# LLM Automation Proposal - Autonomous Scanning Agent

**Date:** 2026-03-19  
**Status:** Proposal / Design Document  
**Priority:** Medium-term (after core optimizations)

---

## Vision

An autonomous agent that can:
1. **Discover** new packages to scan from npm registry
2. **Prioritize** which packages to scan based on risk signals
3. **Execute** scans using glassware
4. **Analyze** findings with LLM
5. **Decide** next actions (re-scan, flag for review, ignore)
6. **Report** findings automatically

---

## Architecture Options

### Option A: Fully Autonomous (High Risk)

```
┌─────────────────────────────────────────────────────────┐
│                    LLM Agent Core                        │
├─────────────────────────────────────────────────────────┤
│  - Package discovery (npm API)                           │
│  - Risk scoring & prioritization                         │
│  - Scan execution orchestration                          │
│  - Finding analysis & classification                     │
│  - Decision making (scan more / stop / report)          │
└─────────────────────────────────────────────────────────┘
         │                    │                    │
         ▼                    ▼                    ▼
   ┌──────────┐        ┌──────────┐        ┌──────────┐
   │  npm API │        │ glassware│        │  LLM API │
   └──────────┘        └──────────┘        └──────────┘
```

**Pros:**
- Runs 24/7 without human intervention
- Can discover novel attack patterns
- Scales infinitely

**Cons:**
- **High cost** - LLM API calls at scale ($$$)
- **Hallucination risk** - May make up package names or findings
- **Wrong decisions** - Could skip important packages or waste resources
- **Rate limit issues** - Both npm and LLM APIs have limits
- **No accountability** - Hard to audit decisions

---

### Option B: Semi-Automated with Checkpoints (Recommended)

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   LLM Suggests  │ →   │  Human Reviews  │ →   │  Agent Executes │
│   Next Actions  │     │  & Approves     │     │  Approved Plan  │
└─────────────────┘     └─────────────────┘     └─────────────────┘
         ▲                                            │
         │                                            ▼
         └───────────────────────────────────┐
                                             │
                                      ┌──────────────┐
                                      │   Results    │
                                      │   & Report   │
                                      └──────────────┘
```

**Workflow:**
1. LLM analyzes current state → suggests next batch to scan
2. Human reviews suggestions → approves/rejects/modifies
3. Agent executes approved scans autonomously
4. LLM analyzes results → generates report
5. Human reviews flagged packages before disclosure

**Pros:**
- Human oversight on critical decisions
- Lower cost (fewer unnecessary LLM calls)
- Audit trail of decisions
- Catches hallucinations before execution

**Cons:**
- Requires human time (but much less than manual)
- Slower than fully autonomous

---

### Option C: Batch Automation (Low Risk)

```
Pre-configured batches → Automated scan → LLM analysis → Human review
```

**Workflow:**
1. Human defines batches (e.g., "500 AI/ML packages")
2. System scans batch automatically
3. LLM analyzes all findings
4. Human reviews only flagged packages

**Pros:**
- Minimal human time required
- Predictable cost
- Easy to audit

**Cons:**
- Less adaptive than agent-based approach
- Can't discover new categories dynamically

---

## Recommended Implementation Path

### Phase 1: Batch Automation (Now)
**Timeline:** 1-2 days  
**Effort:** Low

```python
# Pre-configured batch scanning
python diverse_sampling.py --categories ai-ml crypto devtools --samples 500
python optimized_scanner.py diverse-500.txt --cache
python batch_llm_analyzer.py flagged-500.txt
```

**Features:**
- Category-based sampling
- Package cache (already implemented)
- Batch LLM analysis

---

### Phase 2: LLM-Assisted Prioritization (Next Week)
**Timeline:** 3-5 days  
**Effort:** Medium

```python
# LLM suggests which categories to scan next
python llm_prioritizer.py --context scan-results.json
# Output: "Recommend scanning 'install-scripts' category next"
```

**Features:**
- LLM analyzes scan results
- Suggests next categories/packages
- Human approves before execution

---

### Phase 3: Semi-Autonomous Agent (Future)
**Timeline:** 2-3 weeks  
**Effort:** High

```python
# Agent loop with human checkpoints
while True:
    # Agent analyzes state
    suggestions = agent.suggest_next_actions()
    
    # Human reviews (async, can be hours later)
    approved = human.review_and_approve(suggestions)
    
    # Agent executes approved actions
    results = agent.execute(approved)
    
    # Agent generates report
    report = agent.generate_report(results)
    
    # Human reviews flagged packages
    human.review_flagged(report.flagged)
```

**Features:**
- Autonomous execution between checkpoints
- LLM-powered decision making
- Human oversight on critical decisions
- Full audit trail

---

## Cost Analysis

### Current Manual Process (Per 500 packages)

| Step | Time | Cost |
|------|------|------|
| Define categories | 15 min | Human time |
| Run sampling | 5 min | Human time |
| Run scan | 30 min | Human monitoring |
| LLM analysis | 30 min | ~$5 API cost |
| Review findings | 2 hours | Human time |
| **Total** | **~3.5 hours** | **~$5 + human time** |

### Fully Autonomous (Per 500 packages)

| Step | Time | Cost |
|------|------|------|
| Agent discovery | N/A | ~$10 LLM calls |
| Agent prioritization | N/A | ~$5 LLM calls |
| Scan execution | N/A | $0 |
| LLM analysis | N/A | ~$5 API cost |
| Agent decisions | N/A | ~$10 LLM calls |
| **Total** | **0 human time** | **~$30 API cost** |

### Semi-Autonomous (Per 500 packages)

| Step | Time | Cost |
|------|------|------|
| Human approves batch | 5 min | Human time |
| Agent executes | N/A | ~$5 LLM calls |
| LLM analysis | N/A | ~$5 API cost |
| Human reviews flagged | 30 min | Human time |
| **Total** | **~35 min** | **~$10 + human time** |

**Savings:** 90% human time reduction, 67% cost reduction vs fully autonomous

---

## Risk Mitigation

### Hallucination Prevention

1. **Ground truth verification** - Agent must verify package exists before scanning
2. **Finding validation** - Cross-check LLM findings against glassware output
3. **Confidence thresholds** - Only act on high-confidence decisions
4. **Human checkpoint** - Critical decisions require approval

### Cost Control

1. **Daily budget limits** - Stop if API costs exceed threshold
2. **Batch processing** - Group LLM calls for efficiency
3. **Cache everything** - Never re-scan same package
4. **Tiered analysis** - Simple findings → no LLM, complex → LLM

### Rate Limit Management

1. **npm registry** - Respect rate limits, add delays
2. **LLM API** - Implement token bucket rate limiter
3. **Backoff strategy** - Exponential backoff on 429 errors

---

## Technical Requirements

### Infrastructure

- [ ] Database for scan results (already have corpus.db)
- [ ] Queue system for job management (Redis or SQLite queue)
- [ ] Logging and audit trail
- [ ] Alert system for critical findings

### LLM Integration

- [ ] Multi-model support (NVIDIA, Cerebras, Groq)
- [ ] Prompt versioning and A/B testing
- [ ] Response validation and parsing
- [ ] Context management for long-running sessions

### Agent Framework

- [ ] State machine for agent workflow
- [ ] Decision logging and explainability
- [ ] Human review interface (CLI or web UI)
- [ ] Rollback mechanism for bad decisions

---

## Recommendation

**Start with Phase 1 (Batch Automation) now:**
- Already have all components
- Immediate value with minimal risk
- Learn from real usage before building agent

**Add Phase 2 (LLM Prioritization) next week:**
- Low effort, high value
- Tests LLM decision-making safely
- Builds confidence for Phase 3

**Consider Phase 3 (Semi-Autonomous) after 2-3 weeks:**
- Only after proving Phases 1-2 work well
- Requires robust monitoring and alerting
- Should have clear rollback plan

---

## Downsides of Full Automation

### 1. Loss of Human Intuition
- Humans spot patterns LLMs miss
- Context matters (e.g., package maintainer reputation)
- False positives waste investigation time

### 2. Cost Explosion
- LLM calls add up quickly at scale
- Hallucinations lead to wasted scans
- No natural stopping point

### 3. Accountability Issues
- Who is responsible for missed threats?
- Hard to explain agent decisions to stakeholders
- Audit trail may be incomplete

### 4. Technical Debt
- Agent code is complex and hard to maintain
- Prompt engineering is fragile
- Model changes can break behavior

---

## Conclusion

**Recommended approach:** Semi-automated with human checkpoints

**Rationale:**
- Best balance of efficiency and oversight
- 90% human time reduction achievable
- Maintains accountability
- Allows learning and iteration

**Next steps:**
1. ✅ Package cache (implemented)
2. ⏳ Run 500-package diverse batch (ready)
3. ⏳ Implement LLM prioritization (Phase 2)
4. ⏳ Evaluate results → decide on Phase 3

---

**Prepared by:** glassware autonomous analysis  
**Date:** 2026-03-19  
**Status:** Ready for review and discussion
