(ns my-idea.workspace-test
  (:require [cljs.test :refer-macros [deftest is testing]]
            [my-idea.workspace :as workspace]))

(deftest tree-html-escapes-workspace-controlled-names-and-paths
  (testing "file names cannot break out of text or data-path contexts"
    (let [html (workspace/tree-html
                [{:name "<img src=x onerror='attack'>"
                  :path "folder/'><script>attack()</script>"
                  :directory false
                  :children []}])]
      (is (= "<button class='file' data-path='folder/&#39;&gt;&lt;script&gt;attack()&lt;/script&gt;'>&lt;img src=x onerror=&#39;attack&#39;&gt;</button>"
             html))
      (is (not (.includes html "<script>")))
      (is (not (.includes html "<img")))))

  (testing "directory names are escaped recursively"
    (is (= "<details open><summary>▾ &lt;unsafe&amp;dir&gt;</summary><div><button class='file' data-path='safe/file.my'>safe.my</button></div></details>"
           (workspace/tree-html
            [{:name "<unsafe&dir>"
              :path "ignored"
              :directory true
              :children [{:name "safe.my"
                          :path "safe/file.my"
                          :directory false
                          :children []}]}])))))
