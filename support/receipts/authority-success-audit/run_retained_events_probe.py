"""Compile original current Cambium and exercise retained event/frame contracts."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve();NAME='retained_events_probe'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/(NAME+'_receipt.json')).exists()
rows=[json.loads(l) for l in (HERE/'capture_graphshell_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')]
assert any(r.get('reason')=='build-finished' and r.get('success') for r in rows)
libs={}
for name in ['backstep','command_menu','meristem','sprigging','sceno','scenograph','scenomise','genet_scripted_dom','mere_surface_api','workbench','layout_dom_api','genet_text']:
 matches=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']==name and any(f.endswith('.rlib') for f in r['filenames'])];assert len(matches)==1,(name,len(matches))
 row=matches[0];p=Path(next(f for f in row['filenames'] if f.endswith('.rlib')));libs[name]={'path':str(p),'sha256':sha(p),'features':row['features'],'package_id':row['package_id']}
sources={str(p.relative_to(ROOT)).replace('\\','/'):sha(p) for p in (ROOT/'crates/cambium/cambium/src').rglob('*.rs')}
sources['crates/cambium/cambium/Cargo.toml']=sha(ROOT/'crates/cambium/cambium/Cargo.toml')
rlib=TARGET/'libauthority_success_cambium_events.rlib';binary=TARGET/'authority-success-retained-events-probe.exe';assert not rlib.exists() and not list(TARGET.glob(binary.stem+'.*'))
common=['-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0']
for n,v in libs.items():common+=['--extern',n+'='+v['path']]
for p in sorted({p for r in rows if r.get('reason')=='build-script-executed' for p in r.get('linked_paths',[])}):common+=['-L',p]
commands=[['rustc','--edition=2024','--crate-name','cambium','--crate-type','rlib',str(ROOT/'crates/cambium/cambium/src/lib.rs'),'-C','metadata=authority_success_cambium_events','-o',str(rlib),*common],
 ['rustc','--edition=2024',str(HERE/(NAME+'.rs')),'--extern','cambium='+str(rlib),'-o',str(binary),*common]]
builds=[];ran=None;removed=[]
try:
 for i,cmd in enumerate(commands):
  result=subprocess.run(cmd,cwd=ROOT,capture_output=True,text=True,timeout=90);builds.append(result.returncode)
  with (HERE/(NAME+f'_compile_{i}.log')).open('x',encoding='utf-8') as f:f.write(result.stdout+result.stderr)
  if result.returncode:print(result.stderr);break
 if builds==[0,0]:
  ran=subprocess.run([str(binary)],cwd=ROOT,capture_output=True,text=True,timeout=30)
  with (HERE/(NAME+'_output.log')).open('x',encoding='utf-8') as f:f.write(ran.stdout+ran.stderr)
  print(ran.stdout+ran.stderr)
finally:
 for p in [rlib,*sorted(TARGET.glob(binary.stem+'.*'))]:
  if not p.exists():continue
  assert p.resolve().parent==TARGET;data=p.read_bytes()
  assert (p==rlib and data.startswith(b'!<arch>\n')) or (p.suffix=='.exe' and data.startswith(b'MZ')) or (p.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF'))
  removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
assert all(sha(ROOT/n)==h for n,h in sources.items()) and all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'source_sha256':sources,'libraries':libs,'commands':commands,'build_exits':builds,'run_exit':None if ran is None else ran.returncode,
 'checks':0 if ran is None else ran.stdout.count('PASS '),'probe_sha256':sha(HERE/(NAME+'.rs')),'runner_sha256':sha(Path(__file__)),
 'removed_outputs':removed,'generated_output_removed':not rlib.exists() and not list(TARGET.glob(binary.stem+'.*')),
 'scope':'Entire original current Cambium crate compiled fresh with default/no optional features over exact retained dependency artifacts. Actual current runner, wrappers, components and Frisket; synthetic input and DOM/local-state observations only. No native host, GPU, network, persisted document or fresh full-current transitive Cargo gate. Compilation does not count as source-reading coverage.'}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:f.write(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({'receipt':NAME+'_receipt.json','build_exits':builds,'run_exit':receipt['run_exit'],'checks':receipt['checks']}))
assert builds==[0,0] and ran.returncode==0
