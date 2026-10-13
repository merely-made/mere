// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
const fs=require('fs'),path=require('path'),assert=require('node:assert/strict');
const {chromium}=require('/Users/markik/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/../node_modules/playwright');
const out=path.join(__dirname,'remaining-browser-'+(process.env.RECEIPT_ROUND||'r1'));fs.mkdirSync(out,{recursive:true});
const base='http://127.0.0.1:8847';
(async()=>{
 require('./assert-console-unlocked.cjs')();
 const browser=await chromium.launch({channel:'chrome',headless:false,args:['--enable-unsafe-webgpu']});
 const context=await browser.newContext({viewport:{width:1440,height:1000},deviceScaleFactor:1});
 const page=await context.newPage();page.setDefaultTimeout(30000);const errors=[],runs=[];page.on('pageerror',e=>{errors.push(e.stack||e.message);fs.writeFileSync(path.join(out,'page-errors-live.json'),JSON.stringify(errors,null,2))});
 async function mount(route,folder='web-remaining'){
  await page.goto(base+'/'+folder+route);await page.bringToFront();await page.waitForFunction(()=>document.querySelector('graphshell-tree')?.dataset.ready==='true',null,{timeout:120000});await page.waitForTimeout(250);
 }
 async function run(name,text,expected='ok'){
  await page.evaluate(async text=>{const m=await import('./pkg/graphshell_web.js');m.run_scenario(text)},text);
  await page.waitForFunction(()=>['ok','fail'].includes(document.body.dataset.scenario),null,{timeout:420000});
  const r=await page.evaluate(()=>({receipt:window.graphshellScenario(),host:window.graphshellReceipt(),hidden:document.hidden}));
  await page.screenshot({path:path.join(out,name+'.png')});fs.writeFileSync(path.join(out,name+'.json'),JSON.stringify({...r,errors:[...errors]},null,2));
  assert.equal(r.receipt.result?.result,expected,JSON.stringify(r.receipt));assert.equal(r.receipt.gate_failures.length,0);assert.equal(r.hidden,false);assert.equal(errors.length,0,errors.join('\n'));runs.push(name);console.log('PASS',name);return r;
 }
 try {
  // All eleven laws, plus roles, drag, keys, profiles and dynamic additions.
  const physics=['keys','add'];
  for(const law of physics){
   await mount('/tree.html');const file='p4_tree_physics_'+law+'.scn';await run(file,fs.readFileSync(path.join(__dirname,'web-remaining/scenarios',file),'utf8'));
  }
  await mount('/tree.html');
  const focusTrace=[];
  async function trace(){focusTrace.push(await page.evaluate(()=>{let e=document.activeElement;while(e?.shadowRoot?.activeElement)e=e.shadowRoot.activeElement;return {tag:e?.tagName,role:e?.getAttribute('role'),name:e?.getAttribute('aria-label'),text:e?.textContent?.slice(0,100),attrs:e?Object.fromEntries([...e.attributes].map(a=>[a.name,a.value])):{}}}));fs.writeFileSync(path.join(out,'physical-tab-trace.json'),JSON.stringify(focusTrace,null,2));}
  const law=page.getByRole('combobox',{name:'Physics law',exact:true});await law.click();await law.press('Escape');await law.press('ArrowDown');await law.press('Enter');
  await run('physical-keyboard-choice','settle 2\nassert snap panel-law == charge.barnes-hut\nassert snap physics-law == spring.rapier\n');
  await page.keyboard.press('Tab');await page.keyboard.press('Space');await trace();
  await run('physical-keyboard-overlay','settle 2\nassert snap panel-overlays == degree-repulsion\n');
  for(let i=0;i<12;i++){await page.keyboard.press('Tab');await page.waitForTimeout(35);await trace();}
  await run('physical-keyboard-apply-focus','assert snap focus-kind == button\n');
  await page.keyboard.press('Enter');
  await run('physical-keyboard-applied','settle 2\nassert snap physics-law == charge.barnes-hut\nassert snap physics-overlays == degree-repulsion\nassert snap physics-profile == custom\nassert text Physics set to Charge with Hub room\n');
  await mount('/tree.html?nodes=21&seed=7&links=none&physics_speed=1&gpu=off');
  await run('site-scale-frames',fs.readFileSync(path.join(__dirname,'site-frames.scn'),'utf8'));
  for(const [name,query,file] of [
   ['saved-route-refused','?app=local','p4_tree_item_role.scn'],
   ['saved-edit-refused','?app=local','p4_tree_saved_edit.scn'],
   ['remote-absent-refused','','p4_tree_remote_absent.scn']]){
   await mount('/tree.html'+query);await run(name,fs.readFileSync(path.join(__dirname,'web-remaining/scenarios',file),'utf8'),'fail');
  }
  const adapter=await page.evaluate(async()=>{const a=await navigator.gpu.requestAdapter();return a?{vendor:a.info.vendor,architecture:a.info.architecture,device:a.info.device,description:a.info.description}:null});
  assert.ok(adapter&&adapter.vendor==='amd'&&!/swiftshader/i.test(JSON.stringify(adapter)));assert.equal(errors.length,0);
  fs.writeFileSync(path.join(out,'summary.json'),JSON.stringify({runs,browser:browser.version(),adapter,errors,headed:true},null,2));console.log('DONE',runs.length);
 }catch(e){fs.writeFileSync(path.join(out,'failure.json'),JSON.stringify({message:e.message,errors,state:await page.evaluate(()=>({scenario:window.graphshellScenario?.(),host:window.graphshellReceipt?.(),hidden:document.hidden}))},null,2));await page.screenshot({path:path.join(out,'failure.png')});throw e;}finally{await browser.close()}
})().catch(e=>{console.error(e);process.exitCode=1});
