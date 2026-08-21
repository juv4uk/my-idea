/**
 * Phoneme Bitmask Inspector Tool
 * Visualizes:
 * 1. 16-bit PVC-16 (Phonetic Vector Code for FPGA/ISA)
 * 2. 64-bit Pratyāhāra Bitfields across 14 Śiva Sūtras
 */

// ============================================================================
// 1. PVC-16 CONSTANTS & DEFINITIONS
// ============================================================================

export const PVC16_BITS = {
  // Flag [0]
  VOWEL: 1 << 0,

  // Sthana [5:1]
  STHANA_KANTHYA: 1 << 1,   // Velar/Guttural
  STHANA_TALAVYA: 2 << 1,   // Palatal
  STHANA_MURDHANYA: 3 << 1, // Retroflex
  STHANA_DANTYA: 4 << 1,    // Dental
  STHANA_OSHTHYA: 5 << 1,   // Labial

  // Prayatna [9:6]
  PRAYATNA_SPRSTA: 1 << 6,     // Stop
  PRAYATNA_MAHAPRANA: 1 << 7,  // Aspirated
  PRAYATNA_GHOSHA: 1 << 8,     // Voiced
  PRAYATNA_ANUNASIKA: 1 << 9,  // Nasal

  // Svara / Length [13:10]
  LEN_HRASVA: 1 << 10,  // Short
  LEN_DIRGHA: 2 << 10,  // Long
  LEN_PLUTA: 3 << 10,   // Prolated

  // Modifiers [15:14]
  MOD_PALATALIZED: 1 << 14, // Ukrainian [ь] / Palatalized
  MOD_DIPHTHONG: 1 << 15    // Diphthong
};

export const PVC16_STHANA_NAMES = {
  0: "None / Avyakta",
  1: "Kaṇṭhya (Guttural / Velar)",
  2: "Tālavya (Palatal)",
  3: "Mūrdhanya (Retroflex)",
  4: "Dantya (Dental)",
  5: "Oṣṭhya (Labial)"
};

export const PVC16_PRESETS = {
  // Vowels
  "a": { name: "a (Short Guttural Vowel)", code: PVC16_BITS.VOWEL | PVC16_BITS.STHANA_KANTHYA | PVC16_BITS.PRAYATNA_GHOSHA | PVC16_BITS.LEN_HRASVA },
  "A": { name: "ā (Long Guttural Vowel)", code: PVC16_BITS.VOWEL | PVC16_BITS.STHANA_KANTHYA | PVC16_BITS.PRAYATNA_GHOSHA | PVC16_BITS.LEN_DIRGHA },
  "i": { name: "i (Short Palatal Vowel)", code: PVC16_BITS.VOWEL | PVC16_BITS.STHANA_TALAVYA | PVC16_BITS.PRAYATNA_GHOSHA | PVC16_BITS.LEN_HRASVA },
  "I": { name: "ī (Long Palatal Vowel)", code: PVC16_BITS.VOWEL | PVC16_BITS.STHANA_TALAVYA | PVC16_BITS.PRAYATNA_GHOSHA | PVC16_BITS.LEN_DIRGHA },
  "u": { name: "u (Short Labial Vowel)", code: PVC16_BITS.VOWEL | PVC16_BITS.STHANA_OSHTHYA | PVC16_BITS.PRAYATNA_GHOSHA | PVC16_BITS.LEN_HRASVA },
  "U": { name: "ū (Long Labial Vowel)", code: PVC16_BITS.VOWEL | PVC16_BITS.STHANA_OSHTHYA | PVC16_BITS.PRAYATNA_GHOSHA | PVC16_BITS.LEN_DIRGHA },
  "f": { name: "ṛ (Short Retroflex Vowel)", code: PVC16_BITS.VOWEL | PVC16_BITS.STHANA_MURDHANYA | PVC16_BITS.PRAYATNA_GHOSHA | PVC16_BITS.LEN_HRASVA },
  "x": { name: "ḷ (Short Dental Vowel)", code: PVC16_BITS.VOWEL | PVC16_BITS.STHANA_DANTYA | PVC16_BITS.PRAYATNA_GHOSHA | PVC16_BITS.LEN_HRASVA },

  // Velar Consonants (Kavarga)
  "k": { name: "k (Voiceless Unaspirated Velar Stop)", code: PVC16_BITS.STHANA_KANTHYA | PVC16_BITS.PRAYATNA_SPRSTA },
  "K": { name: "kh (Voiceless Aspirated Velar Stop)", code: PVC16_BITS.STHANA_KANTHYA | PVC16_BITS.PRAYATNA_SPRSTA | PVC16_BITS.PRAYATNA_MAHAPRANA },
  "g": { name: "g (Voiced Unaspirated Velar Stop)", code: PVC16_BITS.STHANA_KANTHYA | PVC16_BITS.PRAYATNA_SPRSTA | PVC16_BITS.PRAYATNA_GHOSHA },
  "G": { name: "gh (Voiced Aspirated Velar Stop)", code: PVC16_BITS.STHANA_KANTHYA | PVC16_BITS.PRAYATNA_SPRSTA | PVC16_BITS.PRAYATNA_MAHAPRANA | PVC16_BITS.PRAYATNA_GHOSHA },
  "N": { name: "ṅ (Velar Nasal)", code: PVC16_BITS.STHANA_KANTHYA | PVC16_BITS.PRAYATNA_SPRSTA | PVC16_BITS.PRAYATNA_GHOSHA | PVC16_BITS.PRAYATNA_ANUNASIKA },

  // Dental Consonants (Tavarga)
  "t": { name: "t (Voiceless Unaspirated Dental Stop)", code: PVC16_BITS.STHANA_DANTYA | PVC16_BITS.PRAYATNA_SPRSTA },
  "T": { name: "th (Voiceless Aspirated Dental Stop)", code: PVC16_BITS.STHANA_DANTYA | PVC16_BITS.PRAYATNA_SPRSTA | PVC16_BITS.PRAYATNA_MAHAPRANA },
  "d": { name: "d (Voiced Unaspirated Dental Stop)", code: PVC16_BITS.STHANA_DANTYA | PVC16_BITS.PRAYATNA_SPRSTA | PVC16_BITS.PRAYATNA_GHOSHA },
  "D": { name: "dh (Voiced Aspirated Dental Stop)", code: PVC16_BITS.STHANA_DANTYA | PVC16_BITS.PRAYATNA_SPRSTA | PVC16_BITS.PRAYATNA_MAHAPRANA | PVC16_BITS.PRAYATNA_GHOSHA },
  "n": { name: "n (Dental Nasal)", code: PVC16_BITS.STHANA_DANTYA | PVC16_BITS.PRAYATNA_SPRSTA | PVC16_BITS.PRAYATNA_GHOSHA | PVC16_BITS.PRAYATNA_ANUNASIKA },

  // Labial Consonants (Pavarga)
  "p": { name: "p (Voiceless Unaspirated Labial Stop)", code: PVC16_BITS.STHANA_OSHTHYA | PVC16_BITS.PRAYATNA_SPRSTA },
  "P": { name: "ph (Voiceless Aspirated Labial Stop)", code: PVC16_BITS.STHANA_OSHTHYA | PVC16_BITS.PRAYATNA_SPRSTA | PVC16_BITS.PRAYATNA_MAHAPRANA },
  "b": { name: "b (Voiced Unaspirated Labial Stop)", code: PVC16_BITS.STHANA_OSHTHYA | PVC16_BITS.PRAYATNA_SPRSTA | PVC16_BITS.PRAYATNA_GHOSHA },
  "B": { name: "bh (Voiced Aspirated Labial Stop)", code: PVC16_BITS.STHANA_OSHTHYA | PVC16_BITS.PRAYATNA_SPRSTA | PVC16_BITS.PRAYATNA_MAHAPRANA | PVC16_BITS.PRAYATNA_GHOSHA },
  "m": { name: "m (Labial Nasal)", code: PVC16_BITS.STHANA_OSHTHYA | PVC16_BITS.PRAYATNA_SPRSTA | PVC16_BITS.PRAYATNA_GHOSHA | PVC16_BITS.PRAYATNA_ANUNASIKA },

  // Ukrainian Extended Phonemes
  "uk_t_soft": { name: "Ukrainian [т'] (Soft Dental Stop)", code: PVC16_BITS.STHANA_DANTYA | PVC16_BITS.PRAYATNA_SPRSTA | PVC16_BITS.MOD_PALATALIZED },
  "uk_d_soft": { name: "Ukrainian [д'] (Soft Voiced Dental Stop)", code: PVC16_BITS.STHANA_DANTYA | PVC16_BITS.PRAYATNA_SPRSTA | PVC16_BITS.PRAYATNA_GHOSHA | PVC16_BITS.MOD_PALATALIZED },
  "uk_n_soft": { name: "Ukrainian [н'] (Soft Dental Nasal)", code: PVC16_BITS.STHANA_DANTYA | PVC16_BITS.PRAYATNA_SPRSTA | PVC16_BITS.PRAYATNA_GHOSHA | PVC16_BITS.PRAYATNA_ANUNASIKA | PVC16_BITS.MOD_PALATALIZED },
  "uk_s_soft": { name: "Ukrainian [с'] (Soft Dental Sibilant)", code: PVC16_BITS.STHANA_DANTYA | PVC16_BITS.MOD_PALATALIZED }
};

// ============================================================================
// 2. 64-BIT PRATYĀHĀRA CANONICAL SOUNDS & BIT POSITIONS
// ============================================================================

export const CANONICAL_64_SOUNDS = [
  // Sutra 1-4: Vowels (ac) -> bits 0..8
  { bit: 0,  slp1: "a", deva: "अ", iast: "a",  sutra: 1,  class: "vowel" },
  { bit: 1,  slp1: "i", deva: "इ", iast: "i",  sutra: 1,  class: "vowel" },
  { bit: 2,  slp1: "u", deva: "उ", iast: "u",  sutra: 1,  class: "vowel" },
  { bit: 3,  slp1: "f", deva: "ऋ", iast: "ṛ",  sutra: 2,  class: "vowel" },
  { bit: 4,  slp1: "x", deva: "ऌ", iast: "ḷ",  sutra: 2,  class: "vowel" },
  { bit: 5,  slp1: "e", deva: "ए", iast: "e",  sutra: 3,  class: "vowel" },
  { bit: 6,  slp1: "o", deva: "ओ", iast: "o",  sutra: 3,  class: "vowel" },
  { bit: 7,  slp1: "E", deva: "ऐ", iast: "ai", sutra: 4,  class: "vowel" },
  { bit: 8,  slp1: "O", deva: "औ", iast: "au", sutra: 4,  class: "vowel" },

  // Sutra 5-6: Semivowels + h -> bits 9..13
  { bit: 9,  slp1: "h", deva: "ह", iast: "ha", sutra: 5,  class: "semivowel" },
  { bit: 10, slp1: "y", deva: "य", iast: "ya", sutra: 5,  class: "semivowel" },
  { bit: 11, slp1: "v", deva: "व", iast: "va", sutra: 5,  class: "semivowel" },
  { bit: 12, slp1: "r", deva: "र", iast: "ra", sutra: 5,  class: "semivowel" },
  { bit: 13, slp1: "l", deva: "ल", iast: "la", sutra: 6,  class: "semivowel" },

  // Sutra 7: Nasals -> bits 14..18
  { bit: 14, slp1: "Y", deva: "ञ", iast: "ña", sutra: 7,  class: "nasal" },
  { bit: 15, slp1: "m", deva: "म", iast: "ma", sutra: 7,  class: "nasal" },
  { bit: 16, slp1: "N", deva: "ङ", iast: "ṅa", sutra: 7,  class: "nasal" },
  { bit: 17, slp1: "R", deva: "ण", iast: "ṇa", sutra: 7,  class: "nasal" },
  { bit: 18, slp1: "n", deva: "न", iast: "na", sutra: 7,  class: "nasal" },

  // Sutra 8-9: Voiced Aspirated Stops -> bits 19..23
  { bit: 19, slp1: "J", deva: "झ", iast: "jha", sutra: 8,  class: "stop-voiced-asp" },
  { bit: 20, slp1: "B", deva: "भ", iast: "bha", sutra: 8,  class: "stop-voiced-asp" },
  { bit: 21, slp1: "G", deva: "घ", iast: "gha", sutra: 9,  class: "stop-voiced-asp" },
  { bit: 22, slp1: "Q", deva: "ढ", iast: "ḍha", sutra: 9,  class: "stop-voiced-asp" },
  { bit: 23, slp1: "D", deva: "ध", iast: "dha", sutra: 9,  class: "stop-voiced-asp" },

  // Sutra 10: Voiced Unaspirated Stops -> bits 24..28
  { bit: 24, slp1: "j", deva: "ज", iast: "ja", sutra: 10, class: "stop-voiced-unasp" },
  { bit: 25, slp1: "b", deva: "ब", iast: "ba", sutra: 10, class: "stop-voiced-unasp" },
  { bit: 26, slp1: "g", deva: "ग", iast: "ga", sutra: 10, class: "stop-voiced-unasp" },
  { bit: 27, slp1: "q", deva: "ड", iast: "ḍa", sutra: 10, class: "stop-voiced-unasp" },
  { bit: 28, slp1: "d", deva: "द", iast: "da", sutra: 10, class: "stop-voiced-unasp" },

  // Sutra 11: Voiceless Aspirated + Stops -> bits 29..36
  { bit: 29, slp1: "K", deva: "ख", iast: "kha", sutra: 11, class: "stop-voiceless-asp" },
  { bit: 30, slp1: "P", deva: "फ", iast: "pha", sutra: 11, class: "stop-voiceless-asp" },
  { bit: 31, slp1: "C", deva: "छ", iast: "cha", sutra: 11, class: "stop-voiceless-asp" },
  { bit: 32, slp1: "W", deva: "ठ", iast: "ṭha", sutra: 11, class: "stop-voiceless-asp" },
  { bit: 33, slp1: "T", deva: "थ", iast: "tha", sutra: 11, class: "stop-voiceless-asp" },
  { bit: 34, slp1: "c", deva: "च", iast: "ca",  sutra: 11, class: "stop-voiceless-unasp" },
  { bit: 35, slp1: "w", deva: "ट", iast: "ṭa",  sutra: 11, class: "stop-voiceless-unasp" },
  { bit: 36, slp1: "t", deva: "त", iast: "ta",  sutra: 11, class: "stop-voiceless-unasp" },

  // Sutra 12: Voiceless Velar/Labial -> bits 37..38
  { bit: 37, slp1: "k", deva: "क", iast: "ka",  sutra: 12, class: "stop-voiceless-unasp" },
  { bit: 38, slp1: "p", deva: "प", iast: "pa",  sutra: 12, class: "stop-voiceless-unasp" },

  // Sutra 13: Sibilants -> bits 39..41
  { bit: 39, slp1: "S", deva: "श", iast: "śa",  sutra: 13, class: "sibilant" },
  { bit: 40, slp1: "z", deva: "ष", iast: "ṣa",  sutra: 13, class: "sibilant" },
  { bit: 41, slp1: "s", deva: "स", iast: "sa",  sutra: 13, class: "sibilant" }
];

export const PRATYAHARAS_64 = {
  "ac":   { name: "ac (All 9 Vowels)", bits: [0,1,2,3,4,5,6,7,8] },
  "ak":   { name: "ak (Simple Vowels: a, i, u, ṛ, ḷ)", bits: [0,1,2,3,4] },
  "ik":   { name: "ik (Closed Vowels: i, u, ṛ, ḷ)", bits: [1,2,3,4] },
  "uk":   { name: "uk (u, ṛ, ḷ)", bits: [2,3,4] },
  "eN":   { name: "eṅ (Guṇa Vowels: e, o)", bits: [5,6] },
  "ec":   { name: "ec (Diphthongs: e, o, ai, au)", bits: [5,6,7,8] },
  "Ec":   { name: "aic (Vṛddhi Vowels: ai, au)", bits: [7,8] },
  "hal":  { name: "hal (All 33 Consonants)", bits: Array.from({length: 33}, (_, i) => i + 9) },
  "al":   { name: "al (All 42 Phonemes)", bits: Array.from({length: 42}, (_, i) => i) },
  "yaR":  { name: "yaṇ (Semivowels: y, v, r, l)", bits: [10,11,12,13] },
  "Sar":  { name: "śar (Sibilants: ś, ṣ, s)", bits: [39,40,41] },
  "Jal":  { name: "jhal (All Non-Nasal Consonants)", bits: Array.from({length: 23}, (_, i) => i + 19) },
  "Jaz":  { name: "jhas (Voiced Aspirated Stops)", bits: [19,20,21,22,23] },
  "jaS":  { name: "jaś (Voiced Unaspirated Stops)", bits: [24,25,26,27,28] },
  "Kar":  { name: "khar (All Voiceless Consonants)", bits: [29,30,31,32,33,34,35,36,37,38,39,40,41] },
  "cay":  { name: "cay (Voiceless Unaspirated Stops: c, ṭ, t, k, p)", bits: [34,35,36,37,38] }
};

export class PhonemeBitmaskInspector {
  constructor(containerElement) {
    this.container = typeof containerElement === "string" 
      ? document.getElementById(containerElement) 
      : containerElement;

    this.pvc16Code = PVC16_PRESETS["a"].code;
    this.comparatorPvc16 = PVC16_PRESETS["i"].code;
    this.pratyaharaMaskBigInt = 0n;

    // Initialize with 'ik' mask
    this.loadPratyahara("ik");

    this.initDOM();
  }

  initDOM() {
    this.container.innerHTML = `
      <div class="bitmask-inspector-root">
        <!-- Sub-tabs: PVC-16 Vector vs 64-bit Pratyahara Matrix -->
        <div class="bitmask-subtabs">
          <button class="bitmask-tab-btn active" id="tab-pvc16">16-bit PVC-16 (Phonetic Vector Code)</button>
          <button class="bitmask-tab-btn" id="tab-pratyahara">64-bit Pratyāhāra Bitmask Engine</button>
        </div>

        <!-- PVC-16 VIEW -->
        <div class="bitmask-view active" id="view-pvc16">
          <div class="pvc16-layout">
            <!-- Left: Vector Controls & Bit Grid -->
            <div class="pvc16-left">
              <div class="pvc16-presets-row">
                <label>Phoneme Preset:</label>
                <select class="pvc16-select" id="pvc16-preset-select">
                  <optgroup label="Sanskrit Vowels (ac)">
                    <option value="a">a (Short Guttural)</option>
                    <option value="A">ā (Long Guttural)</option>
                    <option value="i">i (Short Palatal)</option>
                    <option value="I">ī (Long Palatal)</option>
                    <option value="u">u (Short Labial)</option>
                    <option value="U">ū (Long Labial)</option>
                    <option value="f">ṛ (Short Retroflex)</option>
                    <option value="x">ḷ (Short Dental)</option>
                  </optgroup>
                  <optgroup label="Sanskrit Consonants (hal)">
                    <option value="k">k (Velar Stop)</option>
                    <option value="K">kh (Aspirated Velar Stop)</option>
                    <option value="g">g (Voiced Velar Stop)</option>
                    <option value="G">gh (Voiced Aspirated Velar)</option>
                    <option value="N">ṅ (Velar Nasal)</option>
                    <option value="t">t (Dental Stop)</option>
                    <option value="T">th (Aspirated Dental Stop)</option>
                    <option value="d">d (Voiced Dental Stop)</option>
                    <option value="D">dh (Voiced Aspirated Dental)</option>
                    <option value="n">n (Dental Nasal)</option>
                    <option value="p">p (Labial Stop)</option>
                    <option value="P">ph (Aspirated Labial Stop)</option>
                    <option value="b">b (Voiced Labial Stop)</option>
                    <option value="B">bh (Voiced Aspirated Labial)</option>
                    <option value="m">m (Labial Nasal)</option>
                  </optgroup>
                  <optgroup label="Ukrainian Phonetics (Extensions)">
                    <option value="uk_t_soft">Ukrainian [т'] (Soft Dental)</option>
                    <option value="uk_d_soft">Ukrainian [д'] (Soft Voiced Dental)</option>
                    <option value="uk_n_soft">Ukrainian [н'] (Soft Dental Nasal)</option>
                    <option value="uk_s_soft">Ukrainian [с'] (Soft Sibilant)</option>
                  </optgroup>
                </select>
                <button class="dag-btn" id="btn-toggle-voice">Toggle Voicing (Reg ^ GHOSHA)</button>
                <button class="dag-btn" id="btn-toggle-palat">Toggle [ь] (Reg ^ MOD_PALAT)</button>
              </div>

              <!-- 16-Bit Interactive Grid -->
              <div class="bit-grid-container">
                <div class="bit-grid-title">16-Bit Hardware Vector Register [15:0]</div>
                <div class="bit-grid-16" id="pvc16-bit-switches"></div>
              </div>

              <!-- Vector Fields Breakdown -->
              <div class="pvc16-fields-breakdown">
                <div class="field-box field-mod">
                  <div class="field-name">Modifier [15:14]</div>
                  <div class="field-val" id="pvc16-val-mod">None</div>
                </div>
                <div class="field-box field-svara">
                  <div class="field-name">Svara/Len [13:10]</div>
                  <div class="field-val" id="pvc16-val-svara">Hrasva (Short)</div>
                </div>
                <div class="field-box field-prayatna">
                  <div class="field-name">Prayatna [9:6]</div>
                  <div class="field-val" id="pvc16-val-prayatna">Ghoṣa (Voiced)</div>
                </div>
                <div class="field-box field-sthana">
                  <div class="field-name">Sthāna [5:1]</div>
                  <div class="field-val" id="pvc16-val-sthana">Kaṇṭhya (Guttural)</div>
                </div>
                <div class="field-box field-flag">
                  <div class="field-name">Flag [0]</div>
                  <div class="field-val" id="pvc16-val-flag">1 (Vowel)</div>
                </div>
              </div>
            </div>

            <!-- Right: Articulation Properties & Savarna Calculator -->
            <div class="pvc16-right">
              <!-- Savarna Homorganicity Check (Sutra 1.1.9) -->
              <div class="savarna-card">
                <div class="savarna-header">Savarṇa Checker (1.1.9 tulyāsyaprayatnaṁ savarṇam)</div>
                <div class="savarna-row">
                  <div class="savarna-sound-a">Sound A: <strong id="savarna-name-a">a</strong></div>
                  <div class="savarna-vs">vs</div>
                  <div class="savarna-sound-b">
                    Sound B: 
                    <select class="pvc16-select-sm" id="savarna-select-b">
                      <option value="A">ā</option>
                      <option value="i">i</option>
                      <option value="k">k</option>
                      <option value="t">t</option>
                    </select>
                  </div>
                </div>
                <div class="savarna-verdict" id="savarna-verdict">✓ SAVARṆA (Homorganic: Same Sthāna & Prayatna)</div>
                <div class="savarna-math-formula">
                  <code>wire is_savarna = ((A & 0x3E) == (B & 0x3E)) && ((A & 0x40) == (B & 0x40)) && ((A & 1) == (B & 1));</code>
                  <span class="alu-cost">Cost: 1 Clock Cycle (LUT-2)</span>
                </div>
              </div>

              <!-- Export Code Formats -->
              <div class="pvc16-code-export">
                <div class="code-export-header">Generated Hardware / VM Constants</div>
                <div class="code-block">
                  <span class="code-lang">Verilog:</span> <code id="export-verilog">16'h0000;</code>
                </div>
                <div class="code-block">
                  <span class="code-lang">C / CML:</span> <code id="export-cml">0x0000;</code>
                </div>
                <div class="code-block">
                  <span class="code-lang">My Lisp:</span> <code id="export-lisp">(:pvc16 #x0000)</code>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 64-BIT PRATYĀHĀRA VIEW -->
        <div class="bitmask-view" id="view-pratyahara">
          <div class="pratyahara-layout">
            <div class="pratyahara-toolbar">
              <label>Pāṇinian Canon 42 Pratyāhāras:</label>
              <select class="pvc16-select" id="pratyahara-select">
                <option value="ac">ac (All 9 Vowels: a, i, u, ṛ, ḷ, e, o, ai, au)</option>
                <option value="ak">ak (Simple Vowels: a, i, u, ṛ, ḷ)</option>
                <option value="ik" selected>ik (Closed Vowels: i, u, ṛ, ḷ)</option>
                <option value="uk">uk (u, ṛ, ḷ)</option>
                <option value="eN">eṅ (Guṇa Vowels: e, o)</option>
                <option value="ec">ec (Diphthongs: e, o, ai, au)</option>
                <option value="Ec">aic (Vṛddhi Vowels: ai, au)</option>
                <option value="hal">hal (All 33 Consonants)</option>
                <option value="al">al (Complete 42 Phoneme Universe)</option>
                <option value="yaR">yaṇ (Semivowels: y, v, r, l)</option>
                <option value="Sar">śar (Sibilants: ś, ṣ, s)</option>
                <option value="Jal">jhal (All Non-Nasal Consonants)</option>
                <option value="Jaz">jhas (Voiced Aspirated Stops)</option>
                <option value="jaS">jaś (Voiced Unaspirated Stops)</option>
                <option value="Kar">khar (All Voiceless Consonants)</option>
                <option value="cay">cay (Voiceless Unaspirated Stops)</option>
              </select>
              <button class="dag-btn" id="btn-prat-clear">Clear All</button>
              <button class="dag-btn" id="btn-prat-invert">Invert Mask (~M & AL)</button>
              <span class="prat-mask-hex" id="prat-mask-hex">Mask: 0x000000000000003C</span>
            </div>

            <!-- 42 Phoneme Bit Matrix -->
            <div class="pratyahara-grid" id="pratyahara-bit-grid"></div>

            <!-- Set Operations Workbench -->
            <div class="pratyahara-set-ops">
              <div class="set-ops-header">Single-Cycle Bitwise Set Operations ALU Workbench</div>
              <div class="set-ops-grid">
                <div class="set-op-item">
                  <span class="set-op-title">Membership Check (1.1.71 Ādirantyena sahetā)</span>
                  <code>is_member = (sound_mask & PRATYAHARA_MASK) != 0; // 0.3 ns</code>
                </div>
                <div class="set-op-item">
                  <span class="set-op-title">Class Intersection (ik ∩ ac = ik)</span>
                  <code>MASK_IK & MASK_AC = <span id="op-res-and">0x0000003C</span></code>
                </div>
                <div class="set-op-item">
                  <span class="set-op-title">Class Union (yaṇ ∪ śar)</span>
                  <code>MASK_YAN | MASK_SAR = <span id="op-res-or">0x000003E0</span></code>
                </div>
                <div class="set-op-item">
                  <span class="set-op-title">Class Difference (hal \\ jhal = yaṇ + nasals)</span>
                  <code>MASK_HAL & ~MASK_JHAL = <span id="op-res-diff">0x0007FC00</span></code>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    `;

    this.bindEvents();
    this.renderPVC16();
    this.renderPratyahara();
  }

  bindEvents() {
    const root = this.container;

    // Tab switching
    const tabPvc = root.querySelector("#tab-pvc16");
    const tabPrat = root.querySelector("#tab-pratyahara");
    const viewPvc = root.querySelector("#view-pvc16");
    const viewPrat = root.querySelector("#view-pratyahara");

    tabPvc.onclick = () => {
      tabPvc.classList.add("active");
      tabPrat.classList.remove("active");
      viewPvc.classList.add("active");
      viewPrat.classList.remove("active");
    };

    tabPrat.onclick = () => {
      tabPrat.classList.add("active");
      tabPvc.classList.remove("active");
      viewPrat.classList.add("active");
      viewPvc.classList.remove("active");
    };

    // PVC-16 Presets
    const presetSel = root.querySelector("#pvc16-preset-select");
    presetSel.onchange = (e) => {
      const p = PVC16_PRESETS[e.target.value];
      if (p) {
        this.pvc16Code = p.code;
        this.renderPVC16();
      }
    };

    // Voice toggle
    root.querySelector("#btn-toggle-voice").onclick = () => {
      this.pvc16Code ^= PVC16_BITS.PRAYATNA_GHOSHA;
      this.renderPVC16();
    };

    // Palatalization toggle
    root.querySelector("#btn-toggle-palat").onclick = () => {
      this.pvc16Code ^= PVC16_BITS.MOD_PALATALIZED;
      this.renderPVC16();
    };

    // Savarna Select
    root.querySelector("#savarna-select-b").onchange = (e) => {
      const p = PVC16_PRESETS[e.target.value];
      if (p) {
        this.comparatorPvc16 = p.code;
        this.renderSavarna();
      }
    };

    // Pratyahara select
    root.querySelector("#pratyahara-select").onchange = (e) => {
      this.loadPratyahara(e.target.value);
    };

    root.querySelector("#btn-prat-clear").onclick = () => {
      this.pratyaharaMaskBigInt = 0n;
      this.renderPratyahara();
    };

    root.querySelector("#btn-prat-invert").onclick = () => {
      const fullAlMask = (1n << 42n) - 1n;
      this.pratyaharaMaskBigInt = (~this.pratyaharaMaskBigInt) & fullAlMask;
      this.renderPratyahara();
    };
  }

  loadPratyahara(name) {
    const def = PRATYAHARAS_64[name];
    if (def) {
      let mask = 0n;
      def.bits.forEach(b => {
        mask |= (1n << BigInt(b));
      });
      this.pratyaharaMaskBigInt = mask;
      this.renderPratyahara();
    }
  }

  renderPVC16() {
    const code = this.pvc16Code;
    const switchesContainer = this.container.querySelector("#pvc16-bit-switches");
    switchesContainer.innerHTML = "";

    const bitLabels = [
      "VOW", "KNT", "TAL", "MUR", "DAN", "OSH",
      "SPR", "MAH", "GHO", "ANU",
      "LEN0", "LEN1", "LEN2", "LEN3",
      "PAL", "DIPH"
    ];

    for (let i = 15; i >= 0; i--) {
      const isSet = Boolean(code & (1 << i));
      const bitBtn = document.createElement("button");
      bitBtn.className = `bit-switch ${isSet ? "bit-on" : "bit-off"}`;
      bitBtn.innerHTML = `
        <span class="bit-num">${i}</span>
        <span class="bit-state">${isSet ? "1" : "0"}</span>
        <span class="bit-lbl">${bitLabels[i]}</span>
      `;
      bitBtn.onclick = () => {
        this.pvc16Code ^= (1 << i);
        this.renderPVC16();
      };
      switchesContainer.appendChild(bitBtn);
    }

    // Decode Fields
    const isVowel = Boolean(code & PVC16_BITS.VOWEL);
    const sthanaCode = (code >> 1) & 0x1F;
    const isStop = Boolean(code & PVC16_BITS.PRAYATNA_SPRSTA);
    const isAsp = Boolean(code & PVC16_BITS.PRAYATNA_MAHAPRANA);
    const isVoiced = Boolean(code & PVC16_BITS.PRAYATNA_GHOSHA);
    const isNasal = Boolean(code & PVC16_BITS.PRAYATNA_ANUNASIKA);
    const svaraCode = (code >> 10) & 0x0F;
    const isPalat = Boolean(code & PVC16_BITS.MOD_PALATALIZED);
    const isDiph = Boolean(code & PVC16_BITS.MOD_DIPHTHONG);

    this.container.querySelector("#pvc16-val-flag").textContent = isVowel ? "1 (Vowel / ac)" : "0 (Consonant / hal)";
    this.container.querySelector("#pvc16-val-sthana").textContent = PVC16_STHANA_NAMES[sthanaCode] || `Unknown (${sthanaCode})`;

    const prayatnaParts = [];
    if (isStop) prayatnaParts.push("Spṛṣṭa (Stop)");
    if (isAsp) prayatnaParts.push("Mahāprāṇa (Asp)");
    if (isVoiced) prayatnaParts.push("Ghoṣa (Voiced)");
    if (isNasal) prayatnaParts.push("Anunāsika (Nasal)");
    this.container.querySelector("#pvc16-val-prayatna").textContent = prayatnaParts.join(" + ") || "None";

    let lenStr = "None";
    if (svaraCode === 1) lenStr = "Hrasva (Short / 1 mātrā)";
    else if (svaraCode === 2) lenStr = "Dīrgha (Long / 2 mātrā)";
    else if (svaraCode === 3) lenStr = "Pluta (Prolated / 3 mātrā)";
    this.container.querySelector("#pvc16-val-svara").textContent = lenStr;

    const modParts = [];
    if (isPalat) modParts.push("Palatalized [ь]");
    if (isDiph) modParts.push("Diphthong");
    this.container.querySelector("#pvc16-val-mod").textContent = modParts.join(" + ") || "None";

    // Export code display
    const hexStr = "0x" + code.toString(16).toUpperCase().padStart(4, "0");
    this.container.querySelector("#export-verilog").textContent = `16'h${code.toString(16).toUpperCase().padStart(4, "0")};`;
    this.container.querySelector("#export-cml").textContent = `${hexStr};`;
    this.container.querySelector("#export-lisp").textContent = `(:pvc16 #x${code.toString(16).toUpperCase().padStart(4, "0")})`;

    this.renderSavarna();
  }

  renderSavarna() {
    const a = this.pvc16Code;
    const b = this.comparatorPvc16;

    // Sutra 1.1.9: tulyasyaprayatnam savarnam
    // Same Sthana [5:1] AND Same Spṛṣṭa [6] AND Same Vowel/Consonant flag [0]
    const sthanaA = a & (0x1F << 1);
    const sthanaB = b & (0x1F << 1);
    const sameSthana = (sthanaA === sthanaB) && (sthanaA !== 0);

    const sprstaA = Boolean(a & PVC16_BITS.PRAYATNA_SPRSTA);
    const sprstaB = Boolean(b & PVC16_BITS.PRAYATNA_SPRSTA);
    const vowelA = Boolean(a & PVC16_BITS.VOWEL);
    const vowelB = Boolean(b & PVC16_BITS.VOWEL);

    const samePrayatna = (sprstaA === sprstaB) && (vowelA === vowelB);
    const isSavarna = sameSthana && samePrayatna;

    const verdictEl = this.container.querySelector("#savarna-verdict");
    if (isSavarna) {
      verdictEl.className = "savarna-verdict savarna-yes";
      verdictEl.textContent = "✓ SAVARṆA (Homorganic: Matches Place & Internal Effort)";
    } else {
      verdictEl.className = "savarna-verdict savarna-no";
      verdictEl.textContent = `✗ NOT SAVARṆA (${!sameSthana ? "Different Sthāna" : "Different Prayatna"})`;
    }
  }

  renderPratyahara() {
    const grid = this.container.querySelector("#pratyahara-bit-grid");
    grid.innerHTML = "";

    const mask = this.pratyaharaMaskBigInt;
    this.container.querySelector("#prat-mask-hex").textContent = `Mask: 0x${mask.toString(16).toUpperCase().padStart(16, "0")}`;

    CANONICAL_64_SOUNDS.forEach(sound => {
      const isSet = Boolean((mask >> BigInt(sound.bit)) & 1n);
      const cell = document.createElement("div");
      cell.className = `prat-cell ${sound.class} ${isSet ? "active" : ""}`;
      cell.innerHTML = `
        <div class="prat-cell-bit">${sound.bit}</div>
        <div class="prat-cell-deva">${sound.deva}</div>
        <div class="prat-cell-iast">${sound.iast}</div>
        <div class="prat-cell-sutra">Sūtra ${sound.sutra}</div>
      `;
      cell.onclick = () => {
        this.pratyaharaMaskBigInt ^= (1n << BigInt(sound.bit));
        this.renderPratyahara();
      };
      grid.appendChild(cell);
    });
  }
}
