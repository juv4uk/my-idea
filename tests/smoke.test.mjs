import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

test('Shadow CLJS entry point and trilingual interface exist', () => {
  const source = readFileSync('src-cljs/my_idea/core.cljs', 'utf8');
  assert.match(source, /Tauri \+ ClojureScript/);
  assert.match(source, /"en"/);
  assert.match(source, /"uk"/);
  assert.match(source, /"de"/);
});

test('Tauri serves the compiled dist directory', () => {
  const config = JSON.parse(readFileSync('src-tauri/tauri.conf.json', 'utf8'));
  assert.equal(config.build.frontendDist, '../dist');
  assert.equal(config.productName, 'my-idea');
});

test('CodeMirror 6 is the primary reusable editor', () => {
  const editor = readFileSync('src-cljs/my_idea/editor.cljs', 'utf8');
  assert.match(editor, /@codemirror\/view/);
  assert.match(editor, /lineNumbers/);
  assert.match(editor, /autocompletion/);
  assert.match(editor, /linter/);
});
