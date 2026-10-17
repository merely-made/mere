"""Build and probe the current graph-only Mere facade, preserving shared WIP."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess

HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[2]
TARGET=Path('C:/t/cargo-targets/mere').resolve()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
inventory=json.loads((HERE/'source_inventory.json').read_text())['files']
sources={p:v['sha256'] for p,v in inventory.items() if p.startswith('crates/mere/')}
assert len(sources)==18 and all(sha(ROOT/p)==h for p,h in sources.items())
command=['cargo','--config','build.build-dir="C:/t/cargo-targets/mere/build"','build','-p','mere',
    '--no-default-features','--features','graph','--lib','--target-dir',str(TARGET),'--message-format=json']
with (HERE/'mere_graph_library_build.jsonl').open('w',encoding='utf-8') as out,(HERE/'mere_graph_library_build_stderr.log').open('w',encoding='utf-8') as err:
    built=subprocess.run(command,cwd=ROOT,stdout=out,stderr=err,timeout=300)
assert built.returncode==0
artifacts=[json.loads(line) for line in (HERE/'mere_graph_library_build.jsonl').read_text().splitlines() if line.startswith('{')]
artifact=next(x for x in artifacts if x.get('reason')=='compiler-artifact' and x['target']['name']=='mere')
library=next(Path(p) for p in artifact['filenames'] if p.endswith('.rlib'))
source=HERE/'mere_facade_probe.rs'; binary=TARGET/'authority-success-mere-facade-probe.exe'; pdb=binary.with_suffix('.pdb')
assert binary.parent==TARGET and not binary.exists() and not pdb.exists()
probe_command=['rustc','--edition=2024','--test',str(source),'-L','dependency='+str(TARGET/'build/debug/deps'),
    '--extern','mere='+str(library),'-C','debuginfo=0','-o',str(binary)]
compiled=subprocess.run(probe_command,cwd=ROOT,capture_output=True,text=True)
(HERE/'mere_facade_probe_build.log').write_text(compiled.stdout+compiled.stderr,encoding='utf-8')
assert compiled.returncode==0,compiled.stdout+compiled.stderr
digest=sha(binary)
try:
    ran=subprocess.run([str(binary),'--nocapture','--test-threads=1'],cwd=ROOT,capture_output=True,text=True,timeout=30)
    (HERE/'mere_facade_probe_output.log').write_text(ran.stdout+ran.stderr,encoding='utf-8')
finally:
    assert binary.read_bytes().startswith(b'MZ');binary.unlink()
    if pdb.exists():
        assert pdb.read_bytes().startswith(b'Microsoft C/C++ MSF');pdb.unlink()
assert all(sha(ROOT/p)==h for p,h in sources.items())
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'test_exit':ran.returncode,'tests':3,
    'harness_sha256':sha(source),'source_sha256':sources,
    'library_sha256':{'mere':{'path':str(library),'sha256':sha(library)}},
    'library_build_command':command,'library_build_exit':built.returncode,
    'probe_build_command':probe_command,'probe_build_exit':compiled.returncode,'native_binary_sha256':digest,
    'generated_output_removed':not binary.exists() and not pdb.exists(),
    'scope':'Three native public graph-only facade observations with controls: outline address collisions, equal-title relation bundle order, and corrupt subgraph sidecar replacement. No canvas, SPARQL/query, clip, workbench, renderer or product UI execution.'}
(HERE/'mere_facade_probe_receipt.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
print(ran.stdout)
assert ran.returncode==0,ran.stderr
