"""Run feature-enabled document-lanes CPU observations without source mutations."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
inventory=json.loads((HERE/'source_inventory.json').read_text())['files']
sources={p:v['sha256'] for p,v in inventory.items() if p.startswith('crates/system/document-lanes/')}
assert len(sources)==12 and all(sha(ROOT/p)==h for p,h in sources.items())
rows=[json.loads(line) for line in (HERE/'document_lanes_library_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if line.startswith('{')]
assert any(r.get('reason')=='build-finished' and r.get('success') for r in rows)
libs={}
for name in ['mere_document_lanes','inker','document_canvas','genet_host_api','tokio']:
    record=next(r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']==name)
    path=next(Path(p) for p in record['filenames'] if p.endswith('.rlib'))
    libs[name]={'path':str(path),'sha256':sha(path),'features':record['features'],'package_id':record['package_id']}
binary=TARGET/'authority-success-document-lanes-probe.exe';pdb=binary.with_suffix('.pdb')
assert not binary.exists() and not pdb.exists()
command=['rustc','--edition=2024','--test',str(HERE/'document_lanes_probe.rs'),'-L','dependency='+str(TARGET/'build/debug/deps'),
    '-C','debuginfo=0','-o',str(binary)]
for name,info in libs.items():command+=['--extern',name+'='+info['path']]
native_paths=sorted({p for r in rows if r.get('reason')=='build-script-executed' for p in r.get('linked_paths',[])})
for p in native_paths:command+=['-L',p]
compiled=subprocess.run(command,cwd=ROOT,capture_output=True,text=True)
(HERE/'document_lanes_probe_build.log').write_text(compiled.stdout+compiled.stderr,encoding='utf-8');assert compiled.returncode==0,compiled.stderr
digest=sha(binary)
try:
    ran=subprocess.run([str(binary),'--nocapture','--test-threads=1'],cwd=ROOT,capture_output=True,text=True,timeout=60)
    (HERE/'document_lanes_probe_output.log').write_text(ran.stdout+ran.stderr,encoding='utf-8');print(ran.stdout)
finally:
    assert binary.read_bytes().startswith(b'MZ');binary.unlink()
    if pdb.exists():assert pdb.read_bytes().startswith(b'Microsoft C/C++ MSF');pdb.unlink()
assert ran.returncode==0,ran.stderr
assert all(sha(ROOT/p)==h for p,h in sources.items())
assert all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'tests':7,'test_exit':ran.returncode,
    'source_sha256':sources,'libraries':libs,'native_search_paths':native_paths,'probe_build_command':command,'probe_build_exit':compiled.returncode,
    'native_binary_sha256':digest,'generated_output_removed':not binary.exists() and not pdb.exists(),
    'scope':'Seven current feature-enabled Cargo-built native CPU observations. Mock fetchers, retained layout/session/semantic actions and expected Tokio nesting panic before polling HTTP future. No real network response, GPU rendering, browser or OS accessibility.'}
(HERE/'document_lanes_probe_receipt.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
