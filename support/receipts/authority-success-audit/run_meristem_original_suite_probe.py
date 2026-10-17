"""Run unchanged current Meristem integration and core unit tests over exact suppliers."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve();NAME='meristem_original_suite_probe'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/(NAME+'_receipt.json')).exists()
prior=json.loads((HERE/'meristem_state_probe_receipt.json').read_text());libs=prior['libraries']
assert all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
sources=prior['source_sha256'];assert all(sha(ROOT/n)==h for n,h in sources.items())
library=TARGET/'libauthority_success_meristem_original_suite.rlib'
binaries=[TARGET/'authority-success-meristem-original-integration.exe',TARGET/'authority-success-meristem-original-unit.exe']
assert not library.exists() and all(not list(TARGET.glob(b.stem+'.*')) for b in binaries)
common=['-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0']
for n,v in libs.items():common+=['--extern',n+'='+v['path']]
source=ROOT/'crates/cambium/meristem/src/lib.rs'
commands=[['rustc','--edition=2024','--crate-name','meristem','--crate-type','rlib',str(source),'-C','metadata=authority_success_meristem_original_suite','-o',str(library),*common],
 ['rustc','--edition=2024','--test',str(HERE/(NAME+'.rs')),'--extern','meristem='+str(library),'-o',str(binaries[0]),*common],
 ['rustc','--edition=2024','--test','--crate-name','meristem',str(source),'-o',str(binaries[1]),*common]]
builds=[];runs=[];removed=[]
try:
 for i,cmd in enumerate(commands):
  result=subprocess.run(cmd,cwd=ROOT,capture_output=True,text=True,timeout=90);builds.append(result.returncode)
  with (HERE/(NAME+f'_compile_{i}.log')).open('x',encoding='utf-8') as f:f.write(result.stdout+result.stderr)
  if result.returncode:print(result.stderr);break
 if builds==[0,0,0]:
  for i,b in enumerate(binaries):
   ran=subprocess.run([str(b),'--test-threads=1'],cwd=ROOT,capture_output=True,text=True,timeout=30)
   with (HERE/(NAME+f'_output_{i}.log')).open('x',encoding='utf-8') as f:f.write(ran.stdout+ran.stderr)
   match=re.search(r'test result: ok\. (\d+) passed;',ran.stdout)
   runs.append({'exit':ran.returncode,'checks':int(match.group(1)) if match else 0});print(ran.stdout+ran.stderr)
finally:
 for p in [library,*[p for b in binaries for p in sorted(TARGET.glob(b.stem+'.*'))]]:
  if not p.exists():continue
  assert p.resolve().parent==TARGET;data=p.read_bytes()
  assert (p==library and data.startswith(b'!<arch>\n')) or (p.suffix=='.exe' and data.startswith(b'MZ')) or (p.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF'))
  removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
assert all(sha(ROOT/n)==h for n,h in sources.items()) and all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'source_sha256':sources,'libraries':libs,'commands':commands,'build_exits':builds,'runs':runs,'checks':sum(r['checks'] for r in runs),'runner_sha256':sha(Path(__file__)),'probe_sha256':sha(HERE/(NAME+'.rs')),'removed_outputs':removed,'generated_output_removed':not library.exists() and all(not list(TARGET.glob(b.stem+'.*')) for b in binaries),'scope':'All eight unchanged current Meristem integration files via path modules, and original src/lib.rs unit tests. Fresh current no-optional-feature Meristem over exact retained tracing/anymore. No doctest, optional kurbo, application/native DOM or full fresh supplier graph acceptance.'}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:f.write(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({k:receipt[k] for k in ['build_exits','runs','checks','generated_output_removed']}))
assert builds==[0,0,0] and len(runs)==2 and all(r['exit']==0 for r in runs)
