# glassware-orchestrator CLI

Command-line interface for orchestrating glassware security scans across npm and GitHub.

## Installation

```bash
# Install from source
cargo install --path glassware-orchestrator/orchestrator-cli

# Or build directly
cargo build --release
./target/release/glassware-orchestrator
```

## Quick Start

```bash
# Scan npm packages
glassware-orchestrator scan-npm express lodash

# Scan GitHub repositories
glassware-orchestrator scan-github owner/repo owner2/repo2

# Scan from file list
glassware-orchestrator scan-file packages.txt

# Output to SARIF
glassware-orchestrator scan-npm express --format sarif > results.sarif
```

## Commands

### scan-npm

Scan npm packages for security issues.

```bash
glassware-orchestrator scan-npm [OPTIONS] [PACKAGES]...

# Examples:
glassware-orchestrator scan-npm express lodash
glassware-orchestrator scan-npm --format json express > results.json
glassware-orchestrator scan-npm --severity high express
glassware-orchestrator scan-npm --llm express  # With LLM analysis
```

### scan-github

Scan GitHub repositories for security issues.

```bash
glassware-orchestrator scan-github [OPTIONS] [REPOS]...

# Examples:
glassware-orchestrator scan-github rust-lang/rust tokio-rs/tokio
glassware-orchestrator scan-github --ref main owner/repo
glassware-orchestrator scan-github --format sarif owner/repo > results.sarif
```

### scan-file

Scan packages from a file list.

```bash
glassware-orchestrator scan-file [OPTIONS] <FILE>

# File format (one package per line):
# express
# lodash
# owner/repo  # GitHub repositories

# Examples:
glassware-orchestrator scan-file packages.txt
glassware-orchestrator scan-file --format json repos.txt > results.json
```

### resume

Resume a scan from checkpoint.

```bash
glassware-orchestrator resume [OPTIONS] <SOURCE>

# Examples:
glassware-orchestrator resume npm
glassware-orchestrator resume github
glassware-orchestrator resume npm --format json > results.json
```

### cache-stats

Show cache statistics.

```bash
glassware-orchestrator cache-stats [OPTIONS]

# Examples:
glassware-orchestrator cache-stats
glassware-orchestrator cache-stats --format json
glassware-orchestrator cache-stats --clear  # Clear cache after showing stats
```

### cache-cleanup

Clean up expired cache entries.

```bash
glassware-orchestrator cache-cleanup

# Output:
# Cleaned up 42 expired cache entries
```

## Options

### Global Options

| Option | Description | Default |
|--------|-------------|---------|
| `--format`, `-f` | Output format: `pretty`, `json`, `sarif` | `pretty` |
| `--severity`, `-s` | Minimum severity: `info`, `low`, `medium`, `high`, `critical` | `low` |
| `--quiet`, `-q` | Suppress output, only set exit code | `false` |
| `--verbose`, `-v` | Enable debug logging | `false` |
| `--no-color` | Disable colored output | `false` |
| `--no-cache` | Disable caching | `false` |
| `--cache-db` | Custom cache database path | `.glassware-orchestrator-cache.db` |
| `--cache-ttl` | Cache TTL in days | `7` |
| `--concurrency` | Maximum concurrent scans | `10` |
| `--max-retries` | Maximum retries per operation | `3` |
| `--npm-rate-limit` | npm API rate limit (req/sec) | `2.0` |
| `--github-rate-limit` | GitHub API rate limit (req/sec) | `1.0` |
| `--github-token` | GitHub personal access token | (none) |
| `--threat-threshold` | Threat score threshold for marking as malicious | `5.0` |
| `--help`, `-h` | Show help message | - |
| `--version`, `-V` | Show version | - |

### Output Formats

#### Pretty (Default)

Human-readable output with colors:

```
============================================================
SCAN SUMMARY
============================================================
Total packages scanned: 10
Malicious packages: 2
Total findings: 15
Average threat score: 3.45

Findings by severity:
  Critical: 2
  High: 5
  Medium: 3
  Low: 5

🚨 Malicious Packages Detected:
  - malicious-pkg (1.0.0) [threat score: 7.50]
    [Critical] src/index.js:42 - Invisible character detected
```

#### JSON

Machine-readable JSON output:

```bash
glassware-orchestrator scan-npm express --format json
```

```json
{
  "summary": {
    "total_packages": 10,
    "malicious_packages": 2,
    "total_findings": 15,
    "average_threat_score": 3.45,
    "findings_by_severity": {...},
    "findings_by_category": {...}
  },
  "results": [...],
  "errors": []
}
```

#### SARIF

GitHub Advanced Security compatible SARIF 2.1.0:

```bash
glassware-orchestrator scan-npm express --format sarif > results.sarif
```

Upload to GitHub:

```bash
gh api \
  --method POST \
  -H "Accept: application/vnd.github+json" \
  /repos/OWNER/REO/code-scanning/sarifs \
  -f "sarif=$(base64 -i results.sarif)" \
  -f "ref=refs/heads/main"
```

## Advanced Usage

### Streaming Output

For large scans, use streaming output to prevent OOM:

```bash
# JSON Lines format (one JSON object per line)
glassware-orchestrator scan-file large-list.txt --format json > results.jsonl

# Process results as they arrive
glassware-orchestrator scan-npm --format json pkg1 pkg2 | jq -c '.'
```

### Adversarial Testing

Test packages for evasion techniques:

```bash
# Note: Currently available as library API only
# See orchestrator-core/README.md for usage
```

### Tracing and Logging

Enable debug logging:

```bash
glassware-orchestrator scan-npm express --verbose
```

Custom log level via environment variable:

```bash
RUST_LOG=debug glassware-orchestrator scan-npm express
RUST_LOG=orchestrator_core=trace glassware-orchestrator scan-npm express
```

Log to file:

```bash
# Configure in your application code (library mode)
use orchestrator_core::tracing::{TracingConfig, TracingOutput};

let config = TracingConfig::default()
    .with_output(TracingOutput::File("scan.log".to_string()));
```

### Checkpoint/Resume

Long-running scans automatically save checkpoints:

```bash
# Start scan
glassware-orchestrator scan-file large-list.txt

# If interrupted, resume from checkpoint
glassware-orchestrator resume npm
```

Checkpoints are stored in `.glassware-checkpoints/`.

### Cache Management

View cache statistics:

```bash
glassware-orchestrator cache-stats
```

```
Cache Statistics:
  Total entries: 150
  Expired entries: 12
  npm entries: 100
  GitHub entries: 50
  File entries: 0
```

Clear expired entries:

```bash
glassware-orchestrator cache-cleanup
```

Clear all cache:

```bash
glassware-orchestrator cache-stats --clear
```

### GitHub Integration

Scan private repositories:

```bash
glassware-orchestrator scan-github --github-token $GITHUB_TOKEN owner/private-repo
```

Scan specific branch or tag:

```bash
glassware-orchestrator scan-github --ref v1.0.0 owner/repo
```

### LLM Analysis

Enable LLM analysis for flagged files (requires API key):

```bash
export GLASSWARE_LLM_BASE_URL="https://api.cerebras.ai/v1"
export GLASSWARE_LLM_API_KEY="your-api-key"

glassware-orchestrator scan-npm --llm suspicious-pkg
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | No findings at or above severity threshold |
| 1 | Findings detected |
| 2 | Error (file not found, permission denied, etc.) |

## Examples

### CI/CD Integration

GitHub Actions:

```yaml
- name: Install glassware-orchestrator
  run: cargo install --path glassware-orchestrator/orchestrator-cli

- name: Scan dependencies
  run: |
    glassware-orchestrator scan-file packages.txt --format sarif > results.sarif

- name: Upload SARIF
  uses: github/codeql-action/upload-sarif@v2
  with:
    sarif_file: results.sarif
```

### Batch Scanning

Scan all dependencies from package.json:

```bash
# Extract dependencies
jq -r '.dependencies | keys[]' package.json > deps.txt

# Scan
glassware-orchestrator scan-file deps.txt --format json > audit.json
```

### Automated Reporting

```bash
# Scan and generate report
glassware-orchestrator scan-npm express lodash --format json | \
  jq '.results[] | select(.is_malicious) | .package_name' > malicious.txt

# Send notification
if [ -s malicious.txt ]; then
  echo "Malicious packages detected!" | mail -s "Security Alert" team@example.com
fi
```

## Performance Tips

1. **Enable caching**: Re-scan same packages instantly
   ```bash
   glassware-orchestrator scan-npm express  # First scan
   glassware-orchestrator scan-npm express  # Cached (instant)
   ```

2. **Increase concurrency**: For large scans
   ```bash
   glassware-orchestrator scan-file large-list.txt --concurrency 20
   ```

3. **Use streaming**: For very large result sets
   ```bash
   glassware-orchestrator scan-file large-list.txt --format json > results.jsonl
   ```

4. **Filter by severity**: Reduce output size
   ```bash
   glassware-orchestrator scan-npm pkg --severity high
   ```

## Troubleshooting

### Rate Limit Exceeded

```bash
# Reduce rate limit
glassware-orchestrator scan-npm pkg --npm-rate-limit 1.0
```

### Cache Issues

```bash
# Clear cache and retry
glassware-orchestrator cache-stats --clear
glassware-orchestrator scan-npm pkg
```

### Memory Issues

```bash
# Reduce concurrency
glassware-orchestrator scan-file large-list.txt --concurrency 5

# Use streaming output
glassware-orchestrator scan-file large-list.txt --format json > results.jsonl
```

## License

Apache-2.0

## Contributing

See the main [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.
