// Actual page orchestration evaluated in a native Node VM with controlled I/O.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';
const here=path.dirname(fileURLToPath(import.meta.url));
const root=path.resolve(here,'../../..');
const sourcePath=path.join(root,'ports/distillery/probe/web/probe.js');
const raw=fs.readFileSync(sourcePath), source=raw.toString('utf8');
const sha=data=>crypto.createHash('sha256').update(data).digest('hex');
const receiptPath=path.join(here,'browser_probe_flow_receipt.json');
assert(!fs.existsSync(receiptPath));
const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
const expectedWeightBytes=90_000_000;

class Element {
  constructor(){this.handlers=new Map();this.value='';this.disabled=false;this.checked=false;this.children=[];this.dataset={};}
  addEventListener(name,handler){this.handlers.set(name,handler);}
  append(child){this.children.push(child);}
  replaceChildren(){this.children=[];}
}

async function world(kind) {
  const elements=new Map();
  const get=id=>{if(!elements.has(id))elements.set(id,new Element());return elements.get(id);};
  const workers=[];
  const model={model_id:'audit/synthetic',revision:'0'.repeat(40),model_base_url:'/models/synthetic',
    architecture:'bert',license:'synthetic',pooling:'attention-mask-aware mean',expected_dimensions:384,
    artifacts:{weights:{bytes:expectedWeightBytes,dtype:'F32'},config:{bytes:1},tokenizer:{bytes:1}},
    reference:{source:'synthetic worker contract',first_8:Array(8).fill(0),tolerance:0.001}};
  const matrix={schema:'distillery.browser-model-matrix/v1',models:[model],input:'synthetic input',
    run_count:2,idle_sample_ms:0,frame_bound_ms:33.4,diagnostic_trace:false};
  class Worker {
    constructor(_url,options){this.name=options.name;this.handlers=new Map();this.terminated=false;workers.push(this);}
    addEventListener(name,handler){this.handlers.set(name,handler);}
    terminate(){this.terminated=true;}
    emit(name,data){return this.handlers.get(name)?.({data,...(name==='error'?data:{})});}
    postMessage(message){
      if(message.command!=='run')throw new Error('Unexpected synthetic worker command');
      this.config=message.config;
      if(kind==='silent')return;
      queueMicrotask(()=>{
        if(this.terminated)return;
        if(this.name.endsWith(':cancel')) {
          this.emit('message',{kind:'state',state:'executing',detail:'synthetic cancellation boundary'});
          return;
        }
        const unstable=kind==='unstable';
        const report={schema:'distillery.browser-model-worker/v2',mode:message.config.mode,
          model_id:message.config.model_id,manifest_id:'blake3:synthetic-manifest',
          component_hashes:{config:'config',tokenizer:'tokenizer',weights:'weights'},
          component_sizes:{fetched_config:1,resolved_config:1,tokenizer:1,
            weights:kind==='wrong-size'?1:expectedWeightBytes},
          integrity_matches:kind!=='bad-integrity',
          execution:{kind:'sentence_embedding',dimensions:384,expected_dimensions:384,
            dimensions_match:true,all_finite:true,l2_norm:1,first_8:Array(8).fill(0),
            reference_within_tolerance:true,reference_max_abs_error:0,
            output_hash:unstable?message.config.mode:'same-output',
            repeat_outputs_match:!unstable,matches_prior_worker:message.config.mode==='warm'?!unstable:null}};
        this.emit('message',{kind:'result',report});
      });
    }
  }
  const document={getElementById:get,createElement:()=>new Element(),body:new Element()};
  const window={addEventListener(){}};
  const sandbox={document,window,Worker,navigator:{userAgent:'synthetic-node-vm',platform:'synthetic'},
    requestAnimationFrame:()=>0,crossOriginIsolated:false,performance:{now:()=>performance.now()},
    location:{search:''},URLSearchParams,structuredClone,setTimeout,clearTimeout,
    fetch:async(url,options)=>({ok:true,status:200,url,headers:{get:()=>null},json:async()=>
      String(url).includes('decoder-model')?{schema:'distillery.browser-decoder-row/v1',model:{}}:structuredClone(matrix)})};
  vm.runInNewContext(source,sandbox,{filename:sourcePath});
  await window.distilleryModelProbe.ready;
  await sleep(0);
  return {api:window.distilleryModelProbe,get,workers,document};
}

const rows=[];
{
  const w=await world('silent');let settled=false;
  const submit=w.get('probe-controls').handlers.get('submit')({preventDefault(){}});
  submit.then(()=>{settled=true;});
  const until=Date.now()+2000;
  while(w.workers.length===0){assert(Date.now()<until);await sleep(1);}
  assert.equal(w.api.terminateActive(),true);
  await sleep(30);
  assert.equal(w.workers[0].terminated,true);
  assert.equal(w.document.body.dataset.probeState,'terminated');
  assert.equal(settled,false);
  assert.equal(w.get('run-suite').disabled,true);
  rows.push({name:'manual_termination_leaves_ui_run_unsettled',observation:'terminateActive returned true and terminated the fake worker; UI submission remains pending and run controls remain disabled after the observation window',window_ms:30});
}
{
  const w=await world('silent');
  const submit=w.get('probe-controls').handlers.get('submit')({preventDefault(){}});
  const until=Date.now()+2000;
  while(w.workers.length===0){assert(Date.now()<until);await sleep(1);}
  w.workers[0].emit('error',{message:'synthetic worker failure'});
  await submit;
  assert.equal(w.get('run-suite').disabled,false);
  assert.equal(w.api.receipt().conclusions.row_passed,false);
  rows.push({name:'worker_error_settles_and_reenables_control',control:true});
}
{
  const w=await world('unstable');const row=await w.api.runSuite();
  assert.equal(row.cold.execution.repeat_outputs_match,false);
  assert.equal(row.warm.execution.repeat_outputs_match,false);
  assert.equal(row.conclusions.output_repeated_across_workers,false);
  assert.equal(row.conclusions.row_passed,true);
  rows.push({name:'embedding_row_passes_reported_repeatability_failures',row_passed:true,
    cold_repeats:false,warm_repeats:false,output_repeated_across_workers:false});
}
{
  const w=await world('bad-integrity');const row=await w.api.runSuite();
  assert.equal(row.conclusions.row_passed,false);
  rows.push({name:'integrity_failure_refuses_row',control:true});
}
{
  const w=await world('wrong-size');const result=await w.api.runMatrix();
  assert.equal(result.rows[0].cold.component_sizes.weights,1);
  assert.equal(result.rows[0].warm.component_sizes.weights,1);
  assert.equal(result.conclusions.all_configured_rows_passed,true);
  assert.equal(result.conclusions.largest_successful_weight_bytes,expectedWeightBytes);
  rows.push({name:'matrix_reports_expected_weight_size_despite_observed_mismatch',
    observed_weights_bytes:1,claimed_largest_successful_weight_bytes:expectedWeightBytes,
    all_configured_rows_passed:true});
}
{
  const w=await world('ordinary');const row=await w.api.runSuite();
  assert.equal(row.conclusions.row_passed,true);
  assert.equal(row.conclusions.output_repeated_across_workers,true);
  rows.push({name:'ordinary_consistent_row_passes',control:true});
}
assert.equal(sha(fs.readFileSync(sourcePath)),sha(raw));
const result={date_utc:new Date().toISOString(),source:'ports/distillery/probe/web/probe.js',
  source_sha256:sha(raw),probe_sha256:sha(fs.readFileSync(fileURLToPath(import.meta.url))),
  runtime:process.version,checks:rows.length,exit:0,rows,
  scope:'Complete actual production page JavaScript evaluated in native Node VM. Controlled DOM, fetch and Worker messages isolate orchestration and receipt predicates. No browser, real worker, WebGPU, model execution, network request or user storage. The manual-cancel observation is bounded to30ms plus source proof that no resolver/timeout is called. Wrong-size and unstable outputs are synthetic worker-contract messages, not claims of an actual GPU model failure. Synthetic workers are terminated by the evaluated page; the unresolved Promise owns no real worker or timer.'};
fs.writeFileSync(receiptPath,JSON.stringify(result,null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify({receipt:path.basename(receiptPath),checks:rows.length,exit:0,observations:rows},null,2));
