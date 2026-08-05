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

test('web build is an installable offline PWA without affecting Tauri protocols', () => {
  const html = readFileSync('public/index.html', 'utf8');
  const manifest = JSON.parse(readFileSync('public/manifest.webmanifest', 'utf8'));
  const worker = readFileSync('public/sw.js', 'utf8');
  assert.match(html, /rel="manifest"/);
  assert.match(html, /\^\(https\?:\)\$/);
  assert.equal(manifest.display, 'standalone');
  assert.deepEqual(manifest.icons.map(({ sizes }) => sizes), ['192x192', '512x512']);
  assert.match(worker, /cache\.addAll\(APP_SHELL\)/);
  assert.match(worker, /request\.mode === 'navigate'/);
});

test('CodeMirror 6 is the primary reusable editor', () => {
  const editor = readFileSync('src-cljs/my_idea/editor.cljs', 'utf8');
  assert.match(editor, /@codemirror\/view/);
  assert.match(editor, /lineNumbers/);
  assert.match(editor, /autocompletion/);
  assert.match(editor, /linter/);
  assert.match(editor, /\.\-matches/);
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
  assert.match(rust, /#\[cfg\(desktop\)\][\s\S]*blocking_pick_folder/);
  assert.match(rust, /#\[cfg\(mobile\)\][\s\S]*Storage Access Framework/);
  assert.match(rust, /read_workspace_file/);
  assert.match(rust, /save_workspace_file/);
  assert.match(rust, /starts_with\(root\)/);
  assert.match(rust, /only existing workspace files can be saved/);
});

test('open and save work in both browser and Tauri modes', () => {
  const core = readFileSync('src-cljs/my_idea/core.cljs', 'utf8');
  const workspace = readFileSync('src-cljs/my_idea/workspace.cljs', 'utf8');
  const config = JSON.parse(readFileSync('src-tauri/tauri.conf.json', 'utf8'));
  assert.equal(config.app.withGlobalTauri, true);
  assert.match(core, /choose-browser-workspace!/);
  assert.match(core, /webkitdirectory/);
  assert.match(workspace, /open-browser-workspace/);
  assert.match(workspace, /download!/);
});

test('language and eye-comfort themes use simple cycling buttons', () => {
  const core = readFileSync('src-cljs/my_idea/core.cljs', 'utf8');
  const styles = readFileSync('public/styles.css', 'utf8');
  assert.match(core, /def languages \["uk" "de" "en"\]/);
  assert.match(core, /def themes \["auto" "light" "dark" "sepia" "signal" "amber" "forest"\]/);
  assert.match(core, /id='theme'/);
  assert.match(core, /keep-indexed/);
  assert.match(core, /Hello · Привіт · Hallo/);
  assert.match(styles, /data-theme=sepia/);
  assert.match(styles, /data-theme=signal/);
});

test('active document programming language switches from the bottom status bar', () => {
  const core = readFileSync('src-cljs/my_idea/core.cljs', 'utf8');
  const editor = readFileSync('src-cljs/my_idea/editor.cljs', 'utf8');
  assert.match(core, /def programming-languages \["my-lisp" "clojurescript" "rust" "text"\]/);
  assert.match(core, /id='programming-language'/);
  assert.match(core, /cycle-programming-language!/);
  assert.match(core, /welcome\.my/);
  assert.match(editor, /@codemirror\/lang-rust/);
  assert.match(editor, /language-extensions/);
  const workspace = readFileSync('src-cljs/my_idea/workspace.cljs', 'utf8');
  assert.match(workspace, /ends-with\? lower "\.my"/);
  const sourceFiles = readFileSync('docs/source-files.md', 'utf8');
  assert.match(sourceFiles, /canonical file extension[\s\S]*`\.my`/);
  assert.match(sourceFiles, /Канонічне розширення[\s\S]*`\.my`/);
  assert.match(sourceFiles, /kanonische Dateiendung[\s\S]*`\.my`/);
});

test('CLJS and Rust benchmark the same my-lisp programs', () => {
  const config = readFileSync('shadow-cljs.edn', 'utf8');
  const runner = readFileSync('scripts/benchmark.mjs', 'utf8');
  const rust = readFileSync('crates/my-lisp/examples/benchmark.rs', 'utf8');
  for (const name of ['arithmetic', 'lists', 'recursion', 'closures', 'parser']) {
    assert.match(readFileSync(`benchmarks/${name}.my`, 'utf8'), /·/);
  }
  assert.match(config, /:benchmark/);
  assert.match(runner, /MY_LISP_BENCH_ITERATIONS/);
  assert.match(rust, /BENCH_RESULT/);
});

test('tag releases publish desktop, ARM, Flatpak, Web and signed Android builds', () => {
  const workflow = readFileSync('.github/workflows/publish-release.yml', 'utf8');
  const portable = readFileSync('scripts/make-portable-web.mjs', 'utf8');
  const androidSigning = readFileSync('scripts/setup-android-signing.ps1', 'utf8');
  const androidGradle = readFileSync('scripts/configure-android-signing.mjs', 'utf8');
  assert.match(workflow, /build-desktop:/);
  assert.match(workflow, /build-arm-linux:/);
  assert.match(workflow, /build-flatpak:/);
  assert.match(workflow, /build-web:/);
  assert.match(workflow, /build-android:/);
  assert.match(workflow, /ANDROID_KEYSTORE_BASE64/);
  assert.match(workflow, /android build -- --apk --aab --ci/);
  assert.match(workflow, /rm -rf -- "\$GITHUB_WORKSPACE\/src-tauri\/gen\/android"/);
  assert.match(workflow, /tauri-apps\/tauri-action@v0/);
  assert.equal((workflow.match(/java-version: 21/g) ?? []).length, 5);
  assert.match(portable, /Standalone Web HTML created/);
  assert.match(androidSigning, /ANDROID_KEYSTORE_BASE64/);
  assert.match(androidSigning, /Refusing to overwrite/);
  assert.match(androidGradle, /signingConfigs/);
  assert.match(androidGradle, /rootProject\.file\(\"keystore\.properties\"\)/);
});
