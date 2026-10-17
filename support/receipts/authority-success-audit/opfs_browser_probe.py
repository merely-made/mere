"""Execute unchanged OPFS JS in explicit fake-browser boundaries, no live UI."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,shutil,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];NAME='opfs_browser_probe'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
sources=['ports/muniment-opfs-probe/run-browser.mjs','ports/muniment-opfs-probe/web/probe.js']
guards={p:sha(ROOT/p) for p in sources};base=HERE/'opfs-browser-fixtures';assert not base.exists();base.mkdir()
for source,name in zip(sources,['run-browser.mjs','probe.js']):shutil.copyfile(ROOT/source,base/name)
vm=r'''const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict');
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
'''
(base/'page-probe.cjs').write_text(vm,encoding='utf-8')
result=subprocess.run(['node',str(base/'page-probe.cjs'),str(base/'probe.js')],cwd=base,capture_output=True,text=True,timeout=20)
(base/'page-probe.log').write_text(result.stdout+result.stderr,encoding='utf-8');assert result.returncode==0,result.stderr
page=json.loads(result.stdout)
mock=r'''const fs=require('node:fs');const input=JSON.parse(fs.readFileSync(process.env.AUDIT_CASE_FILE,'utf8'));
const receipt=input.receipt;
const page={on(){},async goto(){},async waitForFunction(){},async exposeFunction(){},async evaluate(fn,arg){
 const code=String(fn);
 if(code.includes('const channel = new BroadcastChannel'))return;
 if(code.includes('await window.munimentOpfsProbe.runLane')){if(input.laneThrows)throw Error('AUDIT injected lane exception');return}
 if(code.includes('return { ok: l?.ok')){const l=receipt.lanes[arg];return {ok:l?.ok??null,failed:l?.failed??false,error:l?.error??null}}
 if(code.includes('window.munimentOpfsProbe.receipt()'))return receipt;
 if(code.includes('window.munimentOpfsProbe.exportedBase64()'))return Buffer.from('AUDIT inert non-database bytes').toString('base64');
 throw Error('unexpected fake-page boundary: '+code);
}};
module.exports={firefox:{async launch(){return {async newContext(){return {async newPage(){return page}}},async close(){}}}}};
'''
module=base/'mock-playwright';module.mkdir();(module/'index.js').write_text(mock,encoding='utf-8')
preload=r'''const cp=require('node:child_process'),fs=require('node:fs');
const input=JSON.parse(fs.readFileSync(process.env.AUDIT_CASE_FILE,'utf8'));
cp.spawnSync=(...args)=>{if(!input.native)throw Error('unexpected native invocation');return {status:input.native.exit,stdout:JSON.stringify({ok:input.native.ok}),stderr:'AUDIT injected native verifier result'}};
require('node:module').syncBuiltinESMExports();
'''
(base/'preload.cjs').write_text(preload,encoding='utf-8')
cases=[
 ('success',{'lanes':{'lane1':{'ok':True}},'conclusions':{'receipt_gate_passed':True}},False,None),
 ('lane-false',{'lanes':{'lane1':{'ok':False}},'conclusions':{'receipt_gate_passed':True}},False,None),
 ('lane-throws',{'lanes':{'lane1':{'failed':True,'error':'AUDIT injected lane exception'}},'conclusions':{'receipt_gate_passed':True}},True,None),
 ('gate-false',{'lanes':{'lane1':{'ok':True}},'conclusions':{'receipt_gate_passed':False,'receipt_gate_failures':['AUDIT planted page error']}},False,None),
 ('native-verify-false',{'lanes':{'lane5':{'ok':True,'portability':{'state':'ran','native_to_browser_ok':True,'extended_to_generation':8}}},'conclusions':{'receipt_gate_passed':True,'lane5_native_to_browser_ok':True}},False,{'exit':1,'ok':False}),
 ('native-verify-true',{'lanes':{'lane5':{'ok':True,'portability':{'state':'ran','native_to_browser_ok':True,'extended_to_generation':8}}},'conclusions':{'receipt_gate_passed':True,'lane5_native_to_browser_ok':True}},False,{'exit':0,'ok':True}),
]
results=[]
for name,receipt,throws,native in cases:
    case=base/name;case.mkdir();(case/'fixtures').mkdir()
    injection=case/'injection.json';injection.write_text(json.dumps({'receipt':receipt,'laneThrows':throws,'native':native}),encoding='utf-8')
    env=dict(os.environ,PLAYWRIGHT_MODULE=str(module),AUDIT_CASE_FILE=str(injection))
    command=['node','--require',str(base/'preload.cjs'),str(base/'run-browser.mjs'),'--engine','firefox','--lanes','5' if native else '1','--out','output.json']
    child=subprocess.run(command,cwd=case,env=env,capture_output=True,text=True,timeout=20)
    (case/'outer-output.log').write_text(child.stdout+child.stderr,encoding='utf-8')
    assert child.returncode==0,(name,child.stderr)
    output=json.loads((case/'output.json').read_text())
    row={'name':name,'outer_exit':child.returncode,'per_lane':output['runner']['per_lane'],'conclusions':output['conclusions']}
    if native:
        port=output['lanes']['lane5']['portability'];assert port['round_trip_both_ways_ok']==native['ok']
        assert port['browser_to_native']['exit_code']==native['exit']
        assert output['lanes']['lane5']['ok'] is True
        row.update({'lane5_ok':output['lanes']['lane5']['ok'],'portability':port})
    results.append(row)
assert all(sha(ROOT/p)==h for p,h in guards.items())
record={'date_utc':datetime.now(timezone.utc).isoformat(),'source_sha256':guards,'node':subprocess.check_output(['node','--version'],text=True).strip(),
 'page_cases':page,'runner_cases':results,'fixture_sha256':{p.relative_to(HERE).as_posix():sha(p) for p in base.rglob('*') if p.is_file()},'runner_sha256':sha(Path(__file__)),
 'qualification':'Complete byte-identical page source executed in Node VM with explicit inert DOM/sessionStorage/navigator boundaries; lane implementations injected only at LANES table and startReceipt acquisition boundary. Actual runAll/runLane/conclude/storageSnapshot execute. Six whole unchanged run-browser CLI copies execute in real Node processes, with an explicitly fake Playwright package, page evaluate outputs and native spawnSync result. Real outer exits and JSON/file writes, no actual browser/OPFS/IndexedDB/worker/server/network/Cargo/native database or power-loss acceptance. Fake database bytes clearly inert. Historical receipts and production source untouched.'}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'page_cases':page,'runner_cases':results,'fixture_bytes':sum(p.stat().st_size for p in base.rglob('*') if p.is_file()),'qualification':record['qualification']},indent=2))
