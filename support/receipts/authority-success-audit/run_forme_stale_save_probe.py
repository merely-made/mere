"""Three product/transfer checks against the successful current library build."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,struct
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
dest=HERE/'forme_stale_save_probe_receipt.json';assert not dest.exists()
sources={n:sha(ROOT/n) for n in ['ports/graphshell/src/forme_session.rs','ports/graphshell/src/forme_workspace.rs','ports/graphshell/src/presentation_editor.rs']}
qualified=json.loads((HERE/'product_transfer_probe_receipt.json').read_text())
libs=qualified['libraries'];assert all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
rows=[json.loads(l) for l in (HERE/'capture_graphshell_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')]
binary=TARGET/'authority-success-forme-stale-save-probe.exe';assert not list(TARGET.glob(binary.stem+'.*'))
command=['rustc','--edition=2024','--test',str(HERE/'forme_stale_save_probe.rs'),'-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0','-o',str(binary)]
for name,lib in libs.items():command+=['--extern',name+'='+lib['path']]
native=sorted({p for r in rows if r.get('reason')=='build-script-executed' for p in r.get('linked_paths',[])})
for p in native:command+=['-L',p]
built=subprocess.run(command,cwd=ROOT,capture_output=True,text=True)
(HERE/'forme_stale_save_probe_compile.log').write_text(built.stdout+built.stderr,encoding='utf-8');assert built.returncode==0,built.stderr
digest=sha(binary);removed=[]
try:
 ran=subprocess.run([str(binary),'--nocapture','--test-threads=1'],cwd=ROOT,capture_output=True,text=True,timeout=45)
 (HERE/'forme_stale_save_probe_output.log').write_text(ran.stdout+ran.stderr,encoding='utf-8');print(ran.stdout)
finally:
 for p in sorted(TARGET.glob(binary.stem+'.*')):
  assert p.resolve().parent==TARGET;data=p.read_bytes()
  if p.suffix=='.exe':assert data.startswith(b'MZ')
  elif p.suffix=='.pdb':assert data.startswith(b'Microsoft C/C++ MSF')
  elif p.suffix=='.lib':assert data.startswith(b'!<arch>\n') and binary.name.encode() in data
  elif p.suffix=='.exp':assert data[:2]==b'\x64\x86' and 0<struct.unpack_from('<H',data,2)[0]<128 and binary.name.encode() in data
  else:raise AssertionError('unknown generated output '+str(p))
  removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
assert ran.returncode==0,ran.stderr
assert all(sha(ROOT/n)==h for n,h in sources.items());assert all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'source_sha256':sources,'libraries':libs,'native_search_paths':native,'probe_build_command':command,'probe_build_exit':built.returncode,'native_binary_sha256':digest,'test_exit':ran.returncode,'tests':2,'removed_outputs':removed,'generated_output_removed':not list(TARGET.glob(binary.stem+'.*')),'scope':'Actual FormeSession, FormeWorkspace and PresentationSession public APIs with independent sessions on one synthetic MemoryBackend slot. Exact qualified libraries from product_transfer_probe_receipt.json, freshly built earlier in this audit at unchanged c1b5d471 source. Demonstrates stale Forme overwrite and working transactional rule-workspace refusal control. No browser/multitab, physical disk, installed profile or concurrent process trial.'}

dest.write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8');print(json.dumps({'receipt':dest.name,'tests':2,'exit':ran.returncode,'generated_output_removed':receipt['generated_output_removed']}))
