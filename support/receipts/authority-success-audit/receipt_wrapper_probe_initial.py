"""Exercise unchanged historical runner copies with explicit subprocess injections.

No Cargo, compiler, browser or GPU is launched. Original receipt files are guarded.
The outer Python interpreter's observed exit is real, not the injected status.
"""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,shutil,subprocess,sys
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
sources={
 'custody':'design_docs/mere_docs/testing/receipts/2026-09-08_stack_pillar_probes/custody-backend/run.py',
 'metadata':'design_docs/cambium_docs/testing/receipts/2026-10-12_flattening_and_device_recovery/qualify_final_graphs.py'}
guards={p:sha(ROOT/p) for p in sources.values()}
historical=[
 'design_docs/mere_docs/testing/receipts/2026-09-08_stack_pillar_probes/custody-backend/run.json',
 'design_docs/mere_docs/testing/receipts/2026-09-08_stack_pillar_probes/custody-backend/run.log',
 'design_docs/cambium_docs/testing/receipts/2026-10-12_flattening_and_device_recovery/final-supplier-qualification.json']
historical_hashes={p:sha(ROOT/p) for p in historical}
base=HERE/'receipt-wrapper-fixtures';assert not base.exists();base.mkdir()
bootstrap='''import json,runpy,subprocess,sys
from pathlib import Path
from unittest.mock import patch
target=Path(sys.argv[1]); spec=json.loads(Path(sys.argv[2]).read_text())
sys.argv=[str(target),spec.get("knot_revision", "unused")]
completed=subprocess.CompletedProcess(["AUDIT injected subprocess"],spec["child_exit"],stdout=spec["stdout"],stderr="AUDIT injected stderr")
with patch("subprocess.run",return_value=completed), patch("subprocess.check_output",return_value="AUDIT injected compiler identity; no compiler execution"):
    runpy.run_path(str(target),run_name="__main__")
'''
(base/'bootstrap.py').write_text(bootstrap,encoding='utf-8')
expected=json.loads((ROOT/historical[-1]).read_text())['expected']
positive={'packages':[{'name':name+'-audit-package','version':'0.0.0',
 'source':f'git+https://github.com/audit/{name}.git?rev={revision}#{revision}'} for name,revision in expected.items()]}
wrong=json.loads(json.dumps(positive));wrong['packages'][0]['source']='git+https://github.com/audit/genet.git?rev='+'0'*40+'#'+'0'*40
cases=[
 ('custody-child-success','custody',{'child_exit':0,'stdout':b'AUDIT injected child success\n'.decode()}),
 ('custody-child-failure','custody',{'child_exit':101,'stdout':'AUDIT injected child failure 101\n'}),
 ('metadata-matched','metadata',{'child_exit':0,'stdout':json.dumps(positive),'knot_revision':expected['knot-editor']}),
 ('metadata-registry-only','metadata',{'child_exit':0,'stdout':json.dumps({'packages':[{'name':'netrender','version':'0.1.2','source':'registry+https://github.com/rust-lang/crates.io-index'}]}),'knot_revision':expected['knot-editor']}),
 ('metadata-mismatched','metadata',{'child_exit':0,'stdout':json.dumps(wrong),'knot_revision':expected['knot-editor']})]
results=[];env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1')
for name,kind,spec in cases:
    folder=base/name;folder.mkdir();runner=folder/'run.py'
    shutil.copyfile(ROOT/sources[kind],runner);assert sha(runner)==guards[sources[kind]]
    (folder/'injection.json').write_text(json.dumps(spec,indent=2),encoding='utf-8')
    result=subprocess.run([sys.executable,'-B',str(base/'bootstrap.py'),str(runner),str(folder/'injection.json')],
        cwd=ROOT,env=env,capture_output=True,text=True,timeout=20)
    (folder/'outer-output.log').write_text(result.stdout+result.stderr,encoding='utf-8')
    row={'case':name,'kind':kind,'outer_process_exit':result.returncode,'injected_child_exit':spec['child_exit'],
      'source_copy_sha256':sha(runner)}
    if kind=='custody':
        record=json.loads((folder/'run.json').read_text());row['recorded_child_exit']=record['exit_code']
        assert row['recorded_child_exit']==spec['child_exit'] and result.returncode==0
    elif name=='metadata-mismatched':
        assert result.returncode!=0 and 'AssertionError' in result.stderr
        assert not (folder/'final-supplier-qualification.json').exists()
    else:
        record=json.loads((folder/'final-supplier-qualification.json').read_text())
        row['accepted_roots']=sum(item['accepted'] for item in record['roots'])
        row['supplier_packages_per_root']=[len(item['suppliers']) for item in record['roots']]
        assert result.returncode==0 and row['accepted_roots']==5
        assert row['supplier_packages_per_root']==([0]*5 if name=='metadata-registry-only' else [4]*5)
    results.append(row)
assert all(sha(ROOT/p)==h for p,h in {**guards,**historical_hashes}.items())
record={'date_utc':datetime.now(timezone.utc).isoformat(),'source_sha256':guards,'preserved_historical_receipt_sha256':historical_hashes,
 'results':results,'fixture_sha256':{p.relative_to(HERE).as_posix():sha(p) for p in base.rglob('*') if p.is_file()},
 'runner_sha256':sha(Path(__file__)),
 'qualification':'Five complete byte-identical historical runner-copy executions in separate real Python processes. Child subprocess boundary is explicitly mocked: injected Cargo status/metadata and compiler string, no actual Cargo/compiler/build/host/browser/network/GPU. Outer exits, file writes and checker branches are actual. Metadata registry-only input is a controlled fixture, not freshly resolved real workspace metadata. Historical published custody child exit0/three tests and nonempty supplier receipts remain byte-preserved; these findings do not invalidate their recorded successful runs.'}
with (HERE/'receipt_wrapper_probe_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
print(json.dumps({'cases':results,'fixture_bytes':sum(p.stat().st_size for p in base.rglob('*') if p.is_file()),'qualification':record['qualification']},indent=2))
