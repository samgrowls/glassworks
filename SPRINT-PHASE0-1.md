# Phase 0 & 1 Implementation Sprint

**Sprint:** v0.9.0.0  
**Duration:** Mar 24 - Apr 4 (2 weeks)  
**Developer:** Primary agent  
**Status:** 🔄 In Progress

---

## Phase 0: Trivial Wins (1-2 days)

### E1: Update IoC Lists ✅ COMPLETE

**File:** `glassware-core/src/blockchain_c2_detector.rs`

**Task List:**
- [x] Add Chrome RAT wallet `DSRUBTz...` to `HARDCODED_WALLETS`
- [x] Add INTEL3 IPs: `104.238.191.54`, `108.61.208.161`, `45.150.34.158`
- [x] Add test cases for new IoCs
- [x] Verify detectors flag these values

**Acceptance Criteria:**
- ✅ New wallets/IPs detected by existing detectors
- ✅ No false positives on legitimate values
- ✅ Tests passing (2 new tests added)

**Estimated:** 0.5 days
**Actual:** 0.5 days

---

### E3: Browser-Kill Patterns ✅ COMPLETE

**File:** `glassware-core/src/detectors/browser_kill.rs` (NEW)

**Task List:**
- [x] Create `BrowserKillDetector` struct
- [x] Implement `Detector` trait
- [x] Add `BROWSER_KILL_PATTERNS` constant array
- [x] Create test fixtures with browser-kill commands
- [x] Test against Part 5 examples
- [x] Register detector in engine

**Patterns Added:**
```rust
const BROWSER_KILL_PATTERNS: &[&str] = &[
    // Windows taskkill
    "taskkill /F /IM chrome.exe",
    "taskkill /F /IM msedge.exe",
    "taskkill /F /IM brave.exe",
    // Unix pkill
    "pkill -9 -f \"Google Chrome\"",
    "pkill -9 chrome",
    // PowerShell
    "Stop-Process -Name chrome -Force",
];
```

**Acceptance Criteria:**
- ✅ All Part 5 browser-kill patterns detected
- ✅ No false positives on legitimate process management
- ✅ Tests passing (6 tests added)

**Estimated:** 1 day
**Actual:** 1 day

---

## Phase 1: JS-Level Detector Additions (3-5 days)

### G3: Typo Attribution Strings ✅

**File:** `glassware-core/src/detectors/attribution.rs` (NEW)

**Task List:**
- [ ] Create `AttributionDetector` struct
- [ ] Implement `Detector` trait
- [ ] Add typo fingerprint constants (verify from Part 5)
- [ ] Register detector in engine
- [ ] Create test fixtures
- [ ] Add to `DetectionCategory` enum

**Typo Fingerprints (from Part 5):**
```rust
const TYPO_FINGERPRINTS: &[(&str, &str)] = &[
    ("Invlaid", "GlassWorm data.dll typo fingerprint"),
    ("LoadLibararyFail", "GlassWorm memexec crate typo"),
    ("NtAllocVmErr", "GlassWorm APC injection typo"),
    ("Corpartion", "GlassWorm led-win32 typo"),
    ("ErorrMessage", "GlassWorm led-win32 typo"),
    ("complite", "GlassWorm extension typo"),
];
```

**Acceptance Criteria:**
- ✅ All 6 typos detected
- ✅ Correct severity (High)
- ✅ No false positives on legitimate typos
- ✅ Tests passing

**Estimated:** 1 day

---

### G4: Exfil JSON Schema Matcher ✅

**File:** `glassware-core/src/detectors/exfil_schema.rs` (NEW)

**Task List:**
- [ ] Create `ExfilSchemaDetector` struct
- [ ] Implement `Detector` trait
- [ ] Extract complete key list from Part 5 exfil schema
- [ ] Implement threshold-based detection (≥3 keys)
- [ ] Make threshold configurable
- [ ] Create test fixtures
- [ ] Register detector

**Exfil Keys (from Part 5):**
```rust
const EXFIL_KEYS: &[&str] = &[
    // Counts
    "cookieCount", "loginCount", "creditCardCount",
    
    // OAuth/Tokens
    "sync_oauth_token", "send_tab_private_key",
    "close_tab_private_key", "session_token",
    
    // Encryption keys
    "app_bound_key", "dpapi_key", "master_key",
    
    // Account info
    "email", "uid", "verified",
    
    // Data arrays
    "cookies", "logins", "credit_cards",
    "autofill", "history", "bookmarks", "tokens",
];
```

**Acceptance Criteria:**
- ✅ ≥3 keys triggers detection
- ✅ Threshold configurable
- ✅ Complete key list from Part 5
- ✅ No false positives on legitimate JSON
- ✅ Tests passing

**Estimated:** 2 days

---

### G5: Socket.IO / TCP Tunnel C2 ✅

**File:** `glassware-core/src/blockchain_c2_detector.rs` (EXTEND) or new `socketio_c2_detector.rs`

**Task List:**
- [ ] Create compound pattern matcher (NOT individual strings)
- [ ] Implement signal scoring system
- [ ] Set threshold ≥3 signals
- [ ] Create test fixtures (true positive + false positive cases)
- [ ] Register detector

**Signal Scoring:**
```rust
// Signal 1: Socket.IO client import (+2 points)
"socket.io-client", "require('socket.io')", "import io from"

// Signal 2: Suspicious port (+1 point)
":5000", ":5001", ":5050", "port: 5000"

// Signal 3: Specific tunnel packages (+1 point)
"tunnel-agent", "socks-proxy-agent", "http-proxy-agent"

// Signal 4: Connection pattern (+1 point)
"io.connect(", "socket.connect(", ".on('connect')"

// Signal 5: GlassWorm endpoints (+2 points)
"/api/socket", "/c2", "/tunnel", "/memo"

// Threshold: ≥3 points required
```

**Test Fixtures:**
- ✅ True positive: Socket.IO + port 5000 + tunnel (5 points → flagged)
- ✅ False positive: Express.js with `io(` only (1 point → NOT flagged)
- ✅ False positive: Flask dev server on :5000 (1 point → NOT flagged)

**Acceptance Criteria:**
- ✅ Compound pattern (≥3 signals)
- ✅ No false positives on Express.js, Flask
- ✅ Detects GlassWorm C2 pattern
- ✅ Tests passing

**Estimated:** 2 days

---

## Severity Scoring Overhaul ✅

**Files:** `glassware-core/src/severity_context.rs` (NEW), `glassware-core/src/reputation.rs` (NEW)

**Task List:**
- [ ] Create `SeverityContext` struct
- [ ] Create `ReputationScore` enum
- [ ] Create `Ecosystem` enum
- [ ] Implement `calculate_multiplier()` with per-category caps
- [ ] Integrate with `Finding` struct
- [ ] Update `risk_scorer.rs`
- [ ] Create test cases
- [ ] Update documentation

**Multiplier Caps:**
```rust
metadata_mult = (age * author * ecosystem).min(3.0)
behavioral_mult = (1.0 + behavior_score).min(4.0)
historical_mult = (history_score).min(3.0)
total = (metadata_mult * behavioral_mult * historical_mult).min(10.0)
```

**Acceptance Criteria:**
- ✅ Per-category caps working
- ✅ Global cap at 10.0
- ✅ No explosion from multiplicative stacking
- ✅ Tests passing

**Estimated:** 2 days (parallel with Phase 1)

---

## Sprint Board

### Week 1 (Mar 24-28)

| Day | Task | Status |
|-----|------|--------|
| Mon | E1: IoC lists | ⏳ Pending |
| Mon | E3: Browser-kill patterns | ⏳ Pending |
| Tue | G3: Typo attribution | ⏳ Pending |
| Wed | G4: Exfil schema (part 1) | ⏳ Pending |
| Thu | G4: Exfil schema (part 2) | ⏳ Pending |
| Fri | G5: Socket.IO C2 | ⏳ Pending |

### Week 2 (Mar 31 - Apr 4)

| Day | Task | Status |
|-----|------|--------|
| Mon | Severity context (part 1) | ⏳ Pending |
| Tue | Severity context (part 2) | ⏳ Pending |
| Wed | Integration testing | ⏳ Pending |
| Thu | Documentation | ⏳ Pending |
| Fri | Buffer / catch-up | ⏳ Pending |

---

## Test Coverage Requirements

Each detector must have:
- [ ] True positive test (known GlassWorm pattern)
- [ ] False positive test (legitimate code)
- [ ] Edge case test (evasion attempt)
- [ ] Threshold test (boundary conditions)

---

## Definition of Done

- [ ] All code implemented
- [ ] All tests passing
- [ ] Documentation updated
- [ ] No clippy warnings
- [ ] No breaking changes
- [ ] Benchmarks show <5% performance impact

---

## Blockers & Risks

| Risk | Mitigation | Status |
|------|------------|--------|
| G5 false positives | Compound pattern matcher | ✅ Addressed |
| G4 key list incomplete | Extract from Part 5 carefully | ⏳ In progress |
| Severity caps too aggressive | Conservative defaults, tunable | ⏳ In progress |
| Typo list verification | Cross-reference with Part 5 | ⏳ In progress |

---

## Progress Tracking

**Start Date:** Mar 24  
**Target End Date:** Apr 4  
**Current Status:** 🔄 In Progress

**Completed:**
- [ ] E1: IoC lists
- [ ] E3: Browser-kill patterns
- [ ] G3: Typo attribution
- [ ] G4: Exfil schema
- [ ] G5: Socket.IO C2
- [ ] Severity context

**Blockers:** None

---

**End of Sprint Plan**
