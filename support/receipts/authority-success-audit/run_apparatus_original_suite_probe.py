"""Fresh original Apparatus units: standard core, then projection plus serde."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,subprocess,tomllib,os
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve();NAME='apparatus_original_suite_probe'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/(NAME+'_receipt.json')).exists()
rows=[json.loads(l) for l in (HERE/'capture_graphshell_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')];libs={}
for name in ['accesskit','tracing','uxtree','serde']:
 matches=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']==name and (name!='serde' or 'rc' in r['features']) and any(f.endswith('.rlib') for f in r['filenames'])];assert len(matches)==1
 row=matches[0];p=Path(next(f for f in row['filenames'] if f.endswith('.rlib')));libs[name]={'path':str(p),'sha256':sha(p),'features':row['features'],'package_id':row['package_id']}
source_root=ROOT/'crates/domain/apparatus';sources={p.relative_to(ROOT).as_posix():sha(p) for p in source_root.rglob('*') if p.is_file()}
version=tomllib.loads((ROOT/'Cargo.toml').read_text(encoding='utf-8-sig'))['workspace']['package']['version']
env=dict(os.environ,CARGO_PKG_VERSION=version)
binaries=[TARGET/'authority-success-apparatus-core-unit.exe',TARGET/'authority-success-apparatus-projection-unit.exe'];assert all(not list(TARGET.glob(b.stem+'.*')) for b in binaries)
common=['rustc','--edition=2024','--test','--crate-name','apparatus',str(source_root/'src/lib.rs'),'-C','debuginfo=0']
commands=[[*common,'-o',str(binaries[0])],[*common,'--cfg','feature="projection"','--cfg','feature="serde"','-L','dependency='+str(TARGET/'build/debug/deps'),'-o',str(binaries[1])]]
for n,v in libs.items():commands[1]+=['--extern',n+'='+v['path']]
builds=[];runs=[];removed=[]
try:
 for i,cmd in enumerate(commands):
  r=subprocess.run(cmd,cwd=ROOT,capture_output=True,text=True,env=env,timeout=90);builds.append(r.returncode)
  with (HERE/(NAME+f'_compile_{i}.log')).open('x',encoding='utf-8') as f:f.write(r.stdout+r.stderr)
  if r.returncode:print(r.stderr);break
  r=subprocess.run([str(binaries[i]),'--test-threads=1'],cwd=ROOT,capture_output=True,text=True,timeout=30)
  with (HERE/(NAME+f'_output_{i}.log')).open('x',encoding='utf-8') as f:f.write(r.stdout+r.stderr)
  m=re.search(r'test result: ok\. (\d+) passed;',r.stdout);runs.append({'exit':r.returncode,'checks':int(m.group(1)) if m else 0});print(r.stdout+r.stderr)
finally:
 for p in [p for b in binaries for p in sorted(TARGET.glob(b.stem+'.*'))]:
  assert p.resolve().parent==TARGET;data=p.read_bytes()
  assert (p.suffix=='.exe' and data.startswith(b'MZ')) or (p.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF'))
  removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
assert all(sha(ROOT/n)==h for n,h in sources.items()) and all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'source_sha256':sources,'libraries':libs,'commands':commands,'environment':{'CARGO_PKG_VERSION':version},'build_exits':builds,'runs':runs,'checks':sum(r['checks'] for r in runs),'runner_sha256':sha(Path(__file__)),'removed_outputs':removed,'generated_output_removed':all(not list(TARGET.glob(b.stem+'.*')) for b in binaries),'scope':'Original current Apparatus std-only core and projection+serde unit configurations, built fresh. First config standard library only; second four exact retained dependencies including Mere uxtree. No fresh whole supplier graph, product diagnostics/export, OS accessibility or application success acceptance.'}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:f.write(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({k:receipt[k] for k in ['build_exits','runs','checks','generated_output_removed']}))
assert builds==[0,0] and len(runs)==2 and all(r['exit']==0 for r in runs)
