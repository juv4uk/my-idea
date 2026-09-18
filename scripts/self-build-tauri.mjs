import { mkdir, writeFile } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');

function fail(message, status = 1) {
  console.error(message);
  process.exit(status);
}

function run(program, args, options = {}) {
  const result = spawnSync(program, args, {
    cwd: repoRoot,
    encoding: 'utf8',
    ...options,
  });
  if (result.error) fail(`${program} could not start: ${result.error.message}`);
  if (result.status !== 0) {
    if (result.stdout) process.stdout.write(result.stdout);
    if (result.stderr) process.stderr.write(result.stderr);
    fail(`${program} ${args.join(' ')} failed`, result.status ?? 1);
  }
  return result;
}

// Rust owns compiler-stage construction and verification. This wrapper only
// transports machine-readable compiler evidence and launches declared stages.
const stage = run(
  'cargo',
  ['run', '--quiet', '--manifest-path', 'src-tauri/Cargo.toml', '--bin', 'self-build-stage'],
);
let evidence;
try {
  evidence = JSON.parse(stage.stdout.trim());
} catch (error) {
  fail(`self-build-stage did not emit valid JSON evidence: ${error.message}`);
}

const gateEnv = {
  ...process.env,
  MY_IDEA_REQUIRE_COMPILER_STAGE: '1',
  MY_IDEA_SELF_BUILD_PLAN_DIGEST: evidence.planDigest,
  MY_IDEA_SELF_BUILD_SOURCE_REVISION: evidence.sourceRevision,
  MY_IDEA_SELF_BUILD_SOURCE_PATH: evidence.sourcePath,
  MY_IDEA_SELF_BUILD_SOURCE_SHA256: evidence.sourceSha256,
  MY_IDEA_SELF_BUILD_COMPILER: evidence.compiler,
  MY_IDEA_SELF_BUILD_COMPILER_REVISION: evidence.compilerRevision,
  MY_IDEA_SELF_BUILD_COMPILER_TARGET: evidence.compilerTarget,
  MY_IDEA_SELF_BUILD_ARTIFACT_PATH: evidence.artifactPath,
  MY_IDEA_SELF_BUILD_ARTIFACT_SHA256: evidence.artifactSha256,
};

// Tauri's existing beforeBuildCommand remains "bun run build", so the actual
// execution order is compiler stage -> frontend stage -> Cargo/Tauri stage.
run('bun', ['run', 'tauri', 'build'], { stdio: 'inherit', env: gateEnv });

const record = {
  schemaVersion: 'self-build-tauri-stage-v1',
  compilerStage: evidence,
  tauriStage: {
    command: ['bun', 'run', 'tauri', 'build'],
    status: 'success',
  },
};
await mkdir(resolve(repoRoot, 'target/self-build'), { recursive: true });
await writeFile(
  resolve(repoRoot, 'target/self-build/tauri-stage.json'),
  `${JSON.stringify(record, null, 2)}\n`,
);
console.log('self-build Tauri stage completed with verified compiler provenance');
