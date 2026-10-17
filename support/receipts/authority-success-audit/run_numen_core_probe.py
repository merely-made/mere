"""Build current default Numen, run its library suite and link native observations."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
TARGET = Path('C:/t/cargo-targets/mere').resolve()
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
inventory = json.loads((HERE/'source_inventory.json').read_text())['files']
sources = {p: v['sha256'] for p,v in inventory.items() if p.startswith('crates/conatus/numen/')}
assert len(sources) == 16 and all(sha(ROOT/p) == h for p,h in sources.items())
base = ['cargo', '--config', 'build.build-dir="C:/t/cargo-targets/mere/build"']
build_command = base + ['build', '-p', 'numen', '--lib', '--target-dir', str(TARGET), '--message-format=json']
with (HERE/'numen_library_build.jsonl').open('w',encoding='utf-8') as out, (HERE/'numen_library_build_stderr.log').open('w',encoding='utf-8') as err:
    built = subprocess.run(build_command, cwd=ROOT, stdout=out, stderr=err, timeout=300)
assert built.returncode == 0
artifacts = [json.loads(line) for line in (HERE/'numen_library_build.jsonl').read_text().splitlines() if line.startswith('{')]
numen = next(x for x in artifacts if x.get('reason')=='compiler-artifact' and x['target']['name']=='numen')
library = next(Path(p) for p in numen['filenames'] if p.endswith('.rlib'))
suite_command = base + ['test', '-p', 'numen', '--lib', '--target-dir', str(TARGET), '--', '--test-threads=1']
with (HERE/'numen_default_suite.log').open('w',encoding='utf-8') as out:
    suite = subprocess.run(suite_command, cwd=ROOT, stdout=out, stderr=subprocess.STDOUT, timeout=300)
assert suite.returncode == 0
binary = TARGET/'authority-success-numen-core-probe.exe'
pdb = binary.with_suffix('.pdb')
assert binary.parent == TARGET and not binary.exists() and not pdb.exists()
source = HERE/'numen_core_probe.rs'
command = ['rustc','--edition=2024','--test',str(source),'-L','dependency='+str(TARGET/'build/debug/deps'),
    '--extern','numen='+str(library),'-C','debuginfo=0','-o',str(binary)]
compiled = subprocess.run(command,cwd=ROOT,capture_output=True,text=True)
(HERE/'numen_core_probe_build.log').write_text(compiled.stdout+compiled.stderr,encoding='utf-8')
assert compiled.returncode == 0, compiled.stdout+compiled.stderr
digest = sha(binary)
try:
    ran = subprocess.run([str(binary),'--nocapture','--test-threads=1'],cwd=ROOT,capture_output=True,text=True,timeout=30)
    (HERE/'numen_core_probe_output.log').write_text(ran.stdout+ran.stderr,encoding='utf-8')
finally:
    assert binary.read_bytes().startswith(b'MZ')
    binary.unlink()
    if pdb.exists():
        assert pdb.read_bytes().startswith(b'Microsoft C/C++ MSF')
        pdb.unlink()
assert all(sha(ROOT/p) == h for p,h in sources.items())
receipt = {'date_utc':datetime.now(timezone.utc).isoformat(), 'test_exit':ran.returncode,'tests':3,
    'harness_sha256':sha(source),'source_sha256':sources,
    'library_sha256':{'numen':{'path':str(library),'sha256':sha(library)}},
    'library_build_command':build_command,'library_build_exit':built.returncode,
    'suite_command':suite_command,'suite_exit':suite.returncode,
    'probe_build_command':command,'probe_build_exit':compiled.returncode,
    'native_binary_sha256':digest,'generated_output_removed':not binary.exists() and not pdb.exists(),
    'scope':'Current default-feature library suite plus three native defect observations with constant-parameter and identity controls. No Burn, Rhai, GPU, renderer or hosted consumer execution.'}
(HERE/'numen_core_probe_receipt.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
print((HERE/'numen_default_suite.log').read_text(encoding='utf-8')[-1500:])
print(ran.stdout)
assert ran.returncode == 0, ran.stderr
