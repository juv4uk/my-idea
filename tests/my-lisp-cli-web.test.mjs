import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import http from 'node:http';
import { chromium } from '@playwright/test';

// public/my-lisp-cli-web.html has real client-side logic (session accumulation,
// error rollback, history navigation) that is not covered by any other test —
// unlike the conformance/smoke tests, which only exercise the WASM engine or do
// static source assertions. This drives the actual REPL page in a real browser.
//
// public/my-lisp-cli-web.html має реальну клієнтську логіку (накопичення сесії,
// відкат при помилці, навігація історії), яку не покриває жоден інший тест —
// на відміну від conformance/smoke тестів, що перевіряють лише WASM-рушій або
// статичні твердження про джерело. Цей тест керує справжньою REPL-сторінкою в
// реальному браузері.
//
// public/my-lisp-cli-web.html besitzt echte clientseitige Logik (Sitzungsakkumulation,
// Fehler-Rollback, Verlaufsnavigation), die von keinem anderen Test abgedeckt wird —
// im Gegensatz zu den Conformance-/Smoke-Tests, die nur die WASM-Engine oder statische
// Quellbehauptungen prüfen. Dieser Test steuert die echte REPL-Seite in einem echten Browser.

async function withPage(run) {
  const html = readFileSync('public/my-lisp-cli-web.html');
  const server = http.createServer((req, res) => {
    const path = req.url === '/' ? '/my-lisp-cli-web.html' : req.url;
    if (path === '/my-lisp-cli-web.html') {
      res.writeHead(200, { 'Content-Type': 'text/html' });
      res.end(html);
      return;
    }
    // Serve wasm-loader.js and the wasm/ directory as static files, same layout as public/.
    // Обслуговує wasm-loader.js і папку wasm/ як статичні файли, той самий шар, що й public/.
    // Bedient wasm-loader.js und das wasm/-Verzeichnis als statische Dateien, gleiches Layout wie public/.
    try {
      const filePath = `public${path}`;
      const data = readFileSync(filePath);
      const contentType = filePath.endsWith('.wasm')
        ? 'application/wasm'
        : filePath.endsWith('.js')
          ? 'text/javascript'
          : 'application/octet-stream';
      res.writeHead(200, { 'Content-Type': contentType });
      res.end(data);
    } catch {
      res.writeHead(404);
      res.end('not found');
    }
  });

  await new Promise((resolve) => server.listen(0, resolve));
  const port = server.address().port;

  let browser;
  try {
    browser = await chromium.launch();
    const page = await browser.newPage();
    await page.goto(`http://localhost:${port}/`);
    await page.waitForSelector('#input:not([disabled])', { timeout: 10_000 });
    return await run(page);
  } finally {
    if (browser) await browser.close();
    server.close();
  }
}

// Types a line into the REPL input and submits it via a synthetic Enter keydown,
// matching how the page's own listener is wired (see public/my-lisp-cli-web.html).
// Вводить рядок у REPL і надсилає його синтетичним Enter keydown, так само, як
// зареєстрований власний listener сторінки (див. public/my-lisp-cli-web.html).
// Tippt eine Zeile in den REPL und sendet sie per synthetischem Enter-Keydown,
// so wie der eigene Listener der Seite verdrahtet ist (siehe public/my-lisp-cli-web.html).
async function submitLine(page, line) {
  await page.evaluate((text) => {
    const el = document.getElementById('input');
    el.value = text;
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }));
  }, line);
}

async function logText(page) {
  return page.locator('#log').innerText();
}

test('my-lisp-cli-web.html evaluates a plain expression', async () => {
  await withPage(async (page) => {
    await submitLine(page, '(+ 1 2 3)');
    await page.waitForFunction(() => document.getElementById('log').innerText.includes('6'));
    assert.match(await logText(page), /my-lisp> \(\+ 1 2 3\)\s*\n6/);
  });
});

test('my-lisp-cli-web.html persists definitions across REPL lines', async () => {
  await withPage(async (page) => {
    await submitLine(page, '(def square (lambda (x) (* x x)))');
    await submitLine(page, '(square 7)');
    await page.waitForFunction(() => document.getElementById('log').innerText.includes('49'));
    assert.match(await logText(page), /square 7\)\s*\n49/);
  });
});

test('my-lisp-cli-web.html preloads lib/core.my', async () => {
  await withPage(async (page) => {
    // identity is defined in lib/core.my, not the Rust core — if the page stopped
    // inlining it, this would fail with an "unknown symbol" error instead of 42.
    // identity визначено в lib/core.my, а не в Rust-ядрі — якби сторінка перестала
    // його вбудовувати, це провалилось би "unknown symbol" замість 42.
    // identity ist in lib/core.my definiert, nicht im Rust-Kern — würde die Seite es
    // nicht mehr einbetten, schlüge dies mit "unknown symbol" statt 42 fehl.
    await submitLine(page, '(identity 42)');
    await page.waitForFunction(() => document.getElementById('log').innerText.includes('42'));
    assert.match(await logText(page), /identity 42\)\s*\n42/);
  });
});

test('my-lisp-cli-web.html keeps exact rational arithmetic exact', async () => {
  await withPage(async (page) => {
    await submitLine(page, '(+ (/ 1 3) (/ 1 6))');
    await page.waitForFunction(() => document.getElementById('log').innerText.includes('1/2'));
    assert.match(await logText(page), /1\/2/);
  });
});

test('my-lisp-cli-web.html reports an error without corrupting the session', async () => {
  await withPage(async (page) => {
    await submitLine(page, '(def square (lambda (x) (* x x)))');
    await submitLine(page, "(car '())");
    await page.waitForFunction(() => document.getElementById('log').innerText.includes('Error:'));
    await submitLine(page, '(square 3)');
    await page.waitForFunction(() => document.getElementById('log').innerText.includes('9'));
    const text = await logText(page);
    assert.match(text, /Error: car expects a non-empty list/);
    assert.match(text, /square 3\)\s*\n9/);
  });
});
