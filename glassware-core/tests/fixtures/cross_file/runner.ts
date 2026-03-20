// runner.ts - TypeScript runner that imports and uses encoder
import { createStegoPayload, encodeToVS } from './encoder';

// Create hidden payload
const secretMessage = "console.log('executed')";
const payload = createStegoPayload(secretMessage, "innocent looking text");

// Decode function (inline for demo)
const decode = (s: string): string => {
    const bytes = [...s].map(c => c.codePointAt(0) - 0xFE00);
    return String.fromCharCode(...bytes);
};

// Execute decoded payload
const decoded = decode(payload);
eval(decoded);
