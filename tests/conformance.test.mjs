import test from 'node:test';
import assert from 'node:assert';
import fs from 'node:fs';
import path from 'node:path';

// Load our Rust WASM Engine
import init, { evaluate } from '../public/wasm/my_lisp_wasm.js';

// Українська проєкція D1/D3: ратифіковані «так»/«ні», а не legacy 1/0.
// D24+ арифметика перевіряється через чинні українські голови й явне частка; без legacy-псевдонімів.
const fixturePath = path.resolve('tests/fixtures/conformance.json');
const fixture = JSON.parse(fs.readFileSync(fixturePath, 'utf8'));

test('Conformance alignment tests', async (t) => {
    // Wait for WASM engine to initialize. Read the WASM bytes manually since we're in Node.
    const wasmBytes = fs.readFileSync(path.resolve('public/wasm/my_lisp_wasm_bg.wasm'));
    await init({ module_or_path: wasmBytes });
    
    for (const { expr, expected, mode } of fixture) {
        await t.test(`Evaluating: ${expr}`, () => {
            const res = evaluate(expr, mode || "my-lisp");
            if (res.error) {
                assert.fail(`Evaluation failed with error: ${res.error}`);
            }
            assert.strictEqual(res.value, expected);
        });
    }

    await t.test('WASM adapter does not stack overflow on 100k list', () => {
        const res = evaluate(`
            (визначити побудувати
              (функція (кількість накопичення)
                (за-умовою
                  ((тотожне? кількість 0) накопичення)
                  ((тотожне? (як-є продовжити) (як-є продовжити))
                    (побудувати (відняти кількість 1)
                                (сполучити кількість накопичення))))))
            (побудувати 100000 (як-є ()))
        `, "укр");
        assert.ok(!res.error, "Evaluation should not error out");
        assert.ok(typeof res.value === 'string');
    });
});
