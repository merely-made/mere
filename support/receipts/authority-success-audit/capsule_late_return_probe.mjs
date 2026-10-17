// Audit-only: unchanged browser page and shared Runtime; explicit boundary doubles.
import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import vm from 'node:vm';

const root = new URL('../../../', import.meta.url);
const paths = {
  page: 'crates/moot/gemot/examples/capsule-library/browser.mjs',
  runtime: 'ports/graphshell/web/applets/runtime.mjs',
};
const sources = Object.fromEntries(await Promise.all(Object.entries(paths).map(async ([k, p]) => [k, await readFile(new URL(p, root), 'utf8')])));
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const source_sha256 = Object.fromEntries(await Promise.all(Object.values(paths).map(async p => [p, sha(await readFile(new URL(p, root)))])));

async function scenario(kind) {
  const entry = { url: 'gemini://fixture/capsule', revision: 'revision-a', author: 'a'.repeat(64), title: 'Delayed capsule', bytes: 14 };
  const pack = { manifest: { author: entry.author, parts: [{ blob: 'fixture-body', bytes: 14 }] } };
  const packBytes = new TextEncoder().encode(JSON.stringify(pack));
  const bodyBytes = new TextEncoder().encode('delayed body\n');
  const fixture = { moot: 'audit', catalogue: { entries: [entry] }, app: pack, capsule_packs: [pack], component_hash: 'component', browser_bindings_hash: 'bindings', publisher_exited: true };
  class Element {
    constructor() { this.textContent = ''; this.checked = true; this.disabled = false; this.hidden = false; this.value = ''; this.children = []; this.listeners = {}; }
    append(...children) { this.children.push(...children); }
    replaceChildren(...children) { this.children = children; }
    addEventListener(name, listener) { (this.listeners[name] ||= []).push(listener); }
    async fire(name) { for (const listener of this.listeners[name] || []) await listener({ target: this }); }
  }
  const elements = new Map();
  const el = id => { if (!elements.has(id)) elements.set(id, new Element()); return elements.get(id); };
  const receiptWindow = {};
  let releaseBody;
  let reachedBody;
  const bodyStarted = new Promise(resolve => { reachedBody = resolve; });
  const delayedBody = new Promise(resolve => { releaseBody = () => resolve(bodyBytes); });
  let partVerifications = 0;
  let closes = 0;
  class HonestWorkerDouble {
    postMessage({ id, command, data }) {
      queueMicrotask(() => {
        if (this.terminated) return;
        let actions = [];
        if (command === 'event' && data.kind === 'catalogue') actions = [{ name: 'capsule-library-view', payload: [entry] }];
        if (command === 'event' && data.kind === 'open') actions = [{ name: 'open-address', payload: { url: entry.url } }];
        this.onmessage({ data: { id, actions, logs: [], refusals: [], value: { component_hash: 'component', jco: 'audit-double' } } });
      });
    }
    terminate() { this.terminated = true; closes++; }
  }
  // IDB double always returns an absent capsule, so opening takes the network path.
  const db = {
    close() {},
    transaction() {
      const tx = { objectStore: () => ({
        get() { const request = { result: undefined }; queueMicrotask(() => tx.oncomplete()); return request; },
        count() { const request = { result: 0 }; queueMicrotask(() => request.onsuccess()); return request; },
      }) };
      return tx;
    },
  };
  const context = vm.createContext({
    document: { getElementById: el, createElement: () => new Element() }, window: receiptWindow,
    indexedDB: { open() { const request = { result: db }; queueMicrotask(() => request.onsuccess()); return request; } },
    Worker: HonestWorkerDouble, WebAssembly: {}, URL, TextDecoder, TextEncoder, Uint8Array,
    setTimeout, clearTimeout, console,
    fetch: async url => { assert.equal(url, 'fixture.json'); return { ok: true, json: async () => fixture }; },
  });
  const verification = new vm.SyntheticModule(['digest', 'verifyPack', 'verifyPart', 'fetchBytes'], function () {
    this.setExport('digest', bytes => new TextDecoder().decode(bytes) === new TextDecoder().decode(packBytes) ? entry.revision : 'body-hash');
    this.setExport('verifyPack', async () => {});
    this.setExport('verifyPart', () => { partVerifications++; });
    this.setExport('fetchBytes', async path => {
      if (path.endsWith('.json')) return packBytes;
      assert.equal(path, `capsules/${entry.revision}.gmi`);
      reachedBody(); return delayedBody;
    });
  }, { context });
  const runtime = new vm.SourceTextModule(sources.runtime, {
    context, identifier: paths.runtime,
    initializeImportMeta(meta) { meta.url = new URL(paths.runtime, root).href; },
  });
  const page = new vm.SourceTextModule(sources.page, { context, identifier: paths.page });
  await page.link(specifier => {
    if (specifier === './verify.mjs') return verification;
    assert.equal(specifier, './runtime.mjs'); return runtime;
  });
  await page.evaluate();
  assert.equal(receiptWindow.__capsuleProof.ready, true);
  assert.equal(receiptWindow.__capsuleProof.error, undefined);
  await el('start').fire('click');
  assert.equal(el('entries').children.length, 1);
  const opening = el('entries').children[0].children[2].fire('click');
  await bodyStarted;
  assert.equal(el('body').textContent, '');
  if (kind === 'revoke') {
    el('grant-nav').checked = false;
    await el('grant-nav').fire('change');
    assert.ok(receiptWindow.__capsuleProof.revoked.includes('power:navigate'));
    assert.equal(el('entries').children[0].children[2].disabled, true);
  } else if (kind === 'stop') {
    await el('stop').fire('click');
    assert.equal(el('stop').disabled, true);
    assert.equal(el('entries').children.length, 0);
    assert.ok(closes > 0, 'unchanged Runtime.close really terminated its Worker double');
  } else assert.equal(kind, 'control');
  const notice_before_return = el('notice').textContent;
  releaseBody(); await opening;
  assert.equal(receiptWindow.__capsuleProof.error, undefined);
  assert.equal(el('body').textContent, 'delayed body\n');
  assert.equal(receiptWindow.__capsuleProof.last_opened, entry.revision);
  assert.equal(partVerifications, 1);
  return { case: kind, verified_part_calls: partVerifications, body_published_after_boundary: kind !== 'control',
    last_opened: receiptWindow.__capsuleProof.last_opened, notice_before_return, notice_after_return: el('notice').textContent,
    closed_workers: closes };
}

const cases = [];
for (const kind of ['control', 'revoke', 'stop']) cases.push(await scenario(kind));
const receipt = { source_sha256, runner_sha256: sha(await readFile(fileURLToPath(import.meta.url))),
  checks: cases.length, cases, node_version: process.version,
  qualification: 'Fresh Node VM execution of unchanged complete standalone browser.mjs and shared runtime.mjs. DOM, honest Worker replies, absent-only IndexedDB, network and cryptographic verification are explicit doubles. This tests host continuation and publication after grant revocation or completed Stop, not actual Chromium/Wasm, signature authenticity, real IDB, live transport or the separately implemented Graphshell host.' };
await writeFile(new URL('./capsule_late_return_probe_receipt.json', import.meta.url), JSON.stringify(receipt, null, 2) + '\n', { flag: 'wx' });
console.log(JSON.stringify(receipt, null, 2));
