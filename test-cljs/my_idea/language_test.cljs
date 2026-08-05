(ns my-idea.language-test
  (:require [cljs.test :refer-macros [deftest is testing]]
            [my-idea.language :as language]))

(deftest mccarthy-primitives
  (testing "atom recognizes atoms and the empty list"
    (is (= true (:value (language/run-program "(atom (quote radio))"))))
    (is (= true (:value (language/run-program "(atom (quote ()))"))))
    (is (= false (:value (language/run-program "(atom (quote (radio antenna)))")))))
  (testing "car, cdr and cons build and inspect lists"
    (is (= 'radio (:value (language/run-program "(car (quote (radio antenna)))"))))
    (is (= '(antenna) (:value (language/run-program "(cdr (quote (radio antenna)))"))))
    (is (= '(radio antenna) (:value (language/run-program "(cons (quote radio) (quote (antenna)))")))))
  (testing "eq compares atoms"
    (is (= true (:value (language/run-program "(eq (quote radio) (quote radio))"))))
    (is (= false (:value (language/run-program "(eq (quote radio) (quote antenna))")))))
  (testing "cond selects the first true clause"
    (is (= 'low (:value (language/run-program "(cond ((< 5 1) (quote high)) (t (quote low)))"))))))

(deftest primitive-errors
  (is (thrown-with-msg? js/Error #"car expects" (language/run-program "(car (quote ()))")))
  (is (thrown-with-msg? js/Error #"eq expects" (language/run-program "(eq (quote (a)) (quote (a)))")))
  (is (thrown-with-msg? js/Error #"cond expects" (language/run-program "(cond (t))"))))
