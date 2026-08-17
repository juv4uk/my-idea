import { spawnSync } from 'node:child_process';

const environment = { ...process.env, MY_LISP_BENCH_ITERATIONS: process.env.MY_LISP_BENCH_ITERATIONS ?? '1000' };

function run(command, args) {
  const result = spawnSync(command, args, { cwd: process.cwd(), env: environment, encoding: 'utf8', shell: false });
  if (result.stderr) process.stderr.write(result.stderr);
  if (result.error) throw result.error;
  if (result.status !== 0) {
    process.stdout.write(result.stdout ?? '');
    process.exit(result.status ?? 1);
  }
  return result.stdout;
}

const outputs = [
  // my-lisp now lives in the external/my-lisp git submodule (github.com/juv4uk/my-lisp).
  // my-lisp тепер живе в git submodule external/my-lisp (github.com/juv4uk/my-lisp).
  run('cargo', ['run', '--quiet', '--release', '--manifest-path', 'external/my-lisp/crates/my-lisp/Cargo.toml', '--example', 'benchmark']),
];
const rows = outputs.flatMap(output => output.split(/\r?\n/))
  .filter(line => line.startsWith('BENCH_RESULT\t'))
  .map(line => {
    const [, engine, name, nanoseconds] = line.split('\t');
    return { engine, name, nanoseconds: Number(nanoseconds) };
  });

console.log('my-lisp benchmark · benchmark my-lisp · my-lisp-Benchmark');
console.log(`iterations · ітерації · Iterationen: ${environment.MY_LISP_BENCH_ITERATIONS}`);
console.table(rows.map(row => ({ engine: row.engine, case: row.name, 'µs/op': (row.nanoseconds / 1000).toFixed(2) })));
