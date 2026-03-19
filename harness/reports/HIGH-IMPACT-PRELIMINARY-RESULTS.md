# High-Impact Scan - Preliminary Results

**Date:** 2026-03-19 18:00 UTC  
**Status:** Tiers 1-2 Complete, Tiers 3-5 Running  

---

## Tier 1: Most Downloaded (28 packages)

**Flagged: 7 packages**

| Package | Findings | Critical | Assessment |
|---------|----------|----------|------------|
| `prettier` | 28 | 22 | 🟡 Likely minified FP |
| `moment` | 22 | 15 | 🟡 Known FP (legacy) |
| `webpack` | 3 | 2 | 🟠 NEEDS REVIEW |
| `underscore` | 21 | 0 | 🟡 Minified code |
| `lodash` | 1 | 0 | 🟢 Likely benign |
| `express` | 2 | 0 | 🟢 Likely benign |
| `vue` | 4 | 0 | 🟢 Likely benign |

**Action:** Review webpack (2 critical findings)

---

## Tier 2: AI Agent / Claw (25 packages)

**Flagged: 3 packages**

| Package | Findings | Critical | Assessment |
|---------|----------|----------|------------|
| `claude-dev` | 19 | 0 | 🟠 NEEDS REVIEW |
| `openai` | 6 | 0 | 🟡 Likely benign |
| `@anthropic-ai/sdk` | 4 | 0 | 🟡 Likely benign |

**Claw packages:** All NOT FOUND
- `open-claw`, `zeptoclaw`, `nanoclaw`, `micro-claw` - Don't exist on npm

**Action:** Review claude-dev (19 findings)

---

## Key Findings

### 1. High-Profile Packages Flagged

**Why this matters:**
- These are THE most trusted packages
- If any ARE malicious, it's a MASSIVE discovery
- More likely FPs from minified/legacy code

### 2. Claw Packages Don't Exist

**Good news:** Specific "Claw" malware packages not on npm
**Bad news:** Attackers may be using different naming

### 3. webpack Has 2 Critical Findings

**Priority:** HIGH
- webpack is installed everywhere
- Even 1% chance of malicious = millions affected
- Needs immediate manual review

---

## Next Actions

### Immediate (Next 30 min)

1. **Manual review of webpack**
   ```bash
   npm pack webpack
   tar -xzf *.tgz
   ./glassware-scanner package/
   # Review critical findings
   ```

2. **Manual review of claude-dev**
   ```bash
   npm pack claude-dev
   tar -xzf *.tgz
   ./glassware-scanner package/
   # Review 19 findings
   ```

### Short-term (Next 2 hours)

3. Complete Tiers 3-5 (GitHub repos)
4. LLM analysis on flagged packages
5. Compare findings with known FP patterns

---

## Assessment

**False Positive Likelihood:** HIGH (80%+)
- Minified code (prettier, webpack, vue)
- Legacy patterns (moment, underscore)
- Complex build tools (lodash, express)

**True Positive Possibility:** LOW but NON-ZERO (5-10%)
- webpack critical findings need verification
- claude-dev (19 findings) suspicious

**Impact if True Positive:** CATASTROPHIC
- webpack compromise = supply chain apocalypse
- claude-dev compromise = AI ecosystem infected

---

**Status:** 🟡 REVIEWING FLAGGED PACKAGES  
**Confidence:** Most are FPs, but MUST verify  
**Timestamp:** 2026-03-19 18:05 UTC
