# High-Altitude Review - Optimization Opportunities

**Date:** 2026-03-19  
**Purpose:** Ensure we're not missing simple low-hanging fruit  
**Scope:** Full system review from 30,000 feet  

---

## Current State Summary

### What We Have Built

| Component | Status | Performance | Notes |
|-----------|--------|-------------|-------|
| **Detection Engine** | ✅ Complete | ~50k LOC/sec | 180 tests passing |
| **Bundled Code Filters** | ✅ Enhanced | 80% FP reduction | Added /out/, /umd/, /gyp/, /lib/ |
| **Package Cache** | ✅ Working | 10x speedup on re-scan | 7-day TTL, SHA256-based |
| **Diverse Sampling** | ✅ Rate-limited | 500+ packages without 429 | 13 categories, exponential backoff |
| **Optimized Scanner** | ✅ Parallel | 4-6x faster | 10 workers, cache-enabled |
| **LLM Analyzer** | ✅ Single provider | ~10s/finding | NVIDIA NIM, conservative |
| **Batch LLM** | ✅ Working | 3 workers | Rate-limited, saves reasoning |
| **LLM Prioritizer** | ✅ Implemented | 80% confidence | Suggests next categories |

### What's In Progress

| Component | Status | ETA | Priority |
|-----------|--------|-----|----------|
| **LLM Orchestration** | 📝 Designed | 6 days | High |
| **Scan Orchestration** | 💭 Planned | 3 days | Medium |
| **Multi-Provider LLM** | 📋 Spec'd | Part of orchestration | High |

---

## Low-Hanging Fruit Review

### ✅ Already Picked (Completed)

1. **Bundled code filtering** - 80% FP reduction ✅
2. **Package caching** - 10x speedup on re-scans ✅
3. **npm rate limiting** - 5x more packages before 429 ✅
4. **Parallel scanning** - 4-6x faster ✅
5. **LLM prompt improvement** - 80% FP reduction ✅
6. **Documentation filtering** - Skip .md files ✅
7. **ClojureScript detection** - Skip /out/, cljs_deps.js ✅

### 🟡 Within Reach (Easy Wins)

1. **Size-based heuristics**
   - **Idea:** Packages >1MB are likely bundled
   - **Effort:** 30 minutes
   - **Impact:** Catch large bundled FPs early
   - **Status:** Not yet implemented

2. **Publisher reputation**
   - **Idea:** Trusted publishers (npm, google, microsoft) get higher threshold
   - **Effort:** 2 hours
   - **Impact:** Reduce FPs on well-known packages
   - **Status:** Not yet implemented

3. **Scan result streaming**
   - **Idea:** Write results to JSONL as they complete
   - **Effort:** 1 hour
   - **Impact:** Real-time monitoring, can tail progress
   - **Status:** Not yet implemented

4. **LLM result caching**
   - **Idea:** Cache LLM analyses by package hash
   - **Effort:** 2 hours
   - **Impact:** Avoid re-analyzing same package
   - **Status:** Not yet implemented

5. **Category-based batching**
   - **Idea:** Group packages by category for batch LLM
   - **Effort:** 1 hour
   - **Impact:** More coherent LLM context, better analysis
   - **Status:** Not yet implemented

### 🔴 High Effort / High Reward (Phase 3)

1. **LLM Orchestration** - Multi-provider with failover
   - **Effort:** 6 days
   - **Impact:** No single point of failure, maximize throughput
   - **Status:** Designed, ready to implement

2. **Scan Orchestration** - Coordinated concurrent scans
   - **Effort:** 3 days
   - **Impact:** Run multiple scans without rate limit conflicts
   - **Status:** Planned

3. **ML Pre-filter** - Train classifier to skip 90% clean packages
   - **Effort:** 2-3 weeks
   - **Impact:** Massive speedup, focus on high-risk only
   - **Status:** Future consideration

---

## Architecture Review

### What's Working Well

1. **Modular design** - Easy to add new detectors, providers
2. **Cache layer** - Prevents duplicate work
3. **Rate limiting** - Respects API limits
4. **Evidence preservation** - Full audit trail
5. **Documentation** - Comprehensive handoff docs

### What Could Be Simpler

1. **Multiple config files** - harness/, llm-analyzer/, glassware-core/
   - **Suggestion:** Unified config system
   - **Effort:** 1 day
   - **Priority:** Low

2. **Manual intervention for LLM** - Need to run batch analyzer separately
   - **Suggestion:** Auto-trigger LLM on flagged packages
   - **Effort:** 2 hours
   - **Priority:** Medium

3. **No real-time dashboard** - Must tail logs
   - **Suggestion:** Simple web dashboard or terminal UI
   - **Effort:** 1-2 days
   - **Priority:** Low

### What's Elegant

1. **Provider interface** - Clean abstraction for LLM providers
2. **Cache integration** - Seamless, doesn't complicate scan logic
3. **Batch processing** - Simple but effective
4. **Evidence structure** - Organized, queryable

---

## Performance Bottlenecks

### Current Bottlenecks

| Bottleneck | Impact | Solution | Effort |
|------------|--------|----------|--------|
| **npm API rate limit** | ~100 packages before 429 | ✅ Rate limiting (done) | - |
| **LLM API rate limit** | 30 RPM per provider | Multi-provider orchestration | 6 days |
| **Sequential LLM** | ~10s per finding | Batch analysis (done) | - |
| **Single scan at a time** | Can't parallelize categories | Scan orchestration | 3 days |
| **Large package analysis** | Timeouts on >1MB | Size-based filtering | 30 min |

### After Optimizations

| Bottleneck | Remaining Impact |
|------------|------------------|
| **LLM cost** | $10-30 per 500 packages |
| **Human review time** | ~30 min per 500 packages |
| **Disk space** | ~100MB per 100 flagged packages |

---

## Recommendations

### Immediate (This Week)

1. ✅ **Bundled code filters** - DONE
2. ⏳ **Size-based heuristics** - 30 min, high impact
3. ⏳ **LLM result caching** - 2 hours, avoids re-work
4. ⏳ **Auto-trigger LLM** - 2 hours, better UX

### Short-term (Next Week)

1. **LLM Orchestration Phase 1** - NVIDIA + Cerebras failover
2. **Scan Orchestration** - Coordinated concurrent scans
3. **Publisher reputation** - Trusted publisher list

### Long-term (Phase 3)

1. **Full LLM orchestration** - All providers, all strategies
2. **ML pre-filter** - Train classifier
3. **Real-time dashboard** - Web or TUI

---

## Design Principles Check

### Are We Following Our Own Advice?

| Principle | Status | Notes |
|-----------|--------|-------|
| **Modular** | ✅ Yes | Providers, detectors, scanners all modular |
| **Configurable** | ✅ Yes | YAML configs, env vars |
| **Observable** | ⚠️ Partial | Logs yes, dashboard no |
| **Resilient** | ⚠️ Partial | Rate limiting yes, failover no |
| **Elegant** | ✅ Yes | Simple interfaces, clear separation |

### What Would Break Elegance?

1. **Hardcoding provider logic** - Don't do it
2. **Tight coupling** - Keep orchestrator separate from providers
3. **Global state** - Use dependency injection
4. **Magic numbers** - Config-driven thresholds

---

## Decision Framework

### When to Add New Feature?

**Ask:**
1. Does this reduce false positives? → **YES** = High priority
2. Does this speed up scanning? → **YES** = Medium priority
3. Does this improve reliability? → **YES** = Medium priority
4. Does this add cool tech? → **ONLY IF** #1-3 are satisfied

### When to Refactor?

**Ask:**
1. Is this blocking new features? → **YES** = Refactor now
2. Is this causing bugs? → **YES** = Refactor now
3. Is this ugly but working? → **NO** = Document, refactor later
4. Will this make tests easier? → **MAYBE** = Consider

---

## Current Priority Order

1. ✅ **Bundled code filters** - DONE
2. ⏳ **Size-based heuristics** - 30 min
3. ⏳ **LLM result caching** - 2 hours
4. ⏳ **LLM Orchestration Phase 1** - 2 days (NVIDIA + Cerebras)
5. ⏳ **Scan Orchestration** - 3 days
6. 📋 **Publisher reputation** - 2 hours
7. 📋 **LLM Orchestration Phase 2** - 4 days (full system)

---

## Conclusion

**We're in a good position:**
- Core functionality solid
- FP rate down 80%
- Rate limiting working
- Cache working
- Clear roadmap for next steps

**Biggest risks:**
1. Over-engineering LLM orchestration (start simple)
2. Losing focus (stick to priority order)
3. Premature optimization (measure first)

**Recommendation:**
1. Pick low-hanging fruit (size heuristics, LLM caching)
2. Implement LLM orchestration Phase 1 (simple failover)
3. Test, measure, iterate

---

**Status:** Ready for next implementation phase  
**Next Decision:** Which low-hanging fruit to pick first?
