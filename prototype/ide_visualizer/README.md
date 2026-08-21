# My-Idea IDE & Visual Tooling Prototypes
`prototype/ide_visualizer`

Interactive Visual Observatory, Pāṇinian Derivation DAG Inspector, and Phoneme Bitmask Tooling for **My-Idea** (Tauri / ClojureScript / CodeMirror 6).

---

## 1. Overview & Components

This prototype delivers rich visual inspection components for grammatical derivation graphs and phoneme vector encodings across the My Lisp ecosystem:

1. **Derivation DAG Inspector (`dag_visualizer.js`, `panini_view.cljs`):**
   - Renders step-by-step Pāṇinian derivations (e.g. *bhavati*, *dadāti*) from `panini-derivation-ir/0.1` JSON.
   - Interactive timeline graph showing immutable state nodes ($S_0 \to S_1 \to \dots \to S_n$) and rule transition arcs.
   - Real-time cryptographic state hash verification using SHA-256 (`canonical-json-sha256-v0.1`).
   - Morphological AST changes with dynamic diffing (added terms, mutated surface forms, elided *it*-markers/*lopa*).
   - Applied rule inspector detailing Aṣṭādhyāyī Sūtras (Devanagari, SLP1, Vidhi / Saṃjñā / Paribhāṣā classifications) and Paribhāṣā conflict resolution rationale (*Apavāda > Utsarga*, *Nitya > Anitya*, *Antaraṅga > Bahiraṅga*, *Para > Pūrva*, *Asiddhatva*).
   - Playback controls: Step forward/backward, Play/Pause auto-advance, direct step slider, and node click navigation.

2. **Phoneme Bitmask Inspector (`bitmask_inspector.js`):**
   - **16-bit PVC-16 (Phonetic Vector Code):**
     - Bitfield breakdown: `[0]` Vowel Flag (*ac* vs *hal*), `[5:1]` Sthāna place of articulation (Kaṇṭhya, Tālavya, Mūrdhanya, Dantya, Oṣṭhya), `[9:6]` Prayatna manner (Spṛṣṭa, Mahāprāṇa, Ghoṣa, Anunāsika), `[13:10]` Svara/Length (Hrasva, Dīrgha, Pluta), `[15:14]` Modifiers (Palatalized Ukrainian `[ь]`, Diphthong).
     - Clickable 16-bit interactive switch register with live articulation decoding.
     - Single-cycle Savarṇa homorganicity checker (Sūtra 1.1.9 *tulyāsyaprayatnaṁ savarṇam*).
     - Hardware & Lisp code generator emitting Verilog (`16'h...`), C/CML (`0x...`), and My Lisp (`(:pvc16 #x...)`).
   - **64-bit Pratyāhāra Bitmask Engine:**
     - 42 Canonical sound positions (bits 0..41) mapped across the 14 Śiva Sūtras.
     - Classical 42 Pratyāhāra selector (`ac`, `hal`, `al`, `ik`, `uk`, `eṅ`, `ec`, `aic`, `yaṇ`, `jhas`, `śar`, `khar`, etc.).
     - Single-cycle ALU set operations workbench ($A \cap B$, $A \cup B$, $A \setminus B$, $A \subseteq B$).

3. **Standalone Browser Application (`index.html` / `ide_visualizer_index.html`):**
   - Complete responsive interface with theme support (Dark, Light, Amber), preloaded fixtures, custom JSON loader, raw IR inspector, and Swarm node coordination status monitor.

4. **Automated Test Runner (`test_visualizer.mjs`):**
   - Validates data schemas, state hash serialization, PVC-16 bit manipulations, and 64-bit Pratyāhāra set operations.

---

## 2. Directory Structure

```
prototype/ide_visualizer/
├── index.html              # Standalone web app demo
├── dag_visualizer.js       # Core Derivation DAG renderer & hash validator
├── bitmask_inspector.js    # 16-bit PVC-16 & 64-bit Pratyāhāra inspector
├── panini_view.cljs        # ClojureScript Reagent/DOM component for My-Idea
├── fixtures.js             # Canonical Derivation IR fixtures (bhavati, dadAti)
├── test_visualizer.mjs     # Test runner & verification suite
└── README.md               # Architecture documentation & guide
```

---

## 3. Integration into My-Idea IDE

To mount the Derivation DAG Inspector and Phoneme Bitmask Tool into `my-idea`:

1. **Include ClojureScript Namespace:**
   Add `[my-idea.panini-view :as panini-view]` into `src-cljs/my_idea/core.cljs`.

2. **Add Topbar Tab Button:**
   Add `<button id='panini-inspector' title='Open Panini Derivation DAG & Bitmask Inspector'>🕉 Grammar Lab</button>` to the IDE topbar.

3. **Render Panini Pane:**
   In `core.cljs`'s right/bottom pane switcher:
   ```clojure
   (when (:panini-inspector @state)
     (panini-view/derivation-step-html active-derivation active-step))
   ```

4. **Tauri IPC Command:**
   Connect to `panini_eval` or live oracle over TCP port 9999 / swarm port 9104 to receive live `panini-derivation-ir/0.1` JSON payloads.

---

## 4. Running the Tests

```bash
# Run standalone test suite
node test_visualizer.mjs
```
All tests verify 100% pass rate for AST derivation chains, bit manipulations, and set algebra.
