"""Link unchanged, hash-checked Pictograph artifacts for bounded CPU observations."""
from pathlib import Path
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
TARGET = Path('C:/t/cargo-targets/mere').resolve()
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
previous = json.loads((HERE / 'pictograph_review_receipt.json').read_text())
assert all(sha(ROOT / p) == h for p,h in previous['source_sha256'].items())
build = HERE / 'pictograph_library_build.jsonl'
assert sha(build) == previous['evidence_sha256'][build.name]
artifacts = json.loads((HERE / 'pictograph_boundary_probe_receipt.json').read_text())['library_sha256']
assert all(sha(Path(v['path'])) == v['sha256'] for v in artifacts.values())
libraries = {name:Path(artifacts[name]['path']) for name in ['pictograph','kernel']}
native = []
for line in build.read_text(encoding='utf-8-sig',errors='replace').splitlines():
    try: item = json.loads(line)
    except ValueError: continue
    if item.get('reason') == 'build-script-executed':
        native.extend(p for p in item.get('linked_paths',[]) if p.startswith('native='))
binary = TARGET / 'authority-success-pictograph-signal-probe.exe'
pdb = binary.with_suffix('.pdb')
assert binary.parent.resolve() == TARGET and not binary.exists() and not pdb.exists()
source = HERE / 'pictograph_signal_probe.rs'
command = ['rustc','--edition=2024','--test',str(source),'-L','dependency='+str(TARGET / 'build/debug/deps'),'-C','debuginfo=0','-o',str(binary)]
for name,path in libraries.items(): command.extend(['--extern',name+'='+str(path)])
for path in sorted(set(native)): command.extend(['-L',path])
built = subprocess.run(command,cwd=ROOT,capture_output=True,text=True)
(HERE / 'pictograph_signal_probe_build.log').write_text(built.stdout+built.stderr,encoding='utf-8')
assert built.returncode == 0,built.stdout+built.stderr
digest = sha(binary)
try:
    ran = subprocess.run([str(binary),'--nocapture','--test-threads=1'],cwd=ROOT,capture_output=True,text=True,timeout=60)
    (HERE / 'pictograph_signal_probe_output.log').write_text(ran.stdout+ran.stderr,encoding='utf-8')
finally:
    assert binary.read_bytes().startswith(b'MZ')
    binary.unlink()
    if pdb.exists():
        assert pdb.read_bytes().startswith(b'Microsoft C/C++ MSF')
        pdb.unlink()
paths = ['crates/canvas/pictograph/src/'+p for p in ['canvas.rs','canvas/strategy.rs','canvas/frame.rs','canvas/tests/affinity.rs','signals.rs','signals/producers.rs','signals/registry.rs','signals/importance.rs']]
receipt = {'test_exit':ran.returncode,'tests':2,'harness_sha256':sha(source),
    'source_sha256':{p:sha(ROOT / p) for p in paths},'library_sha256':{name:artifacts[name] for name in libraries},
    'library_build_log_sha256':sha(build),'build_command':command,'build_exit':built.returncode,
    'native_binary_sha256':digest,'generated_output_removed':not binary.exists() and not pdb.exists(),
    'scope':'Two generated native CPU observations with controls, using prior Cargo-built libraries verified unchanged by source/artifact hashes. No production edits, headed pixels, GPU, package suite, network or store execution.'}
(HERE / 'pictograph_signal_probe_receipt.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
print(ran.stdout)
assert ran.returncode == 0,ran.stderr
