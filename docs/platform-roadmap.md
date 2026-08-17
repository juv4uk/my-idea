# Platform roadmap · Дорожня карта платформ · Plattform-Roadmap

The release order is deliberate: finish and test one distribution boundary before adding the maintenance cost of the next one.

Порядок релізів свідомий: завершуємо й тестуємо одну межу розповсюдження, перш ніж додавати вартість підтримки наступної.

Die Release-Reihenfolge ist bewusst gewählt: Eine Distributionsgrenze wird fertiggestellt und getestet, bevor die Wartungskosten der nächsten hinzukommen.

1. **Android APK/AAB** — signed CI pipeline and direct-device testing · підписаний CI pipeline і тестування на пристроях · signierte CI-Pipeline und Tests auf Geräten.
2. **PWA** — installable, offline-capable web release · встановлюваний web-реліз з офлайн-режимом · installierbares, offlinefähiges Web-Release.
3. **Windows ARM64** — native packages for Windows on ARM · нативні пакети для Windows on ARM · native Pakete für Windows on ARM.
4. **Linux ARM64 Flatpak** — portable ARM desktop package · переносний ARM desktop-пакет · portables ARM-Desktop-Paket.
5. **AUR** — source-oriented Arch Linux distribution · орієнтоване на вихідний код розповсюдження Arch Linux · quellcodeorientierte Arch-Linux-Distribution.
6. **Google Play** — store publication after APK field testing · публікація після польового тестування APK · Store-Veröffentlichung nach APK-Praxistests.
7. **iPadOS/iOS** — touch-first Apple mobile release · Apple mobile-реліз із touch-first інтерфейсом · touchorientiertes mobiles Apple-Release.
8. **Microsoft Store and Mac App Store** — signed store channels after direct packages stabilize · підписані store-канали після стабілізації прямих пакетів · signierte Store-Kanäle nach Stabilisierung direkter Pakete.

## PWA behavior · Поведінка PWA · PWA-Verhalten

The PWA manifest provides a standalone install surface and 192/512-pixel icons. The service worker uses a network-first strategy for same-origin resources and falls back to the last verified cached shell when offline. It registers only for HTTP/HTTPS, so Tauri desktop and mobile protocols remain unaffected.

PWA manifest забезпечує standalone-встановлення та іконки 192/512 пікселів. Service worker використовує network-first для ресурсів того самого origin і без мережі повертається до останньої перевіреної кешованої оболонки. Він реєструється лише для HTTP/HTTPS, тому протоколи Tauri desktop і mobile не змінюються.

Das PWA-Manifest bietet eine eigenständige Installation und Symbole mit 192/512 Pixeln. Der Service Worker verwendet für Ressourcen gleichen Ursprungs eine Network-first-Strategie und greift offline auf die letzte geprüfte Cache-Hülle zurück. Er registriert sich nur für HTTP/HTTPS, sodass Tauri-Desktop- und Mobile-Protokolle unbeeinflusst bleiben.

## Windows ARM64 boundary · Межа Windows ARM64 · Windows-ARM64-Grenze

Release tags cross-compile `aarch64-pc-windows-msvc`, verify PE machine `0xAA64`, and publish a clearly named ARM64 NSIS installer. Details and the physical-device checklist are in [`windows-arm64.md`](windows-arm64.md).

Release-теги крос-компілюють `aarch64-pc-windows-msvc`, перевіряють PE machine `0xAA64` і публікують чітко названий ARM64 NSIS installer. Подробиці й checklist фізичного пристрою містяться у [`windows-arm64.md`](windows-arm64.md).

Release-Tags cross-kompilieren `aarch64-pc-windows-msvc`, prüfen die PE-Maschine `0xAA64` und veröffentlichen einen eindeutig benannten ARM64-NSIS-Installer. Details und die Prüfliste für echte Geräte stehen in [`windows-arm64.md`](windows-arm64.md).
