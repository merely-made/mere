"""Compile current original command modules against retained runner/type artifacts."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve();NAME='command_tree_probe'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/(NAME+'_receipt.json')).exists()
rows=[json.loads(l) for l in (HERE/'capture_graphshell_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')]
assert any(r.get('reason')=='build-finished' and r.get('success') for r in rows)
libs={}
for name in ['cambium','meristem','command_menu','genet_scripted_dom','layout_dom_api']:
 matches=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']==name and any(f.endswith('.rlib') for f in r['filenames'])];assert len(matches)==1,(name,len(matches))
 row=matches[0];p=Path(next(f for f in row['filenames'] if f.endswith('.rlib')));libs[name]={'path':str(p),'sha256':sha(p),'features':row['features'],'package_id':row['package_id']}
sources={n:sha(ROOT/n) for n in ['crates/cambium/cambium/src/command_surface.rs','crates/cambium/cambium/src/command_menu_bar.rs']}
binary=TARGET/'authority-success-command-tree-probe.exe';assert not list(TARGET.glob(binary.stem+'.*'))
cmd=['rustc','--edition=2024',str(HERE/(NAME+'.rs')),'-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0','-o',str(binary)]
for n,v in libs.items():cmd+=['--extern',n+'='+v['path']]
for p in sorted({p for r in rows if r.get('reason')=='build-script-executed' for p in r.get('linked_paths',[])}):cmd+=['-L',p]
built=subprocess.run(cmd,cwd=ROOT,capture_output=True,text=True,timeout=90)
with (HERE/(NAME+'_compile.log')).open('x',encoding='utf-8') as f:f.write(built.stdout+built.stderr)
ran=None;removed=[]
try:
 if built.returncode==0:
  ran=subprocess.run([str(binary)],cwd=ROOT,capture_output=True,text=True,timeout=30)
  with (HERE/(NAME+'_output.log')).open('x',encoding='utf-8') as f:f.write(ran.stdout+ran.stderr)
  print(ran.stdout+ran.stderr)
 else:print(built.stderr)
finally:
 for p in sorted(TARGET.glob(binary.stem+'.*')):
  assert p.resolve().parent==TARGET;data=p.read_bytes()
  assert (p.suffix=='.exe' and data.startswith(b'MZ')) or (p.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF'))
  removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
assert all(sha(ROOT/n)==h for n,h in sources.items()) and all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'source_sha256':sources,'libraries':libs,'command':cmd,'build_exit':built.returncode,'run_exit':None if ran is None else ran.returncode,
 'checks':0 if ran is None else ran.stdout.count('PASS '),'probe_sha256':sha(HERE/(NAME+'.rs')),'runner_sha256':sha(Path(__file__)),
 'removed_outputs':removed,'generated_output_removed':not list(TARGET.glob(binary.stem+'.*')),
 'scope':'Original current command_surface and command_menu_bar modules included by path, current local model types, retained real Cambium runner and ScriptedDom. Synthetic keyboard/pointer input; no native host, OS input, GPU, backend command execution or fresh full-current Cargo gate.'}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:f.write(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({'receipt':NAME+'_receipt.json','build_exit':built.returncode,'run_exit':receipt['run_exit'],'checks':receipt['checks']}))
assert built.returncode==0 and ran.returncode==0
