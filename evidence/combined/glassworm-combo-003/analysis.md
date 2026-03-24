# Evidence Package: glassworm-combo-003

## Attack Pattern

This package demonstrates **combined sandbox evasion + exfiltration**. It performs comprehensive environment checks before exfiltrating sensitive data.

### Attack Chain

**Phase 1: Comprehensive Sandbox Detection**
- CPU count (< 2 cores)
- Memory (< 2GB)
- Uptime (< 60 seconds)
- CI environment variables
- NODE_ENV check
- Debug/inspect flags

**Phase 2: Conditional Execution**
- **In sandbox**: Silent exit after 3 seconds
- **In real environment**: Proceed to exfiltration

**Phase 3: Data Exfiltration**
- Collect environment variables (including secrets)
- Collect system information
- Collect project details
- Base64 encode all data
- POST to external collector with custom headers

### Why Combine These Techniques

- **Avoids analysis** - Won't trigger in sandboxes
- **Stealthy exfil** - Only exfiltrates when safe
- **Comprehensive detection** - Multiple evasion indicators
- **Clean exit** - No errors that might alert analysts

## GlassWorm Indicators

- **os.cpus().length < 2** - CPU detection
- **os.totalmem() < 2GB** - Memory detection
- **os.uptime() < 60** - Uptime detection
- **process.env.CI** - CI detection
- **process.execArgv inspection** - Debug flag check
- **process.exit(0)** - Silent exit
- **Buffer.from().toString('base64')** - Base64 encoding
- **collector.evil.com** - Suspicious domain
- **X-Exfil-ID header** - Custom exfil header

## Expected Detection

- **Detector:** Time Delay Sandbox Evasion (L3)
- **Severity:** Critical
- **Confidence:** 0.95
- **Category:** behavioral/sandbox_evasion

- **Detector:** RDD URL Dependency (L2)
- **Severity:** High
- **Confidence:** 0.88
- **Category:** exfiltration/http

- **Detector:** GlassWare Pattern Detection (L2)
- **Severity:** Critical
- **Confidence:** 0.90
- **Category:** combined_attack

## Source

Synthetic evidence created for Glassworks testing
Based on: https://codeberg.org/tip-o-deincognito/glassworm-writeup
