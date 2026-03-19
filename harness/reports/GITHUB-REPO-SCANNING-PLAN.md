# GitHub Repository Scanning Plan

**Date:** 2026-03-19  
**Status:** Design Proposal  
**Priority:** High (Post-npm scanning phase)  

---

## Executive Summary

**Why GitHub Repos?**

1. **Early Detection** - Catch malware before npm publication
2. **Source Analysis** - Scan actual source code, not bundled dist
3. **Commit History** - Track when malicious code was injected
4. **Maintainer Patterns** - Identify compromised accounts
5. **Intel Correlation** - Match known attack patterns from reports

**Current Intel:**
- 150+ GitHub repositories compromised (Koi Security)
- 72+ VS Code extensions infected
- Attackers target: MCP servers, VS Code extensions, devtools

---

## Target Repository Categories

### Tier 1: High-Value Targets

| Category | Keywords | Estimated Repos | Priority |
|----------|----------|-----------------|----------|
| **MCP Servers** | `mcp`, `model-context-protocol` | ~500 | 🔴 Critical |
| **VS Code Extensions** | `vscode`, `vsce`, `extension` | ~50,000 | 🔴 Critical |
| **Cursor Extensions** | `cursor`, `ai-editor` | ~200 | 🔴 Critical |
| **OpenVSX Extensions** | `openvsx`, `eclipse` | ~5,000 | 🔴 Critical |

### Tier 2: Developer Tools

| Category | Keywords | Estimated Repos | Priority |
|----------|----------|-----------------|----------|
| **Build Tools** | `node-gyp`, `webpack`, `babel` | ~2,000 | 🟡 High |
| **Package Managers** | `npm`, `yarn`, `pnpm` | ~500 | 🟡 High |
| **CLI Tools** | `cli`, `command-line` | ~10,000 | 🟡 High |

### Tier 3: Infrastructure

| Category | Keywords | Estimated Repos | Priority |
|----------|----------|-----------------|----------|
| **Security Tools** | `security`, `scanner`, `audit` | ~1,000 | 🟢 Medium |
| **CI/CD** | `github-actions`, `ci`, `cd` | ~5,000 | 🟢 Medium |
| **Monitoring** | `logging`, `monitoring`, `apm` | ~2,000 | 🟢 Medium |

---

## Discovery Strategies

### Strategy 1: Keyword Search (GitHub API)

```python
# github_scanner.py
SEARCH_QUERIES = {
    "mcp": [
        "mcp-server",
        "model-context-protocol",
        "mcp extension",
    ],
    "vscode": [
        "vscode-extension",
        "vsce",
        "visual-studio-code",
    ],
    "cursor": [
        "cursor-extension",
        "cursor-ide",
    ],
}

def search_github(query: str, sort: str = "updated", order: str = "desc") -> list:
    """Search GitHub for repositories"""
    # Use GitHub Search API
    # https://docs.github.com/en/rest/search/search
    pass
```

**Rate Limits:**
- Unauthenticated: 10 requests/hour
- Authenticated: 30 requests/hour
- GitHub App: 5,000 requests/hour

**Recommendation:** Use GitHub App for production scanning

---

### Strategy 2: Known Compromised Maintainers

```python
# Track maintainers from intel reports
COMPROMISED_MAINTAINERS = [
    "oorzc",           # VS Code extensions (72+ compromised)
    "AstrOOnauta",     # React Native packages
    "iflow-mcp",       # MCP servers
    # Add more from intel reports
]

def get_maintainer_repos(username: str) -> list:
    """Get all repos for a maintainer"""
    # GET /users/{username}/repos
    pass

def get_maintainer_packages(username: str) -> list:
    """Get all npm packages for a maintainer"""
    # Cross-reference with npm API
    pass
```

---

### Strategy 3: npm → GitHub Correlation

```python
# For packages already scanned from npm:
def get_github_repo_from_package(package_info: dict) -> Optional[str]:
    """Extract GitHub repo from npm package metadata"""
    
    # Check repository field
    if "repository" in package_info:
        repo = package_info["repository"]
        if isinstance(repo, dict):
            return repo.get("url")
        return repo
    
    # Check bugs field
    if "bugs" in package_info:
        bugs = package_info["bugs"]
        if "url" in bugs:
            return extract_github_from_url(bugs["url"])
    
    # Check homepage
    if "homepage" in package_info:
        return extract_github_from_url(package_info["homepage"])
    
    return None
```

**Benefit:** Scan source code for packages we already flagged

---

### Strategy 4: Directory/Aggregator Sites

**Examples:**
- https://opencli.co/ (mentioned by user)
- https://github.com/topics/vscode-extension
- https://github.com/topics/mcp-server
- https://marketplace.visualstudio.com/vscode

**Approach:**
1. Scrape aggregator sites for repo lists
2. Extract GitHub URLs
3. Add to scan queue

**Caution:** Respect robots.txt, rate limits

---

## Scanning Workflow

### Phase 1: Repository Discovery

```python
def discover_repositories():
    """Discover repositories to scan"""
    repos = set()
    
    # Strategy 1: Keyword search
    for category, queries in SEARCH_QUERIES.items():
        for query in queries:
            results = search_github(query)
            repos.update(results)
    
    # Strategy 2: Compromised maintainers
    for maintainer in COMPROMISED_MAINTAINERS:
        repos.update(get_maintainer_repos(maintainer))
    
    # Strategy 3: npm correlation
    for package in flagged_npm_packages:
        repo = get_github_repo_from_package(package)
        if repo:
            repos.add(repo)
    
    return list(repos)
```

### Phase 2: Repository Cloning

```python
def clone_repository(repo_url: str, cache_dir: Path) -> Path:
    """Clone repository (or use cached copy)"""
    
    repo_name = extract_repo_name(repo_url)
    cache_path = cache_dir / repo_name
    
    if cache_path.exists():
        # Pull latest
        run_git_command(["git", "pull"], cwd=cache_path)
        return cache_path
    
    # Clone fresh
    run_git_command(["git", "clone", repo_url, str(cache_path)])
    return cache_path
```

**Optimizations:**
- Cache repositories locally
- Only fetch latest commit (shallow clone: `--depth 1`)
- Skip repos >100MB (likely not source code)

### Phase 3: Source Code Scanning

```python
def scan_repository(repo_path: Path) -> ScanResult:
    """Scan repository source code"""
    
    # Skip bundled/compiled directories
    skip_dirs = [
        "node_modules",
        "dist",
        "build",
        "out",
        "vendor",
        ".git",
    ]
    
    # Scan source files only
    source_files = []
    for ext in ["*.js", "*.ts", "*.mjs", "*.jsx", "*.tsx"]:
        source_files.extend(repo_path.rglob(ext))
    
    # Filter out skipped directories
    source_files = [
        f for f in source_files
        if not any(skip in str(f) for skip in skip_dirs)
    ]
    
    # Run glassware on source files
    findings = []
    for file in source_files:
        result = run_glassware(file)
        findings.extend(result.findings)
    
    return ScanResult(findings=findings)
```

**Key Difference from npm:**
- Scan **source code** (`.ts`, `.js`) not bundled (`.d.ts`, `/dist/`)
- Much lower false positive rate
- Can detect injection in source before bundling

### Phase 4: Commit History Analysis

```python
def analyze_commit_history(repo_path: Path) -> list:
    """Analyze commit history for suspicious changes"""
    
    # Get recent commits
    commits = run_git_command(
        ["git", "log", "--oneline", "-20"],
        cwd=repo_path
    )
    
    suspicious_commits = []
    for commit in commits:
        # Look for suspicious patterns
        if any(pattern in commit for pattern in [
            "update dependencies",
            "fix build",
            "minor changes",
            "version bump",
        ]):
            # Check what files changed
            changed_files = run_git_command(
                ["git", "show", "--name-only", commit.split()[0]],
                cwd=repo_path
            )
            
            # Flag if install scripts changed
            if any(f in changed_files for f in [
                "package.json",
                "install.js",
                "preinstall.js",
                "postinstall.js",
            ]):
                suspicious_commits.append({
                    "commit": commit,
                    "changed_files": changed_files,
                    "reason": "Install script modified",
                })
    
    return suspicious_commits
```

**Benefit:** Identify **when** malware was injected

---

## Integration with Current System

### Database Schema Additions

```sql
-- GitHub repositories table
CREATE TABLE github_repos (
    id              INTEGER PRIMARY KEY,
    name            TEXT NOT NULL,  -- owner/repo
    url             TEXT NOT NULL,
    cloned_at       TEXT,
    commit_hash     TEXT,
    scan_run_id     TEXT REFERENCES scan_runs(id),
    finding_count   INTEGER DEFAULT 0,
    suspicious_commits TEXT,  -- JSON array
    UNIQUE(name)
);

-- Link npm packages to GitHub repos
CREATE TABLE package_repo_links (
    package_name    TEXT NOT NULL,
    package_version TEXT NOT NULL,
    repo_name       TEXT NOT NULL,
    discovered_at   TEXT NOT NULL,
    PRIMARY KEY (package_name, package_version)
);
```

### Unified Scan Results

```python
# Same glassware binary works on both:
# npm packages (extracted tarballs)
./target/release/glassware /path/to/package/

# GitHub repos (cloned source)
./target/release/glassware /path/to/repo/

# Results go to same database, different source type
```

---

## Rate Limiting & Authentication

### GitHub API Rate Limits

| Auth Type | Requests/Hour | Recommended For |
|-----------|---------------|-----------------|
| **None** | 10 | Testing only |
| **Personal Token** | 5,000 | Individual scanning |
| **GitHub App** | 5,000 | Production scanning |

### Recommendation: GitHub App

**Benefits:**
- Higher rate limits
- Can install on specific orgs/repos
- Better audit trail
- Can access private repos (if authorized)

**Setup:**
1. Create GitHub App at https://github.com/settings/apps
2. Request permissions: `contents:read`, `metadata:read`
3. Install on target orgs/repos
4. Use JWT for authentication

---

## Prioritization Strategy

### Priority Queue

```python
@dataclass
class RepoPriority:
    repo_url: str
    score: int
    reasons: list[str]

def calculate_priority(repo: dict) -> RepoPriority:
    """Calculate scan priority for a repository"""
    score = 0
    reasons = []
    
    # Factor 1: Category match
    if "mcp" in repo["name"].lower():
        score += 100
        reasons.append("MCP server")
    
    # Factor 2: Maintainer history
    if repo["owner"] in COMPROMISED_MAINTAINERS:
        score += 200
        reasons.append("Compromised maintainer")
    
    # Factor 3: Recent activity
    if is_recently_updated(repo["updated_at"], days=30):
        score += 50
        reasons.append("Recently updated")
    
    # Factor 4: npm correlation
    if has_npm_package(repo):
        score += 75
        reasons.append("Published to npm")
    
    # Factor 5: Stars/popularity
    if repo["stargazers_count"] > 1000:
        score += 25
        reasons.append("Popular repo")
    
    return RepoPriority(
        repo_url=repo["html_url"],
        score=score,
        reasons=reasons,
    )
```

### Scan Order

1. **Score ≥ 300** - Scan immediately (critical)
2. **Score 200-299** - Scan within 24 hours (high)
3. **Score 100-199** - Scan within week (medium)
4. **Score < 100** - Scan when resources available (low)

---

## Expected Performance

### Discovery Phase

| Strategy | Repos Found | Time |
|----------|-------------|------|
| Keyword search (10 queries) | ~500 | 10 minutes |
| Maintainer lookup (5 maintainers) | ~50 | 5 minutes |
| npm correlation (100 packages) | ~75 | 15 minutes |
| **Total** | **~625** | **30 minutes** |

### Scanning Phase

| Operation | Time per Repo | Total (500 repos) |
|-----------|---------------|-------------------|
| Clone (shallow) | ~10s | ~1.5 hours |
| Scan source files | ~5s | ~45 minutes |
| Commit analysis | ~2s | ~17 minutes |
| **Total** | **~17s** | **~3 hours** |

**With parallel cloning/scanning (10 workers):** ~20 minutes total

---

## False Positive Expectations

### npm vs GitHub FP Rates

| Source | Expected FP Rate | Reason |
|--------|------------------|--------|
| **npm packages** | ~5-10% | Bundled code, compiled output |
| **GitHub source** | ~1-2% | Source code only, no bundles |

**Why GitHub is better:**
- Scan `.ts` source, not `/dist/` bundles
- Can skip `node_modules/` entirely
- Can analyze commit history for context
- Can compare clean vs compromised versions

---

## Implementation Phases

### Phase 1: Basic GitHub Scanning (3 days)

- [ ] GitHub API integration (search, clone)
- [ ] Repository discovery (keyword search)
- [ ] Source code scanning (glassware integration)
- [ ] Database schema updates
- [ ] Basic prioritization

**Deliverable:** Scan 500 MCP-related repos

### Phase 2: Advanced Features (2 days)

- [ ] Commit history analysis
- [ ] npm → GitHub correlation
- [ ] Maintainer tracking
- [ ] Priority scoring system

**Deliverable:** Intelligent repo prioritization

### Phase 3: Production Readiness (2 days)

- [ ] GitHub App authentication
- [ ] Rate limit handling
- [ ] Caching optimization
- [ ] Progress tracking/dashboard

**Deliverable:** Production-ready GitHub scanner

**Total:** 7 days for full implementation

---

## Immediate Next Steps

1. ✅ **Wait for current npm scan to complete**
2. ⏳ **Create GitHub App** for authentication
3. ⏳ **Implement Phase 1** (basic scanning)
4. ⏳ **Test on known compromised repos** (from intel reports)
5. ⏳ **Scale to full MCP ecosystem**

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| **GitHub rate limits** | Can't discover repos | Use GitHub App, cache results |
| **Large repos** | Slow cloning | Skip repos >100MB, shallow clone |
| **False positives** | Wasted investigation time | Source-only scanning, commit analysis |
| **Missing repos** | Attackers use new accounts | Multiple discovery strategies |

---

**Status:** Ready for implementation after npm scan completes  
**Next Decision:** Approve Phase 1 implementation plan?
