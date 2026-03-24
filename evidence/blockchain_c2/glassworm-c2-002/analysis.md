# Evidence Package: glassworm-c2-002

## Attack Pattern

This package demonstrates **Solana RPC polling for C2 communication**. It establishes a persistent connection to Solana's mainnet RPC endpoint and periodically polls for transactions containing commands.

### Attack Mechanism

1. **RPC Connection** - Connects to `api.mainnet-beta.solana.com` via HTTP and WebSocket
2. **Wallet Targeting** - Monitors specific wallet `C2_WALLET` for incoming transactions
3. **Polling Loop** - Uses `setInterval` with 3-minute interval (180000ms)
4. **Transaction Parsing** - Fetches and parses full transaction objects
5. **Inner Instruction Analysis** - Looks for hidden commands in inner instructions
6. **Memo Extraction** - Extracts commands from memo-type instructions

### Key Differences from glassworm-c2-001

- Uses WebSocket endpoint in addition to HTTP
- Includes retry logic with MAX_RETRIES
- Parses both main and inner instructions
- More robust error handling

## GlassWorm Indicators

- **api.mainnet-beta.solana.com** - Solana mainnet RPC URL
- **setInterval with 180000ms** - 3-minute polling interval
- **getSignaturesForAddress** - Wallet monitoring function
- **getTransaction** - Transaction fetching
- **innerInstructions** - Looking for hidden data
- **Memo instruction parsing** - Command extraction pattern

## Expected Detection

- **Detector:** Blockchain C2 Detection (L3)
- **Severity:** Critical
- **Confidence:** 0.90
- **Category:** blockchain_c2/solana_rpc

- **Detector:** Time Delay Sandbox Evasion (L3)
- **Severity:** High
- **Confidence:** 0.82
- **Category:** behavioral/polling

- **Detector:** RDD URL Dependency (L2)
- **Severity:** Medium
- **Confidence:** 0.78
- **Category:** external_dependency/solana

## Source

Synthetic evidence created for Glassworks testing
Based on: https://codeberg.org/tip-o-deincognito/glassworm-writeup
