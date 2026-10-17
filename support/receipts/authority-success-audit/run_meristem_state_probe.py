"""Compile current Meristem over two exact suppliers; runtime and const counterexamples."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve();NAME='meristem_state_probe'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/(NAME+'_receipt.json')).exists()
rows=[json.loads(l) for l in (HERE/'capture_graphshell_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')]
libs={}
for name in ['anymore','tracing']:
 candidates=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']==name and any(f.endswith('.rlib') for f in r['filenames'])];assert len(candidates)==1
 row=candidates[0];p=Path(next(f for f in row['filenames'] if f.endswith('.rlib')));libs[name]={'path':str(p),'sha256':sha(p),'features':row['features'],'package_id':row['package_id']}
source_root=ROOT/'crates/cambium/meristem';sources={p.relative_to(ROOT).as_posix():sha(p) for p in source_root.rglob('*') if p.is_file()}
library=TARGET/'libauthority_success_meristem_state.rlib';binary=TARGET/'authority-success-meristem-state-probe.exe';constant=TARGET/'authority-success-meristem-count-const-probe.exe'
assert not library.exists() and not list(TARGET.glob(binary.stem+'.*')) and not list(TARGET.glob(constant.stem+'.*'))
common=['-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0']
for n,v in libs.items():common+=['--extern',n+'='+v['path']]
commands=[['rustc','--edition=2024','--crate-name','meristem','--crate-type','rlib',str(source_root/'src/lib.rs'),'-C','metadata=authority_success_meristem_state','-o',str(library),*common],
 ['rustc','--edition=2024','--test',str(HERE/(NAME+'.rs')),'--extern','meristem='+str(library),'-o',str(binary),*common],
 ['rustc','--edition=2024',str(HERE/'meristem_count_const_probe.rs'),'--extern','meristem='+str(library),'-o',str(constant),*common]]
builds=[];ran=None;removed=[]
try:
 for i,cmd in enumerate(commands):
  result=subprocess.run(cmd,cwd=ROOT,capture_output=True,text=True,timeout=90);builds.append(result.returncode)
  with (HERE/(NAME+f'_compile_{i}.log')).open('x',encoding='utf-8') as f:f.write(result.stdout+result.stderr)
  if result.returncode and i<2:print(result.stderr);break
 if builds==[0,0,1]:
  ran=subprocess.run([str(binary),'--test-threads=1'],cwd=ROOT,capture_output=True,text=True,timeout=30)
  with (HERE/(NAME+'_output.log')).open('x',encoding='utf-8') as f:f.write(ran.stdout+ran.stderr)
  print(ran.stdout+ran.stderr)
finally:
 for p in [library,*sorted(TARGET.glob(binary.stem+'.*')),*sorted(TARGET.glob(constant.stem+'.*'))]:
  if not p.exists():continue
  assert p.resolve().parent==TARGET;data=p.read_bytes()
  assert (p==library and data.startswith(b'!<arch>\n')) or (p.suffix=='.exe' and data.startswith(b'MZ')) or (p.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF'))
  removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
assert all(sha(ROOT/n)==h for n,h in sources.items()) and all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
checks=0 if ran is None or ran.returncode else int(re.search(r'test result: ok\. (\d+) passed;',ran.stdout).group(1))
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'source_sha256':sources,'libraries':libs,'commands':commands,'build_exits':builds,'run_exit':None if ran is None else ran.returncode,'checks':checks,'runner_sha256':sha(Path(__file__)),'probe_sha256':sha(HERE/(NAME+'.rs')),'constant_probe_sha256':sha(HERE/'meristem_count_const_probe.rs'),'removed_outputs':removed,'generated_output_removed':not library.exists() and not list(TARGET.glob(binary.stem+'.*')) and not list(TARGET.glob(constant.stem+'.*')),'scope':'Original current no-optional-feature Meristem, compiled fresh over exact retained tracing/anymore. Three public Count/core sequence counterexamples using the original current Meristem test element/splice fixture. Separate const consumer expects E0080. No native DOM/application routing or fresh whole supplier graph acceptance; compilation is not source-reading credit.'}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:f.write(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({k:receipt[k] for k in ['build_exits','run_exit','checks','generated_output_removed']}))
assert builds==[0,0,1] and checks==3 and ran.returncode==0 and 'E0080' in (HERE/(NAME+'_compile_2.log')).read_text()
