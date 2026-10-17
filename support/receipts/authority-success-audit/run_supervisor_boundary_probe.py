"""Reuse exact fresh predecessor library artifacts; compile only this audit probe."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess, struct
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
dest=HERE/'supervisor_boundary_probe_receipt.json';assert not dest.exists()
prior=json.loads((HERE/'inference_crash_probe_receipt.json').read_text())
assert all(sha(ROOT/n)==h for n,h in prior['source_sha256'].items())
assert all(sha(Path(v['path']))==v['sha256'] for v in prior['libraries'].values())
paths=['ports/distillery/src/authority.rs','ports/distillery/src/mesh_host.rs','ports/distillery/src/mesh_host/courier.rs','ports/distillery/src/mesh_host/host.rs','ports/distillery/src/mesh_host/inflight.rs','ports/distillery/src/mesh_host/sense.rs','crates/mesh/mesh/src/board.rs','crates/mesh/mesh/src/lease.rs']
sources={n:sha(ROOT/n) for n in paths}
rows=[json.loads(l) for l in (HERE/'inference_crash_probe_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')]
assert any(r.get('reason')=='build-finished' and r.get('success') for r in rows)
libs={}
for name in ['distillery','mesh','personae','transport','tokio']:
 matches=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']==name and any(f.endswith('.rlib') for f in r['filenames'])]
 assert len(matches)==1,(name,len(matches))
 r=matches[0];path=next(Path(f) for f in r['filenames'] if f.endswith('.rlib'))
 if path.parent==TARGET/'debug':
  candidates=[p for p in (TARGET/'build/debug/deps').glob('lib'+name+'-*.rlib') if sha(p)==sha(path)];assert candidates;path=candidates[0]
 libs[name]={'path':str(path),'sha256':sha(path),'features':r['features'],'package_id':r['package_id'],'target_source':r['target']['src_path']}
binary=TARGET/'authority-success-supervisor-boundary-probe.exe';assert not list(TARGET.glob(binary.stem+'.*'))
command=['rustc','--edition=2024','--test',str(HERE/'supervisor_boundary_probe.rs'),'-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0','-o',str(binary)]
for name,lib in libs.items():command+=['--extern',name+'='+lib['path']]
native=sorted({p for r in rows if r.get('reason')=='build-script-executed' for p in r.get('linked_paths',[])})
for p in native:command+=['-L',p]
built=subprocess.run(command,cwd=ROOT,capture_output=True,text=True)
(HERE/'supervisor_boundary_probe_compile.log').write_text(built.stdout+built.stderr,encoding='utf-8');assert built.returncode==0,built.stderr
digest=sha(binary);removed=[]
try:
 ran=subprocess.run([str(binary),'--nocapture','--test-threads=1'],cwd=ROOT,capture_output=True,text=True,timeout=45)
 (HERE/'supervisor_boundary_probe_output.log').write_text(ran.stdout+ran.stderr,encoding='utf-8');print(ran.stdout)
finally:
 for p in sorted(TARGET.glob(binary.stem+'.*')):
  assert p.resolve().parent==TARGET;data=p.read_bytes()
  if p.suffix=='.exe':assert data.startswith(b'MZ')
  elif p.suffix=='.pdb':assert data.startswith(b'Microsoft C/C++ MSF')
  elif p.suffix=='.lib':assert data.startswith(b'!<arch>\n') and binary.name.encode() in data
  elif p.suffix=='.exp':assert data[:2]==b'\x64\x86' and 0<struct.unpack_from('<H',data,2)[0]<128 and binary.name.encode() in data
  else:raise AssertionError('unrecognized output '+str(p))
  removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
assert ran.returncode==0,ran.stderr
assert all(sha(ROOT/n)==h for n,h in sources.items())
assert all(sha(Path(lib['path']))==lib['sha256'] for lib in libs.values())
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'source_sha256':sources,'libraries':libs,'prior_build_receipt':{'path':'inference_crash_probe_receipt.json','sha256':sha(HERE/'inference_crash_probe_receipt.json')},'probe_build_command':command,'native_search_paths':native,'native_binary_sha256':digest,'probe_build_exit':built.returncode,'test_exit':ran.returncode,'tests':4,'generated_output_removed':not list(TARGET.glob(binary.stem+'.*')),'removed_outputs':removed,'scope':'Public native APIs using one local collecting blob store for cached-input custody and a single local P2panda/SyncedMesh endpoint for supervisor reaping. Two positive controls. Synthetic in-memory data and manual clock only; no remote peer, real job payload, multi-computer run or forced GC of user storage. Exact previous successful library build reused under unchanged source/artifact hashes; no new Cargo build is implied.'}
dest.write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8');print(json.dumps({'receipt':dest.name,'tests':4,'exit':ran.returncode,'generated_output_removed':receipt['generated_output_removed']}))
