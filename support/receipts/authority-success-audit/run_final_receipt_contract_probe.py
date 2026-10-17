"""Run unchanged receipt wrappers with explicit non-GPU process/OS doubles."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib,json,shutil,subprocess
HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
paths=['support/run_resident_inference_stages.ps1','support/run_resident_layer_streaming.ps1','support/run_resident_weight_formats.ps1','testing/run_sennet_custody.ps1','testing/run_tucket_custody.ps1','support/receipts/instrument_workspace_20261012/verify-browser.cjs']
before={p:sha(ROOT/p) for p in paths}
case=HERE/'final-receipt-contract-cases';case.mkdir(exist_ok=False)
empty=HERE/'final_receipt_empty_tests.rs';empty.write_text('#[test]\n#[ignore]\nfn unrelated_control() {}\n',encoding='utf-8')
fake=HERE/'final_receipt_fake_process.rs';fake.write_text(r'''fn main(){
 let a:Vec<String>=std::env::args().collect();
 if a.len()==6 && a[5]=="pending" {std::fs::write(&a[4],br#"{"result":"failed"}"#).unwrap();return;}
 if let Ok(d)=std::env::var("ESP_STREAMING_WORK_DIR"){std::fs::write(std::path::Path::new(&d).join("burn-wgpu-software.json"),b"{}").unwrap();}
 else if let Ok(d)=std::env::var("ESP_FORMAT_WORK_DIR"){let n=format!("{}-{}.json",std::env::var("ESP_WEIGHT_FORMAT").unwrap(),std::env::var("ESP_WEIGHT_LOADER").unwrap());std::fs::write(std::path::Path::new(&d).join(n),b"{}").unwrap();}
 println!("running 0 tests\n\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out");
}
''',encoding='utf-8')
target=Path('C:/t/cargo-targets/mere');exe1=target/'audit-final-receipt-empty.exe';exe2=target/'audit-final-receipt-fake.exe'
outputs=[exe1,exe1.with_suffix('.pdb'),exe2,exe2.with_suffix('.pdb')]
assert not any(p.exists() for p in outputs)
compiles=[]
for rs,exe,is_test in [(empty,exe1,True),(fake,exe2,False)]:
 cmd=['rustc','--edition=2024',str(rs),'-C','debuginfo=0','-o',str(exe)]+(['--test'] if is_test else [])
 r=subprocess.run(cmd,cwd=ROOT,text=True,capture_output=True,timeout=30);assert r.returncode==0,r.stderr
 compiles.append({'command':cmd,'exit':r.returncode,'stdout':r.stdout,'stderr':r.stderr})
config={'root':str(ROOT),'here':str(HERE),'cases':str(case),'empty':str(exe1),'fake':str(exe2)}
(case/'config.json').write_text(json.dumps(config),encoding='utf-8')
harness=HERE/'final_receipt_contract_harness.ps1'
harness.write_text(r'''param([string]$Config,[string]$Mode)
$ErrorActionPreference='Stop'
$taskCfg=Get-Content -LiteralPath $Config -Raw|ConvertFrom-Json
function nvidia-smi {$global:LASTEXITCODE=0; 'Synthetic adapter,4096,2048'}
function Get-CimInstance {[pscustomobject]@{TotalVisibleMemorySize=1048576;FreePhysicalMemory=524288}}
if($Mode -eq 'stages'){
 & (Join-Path $taskCfg.root 'support/run_resident_inference_stages.ps1') -Executable $taskCfg.empty -ModelDirectory $taskCfg.here -FormatWorkDirectory $taskCfg.here -WorkDirectory (Join-Path $taskCfg.cases 'stages')
}elseif($Mode -eq 'streaming'){
 & (Join-Path $taskCfg.root 'support/run_resident_layer_streaming.ps1') -TestBinary $taskCfg.fake -ModelDirectory $taskCfg.here -WorkDirectory (Join-Path $taskCfg.cases 'streaming')
}elseif($Mode -eq 'formats'){
 & (Join-Path $taskCfg.root 'support/run_resident_weight_formats.ps1') -TestBinary $taskCfg.fake -ModelDirectory $taskCfg.here -WorkDirectory (Join-Path $taskCfg.cases 'formats') -Format f16 -Loader burn-flex
}elseif($Mode -eq 'custody'){
 $output=Join-Path $taskCfg.cases 'custody';New-Item -ItemType Directory -Path $output|Out-Null
 $ProbeBinary=$taskCfg.fake;$taskDevice='00000000-0000-0000-0000-000000000001';$realm='synthetic-only'
 $taskRows=@()
 foreach($scriptName in @('run_sennet_custody.ps1','run_tucket_custody.ps1')){
  $taskTokens=$null;$taskErrors=$null
  $taskAst=[System.Management.Automation.Language.Parser]::ParseFile((Join-Path $taskCfg.root "testing/$scriptName"),[ref]$taskTokens,[ref]$taskErrors)
  if($taskErrors){throw 'Unexpected source parse error'}
  $taskFn=$taskAst.Find({param($taskNode) $taskNode -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $taskNode.Name -eq 'Invoke-Probe'},$true)
  Invoke-Expression $taskFn.Extent.Text
  $taskRow=Invoke-Probe ($scriptName.Replace('.ps1','')) 'synthetic-unused-endpoint' 'pending' $false
  if($taskRow.result -ne 'failed' -or $taskRow.exit_code -ne 0){throw 'Wrong probe control result'}
  $taskRows+=@{script=$scriptName;returned_without_error=$true;returned_result=$taskRow.result;child_exit=$taskRow.exit_code}
 }
 $taskRows|ConvertTo-Json -Depth 6|Set-Content -LiteralPath (Join-Path $taskCfg.cases 'custody-functions.json')
}else{throw 'Unknown mode'}
''',encoding='utf-8')
pwsh=shutil.which('pwsh');assert pwsh
runs=[]
for mode in ['stages','streaming','formats','custody']:
 command=[pwsh,'-NoProfile','-File',str(harness),'-Config',str(case/'config.json'),'-Mode',mode]
 r=subprocess.run(command,cwd=ROOT,text=True,capture_output=True,timeout=30)
 (case/(mode+'.stdout.txt')).write_text(r.stdout,encoding='utf-8');(case/(mode+'.stderr.txt')).write_text(r.stderr,encoding='utf-8')
 assert r.returncode==0,(mode,r.stdout,r.stderr)
 runs.append({'mode':mode,'exit':r.returncode,'command':command})
stages=json.loads((case/'stages/native.host.json').read_text(encoding='utf-8-sig'));assert stages['exit_code']==0
assert '0 passed; 0 failed' in (case/'stages/native.stdout.log').read_text()
stream=json.loads((case/'streaming/host-measurement.json').read_text(encoding='utf-8-sig'));assert stream['fresh_software_receipt'] and stream['exit_code']==0
assert json.loads((case/'streaming/burn-wgpu-software.json').read_text())=={}
formats=json.loads((case/'formats/f16-burn-flex.host.json').read_text(encoding='utf-8-sig'));assert formats['fresh_software_receipt'] and formats['exit_code']==0
assert json.loads((case/'formats/f16-burn-flex.json').read_text())=={}
browser=(ROOT/paths[-1]).read_text(encoding='utf-8-sig')
finally_body=browser.split('.finally(async()=>{',1)[1].rsplit('});',1)[0]
js=HERE/'final_receipt_close_probe.cjs'
js.write_text(r'''const assert=require('node:assert/strict');let captured;const fs={writeFileSync(_path,data){captured=JSON.parse(data)}},path={join(){return 'unused-output'}},out='unused-output';let timeout;const context={async close(){throw Error('synthetic context close refusal')}},browser={async close(){throw Error('synthetic browser close refusal')}},server={close(cb){cb()}};(async()=>{'''+finally_body+r'''})().then(()=>{assert.equal(captured.browser_closed,true);assert.equal(captured.http_closed,true);console.log(JSON.stringify({context_close:'rejected',browser_close:'rejected',recorded:captured}));});''',encoding='utf-8')
r=subprocess.run([shutil.which('node'),str(js)],cwd=ROOT,text=True,capture_output=True,timeout=10);assert r.returncode==0,r.stderr
(case/'browser-close.stdout.txt').write_text(r.stdout,encoding='utf-8');(case/'browser-close.stderr.txt').write_text(r.stderr,encoding='utf-8')
removed=[]
for p in outputs:
 if not p.exists():continue
 assert p.resolve().parent==target.resolve();b=p.read_bytes()
 assert (p.suffix=='.exe' and b.startswith(b'MZ')) or (p.suffix=='.pdb' and b.startswith(b'Microsoft C/C++ MSF'))
 removed.append({'path':str(p),'bytes':len(b),'sha256':sha(p)});p.unlink()
assert before=={p:sha(ROOT/p) for p in paths}
evidence=[p for p in case.rglob('*') if p.is_file()]+[Path(__file__),empty,fake,harness,js]
record={'date_utc':datetime.now(timezone.utc).isoformat(),'exit':0,'source_sha256':before,'compiles':compiles,'runs':runs,'cases':{'stages_accepts_zero_named_tests':True,'streaming_accepts_empty_software_receipt':True,'formats_accepts_empty_software_receipt':True,'custody_functions_return_failed_result_without_rejecting':2,'browser_release_claims_closed_after_close_errors':True},'removed_outputs':removed,'finally_body_sha256':hashlib.sha256(finally_body.encode()).hexdigest(),'evidence_sha256':{p.relative_to(HERE).as_posix():sha(p) for p in evidence},'qualification':'Three unchanged resident wrapper scripts executed with explicit synthetic nvidia-smi/Get-CimInstance functions. Stages used a real rustc empty-filter test executable; other two used a deliberate benign no-test executable writing emptyJSON. No model read, GPU, native acceptance or performance receipt. Both actual AST-extracted Invoke-Probe functions used a fake child returning failedJSON with exit0; outer broker/watch lifecycle not executed. Actual browser finally callback ran with rejecting context/browser doubles and in-memory filesystem; no browser opened. All product source unchanged; generated EXE/PDB removed.'}
with (HERE/'final_receipt_contract_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'exit':0,'cases':record['cases'],'retained_bytes':sum(p.stat().st_size for p in evidence)},indent=2))
