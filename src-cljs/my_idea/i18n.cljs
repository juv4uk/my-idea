(ns my-idea.i18n
  "UI strings and static language/theme/programming-language tables — pure
  data and lookups, no app state. Split out of core.cljs to keep that
  namespace from being the place every new feature's data lands in.")

(def messages
  {"en" {:open "Open folder" :new-file "New File" :save "Save" :save-as "Save As" :run "Run" :files "Explorer" :console "Console" :ast "Language Lab / AST" :preview "Preview" :ecosystem "Ecosystem"
         :themes {"auto" "Auto" "light" "Day" "dark" "Night" "sepia" "Sepia" "signal" "Signal" "amber" "Amber" "forest" "Forest"}}
   "uk" {:open "Відкрити папку" :new-file "Новий файл" :save "Зберегти" :save-as "Зберегти як" :run "Запустити" :files "Файли" :console "Консоль" :ast "Лабораторія мов / AST" :preview "Попередній перегляд" :ecosystem "Екосистема"
         :themes {"auto" "Авто" "light" "День" "dark" "Ніч" "sepia" "Сепія" "signal" "Сигнал" "amber" "Бурштин" "forest" "Ліс"}}
   "de" {:open "Ordner öffnen" :new-file "Neue Datei" :save "Speichern" :save-as "Speichern unter" :run "Starten" :files "Explorer" :console "Konsole" :ast "Sprachlabor / AST" :preview "Vorschau" :ecosystem "Ökosystem"
         :themes {"auto" "Auto" "light" "Tag" "dark" "Nacht" "sepia" "Sepia" "signal" "Signal" "amber" "Bernstein" "forest" "Wald"}}})

(defn t [language key] (get-in messages [language key]))

(def languages ["uk" "de" "en"])
(def themes ["auto" "light" "dark" "sepia" "signal" "amber" "forest"])
(def programming-languages ["my-lisp" "clojurescript" "rust" "markdown" "mermaid" "text"])
(def programming-language-labels {"my-lisp" "my-lisp" "clojurescript" "ClojureScript" "rust" "Rust" "markdown" "Markdown" "mermaid" "Mermaid" "text" "Text"})
(def language-labels {"uk" "UA" "de" "DE" "en" "EN"})
(def theme-icons {"auto" "◐" "light" "☀" "dark" "☾" "sepia" "◉" "signal" "⌁" "amber" "◆" "forest" "♣"})
