import test from 'node:test';
import assert from 'node:assert';
import fs from 'node:fs';
import path from 'node:path';

// Load our Rust WASM Engine
import init, { evaluate } from '../public/wasm/my_lisp_wasm.js';

const fixturePath = path.resolve('tests/fixtures/conformance.json');
const fixture = JSON.parse(fs.readFileSync(fixturePath, 'utf8'));

test('Conformance alignment tests', async (t) => {
    // Wait for WASM engine to initialize. Read the WASM bytes manually since we're in Node.
    const wasmBytes = fs.readFileSync(path.resolve('public/wasm/my_lisp_wasm_bg.wasm'));
    await init(wasmBytes);
    
    for (const { expr, expected } of fixture) {
        await t.test(`Evaluating: ${expr}`, () => {
            const res = evaluate(expr);
            if (res.error) {
                assert.fail(`Evaluation failed with error: ${res.error}`);
            }
            assert.strictEqual(res.value, expected);
        });
    }

    await t.test('WASM adapter does not stack overflow on 100k list', () => {
        const res = evaluate("(def build (lambda (n acc) (cond ((eq n 0) acc) (t (build (- n 1) (cons n acc)))))) (build 100000 '())");
        assert.ok(!res.error, "Evaluation should not error out");
        assert.ok(typeof res.value === 'string');
    });
});
