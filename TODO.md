# glassware - TODO & Improvements

**Last Updated:** 2026-03-18 (Post FP Fix)  
**Priority:** High → Low

---

## ✅ COMPLETED (Today)

### 1. README/Markdown False Positive Fix ✅
**Issue:** Emoji in README files trigger variation selector detections  
**Fixed:** Skip `.md`, `.mdx`, `.txt`, `.rst` files for invisible char and homoglyph detection  
**Result:** `@upstash/context7-mcp` 7 findings → 0 findings  
**Files Modified:** `glassware-core/src/scanner.rs`

---

## 🔴 CRITICAL (Next Session)

**Issue:** Emoji in README files trigger variation selector detections  
**Example:** `@upstash/context7-mcp` - 7 findings, all emoji in README  
**Impact:** Wastes analyst time, reduces trust in detections

**Fix Options:**
1. Skip markdown files entirely (`.md`, `.mdx`)
2. Add emoji context detection for markdown (like we did for code)
3. Lower severity for markdown findings

**Recommended:** Option 1 - skip markdown files for invisible character detection

**Files to modify:**
- `glassware-core/src/scanner.rs` - Add markdown extension skip
- OR `glassware-core/src/detectors/invisible.rs` - Add markdown context check

---

### 2. Homoglyph False Positives in Branding

**Issue:** Legitimate branding uses confusables (e.g., "Сontext7" with Cyrillic С)  
**Example:** `@upstash/context7-mcp` README  
**Impact:** False accusations against legitimate projects

**Fix:**
- Add allowlist for known brands
- Lower severity in documentation files
- Require multiple homoglyphs in same identifier

---

## 🟡 HIGH (Next Sprint)

### 3. Add More Encryption Detectors

**Issue:** Attackers may use alternative ciphers to evade detection  
**Priority Ciphers:**

| Cipher | Priority | Why |
|--------|----------|-----|
| ChaCha20 | HIGH | Modern, fast, used in TLS |
| Salsa20 | HIGH | Predecessor to ChaCha |
| XOR (rolling) | MEDIUM | Simple, common in malware |
| Twofish | LOW | AES alternative |
| Serpent | LOW | AES finalist |
| PRESENT | LOW | Lightweight IoT cipher |
| Speck/Simon | LOW | NSA ciphers |
| HC-128 | LOW | eSTREAM finalist |
| Custom S-Box RC4 | MEDIUM | Variant evasion |

**Implementation:**
- Add to `glassware-core/src/` as new detectors
- Register in `scanner.rs`
- Add to `DetectionCategory` enum

---

### 4. Config File Support

**Issue:** Hardcoded thresholds (download limit, days back, etc.)  
**Solution:** `scan-config.json`

```json
{
  "scan": {
    "download_threshold": 1000,
    "days_back": 365,
    "max_packages": 100,
    "severity_threshold": "info"
  },
  "paths": {
    "evidence_dir": "harness/data/evidence",
    "reports_dir": "harness/reports"
  },
  "exclude": {
    "scopes": ["@iflow-mcp", "@aifabrix"],
    "extensions": [".md", ".mdx", ".txt"],
    "directories": ["node_modules", ".git", "test"]
  },
  "include": {
    "scopes": ["@modelcontextprotocol", "@anthropic-ai"]
  }
}
```

---

## 🟢 MEDIUM (Future)

### 5. Better Large Payload Summarization

**Issue:** 9,000+ finding packages overwhelm reports  
**Example:** `@midscene/mcp` (3,288 findings), `watercrawl-mcp` (9,133)

**Fix:**
- Group by category
- Show top 10 most severe
- Provide payload decode summary
- Add `--summary` flag

---

### 6. LLM Auto-Analysis

**Issue:** Manual `--llm` flag required  
**Solution:** Auto-trigger LLM for high-severity finds

```rust
if findings.critical_count > 10 {
    run_llm_analysis();
}
```

---

### 7. Better Error Handling in Harness

**Issue:** npm pack failures crash batch scans  
**Fix:** Retry logic, better error messages

---

## 📝 DOCUMENTATION TODO

- [ ] Fix naming confusion (GlassWare attack vs glassware tool vs glassworks repo)
- [ ] Add false positive examples to docs
- [ ] Document encryption detection capabilities
- [ ] Create "Anatomy of a Detection" guide
- [ ] Update README with latest findings

---

**Created:** 2026-03-18  
**Next Review:** After Phase 2 complete
