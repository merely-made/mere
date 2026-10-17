import assert from 'node:assert/strict';
import vm from 'node:vm';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {fileURLToPath} from 'node:url';
const here=path.dirname(fileURLToPath(import.meta.url));
const root=path.resolve(here,'../../..');
const receipt=path.join(here,'capture_queue_probe_receipt.json');
assert(!fs.existsSync(receipt));
const modelPath='ports/graphshell/web/extension/capture-model.js';
const backgroundPath='ports/graphshell/web/extension/background.js';
const modelSource=fs.readFileSync(path.join(root,modelPath),'utf8');
const backgroundSource=fs.readFileSync(path.join(root,backgroundPath),'utf8');
const sha=s=>crypto.createHash('sha256').update(s).digest('hex');
const POLICY='graphshell.capture.policy.v1',QUEUE='graphshell.capture.queue.v1';
const clone=v=>JSON.parse(JSON.stringify(v));
const event=()=>({addListener(){}});
function harness(){
 const state={}; let blocker=null;
 const context=vm.createContext({URL,console:{error(){}},crypto:{randomUUID:()=> 'synthetic-scope'},browser:{
  runtime:{getURL:()=> 'moz-extension://synthetic/',onMessage:event()},
  action:{onClicked:event()},tabs:{create(){}},
  permissions:{contains:async()=>true,onAdded:event(),onRemoved:event()},
  history:{onVisited:event(),getVisits:async()=>[],search:async()=>[]},
  storage:{local:{
   async get(keys){return Object.fromEntries(keys.filter(k=>k in state).map(k=>[k,clone(state[k])]))},
   async set(update){
    const snapshot=clone(update);
    if(blocker?.predicate(snapshot)){
      const gate=blocker;blocker=null;gate.entered();await gate.releasePromise;
    }
    Object.assign(state,snapshot);
   },
  }},
 }});
 vm.runInContext(modelSource,context,{filename:modelPath});
 vm.runInContext(backgroundSource,context,{filename:backgroundPath});
 const model=context.GraphshellCaptureModel;
 state[POLICY]={...clone(model.defaultPolicy()),enabled:true};state[QUEUE]=[];
 state['graphshell.capture.live-from.v1']=1000;
 return {state,model,
  queue:visits=>vm.runInContext('queueVisits',context)(visits),
  message:message=>vm.runInContext('handleMessage',context)(message),
  block(predicate){
   let entered,release;const reached=new Promise(r=>entered=r),releasePromise=new Promise(r=>release=r);
   blocker={predicate,entered,releasePromise};return{reached,release};
  },
 };
}
function visit(id,url='https://queue-a.test/'){
 return{source:'firefox',source_scope:{product:'firefox',profile:'synthetic',generation:'g'},
 visit_id:id,parent:{kind:'root'},url,title:'Synthetic',transition:'link',at_ms:1000,
 original_unix_micros:1000001,source_is_local:true,private:false};
}
let checks=0;const results=[];
async function check(name,work){await work();checks++;results.push({name,passed:true});}
await check('Exact queue duplicate is idempotent',async()=>{
 const h=harness();assert.equal(h.model.mergeQueue([visit('a')],[visit('a')]).length,1);
});
await check('Changed URL conflict is rejected',async()=>{
 const h=harness();assert.throws(()=>h.model.mergeQueue([visit('a')],[visit('a','https://different.test/')]),/Conflicting/);
});
await check('Changed exact source time and locality are silently discarded',async()=>{
 const h=harness();
 for(const change of[{original_unix_micros:1000002},{source_is_local:false}]){
  const prior=visit('a'),later={...prior,...change};const merged=h.model.mergeQueue([prior],[later]);
  assert.equal(merged.length,1);assert.deepEqual(clone(merged[0]),prior);
 }
});
await check('A stale configure snapshot overwrites a newly queued visit',async()=>{
 const h=harness();const gate=h.block(update=>POLICY in update);
 const configure=h.message({type:'capture.configure',policy:{...h.state[POLICY],segment_size:2}});
 await gate.reached;
 assert.equal(await h.queue([visit('a')]),1);assert.equal(h.state[QUEUE].length,1);
 gate.release();await configure;
 assert.equal(h.state[POLICY].segment_size,2);assert.equal(h.state[QUEUE].length,0);
});
await check('A pending queue write restores a URL excluded by completed configuration',async()=>{
 const h=harness();const gate=h.block(update=>QUEUE in update && !(POLICY in update));
 const queue=h.queue([visit('a')]);await gate.reached;
 await h.message({type:'capture.configure',policy:{...h.state[POLICY],excluded_origins:['https://queue-a.test']}});
 assert.equal(h.state[QUEUE].length,0);
 gate.release();await queue;
 assert.deepEqual(h.state[POLICY].excluded_origins,['https://queue-a.test']);
 assert.equal(h.state[QUEUE][0].url,'https://queue-a.test/');
});
await check('Sequential configuration preserves eligible visits and removes excluded visits',async()=>{
 const h=harness();await h.queue([visit('a'),visit('b','https://queue-b.test/')]);
 await h.message({type:'capture.configure',policy:{...h.state[POLICY],excluded_origins:['https://queue-a.test']}});
 assert.equal(h.state[QUEUE].length,1);assert.equal(h.state[QUEUE][0].url,'https://queue-b.test/');
});
const output={date_utc:new Date().toISOString(),node:process.version,exit:0,checks,results,
 source_sha256:{[modelPath]:sha(fs.readFileSync(path.join(root,modelPath))),[backgroundPath]:sha(fs.readFileSync(path.join(root,backgroundPath)))},
 scope:'Complete actual capture-model.js and background.js executed in native Node VM with controlled in-memory extension storage and explicit awaited-write interleavings. Reproduces queue/configuration lost update and stale exclusion write, plus queue source-fact comparison omissions. Idempotence, visible conflict refusal and sequential configuration controls pass. No actual browser, extension installation, permission prompt, profile, user data, network or native host was exercised. Exact source-time/locality rejection at the Rust ledger remains separately qualified.'};
fs.writeFileSync(receipt,JSON.stringify(output,null,2)+'\n');console.log(JSON.stringify({receipt:path.basename(receipt),checks,exit:0}));
