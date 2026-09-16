import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const workflowPath = new URL('../.github/workflows/publish-release.yml', import.meta.url);

async function releaseWorkflow() {
  return readFile(workflowPath, 'utf8');
}

test('macOS universal release stages sidecars for both architecture sub-builds and the final universal bundle', async () => {
  const workflow = await releaseWorkflow();

  assert.match(
    workflow,
    /src-tauri\/binaries\/my-lisp-x86_64-apple-darwin/,
    'Tauri universal build invokes an x86_64 sub-build and requires the x86_64 sidecar name',
  );
  assert.match(
    workflow,
    /src-tauri\/binaries\/my-lisp-aarch64-apple-darwin/,
    'Tauri universal build invokes an aarch64 sub-build and requires the aarch64 sidecar name',
  );
  assert.match(
    workflow,
    /src-tauri\/binaries\/my-lisp-universal-apple-darwin/,
    'The final universal bundle still requires the universal sidecar name',
  );
});
