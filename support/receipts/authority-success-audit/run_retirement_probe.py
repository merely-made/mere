"""Athanor retirement observations, exact production source module and retained dependencies."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,struct
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
dest=HERE/'retirement_probe_receipt.json';assert not dest.exists()
sources={n:sha(ROOT/n) for n in ['ports/distillery/athanor/src/lib.rs','ports/distillery/athanor/Cargo.toml','crates/eidetic/eidetic-core/src/deleted.rs','crates/eidetic/eidetic-core/src/typed.rs','crates/eidetic/eidetic-core/src/manifest.rs']}
rows=[json.loads(l) for l in (HERE/'trainer_flora_probe_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')]
assert any(r.get('reason')=='build-finished' and r.get('success') for r in rows)
libs={}
for name in ['alembic','eidetic','kernel','pandect','pollster']:
 matches=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']==name and any(f.endswith('.rlib') for f in r['filenames'])];assert len(matches)==1,(name,len(matches))
 r=matches[0];path=next(Path(f) for f in r['filenames'] if f.endswith('.rlib'))
 if path.parent==TARGET/'debug':
  candidates=[p for p in (TARGET/'build/debug/deps').glob('lib'+name+'-*.rlib') if sha(p)==sha(path)];assert candidates;path=candidates[0]
 libs[name]={'path':str(path),'sha256':sha(path),'features':r['features'],'package_id':r['package_id'],'target_source':r['target']['src_path']}
binary=TARGET/'authority-success-retirement-probe.exe';assert not list(TARGET.glob(binary.stem+'.*'))
command=['rustc','--edition=2024',str(HERE/'retirement_probe.rs'),'-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0','-o',str(binary)]
for name,lib in libs.items():command+=['--extern',name+'='+lib['path']]
native=sorted({p for r in rows if r.get('reason')=='build-script-executed' for p in r.get('linked_paths',[])})
for p in native:command+=['-L',p]
built=subprocess.run(command,cwd=ROOT,capture_output=True,text=True)
(HERE/'retirement_probe_compile.log').write_text(built.stdout+built.stderr,encoding='utf-8');assert built.returncode==0,built.stderr
digest=sha(binary);removed=[]
try:
 ran=subprocess.run([str(binary)],cwd=ROOT,capture_output=True,text=True,timeout=45)
 (HERE/'retirement_probe_output.log').write_text(ran.stdout+ran.stderr,encoding='utf-8');print(ran.stdout)
finally:
 for p in sorted(TARGET.glob(binary.stem+'.*')):
  assert p.resolve().parent==TARGET;data=p.read_bytes()
  if p.suffix=='.exe':assert data.startswith(b'MZ')
  elif p.suffix=='.pdb':assert data.startswith(b'Microsoft C/C++ MSF')
  elif p.suffix=='.lib':assert data.startswith(b'!<arch>\n') and binary.name.encode() in data
  elif p.suffix=='.exp':assert data[:2]==b'\x64\x86' and 0<struct.unpack_from('<H',data,2)[0]<128 and binary.name.encode() in data
  else:raise AssertionError('unknown generated output '+str(p))
  removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
assert ran.returncode==0 and '2 checks passed' in ran.stdout,ran.stderr
assert all(sha(ROOT/n)==h for n,h in sources.items());assert all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'source_sha256':sources,'libraries':libs,'native_search_paths':native,'probe_build_command':command,'probe_build_exit':built.returncode,'native_binary_sha256':digest,'test_exit':ran.returncode,'checks':2,'removed_outputs':removed,'generated_output_removed':not list(TARGET.glob(binary.stem+'.*')),'scope':'Exact production Athanor module compiled directly into the native audit executable, using retained Cargo-built dependencies from trainer_flora_probe_build.jsonl. Public propose_retirement/apply_retirement and real Eidetic record/list APIs. Synthetic memory-backed tombstones only; same-node expired/unexpired pair and different-node control. This is not a new Cargo package test or resident scheduler, disk, UI, user-data or network qualification.'}
dest.write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8');print(json.dumps({'receipt':dest.name,'checks':2,'exit':ran.returncode,'generated_output_removed':receipt['generated_output_removed']}))
