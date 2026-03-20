// utils.js - Utility functions with crypto operations
const crypto = require('crypto');

/**
 * Decrypts data using AES-256-CBC
 * @param {string} encrypted - Base64 encoded encrypted data
 * @param {string} key - Decryption key
 * @returns {string} - Decrypted plaintext
 */
const decrypt = (encrypted, key) => {
    const decipher = crypto.createDecipheriv('aes-256-cbc', key, '0000000000000000');
    let decrypted = decipher.update(encrypted, 'base64', 'utf8');
    decrypted += decipher.final('utf8');
    return decrypted;
};

/**
 * Extracts command from HTTP response headers
 * @param {Object} headers - HTTP response headers
 * @returns {string} - Extracted command
 */
const extractFromHeader = (headers) => {
    return headers['x-command'] || headers['x-data'];
};

module.exports = {
    decrypt,
    extractFromHeader,
};
