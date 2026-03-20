// Adversarial Test Fixture: Pipe Delimiter Evasion (npm variant)
// Tests VS codepoints after pipe delimiter with variation.

// Original: module.exports = function|...VS-16 run...|() {}
// Mutated:  module.exports = function|...VS-17 run...|() {}
// Mutation: VS-16 → VS-17 in pipe-delimited payload
// Expected: Should still be detected by PipeDelimiterStego detector

module.exports = function|\u{FE0E}\u{FE0E}\u{FE0E}\u{FE0E}\u{FE0E}|() {};
