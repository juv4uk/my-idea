# Windows ARM64 releases · Релізи Windows ARM64 · Windows-ARM64-Releases

## English

Windows ARM64 is the third completed distribution boundary after signed Android packages and the installable PWA. The release workflow cross-compiles the native Tauri application for `aarch64-pc-windows-msvc` on a Windows runner and publishes a versioned NSIS installer such as `my-idea_0.3.7_arm64-setup.exe`. Only the portable `my-idea-web.html` asset deliberately omits the version.

CI activates the ARM64 MSVC toolchain and verifies the application executable's PE machine field before upload. The required value is `0xAA64`; a renamed x64 executable cannot pass this check. The NSIS launcher itself may use a different bootstrap architecture, so verification targets the installed application payload before bundling.

Test the installer on physical Windows-on-ARM hardware when available. Verify startup, CodeMirror editing, the bottom programming-language switcher, my-lisp execution, file dialogs, save operations, and uninstall. Microsoft Store/MSIX packaging and store signing remain a later roadmap boundary.

## Українська

Windows ARM64 — третя завершувана межа розповсюдження після підписаних Android-пакетів і встановлюваної PWA. Release workflow крос-компілює нативну Tauri-програму для `aarch64-pc-windows-msvc` на Windows runner та публікує версійний NSIS installer, наприклад `my-idea_0.3.7_arm64-setup.exe`. Лише portable asset `my-idea-web.html` навмисно не містить версії.

CI активує ARM64 MSVC toolchain і перед завантаженням перевіряє поле machine у PE-заголовку програми. Обов’язкове значення — `0xAA64`; перейменований x64-файл не пройде перевірку. Сам NSIS launcher може використовувати іншу bootstrap-архітектуру, тому перевіряється payload встановлюваної програми перед пакуванням.

За наявності слід тестувати installer на фізичному Windows-on-ARM пристрої. Перевіряються запуск, редагування CodeMirror, нижній перемикач мов програмування, виконання my-lisp, файлові діалоги, збереження та видалення програми. Microsoft Store/MSIX-пакування й store-підпис залишаються окремою пізнішою межею roadmap.

## Deutsch

Windows ARM64 ist nach signierten Android-Paketen und der installierbaren PWA die dritte abzuschließende Distributionsgrenze. Der Release-Workflow cross-kompiliert die native Tauri-Anwendung auf einem Windows-Runner für `aarch64-pc-windows-msvc` und veröffentlicht einen versionierten NSIS-Installer wie `my-idea_0.3.7_arm64-setup.exe`. Nur das portable Asset `my-idea-web.html` enthält absichtlich keine Version.

CI aktiviert die ARM64-MSVC-Werkzeugkette und prüft vor dem Upload das PE-Maschinenfeld der Anwendung. Der erforderliche Wert ist `0xAA64`; eine umbenannte x64-Datei besteht diese Prüfung nicht. Der NSIS-Launcher selbst kann eine andere Bootstrap-Architektur verwenden, deshalb wird die installierte Anwendungsnutzlast vor dem Bündeln geprüft.

Der Installer soll nach Möglichkeit auf echter Windows-on-ARM-Hardware getestet werden. Zu prüfen sind Start, CodeMirror-Bearbeitung, der untere Programmiersprachenumschalter, my-lisp-Ausführung, Dateidialoge, Speichern und Deinstallation. Microsoft-Store-/MSIX-Paketierung und Store-Signatur bleiben eine spätere Roadmap-Grenze.
