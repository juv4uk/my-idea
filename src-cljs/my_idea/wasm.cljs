(ns my-idea.wasm)

;; Single atom holding the initialised wasm-bindgen module or nil while loading.
;; Єдиний атом, що зберігає ініціалізований wasm-bindgen модуль або nil під час завантаження.
;; Einzelnes Atom, das das initialisierte wasm-bindgen-Modul oder nil während des Ladens enthält.
(defonce !module (atom nil))
(defonce !failed (atom false))

(defn ready?
  "Returns true once the WASM module has been loaded and initialised.
   Повертає true після завантаження та ініціалізації WASM-модуля.
   Gibt true zurück, sobald das WASM-Modul geladen und initialisiert wurde."
  []
  (some? @!module))

(defn failed?
  "Returns true if the WASM module failed to load.
   Повертає true якщо WASM-модуль не вдалося завантажити.
   Gibt true zurück, wenn das Laden des WASM-Moduls fehlgeschlagen ist."
  []
  @!failed)

(defn load!
  "Asynchronously fetches and initialises the my-lisp WASM module.
   Calls on-ready (no args) when the module is ready.
   Safe to call multiple times; subsequent calls are no-ops if already loaded.

   Асинхронно завантажує та ініціалізує WASM-модуль my-lisp.
   Викликає on-ready (без аргументів) коли модуль готовий.
   Безпечно викликати кілька разів; повторні виклики — no-op якщо вже завантажено.

   Lädt und initialisiert das my-lisp-WASM-Modul asynchron.
   Ruft on-ready (ohne Argumente) auf, wenn das Modul bereit ist.
   Mehrfachaufrufe sind sicher; weitere Aufrufe sind no-ops, falls bereits geladen."
  [on-ready]
  (when-not (ready?)
    ;; js/import cannot be used directly in shadow-cljs release builds because
    ;; the Google Closure Compiler treats `import` as a reserved keyword.
    ;; We delegate to the plain-JS shim (public/wasm-loader.js) which calls
    ;; import() natively and exposes the result via window.loadMyLispWasm.
    ;;
    ;; js/import не можна використовувати напряму в release-збірці shadow-cljs,
    ;; бо Closure Compiler вважає `import` зарезервованим словом.
    ;; Делегуємо до plain-JS шима (public/wasm-loader.js), який викликає
    ;; import() нативно і виставляє результат через window.loadMyLispWasm.
    ;;
    ;; js/import kann in shadow-cljs-Release-Builds nicht direkt verwendet werden,
    ;; da der Closure Compiler `import` als reserviertes Schlüsselwort behandelt.
    ;; Wir delegieren an den plain-JS-Shim (public/wasm-loader.js).
    (-> (js/window.loadMyLispWasm)
        (.then (fn [js-module]
                 (reset! !module js-module)
                 (on-ready)))
        (.catch (fn [err]
                  (reset! !failed true)
                  (js/console.error
                   "my-lisp WASM failed to load – WebAssembly is likely unsupported or blocked"
                   err)
                  (on-ready))))))

(defn evaluate
  "Calls the WASM evaluate(source) function.
   Returns a JS Promise resolving to {:value :output :ast :engine}.
   Must only be called when (ready?) is true.

   Викликає WASM-функцію evaluate(source).
   Повертає JS Promise, що резолвиться до {:value :output :ast :engine}.
   Викликати лише якщо (ready?) є true.

   Ruft die WASM-Funktion evaluate(source) auf.
   Gibt ein JS-Promise zurück, das zu {:value :output :ast :engine} auflöst.
   Darf nur aufgerufen werden, wenn (ready?) true ist."
  [source mode]
  (js/Promise.resolve (.evaluate @!module source mode)))
