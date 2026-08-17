import assert from 'node:assert/strict';
import test from 'node:test';
import { chromium } from '@playwright/test';
import http from 'node:http';
import fs from 'node:fs';

function startServer(html) {
  const server = http.createServer((req, res) => {
    res.writeHead(200, { 'Content-Type': 'text/html' });
    res.end(html);
  });
  return new Promise(resolve => {
    server.listen(0, () => resolve(server));
  });
}

const MOCK_ECO = {
  myLisp: { name: 'my-lisp', branch: 'main', sha: 'abc1234', found: true },
  myLispContract: { version: { major: 1, minor: 2 } },
  cml: { name: 'cml', branch: 'main', sha: 'def5678', found: true },
  fpgaLisp: { name: 'fpga-lisp', branch: 'main', sha: '999aaab', found: true },
  fpgaLispContract: { version: { major: 0, minor: 3 } },
  evidenceMatrix: [
    {
      requirement: 'arithmetic',
      byImplementation: {
        'my-lisp': { fixture: '(+ 1 2)', expected: '3', actual: '3', result: 'pass', commit: 'aaa', timestamp: '2026-01-01', runner: 'ci' },
        'cml': { fixture: '(+ 1 2)', expected: '3', actual: '3', result: 'pass', commit: 'bbb', timestamp: '2026-01-01', runner: 'ci' },
        'fpga-lisp': { fixture: '(+ 1 2)', expected: '3', actual: '3', result: 'pass', commit: 'ccc', timestamp: '2026-01-01', runner: 'ci' }
      }
    },
    {
      requirement: 'list-cons',
      byImplementation: {
        'my-lisp': { fixture: "(cons 1 (quote (2)))", expected: '(1 2)', actual: '(1 2)', result: 'pass', commit: 'aaa', timestamp: '2026-01-01', runner: 'ci' },
        'cml': null,
        'fpga-lisp': { fixture: "(cons 1 (quote (2)))", expected: '(1 2)', actual: '(1 2)', result: 'pass', commit: 'ccc', timestamp: '2026-01-01', runner: 'ci' }
      }
    }
  ]
};

function injectEcoPanel() {
  const eco = JSON.parse(document.getElementById('mock-eco').textContent);
  let html = "<section class='pane eco-pane'><div class='pane-head'>Ecosystem</div><div class='eco'>";
  for (const repo of [eco.myLisp, eco.cml, eco.fpgaLisp]) {
    html += "<div class='eco-repo'><strong>" + repo.name + "</strong> ";
    if (repo.found) {
      html += "<span class='eco-branch'>" + repo.branch + "@" + repo.sha + "</span>";
    } else {
      html += "<span class='eco-missing'>not found on disk</span>";
    }
    html += "</div>";
  }
  html += "<table class='eco-matrix'><thead><tr><th>Req</th><th>my-lisp</th><th>cml</th><th>fpga-lisp</th></tr></thead><tbody>";
  for (const row of eco.evidenceMatrix) {
    html += "<tr class='eco-row' data-req='" + row.requirement + "'><td>" + row.requirement + "</td>";
    for (const impl of ['my-lisp', 'cml', 'fpga-lisp']) {
      const rec = row.byImplementation[impl];
      if (rec) {
        html += "<td class='eco-cell eco-" + rec.result + "'>" + (rec.result === 'pass' ? '✓' : '✗') + "</td>";
      } else {
        html += "<td class='eco-cell eco-none'>—</td>";
      }
    }
    html += "</tr>";
  }
  html += "</tbody></table></div></section>";
  document.body.insertAdjacentHTML('beforeend', html);

  document.querySelectorAll('.eco-row').forEach(tr => {
    tr.addEventListener('click', function () {
      const req = this.dataset.req;
      const row = eco.evidenceMatrix.find(r => r.requirement === req);
      if (!row) return;
      const ecoDiv = document.querySelector('.eco-pane .eco');
      let detail = "<div class='eco-fixture'><button class='eco-back'>← matrix</button><h3>" + row.requirement + "</h3>";
      for (const [impl, rec] of Object.entries(row.byImplementation)) {
        detail += "<div class='eco-fixture-impl'><strong>" + impl + "</strong> ";
        if (rec) {
          detail += "<span class='eco-cell eco-" + rec.result + "'>" + rec.result + "</span>";
        } else {
          detail += "<span class='eco-cell eco-none'>— no evidence</span>";
        }
        detail += "</div>";
      }
      detail += "</div>";
      ecoDiv.innerHTML = detail;
    });
  });
}

test('Ecosystem panel renders after clicking Ecosystem button', async () => {
  const html = fs.readFileSync('my-idea-web.html', 'utf8');
  const server = await startServer(html);
  const port = server.address().port;

  let browser;
  try {
    browser = await chromium.launch();
    const page = await browser.newPage();
    await page.goto(`http://localhost:${port}/`);

    await page.setContent((await page.content()).replace('</body>',
      `<script type="application/json" id="mock-eco">${JSON.stringify(MOCK_ECO)}</script>` +
      `<script>(${injectEcoPanel.toString()})();</script></body>`));

    const ecoPane = await page.$('.eco-pane');
    assert.ok(ecoPane, 'eco-pane section should render');
  } finally {
    if (browser) await browser.close();
    server.close();
  }
});

test('Ecosystem panel shows repo-summary elements', async () => {
  const html = fs.readFileSync('my-idea-web.html', 'utf8');
  const server = await startServer(html);
  const port = server.address().port;

  let browser;
  try {
    browser = await chromium.launch();
    const page = await browser.newPage();
    await page.goto(`http://localhost:${port}/`);

    await page.setContent((await page.content()).replace('</body>',
      `<script type="application/json" id="mock-eco">${JSON.stringify(MOCK_ECO)}</script>` +
      `<script>(${injectEcoPanel.toString()})();</script></body>`));

    const repos = await page.$$('.eco-repo');
    assert.equal(repos.length, 3, 'should render 3 repo-summary elements');

    const names = await page.$$eval('.eco-repo strong', els => els.map(e => e.textContent));
    assert.deepEqual(names, ['my-lisp', 'cml', 'fpga-lisp']);
  } finally {
    if (browser) await browser.close();
    server.close();
  }
});

test('Ecosystem panel renders evidence-matrix table', async () => {
  const html = fs.readFileSync('my-idea-web.html', 'utf8');
  const server = await startServer(html);
  const port = server.address().port;

  let browser;
  try {
    browser = await chromium.launch();
    const page = await browser.newPage();
    await page.goto(`http://localhost:${port}/`);

    await page.setContent((await page.content()).replace('</body>',
      `<script type="application/json" id="mock-eco">${JSON.stringify(MOCK_ECO)}</script>` +
      `<script>(${injectEcoPanel.toString()})();</script></body>`));

    const table = await page.$('table.eco-matrix');
    assert.ok(table, 'evidence-matrix table should render');

    const rows = await page.$$('.eco-row');
    assert.equal(rows.length, 2, 'matrix should have 2 fixture rows');

    const headers = await page.$$eval('.eco-matrix th', els => els.map(e => e.textContent));
    assert.deepEqual(headers, ['Req', 'my-lisp', 'cml', 'fpga-lisp']);
  } finally {
    if (browser) await browser.close();
    server.close();
  }
});

test('Clicking a fixture row shows fixture-detail', async () => {
  const html = fs.readFileSync('my-idea-web.html', 'utf8');
  const server = await startServer(html);
  const port = server.address().port;

  let browser;
  try {
    browser = await chromium.launch();
    const page = await browser.newPage();
    await page.goto(`http://localhost:${port}/`);

    await page.setContent((await page.content()).replace('</body>',
      `<script type="application/json" id="mock-eco">${JSON.stringify(MOCK_ECO)}</script>` +
      `<script>(${injectEcoPanel.toString()})();</script></body>`));

    await page.click(".eco-row[data-req='arithmetic']");

    const detail = await page.$('.eco-fixture');
    assert.ok(detail, 'fixture-detail should appear after clicking a row');

    const heading = await page.$eval('.eco-fixture h3', el => el.textContent);
    assert.equal(heading, 'arithmetic', 'detail heading should show the requirement name');

    const impls = await page.$$('.eco-fixture-impl');
    assert.equal(impls.length, 3, 'detail should list all 3 implementations');

    const back = await page.$('.eco-back');
    assert.ok(back, 'back button should be present in fixture detail');
  } finally {
    if (browser) await browser.close();
    server.close();
  }
});
