# Responsible Disclosure Policy

## Overview

This document outlines the responsible disclosure process for security findings discovered by the glassware scanning harness when analyzing npm packages.

## Scope

This policy applies to:
- Packages flagged by glassware with **Critical** or **High** severity findings
- Packages containing install scripts (`preinstall`, `postinstall`) with any security findings
- Patterns consistent with known supply chain attack techniques (GlassWare, encrypted loaders, credential harvesting)

## Disclosure Timeline

### Phase 1: Verification (Internal)
**Duration: 24-48 hours**

1. Verify findings are not false positives
2. Confirm the malicious pattern (decode payloads if present)
3. Document evidence (file paths, decoded payloads, network indicators)
4. Assign internal severity rating

### Phase 2: Report to npm Security
**Duration: Immediate after verification**

1. Send initial report to **security@npmjs.com**
2. Include:
   - Package name and version
   - Scan report ID from glassware harness
   - Summary of findings with severity
   - Evidence (decoded payloads, suspicious code snippets)
   - IOCs (IP addresses, domains, wallet addresses if present)

**Email Template:**
```
Subject: [URGENT] Supply Chain Attack - {package_name}@{version}

Dear npm Security Team,

Our automated scanning system has identified a potentially malicious package 
on npm that exhibits patterns consistent with supply chain attacks.

Package: {name}@{version}
Report ID: {run_id}
Severity: {critical/high}

Summary:
[Brief description of the malicious pattern]

Evidence:
- Decoded payload: [if applicable]
- Suspicious files: [file paths]
- Network IOCs: [domains/IPs if present]

The package has been archived in our vault for analysis. We request immediate 
investigation and potential removal pending your review.

Please confirm receipt of this report.

Regards,
glassware Security Research
```

### Phase 3: Coordination Period
**Duration: 24-72 hours**

- Wait for npm Security acknowledgment
- Provide additional evidence if requested
- Coordinate on removal timing
- **Do NOT** publish findings publicly during this period

### Phase 4: Public Disclosure
**Timing: After package removal confirmed**

Once npm confirms package removal:

1. Publish sanitized report to project repository
2. Include:
   - Package name (now safe since removed)
   - Technical analysis
   - Timeline of disclosure
   - Recommendations for users
3. Credit npm Security for responsive action

## False Positive Handling

If npm determines a finding is a false positive:

1. Update scanner to reduce similar false positives
2. Document the case for future reference
3. No public disclosure necessary
4. Add package to allowlist if appropriate

## Emergency Contacts

- **npm Security:** security@npmjs.com
- **npm Security (urgent):** https://www.npmjs.com/support (security issues)

## Data Retention

- Scan results: Retained indefinitely in corpus database
- Vault archives: Retained for 1 year after package removal
- Personal data (author names): Only as retrieved from npm metadata

## Legal Considerations

- This is a security research project
- All analysis performed on publicly available npm packages
- Findings shared responsibly with platform security team first
- Public disclosure only after remediation

## Questions?

This policy is a living document. Issues and PRs welcome.

---

*Last updated: 2026-03-18*
