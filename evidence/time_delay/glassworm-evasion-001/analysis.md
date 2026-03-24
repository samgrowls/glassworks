# Evidence Package: glassworm-evasion-001

## Attack Pattern

This package demonstrates **sandbox evasion through CI/CD and VM detection**. The attack delays or prevents execution when it detects analysis environments.

### Detection Techniques

1. **CI Environment Detection**
   - Checks for common CI environment variables
   - Variables: CI, CONTINUOUS_INTEGRATION, GITHUB_ACTIONS, GITLAB_CI, TRAVIS, CIRCLECI, JENKINS_URL, etc.
   - CI environments are commonly used for automated security scanning

2. **VM Detection**
   - CPU count check: VMs often have 1 CPU core
   - Memory check: VMs often have <2GB RAM
   - Hostname patterns: Looks for 'vm', 'virtual', 'sandbox', 'docker', 'container'

3. **Combined Check**
   - `checkEnvironment()` returns CI and VM status
   - `shouldExecute` is false if either is detected

### Evasion Behavior

- **In safe environment**: Executes payload after 60-second delay
- **In CI/VM**: Logs detection, waits 1-2 seconds, then silently exits with code 0
- **Silent exit**: No error, no crash - just clean termination

## GlassWorm Indicators

- **process.env.CI** - CI environment variable check
- **os.cpus().length < 2** - CPU count VM detection
- **os.totalmem() < 2GB** - Memory-based VM detection
- **process.exit(0)** - Silent exit after detection
- **setTimeout with 60000ms** - 60-second execution delay
- **Hostname pattern matching** - Looking for sandbox indicators

## Expected Detection

- **Detector:** Time Delay Sandbox Evasion (L3)
- **Severity:** Critical
- **Confidence:** 0.92
- **Category:** behavioral/sandbox_evasion

- **Detector:** Locale Geofencing (L3)
- **Severity:** Medium
- **Confidence:** 0.75
- **Category:** environmental_detection

## Source

Synthetic evidence created for Glassworks testing
Based on: https://codeberg.org/tip-o-deincognito/glassworm-writeup
