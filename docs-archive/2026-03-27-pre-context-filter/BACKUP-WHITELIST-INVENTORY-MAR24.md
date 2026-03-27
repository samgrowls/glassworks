# Whitelist Inventory - Backup (March 24, 2026)

**Purpose:** Document all whitelist entries before removal for audit trail.

**Created:** 2026-03-24
**Action:** Phase 1 - Emergency Whitelist Removal

---

## Current Dangerous Whitelist Entries

### wave10-1000plus.toml

**packages:**
```
moment, moment-timezone, date-fns, dayjs, i18next, react-intl, intl,
lodash, underscore, express, globalize, prettier, typescript, eslint,
@babel/core, @babel/*, validator, validate
```

**crypto_packages:**
```
ethers, web3, viem, wagmi, @solana/web3.js, bitcoinjs-lib,
hdkey, @metamask/*, node-forge, crypto-js, crypto, bcrypt, bcryptjs,
argon2, pbkdf2, scrypt, firebase, firebase-admin, twilio, vonage
```

**build_tools:**
```
webpack, webpack-, vite, rollup, esbuild, parcel, gulp, grunt,
core-js, babel, postcss, autoprefixer, metro, metro-,
prisma, @prisma/*
```

### wave11-evidence-validation.toml

Same structure as wave10.

### wave12-5000pkg.toml

Same structure as wave10.

### wave8-expanded-hunt.toml

Similar structure with additional entries.

### wave9-500plus.toml

Similar structure.

---

## Dangerous Entries to Remove

### HIGH PRIORITY (Attack Targets)

These are **exactly** the packages attackers target:

**UI Frameworks:**
- `ant-design-vue` (NOT in current configs but mentioned in handoff)
- `element-plus` (NOT in current configs but mentioned in handoff)
- `vuetify` (NOT in current configs but mentioned in handoff)
- `quasar` (NOT in current configs but mentioned in handoff)
- `naive-ui` (NOT in current configs but mentioned in handoff)

**Build Tools (DANGEROUS TO WHITELIST):**
- `webpack`, `webpack-*` - Build tool, high-value target
- `vite` - Build tool, high-value target
- `rollup` - Build tool, high-value target
- `esbuild` - Build tool, high-value target
- `parcel` - Build tool, high-value target
- `gulp` - Build tool, high-value target
- `grunt` - Build tool, high-value target
- `@babel/core`, `@babel/*` - Build tool, high-value target
- `babel` - Build tool, high-value target
- `postcss` - Build tool, high-value target
- `metro`, `metro-*` - Build tool, high-value target
- `prisma`, `@prisma/*` - Build tool, high-value target

**Cloud SDKs (DANGEROUS TO WHITELIST):**
- `firebase`, `firebase-admin` - Cloud SDK, can be compromised
- `@azure/*` - Cloud SDK (in blockchain detector whitelist)
- `@microsoft/*` - Cloud SDK (in blockchain detector whitelist)
- `@aws-sdk/*` - Cloud SDK (in blockchain detector whitelist)
- `@google-cloud/*` - Cloud SDK (in blockchain detector whitelist)

**Core Infrastructure:**
- `express` - Web framework, high-value target
- `lodash` - Utility library, high-value target
- `underscore` - Utility library, high-value target
- `typescript` - Compiler, high-value target
- `prettier` - Code formatter, high-value target
- `eslint` - Linter, high-value target

---

## Entries to KEEP (Legitimate Use Cases)

### i18n Libraries (Legitimate Unicode Use)

These packages legitimately use Unicode for internationalization:

```
moment, moment-timezone, date-fns, dayjs, i18next, react-intl, intl, globalize
```

**Rationale:** These packages deal with locale data, timezone names, and date formatting that naturally contains Unicode characters including some invisible separators.

### Crypto Libraries (Legitimate Blockchain API Use)

These packages legitimately use blockchain APIs:

```
ethers, web3, viem, wagmi, @solana/web3.js, bitcoinjs-lib, hdkey, @metamask/*
```

**Rationale:** These are purpose-built for blockchain interaction. However, known C2 wallets/IPs should STILL be flagged.

### State Management (To Be Removed from Scanner Logic)

```
(No entries currently in use)
```

---

## Action Plan

### Step 1: Remove build_tools whitelist entirely
All entries in `build_tools` should be removed. Detectors should use context-aware logic instead.

### Step 2: Trim crypto_packages
Keep only core crypto libraries:
```
ethers, web3, viem, wagmi, @solana/web3.js, bitcoinjs-lib, hdkey, @metamask/*
```

Remove:
```
node-forge, crypto-js, crypto, bcrypt, bcryptjs, argon2, pbkdf2, scrypt,
firebase, firebase-admin, twilio, vonage
```

### Step 3: Trim packages
Keep only i18n libraries:
```
moment, moment-timezone, date-fns, dayjs, i18next, react-intl, intl, globalize
```

Remove:
```
lodash, underscore, express, validator, validate, prettier, typescript, eslint, @babel/core, @babel/*
```

### Step 4: Modify scanner.rs
Change `is_package_whitelisted()` to always return `false` (no package-level whitelisting).

### Step 5: Fix detectors
- TimeDelay: Remove build tool skip logic
- BlockchainC2: Remove crypto package skip logic (keep known C2 checks)
- InvisibleChar: Remove /dist/, /lib/ directory skips

---

## Audit Trail

**Backup Created:** 2026-03-24
**Files Modified:**
- campaigns/wave10-1000plus.toml
- campaigns/wave11-evidence-validation.toml
- campaigns/wave12-5000pkg.toml
- campaigns/wave8-expanded-hunt.toml
- campaigns/wave9-500plus.toml
- glassware/src/scanner.rs

**Git Tag:** v0.30.0-fp-eliminated (current)
**Next Tag:** v0.31.0-whitelist-removed
