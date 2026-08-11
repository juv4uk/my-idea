// PWA runs only on web origins; Tauri desktop/mobile protocols stay untouched.
// PWA працює лише на web-origin; протоколи Tauri desktop/mobile не змінюються.
// Die PWA läuft nur auf Web-Origins; Tauri-Desktop-/Mobile-Protokolle bleiben unberührt.
if ('serviceWorker' in navigator && /^(https?:)$/.test(location.protocol)) {
  addEventListener('load', () => navigator.serviceWorker.register('./sw.js'));
}
