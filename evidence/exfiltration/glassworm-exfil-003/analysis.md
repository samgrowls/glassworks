# Evidence Package: glassworm-exfil-003

## Attack Pattern

This package demonstrates **DNS-based data exfiltration**. Data is encoded and sent via DNS queries, which are often allowed through firewalls and less monitored than HTTP traffic.

### Attack Mechanism

1. **Data Collection**
   - Environment variables (USER, HOME, PATH)
   - System information (hostname, platform, arch)
   - Timestamp

2. **Data Encoding**
   - JSON.stringify() for serialization
   - Hex encoding (Buffer.toString('hex')) for DNS-safe characters
   - Split into 50-character chunks (DNS label limit is 63)

3. **DNS Exfiltration**
   - Format: `<index>.<data>.exfil.evil.com`
   - Uses `dns.resolveTxt()` for TXT record queries
   - Can also use `dns.resolve4()` for A record queries
   - Attacker's DNS server logs all queries
   - NXDOMAIN responses are expected and ignored

### Why DNS Exfiltration

- **Firewall bypass** - DNS is rarely blocked
- **Low suspicion** - DNS queries are common
- **No response needed** - Query itself delivers data
- **Persistent** - Works even without HTTP access
- **Hard to detect** - Blends with normal DNS traffic

## GlassWorm Indicators

- **dns.resolveTxt()** - DNS TXT record queries
- **dns.resolve4()** - DNS A record queries
- **exfil.evil.com** - Suspicious domain
- **Hex encoding** - Buffer.toString('hex')
- **Subdomain data encoding** - Data in DNS subdomain
- **Chunked DNS queries** - Multiple sequential queries

## Expected Detection

- **Detector:** RDD URL Dependency (L2)
- **Severity:** High
- **Confidence:** 0.85
- **Category:** exfiltration/dns

- **Detector:** GlassWare Pattern Detection (L2)
- **Severity:** Critical
- **Confidence:** 0.82
- **Category:** exfiltration/dns_based

- **Detector:** Encrypted Payload Detection (L2)
- **Severity:** Medium
- **Confidence:** 0.70
- **Category:** encoded_data

## Source

Synthetic evidence created for Glassworks testing
Based on: https://codeberg.org/tip-o-deincognito/glassworm-writeup
