# GlassWorm Configuration Examples

This directory contains example configuration files for different use cases.

## Available Configurations

### 1. `default.toml` - Standard Configuration

**Purpose:** General-purpose configuration for most users

**Characteristics:**
- Balanced thresholds (malicious: 7.0, suspicious: 3.0)
- Comprehensive package whitelists
- Moderate performance settings
- Both Cerebras and NVIDIA LLM configured

**Best for:**
- Local development
- Security research
- General package auditing

**Usage:**
```bash
# Copy to user config directory
cp config-examples/default.toml ~/.config/glassware/config.toml

# Or use as project config
cp config-examples/default.toml .glassware.toml
```

---

### 2. `strict.toml` - Production/Security-Critical Configuration

**Purpose:** Maximum security for production environments

**Characteristics:**
- Lower thresholds (malicious: 6.0, suspicious: 2.5)
- Higher detector weights
- Minimal whitelists
- Stronger LLM models (NVIDIA with 397B fallback)
- Slower, more thorough scanning

**Best for:**
- Production deployments
- Security-critical projects
- Financial/healthcare applications
- Pre-release security audits

**Usage:**
```bash
# For production projects
cp config-examples/strict.toml ~/.config/glassware/config.toml

# Or for specific security-critical project
cp config-examples/strict.toml /path/to/project/.glassware.toml
```

---

### 3. `ci-cd.toml` - CI/CD Pipeline Configuration

**Purpose:** Fast, automated scanning in CI/CD pipelines

**Characteristics:**
- Strict thresholds (malicious: 6.5)
- LLM disabled for speed
- High concurrency (20 parallel scans)
- Aggressive rate limits
- JSON output for parsing
- Short cache TTL (1 day)

**Best for:**
- GitHub Actions
- GitLab CI
- Jenkins pipelines
- Pre-merge security checks

**Usage:**
```bash
# In your CI/CD pipeline
cp config-examples/ci-cd.toml .glassware.toml

# Run scan
glassware-orchestrator scan-npm package1 package2 --format json > results.json

# Parse results
jq '.results[] | select(.is_malicious == true)' results.json
```

---

## Configuration Hierarchy

GlassWorm loads configuration from multiple sources (highest to lowest priority):

1. **CLI flags** - Override everything
2. **Project config** - `.glassware.toml` in current directory
3. **User config** - `~/.config/glassware/config.toml`
4. **Environment variables** - `GLASSWARE_*`
5. **Built-in defaults** - Hardcoded defaults

## Quick Start

```bash
# 1. Initialize default config
glassware-orchestrator config init

# 2. Edit config
glassware-orchestrator config edit

# 3. Validate config
glassware-orchestrator config validate

# 4. Show current config
glassware-orchestrator config show
```

## Configuration Reference

### Scoring

| Setting | Default | Strict | CI/CD | Description |
|---------|---------|--------|-------|-------------|
| `malicious_threshold` | 7.0 | 6.0 | 6.5 | Score >= this is "malicious" |
| `suspicious_threshold` | 3.0 | 2.5 | 3.0 | Score >= this is "suspicious" |
| `category_weight` | 2.0 | 2.5 | 2.0 | Weight per attack category |
| `critical_weight` | 3.0 | 4.0 | 3.5 | Weight per critical finding |
| `high_weight` | 1.5 | 2.0 | 1.5 | Weight per high severity |

### Performance

| Setting | Default | Strict | CI/CD | Description |
|---------|---------|--------|-------|-------------|
| `concurrency` | 10 | 5 | 20 | Parallel scans |
| `npm_rate_limit` | 10.0 | 5.0 | 20.0 | npm requests/second |
| `github_rate_limit` | 5.0 | 2.0 | 10.0 | GitHub requests/second |
| `cache_ttl_days` | 7 | 14 | 1 | Cache expiration |

### Output

| Setting | Default | Strict | CI/CD | Description |
|---------|---------|--------|-------|-------------|
| `format` | pretty | json | json | Output format |
| `min_severity` | low | info | medium | Minimum severity |
| `color` | true | false | false | Colored output |

## Testing Configurations

```bash
# Test with default config
glassware-orchestrator config validate

# Test with custom config
GLASSWARE_CONFIG=/path/to/config.toml glassware-orchestrator scan-npm express

# Test threat scoring
glassware-orchestrator scan-tarball package.tgz --threat-threshold 6.0
```

## See Also

- [Configuration System Design](../docs/CONFIG-SYSTEM-DESIGN.md)
- [User Guide](../docs/USER-GUIDE.md)
- [Workflow Guide](../docs/WORKFLOW-GUIDE.md)
