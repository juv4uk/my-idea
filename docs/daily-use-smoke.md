# Daily-use smoke · Щоденна перевірка my-idea

This is the **10-minute user acceptance path** for the current stabilization phase. It checks what a normal user needs before new features are considered.

## Fastest path: portable Web IDE

1. Open the generated `my-idea-web.html` artifact in a current Chromium/Chrome/Firefox browser.
2. Confirm the IDE renders and the status/footer is responsive.
3. Choose **Open** and select a small folder containing a `.lisp` file.
4. Open the file from the workspace tree.
5. Confirm the language indicator says **SENS**.
6. Edit the file to:

   ```lisp
   (+ 20 22)
   ```

7. Press **⚡ Виконати / Evaluate**. Expected result: `42`.
8. In the REPL enter `:мова ук`, then:

   ```lisp
   (атом? 'мама)
   ```

   Expected: a Ukrainian true answer.
9. Enter a runtime error such as `(unknown-op 1)`. Confirm the diagnostic includes a useful message and source position.
10. Enter a syntax error such as `(+ 1`. Confirm the unclosed-list diagnostic points at the source.
11. Save the document. In Web mode, verify the browser produces the expected `.lisp` download.

## Desktop/Tauri acceptance

The desktop build must additionally prove:

- workspace Open / Save / Save As operate on real files;
- reopen restores a valid workspace without corrupting files;
- the pinned SENS revision reported by the app matches the build pin;
- no ambient `my-lisp`/SENS executable from `PATH` is silently used.

### Deliberately hidden during P0

- **▶ Run** — hidden until #79 provides a backend-owned pinned SENS subprocess adapter with Stop.
- **Compile (cml)** — hidden from the primary UI while strict CML integration is experimental and CML lock drift is tracked upstream.

The canonical daily-use execution action today is **⚡ Виконати**.

## Pass rule

P0 is acceptable for ordinary SENS editing only when the full applicable path above works without an agent repairing the environment first. Any failure should be recorded against #77 with the exact platform, build/revision and failing step.
