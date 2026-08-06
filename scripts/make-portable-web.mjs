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

// EN: Inline wasm-bindgen JS if available, otherwise use simple fetch override.
// UK: Інлайнити wasm-bindgen JS якщо доступний, інакше використати просте перевизначення fetch.
// DE: wasm-bindgen JS inline wenn verfügbar, sonst einfaches fetch-Override verwenden.
let wasmJs;
try {
  wasmJs = await readFile(path.join(root, 'wasm/my_lisp_wasm.js'), 'utf8');
} catch (e) {
  console.warn('wasm/my_lisp_wasm.js not found, using fetch override only');
}

// EN: Create embedded loader with base64 WASM data using initSync.
// UK: Створити embedded loader з base64 WASM даними використовуючи initSync.
// DE: Embedded-Loader mit base64-WASM-Daten unter Verwendung von initSync erstellen.
const embeddedLoader = `
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

${wasmJs || ''}

// EN: Shim for compatibility with existing wasm-loader interface using initSync.
// UK: Shim для сумісності з існуючим wasm-loader інтерфейсом використовуючи initSync.
// DE: Shim für Kompatibilität mit dem bestehenden wasm-loader-Interface unter Verwendung von initSync.
window.loadMyLispWasm = function() {
  if (window.wasm_bindgen) {
    return Promise.resolve(window.wasm_bindgen);
  }
  // Use initSync for embedded WASM bytes instead of fetch/Response hack.
  // initSync використовувати для вбудованих WASM байтів замість fetch/Response хаку.
  // initSync für eingebettete WASM-Bytes verwenden statt fetch/Response-Hack.
  try {
    const module = WebAssembly.instantiate(embeddedWasmBytes);
    if (window.wasm_bindgen && window.wasm_bindgen.initSync) {
      window.wasm_bindgen.initSync({ module: embeddedWasmBytes });
      return Promise.resolve(window.wasm_bindgen);
    }
  } catch (e) {
    console.warn('initSync failed, falling back to dynamic import:', e);
  }
  // Fallback to dynamic import if wasm-bindgen was not inlined.
  // Fallback zu dynamic import wenn wasm-bindgen nicht inline war.
  var dynamicImport = new Function('u', 'return import(u)');
  return dynamicImport('/wasm/my_lisp_wasm.js').then(function (mod) {
    return mod.default().then(function () {
      return mod;
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

