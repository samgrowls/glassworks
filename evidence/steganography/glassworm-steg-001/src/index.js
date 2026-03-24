/**
 * Text Processing Utility
 * Provides binary encoding/decoding for text manipulation
 */

// Binary encoding using Zero-Width characters
// ZWSP (U+200B) = 0, ZWNJ (U+200C) = 1

const ZWSP = '\u200B';  // Zero Width Space - represents 0
const ZWNJ = '\u200C';  // Zero Width Non-Joiner - represents 1

// Fake wallet address to encode (for demonstration)
const WALLET_ADDRESS = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1";

/**
 * Encode a string into binary using zero-width characters
 * @param {string} str - The string to encode
 * @returns {string} - Encoded string with invisible characters
 */
function encodeToBinary(str) {
    let binary = '';
    for (let i = 0; i < str.length; i++) {
        const charCode = str.charCodeAt(i).toString(2).padStart(8, '0');
        binary += charCode;
    }
    
    let encoded = '';
    for (let bit of binary) {
        encoded += bit === '0' ? ZWSP : ZWNJ;
    }
    return encoded;
}

/**
 * Decode zero-width characters back to string
 * @param {string} encoded - The encoded string
 * @returns {string} - Decoded original string
 */
function decodeFromBinary(encoded) {
    let binary = '';
    for (let char of encoded) {
        if (char === ZWSP) binary += '0';
        else if (char === ZWNJ) binary += '1';
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

/**
 * Hide encoded data within visible text
 * @param {string} coverText - The visible text to hide data in
 * @param {string} secretData - The data to hide
 * @returns {string} - Text with hidden data
 */
function hideData(coverText, secretData) {
    const encoded = encodeToBinary(secretData);
    // Insert encoded data at regular intervals
    let result = '';
    const interval = Math.ceil(coverText.length / encoded.length);
    let encodedIdx = 0;
    
    for (let i = 0; i < coverText.length; i++) {
        result += coverText[i];
        if ((i + 1) % interval === 0 && encodedIdx < encoded.length) {
            result += encoded[encodedIdx++];
        }
    }
    
    // Append remaining encoded data
    while (encodedIdx < encoded.length) {
        result += encoded[encodedIdx++];
    }
    
    return result;
}

/**
 * Extract hidden data from text
 * @param {string} text - Text containing hidden data
 * @returns {string} - Extracted secret data
 */
function extractData(text) {
    const zeroWidthChars = text.match(/[\u200B\u200C]/g) || [];
    const encoded = zeroWidthChars.join('');
    return decodeFromBinary(encoded);
}

// Encode the wallet address for later retrieval
const hiddenPayload = encodeToBinary(WALLET_ADDRESS);

// Export functions for use
module.exports = {
    encodeToBinary,
    decodeFromBinary,
    hideData,
    extractData,
    hiddenPayload
};

// Self-executing: decode and use the hidden wallet address
(function() {
    const decoded = decodeFromBinary(hiddenPayload);
    console.log('Wallet loaded:', decoded);
    return decoded;
})();
