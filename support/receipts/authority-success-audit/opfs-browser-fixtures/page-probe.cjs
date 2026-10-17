const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict');
const source=fs.readFileSync(process.argv[2],'utf8');
const elements=new Map();const element=()=>({value:'1',checked:false,disabled:false,dataset:{},textContent:'',addEventListener(){},replaceChildren(){},append(){}});
const document={body:{dataset:{}},visibilityState:'visible',getElementById(id){if(!elements.has(id))elements.set(id,element());return elements.get(id)},createElement:element,querySelectorAll:()=>[]};
const storage=new Map();const sessionStorage={getItem:k=>storage.get(k)??null,setItem:(k,v)=>storage.set(k,v),removeItem:k=>storage.delete(k)};
const scope={document,window:{addEventListener(){}},navigator:{storage:{estimate:async()=>({quota:100,usage:1}),persisted:async()=>false}},sessionStorage,
 location:{href:'http://audit.invalid/probe/',search:''},URL,URLSearchParams,structuredClone,console,
 setTimeout:(fn,ms)=>{const timer=setTimeout(fn,ms);timer.unref();return timer},clearTimeout,
 Worker:class{constructor(){throw Error('AUDIT forbids actual worker execution')}}};
const context=vm.createContext(scope);vm.runInContext(source,context,{filename:'exact-copied-probe.js'});
(async()=>{
 const states=[];
 for(const condition of ['true','false','reject']){
   scope.navigator.storage.persisted=async()=>{if(condition==='reject')throw Error('AUDIT persistence query refused');return condition==='true'};
   const result=await vm.runInContext('storageSnapshot()',context);
   assert.equal(result.state,condition==='false'?'best_effort':'persistent');states.push({condition,result});
 }
 scope.navigator.storage.persisted=async()=>false;
 const runs=[];
 for(const mode of ['success','lane-false','lane-throws']){
   scope.auditMode=mode;
   vm.runInContext(`startReceipt=async()=>{receipt={schema:'AUDIT inert receipt',configuration:{},environment:{},storage:{},lanes:{}}};
    for(const n of Object.keys(LANES))LANES[n]=async()=>{if(n==='1'&&auditMode==='lane-throws')throw Error('AUDIT injected lane failure');return {ok:n!=='1'||auditMode!=='lane-false'}}`,context);
   let rejected=false;try{await vm.runInContext('runAll()',context)}catch{rejected=true}
   const result=vm.runInContext('window.munimentOpfsProbe.receipt()',context);
   assert.equal(rejected,mode==='lane-throws');
   assert.equal(document.body.dataset.probeState,mode==='lane-throws'?'failed':'complete');
   if(mode==='lane-false'){assert.equal(result.lanes.lane1.ok,false);assert.equal(result.conclusions.stop_condition_hit,false);assert.equal(result.conclusions.receipt_gate_passed,true)}
   runs.push({mode,rejected,state:document.body.dataset.probeState,lane1:result.lanes.lane1,conclusions:result.conclusions});
 }
 console.log(JSON.stringify({storage_states:states,run_all:runs}));
})().catch(e=>{console.error(e);process.exitCode=1});
