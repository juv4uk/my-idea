// #50 (IDE-LIVE-EDITOR-WITNESS-1): an honest end-to-end witness that a
// canonical .lisp plugin, evaluated by the real production
// ManagedReplSession/EditorCommandRegistry, changes a real CodeMirror
// buffer in a real browser.
//
// What is real: the browser (headless chromium via Playwright), the
// compiled my-idea-web.html app bundle, the real CodeMirror instance
// (my-idea.editor/mount!), the real commands.cljs dispatch path
// (invoke-editor-command!/dispatch-editor-key!), and the real Rust
// ManagedReplSession/EditorCommandRegistry evaluating the real plugin
// source (editor_bridge_harness.rs links the same my_idea_lib the shipped
// app does).
//
// What is a stand-in: the transport. A packaged my-idea talks to that Rust
// logic over Tauri's own WebView IPC. Automating that would need
// tauri-driver + a WebDriver-capable WebView (webkit2gtk-driver on Linux),
// which isn't installable in this environment (no root, and it isn't
// packaged in the project's Guix channel either) — a documented limitation,
// not a silently-skipped one. This test swaps only that one hop for
// page.exposeFunction bridging to the harness's stdio, using the exact
// same command names/argument shapes the real Tauri commands in lib.rs
// use.
//
// Prerequisite (not run by `bun run test` — cross-language, needs a Rust
// build): `cargo build --bin editor_bridge_harness` inside src-tauri (or
// set MY_IDEA_EDITOR_HARNESS_BIN to a prebuilt one), then
// `bun run witness:editor`.

import assert from 'node:assert/strict';
import test from 'node:test';
import { chromium } from '@playwright/test';
import { spawn } from 'node:child_process';
import http from 'node:http';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const HARNESS_BIN =
  process.env.MY_IDEA_EDITOR_HARNESS_BIN ??
  path.join('src-tauri', 'target', 'debug', 'editor_bridge_harness');

const BRACKET_PLUGIN = `
(editor/register-command "bracket-selection"
  (lambda ()
    (editor/replace-selection
      (string-append "["
        (string-append (editor/selection) "]"))))
  "Wrap selection in brackets")

(editor/keymap "Ctrl-[" "bracket-selection")
`;

function startHarness(pluginsDir) {
  const child = spawn(HARNESS_BIN, [pluginsDir], { stdio: ['pipe', 'pipe', 'pipe'] });
  const pending = [];
  let buffer = '';
  child.stdout.on('data', (chunk) => {
    buffer += chunk.toString('utf8');
    let index;
    while ((index = buffer.indexOf('\n')) !== -1) {
      const line = buffer.slice(0, index);
      buffer = buffer.slice(index + 1);
      if (line.trim().length === 0) continue;
      const resolve = pending.shift();
      if (resolve) resolve(JSON.parse(line));
    }
  });
  return {
    child,
    invoke(cmd, args) {
      return new Promise((resolve) => {
        pending.push(resolve);
        child.stdin.write(`${JSON.stringify({ cmd, args })}\n`);
      });
    },
    stop() {
      child.kill();
    },
  };
}

test('#50: a canonical .lisp plugin changes a real live CodeMirror buffer end-to-end', async () => {
  assert.ok(
    fs.existsSync(HARNESS_BIN),
    `editor_bridge_harness binary not found at ${HARNESS_BIN} — build it first: ` +
      `cd src-tauri && cargo build --bin editor_bridge_harness`,
  );

  const pluginsRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'my-idea-live-witness-'));
  fs.mkdirSync(path.join(pluginsRoot, 'plugins'));
  fs.writeFileSync(path.join(pluginsRoot, 'plugins', 'bracket.lisp'), BRACKET_PLUGIN);

  const harness = startHarness(pluginsRoot);
  const html = fs.readFileSync('my-idea-web.html');
  const server = http.createServer((req, res) => {
    res.writeHead(200, { 'Content-Type': 'text/html' });
    res.end(html);
  });
  await new Promise((resolve) => server.listen(0, resolve));
  const port = server.address().port;

  let browser;
  try {
    browser = await chromium.launch();
    const context = await browser.newContext();
    const page = await context.newPage();

    // Installed before the bundled app's own init-fn auto-runs (shadow-cljs
    // wires my-idea.core/init to fire on load): this is the real
    // workspace.native? boundary the shipped app checks everywhere.
    await page.exposeFunction('__editorHarnessInvoke', (cmd, args) => harness.invoke(cmd, args));
    await page.addInitScript(() => {
      window.__TAURI__ = {
        core: {
          invoke: (cmd, args) =>
            window.__editorHarnessInvoke(cmd, args).then((response) => {
              if (!response.ok) return Promise.reject(response.error);
              return response.result;
            }),
        },
      };
    });

    await page.goto(`http://localhost:${port}/`);
    // The #editor container is part of the app chrome unconditionally
    // (rendered regardless of whether a document/workspace is open); wait
    // for the normal boot's render! to have produced it.
    await page.waitForSelector('#editor');

    await page.evaluate(
      ([containerId, text, key]) => window.my_idea.core.mount_editor_witness_BANG_(containerId, text, key),
      ['editor', 'hello world', 'Ctrl-['],
    );
    await page.waitForSelector('#editor .cm-content');

    const initial = await page.textContent('#editor .cm-content');
    assert.equal(initial, 'hello world');

    // select "world" (offsets 6..11) via a real CodeMirror selection
    // transaction, then invoke the plugin command via the command route.
    await page.evaluate(([from, to]) => window.my_idea.core.witness_select_range_BANG_(from, to), [6, 11]);
    await page.evaluate((name) => window.my_idea.core.witness_invoke_command_BANG_(name), 'bracket-selection');

    await page.waitForFunction(() => document.querySelector('#editor .cm-content')?.textContent === 'hello [world]');
    assert.equal(await page.textContent('#editor .cm-content'), 'hello [world]');

    // reload: same plugin loaded again must not duplicate the command.
    const reloadReport = await page.evaluate(() => window.__editorHarnessInvoke('reload_plugins', {}));
    assert.equal(reloadReport.ok, true);
    const commandsAfterReload = await page.evaluate(() => window.__editorHarnessInvoke('editor_list_commands', {}));
    assert.deepEqual(commandsAfterReload.result, ['bracket-selection']);

    // Negative witnesses, checked at the same dispatch layer the CLJS
    // bridge calls (session.invoke_command/dispatch_key): fail-closed
    // means no EditorEffect is ever produced, so no replace-selection can
    // ever have run -- an unknown command or unbound key cannot mutate the
    // editor because the effect that would mutate it was never created.
    const unknownCommand = await page.evaluate(() =>
      window.__editorHarnessInvoke('editor_invoke_command', {
        name: 'does-not-exist',
        state: { buffer: 'hello [world]', selection: '' },
      }),
    );
    assert.equal(unknownCommand.ok, false, 'an unknown command must fail closed, not silently no-op');

    const unboundKey = await page.evaluate(() =>
      window.__editorHarnessInvoke('editor_dispatch_key', {
        key: 'Ctrl-Never-Bound',
        state: { buffer: 'hello [world]', selection: '' },
      }),
    );
    assert.equal(unboundKey.ok, false, 'an unbound key must fail closed, not silently no-op');

    // Separately: driving the *real* end-to-end CLJS path with the same
    // unknown command must not crash or lock up the editor -- it logs to
    // the REPL console (a real render!, which is why this isn't asserted
    // via a DOM-diff here: the console panel is not this witness's mount
    // point). The page staying responsive afterward is the actual claim.
    await page.evaluate((name) => window.my_idea.core.witness_invoke_command_BANG_(name), 'does-not-exist');
    const stillResponsive = await page.evaluate(() => document.readyState);
    assert.equal(stillResponsive, 'complete', 'the page must stay responsive after a plugin/dispatch error');
  } finally {
    if (browser) await browser.close();
    server.close();
    harness.stop();
    fs.rmSync(pluginsRoot, { recursive: true, force: true });
  }
});
