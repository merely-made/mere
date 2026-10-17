"""Original current Sprigging, unchanged unit suite plus actual cache counterexamples."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve();NAME='sprigging_retention_probe'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/(NAME+'_receipt.json')).exists()
rows=[json.loads(l) for l in (HERE/'capture_graphshell_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')];libs={}
for name in ['paint_list_api','accesskit','vello']:
 matches=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']==name and any(f.endswith('.rlib') for f in r['filenames'])];assert len(matches)==1
 row=matches[0];p=Path(next(f for f in row['filenames'] if f.endswith('.rlib')));libs[name]={'path':str(p),'sha256':sha(p),'features':row['features'],'package_id':row['package_id']}
source_root=ROOT/'crates/cambium/sprigging';sources={p.relative_to(ROOT).as_posix():sha(p) for p in source_root.rglob('*') if p.is_file()}
library=TARGET/'libauthority_success_sprigging_retention.rlib';binaries=[TARGET/'authority-success-sprigging-retention.exe',TARGET/'authority-success-sprigging-original-unit.exe']
assert not library.exists() and all(not list(TARGET.glob(b.stem+'.*')) for b in binaries)
common=['-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0']
for n,v in libs.items():common+=['--extern',n+'='+v['path']]
source=source_root/'src/lib.rs'
commands=[['rustc','--edition=2024','--crate-name','sprigging','--crate-type','rlib',str(source),'-C','metadata=authority_success_sprigging_retention','-o',str(library),*common],
 ['rustc','--edition=2024','--test',str(HERE/(NAME+'.rs')),'--extern','sprigging='+str(library),'-o',str(binaries[0]),*common],
 ['rustc','--edition=2024','--test','--crate-name','sprigging',str(source),'-o',str(binaries[1]),*common]]
builds=[];runs=[];removed=[]
try:
 for i,cmd in enumerate(commands):
  r=subprocess.run(cmd,cwd=ROOT,capture_output=True,text=True,timeout=90);builds.append(r.returncode)
  with (HERE/(NAME+f'_compile_{i}.log')).open('x',encoding='utf-8') as f:f.write(r.stdout+r.stderr)
  if r.returncode:print(r.stderr);break
 if builds==[0,0,0]:
  for i,b in enumerate(binaries):
   r=subprocess.run([str(b),'--test-threads=1'],cwd=ROOT,capture_output=True,text=True,timeout=30)
   with (HERE/(NAME+f'_output_{i}.log')).open('x',encoding='utf-8') as f:f.write(r.stdout+r.stderr)
   m=re.search(r'test result: ok\. (\d+) passed;',r.stdout);runs.append({'exit':r.returncode,'checks':int(m.group(1)) if m else 0});print(r.stdout+r.stderr)
finally:
 for p in [library,*[p for b in binaries for p in sorted(TARGET.glob(b.stem+'.*'))]]:
  if not p.exists():continue
  assert p.resolve().parent==TARGET;data=p.read_bytes()
  assert (p==library and data.startswith(b'!<arch>\n')) or (p.suffix=='.exe' and data.startswith(b'MZ')) or (p.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF'))
  removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
assert all(sha(ROOT/n)==h for n,h in sources.items()) and all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'source_sha256':sources,'libraries':libs,'commands':commands,'build_exits':builds,'runs':runs,'checks':sum(r['checks'] for r in runs),'runner_sha256':sha(Path(__file__)),'probe_sha256':sha(HERE/(NAME+'.rs')),'removed_outputs':removed,'generated_output_removed':not library.exists() and all(not list(TARGET.glob(b.stem+'.*')) for b in binaries),'scope':'Fresh original current Sprigging over three exact retained libraries; original core unit cases and three actual registry/cache counterexamples. No GPU rasterization, live product replacement/style-change consumer, native hit dispatch or whole fresh supplier graph acceptance.'}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:f.write(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({k:receipt[k] for k in ['build_exits','runs','checks','generated_output_removed']}))
assert builds==[0,0,0] and len(runs)==2 and all(r['exit']==0 for r in runs) and runs[0]['checks']==3
