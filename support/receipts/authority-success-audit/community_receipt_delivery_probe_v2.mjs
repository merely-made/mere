// Execute the exact current loader listener with controlled DOM/HTTP outcomes.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import { createHash } from 'node:crypto';
const here = new URL('./', import.meta.url);
const output = new URL('community_receipt_delivery_probe_v2_receipt.json', here);
assert.equal(fs.existsSync(output), false);
const file = new URL('../../../ports/graphshell/web/loader.js', here);
const source = fs.readFileSync(file, 'utf8');
const eventAt = source.indexOf('"graphshell-scenario-complete"');
const start = source.lastIndexOf('document.addEventListener(', eventAt);
assert(start >= 0);
const optionsAt = source.indexOf('{ once: true }', start);
const end = source.indexOf(');', optionsAt) + 2;
assert(end > start);
const listener = source.slice(start, end);
const tests = [];
for (const outcome of ['204', '500', 'transport rejection', 'page gate']) {
  let handler, requested;
  const body = { dataset: { scenario: 'ok' } };
  const scenario = { gate_failures: outcome === 'page gate' ? ['planted error'] : [],
    result: { result: 'ok', log: [] } };
  const context = vm.createContext({
    document: { body, querySelectorAll: () => [], addEventListener(name, fn) {
      assert.equal(name, 'graphshell-scenario-complete'); handler = fn;
    } },
    window: { graphshellScenario: () => scenario, graphshellReceipt: () => ({ fixture: true }) },
    sink: 'https://fixture.invalid/scenario-receipt',
    fetch: async (_url, request) => {
      requested = JSON.parse(request.body);
      if (outcome === 'transport rejection') throw new Error('fixture network failure');
      return { ok: outcome !== '500', status: outcome === '500' ? 500 : 204 };
    },
  });
  new vm.Script(listener, { filename: 'loader.js:exact-scenario-complete-listener' }).runInContext(context);
  await handler();
  if (outcome === 'transport rejection') assert.match(body.dataset.scenarioSink, /^failed:/);
  else assert.equal(body.dataset.scenarioSink, 'delivered');
  if (outcome === 'page gate') {
    assert.equal(requested.scenario.result.result, 'fail'); assert.equal(body.dataset.scenario, 'fail');
  }
  const name = outcome === '500' ? 'counterexample: HTTP 500 is reported delivered' : 'control: ' + outcome;
  tests.push(name); console.log('PASS ' + name);
}
fs.writeFileSync(output, JSON.stringify({
  tests: tests.length, passed: tests,
  source_sha256: { 'ports/graphshell/web/loader.js': createHash('sha256').update(fs.readFileSync(file)).digest('hex') },
  exact_extracted_listener: { start_char: start, end_char: end, sha256: createHash('sha256').update(listener).digest('hex') },
  diagnostic_sha256: createHash('sha256').update(fs.readFileSync(new URL(import.meta.url))).digest('hex'),
  scope: 'Exact current scenario-complete listener, unchanged; synthetic DOM and fetch responses. No browser, server, network or durable sink write. HTTP rejection is distinguished from transport rejection; existing page gate still forces a failed scenario.',
}, null, 2) + '\n', { flag: 'wx' });
