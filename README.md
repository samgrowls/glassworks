# glassworm

**One binary. Zero dependencies. Finds invisible attacks hiding in your code.**

glassworm detects steganographic payloads, invisible Unicode characters, and
bidirectional text attacks in source code. Built in Rust. Ships as a single
binary under 2MB.

## Why

In March 2026, the GlassWorm campaign compromised 72+ VS Code extensions and
150+ GitHub repositories using invisible Unicode characters to hide malicious
payloads in plain sight. The code looks normal in your editor. The payload is
invisible. glassworm makes it visible.
$ glassworm project/

⚠ CRITICAL project/preinstall.js:47 Steganographic payload detected Hidden: 8,412 invisible codepoints → 8,412 bytes decoded Entropy: 7.98 bits/byte — encrypted or compressed payload

⚠ CRITICAL project/index.js:23 Steganographic payload detected Hidden: 1,247 invisible codepoints → 1,247 bytes decoded Entropy: 4.72 bits/byte — plaintext code

┌─ Decoded payload ──────────────────────────────────┐ │ const https = require('https'); │ │ const os = require('os'); │ │ const data = JSON.stringify({ │ │ hostname: os.hostname(), │ │ platform: os.platform(), │ │ ... │ │ (1,247 bytes total — showing first 512) │ └─────────────────────────────────────────────────────┘

⚠ HIGH project/utils.js:12 GlassWorm decoder function detected Pattern: codePointAt with variation selector range constants (0xFE00)

3 findings in 847 files (0.12s)

## Install **From source (recommended):** ```bash cargo install glassworm-cli
From binary release:

\# macOS / Linux
curl -sSL https://github.com/PropertySightlines/glassworm/releases/latest/download/glassworm-$(uname -s)-$(uname -m) -o glassworm
chmod +x glassworm
Verify checksums against the SHA256 values listed in each release.

Usage
\# Scan a directory
glassworm .

\# Scan specific files
glassworm src/index.js package.json

\# JSON output
glassworm --format json .

\# SARIF output (GitHub Advanced Security)
glassworm --format sarif . > results.sarif

\# Only critical/high findings
glassworm --severity high .

\# Silent mode — exit code only
glassworm --quiet .
What It Detects
Detection	Severity	Description
VS Stego Payload	Critical	Dense runs of Unicode Variation Selectors encoding hidden data (GlassWorm technique)
Decoder Function	High	codePointAt + 0xFE00/0xE0100 patterns matching GlassWorm decoder logic
Bidi Override	Critical	Bidirectional text overrides that reorder displayed code (Trojan Source)
Zero-Width Characters	Medium–High	ZWSP, ZWNJ, ZWJ, word joiners in code contexts
Homoglyphs	Medium–High	Mixed-script identifiers using Cyrillic/Greek lookalikes
Tag Characters	High	Unicode tag characters (U+E0001–U+E007F) in source files
Decoded Payload Display
When glassworm finds a steganographic payload, it doesn't just flag it — it decodes and displays what's hidden. For unencrypted payloads, you see the actual malicious code. For encrypted payloads, you see byte count, entropy score, and a hex preview.

CI Integration
GitHub Actions
name: glassworm
on: [push, pull_request]

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install glassworm
        run: cargo install glassworm-cli
      - name: Scan
        run: glassworm .
Exit code 1 on findings, 0 when clean.

SARIF Upload (GitHub Advanced Security)
- name: Scan (SARIF)
        run: glassworm --format sarif . > results.sarif
        continue-on-error: true
      - uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: results.sarif
Findings appear directly in the Security tab and as PR annotations.

Exit Codes
Code	Meaning
0	No findings at or above severity threshold
1	Findings detected
2	Error (file not found, permission denied)
How It Works
glassworm scans source files character by character, classifying each Unicode codepoint. When it finds suspicious characters — invisible formatters, bidi overrides, variation selectors, tag characters — it evaluates density, context (comment, string, identifier), and file type to determine severity.

For steganographic payloads, glassworm reverses the encoding: Unicode Variation Selectors (U+FE00–U+FE0F, U+E0100–U+E01EF) map to byte values 0x00–0xFF via a simple substitution cipher. The resulting bytes are analyzed for entropy. High entropy (>7.0 bits/byte) indicates encrypted content. Low entropy with valid UTF-8 indicates readable code — which glassworm displays directly.

No network calls. No config files. No dependencies beyond the binary itself.

Scanning VS Code Extensions
VS Code extensions (.vsix files) are zip archives:

unzip suspicious.vsix -d temp/
glassworm temp/
rm -rf temp/
The GlassWorm Campaign
glassworm is named after the GlassWorm threat campaign, active since October 2025. The campaign uses invisible Unicode steganography to hide malicious payloads in JavaScript/TypeScript files across npm packages, GitHub repositories, and VS Code extensions.

For detailed technical analysis, see the glassworm writeup.

Comparison
Feature	glassworm	anti-trojan-source
Language	Rust (single binary)	JavaScript (requires Node.js)
Stego decoding	✅ Decodes + displays payload	❌
Entropy analysis	✅	❌
SARIF output	✅	❌
Bidi detection	✅	✅
Zero-width detection	✅	✅
Homoglyphs	✅	❌
Install	cargo install / single binary	npm install
Related
Coax — Full code trust scanner. Secrets detection, Unicode attacks, entropy analysis, verification. glassworm's detection engine originated from Coax.

License
MIT License Copyright (c) 2026 glassworm contributors Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions: The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software. THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.