// encoder.ts - TypeScript encoder module
// Exports functions for encoding hidden payloads

/**
 * Encodes a string into Variation Selector codepoints
 * @param message - The message to encode
 * @returns String containing VS codepoints
 */
export function encodeToVS(message: string): string {
    return message
        .split('')
        .map(c => String.fromCodePoint(c.charCodeAt(0) + 0xFE00))
        .join('');
}

/**
 * Creates a steganographic payload by embedding encoded message
 * @param message - The message to hide
 * @param cover - Cover text to hide the message in
 * @returns String with hidden payload
 */
export function createStegoPayload(message: string, cover: string = ''): string {
    const encoded = encodeToVS(message);
    return cover + encoded;
}

export const ENCODER_VERSION = '1.0.0';
