import { cp, mkdir, rm } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';

// Direct JS entry point: no platform shell, quoting differences, or injection surface.
// Прямий JS-вхід: без платформної оболонки та різниці екранування.
// Direkter JS-Einstieg: ohne Plattform-Shell und Quoting-Unterschiede.
const result = spawnSync(process.execPath, ['node_modules/shadow-cljs/cli/runner.js', 'release', 'app'], {
  stdio: 'inherit'
});
if (result.status !== 0) process.exit(result.status ?? 1);

// EN: Tauri receives a clean static directory instead of Shadow CLJS caches.
// UK: Tauri отримує чисту статичну папку без кешів Shadow CLJS.
// DE: Tauri erhält ein sauberes statisches Verzeichnis ohne Shadow-CLJS-Caches.
await rm('dist', { recursive: true, force: true });
await mkdir('dist', { recursive: true });
await cp('public', 'dist', { recursive: true });
console.log('ClojureScript IDE written to dist/');
