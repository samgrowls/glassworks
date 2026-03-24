# Evidence Package: glassworm-exfil-002

## Attack Pattern

This package demonstrates **environment variable exfiltration via base64 encoding**. It collects all environment variables (including secrets, API keys, tokens) and sends them to an external server.

### Attack Mechanism

1. **Environment Collection**
   - Iterates through all `process.env` variables
   - Collects everything (not just sensitive patterns)
   - Includes: API keys, secrets, tokens, passwords, paths

2. **Base64 Encoding**
   - `Buffer.from(JSON.stringify(process.env)).toString('base64')`
   - Single-line encoding of entire environment
   - Obscures data from casual inspection

3. **HTTP Exfiltration**
   - POST request to external collector
   - Custom headers: `X-Data-Type: env-vars`, `X-Encoding: base64`
   - JSON body with encoded data, timestamp, hostname

### Dangerous Pattern

```javascript
// One-liner that exfiltrates everything
const encoded = Buffer.from(JSON.stringify(process.env)).toString('base64');
```

### Why Environment Exfiltration

- **API Keys** - Cloud provider credentials
- **Database passwords** - Connection strings
- **Auth tokens** - Session/authentication tokens
- **Secret keys** - Signing/encryption keys
- **Internal URLs** - Infrastructure details

## GlassWorm Indicators

- **Buffer.from(process.env)** - Environment variable access
- **toString('base64')** - Base64 encoding
- **JSON.stringify(process.env)** - Full environment serialization
- **data.collector.evil.com** - Suspicious domain
- **X-Data-Type: env-vars** - Explicit data type header
- **X-Encoding: base64** - Encoding disclosure

## Expected Detection

- **Detector:** RDD URL Dependency (L2)
- **Severity:** High
- **Confidence:** 0.90
- **Category:** exfiltration/environment

- **Detector:** Encrypted Payload Detection (L2)
- **Severity:** High
- **Confidence:** 0.85
- **Category:** base64_encoding

- **Detector:** GlassWare Pattern Detection (L2)
- **Severity:** Critical
- **Confidence:** 0.88
- **Category:** exfiltration/env_vars

## Source

Synthetic evidence created for Glassworks testing
Based on: https://codeberg.org/tip-o-deincognito/glassworm-writeup
