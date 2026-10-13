// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
const fs=require('fs'),path=require('path'),assert=require('node:assert/strict');
const {chromium}=require('/Users/markik/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const root=__dirname,out=path.join(root,'keyboard-diagnostic');fs.mkdirSync(out,{recursive:true});
(async()=>{
 require('./assert-console-unlocked.cjs')();
 const browser=await chromium.launch({channel:'chrome',headless:false,args:['--enable-unsafe-webgpu']});const rows=[];
 try{
  for(const folder of ['web-baseline','web-final']){
   const context=await browser.newContext({viewport:{width:1440,height:1000},deviceScaleFactor:1});const page=await context.newPage();const errors=[];page.on('pageerror',e=>errors.push(e.message));
   await page.goto('http://127.0.0.1:8847/'+folder+'/tree.html');await page.bringToFront();await page.waitForFunction(()=>document.querySelector('graphshell-tree')?.dataset.ready==='true',null,{timeout:90000});
   await page.evaluate(()=>{
    window.__focusTimeline=[];const describe=e=>e?{tag:e.tagName,role:e.getAttribute('role'),label:e.getAttribute('aria-label'),text:e.textContent?.slice(0,150),class:e.getAttribute('class'),id:e.id,attrs:Object.fromEntries([...e.attributes].map(a=>[a.name,a.value]))}:null;window.__describeFocus=describe;
    document.addEventListener('focusin',e=>window.__focusTimeline.push({time:performance.now(),node:describe(e.target)}),true);
   });
   const script=fs.readFileSync(path.join(root,'web-final/scenarios/p4_tree_physics_keys.scn'),'utf8');
   await page.evaluate(async s=>{const m=await import('./pkg/graphshell_web.js');m.run_scenario(s)},script);
   await page.waitForFunction(()=>['ok','fail'].includes(document.body.dataset.scenario),null,{timeout:60000});
   const r=await page.evaluate(()=>({scenario:window.graphshellScenario(),host:window.graphshellReceipt(),active:window.__describeFocus(document.activeElement),timeline:window.__focusTimeline,tabOrder:[...document.querySelectorAll('button,input,select,[tabindex]')].filter(e=>e.tabIndex>=0).map(e=>window.__describeFocus(e)),hidden:document.hidden}));
   r.errors=errors;fs.writeFileSync(path.join(out,folder+'.json'),JSON.stringify(r,null,2));await page.screenshot({path:path.join(out,folder+'.png')});assert.equal(errors.length,0);assert.equal(r.scenario.gate_failures.length,0);assert.equal(r.hidden,false);
   const adapter=await page.evaluate(async()=>{const a=await navigator.gpu.requestAdapter();return {vendor:a.info.vendor,architecture:a.info.architecture}});assert.equal(adapter.vendor,'amd');rows.push({folder,result:r.scenario.result.result,active:r.active,adapter,errors});console.log('RECORDED',folder,r.scenario.result.result,JSON.stringify(r.active));await context.close();
  }
  fs.writeFileSync(path.join(out,'summary.json'),JSON.stringify({diagnosisOnly:true,originalAssertionsUnchanged:true,rows},null,2));
 }finally{await browser.close()}
})().catch(e=>{console.error(e);process.exitCode=1});
