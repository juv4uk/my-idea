import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const workflowPath = new URL('../.github/workflows/publish-release.yml', import.meta.url);

test('Flatpak release stages the x86_64 Linux my-lisp sidecar before Tauri build', async () => {
  const workflow = await readFile(workflowPath, 'utf8');
  const flatpakJob = workflow.match(
    /  build-flatpak:[\s\S]*?(?=\n  build-web:)/,
  )?.[0] ?? '';

  assert.notEqual(flatpakJob, '', 'Flatpak release job must exist');
  assert.match(
    flatpakJob,
    /Build my-lisp CLI sidecar/,
    'Flatpak runs in a fresh job and must build its own desktop sidecar',
  );
  assert.match(
    flatpakJob,
    /src-tauri\/binaries\/my-lisp-x86_64-unknown-linux-gnu/,
    'Tauri requires the target-suffixed x86_64 Linux sidecar path',
  );

  const sidecarIndex = flatpakJob.indexOf('Build my-lisp CLI sidecar');
  const tauriIndex = flatpakJob.indexOf('bunx tauri build');
  assert.ok(sidecarIndex >= 0 && tauriIndex >= 0 && sidecarIndex < tauriIndex,
    'Flatpak sidecar staging must happen before bunx tauri build');
});
