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

test('workspace model tracks files, tabs and dirty documents', () => {
  const workspace = readFileSync('src-cljs/my_idea/workspace.cljs', 'utf8');
  assert.match(workspace, /open-document/);
  assert.match(workspace, /close-document/);
  assert.match(workspace, /dirty\?/);
  assert.match(workspace, /my-idea:workspace/);
});

test('native file commands are constrained to the selected workspace', () => {
  const rust = readFileSync('src-tauri/src/lib.rs', 'utf8');
  assert.match(rust, /choose_workspace/);
  assert.match(rust, /read_workspace_file/);
  assert.match(rust, /save_workspace_file/);
  assert.match(rust, /starts_with\(root\)/);
  assert.match(rust, /only existing workspace files can be saved/);
});

test('language and eye-comfort themes use simple cycling buttons', () => {
  const core = readFileSync('src-cljs/my_idea/core.cljs', 'utf8');
  const styles = readFileSync('public/styles.css', 'utf8');
  assert.match(core, /def languages \["uk" "de" "en"\]/);
  assert.match(core, /def themes \["auto" "light" "dark" "sepia"\]/);
  assert.match(core, /id='theme'/);
  assert.match(styles, /data-theme=sepia/);
});
