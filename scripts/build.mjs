import { cp, mkdir, rm } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';

// Step 1: Compile my-lisp to WebAssembly for the browser/PWA build.
// Крок 1: Компілюємо my-lisp до WebAssembly для браузерної/PWA збірки.
// Schritt 1: my-lisp für den Browser/PWA-Build zu WebAssembly kompilieren.
const wasm = spawnSync(
  'wasm-pack',
  ['build', 'crates/my-lisp-wasm', '--target', 'web', '--out-dir', '../../public/wasm', '--no-pack'],
  { stdio: 'inherit' }
);
if (wasm.status !== 0) process.exit(wasm.status ?? 1);

// Step 2: Compile ClojureScript (shadow-cljs release app).
// Крок 2: Компілюємо ClojureScript (shadow-cljs release app).
// Schritt 2: ClojureScript kompilieren (shadow-cljs release app).
const cljs = spawnSync(process.execPath, ['node_modules/shadow-cljs/cli/runner.js', 'release', 'app'], {
  stdio: 'inherit'
});
if (cljs.status !== 0) process.exit(cljs.status ?? 1);

// Step 3: Assemble clean dist/ for Tauri.
// Крок 3: Збираємо чистий dist/ для Tauri.
// Schritt 3: Sauberes dist/-Verzeichnis für Tauri zusammenstellen.
await rm('dist', { recursive: true, force: true });
await mkdir('dist', { recursive: true });
await cp('public', 'dist', { recursive: true });
console.log('WASM + ClojureScript IDE written to dist/');
