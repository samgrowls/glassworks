# Evidence Package: glassworm-c2-001

## Attack Pattern

This package demonstrates **blockchain-based Command & Control (C2) using Solana**. The attack uses the Solana blockchain as a decentralized C2 channel that is difficult to block or trace.

### Attack Mechanism

1. **Wallet Monitoring** - The package monitors a specific Solana wallet address (`MONITOR_WALLET`)
2. **Periodic Polling** - Uses `setInterval` with 5-minute interval (300000ms) to poll for new transactions
3. **Transaction Analysis** - Fetches transaction signatures using `getSignaturesForAddress`
4. **Command Extraction** - Parses transaction metadata for inner instructions containing commands
5. **Decentralized C2** - Commands are embedded in blockchain transactions, making them immutable and hard to trace

### Why Blockchain C2

- **Resilient** - No central server to take down
- **Anonymous** - Wallet addresses are pseudonymous
- **Legitimate Traffic** - Blockchain API calls look normal
- **Encrypted** - Transaction data can be encoded

## GlassWorm Indicators

- **getSignaturesForAddress** - Solana web3.js function for wallet monitoring
- **setInterval with 300000ms** - 5-minute polling interval (classic C2 pattern)
- **api.mainnet-beta.solana.com** - Solana mainnet RPC endpoint
- **innerInstructions parsing** - Looking for hidden data in transactions
- **Auto-start monitoring** - Begins C2 immediately on import

## Expected Detection

- **Detector:** Blockchain C2 Detection (L3)
- **Severity:** Critical
- **Confidence:** 0.92
- **Category:** blockchain_c2/solana

- **Detector:** Time Delay Sandbox Evasion (L3)
- **Severity:** High
- **Confidence:** 0.85
- **Category:** behavioral/polling

- **Detector:** RDD URL Dependency (L2)
- **Severity:** Medium
- **Confidence:** 0.75
- **Category:** external_dependency

## Source

Synthetic evidence created for Glassworks testing
Based on: https://codeberg.org/tip-o-deincognito/glassworm-writeup
