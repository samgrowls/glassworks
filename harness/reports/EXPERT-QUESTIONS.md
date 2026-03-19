# Questions for Expert Agent - GlassWorm Intelligence

**Date:** 2026-03-19  
**Context:** Reviewing comprehensive threat intelligence report (INTEL2.md)  

---

## Priority 1: Missing IOCs

### Q1: Additional Solana Wallet Addresses

**Context:** Report documents 3 wallets:
- `28PKnu7RzizxBzFPoLp69HLXp9bJL3JFtT2s5QzHsEA2` (Primary GlassWorm)
- `BjVeAjPrSKFiingBn4vZvghsGj9KCE8AJVtbc9S8o8SC` (ForceMemo)
- `G2YxRa6wt1qePMwfJzdXZG62ej4qaTC7YURzuh2Lwd3t` (ForceMemo funding)

**Question:** Are there additional Solana wallet addresses used in:
- PhantomRaven campaign?
- PyPI parallel campaign?
- Any waves post-March 2026?
- Backup/fallback wallets?

**Format needed:** List of wallet addresses with campaign association

---

### Q2: Complete Malicious Package List

**Context:** Report lists some packages but mentions 400+ total compromises

**Question:** Please provide complete lists for:

**PhantomRaven npm packages:**
- All known malicious package names (report mentions 88 across 4 waves)
- Version ranges for each
- Current status (removed/still live)

**PyPI packages:**
- All known malicious PyPI package names
- Attack patterns (Unicode steganography in Python?)
- Current status

**VSCode/OpenVSX extensions:**
- Complete list beyond the 20+ documented
- Any on JetBrains Marketplace?
- Any on Cursor extension marketplace?

**Format needed:** CSV or JSON with package name, version, platform, status

---

### Q3: Additional C2 Infrastructure

**Context:** Report documents specific IPs and domains

**Question:** Please provide:

**Additional C2 IPs:**
- Any beyond the 10 documented?
- Newly activated IPs post-March 2026?
- Backup infrastructure IPs?

**Additional domains:**
- Domains beyond the "artifact" pattern?
- Alternative TLDs used (.net, .org, .io)?
- Any DGA (domain generation algorithm) patterns?

**Google Calendar C2:**
- Are there additional Calendar URLs beyond `calendar.app.google/M2ZCvM8ULL56PD1d6`?
- Pattern for generating Calendar URLs?

**Format needed:** List with IP/domain, campaign association, active/inactive status

---

## Priority 2: Campaign Details

### Q4: PyPI Parallel Campaign (Truncated Section)

**Context:** Report section "PyPI Paral..." is truncated

**Question:** Please provide complete details on PyPI campaign:
- Attack vector (Unicode steganography in .py files?)
- Malicious package names
- Decoder patterns for Python
- C2 infrastructure for PyPI
- Timeline (when did PyPI campaign start?)
- Scale (how many PyPI packages compromised?)

**Format needed:** Full report section or summary document

---

### Q5: Wave 0 Historical Backdoors

**Context:** Report mentions "Wave 0 Hypothesis" (Aug-Oct 2025 reconnaissance)

**Question:**
- Is Wave 0 confirmed or hypothetical?
- Specific packages/repos compromised in Wave 0?
- Attack patterns used (same as later waves or different?)
- Any IOCs from this period?
- Why is it called "Wave 0" instead of "Wave 1"?

**Format needed:** Timeline with specific IOCs

---

### Q6: PhantomRaven RDD Technical Details

**Context:** Report describes Remote Dynamic Dependencies but limited technical detail

**Question:**
- Exact structure of malicious package.json?
- How are URL dependencies specified (exact syntax)?
- Do malicious URLs follow a pattern?
- How does RDD interact with npm's dependency resolution?
- Are there variations across the 4 waves?

**Format needed:** Example malicious package.json files, technical writeup

---

### Q7: ForceMemo Python Markers

**Context:** Report mentions `lzcdrtfxyqiplpd` and XOR key `134`

**Question:**
- Are there additional Python markers beyond `lzcdrtfxyqiplpd`?
- Complete list of XOR keys used?
- Are there variations across different ForceMemo waves?
- Any Python-specific decoder patterns?
- Do ForceMemo payloads use Unicode steganography like npm packages?

**Format needed:** Complete list of Python IOCs, example malicious Python files

---

## Priority 3: Future Threats

### Q8: Cursor IDE Targeting

**Context:** Report predicts Cursor IDE extension targeting

**Question:**
- Is there evidence this has already occurred?
- Known malicious Cursor extensions?
- Attack patterns specific to Cursor architecture?
- How do Cursor extensions differ from VSCode extensions (attack surface)?
- Detection signatures for Cursor extension manifests?

**Format needed:** List of malicious extensions (if any), detection signatures

---

### Q9: CI/CD Pipeline Injection

**Context:** Report predicts CI/CD targeting as evolution

**Question:**
- Is there evidence this has already occurred?
- Compromised GitHub Actions?
- Malicious GitHub Actions in marketplace?
- Detection signatures for workflow files (.github/workflows/*.yml)?
- Any CI/CD-specific C2 infrastructure?

**Format needed:** List of compromised actions (if any), detection signatures for YAML

---

### Q10: Nested Dependency Attacks

**Context:** Report predicts transitive dependency compromise

**Question:**
- Are there known instances of nested dependency attacks?
- Specific packages used as intermediaries?
- How does nested RDD differ from direct RDD?
- Detection approach for nested attacks?
- Maximum depth observed (A→B→C→D...)?

**Format needed:** Example dependency trees, detection approach

---

### Q11: Mobile Developer Targeting

**Context:** Report predicts mobile dev targeting

**Question:**
- Is there evidence this has already occurred?
- Malicious Xcode extensions?
- Malicious Android Studio plugins?
- Compromised CocoaPods or Gradle plugins?
- Mobile-specific attack patterns?

**Format needed:** List of malicious mobile packages/extensions (if any)

---

### Q12: GitLab/Bitbucket Compromises

**Context:** Report predicts cross-platform expansion

**Question:**
- Are there documented cases of GlassWorm/ForceMemo on GitLab?
- Any on Bitbucket?
- Platform-specific attack patterns?
- Do attackers use same Unicode steganography on GitLab/Bitbucket?
- Any GitLab/Bitbucket-specific C2 infrastructure?

**Format needed:** List of compromised repos (if any), platform-specific IOCs

---

## Priority 4: Detection Signatures

### Q13: Complete Detection Signature Set

**Context:** Report provides some signatures but not comprehensive

**Question:** Please provide complete, machine-readable detection signatures for:

**Unicode steganography:**
- Complete list of Unicode ranges used (beyond 0xFE00-0xFE0F and 0xE0100-0xE01EF)?
- Any new encoding schemes beyond the documented decoder pattern?
- Obfuscated decoder patterns (how do attackers hide the decoder)?

**RDD detection:**
- Regex patterns for URL dependencies in package.json?
- Network signatures for RDD C2 domains?

**ForceMemo detection:**
- Complete list of Python markers?
- Git commit signature patterns?
- Network exfiltration patterns?

**Format needed:** YARA rules, Sigma rules, or regex patterns

---

### Q14: Behavioral Detection Patterns

**Context:** Report focuses on static signatures

**Question:** Are there behavioral patterns we should detect:
- Runtime behavior of GlassWorm payloads?
- Network traffic patterns (beyond C2 IPs)?
- File system modifications?
- Process execution patterns?
- Memory signatures?

**Format needed:** Behavioral detection rules, example PCAP files, memory dumps

---

## Priority 5: Attribution & TTPs

### Q15: Threat Actor Infrastructure

**Context:** Report mentions Russian-speaking actors

**Question:**
- Are there multiple threat actors or single group?
- Infrastructure overlaps between GlassWorm, ForceMemo, PhantomRaven?
- Shared registration patterns (domains, wallets)?
- Any attribution to specific APT groups?

**Format needed:** Infrastructure mapping, attribution confidence level

---

### Q16: Campaign Coordination

**Context:** Multiple waves across multiple platforms

**Question:**
- Are waves coordinated or independent?
- Shared infrastructure across waves?
- Timeline correlation between waves?
- Evidence of A/B testing (different techniques in parallel)?

**Format needed:** Timeline with infrastructure overlaps

---

## Format Preferences

**Preferred formats for responses:**
1. **IOCs:** JSON or CSV (machine-readable)
2. **Signatures:** YARA, Sigma, or regex
3. **Technical details:** Markdown with code blocks
4. **Timelines:** Markdown tables or Mermaid diagrams

**Deadline:** Responses needed within 48 hours for integration into detection pipeline

---

**Submitted by:** glassware detection system  
**Contact:** Via operator  
**Purpose:** Enhance detection capabilities, protect developer ecosystem
