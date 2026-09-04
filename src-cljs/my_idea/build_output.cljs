(ns my-idea.build-output
  "Build Output panel — subscribes to Tauri 'build-output' events and
  maintains ordered lines with stream tags, active profile, exit state.
  Mirrors the lsp.cljs event-listener pattern."
  (:require [clojure.string :as str]
            [my-idea.workspace :as workspace]))

(defonce events* (atom []))
(defonce active-run* (atom nil))
(defonce active-profile* (atom nil))
(defonce exit-state* (atom nil))
(defonce exit-code* (atom nil))
(defonce missing-tool* (atom nil))
(defonce listening?* (atom false))
(defonce listener-ready* (atom nil))

(defn- event-listen []
  (some-> (aget js/window "__TAURI__") (aget "event") (aget "listen")))

(defn- detect-missing-tool? [line]
  (or (str/includes? line "command not found")
      (str/includes? line "No such file or directory")
      (str/includes? line "could not start")
      (str/includes? line "executable not found"))

(defn- detect-missing-tool [line]
  (let [parts (str/split line #"\s+")]
    (when (seq parts)
      (str "Missing tool: " (first parts) ". Ensure it is installed and in PATH."))))

(defn init! []
  (when (and (workspace/native?) (not @listening?*))
    (when-let [listen (event-listen)]
      (reset! listening?* true)
      (reset! listener-ready*
              (listen "build-output"
                      (fn [^js event]
                        (let [e (js->clj (.-payload event) :keywordize-keys true)]
                          (when (= (:schema e) 1)
                            (handle-event! e)))))))))

(defn- after-listener [f]
  (if-let [ready @listener-ready*]
    (.then ready f)
    (f)))

(defn handle-event! [e]
  (let [run-id (:runId e)
        seq (:sequence e)
        stream (:stream e)
        line (:line e)
        state (:state e)
        profile (:profile e)
        code (:exitCode e)]
    (swap! active-profile* (fn [_] profile))
    (cond
      (= state "running")
      (do
        (reset! active-run* run-id)
        (reset! exit-state* nil)
        (reset! exit-code* nil)
        (reset! missing-tool* nil)
        (swap! events* conj (assoc e :_local-seq seq)))
      (= state "succeeded")
      (do
        (reset! exit-state* "succeeded")
        (reset! exit-code* code)
        (swap! events* conj (assoc e :_local-seq seq)))
      (= state "failed")
      (do
        (reset! exit-state* "failed")
        (reset! exit-code* code)
        (swap! events* conj (assoc e :_local-seq seq))
        (when (and (nil? @missing-tool*) (detect-missing-tool? line))
          (reset! missing-tool* (detect-missing-tool line))))
      (= state "cancelled")
      (do
        (reset! exit-state* "cancelled")
        (reset! exit-code* code)
        (swap! events* conj (assoc e :_local-seq seq)))
      :else
      (swap! events* conj (assoc e :_local-seq seq)))))

(defn get-events []
  @events*)

(defn get-active-run []
  @active-run*)

(defn get-active-profile []
  @active-profile*)

(defn get-exit-state []
  @exit-state*)

(defn get-exit-code []
  @exit-code*)

(defn get-missing-tool []
  @missing-tool*)

(defn clear! []
  (reset! events* [])
  (reset! active-run* nil)
  (reset! active-profile* nil)
  (reset! exit-state* nil)
  (reset! exit-code* nil)
  (reset! missing-tool* nil))

(defn has-active-build? []
  (some? @active-run*)))
