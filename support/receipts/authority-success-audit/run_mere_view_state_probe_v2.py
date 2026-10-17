"""Compile current original Mere-view over exact retained dependency bytes."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[2]; TARGET=Path('C:/t/cargo-targets/mere').resolve()
NAME='mere_view_state_probe_v2'; receipt=HERE/(NAME+'_receipt.json'); assert not receipt.exists()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
rows=[json.loads(l) for l in (HERE/'capture_graphshell_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')]
assert any(r.get('reason')=='build-finished' and r.get('success') for r in rows)
libs={}
for name in ['curation','cambium','cartography','pictograph','incipit','kernel','sprigging','uuid']:
 matches=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']==name and any(f.endswith('.rlib') for f in r['filenames'])]
 assert len(matches)==1,(name,len(matches))
 row=matches[0]; path=Path(next(f for f in row['filenames'] if f.endswith('.rlib')))
 libs[name]={'path':str(path),'sha256':sha(path),'package_id':row['package_id'],'features':row['features']}
native=sorted({p for r in rows if r.get('reason')=='build-script-executed' for p in r.get('linked_paths',[])})
sources={str(p.relative_to(ROOT)).replace('\\','/'):sha(p) for p in (ROOT/'crates/cambium/mere-view').rglob('*.rs')}
common=['-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0']
for name,lib in libs.items():common+=['--extern',name+'='+lib['path']]
for p in native:common+=['-L',p]
rlib=TARGET/'libauthority_success_mere_view_v2.rlib'; binary=TARGET/'authority-success-mere-view-state-probe-v2.exe'
assert not rlib.exists() and not list(TARGET.glob(binary.stem+'.*'))
commands=[['rustc','--edition=2024','--crate-name','mere_view','--crate-type','rlib',str(ROOT/'crates/cambium/mere-view/src/lib.rs'),'-C','metadata=authority_success_mere_view','-o',str(rlib),*common],
 ['rustc','--edition=2024',str(HERE/'mere_view_state_probe.rs'),'--extern','mere_view='+str(rlib),'-o',str(binary),*common]]
builds=[]; ran=None; removed=[]
try:
 for i,cmd in enumerate(commands):
  result=subprocess.run(cmd,cwd=ROOT,capture_output=True,text=True,timeout=90); builds.append(result.returncode)
  with (HERE/(NAME+f'_compile_{i}.log')).open('x',encoding='utf-8') as f:f.write(result.stdout+result.stderr)
  if result.returncode:print(result.stderr);break
 if builds==[0,0]:
  ran=subprocess.run([str(binary)],cwd=ROOT,capture_output=True,text=True,timeout=30)
  with (HERE/(NAME+'_output.log')).open('x',encoding='utf-8') as f:f.write(ran.stdout+ran.stderr)
  print(ran.stdout+ran.stderr)
finally:
 for p in [rlib,*sorted(TARGET.glob(binary.stem+'.*'))]:
  if not p.exists():continue
  assert p.resolve().parent==TARGET; data=p.read_bytes()
  assert (p==rlib and data.startswith(b'!<arch>\n')) or (p.suffix=='.exe' and data.startswith(b'MZ')) or (p.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF')),str(p)
  removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
assert all(sha(ROOT/n)==h for n,h in sources.items())
assert all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
value={'date_utc':datetime.now(timezone.utc).isoformat(),'source_sha256':sources,'libraries':libs,'commands':commands,'build_exits':builds,
 'run_exit':None if ran is None else ran.returncode,'checks':0 if ran is None else ran.stdout.count('PASS '),'removed_outputs':removed,
 'generated_output_removed':not rlib.exists() and not list(TARGET.glob(binary.stem+'.*')),
 'probe_sha256':sha(HERE/'mere_view_state_probe.rs'),'runner_sha256':sha(Path(__file__)),
 'scope':'Entire original current Mere-view crate compiled fresh over exact retained dependency bytes from earlier successful Graphshell build. Public state and Swatch methods only. No rendered DOM, native GUI, owner merge apply, persistence, network, or full-current Cargo gate.'}
with receipt.open('x',encoding='utf-8') as f:f.write(json.dumps(value,indent=2)+'\n')
print(json.dumps({'receipt':receipt.name,'build_exits':builds,'run_exit':value['run_exit'],'checks':value['checks']}))
assert builds==[0,0] and ran.returncode==0
