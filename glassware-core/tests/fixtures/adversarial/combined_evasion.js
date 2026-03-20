// Adversarial Test Fixture: Combined Evasion Techniques
// Tests multiple evasion techniques applied simultaneously.

// Mutations applied:
// 1. VS-16 → VS-17 substitution
// 2. Variable renaming: decoder → _decoder
// 3. Encoding: base64 → custom encoding

const _decoder = (s) => {
    // Custom hex decoding instead of base64
    return Buffer.from(s, 'hex').toString();
};

const p\u{FE0E}ayload = _decoder('6d616c6963696f7573');
eval(payload);
