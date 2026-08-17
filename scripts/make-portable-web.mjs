import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import process from 'node:process';

const [, , inputPath, outputPath] = process.argv;
if (!inputPath || !outputPath) {
  throw new Error('Usage: node scripts/make-portable-web.mjs <input.html> <output.html>');
}

const root = path.dirname(inputPath);
let html = await readFile(inputPath, 'utf8');

// EN: Inline local styles and scripts so the demo remains usable offline.
// UK: Вбудувати локальні стилі й скрипти, щоб демо працювало офлайн.
// DE: Lokale Stile und Skripte einbetten, damit die Demo offline funktioniert.
html = await replaceAsync(html, /<link\s+rel=["']stylesheet["']\s+href=["']([^"']+)["']\s*\/?>/gi,
  async (_, reference) => `<style>${await readLocal(reference)}</style>`);

// EN: Embed WASM as base64 and replace wasm-loader with embedded version.
// UK: Вбудувати WASM як base64 і замінити wasm-loader на embedded версію.
// DE: WASM als base64 einbetten und wasm-loader durch embedded Version ersetzen.
const wasmBytes = await readFile(path.join(root, 'wasm/my_lisp_wasm_bg.wasm'));
const wasmBase64 = wasmBytes.toString('base64');

// EN: Inline wasm-bindgen JS if available for true standalone operation.
// UK: Інлайнити wasm-bindgen JS якщо доступний для справжньої автономної роботи.
// DE: wasm-bindgen JS inline wenn verfügbar für echte eigenständige Operation.
let wasmJs;
try {
  wasmJs = await readFile(path.join(root, 'wasm/my_lisp_wasm.js'), 'utf8');
  // EN: Replace import.meta.url to avoid invalid URL errors when loaded via Blob URL.
  wasmJs = wasmJs.replaceAll('import.meta.url', 'location.href');
} catch (e) {
  console.warn('wasm/my_lisp_wasm.js not found, WASM-only mode');
}

// EN: Create embedded loader with base64 WASM data and inline wasm-bindgen JS.
// UK: Створити embedded loader з base64 WASM даними та інлайнити wasm-bindgen JS.
// DE: Embedded-Loader mit base64-WASM-Daten und inline wasm-bindgen JS erstellen.
let embeddedLoader = `
// Embedded WASM loader for standalone HTML artifact.
// Embedded WASM-Loader für standalone HTML-Artefakt.
function decodeBase64(base64) {
  const binaryString = atob(base64);
  const bytes = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return bytes;
}

// EN: Embedded WASM binary data.
// UK: Вбудовані WASM бінарні дані.
// DE: Eingebettete WASM-Binärdaten.
const embeddedWasmBytes = decodeBase64('${wasmBase64}');
`;

if (wasmJs) {
  embeddedLoader += `
// EN: Embedded wasm-bindgen JS text.
// UK: Вбудований текст wasm-bindgen JS.
// DE: Eingebetteter wasm-bindgen JS Text.
const embeddedWasmJs = ${JSON.stringify(wasmJs)};
`;
}

embeddedLoader += `
// EN: Shim for compatibility with existing wasm-loader interface.
// UK: Shim для сумісності з існуючим wasm-loader інтерфейсом.
// DE: Shim für Kompatibilität mit dem bestehenden wasm-loader-Interface.
window.loadMyLispWasm = function() {
  if (window.wasm_bindgen) {
    return Promise.resolve(window.wasm_bindgen);
  }
  // Use dynamic import with fetch override for embedded WASM.
  // Використовувати dynamic import з fetch override для вбудованого WASM.
  // dynamic import mit fetch-Override für eingebettetes WASM verwenden.
  const originalFetch = window.fetch;
  window.fetch = function(url, options) {
    const urlStr = url instanceof URL ? url.toString() : (typeof url === 'string' ? url : (url && url.url ? url.url : ''));
    console.log('fetch shim called with urlStr:', urlStr, 'url:', url);
    if (urlStr.endsWith('.wasm') || urlStr.includes('.wasm')) {
      return Promise.resolve(new Response(embeddedWasmBytes.buffer, {
        status: 200,
        headers: { 'content-type': 'application/wasm' }
      }));
    }
    return originalFetch.apply(this, arguments);
  };
  var dynamicImport = new Function('u', 'return import(u)');
  ${wasmJs ? `
  const wasmJsBlob = new Blob([embeddedWasmJs], { type: 'text/javascript' });
  const wasmJsUrl = URL.createObjectURL(wasmJsBlob);
  return dynamicImport(wasmJsUrl).then(function (mod) {` : `return dynamicImport('/wasm/my_lisp_wasm.js').then(function (mod) {`}
    return mod.default().then(function () {
      window.fetch = originalFetch;
      return mod;
    }).catch(function(e) {
      window.fetch = originalFetch;
      throw e;
    });
  });
};
`;

html = html.replace(
  /<script\s+src=["']\.\/wasm-loader\.js["']\s*><\/script>/,
  `<script>${embeddedLoader.replaceAll('</script>', '<\\/script>')}</script>`
);

html = await replaceAsync(html, /<script\s+src=["']([^"']+)["']\s*><\/script>/gi,
  async (_, reference) => `<script>${(await readLocal(reference)).replaceAll('</script>', '<\\/script>')}</script>`);

if (/<script[^>]+src=|<link[^>]+rel=["']stylesheet["']/i.test(html)) {
  throw new Error('Portable Web HTML still contains an external local asset reference');
}

await writeFile(outputPath, html, 'utf8');
console.log(`Standalone Web HTML created: ${outputPath} (${Buffer.byteLength(html)} bytes, WASM: ${wasmBytes.length} bytes)`);

async function readLocal(reference) {
  if (/^(?:[a-z]+:)?\/\//i.test(reference)) throw new Error(`Remote asset cannot be embedded: ${reference}`);
  return readFile(path.join(root, reference.replace(/^\.\//, '')), 'utf8');
}

async function replaceAsync(source, pattern, replacer) {
  const matches = [...source.matchAll(pattern)];
  const replacements = await Promise.all(matches.map((match) => replacer(...match)));
  let index = 0;
  return source.replace(pattern, () => replacements[index++]);
}

