# High-Impact Target Scan Strategy

**Date:** 2026-03-19 17:45 UTC  
**Objective:** Scan where attackers would strike for MAXIMUM IMPACT

---

## Strategic Pivot

**Previous:** Evasion patterns (typosquats, fake orgs) → 0 malicious  
**New:** High-impact targets (popular, trusted, hot topics) → Targeting attacker sweet spots

---

## Attacker Psychology

> "Why compromise a typosquat when I can compromise the real thing?"

| Target | Why Attackers Want It |
|--------|----------------------|
| **High downloads** | Millions of victims per compromise |
| **Many dependents** | Supply chain multiplier effect |
| **Hot topics (AI/Claw)** | Less scrutiny, high trust |
| **High PR volume** | Overwhelmed maintainers = easier injection |
| **Most starred** | Highest trust = least suspicion |

---

## Scan Tiers

### Tier 1: Most Downloaded (30 pkgs)
lodash, axios, express, react, typescript, webpack...

### Tier 2: AI Agent / MCP / Claw (30 pkgs) ⭐
langchain, openai, anthropic, mcp, **open-claw, zeptoclaw, nanoclaw**, cursor, copilot...

### Tier 3: High PR Repos - GitHub (300 repos)
langchain-ai/langchain, anthropics/*, openai/*, huggingface/*...

### Tier 4: Most Starred - GitHub (250 repos)
vuejs/core, facebook/react, microsoft/TypeScript, vitejs/vite...

### Tier 5: Most Depended-Upon (20 pkgs)
lodash, async, chalk, minimist, safe-buffer, inherits...

**Total:** ~630 high-value targets  
**ETA:** 4-8 hours

---

## Why "Claw" Packages Are Critical

**Intel:** Attackers targeting AI/autonomous agent space

**Claw packages:**
- open-claw, zeptoclaw, nanoclaw, micro-claw
- New ecosystem = less security scrutiny
- Autonomous behavior = perfect cover for malicious actions
- Network access expected = C2 communication blends in

---

## Monitoring

```bash
cd harness
tail -f high-impact-background.log
cat high-impact-t*-results.json | jq '{scanned, flagged}'
```

---

**Status:** 🟡 Running (5 tiers)  
**Confidence:** HIGH (attacker sweet spots)  
**Timestamp:** 2026-03-19 17:50 UTC
