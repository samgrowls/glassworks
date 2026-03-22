# Wave 4 Plan — GlassWorm Hunt (500 Packages)

**Target:** Find active GlassWorm infections in the wild  
**Timeline:** After Wave 3 validation  
**Expected Duration:** 2-3 hours

---

## Target Categories (GlassWorm-Specific)

### 1. React Native Ecosystem (High Priority)

Based on confirmed GlassWorm targets:
- `react-native-country-select` (confirmed malicious)
- `react-native-international-phone-number` (confirmed malicious)

**Similar packages to scan:**
```
react-native-country-picker
react-native-country-code
react-native-phone-input
react-native-phone-number-input
react-native-international-phone-picker
react-native-otp-input
react-native-sms-retriever
react-native-phone-verify
react-native-flag
react-native-flags
react-native-countries
react-native-region-picker
react-native-locale-picker
react-native-localize
react-native-i18n
react-native-globalize
```

### 2. MCP (Model Context Protocol) Servers

Recent campaign targeting MCP infrastructure:
```
@mcp/*
mcp-server
mcp-client
model-context-protocol
ai-connector
llm-connector
cursor-mcp
claude-mcp
```

### 3. VS Code Extensions

Original GlassWorm targets (72+ extensions compromised):
```
github-repository search:
- "vscode extension" + "country"
- "vscode extension" + "phone"
- "vscode extension" + "locale"
- "vscode extension" + "i18n"
```

### 4. Unicode/Locale Heavy Packages

GlassWorm hides payloads in locale data:
```
globalize
cldr-*
i18n-*
locale-*
country-*
language-*
timezone-*
date-format-*
number-format-*
```

### 5. Packages with Install Scripts

Common infection vector:
```
Packages with:
- install.js
- preinstall.js
- postinstall.js
- node-gyp rebuild
```

### 6. Recently Updated (Last 3 Months)

Fresh infections more likely:
```bash
# Search npm for recently updated
npm view pkg time.modified
# Filter: 2026-01-01 to present
```

---

## Search Strategy

### npm Registry Searches

```bash
# React Native country/phone packages
npm search react-native-country
npm search react-native-phone
npm search react-native-otp
npm search react-native-locale

# i18n packages
npm search globalize
npm search i18n
npm search locale

# Recently updated
npm view <pkg> time.modified
```

### GitHub Repository Searches

```bash
# Using glassware-orchestrator
glassware-orchestrator search-github "react-native country picker" --max-results 50
glassware-orchestrator search-github "react-native phone input" --max-results 50
glassware-orchestrator search-github "mcp server" --max-results 50
glassware-orchestrator search-github "vscode extension locale" --max-results 50
```

---

## Wave 4 Package List (500 Target)

### Evidence (4 packages)
- react-native-country-select@0.3.91
- react-native-international-phone-number@0.11.8
- aifabrix-miso-client@4.7.2
- iflow-mcp-watercrawl-mcp@1.3.4

### React Native Ecosystem (100 packages)
- All country picker variants
- All phone input variants
- All OTP/SMS packages
- All locale/regional packages

### MCP/AI Infrastructure (100 packages)
- MCP servers and clients
- AI connectors
- LLM integration packages

### Unicode/Locale Heavy (100 packages)
- globalize and variants
- CLDR data packages
- i18n frameworks
- Date/time formatting

### Install Script Heavy (100 packages)
- Native module builders
- Prebuild packages
- node-gyp dependent

### Random Recent (96 packages)
- Recently updated packages
- Low-maintenance packages
- Single-author packages

---

## Detection Criteria

### High Priority Signals

1. **Known C2 Wallets/IPs**
   - BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC
   - 28PKnu7RzizxBzFPoLp69HLXp9bJL3JFtT2s5QzHsEA2
   - 45.32.150.251, 45.32.151.157, etc.

2. **GlassWorm Patterns**
   - Stego decoder patterns
   - eval(atob(...)) patterns
   - Header C2 patterns

3. **Behavioral Chains**
   - Obfuscation + Execution
   - Locale check + Network call
   - Time delay + Exfil

### LLM Triage Priority

**Cerebras (Fast) for:**
- All packages scoring > 3.0
- All React Native ecosystem packages
- All MCP packages

**NVIDIA (Deep) for:**
- Packages scoring > 7.0
- Packages with known C2 indicators
- Packages with complex obfuscation

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Known malicious detected | 4/4 (100%) |
| False positive rate | < 2% |
| New suspicious findings | 5-10 |
| Confirmed new malicious | 1+ |
| LLM triage accuracy | > 90% |

---

## Response Plan

### If New Malicious Found

1. **Immediate:**
   - Document findings
   - Capture evidence (tarball)
   - Do NOT alert attacker

2. **Within 24 hours:**
   - Report to npm Security
   - Contact package maintainer (if legitimate owner)
   - Prepare disclosure

3. **Within 72 hours:**
   - Public disclosure (if npm doesn't act)
   - Blog post with findings
   - Update detection signatures

---

## Timeline

| Phase | Duration | Packages |
|-------|----------|----------|
| Wave 4A (React Native) | 30 min | 100 |
| Wave 4B (MCP/AI) | 30 min | 100 |
| Wave 4C (Unicode/Locale) | 30 min | 100 |
| Wave 4D (Install Scripts) | 30 min | 100 |
| Wave 4E (Random Recent) | 30 min | 96 |
| **Total** | **2.5 hours** | **500** |

---

## Post-Wave Analysis

1. **Generate report:**
```bash
cat wave4-results-*.json | jq '.results[] | select(.is_malicious == true)'
```

2. **Manual review of flagged:**
- Check file paths
- Review suspicious code
- Compare with known GlassWorm patterns

3. **LLM analysis of high-severity:**
```bash
glassware-orchestrator scan-npm suspicious-pkg --llm
```

4. **Evidence preservation:**
```bash
npm pack suspicious-pkg@version
mv suspicious-pkg-version.tgz evidence/
```

---

**Ready to execute after Wave 3 validation.**
