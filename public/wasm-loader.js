// Thin loader shim for the wasm-bindgen my-lisp module.
// Uses new Function('return import(u)')(url) to call dynamic import() from a plain
// (non-module) script. This keeps the file compatible with shadow-cljs release builds
// whose Google Closure Compiler cannot handle `import` as a keyword in CLJS source.
//
// Тонкий шим-завантажувач для wasm-bindgen-модуля my-lisp.
// Використовує new Function('return import(u)')(url), щоб викликати dynamic import()
// зі звичайного (не module) скрипта. Це сумісно з release-збіркою shadow-cljs,
// чий Google Closure Compiler не може обробляти `import` як ключове слово у CLJS.
//
// Dünner Loader-Shim für das wasm-bindgen my-lisp Modul.
// Verwendet new Function('return import(u)')(url) um dynamic import() aus einem
// normalen (Nicht-Modul) Skript aufzurufen. Kompatibel mit shadow-cljs-Release-Builds.

/**
 * Loads and initialises the my-lisp WASM module.
 * Returns a Promise that resolves to the initialised wasm-bindgen module object.
 *
 * Завантажує та ініціалізує WASM-модуль my-lisp.
 * Повертає Promise, що резолвиться до ініціалізованого wasm-bindgen модуля.
 *
 * Lädt und initialisiert das my-lisp-WASM-Modul.
 * Gibt ein Promise zurück, das zum initialisierten wasm-bindgen-Modul auflöst.
 *
 * @returns {Promise<object>}
 */
window.loadMyLispWasm = function () {
  // new Function avoids Closure Compiler keyword conflict with `import`.
  // new Function уникає конфлікту Closure Compiler з ключовим словом `import`.
  // new Function vermeidet den Schlüsselwortkonflikt des Closure Compilers mit `import`.
  var dynamicImport = new Function('u', 'return import(u)');
  return dynamicImport('./wasm/my_lisp_wasm.js').then(function (mod) {
    return mod.default().then(function () {
      return mod;
    });
  });
};

