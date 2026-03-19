# False Positive Alert: prettier@3.8.1

**Date:** 2026-03-19  
**Package:** `prettier@3.8.1` (LEGITIMATE)  
**Initial Findings:** 28 (22 critical)  
**Verdict:** ✅ **FALSE POSITIVE - Legitimate Package**  

---

## Initial Detection

The new behavioral detectors flagged prettier with:
- `glassware_pattern`: 25 findings
- `time_delay_sandbox_evasion`: 3 findings

**Initial concern:** Could this be a typosquatting attack on prettier?

---

## Detailed Analysis

### Package Verification

| Check | Expected | Found | Match |
|-------|----------|-------|-------|
| **Author** | James Long | James Long | ✅ |
| **Repository** | prettier/prettier | prettier/prettier | ✅ |
| **Version** | 3.8.1 | 3.8.1 | ✅ |
| **npm view matches** | Yes | Yes | ✅ |

**Verdict:** This is the **LEGITIMATE prettier package**, not a typosquat.

---

### False Positive Patterns

#### 1. Time Delay Sandbox Evasion (3 findings)

**Triggering code in `internal/legacy-cli.mjs`:**
```javascript
exports.isCI = !!(env3.CI !== "false" &&
  (env3.BUILD_ID ||
  env3.BUILD_NUMBER ||
  env3.CI ||
  env3.CI_APP_ID ||
  env3.CI_BUILD_ID ||
  env3.CI_BUILD_NUMBER ||
  ...
```

**Actual purpose:** CI environment detection for prettier CLI  
**Why flagged:** Pattern matches `env3.CI` check followed by conditional logic  
**Verdict:** ✅ **LEGITIMATE** - Standard CI detection for formatter behavior

---

#### 2. GlassWare Decoder Pattern (25 findings)

**Triggering code in `plugins/yaml.js`:**
```javascript
parseCharCode(e,n,r){
  let{src:s}=this.context,i=s.substr(e,n),a=i.length===n&&/^[0-9a-fA-F]+$/.test(i)?
  parseInt(i,16):NaN;
  return isNaN(a)?...:String.fromCodePoint(a)
}
```

**Actual purpose:** YAML escape sequence parser (converting `\xHH` to characters)  
**Why flagged:** Pattern resembles decoder function (hex parsing + character conversion)  
**Verdict:** ✅ **LEGITIMATE** - Standard YAML parser escape sequence handling

---

### Why This is a False Positive

**Root causes:**

1. **CI detection is common** - Many legitimate packages check for CI environments
   - prettier: For formatting behavior
   - Test runners: For output formatting
   - Build tools: For optimization

2. **Hex parsing is common** - Many legitimate packages parse hex codes
   - YAML parsers: Escape sequences
   - CSS parsers: Color codes
   - String utilities: Unicode handling

3. **Size heuristic didn't apply** - Individual parser files are <100KB
   - Total package: 1.3MB (would be skipped)
   - Individual files: 50-600KB (not skipped)

---

## Lessons Learned

### What Worked

1. ✅ **Human review caught FP** - Verified package legitimacy
2. ✅ **Multiple signals required** - Single pattern didn't auto-flag
3. ✅ **Risk scoring worked** - High findings but requires review

### What Needs Improvement

1. ⚠️ **CI detection pattern too broad** - Catches legitimate CI checks
   - **Fix:** Require additional malicious context (eval, atob, etc.)

2. ⚠️ **Hex parser pattern too broad** - Catches escape sequence handlers
   - **Fix:** Add context analysis (is it in a parser? string utility?)

3. ⚠️ **File-level vs package-level size** - Large packages with small files
   - **Fix:** Consider package-level size heuristic too

---

## Recommendations

### Immediate

1. ✅ **Whitelist prettier** - Add to known legitimate packages
2. ✅ **No action needed** - Package is safe

### Short-term

1. **Tune CI detector:**
   - Require additional malicious patterns
   - Skip if only checking `env.CI` without eval/exec

2. **Tune decoder detector:**
   - Add parser context (YAML, CSS, JSON parsers)
   - Skip if in file named `*parser*`, `*lexer*`, `*tokenizer*`

3. **Add package-level size heuristic:**
   - Skip homoglyph/bidi for packages >500KB total
   - Individual file check + total package check

### Long-term

1. **Implement allowlist system** - Known legitimate packages skip behavioral checks
2. **Add parser detection** - Identify YAML/CSS/JSON parsers, skip decoder patterns
3. **Contextual analysis** - Is the "decoder" actually parsing user input?

---

## Impact Assessment

**If we hadn't caught this FP:**
- Would have falsely accused prettier (50M+ weekly downloads)
- Massive false alarm
- Loss of trust in detection system

**Why this is GOOD:**
- System caught something unusual
- Human review confirmed legitimate
- Now we can tune detectors to avoid similar FPs

---

## Conclusion

**prettier@3.8.1 is 100% LEGITIMATE.**

The behavioral detectors are working but need tuning:
- CI detection is too aggressive
- Decoder patterns catch legitimate parsers
- Need package-level context, not just file-level

**Risk Level:** NONE (legitimate package)  
**Action:** Tune detectors, add to allowlist  
**Learning:** Valuable FP for improving detection accuracy

---

**Analyst:** Automated + Human Review  
**Timestamp:** 2026-03-19 15:00 UTC  
**Status:** ✅ FALSE POSITIVE CONFIRMED
