;; @deprecated — safety-net only, не приймати нові фічі мови сюди
;; In the web/PWA build this interpreter is used only as a temporary
;; fallback while the canonical WASM engine (my-idea.wasm) loads asynchronously.
;; Do not add new language features here; implement them in crates/my-lisp instead.
;;
;; @deprecated — safety-net only, не приймати нові фічі мови сюди
;; У веб/PWA збірці цей інтерпретатор використовується лише як тимчасовий
;; fallback доки канонічний WASM-рушій (my-idea.wasm) завантажується асинхронно.
;; Не додавати нові можливості мови тут; реалізовувати їх у crates/my-lisp.
;;
;; @deprecated — safety-net only, не приймати нові фічі мови сюди
;; Im Web/PWA-Build wird dieser Interpreter nur als temporärer Fallback
;; verwendet, während die kanonische WASM-Engine (my-idea.wasm) asynchron lädt.
;; Keine neuen Sprachfeatures hier hinzufügen; in crates/my-lisp implementieren.
(ns my-idea.language
  (:require [cljs.reader :as reader]
            [clojure.string :as str]))

;; Exact fractions keep arithmetic deterministic instead of leaking JavaScript floating-point values.
;; Точні дроби зберігають арифметику детермінованою без витоку JavaScript floating-point значень.
;; Exakte Brüche halten die Arithmetik deterministisch, ohne JavaScript-Gleitkommawerte durchzulassen.
(deftype Rational [numerator denominator]
  IEquiv
  (-equiv [_ other]
    (and (instance? Rational other)
         (= numerator (.-numerator ^Rational other))
         (= denominator (.-denominator ^Rational other))))
  IPrintWithWriter
  (-pr-writer [_ writer _]
    (-write writer (str numerator "/" denominator)))
  Object
  (toString [_] (str numerator "/" denominator)))

(defn rational? [value] (instance? Rational value))

(defn- gcd [left right]
  (loop [a (js/Math.abs left) b (js/Math.abs right)]
    (if (zero? b) a (recur b (mod a b)))))

(defn- rational [numerator denominator]
  (when-not (and (js/Number.isSafeInteger numerator) (js/Number.isSafeInteger denominator))
    (throw (js/Error. "rational number overflow · переповнення раціонального числа · Überlauf der rationalen Zahl")))
  (when (zero? denominator)
    (throw (js/Error. "division by zero · ділення на нуль · Division durch null")))
  (let [sign (if (neg? denominator) -1 1)
        divisor (gcd numerator denominator)
        numerator (/ (* sign numerator) divisor)
        denominator (/ (js/Math.abs denominator) divisor)]
    (if (= denominator 1) numerator (Rational. numerator denominator))))

(defn- fraction-parts [value]
  (cond
    (rational? value) [(.-numerator ^Rational value) (.-denominator ^Rational value)]
    (and (number? value) (js/Number.isSafeInteger value)) [value 1]
    :else (throw (js/Error. "expects exact integers or rational numbers"))))

(defn- exact? [value]
  (or (rational? value) (and (number? value) (js/Number.isSafeInteger value))))

(defn- to-float [value]
  (if (rational? value)
    (/ (.-numerator ^Rational value) (.-denominator ^Rational value))
    value))

(defn- do-add [a b]
  (if (and (exact? a) (exact? b))
    (let [[an ad] (fraction-parts a)
          [bn bd] (fraction-parts b)]
      (rational (+ (* an bd) (* bn ad)) (* ad bd)))
    (+ (to-float a) (to-float b))))

(defn- exact-add
  ([] 0)
  ([value] value)
  ([value & others] (reduce do-add value others)))

(defn- do-sub [a b]
  (if (and (exact? a) (exact? b))
    (let [[an ad] (fraction-parts a)
          [bn bd] (fraction-parts b)]
      (rational (- (* an bd) (* bn ad)) (* ad bd)))
    (- (to-float a) (to-float b))))

(defn- exact-subtract
  ([value]
   (if (exact? value)
     (let [[n d] (fraction-parts value)] (rational (- n) d))
     (- (to-float value))))
  ([value & others] (reduce do-sub value others)))

(defn- do-mul [a b]
  (if (and (exact? a) (exact? b))
    (let [[an ad] (fraction-parts a)
          [bn bd] (fraction-parts b)]
      (rational (* an bn) (* ad bd)))
    (* (to-float a) (to-float b))))

(defn- exact-multiply
  ([] 1)
  ([value] value)
  ([value & others] (reduce do-mul value others)))

(defn- do-div [a b]
  (if (and (exact? a) (exact? b))
    (let [[an ad] (fraction-parts a)
          [bn bd] (fraction-parts b)]
      (rational (* an bd) (* ad bn)))
    (/ (to-float a) (to-float b))))

(defn- exact-divide
  ([value]
   (if (exact? value)
     (let [[n d] (fraction-parts value)] (rational d n))
     (/ 1 (to-float value))))
  ([value & others] (reduce do-div value others)))

(defn- lisp-atom? [value]
  (if (or (not (seq? value)) (empty? value)) 't '()))

(defn- require-cell [name value]
  (when-not (and (seq? value) (seq value))
    (throw (js/Error. (str name " expects a non-empty list"))))
  value)

(defn- lisp-car [value]
  (first (require-cell "car" value)))

(defn- lisp-cdr [value]
  (rest (require-cell "cdr" value)))

(defn- lisp-eq [left right]
  (when-not (and (or (not (seq? left)) (empty? left)) (or (not (seq? right)) (empty? right)))
    (throw (js/Error. "eq expects two atoms")))
  (if (= left right) 't '()))

(defn- lisp-bool [b] (if b 't '()))

(def builtins
  {'+ exact-add, '- exact-subtract, '* exact-multiply, '/ exact-divide,
   '= (fn [a b] (lisp-bool (if (or (exact? a) (exact? b)) (= (to-float a) (to-float b)) (= a b)))),
   '< (fn [a b] (lisp-bool (< (to-float a) (to-float b)))),
   '> (fn [a b] (lisp-bool (> (to-float a) (to-float b)))),
   'str str, 'list list, 'vector vector, 'count count,
   'atom lisp-atom?, 'eq lisp-eq, 'car lisp-car, 'cdr lisp-cdr, 'cons cons})

(defn- expand-quotes [forms]
  (loop [remaining forms result []]
    (if-let [form (first remaining)]
      (if (and (= form (symbol "'")) (next remaining))
        (let [target (second remaining)
              expanded-target (if (sequential? target) (expand-quotes target) target)]
          (recur (drop 2 remaining)
                 (conj result (list 'quote expanded-target))))
        (recur (rest remaining)
               (conj result (if (sequential? form) (expand-quotes form) form))))
      (if (vector? forms) result (apply list result)))))

(defn parse-program [source]
  (let [spaced-source (str/replace source "'" "' ")]
    (expand-quotes (reader/read-string (str "[" spaced-source "]")))))

(declare evaluate)

(defn- closure [parameters body environment]
  {::closure true :parameters parameters :body body :environment (atom environment)})

(defn- closure? [value] (true? (::closure value)))

(defn- apply-closure [value arguments output]
  (let [{:keys [parameters body environment]} value]
    (when-not (= (count parameters) (count arguments))
      (throw (js/Error. (str "lambda arity mismatch · невідповідна кількість аргументів lambda · falsche Lambda-Stelligkeit: " (count arguments)))))
    (reduce (fn [[_ env out] expression] (evaluate expression env out))
            [nil (merge @environment (zipmap parameters arguments)) output]
            body)))

(defn- evaluate-cond [clauses environment output]
  (if-let [clause (first clauses)]
    (do
      (when-not (and (sequential? clause) (= 2 (count clause)))
        (throw (js/Error. "cond expects (test expression) clauses")))
      (let [[condition environment output] (evaluate (first clause) environment output)]
        (if (not= condition '())
          (evaluate (second clause) environment output)
          (recur (rest clauses) environment output))))
    [nil environment output]))

(defn- evaluate-list [form environment output]
  (let [operator (first form)]
    (case operator
      quote [(second form) environment output]
      lambda (let [parameters (second form) body (drop 2 form)]
               (when-not (and (sequential? parameters) (every? symbol? parameters) (seq body))
                 (throw (js/Error. "lambda expects symbol parameters and a body · lambda очікує параметри-символи й тіло · lambda erwartet Symbolparameter und einen Rumpf")))
               [(closure parameters body environment) environment output])
      cond (evaluate-cond (rest form) environment output)
      if (let [[condition environment output] (evaluate (second form) environment output)]
           (evaluate (if (not= condition '()) (nth form 2) (nth form 3 nil)) environment output))
      def (let [name (second form)
                [value environment output] (evaluate (nth form 2) environment output)
                next-environment (assoc environment name value)]
            (when (closure? value)
              (reset! (:environment value) next-environment))
            [value next-environment output])
      println (let [[values environment output]
                    (reduce (fn [[values env out] expression]
                              (let [[value next-env next-out] (evaluate expression env out)]
                                [(conj values value) next-env next-out]))
                            [[] environment output]
                            (rest form))]
                [nil environment (conj output (str/join " " values))])
      (let [[function environment output] (evaluate operator environment output)
            [arguments environment output]
            (reduce (fn [[values env out] expression]
                      (let [[value next-env next-out] (evaluate expression env out)]
                        [(conj values value) next-env next-out]))
                    [[] environment output]
                    (rest form))]
        (when-not (or (fn? function) (closure? function))
          (throw (js/Error. (str operator " is not callable"))))
        (if (closure? function)
          (apply-closure function arguments output)
          [(apply function arguments) environment output])))))

(defn evaluate [form environment output]
  (cond
    (symbol? form) (if (contains? environment form)
                     [(get environment form) environment output]
                     (throw (js/Error. (str "Unknown symbol: " form))))
    (and (seq? form) (empty? form)) ['() environment output]
    (seq? form) (evaluate-list form environment output)
    :else [form environment output]))

(defn run-program [source]
  (let [forms (parse-program source)
        initial (merge builtins {'pi js/Math.PI, 't 't})
        [value environment output]
        (reduce (fn [[_ env out] form] (evaluate form env out)) [nil initial []] forms)]
    {:value value :environment environment :output output :forms forms}))
