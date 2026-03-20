// Adversarial Test Fixture: Variable Renaming Evasion
// Tests detector resilience against decoder variable renaming.

// Original: const decoder = (s) => atob(s);
// Mutated:  const d3c0d3r = (s) => atob(s);
// Mutation: decoder → d3c0d3r (leet speak)
// Expected: Semantic detector should still identify decoder pattern

const d3c0d3r = (s) => atob(s);
const payload = d3c0d3r('bWFsaWNpb3Vz');
eval(payload);
