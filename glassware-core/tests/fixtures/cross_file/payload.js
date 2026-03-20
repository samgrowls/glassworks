// payload.js - Payload execution file
// This file imports the decoder and executes the decoded payload

import { decoder, decodePayload } from './decoder.js';

// Hidden payload using Variation Selectors (U+FE00-U+FE0F)
// Encoded message: "malicious"
const hiddenPayload = "test\u{FE00}\u{FE01}\u{FE02}\u{FE03}\u{FE04}\u{FE05}\u{FE06}\u{FE07}\u{FE08}data";

// Decode and execute the payload
const decoded = decodePayload(hiddenPayload);
eval(decoded);

// Alternative: direct decoder usage
const result = decoder(hiddenPayload);
Function(result)();
