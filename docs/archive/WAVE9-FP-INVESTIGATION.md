# Wave 9 False Positive Investigation

**Date:** 2026-03-24
**Status:** 🟡 INVESTIGATION IN PROGRESS
**Priority:** CRITICAL - Blocks production use

---

## Problem Statement

Wave 9 flagged **57+ packages as malicious** including:
- `@angular/core`, `@angular/cli`, `@angular/material` - Official Angular packages
- `@vueuse/core` - Vue composition utilities
- `mobx`, `mobx-react` - Popular state management
- `react-native` - Official React Native framework
- `dotenv` - Environment variable loader (18M+ weekly downloads)
- `graphql`, `graphql-tag` - GraphQL foundation
- `@azure/msal-browser` - Microsoft authentication
- `@formatjs/*` - Internationalization libraries

**These are almost certainly FALSE POSITIVES.**

---

## Initial Analysis

### Sample Investigation: @angular/core@17.1.0

**Findings:** 114 total
- **105 InvisibleCharacter** (U+FFFD replacement character)
- **8 GlasswarePattern** (eval_pattern confidence 95%)
- **1 Unknown**

**Root Cause:** 
- U+FFFD in i18n/locale data files (bundled with Angular)
- eval_pattern likely from minified/bundled code

**Verdict:** ❌ **FALSE POSITIVE** - Needs whitelisting

### Sample Investigation: mobx@6.12.0

**Findings:** 10 total
- **2 GlasswarePattern**
- **8 Unknown**

**LLM Analysis:** `malicious=true, confidence=0.75`

**BUT:** mobx IS in whitelist (state_management), so NOT flagged as malicious.

**Verdict:** ✅ **Whitelist working** - But LLM needs tuning

---

## Whitelist Gaps Identified

Wave 9 whitelist is MISSING major categories:

### 1. Web Frameworks
```toml
# MISSING - Should add:
frameworks = [
    "@angular/*",
    "@angular/core",
    "@angular/cli",
    "@angular/material",
    "@angular/common",
    "@angular/compiler",
    "@angular/router",
    "@angular/forms",
    "@angular/platform-browser",
]
```

### 2. Vue Ecosystem
```toml
# MISSING - Should add:
vue_ecosystem = [
    "@vueuse/*",
    "@vueuse/core",
    "@vueuse/firebase",
    "vue",
    "vuex",
    "vue-router",
    "nuxt",
    "pinia",
]
```

### 3. State Management
```toml
# Wave 9 has empty state_management!
state_management = [
    "mobx",
    "mobx-react",
    "mobx-react-lite",
    "vuex",
    "pinia",
    "recoil",
    "zustand",
    "jotai",
    "valtio",
    "immer",
]
```

### 4. i18n Libraries (Beyond moment)
```toml
# MISSING - Should add:
i18n_extended = [
    "@formatjs/*",
    "@formatjs/icu-messageformat-parser",
    "@formatjs/intl-displaynames",
    "@formatjs/intl-listformat",
    "@formatjs/intl-numberformat",
    "@formatjs/intl-relativetimeformat",
    "intl",
    "i18n-iso-countries",
    "i18next",
    "react-intl",
    "globalize",
]
```

### 5. Developer Tools
```toml
# MISSING - Should add:
dev_tools = [
    "dotenv",
    "graphql",
    "graphql-tag",
    "apollo-client",
    "@apollo/client",
    "webpack",
    "vite",
    "rollup",
    "esbuild",
    "babel",
    "@babel/*",
    "typescript",
    "eslint",
    "prettier",
]
```

### 6. Cloud/Authentication SDKs
```toml
# MISSING - Should add:
cloud_sdks = [
    "@azure/*",
    "@azure/msal-browser",
    "@azure/msal-node",
    "@firebase/*",
    "firebase",
    "firebase-admin",
    "@aws-sdk/*",
    "@google-cloud/*",
]
```

### 7. React Native Ecosystem
```toml
# MISSING - Should add:
react_native = [
    "react-native",
    "@react-native/*",
    "@react-native-firebase/*",
    "expo",
    "expo-*",
    "react-native-maps",
    "react-native-vector-icons",
    "react-native-reanimated",
    "react-native-screens",
    "react-native-safe-area-context",
]
```

---

## Investigation Methodology

### Step 1: Identify High-FP Categories

From Wave 9 logs, extract packages flagged with:
- >50 findings (likely i18n/locale data)
- InvisibleCharacter category dominant
- Popular packages (>1M weekly downloads)

### Step 2: LLM-Assisted Investigation

For each suspicious package:
```bash
./target/release/glassware scan-npm <package> --llm
```

Review LLM verdict:
- If `confidence < 0.5` → Likely FP
- If `confidence > 0.8` → Investigate manually
- Check if package is in whitelist

### Step 3: Manual Verification

For high-confidence flags:
1. Check npm download stats
2. Review package publisher (official org?)
3. Check GitHub repo stars/contributors
4. Look for security advisories
5. Compare with known malicious patterns

### Step 4: Whitelist Addition

If confirmed FP:
1. Add to appropriate whitelist category
2. Document reason for addition
3. Test with re-scan
4. Update campaign configs

---

## Action Items

### Immediate (This Session)

1. **Stop Wave 9/10/12 runs** - Prevent more FPs
2. **Extract FP list** - Parse Wave 9 logs for flagged packages
3. **Categorize FPs** - Group by ecosystem (Angular, Vue, etc.)
4. **Add to whitelist** - Update global config and campaign configs
5. **Re-run Wave 9** - Validate FP rate reduced

### Short-Term (This Week)

1. **LLM batch analysis** - Run LLM on top 50 flagged packages
2. **Manual review** - Investigate packages with high LLM confidence
3. **Whitelist expansion** - Add all confirmed FPs
4. **Tune eval_pattern detector** - Reduce minified code FPs
5. **Add U+FFFD exception** - Replacement character in i18n files

### Medium-Term (Next Week)

1. **Implement tiered whitelisting:**
   - Tier 1: Official frameworks (Angular, Vue, React)
   - Tier 2: Popular libraries (>1M downloads)
   - Tier 3: Community packages (case-by-case)

2. **Improve detectors:**
   - eval_pattern: Check for minification markers
   - InvisibleCharacter: Exception for U+FFFD in locale files
   - Add file path heuristics (skip *.d.ts, *.min.js)

3. **Add FP regression tests:**
   - Test popular packages never flag
   - CI/CD whitelist validation

---

## Testing Commands

```bash
# Investigate specific package
./target/release/glassware scan-npm @angular/core@17.1.0 --llm

# Check findings breakdown
./target/release/glassware scan-npm @angular/core@17.1.0 2>&1 | grep -A 20 "Findings by"

# Test whitelist addition
./target/release/glassware scan-npm mobx@6.12.0  # Should NOT flag (whitelisted)

# Re-run Wave 9 with updated whitelist
rm .glassware-orchestrator-cache.db
./target/release/glassware campaign run campaigns/wave9-500plus.toml
```

---

## Preliminary Whitelist Additions

Based on initial analysis, recommend adding:

```toml
[settings.whitelist]
# Web frameworks
frameworks = [
    "@angular/*",
    "@angular/core",
    "@angular/cli", 
    "@angular/common",
    "@angular/compiler",
    "@angular/core",
    "@angular/elements",
    "@angular/forms",
    "@angular/material",
    "@angular/platform-browser",
    "@angular/platform-server",
    "@angular/router",
    "@angular/service-worker",
    "@angular/upgrade",
]

# Vue ecosystem
vue = [
    "@vueuse/*",
    "vue",
    "vuex",
    "vue-router",
    "pinia",
    "nuxt",
    "@nuxt/*",
]

# State management
state_management = [
    "mobx",
    "mobx-react",
    "mobx-react-lite",
    "mobx-state-tree",
    "vuex",
    "pinia",
    "recoil",
    "zustand",
    "jotai",
    "valtio",
    "immer",
    "effector",
]

# i18n extended
i18n = [
    "@formatjs/*",
    "intl",
    "i18n-iso-countries",
    "i18next",
    "react-intl",
    "globalize",
    "cldr",
    "cldrjs",
]

# Developer tools
dev_tools = [
    "dotenv",
    "graphql",
    "graphql-tag",
    "@apollo/client",
    "apollo-client",
    "webpack",
    "webpack-cli",
    "webpack-dev-server",
    "vite",
    "rollup",
    "esbuild",
]

# Cloud SDKs
cloud = [
    "@azure/*",
    "@firebase/*",
    "firebase",
    "firebase-admin",
    "@aws-sdk/*",
    "@google-cloud/*",
    "@supabase/*",
]

# React Native
react_native = [
    "react-native",
    "@react-native/*",
    "@react-native-firebase/*",
    "expo",
    "expo-*",
]
```

---

**Last Updated:** 2026-03-24 08:15 UTC
**Investigator:** Qwen-Coder
**Status:** Ready for systematic investigation
