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

test('Windows ARM64 sidecar build pins Cargo to the verified MSVC linker', async () => {
  const workflow = await releaseWorkflow();
  const windowsArm64Job = workflow.match(
    /  build-windows-arm64:[\s\S]*?(?=\n  build-flatpak:)/,
  )?.[0] ?? '';
  const sidecarStep = windowsArm64Job.match(
    /      - name: Build my-lisp CLI sidecar[\s\S]*?(?=\n      - name: Build native Windows ARM64)/,
  )?.[0] ?? '';

  assert.notEqual(windowsArm64Job, '', 'Windows ARM64 release job must exist');
  assert.notEqual(sidecarStep, '', 'Windows ARM64 my-lisp sidecar step must exist');
  assert.match(
    sidecarStep,
    /shell: pwsh/,
    'Git Bash prepends its own /usr/bin/link.exe; the ARM64 sidecar build must stay in the MSVC PowerShell environment',
  );
  assert.match(
    sidecarStep,
    /CARGO_TARGET_AARCH64_PC_WINDOWS_MSVC_LINKER/,
    'Cargo must be pinned to the verified MSVC ARM64 linker instead of relying on PATH lookup',
  );
  assert.doesNotMatch(
    sidecarStep,
    /shell: bash/,
    'The Windows ARM64 sidecar build must not run under Git Bash because it shadows MSVC link.exe',
  );
});
