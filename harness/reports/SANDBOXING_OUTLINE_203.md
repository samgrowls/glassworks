# 🧪 Sandboxing Research — Sub-Second VM + Syscall Visibility

Now to your second request. You want to execute flagged packages safely with:
- <200ms startup
- Syscall tracing
- Network/filesystem isolation
- Free for development

---

## Current Landscape (2026)

### Option 1: Firecracker + eBPF (Recommended)

```
┌─────────────────────────────────────────┐
│  Firecracker MicroVM (125ms boot)       │
│  ┌─────────────────────────────────┐    │
│  │  Node.js/Python Runtime         │    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────┐
│  eBPF Tracing (Tracee/Tetragon)         │
│  - syscall: open, execve, connect       │
│  - network: outbound connections        │
│  - file: writes outside package dir     │
└─────────────────────────────────────────┘
```

**Pros:**
- Production-proven (AWS Lambda)
- Strong isolation
- Snapshot/restore → <100ms with prewarm
- eBPF gives syscall visibility

**Cons:**
- Requires root/CAP_BPF for eBPF
- More complex setup

**Free Tier:** Self-hosted (no cost)

---

### Option 2: gVisor (Easier Setup)

```
┌─────────────────────────────────────────┐
│  gVisor Sandbox (user-space kernel)     │
│  - syscall interception built-in        │
│  - network namespace isolation          │
└─────────────────────────────────────────┘
```

**Pros:**
- No root required (mostly)
- Built-in syscall tracing via `strace`-like output
- Kubernetes-native

**Cons:**
- Slower startup (~500ms)
- Incomplete syscall coverage
- Not true VM isolation

**Free Tier:** Self-hosted

---

### Option 3: WASM Sandbox (Fastest, Limited)

```
┌─────────────────────────────────────────┐
│  Wasmtime/Wasmer (<10ms startup)        │
│  - deterministic execution              │
│  - full syscall control                 │
└─────────────────────────────────────────┘
```

**Pros:**
- Fastest startup
- Perfect for JS analysis (via WASI)
- Easy instrumentation

**Cons:**
- Can't run native npm packages directly
- Need translation layer (e.g., Javy for JS→WASM)

**Free Tier:** Self-hosted

---

### Option 4: E2B (Closest to Your Need)

**E2B** provides ephemeral sandboxes with:
- ~500ms startup
- Built-in filesystem/network isolation
- Python/Node.js runtimes
- Free tier: 500 sandbox-hours/month

**Missing:** Deep syscall tracing (they provide file/network logs, not raw syscalls)

**URL:** https://e2b.dev

---

### Option 5: Tracee (Aqua Security) — eBPF Only

**Tracee** is eBPF-based runtime security:
- Syscall visibility
- File/network/process events
- Policy engine

**Use Case:** Run packages in Docker, attach Tracee externally.

**URL:** https://github.com/aquasecurity/tracee

---

## 🏆 Recommended Architecture for You

### Phase 1 (1-2 weeks, Free)

```bash
# Execution: Docker (familiar, easy)
docker run --rm -v ./pkg:/pkg node:20 npm install /pkg

# Instrumentation: Tracee (eBPF)
tracee-ebpf -e security_file_open -e security_socket_connect
```

**Goal:** Validate sandboxing value quickly.

---

### Phase 2 (2-4 weeks, Free)

```bash
# Execution: Firecracker with snapshot
firecracker --api-sock /tmp/firecracker.sock

# Pre-warmed snapshot with Node.js
# Boot time: ~100ms

# Instrumentation: eBPF (Tracee or custom)
```

**Goal:** Achieve sub-second execution.

---

### Phase 3 (Advanced)

```
┌──────────────────────────────────────────────┐
│  Your Rust Engine                            │
│  - Selects packages for sandbox              │
│  - Feeds static findings to guide execution  │
└──────────────────────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────┐
│  Firecracker + eBPF Stack                    │
│  - Executes only suspicious code paths       │
│  - Records syscall trace                     │
│  - Feeds back to risk_scorer                 │
└──────────────────────────────────────────────┘
```

**Differentiation:** Static + behavioral + runtime in one pipeline.

---

## 📋 Concrete Next Steps

### Week 1: Docker + Tracee POC
```bash
# 1. Install Tracee
docker pull aquasec/tracee:latest

# 2. Run flagged package in Docker
docker run --rm --name sandbox node:20 npm install <flagged-pkg>

# 3. Attach Tracee in parallel
docker run --rm --pid=container:sandbox aquasec/tracee:latest

# 4. Capture: exec, file writes, network
```

### Week 2-3: Firecracker Integration
```bash
# 1. Set up Firecracker with jailer
# 2. Create snapshot with Node.js pre-installed
# 3. Measure boot time (target: <200ms)
# 4. Integrate eBPF tracing
```

### Week 4: Feed Back to glassworks
```rust
// New module: glassware-core/src/sandbox.rs
pub struct SandboxResult {
    pub syscalls: Vec<SyscallEvent>,
    pub network_connections: Vec<Connection>,
    pub file_writes: Vec<FileWrite>,
    pub threat_score: f64,
}

// Integrate with existing risk_scorer
pub fn calculate_hybrid_risk(
    static_findings: &[Finding],
    sandbox_result: &SandboxResult,
) -> u32 { ... }
```

---

## 🎯 Bottom Line

### Code Review Summary

| Aspect | Status | Notes |
|--------|--------|-------|
| Architecture | ✅ Strong | Tiered detection, cross-file analysis |
| Detector System | ✅ Good | 17 detectors, but linear execution |
| Risk Scoring | ⚠️ Partial | Cumulative but missing context |
| LLM Integration | ✅ Good | First-class, not bolted-on |
| Performance | ✅ Good | Parallel + caching working |
| Testing | ✅ Good | 177 tests passing |

### Sandboxing Recommendation

**Start with:** Docker + Tracee (Phase 1)
**Target:** Firecracker + eBPF (Phase 2)
**Avoid:** Pure WASM (can't run npm packages directly)

**Closest off-the-shelf:** E2B (but lacks syscall depth)

---

