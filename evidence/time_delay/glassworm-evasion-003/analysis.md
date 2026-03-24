# Evidence Package: glassworm-evasion-003

## Attack Pattern

This package demonstrates **silent exit and time delay evasion techniques**. It combines multiple detection methods with delayed execution to evade automated analysis.

### Detection Techniques

1. **Resource Checks**
   - CPU count < 2
   - Memory < 2GB

2. **Environment Checks**
   - NODE_ENV === 'test' or 'development'
   - Debug/inspect flags in process.execArgv

### Evasion Behaviors

1. **Silent Exit**
   - Exits with code 0 (success)
   - Clears console output with newlines
   - No error messages or stack traces
   - Waits 60 seconds before exiting (analyst may have moved on)

2. **Delayed Execution**
   - In safe environments: 60-second delay before payload
   - Analysts often only wait 30 seconds for execution
   - Payload runs after analyst has stopped monitoring

### Why This Works

- **Silent exit**: No crash = no alert triggered
- **60-second delay**: Exceeds typical analysis timeout
- **Success exit code**: Package appears to work normally
- **Console clearing**: Hides evidence of execution

## GlassWorm Indicators

- **process.exit(0)** - Silent exit with success code
- **setTimeout with 60000ms** - 60-second delay
- **os.cpus().length < 2** - CPU detection
- **os.totalmem() < 2GB** - Memory detection
- **NODE_ENV check** - Environment detection
- **process.execArgv inspection** - Debug flag detection
- **Console clearing** - Evidence destruction

## Expected Detection

- **Detector:** Time Delay Sandbox Evasion (L3)
- **Severity:** Critical
- **Confidence:** 0.95
- **Category:** behavioral/silent_exit

- **Detector:** Time Delay Sandbox Evasion (L3)
- **Severity:** High
- **Confidence:** 0.90
- **Category:** behavioral/delayed_execution

## Source

Synthetic evidence created for Glassworks testing
Based on: https://codeberg.org/tip-o-deincognito/glassworm-writeup
