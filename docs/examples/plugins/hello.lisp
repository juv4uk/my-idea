;; Приклад плагіна для my-idea (~/.config/my-idea/plugins/hello.lisp) ·
;; Example my-idea plugin.
;;
;; Скопіюй у ~/.config/my-idea/plugins/ і натисни "Reload plugins" (кнопка
;; кружечка в шапці my-idea) — без перезапуску застосунку.
;; Copy into ~/.config/my-idea/plugins/ and press "Reload plugins" (the
;; circular-arrow button in my-idea's header) — no app restart needed.

;; Команда: обгортає виділений текст квадратними дужками.
;; A command: wraps the current selection in brackets.
(editor/register-command "bracket-selection"
  (lambda ()
    (editor/replace-selection (string-append "[" (string-append (editor/selection) "]"))))
  "Wrap the current selection in brackets")

;; Прив'язка клавіші до команди вище — своя, не CodeMirror-дефолтна.
;; A keymap binding for the command above.
(editor/keymap "Ctrl-Shift-b" "bracket-selection")

;; Хук на подію: after-open спрацьовує щоразу, коли my-idea відкриває файл.
;; An event hook: after-open fires every time my-idea opens a file.
(editor/on "after-open" "greet-on-open"
  (lambda ()
    (editor/message "Файл відкрито - плагін живий · file opened - plugin is alive")))
