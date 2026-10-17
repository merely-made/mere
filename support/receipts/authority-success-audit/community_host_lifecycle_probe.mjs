// Audit diagnostic. Imports current production modules without modifying them.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { mountCapsuleApplet } from '../../../ports/graphshell/web/applets/host.mjs';
import { Runtime } from '../../../ports/graphshell/web/applets/runtime.mjs';

const here = new URL('./', import.meta.url);
const receipt = new URL('community_host_lifecycle_probe_receipt.json', here);
assert.equal(fs.existsSync(receipt), false, 'preserve prior evidence');
const tests = [];
async function check(name, fn) {
  await fn(); tests.push(name); console.log('PASS ' + name);
}
const sourceNames = [
  'ports/graphshell/web/applets/host.mjs',
  'ports/graphshell/web/applets/runtime.mjs',
  'ports/graphshell/web/mount.js',
  'ports/graphshell/src/bin/moot_resources_host.rs',
  'ports/graphshell/src/web_tree/applet.rs',
  'ports/graphshell/web/tests/presentation-mount.mjs',
];
const sourceSha = Object.fromEntries(sourceNames.map(name => [name,
  createHash('sha256').update(fs.readFileSync(new URL('../../../' + name, here))).digest('hex')]));
const originalTimeout = globalThis.setTimeout;
const originalFetch = globalThis.fetch;
globalThis.location = { href: 'https://fixture.invalid/' };
globalThis.setTimeout = () => 0; // No background pump is run by this fixture.
function fixture() {
  let generation = 0, reviews = 0, closes = 0;
  const gs = {
    review_applet() { reviews++; return ++generation; },
    applet_receipt: () => JSON.stringify({ generation, component_hash: 'fixture' }),
    applet_commands: () => '[]',
    applet_close() { closes++; generation++; },
  };
  const root = { dataset: { ready: 'true' } };
  const config = { baseURL: 'https://fixture.invalid/', packURL: 'pack.json',
    packHash: 'fixture', bindingsHash: '0'.repeat(64), disclosure: { capsules: [] } };
  return { gs, root, config, state: () => ({ generation, reviews, closes }) };
}
const response = url => new Response(String(url).endsWith('pack.json') ? '{}' : 'component');
globalThis.fetch = async url => response(url);
try {
  await check('control: duplicate live root is refused', async () => {
    const f = fixture(), host = await mountCapsuleApplet(f.root, f.gs, f.config);
    await assert.rejects(mountCapsuleApplet(f.root, f.gs, f.config), /already has/);
    assert.equal(f.state().reviews, 1); host.close();
  });
  await check('counterexample: closed handle reviews and installs another scope', async () => {
    const f = fixture(), host = await mountCapsuleApplet(f.root, f.gs, f.config);
    host.close(); const before = f.state();
    const generation = await host.review(f.config);
    assert.equal(f.state().reviews, before.reviews + 1);
    assert.equal(generation, before.generation + 1);
  });
  await check('counterexample: pending review installs after close', async () => {
    const f = fixture(), host = await mountCapsuleApplet(f.root, f.gs, f.config);
    let release;
    globalThis.fetch = url => String(url).endsWith('pack.json')
      ? new Promise(resolve => { release = () => resolve(response(url)); })
      : Promise.resolve(response(url));
    const pending = host.review(f.config); assert.equal(typeof release, 'function');
    host.close(); const before = f.state(); release(); await pending;
    assert.equal(f.state().reviews, before.reviews + 1);
    globalThis.fetch = async url => response(url);
  });
  await check('counterexample: repeated stale close displaces remount and removes live root guard', async () => {
    const f = fixture(), first = await mountCapsuleApplet(f.root, f.gs, f.config);
    first.close(); const second = await mountCapsuleApplet(f.root, f.gs, f.config);
    const before = f.state(); first.close();
    assert.equal(f.state().closes, before.closes + 1);
    const third = await mountCapsuleApplet(f.root, f.gs, f.config);
    assert.equal(f.state().reviews, before.reviews + 1);
    second.close(); third.close();
  });
} finally {
  globalThis.setTimeout = originalTimeout; globalThis.fetch = originalFetch;
}

class Worker {
  static last;
  constructor() { Worker.last = this; this.messages = []; this.terminations = 0; }
  postMessage(value) { this.messages.push(value); }
  terminate() { this.terminations++; }
}
globalThis.Worker = Worker;
await check('control: worker completion consumes exact pending request', async () => {
  const runtime = new Runtime(), promise = runtime.call('event', 'fixture');
  runtime.worker.onmessage({ data: { id: 1, value: 'done' } });
  assert.equal((await promise).value, 'done'); assert.equal(runtime.pending.size, 0); runtime.close();
});
await check('control: close rejects pending work and ignores late worker completion', async () => {
  const runtime = new Runtime(), a = runtime.call('event', 'a'), b = runtime.call('event', 'b');
  const rejected = [assert.rejects(a, /Applet stopped/), assert.rejects(b, /Applet stopped/)];
  runtime.close(); await Promise.all(rejected);
  runtime.worker.onmessage({ data: { id: 1, value: 'late' } });
  assert.equal(runtime.pending.size, 0); assert.equal(runtime.closed, true);
});
await check('control: deadline terminates worker and rejects all pending turns', async () => {
  const runtime = new Runtime(), a = runtime.call('event', 'a', 5), b = runtime.call('event', 'b');
  await Promise.all([assert.rejects(a, /interrupted/), assert.rejects(b, /interrupted/)]);
  assert.equal(runtime.pending.size, 0); assert.equal(runtime.worker.terminations, 1);
});

const definitions = new Map(), observers = [];
globalThis.HTMLElement = class {};
globalThis.customElements = { get: name => definitions.get(name), define: (name, type) => definitions.set(name, type) };
globalThis.MutationObserver = class {
  constructor(callback) { this.callback = callback; observers.push(this); }
  observe(target) { this.target = target; }
  disconnect() {}
};
location.search = '?render_backend=auto';
const source = fs.readFileSync(new URL('../../../ports/graphshell/web/mount.js', here), 'utf8');
const { defineElements } = await import('data:text/javascript;base64,' + Buffer.from(source).toString('base64'));
await check('counterexample: asynchronous Auto fallback load failure clears refusal and leaves mounted latch', async () => {
  let disposed = 0, attached = 0;
  defineElements(async choice => {
    if (choice === 'webgl2') throw new Error('WebGL2 artifact unavailable');
    return { artifact: 'webgpu', module: {
      mount_tree() { attached++; }, dispose_tree() { disposed++; },
    } };
  });
  const root = new (definitions.get('graphshell-tree'))();
  root.dataset = {}; root.attrs = new Map(); root.isConnected = true;
  root.hasAttribute = k => root.attrs.has(k); root.getAttribute = k => root.attrs.get(k) ?? null;
  root.setAttribute = (k, v) => root.attrs.set(k, v); root.removeAttribute = k => root.attrs.delete(k);
  await root.connectedCallback();
  root.setAttribute('data-mount-error', 'WebGPU: no adapter; WebGL2: not compiled into this host');
  await assert.rejects(observers.at(-1).callback(), /WebGL2 artifact unavailable/);
  assert.equal(disposed, 1); assert.equal(root.getAttribute('data-mount-error'), null);
  assert.equal(root.dataset.mounted, 'true');
  await root.connectedCallback(); assert.equal(attached, 1);
});
const value = {
  tests: tests.length, passed: tests, source_sha256: sourceSha,
  diagnostic_sha256: createHash('sha256').update(fs.readFileSync(fileURLToPath(import.meta.url))).digest('hex'),
  scope: 'Actual unmodified JS host/runtime and element module; synthetic root, gs callbacks, fetch responses, workers and MutationObserver. No browser renderer, Rust signature validation, real worker process, network, installed profile or device qualification.',
  production_changes: false,
};
fs.writeFileSync(receipt, JSON.stringify(value, null, 2) + '\n', { flag: 'wx' });
console.log(JSON.stringify({ tests: value.tests, receipt: fileURLToPath(receipt) }));
