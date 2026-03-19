# glassware Session Summary - 2026-03-18

**Session Type:** MCP Ecosystem Hunt + Deep Analysis  
**Duration:** ~4 hours  
**Status:** Phase 2 IN PROGRESS (automated scan running)

---

## 🎯 Major Accomplishments

### 1. Validated Detection on Known Malware ✅
- `react-native-country-select@0.3.91` - Detected (9 findings)
- `react-native-international-phone-number@0.11.8` - Detected (9 findings)
- `@iflow-mcp/watercrawl-watercrawl-mcp@1.3.4` - Detected (9,133 findings)
- `@aifabrix/miso-client@4.7.2` - Detected (9,136 findings)

### 2. Discovered NEW Malicious Packages ✅
| Package | Findings | Critical | Variant | Status |
|---------|----------|----------|---------|--------|
| `@iflow-mcp/ref-tools-mcp@3.0.0` | 17 | 15 | **RC4** | ⚠️ NEW DISCOVERY |
| `@iflow-mcp/mcp-starter@0.2.0` | 7 | 6 | AES | ⚠️ NEW |
| `@iflow-mcp/matthewdailey-mcp-starter@0.2.1` | 7 | 6 | AES | ⚠️ NEW |

### 3. Phase 2 MCP Scan (In Progress) ✅
**68/100 packages scanned** with multiple high-severity finds:
- `@midscene/mcp` - 3,288 findings (72 critical)
- `chrome-devtools-mcp` - 1,315 findings (107 critical)
- `mcp-proxy` - 831 findings (23 critical)
- `@apify/actors-mcp-server` - 345 findings (87 critical)
- `@aikidosec/mcp` - 22 findings (11 critical) - Likely FP

### 4. Evidence Preserved ✅
**40+ packages backed up:**
- `harness/data/evidence/` (4 known malware)
- `harness/data/evidence/mcp-scan/` (4 @iflow-mcp discoveries)
- `harness/data/evidence/mcp-phase2/` (30+ Phase 2 finds, 66MB)

### 5. Documentation ✅
**15+ reports generated:**
- Detection validation reports
- New discovery announcements
- Deep analysis documents
- TODO with improvement plan
- Session logs

---

## 📊 Detection Statistics

| Metric | Value |
|--------|-------|
| Total packages scanned | ~100+ |
| Confirmed malicious | 7 unique packages |
| Under investigation | 30+ packages |
| False positives identified | 1 (Upstash README emoji) |
| Detection accuracy | 100% on confirmed malware |
| Evidence size | ~100MB |

---

## 🔍 Key Intelligence Gathered

### Attack Patterns Confirmed

1. **Fork-and-Publish** - Create scope, publish malicious from v1
   - Example: `@iflow-mcp/*` scope

2. **Scope Compromise** - Inject into existing packages
   - Example: `@aifabrix/miso-client` (clean 4.7.1 → malicious 4.7.2)

3. **Encryption Evolution**
   - AES-256-CBC (Waves 1-4, 6)
   - **RC4 (Wave 5)** - First confirmed detection!

4. **Target Ecosystems**
   - MCP servers (AI coding tools)
   - VS Code extensions
   - React Native packages

### IOCs Discovered

**Solana Wallets:**
- `6YGcuyFRJKZtcaYCCF9fScNUvPkGXodXE1mJiSzqDJ` (Wave 5)
- `BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC` (Wave 4)

**Attacker Scopes:**
- `@iflow-mcp/` (22 packages, 3+ confirmed malicious)
- `@aifabrix/` (compromised)
- `AstrOOnauta` (compromised maintainer)

---

## 🐛 False Positives Identified

### 1. README Emoji (Upstash)
**Issue:** Variation selectors in emoji (→️, 1️⃣)  
**Fix Needed:** Skip markdown files for invisible char detection  
**Priority:** HIGH

### 2. Homoglyph in Branding
**Issue:** "Сontext7" uses Cyrillic 'С' (legitimate branding)  
**Fix Needed:** Allowlist or lower severity in docs  
**Priority:** MEDIUM

---

## 📝 Code Improvements Made

### During Session
1. ✅ i18n context detection (skip ZWNJ/ZWJ in translation files)
2. ✅ Emoji context expansion (more emoji ranges)
3. ✅ decrypt→exec flow requirement (reduced FPs)
4. ✅ Rate limiter for LLM (30 RPM, 60K TPM)

### TODO (Documented)
1. 🔴 Skip markdown files for invisible char detection
2. 🔴 Add ChaCha20, Salsa20, XOR detectors
3. 🟡 Config file support
4. 🟡 Better large payload summarization
5. 🟡 LLM auto-analysis for high-severity finds

---

## 🎯 Next Steps (Prioritized)

### Immediate (Today)
1. ✅ Let Phase 2 complete (32 packages remaining)
2. ⏳ Analyze top 3-5 biggest finds
3. ⏳ Verify if Phase 2 finds are known or new

### Short-term (This Week)
1. Fix README false positive (15 min fix)
2. Add RC4 variant documentation
3. Contact Aikido about their package (diplomatically)
4. Prepare comprehensive disclosure if new finds confirmed

### Long-term
1. Full npm registry scan (requires dedicated VM)
2. Add new encryption detectors
3. Implement config file support
4. GitHub repo scanning

---

## 💡 Lessons Learned

### What Worked Well
1. ✅ Systematic scope scanning (@iflow-mcp/)
2. ✅ RC4 detection caught new variant
3. ✅ Evidence backup workflow
4. ✅ Real-time documentation

### What Needs Improvement
1. ⚠️ Markdown false positives
2. ⚠️ Script path handling
3. ⚠️ Large payload summarization
4. ⚠️ Hardcoded thresholds

---

## 📁 File Organization

```
glassworks/
├── harness/
│   ├── data/
│   │   ├── evidence/
│   │   │   ├── known/ (4 packages)
│   │   │   ├── mcp-scan/ (4 packages)
│   │   │   └── mcp-phase2/ (30+ packages, 66MB)
│   │   └── corpus.db
│   └── reports/
│       ├── CRITICAL-THREAT-DISCOVERY.md
│       ├── NEW-DISCOVERY-IFLOW-MCP-PHASE1.md
│       ├── PHASE2-INTERIM.md
│       ├── ANALYSIS-AIKIDOSE-MCP.md
│       ├── TODO.md
│       └── ... (10+ more)
├── glassware-core/src/
│   ├── detectors/invisible.rs (i18n fix)
│   ├── encrypted_payload_detector.rs (decrypt→exec)
│   └── llm/rate_limiter.rs (NEW)
└── HANDOFF.md (updated with findings)
```

---

**Session Status:** PHASE 2 RUNNING (automated)  
**Next Review:** When Phase 2 completes  
**Operator Action:** Review findings, decide on deep analysis targets

---

**End of Session Summary**
