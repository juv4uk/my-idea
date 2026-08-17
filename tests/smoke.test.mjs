import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

test('Shadow CLJS entry point and trilingual interface exist', () => {
  const source = readFileSync('src-cljs/my_idea/core.cljs', 'utf8');
  assert.match(source, /Tauri \+ ClojureScript/);
  // Trilingual message tables live in i18n.cljs, not core.cljs.
  const i18n = readFileSync('src-cljs/my_idea/i18n.cljs', 'utf8');
  assert.match(i18n, /"en"/);
  assert.match(i18n, /"uk"/);
  assert.match(i18n, /"de"/);
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
  assert.match(worker, /Promise\.all\([\s\S]*cache\.add\(url\)/);
  assert.match(worker, /request\.mode === 'navigate'/);
  const readme = readFileSync('README.md', 'utf8');
  assert.match(readme, /releases\/latest\/download\/my-idea-web\.html/);
  assert.match(readme, /Без встановлення та облікового запису/);
  assert.match(readme, /Ohne Installation und Benutzerkonto/);
});

test('CodeMirror 6 is the primary reusable editor', () => {
  const editor = readFileSync('src-cljs/my_idea/editor.cljs', 'utf8');
  assert.match(editor, /@codemirror\/view/);
  assert.match(editor, /lineNumbers/);
  assert.match(editor, /autocompletion/);
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

test('frontend wiring exposes the independent Rust my-lisp command', () => {
  const cargo = readFileSync('src-tauri/Cargo.toml', 'utf8');
  const rust = readFileSync('src-tauri/src/lib.rs', 'utf8');
  const commands = readFileSync('src-cljs/my_idea/commands.cljs', 'utf8');
  assert.match(cargo, /my-lisp\s*=\s*\{\s*git\s*=\s*"https:\/\/github\.com\/juv4uk\/my-lisp\.git"/);
  assert.match(rust, /fn evaluate_my_lisp/);
  assert.match(commands, /invoke! "evaluate_my_lisp"/);
  // ClojureScript prototype is removed — WASM is the only web engine
  // ClojureScript-прототип видалено — WASM є єдиним веб-рушієм
  assert.match(commands, /WASM engine is loading/);
  assert.match(commands, /wasm\/ready\?/);
});

test('WASM crate and ClojureScript bindings are present and correctly wired', () => {
  // my-lisp-wasm lives in the external/my-lisp git submodule (github.com/juv4uk/my-lisp),
  // not a local crates/ directory — wasm-pack needs a real checkout, which a Cargo git
  // dependency alone can't provide (see scripts/build.mjs).
  const wasmCargo = readFileSync('external/my-lisp/crates/my-lisp-wasm/Cargo.toml', 'utf8');
  const wasmLib = readFileSync('external/my-lisp/crates/my-lisp-wasm/src/lib.rs', 'utf8');
  const wasmCljs = readFileSync('src-cljs/my_idea/wasm.cljs', 'utf8');
  const core = readFileSync('src-cljs/my_idea/core.cljs', 'utf8');
  // Crate is a cdylib that depends on my-lisp and wasm-bindgen
  assert.match(wasmCargo, /cdylib/);
  assert.match(wasmCargo, /wasm-bindgen/);
  assert.match(wasmCargo, /my-lisp\s*=/);
  // The evaluate function mirrors Tauri contract
  assert.match(wasmLib, /#\[wasm_bindgen\]/);
  assert.match(wasmLib, /pub fn evaluate/);
  assert.match(wasmLib, /my-lisp · WASM/);
  // CLJS bindings load the module and expose ready? / evaluate
  assert.match(wasmCljs, /ready\?/);
  assert.match(wasmCljs, /load!/);
  // The loader uses a plain-JS shim (wasm-loader.js) to bypass Closure Compiler;
  // js/import cannot be used directly in release builds.
  // Завантажувач використовує plain-JS шим (wasm-loader.js) для обходу Closure Compiler;
  // js/import не можна використовувати напряму у release-збірках.
  assert.match(wasmCljs, /loadMyLispWasm/);
  const wasmLoader = readFileSync('public/wasm-loader.js', 'utf8');
  assert.match(wasmLoader, /\/wasm\/my_lisp_wasm\.js/);
  assert.match(wasmLoader, /loadMyLispWasm/);
  // commands.cljs uses the WASM module in the web branch (extracted from core.cljs)
  const commands = readFileSync('src-cljs/my_idea/commands.cljs', 'utf8');
  assert.match(commands, /my-idea.wasm/);
  assert.match(commands, /wasm\/evaluate/);
  // wasm/load! is called only in the web build (when-not native?)
  assert.match(core, /wasm\/load!/);
  assert.match(core, /when-not.*workspace\/native\?/s);

});

test('open and save work in both browser and Tauri modes', () => {
  const core = readFileSync('src-cljs/my_idea/core.cljs', 'utf8');
  const commands = readFileSync('src-cljs/my_idea/commands.cljs', 'utf8');
  const workspace = readFileSync('src-cljs/my_idea/workspace.cljs', 'utf8');
  const config = JSON.parse(readFileSync('src-tauri/tauri.conf.json', 'utf8'));
  assert.equal(config.app.withGlobalTauri, true);
  assert.match(commands, /choose-browser-workspace!/);
  assert.match(commands, /webkitdirectory/);
  assert.match(workspace, /open-browser-workspace/);
  assert.match(workspace, /download!/);
});

test('language and eye-comfort themes use simple cycling buttons', () => {
  const core = readFileSync('src-cljs/my_idea/core.cljs', 'utf8');
  const i18n = readFileSync('src-cljs/my_idea/i18n.cljs', 'utf8');
  const util = readFileSync('src-cljs/my_idea/util.cljs', 'utf8');
  const styles = readFileSync('public/styles.css', 'utf8');
  assert.match(i18n, /def languages \["uk" "de" "en"\]/);
  assert.match(i18n, /def themes \["auto" "light" "dark" "sepia" "signal" "amber" "forest"\]/);
  assert.match(core, /id='theme'/);
  assert.match(util, /keep-indexed/);
  assert.match(core, /Hello · Привіт · Hallo/);
  assert.match(styles, /data-theme=sepia/);
  assert.match(styles, /data-theme=signal/);
});

test('active document programming language switches from the bottom status bar', () => {
  const core = readFileSync('src-cljs/my_idea/core.cljs', 'utf8');
  const i18n = readFileSync('src-cljs/my_idea/i18n.cljs', 'utf8');
  const editor = readFileSync('src-cljs/my_idea/editor.cljs', 'utf8');
  assert.match(i18n, /def programming-languages \["my-lisp" "clojurescript" "rust" "markdown" "mermaid" "text"\]/);
  assert.match(core, /id='programming-language'/);
  assert.match(core, /cycle-programming-language!/);
  assert.match(core, /language-mode/);
  assert.match(editor, /@codemirror\/lang-rust/);
  assert.match(editor, /language-extensions/);
  const workspace = readFileSync('src-cljs/my_idea/workspace.cljs', 'utf8');
  assert.match(workspace, /ends-with\? lower "\.my"/);
  const sourceFiles = readFileSync('docs/source-files.md', 'utf8');
  assert.match(sourceFiles, /canonical file extension[\s\S]*`\.my`/);
  assert.match(sourceFiles, /Канонічне розширення[\s\S]*`\.my`/);
  assert.match(sourceFiles, /kanonische Dateiendung[\s\S]*`\.my`/);
});

test('Rust benchmarks the my-lisp programs', () => {
  const runner = readFileSync('scripts/benchmark.mjs', 'utf8');
  const rust = readFileSync('external/my-lisp/crates/my-lisp/examples/benchmark.rs', 'utf8');
  for (const name of ['arithmetic', 'lists', 'recursion', 'closures', 'parser']) {
    assert.match(readFileSync(`benchmarks/${name}.my`, 'utf8'), /·/);
  }
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
  assert.match(workflow, /build-windows-arm64:/);
  assert.match(workflow, /aarch64-pc-windows-msvc/);
  assert.match(workflow, /verify-windows-architecture\.ps1/);
  assert.match(workflow, /my-idea_\$\(\$env:RELEASE_TAG\.TrimStart\('v'\)\)_arm64-setup\.exe/);
  assert.match(workflow, /my-idea_\$\{VERSION\}_android\.apk/);
  assert.match(workflow, /my-idea_\$\{RELEASE_TAG#v\}_x86_64\.flatpak/);
  assert.match(workflow, /gh release upload "\$RELEASE_TAG" my-idea-web\.html/);
  assert.match(workflow, /ANDROID_KEYSTORE_BASE64/);
  assert.match(workflow, /android build -- --apk --aab --ci/);
  assert.match(workflow, /rm -rf -- "\$GITHUB_WORKSPACE\/src-tauri\/gen\/android"/);
  assert.match(workflow, /tauri-apps\/tauri-action@v0/);
  assert.match(workflow, /assetNamePattern:\s*'\[name\]_\[version\]_\[arch\]\[setup\]\[ext\]'/);
  assert.doesNotMatch(workflow, /releaseAssetNamePattern/);
  assert.equal((workflow.match(/Swatinem\/rust-cache@v2/g) ?? []).length, 5);
  assert.match(workflow, /shared-key: desktop-Linux/);
  assert.equal((workflow.match(/java-version: 21/g) ?? []).length, 6);
  assert.match(portable, /Standalone Web HTML created/);
  assert.match(androidSigning, /ANDROID_KEYSTORE_BASE64/);
  assert.match(androidSigning, /Refusing to overwrite/);
  assert.match(androidGradle, /signingConfigs/);
  assert.match(androidGradle, /rootProject\.file\(\"keystore\.properties\"\)/);
});

import { chromium } from '@playwright/test';
import http from 'node:http';
import fs from 'node:fs';

test('Standalone web artifact does not stack overflow on 100k list', async () => {
  const html = fs.readFileSync('my-idea-web.html');
  const server = http.createServer((req, res) => {
    res.writeHead(200, { 'Content-Type': 'text/html' });
    res.end(html);
  });
  
  await new Promise(resolve => server.listen(0, resolve));
  const port = server.address().port;
  
  let browser;
  try {
    browser = await chromium.launch();
    const context = await browser.newContext();
    const page = await context.newPage();
    
    await page.goto(`http://localhost:${port}/`);
    
    const res = await page.evaluate(async () => {
      const wasm = await window.loadMyLispWasm();
      return wasm.evaluate(`
        (def build 
          (lambda (n acc)
            (cond ((eq n 0) acc)
                  ('t (build (- n 1) (cons n acc))))))
        (build 100000 '())
      `);
    });
    
    assert.ok(res !== undefined, "Result should not be undefined");
    assert.ok(res.error === undefined, "Should not return an error: " + res.error);
  } finally {
    if (browser) await browser.close();
    server.close();
  }
});
