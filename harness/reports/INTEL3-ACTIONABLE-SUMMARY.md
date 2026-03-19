# INTEL3.md - Actionable Intelligence Summary

**Date:** 2026-03-19  
**Source:** Expert intelligence response to 16 questions  

---

## 🎯 NEW Actionable IOCs

### 1. Additional Solana Wallets

```rust
const KNOWN_C2_WALLETS: &[&str] = &[
    // Already have:
    "BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC",  // GlassWorm
    "28PKnu7RzizxBzFPoLp69HLXp9bJL3JFtT2s5QzHsEA2",  // Primary GlassWorm
    
    // NEW from INTEL3:
    "G2YxRa6wt1qePMwfJzdXZG62ej4qaTC7YURzuh2Lwd3t",  // ForceMemo funding
    "DSRUBTziADDHSik7WQvSMjvwCHFsbsThrbbjWMoJPUiW",  // Chrome RAT C2 ⭐ NEW
];
```

### 2. New C2 IPs

```rust
const KNOWN_C2_IPS: &[&str] = &[
    // Already have:
    "217.69.3.218", "199.247.10.166", "45.32.150.251",
    
    // NEW from INTEL3:
    "104.238.191.54",   // GlassWorm Native exfil ⭐ NEW
    "108.61.208.161",   // GlassWorm Native exfil ⭐ NEW
    "45.150.34.158",    // Chrome RAT seed exfil ⭐ NEW
];
```

### 3. Complete PhantomRaven Package List

**126 malicious npm packages identified:**

**Wave 1 (Aug 2025) - 21 packages:**
- `fq-ui`, `mocha-no-only`, `ft-flow`, `ul-inline`, `jest-hoist`
- `mourner`, `unused-imports`, `polyfill-corejs3`, `polyfill-regenerator`
- `@aio-commerce-sdk/*` (multiple packages)

**Wave 2 (Nov 2025-Feb 2026) - 50 packages:**
- `@gitlab-lsp/*` (multiple packages)
- `@gitlab-test/*` (multiple packages)
- `durablefunctionsmonitor*`, `chai-friendly`, `airbnb-*`

**Wave 3 (Feb 13-17, 2026) - 34 packages:**
- `eslint-comments`, `wdr-beam`, `lion-based-ui`, `crowdstrike`
- `@msdyn365-commerce-marketplace/*` (multiple packages)
- `react-important-stuff`, `audio-game`, `faltest`

**Wave 4 (Feb 18, 2026) - 4 packages:**
- `sort-class-members`, `sort-keys-fix`, `sort-keys-plus`
- `typescript-compat`, `typescript-sort-keys`

### 4. VSCode Extension Waves

**Wave 1 (Oct 2025) - 7 extensions:**
- `codejoy.codejoy-vscode-extension`
- `l-igh-t.vscode-theme-seti-folder`
- `ginfuru.better-nunjucks`
- `SIRILMP.dark-theme-sm`

**Wave 2 (Dec 2025) - 24 extensions:**
- `bphpburn.icons-vscode`
- `clangdcode.clangd-vscode`
- `msjsdreact.react-native-vscode`
- `saoudrizvsce.claude-dev`
- `vitalik.solidity`

**Wave 3 (Native Binary) - 18 extensions:**
- `bphpburnsus.iconesvscode`
- `iconkieftwo.icon-theme-materiall`
- `msjsdreact.react-native-vsce`
- `prettier-vsc.vsce-prettier`

**Wave 4 (macOS) - 3 extensions:**
- `studio-velte-distributor.pro-svelte-extension`
- `cudra-production.vsce-prettier-pro`
- `Puccin-development.full-access-catppuccin-pro-extension`

### 5. React Native Mobile Packages

| Package | Malicious Version | Clean Version | Downloads |
|---------|------------------|---------------|-----------|
| `react-native-country-select` | 0.3.91 | 0.3.9 | 9,072/wk |
| `react-native-international-phone-number` | 0.11.8 | 0.11.7 | 20,691/wk |

### 6. ForceMemo Python Markers

```python
# ForceMemo payload markers
FORCEMO_MARKERS = [
    "lzcdrtfxyqiplpd",  # Base64 payload blob
    "idzextbcjbgkdih",   # XOR key constant (134)
    "aqgqzxkfjzbdnhz",   # Base64 module alias
    "wogyjaaijwqbpxe",   # Zlib module alias
]

# Hex signatures for binary scanning
HEX_SIGNATURES = [
    b'\x6c\x7a\x63\x64\x72\x74\x66\x78\x79\x71\x69\x70\x6c\x70\x64',  # lzcdrtfxyqiplpd
    b'\x69\x64\x7a\x65\x78\x74\x62\x63\x6a\x62\x67\x6b\x64\x69\x68',  # idzextbcjbgkdih
    b'\x61\x71\x67\x71\x7a\x78\x6b\x66\x6a\x7a\x62\x64\x6e\x68\x7a',  # aqgqzxkfjzbdnhz
]
```

### 7. RDD package.json Pattern

```json
{
  "dependencies": {
    "ui-styles-pkg": "http://packages.storeartifact.com/npm/unused-imports"
  },
  "devDependencies": {
    "ui-styles-pkg": "http://packages.storeartifact.com/npm/unused-imports"
  },
  "author": "JPD"
}
```

**Detection:** Look for URL dependencies + "JPD" author

---

## ✅ Implementation Priority

### Immediate (Today)

1. ✅ **Add new Solana wallets** to blockchain_c2_detector
2. ✅ **Add new C2 IPs** to detector
3. ✅ **Scan PhantomRaven packages** from the 126 package list
4. ✅ **Scan VSCode extension waves** from the lists above

### Short-term (This Week)

1. **Add RDD detector** - Detect URL dependencies in package.json
2. **Add ForceMemo Python detector** - Search for markers in Python files
3. **Add "JPD" author detector** - Flag packages with "JPD" author
4. **Expand VSCode scanning** - Scan all identified malicious extensions

---

## 📊 Scan Priority List

### High Priority (Scan Immediately)

**PhantomRaven packages still live:**
```
Wave 3 (34 packages, Feb 13-17):
- eslint-comments
- wdr-beam
- lion-based-ui
- crowdstrike
- @msdyn365-commerce-marketplace/*
- react-important-stuff

Wave 4 (4 packages, Feb 18):
- sort-class-members
- sort-keys-fix
- sort-keys-plus
- typescript-compat
- typescript-sort-keys
```

**VSCode Extension Waves:**
```
Wave 3 (18 extensions):
- bphpburnsus/iconesvscode
- iconkieftwo/icon-theme-materiall
- msjsdreact/react-native-vsce
- prettier-vsc/vsce-prettier

Wave 4 (3 extensions):
- studio-velte-distributor/pro-svelte-extension
- cudra-production/vsce-prettier-pro
- Puccin-development/full-access-catppuccin-pro-extension
```

---

## 🎯 Recommended Next Actions

1. **Scan PhantomRaven Wave 3 & 4 packages** (38 packages)
2. **Scan VSCode Wave 3 & 4 extensions** (21 packages)
3. **Add RDD detector** for URL dependencies
4. **Add "JPD" author detector**

---

**Status:** Ready to implement  
**Next:** Scan high-priority packages from intel
