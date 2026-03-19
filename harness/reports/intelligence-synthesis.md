# GlassWare Intelligence Synthesis

**Date:** 2026-03-18  
**Sources:** 
- Aikido Security (Mar 16, 2026)
- Codeberg researcher (tip-o-deincognito)
- MalwareBazaar IOCs

---

## Critical Finding: We're Looking in the Wrong Place

### The Real Target Profile

From the Aikido reports, the **actual** GlassWare attacks targeted:

| Package | Type | Downloads/Week | Attack Vector |
|---------|------|----------------|---------------|
| `react-native-country-select` | React Native | 29,763 | `preinstall.js` |
| `react-native-international-phone-number` | React Native | 29,763 | `preinstall.js` |
| `@aifabrix/miso-client` | AI/ML | Unknown | Unicode stego |
| `quartz.quartz-markdown-editor` | VS Code Extension | Unknown | GitHub injection |

**Key Insight:** These are **legitimate, popular packages** that were **compromised via maintainer credentials**, not newly published malicious packages.

---

## Attack Patterns Confirmed

### Pattern 1: Compromised Maintainer Accounts
```
✓ Attacker gains maintainer access (stolen credentials)
✓ Publishes malicious version (0.3.91 vs 0.3.9)
✓ Same package name, different version
✓ Minimal diff (2 files changed)
```

### Pattern 2: Install Script Injection
```javascript
// package.json
"scripts": {
  "preinstall": "node install.js"  // ← NEW FILE
}
```

### Pattern 3: Obfuscated Decoder
```javascript
// Obfuscation markers (from Aikido)
0x45b, 'nSeb', -0x109, -0xd4  // Hex string rotation
codePointAt(0)                // Unicode extraction
0xFE00, 0xFE0F                // Variation Selector ranges
eval(atob(payload))           // Dynamic execution
```

### Pattern 4: Multi-Stage C2
```
Stage 1: Solana RPC → getSignaturesForAddress
Stage 2: Base64 URL → AES-256-CBC decrypt
Stage 3: Google Calendar indirection → Final payload
```

### Pattern 5: Geographic Exclusion
```javascript
// Russian locale check (common in Russian-speaking criminal malware)
if (/ru_RU|ru-RU|Russian|russian/i.test(locale)) {
  return; // Don't infect Russian systems
}
```

---

## Our Detection Coverage

| Pattern | Our Detector | Status |
|---------|-------------|--------|
| Variation Selectors (U+FE00-U+FE0F) | GW001, GW003 | ✅ Detected |
| `codePointAt` + hex constants | GW002 | ✅ Detected |
| `eval(atob())` flow | GW005 | ✅ Detected (with decrypt) |
| AES-256-CBC decrypt | GW006 | ✅ Detected |
| Solana RPC calls | ❌ None | ⚠️ Gap |
| Russian locale check | ❌ None | ℹ️ Not malicious itself |
| Obfuscation patterns | ❌ None | ⚠️ Gap |
| Chrome extension sideloader | ❌ None | ⚠️ Gap |

---

## Why We Haven't Found Anything (Yet)

### 1. Wrong Search Strategy

**Current approach:** Scan NEW packages with install scripts  
**Reality:** Attacks target EXISTING popular packages via compromise

**Current approach:** Look for suspicious package names  
**Reality:** Package names are legitimate (`react-native-country-select`)

**Current approach:** Filter by low downloads (<1000/wk)  
**Reality:** Targets have HIGH downloads (29,763/wk)

### 2. The Real Attack Surface

```
┌─────────────────────────────────────────────────────────────┐
│  npm Registry: 3+ million packages                          │
│  ↓                                                          │
│  Popular packages with install scripts: ~50,000             │
│  ↓                                                          │
│  Maintainer accounts with stolen credentials: UNKNOWN       │
│  ↓                                                          │
│  Compromised packages in the wild: 151+ (confirmed)         │
│  ↓                                                          │
│  Our scan coverage: 12 packages (0.008%)                    │
└─────────────────────────────────────────────────────────────┘
```

---

## Revised Strategy

### Immediate Actions (Today)

#### 1. Scan Known Compromised Packages
```bash
# Download and scan the ACTUAL malicious packages
npm pack react-native-country-select@0.3.91
npm pack react-native-international-phone-number@0.11.8
./target/release/glassware --llm package/
```

#### 2. Add Missing Detectors
- [ ] Solana wallet/address detector
- [ ] Obfuscation pattern detector (hex rotation)
- [ ] Russian locale check detector (for context)

#### 3. Change Selection Criteria
```python
# Remove download threshold (targets are popular)
# Remove "suspicious name" requirement
# Add: packages with RECENT version updates
# Add: packages with install scripts CHANGED in last 30 days
```

### Short-term (This Week)

#### 1. GitHub Repo Monitoring
```bash
# Scan repos for injection patterns (like researcher did)
# Target: VS Code extensions, Cursor extensions, React Native packages
# Pattern: codePointAt + 0xFE00 in preinstall.js
```

#### 2. Version Diff Analysis
```bash
# For each package:
# 1. Get version N (clean)
# 2. Get version N+1 (potentially compromised)
# 3. Diff the two versions
# 4. Flag if: new install script, new obfuscated file
```

#### 3. Maintainer Account Monitoring
```bash
# Track packages by known compromised maintainers
# AstrOOnauta (confirmed)
# oorzc (confirmed)
# Monitor for new publications
```

---

## Evidence We Need to Collect

### For Disclosure
1. ✅ Package name and version
2. ✅ SHA-256 hash of malicious files
3. ✅ Decoded payload (if stego)
4. ✅ C2 infrastructure (URLs, wallets, IPs)
5. ✅ Timeline (published, discovered, reported)
6. ✅ Download counts (impact assessment)

### For Detection Improvement
1. ✅ Obfuscation patterns
2. ✅ New decoder variants
3. ✅ C2 rotation patterns
4. ✅ Geographic exclusion logic
5. ✅ Persistence mechanisms

---

## Revised Success Metrics

| Metric | Old Target | New Target |
|--------|-----------|------------|
| Packages scanned | 500 | 50 (high-value) |
| Detection rate | >1% | Scan known-compromised first |
| False positives | <5% | <1% (quality over quantity) |
| Time per package | <5s | <30s (with deep analysis) |

---

## Next Scan Plan

### Step 1: Validate Detection (Priority: CRITICAL)
```bash
# Download known malicious packages
cd /tmp
npm pack react-native-country-select@0.3.91
npm pack react-native-international-phone-number@0.11.8
tar -xzf react-native-country-select-0.3.91.tgz
tar -xzf react-native-international-phone-number-0.11.8.tgz

# Scan with glassware
cd /home/property.sightlines/samgrowls/glassworks
./target/release/glassware --format json --severity info /tmp/package/
```

**Expected:** Should detect decoder patterns, eval flows, possibly Solana C2

### Step 2: Add Missing Detectors
- Solana wallet regex
- Obfuscation pattern detector
- Chrome extension manifest analyzer

### Step 3: Targeted High-Value Scan
- VS Code extensions (100 packages)
- React Native packages (100 packages)
- Recently updated devtools (100 packages)

---

## Key Takeaway

**We've been scanning the wrong packages.**

The GlassWare operators aren't publishing new malicious packages. They're **compromising existing popular packages** via stolen maintainer credentials.

**Our Tier 1 filter (low downloads, new packages) is filtering out the actual targets.**

We need to:
1. ✅ Validate detection on known-compromised packages
2. ✅ Scan popular packages with recent updates
3. ✅ Monitor maintainer accounts, not package names
4. ✅ Add detectors for Solana C2 and obfuscation

---

**Prepared by:** glassware team  
**Action:** Immediate pivot to high-value target scanning
