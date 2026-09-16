import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

const androidConfigPath = new URL('../src-tauri/tauri.android.conf.json', import.meta.url);
const workflowPath = new URL('../.github/workflows/publish-release.yml', import.meta.url);

test('Android config disables the desktop-only external my-lisp sidecar', async () => {
  const config = JSON.parse(await readFile(androidConfigPath, 'utf8'));
  assert.deepEqual(
    config.bundle?.externalBin,
    [],
    'Android must not inherit the desktop CLI sidecar requirement',
  );
});

test('release repair recreates the Android sidecar override after checking out an existing tag', async () => {
  const workflow = await readFile(workflowPath, 'utf8');
  const androidJob = workflow.match(
    /  build-android:[\s\S]*$/,
  )?.[0] ?? '';

  assert.notEqual(androidJob, '', 'Android release job must exist');
  assert.match(
    androidJob,
    /tauri\.android\.conf\.json/,
    'workflow_dispatch rebuilds an old tag, so the release job must recreate the Android override after checkout',
  );
  assert.match(
    androidJob,
    /externalBin["']?\s*:\s*\[\]/,
    'the recreated Android config must clear bundle.externalBin',
  );
});
