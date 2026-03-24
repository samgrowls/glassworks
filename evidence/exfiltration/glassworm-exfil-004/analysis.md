# Evidence Package: glassworm-exfil-004

## Attack Pattern

This package demonstrates **GitHub API-based exfiltration using Gists**. It leverages GitHub's infrastructure to store exfiltrated data in secret gists, which are difficult to detect and trace.

### Attack Mechanism

1. **Data Collection**
   - NPM tokens
   - GitHub tokens
   - AWS credentials
   - System information
   - Project details

2. **Data Encoding**
   - JSON serialization
   - Base64 encoding

3. **GitHub Gist Exfiltration**
   - Creates **secret** gists (not public)
   - Uses `POST /gists` endpoint
   - Stores encoded data in gist files
   - Includes metadata file with timestamp/hostname
   - Can update existing gists for ongoing exfil

### Why GitHub Exfiltration

- **Legitimate traffic** - GitHub API calls are common
- **Trusted domain** - api.github.com is rarely blocked
- **Secret gists** - Not indexed or searchable
- **Persistent storage** - Data remains accessible
- **Hard to trace** - Looks like normal development activity
- **Token reuse** - Stolen tokens enable further access

### API Endpoints Used

- `POST https://api.github.com/gists` - Create new gist
- `PATCH https://api.github.com/gists/:id` - Update existing gist

## GlassWorm Indicators

- **api.github.com/gists** - GitHub Gists API endpoint
- **Authorization: token** - GitHub token authentication
- **public: false** - Secret gist creation
- **Buffer.from().toString('base64')** - Base64 encoding
- **X-Exfil-ID header** - Custom tracking header
- **GlassWorm-Exfil User-Agent** - Suspicious user agent

## Expected Detection

- **Detector:** RDD URL Dependency (L2)
- **Severity:** High
- **Confidence:** 0.88
- **Category:** exfiltration/github

- **Detector:** GlassWare Pattern Detection (L2)
- **Severity:** Critical
- **Confidence:** 0.85
- **Category:** exfiltration/api_based

- **Detector:** JPD Author Signature (L2)
- **Severity:** Medium
- **Confidence:** 0.70
- **Category:** suspicious_user_agent

## Source

Synthetic evidence created for Glassworks testing
Based on: https://codeberg.org/tip-o-deincognito/glassworm-writeup
