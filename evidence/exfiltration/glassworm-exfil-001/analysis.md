# Evidence Package: glassworm-exfil-001

## Attack Pattern

This package demonstrates **data exfiltration via custom HTTP headers**. Sensitive data is encoded and transmitted in non-standard HTTP headers to evade detection.

### Attack Mechanism

1. **Data Collection**
   - Environment variables (HOME, USER, PATH)
   - System information (platform, arch, Node version)
   - Working directory
   - Hostname
   - Timestamp

2. **Data Encoding**
   - JSON.stringify() to serialize
   - Base64 encoding for safe transmission
   - Split into 500-byte chunks

3. **Header Exfiltration**
   - Custom headers: `X-Exfil-ID`, `X-Session-Token`, `X-Timestamp`
   - Data chunks in `X-Data-0`, `X-Data-1`, etc.
   - Decoy body: `{"status": "ok"}` to appear normal

### Why Header Exfiltration

- **Hidden in plain sight** - Custom headers are common
- **Bypasses body inspection** - Security tools focus on request body
- **Session tracking** - Token allows correlating multiple requests
- **Chunked transmission** - Large data split across headers

## GlassWorm Indicators

- **X-Exfil-ID header** - Suspicious custom header
- **X-Session-Token header** - Session tracking
- **X-Data-* headers** - Data chunks in headers
- **Base64 encoding** - Buffer.from().toString('base64')
- **collector.evil.com** - Suspicious domain
- **Environment data collection** - Collecting sensitive info

## Expected Detection

- **Detector:** RDD URL Dependency (L2)
- **Severity:** High
- **Confidence:** 0.88
- **Category:** exfiltration/http

- **Detector:** GlassWare Pattern Detection (L2)
- **Severity:** Critical
- **Confidence:** 0.85
- **Category:** exfiltration/header_based

- **Detector:** Encrypted Payload Detection (L2)
- **Severity:** Medium
- **Confidence:** 0.75
- **Category:** encoded_data

## Source

Synthetic evidence created for Glassworks testing
Based on: https://codeberg.org/tip-o-deincognito/glassworm-writeup
