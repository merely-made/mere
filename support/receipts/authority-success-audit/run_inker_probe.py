"""Run source-preserving observations against the exact current Cargo library."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
inventory=json.loads((HERE/'source_inventory.json').read_text())['files']
sources={p:v['sha256'] for p,v in inventory.items() if p.startswith('crates/inker/inker/')}
assert len(sources)==23 and all(sha(ROOT/p)==h for p,h in sources.items())
rows=[json.loads(line) for line in (HERE/'inker_library_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if line.startswith('{')]
assert any(r.get('reason')=='build-finished' and r.get('success') for r in rows)
record=next(r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']=='inker')
path=next(Path(p) for p in record['filenames'] if p.endswith('.rlib'))
libs={'inker':{'path':str(path),'sha256':sha(path),'features':record['features'],'package_id':record['package_id']}}
binary=TARGET/'authority-success-inker-probe.exe';pdb=binary.with_suffix('.pdb')
assert not binary.exists() and not pdb.exists()
command=['rustc','--edition=2024','--test',str(HERE/'inker_probe.rs'),'-L','dependency='+str(TARGET/'build/debug/deps'),
    '-C','debuginfo=0','--extern','inker='+str(path),'-o',str(binary)]
compiled=subprocess.run(command,cwd=ROOT,capture_output=True,text=True)
(HERE/'inker_probe_build.log').write_text(compiled.stdout+compiled.stderr,encoding='utf-8');assert compiled.returncode==0,compiled.stderr
digest=sha(binary)
try:
    ran=subprocess.run([str(binary),'--nocapture','--test-threads=1'],cwd=ROOT,capture_output=True,text=True,timeout=60)
    (HERE/'inker_probe_output.log').write_text(ran.stdout+ran.stderr,encoding='utf-8');print(ran.stdout)
finally:
    assert binary.read_bytes().startswith(b'MZ');binary.unlink()
    if pdb.exists():assert pdb.read_bytes().startswith(b'Microsoft C/C++ MSF');pdb.unlink()
assert ran.returncode==0,ran.stderr
assert all(sha(ROOT/p)==h for p,h in sources.items())
assert sha(path)==libs['inker']['sha256']
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'tests':9,'test_exit':ran.returncode,
    'source_sha256':sources,'libraries':libs,'probe_build_command':command,'probe_build_exit':compiled.returncode,
    'native_binary_sha256':digest,'generated_output_removed':not binary.exists() and not pdb.exists(),
    'scope':'Nine current default Cargo-built native library observations with controls. Serialization collisions demonstrate non-injectivity without a parser reparse. Mock flip and Scry surfaces only. No network, browser, host, GPU, OS accessibility or optional genet-donor execution; release warning suppression is source-qualified, not release-built.'}
(HERE/'inker_probe_receipt.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
