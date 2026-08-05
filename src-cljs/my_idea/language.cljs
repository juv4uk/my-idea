(ns my-idea.language
  (:require [cljs.reader :as reader]
            [clojure.string :as str]))

(def builtins
  {'+ +, '- -, '* *, '/ /, '= =, '< <, '> >,
   'str str, 'list list, 'vector vector, 'count count})

(defn parse-program [source]
  (reader/read-string (str "[" source "]")))

(declare evaluate)

(defn- evaluate-list [form environment output]
  (let [operator (first form)]
    (case operator
      quote [(second form) environment output]
      if (let [[condition environment output] (evaluate (second form) environment output)]
           (evaluate (if condition (nth form 2) (nth form 3 nil)) environment output))
      def (let [[value environment output] (evaluate (nth form 2) environment output)]
            [value (assoc environment (second form) value) output])
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
        (when-not (fn? function)
          (throw (js/Error. (str operator " is not callable"))))
        [(apply function arguments) environment output]))))

(defn evaluate [form environment output]
  (cond
    (symbol? form) (if (contains? environment form)
                     [(get environment form) environment output]
                     (throw (js/Error. (str "Unknown symbol: " form))))
    (seq? form) (evaluate-list form environment output)
    :else [form environment output]))

(defn run-program [source]
  (let [forms (parse-program source)
        initial (merge builtins {'pi js/Math.PI})
        [value environment output]
        (reduce (fn [[_ env out] form] (evaluate form env out)) [nil initial []] forms)]
    {:value value :environment environment :output output :forms forms}))
