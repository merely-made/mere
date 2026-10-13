// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
const fs=require('fs'), path=require('path'), assert=require('node:assert/strict');
const {chromium}=require('/Users/markik/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const out=path.join(__dirname,'commands-browser-'+(process.env.RECEIPT_ROUND||'r1')); fs.mkdirSync(out,{recursive:true});
const base='http://127.0.0.1:8847/web-combined';
(async()=>{
 require('./assert-console-unlocked.cjs')();
 const browser=await chromium.launch({channel:'chrome',headless:false,args:['--enable-unsafe-webgpu']});
 const context=await browser.newContext({viewport:{width:1440,height:1000},deviceScaleFactor:1});
 const page=await context.newPage();page.setDefaultTimeout(30000);
 const errors=[],runs=[]; page.on('pageerror',e=>errors.push(e.message));
 async function mount(url){await page.goto(base+url);await page.bringToFront();await page.waitForFunction(()=>document.querySelector('graphshell-tree')?.dataset.ready==='true'||document.title.includes('FAIL'));await page.waitForTimeout(300);}
 async function start(text){await page.evaluate(async text=>{const m=await import('./pkg/graphshell_web.js');m.run_scenario(text)},text);}
 async function finish(name, expected='ok'){
  await page.waitForFunction(()=>['ok','fail'].includes(document.body.dataset.scenario),null,{timeout:120000});
  const r=await page.evaluate(()=>({scenario:window.graphshellScenario(),host:window.graphshellReceipt(),hidden:document.hidden}));
  fs.writeFileSync(path.join(out,name+'.json'),JSON.stringify({...r,errors:[...errors]},null,2));
  await page.screenshot({path:path.join(out,name+'.png')});
  assert.equal(r.scenario.result?.result,expected,JSON.stringify(r.scenario));assert.equal(r.hidden,false);assert.equal(errors.length,0,errors.join('\n'));assert.equal(r.scenario.gate_failures.length,0);
  runs.push(name);console.log('PASS',name);return r;
 }
 async function scenario(name,text){await start(text);return finish(name);}
 const search=()=>page.getByRole('textbox',{name:'Search graph commands',exact:true});
 const trigger=()=>page.getByRole('button',{name:'Graph commands',exact:true});
 async function palette(){await trigger().press('Enter');await search().waitFor();}
 async function command(query){await palette();await search().pressSequentially(query);await search().press('Enter');await page.waitForTimeout(150);}
 try{
  for(const [name,url,file] of (process.env.ONLY_PHYSICAL?[]:[
   ['commands-fixture','/tree.html','viewer_commands.scn'],
   ['commands-grouped','/grouped-dataset.html','viewer_commands.scn'],
   ['fold-page-gestures','/grouped-dataset.html?wheel=page','hosted_fold_gestures.scn'],
   ['fold-history','/tree.html?dataset=fixtures/grouped-host-history.json&grouping=contains','hosted_folds_history.scn'],
  ])){await mount(url);await scenario(name,fs.readFileSync(path.join(__dirname,'web-combined/scenarios',file),'utf8'));}
  await mount('/grouped-dataset.html');
  // External host-page focus is not stolen when a retained popup disappears.
  await palette();
  await page.evaluate(()=>{const b=document.createElement('button');b.id='external-focus-control';b.textContent='Host page focus';document.body.prepend(b);b.focus()});
  await scenario('external-page-focus','key Escape\nsettle 2\nassert snap commands-open == false\n');
  assert.equal(await page.evaluate(()=>document.activeElement?.id),'external-focus-control');
  await page.locator('#external-focus-control').evaluate(e=>e.remove());
  // Physical keyboard: opening, search, popup footer buttons, dismissal/focus.
  await trigger().focus();await page.keyboard.press('Control+k');await search().waitFor();
  assert.equal(await search().evaluate(e=>e===document.activeElement),true);
  await search().pressSequentially('Zoom in');await search().press('Enter');
  await scenario('keyboard-zoom','assert snap command-last == Zoom in\nassert snap focus-kind == graph\n');
  await page.evaluate(()=>{window.__commandKeys=[];window.addEventListener('keydown',e=>window.__commandKeys.push({key:e.key,code:e.code,shift:e.shiftKey,ctrl:e.ctrlKey,meta:e.metaKey,target:e.target.outerHTML?.slice(0,500)}),true)});await page.keyboard.press('Shift+F10');await page.waitForTimeout(250);fs.writeFileSync(path.join(out,'shift-f10-debug.json'),JSON.stringify(await page.evaluate(()=>({events:window.__commandKeys,active:document.activeElement?.outerHTML?.slice(0,1000)})),null,2));await search().waitFor();
  await page.getByRole('button',{name:'All commands',exact:true}).press('Enter');
  await page.waitForTimeout(350);
  fs.writeFileSync(path.join(out,'all-commands-debug.json'),JSON.stringify(await page.evaluate(()=>({active:document.activeElement?.outerHTML,options:[...document.querySelectorAll('[role=option]')].map(e=>({name:e.getAttribute('aria-label'),text:e.textContent})),dialogs:[...document.querySelectorAll('[role=dialog]')].map(e=>e.outerHTML),host:window.graphshellReceipt?.()})),null,2));
  await page.getByRole('option',{name:'Wheel: scroll page',exact:true}).waitFor({state:'attached'});
  await page.getByRole('button',{name:'Close commands',exact:true}).press('Enter');
  await scenario('keyboard-footer','assert snap commands-open == false\nassert snap focus-kind == graph\n');
  // Physical right click opens; the captured right drag remains box selection.
  const graph=await page.getByRole('group',{name:/^\d+ of \d+ shown$/}).boundingBox();assert.ok(graph&&graph.width>300);
  const at={x:graph.x+graph.width-36,y:graph.y+graph.height-36};
  await page.mouse.click(at.x,at.y,{button:'right'});await search().waitFor();
  await page.screenshot({path:path.join(out,'right-click-menu.png')});
  await search().press('Escape');
  await trigger().focus();await page.mouse.move(at.x-160,at.y-160);await page.mouse.down({button:'right'});await page.mouse.move(at.x-60,at.y-60,{steps:5});await page.mouse.up({button:'right'});
  await scenario('right-drag','assert snap commands-open == false\nassert snap dragging == false\nassert snap focus-kind == graph\n');
  // Physical background drag is a camera action, without changing geometry.
  await start('remember physical-pan\nsettle 180\ndiffers physical-pan camera\nsame physical-pan geometry\n');
  await page.waitForTimeout(200);await page.mouse.move(at.x-5,at.y-5);await page.mouse.down();await page.mouse.move(at.x-100,at.y-60,{steps:8});await page.mouse.up();await finish('physical-background-pan');
  // Context Fold commands use the existing hierarchy/projection owner.
  await command('Expand mere');await scenario('command-fold-expand','assert snap commands-open == false\nassert snap command-last == Expand mere\n');
  await command('Enter mere workspace');await scenario('command-fold-enter','assert snap nodes == 9\n');
  await scenario('prepare-pointer','set-zoom 1\ncenter-node urn:host-dataset:crate:mere:sceno\nclick-node urn:host-dataset:crate:mere:sceno\nsettle 3\n');
  const leaf=await page.getByRole('group',{name:/^\d+ of \d+ shown$/}).boundingBox();assert.ok(leaf);
  await page.mouse.click(leaf.x+leaf.width-20,leaf.y+leaf.height-20);await page.waitForTimeout(150);
  await scenario('empty-selection','assert snap selection-fit-available == false\n');
  await page.mouse.move(leaf.x+leaf.width/2-24,leaf.y+leaf.height/2-24);await page.mouse.down({button:'right'});await page.mouse.move(leaf.x+leaf.width/2+24,leaf.y+leaf.height/2+24,{steps:5});await page.mouse.up({button:'right'});
  await scenario('right-box-selection','assert snap selection-fit-available == true\nassert snap commands-open == false\nassert snap focus-kind == graph\n');
  await scenario('prepare-selected-zoom','click-node urn:host-dataset:crate:mere:sceno\nsettle 3\n');
  await start('remember physical-zoom\nsettle 180\ndiffers physical-zoom zoom\nsame physical-zoom geometry picked\nassert snap selection-center-distance <= 2\n');
  await page.waitForTimeout(200);await page.mouse.move(leaf.x+leaf.width/2,leaf.y+leaf.height/2);await page.mouse.wheel(0,-180);await finish('physical-pointer-zoom');
  await scenario('prepare-node-drag','set-zoom 1\ncenter-node urn:host-dataset:crate:mere:sceno\nsettle 3\n');
  await start('remember physical-node\nsettle 180\ndiffers physical-node geometry\nsame physical-node camera zoom\nassert snap dragging == false\n');
  await page.waitForTimeout(200);await page.mouse.move(leaf.x+leaf.width/2,leaf.y+leaf.height/2);await page.mouse.down();await page.mouse.move(leaf.x+leaf.width/2+70,leaf.y+leaf.height/2+15,{steps:8});await page.mouse.up();await finish('physical-node-drag');
  await command('Whole graph');await scenario('command-fold-leave','assert snap commands-open == false\n');
  await palette();await search().pressSequentially('Pan left');await search().press('ArrowDown');
  await page.getByRole('button',{name:'Keep selected command in menu',exact:true}).press('Enter');
  await page.getByRole('button',{name:'Close commands',exact:true}).press('Enter');
  await palette();assert.ok(await page.getByRole('option',{name:'Pan left',exact:true}).count());await search().press('Escape');
  await page.setViewportSize({width:420,height:800});await page.waitForTimeout(400);await palette();
  const dialog=await page.getByRole('dialog',{name:'Graph commands',exact:true}).boundingBox();assert.ok(dialog&&dialog.x>=0&&dialog.x+dialog.width<=421);
  await search().pressSequentially('Wheel: pan graph');await page.screenshot({path:path.join(out,'narrow-menu.png')});await search().press('Enter');
  await scenario('narrow-commands','assert snap wheel-mode == pan\nassert snap commands-open == false\nassert snap focus-kind == graph\n');
  await page.setViewportSize({width:1440,height:1000});
  // A real embedding page: zoom does not scroll it; explicit page preference does.
  await mount('/viewer.html');assert.equal(await page.evaluate(()=>typeof window.graphshellScenario),'undefined');
  let box=await page.getByRole('group',{name:/^\d+ of \d+ shown$/}).boundingBox();assert.ok(box);
  await page.mouse.move(box.x+box.width/2,box.y+box.height/2);let before=await page.evaluate(()=>scrollY);
  await page.mouse.wheel(0,220);await page.waitForTimeout(500);assert.equal(await page.evaluate(()=>scrollY),before);
  await command('Wheel: scroll page');box=await page.getByRole('group',{name:/^\d+ of \d+ shown$/}).boundingBox();
  await page.mouse.move(box.x+box.width/2,box.y+box.height/2);before=await page.evaluate(()=>scrollY);
  await page.mouse.wheel(0,220);await page.waitForTimeout(500);assert.ok(await page.evaluate(()=>scrollY)>before);
  await page.screenshot({path:path.join(out,'plain-viewer.png')});runs.push('plain-physical-wheel');
  await page.evaluate(()=>scrollTo(0,0));await page.waitForTimeout(100);
  const heading=await page.getByRole('heading',{name:'Project graph',exact:true}).boundingBox();assert.ok(heading);
  await page.mouse.move(heading.x+25,heading.y+10);await page.mouse.wheel(0,160);await page.waitForTimeout(500);assert.ok(await page.evaluate(()=>scrollY)>0);runs.push('outside-graph-wheel');
  await palette();await search().pressSequentially('Fit selection');await search().press('ArrowDown');
  const cdp=await context.newCDPSession(page);const ax=await cdp.send('Accessibility.getFullAXTree');
  assert.ok(ax.nodes.some(n=>n.role?.value==='textbox'&&n.name?.value==='Search graph commands'));
  assert.ok(ax.nodes.some(n=>n.role?.value==='option'&&n.name?.value==='Fit selection'&&n.description?.value==='Select a visible node first'&&n.properties?.some(p=>p.name==='disabled'&&p.value?.value===true)));
  await search().press('Escape');
  fs.writeFileSync(path.join(out,'chrome-accessibility.json'),JSON.stringify(ax,null,2));
  // Refuse an unknown wheel preference before mounting a misleading fallback.
  await page.goto(base+'/tree.html?wheel=unknown');await page.waitForFunction(()=>document.title.includes('FAIL')&&document.title.toLowerCase().includes('wheel'));
  const refusal=await page.title();assert.match(refusal,/wheel/i);assert.notEqual(await page.locator('graphshell-tree').getAttribute('data-ready'),'true');fs.writeFileSync(path.join(out,'unknown-wheel-refusal.json'),JSON.stringify({title:refusal,handled:true,errors},null,2));runs.push('unknown-wheel-refusal');
  await mount('/grouped-dataset.html');
  await page.evaluate(()=>{const original=Event.prototype.preventDefault;Event.prototype.preventDefault=function(){if(this.type==='wheel')return;return original.call(this)}});
  await start('wheel-graph -90\nsettle 3\nassert snap gesture == wheel kept false\n');
  const control=await finish('wheel-planted-control','fail');
  const failures=control.scenario.result.log.filter(line=>line.startsWith('FAIL:'));assert.equal(failures.length,1);assert.match(failures[0],/wheel kept true/);
  const adapter=await page.evaluate(async()=>{const a=await navigator.gpu.requestAdapter();return a?{vendor:a.info.vendor,architecture:a.info.architecture,device:a.info.device,description:a.info.description}:null});
  assert.ok(adapter&&!/swiftshader/i.test(JSON.stringify(adapter)));assert.equal(errors.length,0,errors.join('\n'));
  fs.writeFileSync(path.join(out,'summary.json'),JSON.stringify({runs,headed:true,browser:browser.version(),adapter,errors},null,2));console.log('DONE',JSON.stringify({runs,adapter}));
 }catch(e){fs.writeFileSync(path.join(out,'failure.json'),JSON.stringify({message:e.message,errors,active:await page.evaluate(()=>document.activeElement?.outerHTML)},null,2));await page.screenshot({path:path.join(out,'failure.png')});throw e;}finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1});
