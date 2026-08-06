const CACHE_NAME = 'my-idea-pwa-v1';
const APP_SHELL = [
  './index.html',
  './styles.css',
  './js/main.js',
  './manifest.webmanifest',
  './icons/pwa-192.png',
  './icons/pwa-512.png',
  './wasm-loader.js',
  './wasm/my_lisp_wasm.js',
  './wasm/my_lisp_wasm_bg.wasm',
  './vendor/marked.min.js',
  './vendor/mermaid.min.js'
];

// Cache the complete shell atomically so a partial install never replaces a working version.
// Кешуємо всю оболонку атомарно, щоб неповне встановлення не замінило робочу версію.
// Die gesamte Hülle wird atomar gecacht, damit keine unvollständige Installation eine funktionierende Version ersetzt.
self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => {
      return Promise.all(
        APP_SHELL.map((url) => {
          return cache.add(url).catch((err) => {
            console.error('Failed to cache', url, err);
          });
        })
      );
    })
  );
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys()
      .then((names) => Promise.all(names.filter((name) => name !== CACHE_NAME).map((name) => caches.delete(name))))
      .then(() => self.clients.claim()),
  );
});

// Prefer fresh same-origin files; use the last verified response when the network is unavailable.
// Надаємо перевагу свіжим локальним ресурсам, а без мережі використовуємо останню перевірену відповідь.
// Frische Dateien gleichen Ursprungs werden bevorzugt; ohne Netzwerk dient die letzte geprüfte Antwort als Ersatz.
self.addEventListener('fetch', (event) => {
  const requestUrl = new URL(event.request.url);
  if (event.request.method !== 'GET' || requestUrl.origin !== self.location.origin) return;

  event.respondWith(
    fetch(event.request)
      .then((response) => {
        if (response.ok) {
          const copy = response.clone();
          event.waitUntil(caches.open(CACHE_NAME).then((cache) => cache.put(event.request, copy)));
        }
        return response;
      })
      .catch(async () => {
        const cached = await caches.match(event.request);
        if (cached) return cached;
        if (event.request.mode === 'navigate') return caches.match('./index.html');
        throw new Error('Offline resource unavailable · Ресурс офлайн недоступний · Offline-Ressource nicht verfügbar');
      }),
  );
});
