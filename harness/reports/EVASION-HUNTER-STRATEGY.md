# Evasion Hunter Scan Strategy

**Date:** 2026-03-19 17:30 UTC  
**Objective:** Find GlassWare worms using attacker evasion patterns  

---

## Strategic Analysis

### What We've Scanned ✅

| Category | Scanned | Malicious | Notes |
|----------|---------|-----------|-------|
| High-risk npm | 622 | 0 | AI/ML, native build, install scripts |
| VSCode extensions | 176 | 0 | All minified code FPs |
| GitHub MCP | ~200 | 0 | Clean |
| GitHub VSCode | ~400 | 0 | Clean |
| GitHub Cursor | ~100 | 0 | Clean |
| GitHub DevTools | ~200 | 0 | Clean |
| **Total** | **~1,700** | **0** | **1 confirmed (@iflow-mcp from earlier)** |

### Attacker Mindset: How to Evade Us 🎯

**Observation:** Attackers know we're scanning. They'll adapt.

**Evasion Pattern 1: Typosquatting**
- We scan: `lodash`, `axios`, `express`
- They publish: `lodahs`, `axi0s`, `expres`
- **Why it works:** Developers make typos, install wrong package

**Evasion Pattern 2: Organization Impersonation**
- We scan: `@iflow-mcp`, `@aifabrix` (known malicious)
- They publish: `@microsoft-utils`, `@aws-tools`
- **Why it works:** Trust in big names, less scrutiny

**Evasion Pattern 3: Fork Bombs**
- We scan: Popular packages
- They publish: "lodash-maintained", "express-active-fork"
- **Why it works:** Developers want "maintained" forks of abandoned projects

**Evasion Pattern 4: Abandoned Repo Takeover**
- We scan: Active repos
- They compromise: Abandoned repos with many dependents
- **Why it works:** Maintainers gone, no one watching

**Evasion Pattern 5: Time-Delayed Payloads**
- We scan: New packages immediately
- They: Publish clean, add malicious code in update weeks later
- **Why it works:** We don't re-scan old packages

**Evasion Pattern 6: Dependency Confusion**
- We scan: Public registry
- They publish: Higher versions of internal company packages
- **Why it works:** Companies accidentally install public version

---

## Evasion Hunter Scan Design

### Tier 1: Typosquatting Hunt (36 packages)

**Target:** Common typos of popular packages

```
lodahs, ladash, ldash (lodash)
axi0s, axois, axos (axios)
expres, expresss (express)
reacct, react-lib (react)
momnet, mment (moment)
...
```

**Status:** ✅ Complete  
**Result:** 16 scanned, 20 errors (not found)  
**Flagged:** 0

**Interpretation:** Most typosquats don't exist yet OR have been removed

---

### Tier 2: Organization Impersonation (12 packages)

**Target:** Fake Microsoft/Google/AWS/Facebook packages

```
@microsoft-utils/request
@google-cloud-tools/storage
@aws-sdk-extras/s3
@facebook-react/components
@vue-official/core
...
```

**Status:** ✅ Complete  
**Result:** 0 scanned, 12 errors (not found)  
**Flagged:** 0

**Interpretation:** npm has been removing impersonation packages OR attackers haven't tried this yet

---

### Tier 3: Fork Bombs (14 packages)

**Target:** "Maintained" forks of popular packages

```
lodash-maintained
express-active-fork
axios-updated
react-community-fork
...
```

**Status:** ✅ Complete  
**Result:** 0 scanned, 14 errors (not found)  
**Flagged:** 0

**Interpretation:** Fork bombs don't exist on npm yet OR removed quickly

---

### Tier 4: GitHub Abandoned Repos (300 repos)

**Target:** Abandoned repos with >500 stars

**Search queries:**
- `pushed:<2024-01-01 stars:>500`
- `abandoned maintained fork`
- `unmaintained alternative`

**Status:** 🟡 Running  
**Expected:** 300 repos, 2-4 hours  
**Why GitHub?** Attackers can takeover abandoned repos that still have dependents

---

### Tier 5: Recent Updates (Coming Next)

**Target:** Packages updated in last 7 days with >10k weekly downloads

**Rationale:** Catch time-delayed payloads (clean first, malicious update)

**Implementation:** Query npm API for recently updated popular packages

---

## Early Results Analysis

### Key Finding: 0 Malicious Found

**Possible explanations:**

1. **Attackers are patient** - They're not rushing to publish
2. **npm is effective** - Removing malicious packages quickly
3. **We're looking in wrong places** - Need to think more creatively
4. **Attack moved to other platforms** - PyPI, Cargo, Go modules?

### High Error Rate (Not Found)

**Tier 1:** 20/36 not found (56%)  
**Tier 2:** 12/12 not found (100%)  
**Tier 3:** 14/14 not found (100%)

**Interpretation:**
- ✅ Good news: Most evasion patterns don't exist yet
- ⚠️ Bad news: We might be thinking like defenders, not attackers

---

## Next Strategic Thinking

### What Would I Do as an Attacker?

**1. Target Supply Chain Dependencies**
- Compromise `minimist`, `inherits`, `safe-buffer` (used by everything)
- One compromise → thousands of downstream packages

**2. Exploit Trust Relationships**
- Compromise maintainer account of popular package
- Push malicious update from legitimate account

**3. Use AI-Generated Code**
- LLM-generated code looks legitimate
- Harder to detect with pattern matching

**4. Target Build Tools**
- Compromise webpack plugins, babel presets, eslint plugins
- Inject malicious code during build, not in package

**5. Exploit CI/CD Pipelines**
- Compromise GitHub Actions, npm publish workflows
- Inject malicious code during release process

---

## Recommended Next Scans

### Scan 6: Supply Chain Dependencies (500 packages)

```bash
# Most depended-upon packages
cat > supply-chain.txt << EOF
minimist
inherits
safe-buffer
lodash
async
chalk
commander
debug
underscore
moment
EOF
```

### Scan 7: Build Tools (300 packages)

```bash
# Webpack/Babel/ESLint plugins
cat > build-tools.txt << EOF
webpack-plugin-*
babel-preset-*
eslint-plugin-*
rollup-plugin-*
vite-plugin-*
EOF
```

### Scan 8: Maintainer Account Scan

```bash
# Scan all packages by specific maintainers
# (requires npm API access)
```

---

## Monitoring Commands

```bash
# Check Tier 4 progress
cd harness
tail -f evasion-github.log

# Check results
cat evasion-t*-results.json | jq -s 'map({tier: .name, scanned: .scanned, flagged: .flagged})'

# View flagged packages
cat evasion-*-results.json | jq -r '.flagged_packages[]?.package' 2>/dev/null
```

---

**Status:** Tiers 1-3 complete (0 malicious), Tier 4 running  
**Next:** Analyze results, plan Scan 6-8  
**Timestamp:** 2026-03-19 17:35 UTC
