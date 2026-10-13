const fs=require('fs'),path=require('path'),assert=require('node:assert/strict');
const {chromium}=require('/Users/markik/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
const out=path.join(__dirname,'product-browser-'+(process.env.RECEIPT_ROUND||'r1'));fs.mkdirSync(out,{recursive:true});
const base='http://127.0.0.1:8846';
(async()=>{
 require('./assert-console-unlocked.cjs')();
 const browser=await chromium.launch({channel:'chrome',headless:false,args:['--enable-unsafe-webgpu']});
 const runs=[];
 try{
  for(const files of [['p4_tree_forme.scn','p4_tree_forme_reopen.scn']]){
   const ladder=files[0].includes('node_ladder');
   const context=await browser.newContext({viewport:ladder?{width:860,height:720}:{width:1440,height:1000},deviceScaleFactor:1,reducedMotion:'no-preference'});
   await context.addInitScript(()=>{Error.stackTraceLimit=100;console.log('diagnostic document start',Date.now(),location.href)});const page=await context.newPage(),errors=[];const messages=[];page.on('console',m=>{messages.push({at:Date.now(),url:page.url(),location:m.location(),type:m.type(),text:m.text()});fs.writeFileSync(path.join(out,'console-live.json'),JSON.stringify({group:files,messages},null,2))});page.on('pageerror',e=>{errors.push(e.stack||e.message);fs.writeFileSync(path.join(out,'page-errors-live.json'),JSON.stringify({group:files,errors},null,2))});
   for(const file of files){
    await page.goto(base+'/tree.html?app=local&motion=normal'+(files[0].includes('field_inspection')||files[0].includes('presentation_live')||files[0].includes('coupling_strength')?'&fixture=field-inspection':''));await page.bringToFront();
    await page.waitForFunction(()=>document.querySelector('graphshell-tree')?.dataset.ready==='true',null,{timeout:120000});
    async function scenario(text){await page.evaluate(async text=>{const m=await import('./pkg/graphshell_web.js');m.run_scenario(text)},text);await page.waitForFunction(()=>['ok','fail'].includes(document.body.dataset.scenario),null,{timeout:120000});const r=await page.evaluate(()=>window.graphshellScenario());fs.writeFileSync(path.join(out,file+'.latest.json'),JSON.stringify({receipt:r,errors},null,2));await page.screenshot({path:path.join(out,file+'.latest.png')});assert.equal(r.result?.result,'ok',JSON.stringify(r));assert.equal(r.gate_failures.length,0);return r;}
    let receipt;
    if(file==='shortcuts'){
      await scenario('click-node https://example.test/i2p-port\nkey Enter\nsettle 2\nremember title-before-shortcut\n');
      await page.getByRole('textbox',{name:'Title',exact:true}).focus();await page.keyboard.press('Control+k');
      const search=page.getByRole('textbox',{name:'Search graph commands',exact:true});await search.waitFor();await search.press('Escape');
      receipt=await scenario('same title-before-shortcut detail-title\nassert snap commands-open == false\nassert snap detail-open == true\n');
    }else receipt=await scenario(fs.readFileSync(path.join(__dirname,'web-product/scenarios',file),'utf8'));
    fs.writeFileSync(path.join(out,file+'.after-capture.json'),JSON.stringify({receipt,errors},null,2));
    assert.equal(errors.length,0,errors.join('\n'));assert.equal(await page.evaluate(()=>document.hidden),false);
    fs.writeFileSync(path.join(out,file+'.json'),JSON.stringify({receipt,errors},null,2));await page.screenshot({path:path.join(out,file+'.png')});runs.push(file);console.log('PASS',file);
   }
   await context.close();
  }
  fs.writeFileSync(path.join(out,'summary.json'),JSON.stringify({runs,browser:browser.version(),headed:true},null,2));
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1});
