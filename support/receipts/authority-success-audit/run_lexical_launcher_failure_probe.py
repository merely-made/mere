"""Execute unchanged launcher with child-only PATH/native-command doubles."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,shutil,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
NAME='lexical_launcher_failure_probe';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/(NAME+'_receipt.json')).exists()
source='crates/probes/mesh-lexical-wasm/run.ps1';before=sha(ROOT/source)
stubs=HERE/'probe-native-stubs';pwsh=shutil.which('pwsh');assert pwsh
command=[pwsh,'-NoLogo','-NoProfile','-NonInteractive','-File',str(ROOT/source)]
cases=[]
for label,code in [('successful_build_control',0),('failed_build_still_runs_node',7)]:
    env=dict(os.environ);env['PATH']=str(stubs)+os.pathsep+env.get('PATH','');env['AUDIT_CARGO_EXIT']=str(code)
    result=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,timeout=30)
    row={'case':label,'cargo_stub_exit':code,'launcher_exit':result.returncode,
         'node_reached':'AUDIT_NODE_STUB_REACHED' in result.stdout,'stdout':result.stdout,'stderr':result.stderr}
    cases.append(row)
    assert f'AUDIT_CARGO_STUB_EXIT_{code}' in result.stdout
    assert row['node_reached'] and row['launcher_exit']==0,row
assert sha(ROOT/source)==before
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),
    'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
    'source_sha256':{source:before},'command':command,'checks':2,'audit_cases':2,'original_tests':0,'cases':cases,
    'runner_sha256':sha(Path(__file__)),'stub_sha256':{p.name:sha(p) for p in stubs.iterdir() if p.is_file()},
    'generated_build_output':False,'qualification':'Unchanged complete production PowerShell launcher executed in fresh no-profile child PowerShell. Child-only PATH selects explicit cargo.cmd and node.cmd doubles. Cargo exit7 is ignored; Node double is invoked and launcher exits0, with successful-build control. No real Cargo build, stale Wasm loading, native/wasm digest comparison, Node engine or hardware execution; stale-artifact reuse is the source-level consequence if the existing Wasm path is present.'}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:json.dump(receipt,f,indent=2);f.write('\n')
print(json.dumps(receipt,indent=2))
