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
html = await replaceAsync(html, /<script\s+src=["']([^"']+)["']\s*><\/script>/gi,
  async (_, reference) => `<script>${(await readLocal(reference)).replaceAll('</script>', '<\\/script>')}</script>`);

if (/<script[^>]+src=|<link[^>]+rel=["']stylesheet["']/i.test(html)) {
  throw new Error('Portable Web HTML still contains an external local asset reference');
}

await writeFile(outputPath, html, 'utf8');
console.log(`Standalone Web HTML created: ${outputPath} (${Buffer.byteLength(html)} bytes)`);

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

