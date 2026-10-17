// Evaluate the unchanged error hooks in a standard Node VM with fixture DOM storage.
const fs = require('node:fs');
const vm = require('node:vm');
const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const path = require('node:path');
const here = __dirname;
const root = path.resolve(here, '../../..');
const source = 'crates/cambium/examples/genet_web_smoke/index.html';
const html = fs.readFileSync(path.join(root, source), 'utf8');
const start = html.indexOf('  const origError');
const end = html.indexOf('  try {', start);
assert(start >= 0 && end > start);
const hooks = html.slice(start, end);
const sha = x => crypto.createHash('sha256').update(x).digest('hex');
const destination = path.join(here, 'smoke_title_probe_receipt.json');
assert(!fs.existsSync(destination));
function fixture(title) {
  const listeners = {};
  const errors = [];
  const context = { document: { title }, console: { error: (...a) => errors.push(a) }, window: { addEventListener: (n, fn) => { listeners[n] = fn; } } };
  vm.runInNewContext(hooks, context);
  return { context, listeners, errors };
}
const early = fixture('stage: present');
early.context.console.error('GPU validation failed');
assert.equal(early.context.document.title, 'ERR: GPU validation failed');
const late = fixture('SMOKE PASS');
late.context.console.error('GPU validation failed');
assert.equal(late.context.document.title, 'SMOKE PASS');
late.listeners.error({ message: 'uncaught later failure' });
assert.equal(late.context.document.title, 'SMOKE PASS');
assert.equal(late.errors.length, 1, 'error remains logged, but title outcome is unchanged');
const receipt = { date_utc: new Date().toISOString(), source_sha256: { [source]: sha(fs.readFileSync(path.join(root, source))) }, runner_sha256: sha(fs.readFileSync(__filename)), extracted_hooks_sha256: sha(hooks), checks: 3, run_exit: 0, generated_output_removed: true, scope: 'Three assertions around unchanged smoke console/window error hooks in Node VM fixture storage: before PASS title fails, after PASS both console.error and window error leave PASS. This is JavaScript hook logic, not a browser/WebGPU/rendered-pixel run. Console logging itself is retained.' };
fs.writeFileSync(destination, JSON.stringify(receipt, null, 2) + '\n', { flag: 'wx' });
process.stdout.write(JSON.stringify({ checks: 3, title_after_late_errors: late.context.document.title }) + '\n');
