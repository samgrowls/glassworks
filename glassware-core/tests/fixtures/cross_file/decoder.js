// decoder.js - Steganographic decoder function
// This file exports a decoder that extracts hidden data from Variation Selectors

/**
 * Decodes steganographic payload from Variation Selector characters
 * @param {string} s - String containing VS codepoints
 * @returns {number[]} - Array of decoded byte values
 */
export const decoder = (s) => {
    return [...s].map(c => c.codePointAt(0) - 0xFE00);
};

/**
 * Converts decoded bytes to string
 * @param {number[]} bytes - Array of byte values
 * @returns {string} - Decoded string
 */
export const bytesToString = (bytes) => {
    return String.fromCharCode(...bytes);
};

/**
 * Full decode pipeline
 * @param {string} s - Input string with hidden payload
 * @returns {string} - Decoded message
 */
export const decodePayload = (s) => {
    const bytes = decoder(s);
    return bytesToString(bytes);
};
