# GitHub Token Guide

**Date:** 2026-03-19  

---

## Do You Need a Token?

**Short answer:** NO, not for public repo scanning!

**Long answer:**
- **Unauthenticated:** 10 requests/hour (enough for ~300 searches/hour)
- **Our mixed scan:** Uses ~20 searches total
- **Conclusion:** Well within limits, no token needed

---

## When You WOULD Need a Token

### Scenario 1: Large-Scale Scanning

If you want to scan 5,000+ repos:
- Unauthenticated: Would take ~17 hours (rate limited)
- Authenticated: ~5-6 hours (3x faster)

### Scenario 2: Private Repos

If you need to scan private repos (not our current use case):
- Token required
- Fine-grained token with `Contents: Read` permission

### Scenario 3: Production/CI

If running in CI/CD or production:
- Token recommended for stability
- GitHub App recommended for very large scale (5,000 req/hour)

---

## Token Types

### Option 1: Fine-Grained Token (Recommended)

**Create at:** `github.com/settings/tokens`

**Configuration:**
```
Token type: Fine-grained
Resource owner: All repositories (or select specific orgs)
Repository access: All repositories (or select specific)
Permissions:
  - Contents: Read (for cloning)
  - (Nothing else needed for public repos)
```

**Pros:**
- Minimal permissions
- Can scope to specific orgs
- Easy to revoke

**Cons:**
- Beta feature (but stable)

---

### Option 2: Classic Token

**Create at:** `github.com/settings/tokens`

**Configuration:**
```
Token type: Classic
Scopes: (none needed for public repos)
  - OR: repo (if scanning private repos)
```

**Pros:**
- Stable, well-tested
- Works with all GitHub features

**Cons:**
- All-or-nothing permissions
- More permissions than needed

---

## How to Use Token

### Option A: Export Temporarily

```bash
export GITHUB_TOKEN="ghp_xxxxxxxxxxxxx"
python3 github_scanner.py --queries "mcp" --max-repos 500
```

### Option B: Add to ~/.env

```bash
# ~/.env
GITHUB_TOKEN=ghp_xxxxxxxxxxxxx
```

Then source it:
```bash
source ~/.env
python3 github_scanner.py --queries "mcp" --max-repos 500
```

### Option C: Pass as Argument

```bash
GITHUB_TOKEN="ghp_xxxxxxxxxxxxx" python3 github_scanner.py \
  --queries "mcp" \
  --max-repos 500
```

---

## Token Format

Tokens start with:
- **Fine-grained:** `github_pat_`
- **Classic:** `ghp_` (personal), `gho_` (oauth), `ghu_` (user), `ghs_` (server)

Example:
```
ghp_1a2b3c4d5e6f7g8h9i0j
github_pat_1a2b3c4d5e6f7g8h9i0j_abcdefghijklmnop
```

---

## Security Best Practices

1. **Never commit tokens** to git
2. **Use fine-grained tokens** with minimal permissions
3. **Rotate tokens** every 90 days
4. **Revoke unused tokens** immediately
5. **Use environment variables** instead of hardcoding

---

## Current Scan Status

**Mixed scan (900 repos):**
- **Without token:** ~2-4 hours (current)
- **With token:** ~2-4 hours (same, we're within limits)

**Recommendation:** Don't add token for this scan. Save it for larger scans (5,000+ repos).

---

## Monitoring Current Scan

```bash
# Watch progress
tail -f harness/github-mixed-scan.log

# Check status
ps aux | grep github_scanner | grep -v grep

# View results
cat harness/github-mixed-scan-results.json | jq '{scanned, flagged, errors}'
```

---

**Bottom line:** Token is optional for our current use case. Add it later if you need higher rate limits for larger scans.
