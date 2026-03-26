# GlassWorm Compilation Debug Prompt

```markdown
# GlassWorm Binary Compilation Debug - Expert Investigation

## Context
We have a Rust project (glassworks) where code changes are NOT reflecting in the compiled binary output, despite:
- Source files being modified correctly
- Binary being rebuilt (different hashes confirmed)
- Old strings NOT present in new binary (verified via strings command)
- Output messages remaining IDENTICAL

## Current State
- Branch: `v0.57.0-glassworm-fix-attempt`
- Binary: `./target/release/glassware`
- Expected: GlassWorm detector requires BOTH invisible chars + decoder
- Actual: Still showing "decoder_pattern" messages, flagging firebase incorrectly

## Investigation Tasks

### Phase 1: Build System Audit
```bash
# Clean everything and rebuild from scratch
cargo clean
rm -rf target/
cargo build --release -p glassware --verbose 2>&1 | tee build.log

# Check which files are actually being compiled
grep -r "Compiling glassware" build.log

# Verify binary path
ls -la target/release/glassware
file target/release/glassware
```

### Phase 2: String Verification in Binary
```bash
# Search for OLD strings (should NOT exist)
strings target/release/glassware | grep -i "decoder_pattern"
strings target/release/glassware | grep -i "GlassWare attack"

# Search for NEW strings (should exist)
strings target/release/glassware | grep -i "invisible chars"
strings target/release/glassware | grep -i "GlassWorm steganography"

# Get binary hash for verification
sha256sum target/release/glassware
```

### Phase 3: Source Code Verification
```bash
# Find ALL files containing the old message
grep -rn "decoder_pattern" src/ --include="*.rs"
grep -rn "GlassWare attack" src/ --include="*.rs"

# Find ALL files containing detector logic
grep -rn "detect_glassware" src/ --include="*.rs"
grep -rn "GlassWorm" src/ --include="*.rs"

# Check for duplicate detector implementations
find src/ -name "*.rs" -exec grep -l "decoder" {} \;
```

### Phase 4: Check for Multiple Binaries/Targets
```bash
# List all binary targets
grep -r "^\[\[bin\]\]" Cargo.toml
ls -la src/bin/

# Check if there are multiple glassware binaries
find target/release -name "*glass*" -type f

# Verify which binary we're actually running
./target/release/glassware --version 2>/dev/null || echo "No version flag"
```

### Phase 5: Check Build Scripts and Features
```bash
# Check for build.rs
cat build.rs 2>/dev/null || echo "No build.rs"

# Check Cargo.toml features
grep -A 20 "\[features\]" Cargo.toml

# Check for conditional compilation
grep -rn "#\[cfg" src/ --include="*.rs" | head -30
```

### Phase 6: Runtime Debugging
```bash
# Add debug output to trace execution path
# Temporarily modify the detector to print which code path runs

# Run with RUST_LOG
RUST_LOG=debug ./target/release/glassware scan-npm firebase@10.7.2 2>&1 | head -100

# Check if there's config caching
find . -name "*.json" -o -name "*.toml" -o -name "*.yaml" | grep -v target | grep -v node_modules
```

### Phase 7: Check for Cached/Generated Code
```bash
# Check for generated source files
find src/ -name "*.rs" -newer Cargo.toml 2>/dev/null

# Check for build-cache directories
find . -name ".cache" -o -name "cache" -o -name "*.cache" | grep -v target | grep -v node_modules

# Check if there's a pre-built binary being used
which glassware
alias glassware
```

### Phase 8: Dependency Check
```bash
# Check if glassware is a dependency being pulled from elsewhere
grep -r "glassware" Cargo.toml

# Check workspace members
grep -A 10 "\[workspace\]" Cargo.toml

# Verify we're building the right crate
cargo metadata --format-version 1 | jq '.packages[] | select(.name == "glassware") | .targets'
```

## Critical Questions to Answer

1. **Is there a build.rs that's generating code?**
2. **Are there multiple binary targets with similar names?**
3. **Is there a feature flag that's enabling old code paths?**
4. **Is the binary being executed from a different location than we think?**
5. **Is there runtime configuration overriding compile-time logic?**
6. **Are there macro expansions generating the old strings?**
7. **Is there a separate detector module that wasn't updated?**

## Expected Deliverables

1. **Build Audit Report** - Which files are compiled, in what order
2. **String Map** - Where each output message originates in source
3. **Binary Verification** - Confirmation of which binary is being executed
4. **Root Cause** - Why old output persists despite new code
5. **Fix Plan** - Specific steps to resolve the issue

## Immediate Action Items

1. Run `cargo clean` and full rebuild with verbose output
2. Grep entire codebase for "decoder_pattern" string literal
3. Verify the exact binary being executed (path, hash, timestamps)
4. Check for any runtime config files that might override detection logic
5. Look for duplicate detector implementations across modules

## Documentation to Review

- `GLASSWORM-FIX-STATUS.md` - Full investigation details
- `HONEST-ASSESSMENT.md` - Root cause analysis
- `CRITICAL-FIXES-APPLIED.md` - What was supposedly fixed

## Success Criteria

After fix:
- `./target/release/glassware scan-npm firebase@10.7.2` should NOT flag firebase
- Output should show new message format if GlassWorm is detected
- Binary strings should match source code exactly
```

---

## Quick Diagnostic Commands (Run These First)

```bash
# 1. Complete clean rebuild
cd glassworks
cargo clean && cargo build --release -p glassware --verbose 2>&1 | tail -50

# 2. Verify binary content
strings target/release/glassware | grep -E "(decoder_pattern|invisible|GlassWorm)" | head -20

# 3. Find source of old messages
grep -rn "decoder_pattern\|GlassWare attack" src/ --include="*.rs"

# 4. Check what's actually being executed
which glassware
ls -la target/release/glassware
./target/release/glassware scan-npm firebase@10.7.2 2>&1 | head -30
```

---

## Most Likely Causes (Ranked)

1. **Cargo cache not cleared** - `cargo clean` not run before rebuild
2. **Wrong binary target** - Multiple `[[bin]]` entries in Cargo.toml
3. **Build script generating code** - `build.rs` creating old strings
4. **Feature flags** - Old code path enabled via features
5. **Runtime config** - JSON/TOML config overriding compile logic
6. **Macro expansion** - Proc macros generating old strings
7. **Separate crate** - glassware pulled as dependency, not local build
8. **Path issue** - Executing different binary than the one built