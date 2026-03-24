/**
 * Combined Steganography Utility
 * 
 * This module combines multiple steganographic techniques:
 * 1. Zero-width binary encoding (ZWSP/ZWNJ)
 * 2. Package.json metadata hiding
 * 3. Base64 in comments with invisible chars
 * 4. Combined extraction/decoding
 * 
 * Hidden payload: M\u200BH\u200Cx\u200B3\u200CN\u200BD... (base64 + zero-width)
 * Decoded: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1
 */

const fs = require('fs');
const path = require('path');

// Zero-width character constants
const ZWSP = '\u200B';  // Binary 0
const ZWNJ = '\u200C';  // Binary 1

/**
 * Encode string to zero-width binary
 */
function encodeToBinary(str) {
    let binary = '';
    for (let i = 0; i < str.length; i++) {
        binary += str.charCodeAt(i).toString(2).padStart(8, '0');
    }
    return binary.split('').map(b => b === '0' ? ZWSP : ZWNJ).join('');
}

/**
 * Decode zero-width binary to string
 */
function decodeFromBinary(encoded) {
    let binary = '';
    for (let char of encoded) {
        binary += char === ZWSP ? '0' : '1';
    }
    let decoded = '';
    for (let i = 0; i < binary.length; i += 8) {
        decoded += String.fromCharCode(parseInt(binary.slice(i, i + 8), 2));
    }
    return decoded;
}

/**
 * Extract zero-width chars from package.json
 */
function extractFromPackageJson() {
    const pkgPath = path.join(__dirname, '..', 'package.json');
    const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));
    
    const fields = [pkg.description, pkg.author, ...(pkg.keywords || [])];
    const combined = fields.join('');
    return combined.match(/[\u200B\u200C]/g)?.join('') || '';
}

/**
 * Extract base64 from comments with zero-width chars
 */
function extractFromComments(source) {
    const commentRegex = /\/\*[\s\S]*?\*\//g;
    const comments = source.match(commentRegex) || [];
    
    for (const comment of comments) {
        const clean = comment.replace(/[\u200B\u200C]/g, '');
        const base64Match = clean.match(/[A-Za-z0-9+/=]{20,}/);
        if (base64Match) {
            try {
                const decoded = Buffer.from(base64Match[0], 'base64').toString('utf8');
                if (decoded.startsWith('0x')) return decoded;
            } catch (e) {}
        }
    }
    return null;
}

/**
 * Extract hidden payload from package.json _steg_payload field
 */
function extractFromHiddenField() {
    const pkgPath = path.join(__dirname, '..', 'package.json');
    const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));
    
    if (pkg._steg_payload) {
        return Buffer.from(pkg._steg_payload, 'base64').toString('utf8');
    }
    return null;
}

/**
 * Combined extraction - tries all methods
 */
function extractAllPayloads() {
    const payloads = {};
    
    // Method 1: Hidden field
    payloads.hiddenField = extractFromHiddenField();
    
    // Method 2: Zero-width in package.json
    const zwData = extractFromPackageJson();
    if (zwData.length > 0) {
        payloads.zeroWidth = decodeFromBinary(zwData);
    }
    
    // Method 3: Comments
    const source = fs.readFileSync(__filename, 'utf8');
    payloads.comments = extractFromComments(source);
    
    return payloads;
}

/**
 * Create combined steganographic payload
 */
function createCombinedPayload(secret) {
    return {
        binary: encodeToBinary(secret),
        base64: Buffer.from(secret).toString('base64'),
        base64WithZW: Buffer.from(secret).toString('base64').split('').map((c, i) => 
            c + (i % 2 === 0 ? ZWSP : ZWNJ)
        ).join('')
    };
}

// Export functions
module.exports = {
    encodeToBinary,
    decodeFromBinary,
    extractFromPackageJson,
    extractFromComments,
    extractFromHiddenField,
    extractAllPayloads,
    createCombinedPayload
};

// Auto-execute and display all extracted payloads
console.log('=== Combined Steganography Extraction ===');
const allPayloads = extractAllPayloads();
for (const [method, payload] of Object.entries(allPayloads)) {
    if (payload) {
        console.log(`${method}: ${payload}`);
    }
}
