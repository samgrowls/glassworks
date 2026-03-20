// Adversarial Test Fixture: Encoding Variation Evasion
// Tests Base64 → Hex encoding substitution.

// Original: const data = Buffer.from('base64');
// Mutated:  const data = Buffer.from('hex');
// Mutation: Base64 encoding → Hex encoding reference
// Expected: Should be flagged by EncryptedPayloadDetector

const data = Buffer.from('hex');
const payload = data.toString();
eval(payload);
