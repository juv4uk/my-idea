import { cp, mkdir, rm } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';

// Step 1: Compile my-lisp to WebAssembly for the browser/PWA build.
// my-lisp lives in its own repository (github.com/juv4uk/my-lisp) and is
// vendored here as the external/my-lisp git submodule; wasm-pack needs a
// local Cargo.toml, which a Cargo git dependency alone cannot provide.
//
// Крок 1: Компілюємо my-lisp до WebAssembly для браузерної/PWA збірки.
// my-lisp живе у власному репозиторії (github.com/juv4uk/my-lisp) і тут
// підключений як git submodule external/my-lisp; wasm-pack потребує
// локального Cargo.toml, якого сама Cargo git-залежність не дає.
//
// Schritt 1: my-lisp für den Browser/PWA-Build zu WebAssembly kompilieren.
// my-lisp lebt in einem eigenen Repository (github.com/juv4uk/my-lisp) und
// ist hier als Git-Submodul external/my-lisp eingebunden; wasm-pack braucht
// eine lokale Cargo.toml, die eine reine Cargo-Git-Abhängigkeit nicht liefert.
const wasm = spawnSync(
  'wasm-pack',
  ['build', 'external/my-lisp/crates/my-lisp-wasm', '--target', 'web', '--out-dir', '../../../../public/wasm', '--no-pack'],
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
