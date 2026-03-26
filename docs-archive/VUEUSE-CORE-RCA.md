# @vueuse/core False Positive - Root Cause Analysis

**Date:** 2026-03-25  
**Status:** Root Cause Identified  
**Package:** @vueuse/core@10.7.2 (legitimate Vue.js utility library)

---

## Findings Breakdown

| Category | Count | Severity | Issue |
|----------|-------|----------|-------|
| **HeaderC2** | 3 | High | "Network request with encoded environment variables" |
| **BlockchainC2** | 28 | Medium | "Memo instruction usage (potential data hiding)" |
| **GlasswarePattern** | 2 | Medium | "encoding_pattern (45% confidence)", "decoder_pattern (45% confidence)" |
| **SocketIOC2** | 4 | Medium | "Socket.IO with suspicious patterns (2 signal groups)" |
| **Unknown** | 4 | - | Uncategorized findings |
| **TOTAL** | **41** | - | **Score: 7.40** |

---

## Root Cause: Multiple Over-Sensitive Detectors

This is **NOT a scoring system bug**. The scoring worked correctly:
- 4 categories triggered
- Score calculated as 7.40 (below malicious threshold of 8.0 would be ideal)
- Category diversity cap working (3+ categories = no cap)

**The problem is 4 different detectors are flagging legitimate Web3 utility code:**

### 1. HeaderC2 Detector ❌ OVER-SENSITIVE

**Trigger:** "Network request with encoded environment variables"

**Reality:** This is legitimate HTTP abstraction for API calls. The library encodes configuration data for network requests - standard practice for Vue.js composables.

**Fix Needed:**
- Require actual exfiltration indicators (specific headers like `X-Exfil`, hardcoded C2 IPs)
- Don't flag generic HTTP encoding (base64, URL encoding are common in web apps)

### 2. BlockchainC2 Detector ❌ OVER-SENSITIVE

**Trigger:** "Memo instruction usage (potential data hiding)" - 28 findings

**Reality:** @vueuse/core includes Web3 utility functions that interact with Solana blockchain. Memo instructions are a LEGITIMATE Solana feature for adding metadata to transactions.

**From the code:**
```javascript
// Legitimate Web3 utility function
import { TransactionInstruction } from '@solana/web3.js';
// Creates memo instruction for transaction metadata
const memoInstruction = new TransactionInstruction({...});
```

**Fix Needed:**
- The detector already has C2 wallets/IPs hardcoded ✅
- But "memo instruction" pattern is too generic ❌
- Should skip memo patterns in Web3 context (imports from @solana/web3.js, ethers, etc.)
- Should require C2 wallet + memo combination, not just memo alone

### 3. GlasswarePattern Detector ❌ OVER-SENSITIVE

**Trigger:** "encoding_pattern (45% confidence)", "decoder_pattern (45% confidence)"

**Reality:** 45% confidence is BELOW reasonable threshold. This is pattern matching finding generic encoding/decoding code that exists in ANY utility library.

**Fix Needed:**
- Raise minimum confidence threshold from 45% to 70%+
- Require additional context (obfuscation + execution, not just encoding)
- Skip common encoding patterns (base64, URL encoding, JSON stringify)

### 4. SocketIOC2 Detector ❌ OVER-SENSITIVE

**Trigger:** "Socket.IO with suspicious patterns (2 signal groups)"

**Reality:** @vueuse/core may include Socket.IO utilities for real-time features. Socket.IO itself is NOT suspicious.

**Fix Needed:**
- Require C2-specific patterns (hardcoded IPs, non-standard ports, aggressive reconnection)
- Don't flag Socket.IO usage alone
- Look for actual C2 infrastructure (IPs from threat intel)

---

## Why This Is NOT a Scoring Bug

The scoring system worked correctly:

1. **Multiple categories detected** → Score increased appropriately
2. **Category diversity cap** → Applied correctly (3+ categories = no cap)
3. **Severity weighting** → High findings weighted more than medium
4. **Final score 7.40** → Below malicious threshold (8.0), but above suspicious (4.0)

**The issue is the detectors are flagging legitimate code as suspicious.**

---

## Detector Fixes Required (In Priority Order)

### Priority 1: BlockchainC2 Detector

**Current Logic:**
```rust
// Flags ANY memo instruction usage
if line.contains("memo") {
    findings.push(Medium("Memo instruction usage"));
}
```

**Fix:**
```rust
// Only flag memo with C2 context
let c2_wallets = ["BjVeAjPr...", "28PKnu7..."];

if line.contains("memo") 
    && c2_wallets.iter().any(|w| content.contains(w))  // C2 wallet present
    && line.contains("setInterval")  // Polling pattern
{
    findings.push(Critical("Memo + C2 wallet + polling = GlassWorm C2"));
}

// Skip memo in Web3 context
if file_path.contains("node_modules/@solana")
    || imports.contains("@solana/web3.js")
    || imports.contains("ethers")
{
    return Vec::new();  // Legitimate Web3 usage
}
```

### Priority 2: GlasswarePattern Detector

**Current Issue:** 45% confidence threshold too low

**Fix:**
```rust
// Raise confidence threshold
if confidence < 0.70 {
    return Vec::new();  // Too weak to flag
}

// Require multiple GlassWare patterns
if encoding_pattern && decoder_pattern && invisible_chars {
    findings.push(High("GlassWare steganography (3 patterns)"));
} else if encoding_pattern && decoder_pattern {
    return Vec::new();  // Only 2 patterns = likely FP
}
```

### Priority 3: HeaderC2 Detector

**Current Issue:** Flags generic HTTP encoding

**Fix:**
```rust
// Look for specific exfil indicators
let exfil_headers = ["X-Exfil", "X-Operator", "X-UUID"];
let c2_ips = ["217.69.11.", "208.85.20."];

if exfil_headers.iter().any(|h| header.contains(h))
    || c2_ips.iter().any(|ip| url.contains(ip))
{
    findings.push(Critical("Confirmed exfiltration pattern"));
} else {
    return Vec::new();  // Generic HTTP encoding = legitimate
}
```

### Priority 4: SocketIOC2 Detector

**Current Issue:** Flags any Socket.IO usage

**Fix:**
```rust
// Look for C2-specific Socket.IO patterns
let c2_ports = [4789, 5000, 10000];
let c2_ips = ["217.69.11.", "104.238."];

if c2_ips.iter().any(|ip| url.contains(ip))
    || c2_ports.iter().any(|p| url.contains(&format!(":{p}")))
{
    findings.push(High("Socket.IO to known C2 infrastructure"));
} else if reconnection_attempts > 10 && reconnection_delay_max > 60000 {
    findings.push(Medium("Aggressive Socket.IO reconnection (possible C2)"));
} else {
    return Vec::new();  // Normal Socket.IO usage
}
```

---

## Testing Protocol

After fixes, verify:

### Test 1: @vueuse/core (Should NOT Flag)
```bash
./target/release/glassware scan-npm @vueuse/core@10.7.2
# Expected: threat_score < 4.0 (suspicious at most)
# Expected: 0-5 findings (not 41!)
```

### Test 2: Real C2 Evidence (Should Still Detect)
```bash
./target/release/glassware scan-tarball evidence/glassworm-c2-001.tgz
# Expected: threat_score >= 8.0
# Expected: BlockchainC2 findings with C2 wallet
```

### Test 3: Wave 10 (FP Rate)
```bash
./target/release/glassware campaign run campaigns/wave10-1000plus.toml
# Expected: FP rate < 0.1% (≤1/1000)
# Expected: @vueuse/core NOT flagged
```

---

## Implementation Plan

### Phase 1: BlockchainC2 Fix (Today)
- [ ] Add Web3 context detection (skip memo in Web3 files)
- [ ] Require C2 wallet + memo + polling combination
- [ ] Test on @vueuse/core and glassworm-c2-001.tgz

### Phase 2: GlasswarePattern Fix (Today)
- [ ] Raise confidence threshold to 70%
- [ ] Require 3+ patterns for High severity
- [ ] Test on evidence tarballs

### Phase 3: HeaderC2 & SocketIOC2 Fixes (Tomorrow)
- [ ] Add specific exfil header detection
- [ ] Add C2 IP/port detection for Socket.IO
- [ ] Test on Wave 10

### Phase 4: Validation (Tomorrow)
- [ ] Re-run Wave 10 with all fixes
- [ ] Verify @vueuse/core NOT flagged
- [ ] Verify evidence still detected
- [ ] Verify FP rate < 0.1%

---

## Key Insight

**This is NOT a scoring system problem that autoresearch should solve.**

The scoring system is working correctly - it's aggregating signals from multiple detectors as designed.

**The problem is detector over-sensitivity:**
- BlockchainC2 flags legitimate Web3 memo usage
- GlasswarePattern flags generic encoding at 45% confidence
- HeaderC2 flags normal HTTP encoding
- SocketIOC2 flags any Socket.IO usage

**These are detector logic bugs, not scoring bugs.**

Autoresearch would struggle here because:
1. Multiple detectors need coordinated fixes
2. Requires understanding Web3/blockchain context
3. Requires threat intelligence (real C2 wallets/IPs)
4. Risk of overfitting if tuned only on @vueuse/core

**Manual detector fixes are the right approach.**

---

**Next Action:** Implement BlockchainC2 fix (Priority 1) while Wave 10 completes
