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

(deftest exact-rational-division
  (testing "division returns a reduced exact rational"
    (is (= "5/336" (pr-str (:value (language/run-program "(/ 5 6 8 7)")))))
    (is (= 2 (:value (language/run-program "(/ 8 4)"))))
    (is (= "3/2" (pr-str (:value (language/run-program "(/ (/ 2 3))"))))))
  (testing "division by zero is explicit and trilingual"
    (is (thrown-with-msg? js/Error #"division by zero" (language/run-program "(/ 1 0)")))))

(deftest lambda-and-derived-functions
  (is (= 'antenna (:value (language/run-program "(def second (lambda (values) (car (cdr values)))) (second (quote (radio antenna)))"))))
  (is (= '(radio antenna) (:value (language/run-program "((lambda (left right) (cons left (cons right (quote ())))) (quote radio) (quote antenna))")))))

(deftest demo-source-with-single-quote-sugar
  (let [source "; my-lisp · Rust/CLJS shared contract\n(def greeting \"Hello · Привіт · Hallo\")\n(def second (lambda (values) (car (cdr values))))\n(cons greeting (cons (second '(radio antenna)) '()))"]
    (is (= '("Hello · Привіт · Hallo" antenna) (:value (language/run-program source))))))
