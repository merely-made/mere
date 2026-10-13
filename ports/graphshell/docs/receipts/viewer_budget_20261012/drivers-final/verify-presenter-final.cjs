// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
const fs=require('fs'),path=require('path'),assert=require('node:assert/strict');
const {chromium}=require('/Users/markik/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const out=path.join(__dirname,'presenter-final-browser-'+(process.env.RECEIPT_ROUND||'r1'));fs.mkdirSync(out,{recursive:true});
const base='http://127.0.0.1:8847/web-final';
(async()=>{
 require('./assert-console-unlocked.cjs')();
 const browser=await chromium.launch({channel:'chrome',headless:false,args:['--enable-unsafe-webgpu']});
 const runs=[];
 try {
  const context=await browser.newContext({viewport:{width:1440,height:1000},deviceScaleFactor:1});
  const page=await context.newPage();page.setDefaultTimeout(30000);let errors=[];page.on('pageerror',e=>errors.push(e.message));
  async function mount(url){errors=[];await page.goto(base+url);await page.bringToFront();await page.waitForFunction(()=>document.querySelector('graphshell-tree')?.dataset.ready==='true',null,{timeout:120000});}
  async function run(name,text,expected='ok'){
   await page.evaluate(async text=>{const m=await import('./pkg/graphshell_web.js');m.run_scenario(text)},text);
   await page.waitForFunction(()=>['ok','fail'].includes(document.body.dataset.scenario),null,{timeout:120000});
   const result=await page.evaluate(()=>({receipt:window.graphshellScenario(),host:window.graphshellReceipt(),hidden:document.hidden}));
   fs.writeFileSync(path.join(out,name+'.json'),JSON.stringify({...result,errors},null,2));
   await page.screenshot({path:path.join(out,name+'.png')});
   assert.equal(result.receipt.result?.result,expected,JSON.stringify(result.receipt));assert.equal(result.receipt.gate_failures.length,0);assert.equal(errors.length,0);assert.equal(result.hidden,false);
   console.log('PASS',name);runs.push(name);return result;
  }
  const source=file=>fs.readFileSync(path.join(__dirname,'web-final/scenarios',file),'utf8');
  await mount('/grouped-dataset.html?presenter=inspector');
  const initialAdapter=await page.evaluate(async()=>{const a=await navigator.gpu.requestAdapter();return a?{vendor:a.info.vendor,architecture:a.info.architecture,device:a.info.device,description:a.info.description}:null});assert.ok(initialAdapter&&/amd|ati|radeon/i.test(JSON.stringify(initialAdapter))&&!/swiftshader|llvmpipe/i.test(JSON.stringify(initialAdapter)));fs.writeFileSync(path.join(out,'adapter.json'),JSON.stringify(initialAdapter,null,2));
  await run('fold-selection',source('hosted_presenter.scn'));
  await mount('/tree.html?dataset=fixtures/grouped-host-history.json&grouping=contains&presenter=inspector');
  await run('checkpoint-selection',source('hosted_presenter_history.scn'));
  await mount('/tree.html?dataset=fixtures/site-repository-host-dataset.json&presenter=inspector');
  const choose=page.getByRole('button',{name:'Choose projects',exact:true});await choose.press('Enter');
  const select=page.getByRole('button',{name:'Select Mere',exact:true});await select.press('Enter');
  await run('site-keyboard-selection','assert snap presenter-selected == mere\nassert snap presenter-inspected == mere\nassert snap canvas-selection-count == 1\n');
  assert.ok(await page.getByText('Mere',{exact:true}).count());
  const panel=await page.getByRole('region',{name:'Project inspector',exact:true}).boundingBox();
  const graph=await page.getByRole('group',{name:/^\d+ of \d+ shown$/}).boundingBox();assert.ok(panel&&graph);
  assert.ok(panel.x>=graph.x+graph.width-2,'wide inspector must sit beside the graph');
  await page.getByRole('button',{name:'Show source details',exact:true}).press('Enter');
  const cdp=await context.newCDPSession(page);const ax=await cdp.send('Accessibility.getFullAXTree');
  fs.writeFileSync(path.join(out,'site-accessibility.json'),JSON.stringify(ax,null,2));
  assert.ok(ax.nodes.some(n=>n.role?.value==='region'&&n.name?.value==='Project inspector'));
  assert.ok(ax.nodes.some(n=>n.role?.value==='button'&&n.name?.value==='Clear project selection'));
  await page.screenshot({path:path.join(out,'site-wide-inspector.png')});
  const related=page.getByRole('button',{name:/^Inspect related /}).first();
  assert.ok(await related.count(),'Mere must disclose a related project');{
   await related.press('Enter');await run('related-inspection-keeps-subject','assert snap presenter-selected == mere\nassert snap canvas-selection-count == 1\n');
  }
  await page.getByRole('button',{name:'Clear project selection',exact:true}).press('Enter');
  await run('keyboard-clear-focus','assert snap presenter-selection-count == 0\nassert snap presenter-inspector-open == false\nassert snap focus-kind == graph\n');
  await page.setViewportSize({width:420,height:800});await mount('/grouped-dataset.html?presenter=inspector');
  await page.getByRole('button',{name:'Choose projects',exact:true}).press('Enter');
  await page.getByRole('button',{name:'Select genet',exact:true}).press('Enter');
  await run('narrow-keyboard-selection','assert snap presenter-selected == repo:genet\nassert snap presenter-inspected == repo:genet\n');
  const narrowPanel=await page.getByRole('region',{name:'Project inspector',exact:true}).boundingBox();
  const narrowGraph=await page.getByRole('group',{name:/^\d+ of \d+ shown$/}).boundingBox();assert.ok(narrowPanel&&narrowGraph);
  assert.ok(narrowPanel.y>=narrowGraph.y+narrowGraph.height-2,'narrow inspector must stack under the graph');
  assert.ok(narrowPanel.x>=-1&&narrowPanel.x+narrowPanel.width<=421,'inspector must fit narrow width');
  await page.screenshot({path:path.join(out,'narrow-inspector.png')});
  await page.getByRole('button',{name:'Graph commands',exact:true}).press('Enter');
  const search=page.getByRole('textbox',{name:'Search graph commands',exact:true});await search.waitFor();await search.press('Escape');
  await run('narrow-command-focus','assert snap commands-open == false\nassert snap focus-kind == graph\nassert snap presenter-selected == repo:genet\n');
  await page.setViewportSize({width:1440,height:1000});await mount('/grouped-dataset.html?presenter=inspector');
  const control=source('hosted_presenter.scn').replace('click role:button Collapse mere','click role:button Clear project selection\nclick role:button Collapse mere');
  const fault=await run('selection-loss-control',control,'fail');
  assert.match(JSON.stringify(fault.receipt.result.log),/presenter-selected/);
  await page.goto(base+'/tree.html?dataset=fixtures/site-repository-host-dataset.json&presenter=unknown');
  await page.waitForFunction(()=>document.title.includes('Unknown presenter'));assert.notEqual(await page.locator('graphshell-tree').getAttribute('data-ready'),'true');assert.equal(errors.length,0);fs.writeFileSync(path.join(out,'unknown-profile-refusal.json'),JSON.stringify({title:await page.title(),errors,handled:true},null,2));runs.push('unknown-profile-refusal');
  await mount('/viewer.html?dataset=fixtures/site-repository-host-history.json');assert.equal(await page.evaluate(()=>typeof window.graphshellScenario),'undefined');await page.getByRole('button',{name:'Choose projects',exact:true}).press('Enter');await page.getByRole('button',{name:'Select Mere',exact:true}).press('Enter');assert.equal(await page.getByRole('link',{name:'Project page',exact:true}).count(),0);runs.push('historical-undisclosed-links-omitted');
  await mount('/viewer.html');assert.equal(await page.evaluate(()=>typeof window.graphshellScenario),'undefined');
  await page.getByRole('button',{name:'Choose projects',exact:true}).press('Enter');
  await page.getByRole('button',{name:'Select Mere',exact:true}).press('Enter');
  const link=page.getByRole('link',{name:'Project page',exact:true});assert.equal(await link.count(),1);
  await page.screenshot({path:path.join(out,'plain-site-inspector.png')});
  const adapter=await page.evaluate(async()=>{const a=await navigator.gpu.requestAdapter();return a?{vendor:a.info.vendor,architecture:a.info.architecture,device:a.info.device,description:a.info.description}:null});
  assert.ok(adapter&&!/swiftshader/i.test(JSON.stringify(adapter)));assert.equal(errors.length,0);
  // Verify real navigation without fetching the external page in this test profile.
  await page.route('https://merelyllc.com/projects/mere/',route=>route.fulfill({contentType:'text/html',body:'<title>Project navigation receipt</title><p>Mere project navigation target</p>'}));
  await link.press('Enter');await page.waitForURL('https://merelyllc.com/projects/mere/');runs.push('project-link-activation');
  fs.writeFileSync(path.join(out,'summary.json'),JSON.stringify({runs,headed:true,browser:browser.version(),adapter,errors},null,2));
  await context.close();console.log('DONE',runs.length);
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1});
