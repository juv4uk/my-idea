/**
 * Test Suite for My-Idea IDE Visualizer Prototypes
 * Tests Derivation DAG integrity, Canonical Hashes, PVC-16 Vector math, and 64-bit Pratyāhāras.
 */

import { DERIVATION_FIXTURES } from './fixtures.js';
import { computeCanonicalStatePayload } from './dag_visualizer.js';
import { PVC16_BITS, PVC16_PRESETS, CANONICAL_64_SOUNDS, PRATYAHARAS_64 } from './bitmask_inspector.js';

let passed = 0;
let failed = 0;

function assert(condition, message) {
  if (condition) {
    console.log(`  ✓ PASS: ${message}`);
    passed++;
  } else {
    console.error(`  ✗ FAIL: ${message}`);
    failed++;
  }
}

function assertEqual(actual, expected, message) {
  if (actual === expected) {
    console.log(`  ✓ PASS: ${message}`);
    passed++;
  } else {
    console.error(`  ✗ FAIL: ${message} | Expected: ${expected}, Got: ${actual}`);
    failed++;
  }
}

console.log("================================================================================");
console.log("MY-IDEA VISUAL TOOLING TEST SUITE: DERIVATION DAG & PHONEME BITMASKS");
console.log("================================================================================\n");

// -----------------------------------------------------------------------------
// 1. DERIVATION IR FIXTURES TESTS
// -----------------------------------------------------------------------------
console.log("--- 1. Derivation IR Fixtures & State Chains ---");

const bhavati = DERIVATION_FIXTURES.bhavati;
assert(Boolean(bhavati), "bhavati derivation fixture exists");
assertEqual(bhavati.derivation_id, "drv:canonical:bhavati-v0.1", "bhavati ID matches canonical schema");
assertEqual(bhavati.states.length, 9, "bhavati contains exactly 9 derivation states (S0..S8)");
assertEqual(bhavati.states[8].terms[0].surface_form, "Bavati", "Final state term surface is 'Bavati'");
assertEqual(bhavati.rules.length, 8, "bhavati has 8 applied Aṣṭādhyāyī rules");

const dadati = DERIVATION_FIXTURES.dadati;
assert(Boolean(dadati), "dadAti derivation fixture exists");
assertEqual(dadati.derivation_id, "drv:canonical:dadati-v0.1", "dadAti ID matches canonical schema");
assertEqual(dadati.states.length, 10, "dadAti contains exactly 10 derivation states (S0..S9)");
assertEqual(dadati.states[9].terms[0].surface_form, "dadAti", "Final state term surface is 'dadAti'");
assertEqual(dadati.rules.length, 10, "dadAti has 10 applied Aṣṭādhyāyī rules");

// Test canonical serialization stability
const s0Payload = computeCanonicalStatePayload(bhavati.states[0]);
assert(s0Payload.includes('"schema":"panini-state/0.1"'), "Canonical payload contains schema");
assert(s0Payload.includes('"terms":["term:root-BU"]'), "Canonical payload lists terms correctly");

// -----------------------------------------------------------------------------
// 2. 16-BIT PVC-16 VECTOR ENCODING & ARITHMETIC TESTS
// -----------------------------------------------------------------------------
console.log("\n--- 2. 16-Bit PVC-16 Hardware Vector Encoding ---");

const soundA = PVC16_PRESETS["a"].code;
const soundBigA = PVC16_PRESETS["A"].code;
const soundI = PVC16_PRESETS["i"].code;
const soundK = PVC16_PRESETS["k"].code;

assert(Boolean(soundA & PVC16_BITS.VOWEL), "Sound 'a' has VOWEL bit [0] set");
assert(!Boolean(soundK & PVC16_BITS.VOWEL), "Sound 'k' has VOWEL bit [0] cleared (Consonant)");

// Test Sthāna place of articulation
const sthanaA = (soundA >> 1) & 0x1F;
const sthanaBigA = (soundBigA >> 1) & 0x1F;
const sthanaI = (soundI >> 1) & 0x1F;
assertEqual(sthanaA, 1, "'a' is Kaṇṭhya (1)");
assertEqual(sthanaBigA, 1, "'ā' is Kaṇṭhya (1)");
assertEqual(sthanaI, 2, "'i' is Tālavya (2)");

// Test Savarṇa (1.1.9)
const isSavarna_a_A = (sthanaA === sthanaBigA) && 
                      (Boolean(soundA & PVC16_BITS.PRAYATNA_SPRSTA) === Boolean(soundBigA & PVC16_BITS.PRAYATNA_SPRSTA)) &&
                      (Boolean(soundA & PVC16_BITS.VOWEL) === Boolean(soundBigA & PVC16_BITS.VOWEL));
assert(isSavarna_a_A, "'a' and 'ā' are Savarṇa (homorganic vowels)");

const isSavarna_a_i = (sthanaA === sthanaI);
assert(!isSavarna_a_i, "'a' and 'i' are NOT Savarṇa (different Sthāna)");

// Test Palatalization Modifier (Ukrainian [ь])
const softT = PVC16_PRESETS["uk_t_soft"].code;
assert(Boolean(softT & PVC16_BITS.MOD_PALATALIZED), "Ukrainian [т'] has MOD_PALATALIZED bit 14 set");
assertEqual((softT >> 1) & 0x1F, 4, "Ukrainian [т'] retains Dantya (Dental 4) place of articulation");

// -----------------------------------------------------------------------------
// 3. 64-BIT PRATYĀHĀRA BITMASK ENGINE TESTS
// -----------------------------------------------------------------------------
console.log("\n--- 3. 64-Bit Pratyāhāra Bitmask Engine ---");

assertEqual(CANONICAL_64_SOUNDS.length, 42, "Universe contains exactly 42 canonical Śiva Sūtra phonemes");

function computeMask(bitIndices) {
  let mask = 0n;
  bitIndices.forEach(b => { mask |= (1n << BigInt(b)); });
  return mask;
}

const maskAc = computeMask(PRATYAHARAS_64["ac"].bits);
const maskHal = computeMask(PRATYAHARAS_64["hal"].bits);
const maskAl = computeMask(PRATYAHARAS_64["al"].bits);
const maskIk = computeMask(PRATYAHARAS_64["ik"].bits);
const maskYan = computeMask(PRATYAHARAS_64["yaR"].bits);

// Disjoint partition: ac ∩ hal == 0
assertEqual((maskAc & maskHal), 0n, "ac ∩ hal == ∅ (Vowels and Consonants are disjoint)");

// Complete union: ac ∪ hal == al
assertEqual((maskAc | maskHal), maskAl, "ac ∪ hal == al (Complete phoneme universe)");

// Subset inclusion: ik ⊆ ac
assertEqual((maskIk & ~maskAc), 0n, "ik ⊆ ac (Closed vowels are a strict subset of vowels)");

// Semivowel intersection: yaṇ ∩ hal == yaṇ
assertEqual((maskYan & maskHal), maskYan, "yaṇ ⊆ hal (Semivowels are consonants)");

console.log("\n================================================================================");
console.log(`TEST SUMMARY: ${passed} Passed, ${failed} Failed`);
console.log("================================================================================\n");

if (failed > 0) {
  process.exit(1);
}
