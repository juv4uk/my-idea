(ns my-idea.panini-view
  "Paninian Grammar Machine & Phoneme Bitmask Inspector UI Components for My-Idea.
   Provides pure HTML renderers and interactive Reagent/DOM mounts for
   Derivation DAGs (bhavati, dadAti), PVC-16 registers, and 64-bit Pratyāhāras."
  (:require [clojure.string :as str]
            [my-idea.util :as util]))

(defn- esc [x] (util/esc x))

;; ============================================================================
;; 1. PVC-16 (Phonetic Vector Code) HTML BUILDERS
;; ============================================================================

(def sthana-labels
  {0 "None / Avyakta"
   1 "Kaṇṭhya (Velar/Guttural)"
   2 "Tālavya (Palatal)"
   3 "Mūrdhanya (Retroflex)"
   4 "Dantya (Dental)"
   5 "Oṣṭhya (Labial)"})

(defn pvc16-decode [code]
  (let [vowel? (pos? (bit-and code 1))
        sthana-code (bit-shift-right (bit-and code 2r0000000000111110) 1)
        sprsta? (pos? (bit-and code (bit-shift-left 1 6)))
        mahaprana? (pos? (bit-and code (bit-shift-left 1 7)))
        ghosha? (pos? (bit-and code (bit-shift-left 1 8)))
        anunasika? (pos? (bit-and code (bit-shift-left 1 9)))
        svara-len (bit-shift-right (bit-and code 2r0011110000000000) 10)
        palatalized? (pos? (bit-and code (bit-shift-left 1 14)))
        diphthong? (pos? (bit-and code (bit-shift-left 1 15)))]
    {:vowel? vowel?
     :sthana (get sthana-labels sthana-code (str "Unknown (" sthana-code ")"))
     :sprsta? sprsta?
     :mahaprana? mahaprana?
     :ghosha? ghosha?
     :anunasika? anunasika?
     :length (case svara-len
               1 "Hrasva (Short)"
               2 "Dīrgha (Long)"
               3 "Pluta (Prolated)"
               "None")
     :palatalized? palatalized?
     :diphthong? diphthong?}))

(defn pvc16-vector-html [code]
  (let [decoded (pvc16-decode code)
        hex-str (str "0x" (.toString code 16))]
    (str "<div class='pvc16-summary-card'>"
         "<div class='pvc16-hex'><strong>PVC-16:</strong> <code>" (esc hex-str) "</code></div>"
         "<div class='pvc16-traits'>"
         "<span class='trait-pill'>" (if (:vowel? decoded) "Vowel (ac)" "Consonant (hal)") "</span> "
         "<span class='trait-pill'>" (esc (:sthana decoded)) "</span> "
         (when (:ghosha? decoded) "<span class='trait-pill ghosha'>Ghoṣa (Voiced)</span> ")
         (when (:mahaprana? decoded) "<span class='trait-pill'>Mahāprāṇa (Asp)</span> ")
         (when (:sprsta? decoded) "<span class='trait-pill'>Spṛṣṭa (Stop)</span> ")
         (when (:palatalized? decoded) "<span class='trait-pill palat'>Palatalized [ь]</span> ")
         "</div>"
         "</div>")))

;; ============================================================================
;; 2. DERIVATION DAG HTML BUILDERS
;; ============================================================================

(defn term-node-html [term prev-term]
  (let [new? (nil? prev-term)
        modified? (and prev-term (not= (:surface-form term) (:surface-form prev-term)))
        status-class (cond new? "term-added" modified? "term-modified" :else "term-stable")]
    (str "<div class='dag-term-node " status-class "'>"
         "<div class='dag-term-top'><span class='dag-term-kind'>" (esc (:kind term)) "</span>"
         "<span class='dag-term-id'>" (esc (:id term)) "</span></div>"
         "<div class='dag-term-forms'>"
         "<div class='dag-term-surface'><label>Surface:</label> <strong>" (esc (or (:surface-form term) "∅")) "</strong></div>"
         "<div class='dag-term-source'><label>Source:</label> <span>" (esc (:source-form term)) "</span></div>"
         "</div>"
         "<div class='dag-term-designations'>"
         (apply str (map #(str "<span class='dag-desig-tag'>" (esc %) "</span>") (:designations term)))
         "</div>"
         "</div>")))

(defn derivation-step-html [derivation step-idx]
  (let [states (:states derivation)
        st (nth states step-idx nil)
        prev-st (when (pos? step-idx) (nth states (dec step-idx) nil))
        prev-terms-map (into {} (map (juxt :id identity) (:terms prev-st)))]
    (if-not st
      "<div class='dag-empty'>No step selected</div>"
      (str "<div class='dag-step-inspector'>"
           "<div class='dag-step-header'>"
           "<h4>State S" step-idx ": <code>" (esc (:id st)) "</code></h4>"
           "<span class='dag-hash'>" (esc (:hash st)) "</span>"
           "</div>"
           "<div class='dag-terms-container'>"
           (apply str (map #(term-node-html % (get prev-terms-map (:id %))) (:terms st)))
           "</div>"
           "</div>"))))

(defn derivation-dag-overview-html [derivation active-step]
  (let [states (:states derivation)]
    (str "<div class='dag-overview-timeline'>"
         (apply str
                (map-indexed
                 (fn [idx st]
                   (let [active? (= idx active-step)
                         surface (str/join " + " (remove str/blank? (map :surface-form (:terms st))))]
                     (str "<div class='dag-timeline-node" (when active? " active") "' data-step='" idx "'>"
                          "<div class='node-step-badge'>S" idx "</div>"
                          "<div class='node-step-surface'>" (esc surface) "</div>"
                          "</div>")))
                 states))
         "</div>")))
