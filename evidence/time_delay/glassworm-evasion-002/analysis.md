# Evidence Package: glassworm-evasion-002

## Attack Pattern

This package demonstrates **system resource-based sandbox detection**. It checks CPU count, memory, and uptime to identify analysis environments that typically have limited resources.

### Detection Techniques

1. **CPU Count Check**
   - Threshold: Minimum 2 CPU cores
   - Sandboxes often allocate only 1 core
   - Also checks CPU model and speed

2. **Memory Check**
   - Threshold: Minimum 2GB total RAM
   - Sandboxes often have 512MB-1GB
   - Also checks free memory percentage

3. **Uptime Check**
   - Threshold: Minimum 60 seconds uptime
   - Fresh VMs/sandboxes have near-zero uptime
   - Analysts spin up fresh VMs for each sample

### Evasion Behavior

- **2+ suspicious indicators**: Classified as "likely sandbox"
- **In sandbox**: 2-minute delay then silent exit
- **In real environment**: 5-second delay then payload execution

### Why This Works

- Automated analysis systems have resource constraints
- Running many samples in parallel requires limiting resources
- Fresh VMs are spun up for each sample analysis
- Analysts may not wait 2+ minutes for execution

## GlassWorm Indicators

- **os.cpus().length < 2** - CPU count threshold
- **os.totalmem() < 2GB** - Memory threshold
- **os.uptime() < 60** - Uptime threshold
- **process.exit(0)** - Silent exit
- **setTimeout with 120000ms** - 2-minute sandbox delay
- **suspiciousCount >= 2** - Multi-indicator detection

## Expected Detection

- **Detector:** Time Delay Sandbox Evasion (L3)
- **Severity:** Critical
- **Confidence:** 0.94
- **Category:** behavioral/sandbox_evasion

- **Detector:** Time Delay Sandbox Evasion (L3)
- **Severity:** High
- **Confidence:** 0.88
- **Category:** behavioral/resource_detection

## Source

Synthetic evidence created for Glassworks testing
Based on: https://codeberg.org/tip-o-deincognito/glassworm-writeup
