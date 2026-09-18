import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

test('package exposes the bounded self-build Tauri wrapper', async () => {
  const pkg = JSON.parse(await readFile(new URL('../package.json', import.meta.url), 'utf8'));
  assert.equal(pkg.scripts['self-build:tauri'], 'node scripts/self-build-tauri.mjs');
});

test('self-build wrapper consumes Rust evidence and invokes ordinary Tauri build', async () => {
  const script = await readFile(new URL('../scripts/self-build-tauri.mjs', import.meta.url), 'utf8');

  assert.match(script, /self-build-stage/);
  assert.match(script, /MY_IDEA_REQUIRE_COMPILER_STAGE/);
  assert.match(script, /bun['"],\s*\['run', 'tauri', 'build'\]/);
  assert.doesNotMatch(script, /parse.*lisp|eval.*lisp|interpret.*lisp/i);
});
