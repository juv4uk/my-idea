import { mkdir, rm } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';

// Step 1: Compile the sens engine (formerly my-lisp) to WebAssembly for the
// browser/PWA build. The engine lives in its own repository
// (github.com/juv4uk/sens) and is vendored here as the external/sens git
// submodule; wasm-pack needs a local Cargo.toml, which a Cargo git dependency
// alone cannot provide.
//
// Крок 1: Компілюємо движок sens (колишній my-lisp) до WebAssembly для
// браузерної/PWA збірки. Двиг живе у власному репозиторії
// (github.com/juv4uk/sens) і тут підключений як git submodule external/sens;
// wasm-pack потребує локального Cargo.toml, якого сама Cargo git-залежність
// не дає.
//
// Schritt 1: Die Sens-Engine (vormals my-lisp) für die Browser-/PWA-Build
// nach WebAssembly kompilieren. Die Engine liegt im eigenen Repository
// (github.com/juv4uk/sens) und ist hier als Git-Submodul external/sens
// eingebunden; wasm-pack braucht eine lokale Cargo.toml, die eine reine
// Cargo-Git-Abhängigkeit nicht liefert.
// --out-name pins the artifact basename. wasm-pack defaults it to the crate's
// package name, which became `sens-wasm` in the external/my-lisp ->
// external/sens rename (40ec78f) — the bundle would then emit sens_wasm.js /
// sens_wasm_bg.wasm while wasm-loader.js, sw.js, make-portable-web.mjs and
// the conformance test all still resolve my_lisp_wasm*. Without this flag the
// web job fails at make-portable-web with ENOENT on
// dist/wasm/my_lisp_wasm_bg.wasm.
//
// --out-name фіксує базову назву артефактів. За замовчуванням wasm-pack
// бере ім'я пакета crate, яким після ребрендингу external/my-lisp ->
// external/sens (40ec78f) став `sens-wasm` — тоді збірка дає
// sens_wasm.js / sens_wasm_bg.wasm, а wasm-loader.js, sw.js,
// make-portable-web.mjs та conformance-тест усе ще шукають my_lisp_wasm*.
// Без цього прапорця web-джоб падає у make-portable-web з ENOENT на
// dist/wasm/my_lisp_wasm_bg.wasm.
//
// Ohne --out-name nutzt wasm-pack den Paketnamen des Crate als Dateibasis,
// der seit dem Rename external/my-lisp -> external/sens (40ec78f)
// `sens-wasm` lautet — die Bundles hießen dann sens_wasm.js /
// sens_wasm_bg.wasm, während wasm-loader.js, sw.js, make-portable-web.mjs
// und der Conformance-Test weiterhin my_lisp_wasm* auflösen. Ohne diesen
// Schalter scheitert der Web-Job in make-portable-web mit ENOENT auf
// dist/wasm/my_lisp_wasm_bg.wasm.
const wasm = spawnSync(
  'wasm-pack',
  [
    'build',
    'external/sens/crates/sens-wasm',
    '--target',
    'web',
    '--out-dir',
    '../../../../public/wasm',
    '--out-name',
    'my_lisp_wasm',
    '--no-pack'
  ],
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
//
// Shells out to the system `cp` instead of node:fs/promises' cp/copyFile:
// on this repo's DrvFs mount, Node's copyFile (via libuv's copy_file_range
// attempt) fails with EPERM instead of falling back to a plain read+write
// the way Rust's std::fs::copy does — GNU coreutils' own cp doesn't hit
// this. Same DrvFs-permission-op class as the other issues documented in
// AGENTS.md, just a fourth spot.
//
// Шеляться до системного `cp` замість node:fs/promises' cp/copyFile: на
// цьому DrvFs-монтуванні Node's copyFile провалюється з EPERM замість
// fallback на read+write — GNU coreutils cp цього не має.
await rm('dist', { recursive: true, force: true });
await mkdir('dist', { recursive: true });
const copy = spawnSync('cp', ['-r', 'public/.', 'dist/'], { stdio: 'inherit' });
if (copy.status !== 0) process.exit(copy.status ?? 1);
console.log('WASM + ClojureScript IDE written to dist/');
