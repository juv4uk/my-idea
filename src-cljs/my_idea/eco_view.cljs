(ns my-idea.eco-view
  "HTML builders for the System Observatory ecosystem panel — pure
  functions of their arguments, no app-state access, split out of
  core.cljs as part of continuing the i18n.cljs/util.cljs split."
  (:require [clojure.string :as str]
            [my-idea.util :as util]))

(defn- esc [x] (util/esc x))

(def result-icon {"pass" "✓" "fail" "✗" "skip" "·"})

(defn repo-summary-html [repo contract]
  (str "<div class='eco-repo'><strong>" (esc (:name repo)) "</strong> "
       (if (:found repo)
         (str "<span class='eco-branch'>" (esc (:branch repo)) "@" (esc (:sha repo)) "</span>")
         "<span class='eco-missing'>not found on disk</span>")
       (when contract
         (str " <span class='eco-version'>v" (get-in contract [:version :major]) "."
              (get-in contract [:version :minor]) "</span>"))
       "</div>"))

(defn embedded-drift-html [embedded-sha my-lisp-repo]
  (when embedded-sha
    (let [disk-sha (:sha my-lisp-repo)
          same? (and disk-sha (str/starts-with? disk-sha embedded-sha))]
      (str "<div class='eco-drift" (when-not same? " eco-drift-warn") "'>"
           "embedded engine (this app's own Rust runtime): <code>" (esc embedded-sha) "</code>"
           (if same?
             " — matches my-lisp on disk"
             (str " — <strong>differs</strong> from my-lisp on disk (" (esc (or disk-sha "unknown")) "). "
                  "Local eval, the live TCP oracle, and the my-lisp repo checkout can each be a different revision — "
                  "use 🔮 Oracle / ⚖ Compare to check the live one, not just this panel."))
           "</div>"))))

(defn evidence-cell-html [row impl-key]
  (if-let [rec (get-in row [:byImplementation impl-key])]
    (str "<td class='eco-cell eco-" (esc (:result rec)) "' title='"
         (esc (str (:fixture rec) " — expected " (:expected rec) ", actual " (:actual rec)
                    " (" (:commit rec) ", " (:timestamp rec) ")"
                    (when (:note rec) (str " — " (:note rec)))))
         "'>" (get result-icon (:result rec) "?") "</td>")
    "<td class='eco-cell eco-none'>—</td>"))

(defn evidence-matrix-html [matrix]
  (str "<table class='eco-matrix'><thead><tr><th>Req</th><th>my-lisp</th><th>cml</th><th>fpga-lisp</th></tr></thead><tbody>"
       (apply str (map (fn [row]
                          (str "<tr class='eco-row' data-req='" (esc (:requirement row)) "'><td>" (esc (:requirement row)) "</td>"
                               (evidence-cell-html row :my-lisp)
                               (evidence-cell-html row :cml)
                               (evidence-cell-html row :fpga-lisp)
                               "</tr>"))
                        matrix))
       "</tbody></table>"))

(defn fixture-record-html [impl-name rec]
  (str "<div class='eco-fixture-impl'><strong>" (esc impl-name) "</strong> "
       (if rec
         (str "<span class='eco-cell eco-" (esc (:result rec)) "'>" (get result-icon (:result rec) "?") " " (esc (:result rec)) "</span>"
              "<dl>"
              "<dt>expected</dt><dd><code>" (esc (:expected rec)) "</code></dd>"
              "<dt>actual</dt><dd><code>" (esc (:actual rec)) "</code></dd>"
              "<dt>commit</dt><dd>" (esc (:commit rec)) " · " (esc (:runner rec)) " · " (esc (:timestamp rec)) "</dd>"
              (when-let [env (:environment rec)]
                (str "<dt>environment</dt><dd>"
                     (str/join " · " (remove nil? [(when-let [r (:guixRevision env)] (str "guix " (subs r 0 (min 7 (count r)))))
                                                    (:channels env)
                                                    (:manifest env)]))
                     "</dd>"))
              (when (:note rec) (str "<dt>note</dt><dd>" (esc (:note rec)) "</dd>"))
              "</dl>")
         "<span class='eco-cell eco-none'>— no evidence</span>")
       "</div>"))

(defn causal-chain-html [by-impl]
  (let [my-lisp (:my-lisp by-impl)
        cml (:cml by-impl)
        fpga (:fpga-lisp by-impl)
        all-three? (and my-lisp cml fpga)
        all-pass? (and all-three? (every? #(= (:result %) "pass") [my-lisp cml fpga]))
        agree? (and all-pass? (= (:actual my-lisp) (:actual cml) (:actual fpga)))]
    (when all-three?
      (str "<div class='eco-chain" (when agree? " eco-chain-agree") "'>"
           "<div class='eco-chain-row'>"
           "<span class='eco-chain-step'>SOURCE</span>"
           "<span class='eco-chain-arrow'>→</span>"
           "<span class='eco-chain-step'>my-lisp oracle<br><code>" (esc (:actual my-lisp)) "</code> "
           (get result-icon (:result my-lisp) "?") "</span>"
           "<span class='eco-chain-arrow'>→</span>"
           "<span class='eco-chain-step'>cml compile<br><code>" (esc (:actual cml)) "</code> "
           (get result-icon (:result cml) "?") "</span>"
           "<span class='eco-chain-arrow'>→</span>"
           "<span class='eco-chain-step'>fpga-lisp execute<br><code>" (esc (:actual fpga)) "</code> "
           (get result-icon (:result fpga) "?") "</span>"
           "</div>"
           "<div class='eco-chain-verdict'>" (if agree? "✓ SEMANTIC AGREEMENT" "✗ MISMATCH") "</div>"
           "</div>"))))

(defn fixture-detail-html [row]
  (let [by-impl (:byImplementation row)
        any-rec (or (:my-lisp by-impl) (:cml by-impl) (:fpga-lisp by-impl))]
    (str "<div class='eco-fixture'>"
         "<button id='eco-back' class='eco-back'>← matrix</button>"
         "<h3>" (esc (:requirement row)) "</h3>"
         (when any-rec (str "<pre class='eco-fixture-source'>" (esc (:fixture any-rec)) "</pre>"))
         (or (causal-chain-html by-impl) "")
         (fixture-record-html "my-lisp" (:my-lisp by-impl))
         (fixture-record-html "cml" (:cml by-impl))
         (fixture-record-html "fpga-lisp" (:fpga-lisp by-impl))
         "</div>")))

(defn compatibility-html [compat]
  (when compat
    (str "<div class='eco-compat'>language "
         (if (:languageMatch compat) "✓" "✗")
         " · isa " (if (:isaMatch compat) "✓" "✗") "</div>")))

(defn ecosystem-html [eco selected-requirement]
  (if-let [row (and selected-requirement
                     (first (filter #(= (:requirement %) selected-requirement) (:evidenceMatrix eco))))]
    (str "<div class='eco'>" (fixture-detail-html row) "</div>")
    (str "<div class='eco'>"
         (repo-summary-html (:myLisp eco) (:myLispContract eco))
         (or (embedded-drift-html (:embeddedMyLispSha eco) (:myLisp eco)) "")
         (repo-summary-html (:cml eco) nil)
         (repo-summary-html (:fpgaLisp eco) (:fpgaLispContract eco))
         (or (compatibility-html (:compatibility eco)) "")
         (evidence-matrix-html (:evidenceMatrix eco))
         "</div>")))
