// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
// Prepared only. Root owns all browser/GPU launches and adapter requests.
const fs = require('fs'), path = require('path'), crypto = require('crypto');
const assert = require('node:assert/strict');
const { chromium } = require(process.env.PLAYWRIGHT_MODULE || '/Users/markik/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const flags = Object.fromEntries(process.argv.slice(2).reduce((items, item, index, args) => item.startsWith('--') ? [...items, [item.slice(2), args[index + 1]]] : items, []));
for (const name of ['base', 'output', 'identity', 'scenario']) assert.ok(flags[name], `--${name} required`);
const out = path.resolve(flags.output), identity = JSON.parse(fs.readFileSync(flags.identity));
assert.equal(identity.sealed, true, 'a source-bound current build must be sealed');
fs.mkdirSync(out, { recursive: true });
const write = (name, value) => fs.writeFileSync(path.join(out, name), JSON.stringify(value, null, 2));
const sha = buffer => crypto.createHash('sha256').update(buffer).digest('hex');
async function fileSha(file) {
  const hash = crypto.createHash('sha256');
  for await (const chunk of fs.createReadStream(file)) hash.update(chunk);
  return hash.digest('hex');
}
let browser, context, page;
const errors = [], runs = [];

async function pixels(dataUrl) {
  return page.evaluate(async dataUrl => {
    const image = new Image(); image.src = dataUrl; await image.decode();
    // Decode only the actual host readback PNG; this canvas creates no app content.
    const canvas = document.createElement('canvas'); canvas.width = image.naturalWidth; canvas.height = image.naturalHeight;
    const cx = canvas.getContext('2d', { willReadFrequently: true }); cx.drawImage(image, 0, 0);
    function measure(x, y, width, height) {
      const data = cx.getImageData(x, y, width, height).data;
      const colors = new Map(); let opaque = 0, count = 0;
      for (let index = 0; index < data.length; index += 16) {
        const key = `${data[index]},${data[index + 1]},${data[index + 2]},${data[index + 3]}`;
        colors.set(key, (colors.get(key) || 0) + 1); opaque += data[index + 3] > 127; count++;
      }
      let dominant = 0; for (const number of colors.values()) dominant = Math.max(dominant, number);
      let idleNodeAccentPixels = 0;
      // Actual Canvas IDLE node fill, canvas/palette.rs; unaffected by the theme's face inset colors.
      // UI overlays alone must not satisfy producer-content acceptance.
      for (let index = 0; index < data.length; index += 4) {
        if (data[index + 3] >= 250 && Math.abs(data[index] - 54) <= 3 && Math.abs(data[index + 1] - 92) <= 3 && Math.abs(data[index + 2] - 156) <= 3) idleNodeAccentPixels++;
      }
      return { x, y, width, height, idleNodeAccentPixels, sampled: count, colors: colors.size,
        dominantFraction: dominant / count, opaqueFraction: opaque / count };
    }
    const surface = document.getElementById('graphshell-canvas');
    const slot = document.querySelector('graphshell-tree [role="group"][aria-label$=" shown"]');
    if (!surface || !slot) throw new Error('actual producer semantic geometry is unavailable');
    const target = surface.getBoundingClientRect(), leaf = slot.getBoundingClientRect();
    const scaleX = canvas.width / target.width, scaleY = canvas.height / target.height;
    const x = Math.max(0, Math.floor((leaf.x - target.x) * scaleX));
    const y = Math.max(0, Math.floor((leaf.y - target.y) * scaleY));
    const width = Math.min(canvas.width - x, Math.floor(leaf.width * scaleX));
    const height = Math.min(canvas.height - y, Math.floor(leaf.height * scaleY));
    if (width < 32 || height < 32) throw new Error('actual producer geometry is empty or outside the capture');
    return { ...measure(0, 0, canvas.width, canvas.height),
      producer: measure(x, y, width, height), producerSemanticName: slot.getAttribute('aria-label') };
  }, dataUrl);
}

async function run(name, text) {
  // publish_capture appends images; only inspect this run's new readbacks.
  const captureStart = await page.locator('#scenario-captures img').count();
  await page.evaluate(async text => (await import('./pkg/graphshell_web.js')).run_scenario(text), text);
  await page.waitForFunction(() => ['ok', 'fail'].includes(document.body.dataset.scenario), null, { timeout: 180000 });
  const report = await page.evaluate(captureStart => ({ scenario: window.graphshellScenario(), receipt: window.graphshellReceipt(),
    hidden: document.hidden, appearance: Object.fromEntries([...document.querySelector('graphshell-tree').attributes].filter(a => a.name.startsWith('data-appearance')).map(a => [a.name, a.value])),
    captures: [...document.querySelectorAll('#scenario-captures img')].slice(captureStart).map(image => ({ name: image.dataset.capture, dataUrl: image.src })) }), captureStart);
  const directory = path.join(out, name); fs.mkdirSync(directory, { recursive: true });
  report.pixelMetrics = [];
  for (const capture of report.captures) {
    const bytes = Buffer.from(capture.dataUrl.split(',')[1], 'base64');
    fs.writeFileSync(path.join(directory, capture.name + '.png'), bytes);
    const metric = { name: capture.name, sha256: sha(bytes), ...await pixels(capture.dataUrl) };
    report.pixelMetrics.push(metric);
  }
  delete report.captures;
  report.errors = [...errors];
  fs.writeFileSync(path.join(directory, 'result.json'), JSON.stringify(report, null, 2));
  await page.screenshot({ path: path.join(directory, 'browser.png') });
  assert.equal(report.hidden, false, 'visible frame pump required');
  assert.equal(report.scenario.result?.result, 'ok', JSON.stringify(report.scenario));
  assert.equal(report.scenario.gate_failures.length, 0);
  assert.equal(report.scenario.errors.length, 0, report.scenario.errors.join('\n'));
  assert.equal(errors.length, 0, errors.join('\n'));
  assert.ok(report.pixelMetrics.length > 0, 'actual composed-frame capture required');
  for (const metric of report.pixelMetrics) {
    assert.ok(metric.width > 100 && metric.height > 100 && metric.colors >= 16 && metric.dominantFraction < 0.995 && metric.opaqueFraction > 0.25,
      'nonblank host readback failed: ' + JSON.stringify(metric));
    assert.ok(metric.producer.colors >= 16 && metric.producer.dominantFraction < 0.995 && metric.producer.opaqueFraction > 0.25,
      'nonblank actual producer region failed: ' + JSON.stringify(metric));
    assert.ok(metric.producer.idleNodeAccentPixels >= 64, 'actual idle graph-node paint must be visible in the producer region: ' + JSON.stringify(metric));
  }
  runs.push({ name, result: 'pass', captures: report.pixelMetrics.length });
  console.log('PASS', name);
  return report;
}

async function mount() {
  await page.goto(flags.base + '/tree.html?app=local'); await page.bringToFront();
  await page.waitForFunction(() => document.querySelector('graphshell-tree')?.dataset.ready === 'true' || document.title.includes('FAIL'), null, { timeout: 120000 });
  assert.ok(!await page.title().then(title => title.includes('FAIL')), 'production boot refused');
}

async function ensurePaused(name) {
  // Analytic arrangement boot now starts paused; query the current semantic
  // control, and issue the actual Pause action only when physics is playing.
  await page.waitForFunction(() => [...document.querySelectorAll('graphshell-tree [role="button"], graphshell-tree button')]
    .some(node => ['Play physics', 'Pause physics'].includes(node.getAttribute('aria-label') || node.textContent.trim())));
  const pause = page.getByRole('button', { name: 'Pause physics', exact: true });
  const count = await pause.count(); assert.ok(count <= 1, 'unique physics toggle required');
  if (count === 1) {
    await run(name + '-ensure-paused', `click role:button Pause physics\nsettle 2\nassert snap physics-paused == true\ncapture renderer_${name}_paused\nwait 120\n`);
  } else {
    assert.equal(await page.getByRole('button', { name: 'Play physics', exact: true }).count(), 1,
      'paused state must expose the current Play control');
  }
}

function captureScenario(name) {
  return `assert snap ready == true\nassert snap nodes >= 1\nassert snap finite == true\nassert snap physics-paused == true\nsettle 2\nremember authority\ncapture ${name}\nwait 120\nsame authority nodes graph-session geometry source rules-committed\n`;
}

(async () => {
  try {
    // No unsafe-WebGPU, software-renderer or GPU-disabling launch flags.
    browser = flags.cdp ? await chromium.connectOverCDP(flags.cdp) : await chromium.launch({ channel: 'chrome', headless: false });
    context = await browser.newContext({ viewport: { width: 1440, height: 1000 }, deviceScaleFactor: 1, reducedMotion: 'no-preference' });
    page = await context.newPage(); page.on('pageerror', error => errors.push(error.message));
    await page.goto(flags.base + '/qualification-probe.html'); await page.bringToFront();
    await page.waitForFunction(() => document.getElementById('probe')?.textContent.startsWith('{'));
    const capability = await page.locator('#probe').evaluate(element => JSON.parse(element.textContent));
    capability.actualDefaultAdapter = await page.evaluate(async () => {
      const adapter = await navigator.gpu?.requestAdapter();
      if (!adapter) return { available: false, request: 'default' };
      const info = adapter.info;
      const adapterFlag = typeof adapter.isFallbackAdapter === 'boolean';
      const infoFlag = typeof info?.isFallbackAdapter === 'boolean';
      return { available: true, request: 'default',
        info: info ? { vendor: info.vendor, architecture: info.architecture, device: info.device, description: info.description } : null,
        isFallbackAdapter: adapterFlag ? adapter.isFallbackAdapter : infoFlag ? info.isFallbackAdapter : null,
        classificationSource: adapterFlag ? 'GPUAdapter.isFallbackAdapter' : infoFlag ? 'GPUAdapterInfo.isFallbackAdapter' : 'not_reported' };
    });
    write('capability.json', capability);
    assert.equal(capability.secureContext, true, 'secure context required');
    assert.equal(capability.webgpu, true, 'REFUSED: current browser WebGPU unavailable');
    assert.ok(capability.adapter, 'REFUSED: current default WebGPU adapter unavailable');
    assert.equal(capability.actualDefaultAdapter.available, true);
    assert.notEqual(capability.actualDefaultAdapter.isFallbackAdapter, true, 'REFUSED: default adapter reports fallback');
    // The isolated server maps /pkg directly to this sealed directory. Hash
    // files as streams and compare HTTP lengths; avoid another huge Wasm copy.
    for (const [name, artifact] of Object.entries(identity.artifacts)) {
      const file = path.join(identity.package, name);
      assert.equal(fs.statSync(file).size, artifact.bytes);
      assert.equal(await fileSha(file), artifact.sha256);
      const response = await context.request.head(flags.base + '/pkg/' + name);
      assert.equal(response.ok(), true);
      assert.equal(Number(response.headers()['content-length']), artifact.bytes);
    }
    await mount(); await ensurePaused('cold');
    const cold = await run('production', fs.readFileSync(flags.scenario, 'utf8'));
    const session = cold.scenario.result.saved_graph?.session;
    assert.ok(session, 'actual saved application session identity required');
    await page.locator('.graphshell-appearance > summary').click();
    assert.equal(await page.locator('.graphshell-appearance').evaluate(node => node.open), true, 'actual appearance panel must be open');
    const mode = page.locator('[data-appearance-mode-select]');
    for (const choice of ['dark', 'light', 'hc_dark', 'hc_light']) {
      await mode.selectOption(choice);
      await page.waitForFunction(choice => document.querySelector('graphshell-tree').getAttribute('data-appearance-mode') === choice, choice);
      const report = await run('mode-' + choice, captureScenario('renderer_mode_' + choice));
      assert.equal(report.scenario.result.saved_graph.session, session);
    }
    await page.setViewportSize({ width: 420, height: 900 });
    const toolsToggle = page.getByRole('button', { name: 'Graph tools', exact: true });
    await toolsToggle.waitFor();
    const narrowActions = (await toolsToggle.getAttribute('aria-expanded') === 'true' ? 'click role:button Graph tools\nsettle 2\n' : '') + 'click role:button Fit graph\nsettle 2\n';
    const narrow = await run('narrow', narrowActions + captureScenario('renderer_narrow'));
    assert.equal(await toolsToggle.getAttribute('aria-expanded'), 'false', 'narrow graph must be exposed rather than covered by the tools drawer');
    assert.equal(narrow.scenario.result.saved_graph.session, session);
    await page.setViewportSize({ width: 1440, height: 1000 });
    const resized = await run('resized', captureScenario('renderer_resized'));
    assert.notDeepEqual(narrow.scenario.result.canvas_physical_size, resized.scenario.result.canvas_physical_size);
    assert.equal(resized.scenario.result.saved_graph.session, session);
    // Fresh page, same isolated context/origin: actual stored graph rebind.
    await page.close(); page = await context.newPage(); page.on('pageerror', error => errors.push(error.message));
    await mount(); await ensurePaused('rebound');
    const rebound = await run('rebound', captureScenario('renderer_rebound'));
    assert.equal(rebound.scenario.result.saved_graph.session, session);
    assert.equal(rebound.appearance['data-appearance-mode'], 'hc_light');
    write('qualification.json', { status: 'pass', runs, browser: browser.version(), identity, errors, capability, driver_sha256: sha(fs.readFileSync(__filename)), scenario_sha256: sha(fs.readFileSync(flags.scenario)),
      graphPaintGuard: { paletteSource: 'crates/canvas/pictograph/src/canvas/palette.rs', sourceSha256: identity.sources[0].source_sha256['crates/canvas/pictograph/src/canvas/palette.rs'], idleRgb: [54,92,156], perChannelTolerance: 3, minimumOpaquePixels: 64 },
      scope: 'Current product CanvasProducer and Rootstock GPU readback; draft, mode, resize, page rebind, input and zoom authority',
      limitations: ['Pixel metrics are nonblank guards, not cross-renderer pixel equivalence', 'Browser CanvasProducer portable resource-bundle export is not implemented', capability.actualDefaultAdapter.isFallbackAdapter === null ? 'Default adapter fallback classification unavailable in this browser API' : 'Default adapter fallback classification recorded in capability.json'] });
  } catch (error) {
    write('qualification.json', { status: 'refused_or_failed', reason: String(error), runs, errors, identity });
    console.error(error); process.exitCode = 1;
  } finally {
    await context?.close(); if (!flags.cdp) await browser?.close();
  }
})();
