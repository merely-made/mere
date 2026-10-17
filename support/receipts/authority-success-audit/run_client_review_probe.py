"""Bounded native public client diagnostics from a fresh production lib build."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
TARGET = Path('C:/t/cargo-targets/mere').resolve()
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
libraries, native, finished = {}, [], False
names = {'graphshell_client', 'chirograph', 'sceno', 'scenotime', 'serde_json'}
for line in (HERE/'graphshell_client_build_output.jsonl').read_text(encoding='utf-8-sig', errors='replace').splitlines():
    try: item = json.loads(line)
    except ValueError: continue
    if item.get('reason') == 'build-finished': finished = item.get('success', False)
    if item.get('reason') == 'build-script-executed': native.extend(p for p in item.get('linked_paths',[]) if p.startswith('native='))
    if item.get('reason') == 'compiler-artifact' and item['target']['name'] in names:
        paths = [Path(p) for p in item['filenames'] if p.endswith('.rlib')]
        assert len(paths) == 1
        libraries[item['target']['name']] = paths[0]
assert finished and set(libraries) == names
# For a primary package Cargo reports the top-level lib copy; dependencies and
# crate metadata live under the configured build-dir, not beside that copy.
dep = TARGET/'build/debug/deps'
assert dep.is_dir()
binary = TARGET/'authority-success-client-review-probe.exe'
pdb = binary.with_suffix('.pdb')
assert binary.parent.resolve() == TARGET and not binary.exists() and not pdb.exists()
source = HERE/'client_review_probe.rs'
command = ['rustc','--edition=2024','--test',str(source),'-L','dependency='+str(dep),'-C','debuginfo=0','-o',str(binary)]
for name,path in libraries.items(): command.extend(['--extern',name+'='+str(path)])
for path in sorted(set(native)): command.extend(['-L',path])
built = subprocess.run(command,cwd=ROOT,capture_output=True,text=True)
(HERE/'client_review_probe_build_output.txt').write_text(built.stdout+built.stderr,encoding='utf-8')
assert built.returncode == 0, built.stdout+built.stderr
binary_sha = sha(binary)
tested = subprocess.run([str(binary),'--nocapture'],cwd=ROOT,capture_output=True,text=True,timeout=30)
(HERE/'client_review_probe_output.txt').write_text(tested.stdout+tested.stderr,encoding='utf-8')
assert binary.read_bytes().startswith(b'MZ')
binary.unlink()
if pdb.exists():
    assert pdb.read_bytes().startswith(b'Microsoft C/C++ MSF')
    pdb.unlink()
paths = ['crates/graphshell/graphshell-client/src/'+p for p in ['core.rs','driver.rs','fold.rs','frozen.rs','lib.rs','pump.rs','remote.rs','session.rs']]
receipt = {'date_utc':datetime.now(timezone.utc).isoformat(), 'scope':'Current public native client core, state, frozen and remote lifecycle APIs on generated validated snapshots/replies. No network, browser, live accessibility stack or installed resident executed.',
 'build_command':command,'build_exit':built.returncode,'test_exit':tested.returncode,'tests':8,
 'source_sha256':{p:sha(ROOT/p) for p in paths}, 'library_sha256':{n:{'path':str(p),'sha256':sha(p)} for n,p in libraries.items()},
 'harness_sha256':sha(source),'native_binary_sha256':binary_sha,'generated_native_output_removed':not binary.exists() and not pdb.exists()}
(HERE/'client_review_probe_receipt.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
print(tested.stdout)
assert tested.returncode == 0, tested.stderr
print(json.dumps({'test_exit':tested.returncode,'generated_output_removed':receipt['generated_native_output_removed']}))
