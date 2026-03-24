# Evidence Package: glassworm-c2-003

## Attack Pattern

This package demonstrates **transaction metadata parsing for C2 command extraction**. It specifically targets the `meta.innerInstructions` field of Solana transactions to find hidden commands.

### Attack Mechanism

1. **Transaction Fetching** - Uses `getTransaction` to retrieve full transaction objects
2. **Inner Instruction Parsing** - Extracts `meta.innerInstructions` from transaction metadata
3. **Command Pattern Matching** - Looks for:
   - Memo instructions with command data
   - Transfer instructions with encoded amounts
   - Custom program instructions
4. **Command Format** - Commands follow `CMD:ACTION:PARAMS` format in memo fields
5. **Action Types** - Supports EXFIL, EXEC, WAIT commands

### Metadata Fields Analyzed

- `tx.meta.innerInstructions` - Nested instruction data
- `instr.parsed.type` - Instruction type (memo, transfer, etc.)
- `instr.parsed.info.memo` - Memo content
- `instr.parsed.info.amount` - Transfer amount (potentially encoded)
- `instr.parsed.program` - Program identifier

## GlassWorm Indicators

- **getTransaction** - Solana transaction fetching
- **meta.innerInstructions** - Looking for hidden data in metadata
- **Memo instruction parsing** - Command extraction from memos
- **setInterval with 240000ms** - 4-minute polling interval
- **CMD:ACTION:PARAMS format** - Command parsing pattern
- **EXFIL/EXEC/WAIT actions** - Malicious command types

## Expected Detection

- **Detector:** Blockchain C2 Detection (L3)
- **Severity:** Critical
- **Confidence:** 0.93
- **Category:** blockchain_c2/metadata_parsing

- **Detector:** GlassWare Pattern Detection (L2)
- **Severity:** High
- **Confidence:** 0.85
- **Category:** command_parsing

- **Detector:** Time Delay Sandbox Evasion (L3)
- **Severity:** High
- **Confidence:** 0.80
- **Category:** behavioral/polling

## Source

Synthetic evidence created for Glassworks testing
Based on: https://codeberg.org/tip-o-deincognito/glassworm-writeup
