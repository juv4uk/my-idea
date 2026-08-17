(ns my-idea.preview)

;; Cache for loaded modules
(defonce modules (atom nil))

(defn- load-script! [src global-name]
  (js/Promise.
   (fn [resolve reject]
     (if-let [existing (aget js/window global-name)]
       (resolve existing)
       (let [script (js/document.createElement "script")]
         (set! (.-src script) src)
         (set! (.-onload script) #(resolve (aget js/window global-name)))
         (set! (.-onerror script) reject)
         (.appendChild js/document.head script))))))

(defn- load-modules! []
  (if-let [m @modules]
    (js/Promise.resolve m)
    (-> (js/Promise.all #js [(load-script! "./vendor/marked.min.js" "marked")
                            (load-script! "./vendor/mermaid.min.js" "mermaid")
                            (load-script! "./vendor/dompurify.min.js" "DOMPurify")])
        (.then (fn [[marked-obj mermaid-obj dompurify-obj]]
                 (let [m {:marked marked-obj :mermaid mermaid-obj :dompurify dompurify-obj}]
                   (reset! modules m)
                   (.initialize (:mermaid m) #js {:startOnLoad false})
                   m))))))

(defn- render-markdown! [marked-fn ^js dompurify-obj source element]
  (-> (js/Promise.resolve (.parse marked-fn source))
      (.then (fn [html]
               (set! (.-innerHTML element) (.sanitize dompurify-obj html))))))

(defn- render-mermaid! [mermaid-obj source element]
  (let [id "mermaid-preview-svg"]
    (-> (.render mermaid-obj id source)
        (.then (fn [result]
                 (set! (.-innerHTML element) (goog.object/get result "svg"))))
        (.catch (fn [e]
                  (when-let [err-el (.getElementById js/document (str "d" id))]
                    (.remove err-el))
                  (set! (.-innerHTML element) (str "<pre class='error'>" (.-message e) "</pre>")))))))

(defn render! [source mode element]
  (-> (load-modules!)
      (.then (fn [{:keys [marked mermaid dompurify]}]
               (case mode
                 "markdown" (render-markdown! marked dompurify source element)
                 "mermaid" (render-mermaid! mermaid source element)
                 (set! (.-innerHTML element) ""))))
      (.catch (fn [e]
                (set! (.-innerHTML element) (str "<pre class='error'>Failed to load preview modules: " (.-message e) "</pre>"))))))
