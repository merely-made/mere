"""Link diagnostics to fresh public library artifacts, remove only our binary."""
from pathlib import Path
import hashlib
import json
import subprocess

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
TARGET=Path('C:/t/cargo-targets/mere').resolve()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
libraries,native,finished={},[],False
names={'sceno','scenomise','scenotime','serde_json'}
for line in (HERE/'scenomise_probe_library_build.jsonl').read_text(encoding='utf-8-sig',errors='replace').splitlines():
    try: item=json.loads(line)
    except ValueError: continue
    if item.get('reason')=='build-finished': finished=item.get('success',False)
    if item.get('reason')=='build-script-executed': native.extend(p for p in item.get('linked_paths',[]) if p.startswith('native='))
    if item.get('reason')=='compiler-artifact' and item['target']['name'] in names:
        files=[Path(p) for p in item['filenames'] if p.endswith('.rlib')]
        assert len(files)==1
        libraries[item['target']['name']]=files[0]
assert finished and set(libraries)==names
binary=TARGET/'authority-success-scenomise-solver-probe.exe'
pdb=binary.with_suffix('.pdb')
assert binary.parent.resolve()==TARGET and not binary.exists() and not pdb.exists()
source=HERE/'scenomise_solver_probe.rs'
command=['rustc','--edition=2024','--test',str(source),'-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0','-o',str(binary)]
for name,path in libraries.items(): command.extend(['--extern',name+'='+str(path)])
for path in sorted(set(native)): command.extend(['-L',path])
built=subprocess.run(command,cwd=ROOT,capture_output=True,text=True)
(HERE/'scenomise_solver_probe_build.log').write_text(built.stdout+built.stderr,encoding='utf-8')
assert built.returncode==0,built.stdout+built.stderr
digest=sha(binary)
ran=subprocess.run([str(binary),'--nocapture'],cwd=ROOT,capture_output=True,text=True,timeout=30)
(HERE/'scenomise_solver_probe_output.log').write_text(ran.stdout+ran.stderr,encoding='utf-8')
assert binary.read_bytes().startswith(b'MZ')
binary.unlink()
if pdb.exists():
    assert pdb.read_bytes().startswith(b'Microsoft C/C++ MSF')
    pdb.unlink()
paths=['crates/cambium/scenes/scenomise/src/'+s for s in ['catalog.rs','solve.rs','families/mod.rs','families/axial.rs','families/lsystem.rs','families/penrose.rs','families/radial.rs','families/setscape.rs']]
receipt={'test_exit':ran.returncode,'tests':8,'harness_sha256':sha(source),'source_sha256':{p:sha(ROOT/p) for p in paths},
    'library_sha256':{n:{'path':str(p),'sha256':sha(p)} for n,p in libraries.items()},
    'build_command':command,'build_exit':built.returncode,'native_binary_sha256':digest,
    'generated_output_removed':not binary.exists() and not pdb.exists(),
    'scope':'Native public API diagnostics only. Depth-255 admission is tested, hazardous expansion is not executed. No renderer/network/live editor or product persistence gate.'}
(HERE/'scenomise_solver_probe_receipt.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
print(ran.stdout)
assert ran.returncode==0,ran.stderr
