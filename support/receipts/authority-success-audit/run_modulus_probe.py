"""Capture current Modulus tests and two bounded public-API defect observations."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,subprocess

HERE=Path(__file__).resolve().parent; ROOT=HERE.parents[2]
TARGET=Path('C:/t/cargo-targets/mere').resolve()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
inventory=json.loads((HERE/'source_inventory.json').read_text())['files']
sources={p:v['sha256'] for p,v in inventory.items() if p.startswith('crates/conatus/modulus/')}
assert len(sources)==12 and all(sha(ROOT/p)==h for p,h in sources.items())
base=['cargo','--config','build.build-dir="C:/t/cargo-targets/mere/build"']
build_command=base+['build','-p','modulus','--lib','--target-dir',str(TARGET),'--message-format=json']
with (HERE/'modulus_library_build.jsonl').open('w',encoding='utf-8') as out,(HERE/'modulus_library_build_stderr.log').open('w',encoding='utf-8') as err:
    built=subprocess.run(build_command,cwd=ROOT,stdout=out,stderr=err,timeout=180)
assert built.returncode==0
artifacts=[json.loads(line) for line in (HERE/'modulus_library_build.jsonl').read_text().splitlines() if line.startswith('{')]
artifact=next(x for x in artifacts if x.get('reason')=='compiler-artifact' and x['target']['name']=='modulus')
library=next(Path(p) for p in artifact['filenames'] if p.endswith('.rlib'))
suite_command=base+['test','-p','modulus','--target-dir',str(TARGET),'--','--test-threads=1']
with (HERE/'modulus_default_suite.log').open('w',encoding='utf-8') as out:
    suite=subprocess.run(suite_command,cwd=ROOT,stdout=out,stderr=subprocess.STDOUT,timeout=300)
assert suite.returncode==0
source=HERE/'modulus_probe.rs'; binary=TARGET/'authority-success-modulus-probe.exe'; pdb=binary.with_suffix('.pdb')
assert binary.parent==TARGET and not binary.exists() and not pdb.exists()
command=['rustc','--edition=2024','--test',str(source),'-L','dependency='+str(TARGET/'build/debug/deps'),
    '--extern','modulus='+str(library),'-C','debuginfo=0','-o',str(binary)]
compiled=subprocess.run(command,cwd=ROOT,capture_output=True,text=True)
(HERE/'modulus_probe_build.log').write_text(compiled.stdout+compiled.stderr,encoding='utf-8')
assert compiled.returncode==0,compiled.stdout+compiled.stderr
digest=sha(binary)
try:
    ran=subprocess.run([str(binary),'--nocapture','--test-threads=1'],cwd=ROOT,capture_output=True,text=True,timeout=30)
    (HERE/'modulus_probe_output.log').write_text(ran.stdout+ran.stderr,encoding='utf-8')
finally:
    assert binary.read_bytes().startswith(b'MZ'); binary.unlink()
    if pdb.exists():
        assert pdb.read_bytes().startswith(b'Microsoft C/C++ MSF'); pdb.unlink()
assert all(sha(ROOT/p)==h for p,h in sources.items())
passed=[int(n) for n in re.findall(r'test result: ok\. (\d+) passed;', (HERE/'modulus_default_suite.log').read_text())]
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'test_exit':ran.returncode,'tests':2,
    'harness_sha256':sha(source),'source_sha256':sources,
    'library_sha256':{'modulus':{'path':str(library),'sha256':sha(library)}},
    'library_build_command':build_command,'library_build_exit':built.returncode,
    'suite_command':suite_command,'suite_exit':suite.returncode,'suite_passed_counts':passed,
    'probe_build_command':command,'probe_build_exit':compiled.returncode,'native_binary_sha256':digest,
    'generated_output_removed':not binary.exists() and not pdb.exists(),
    'scope':'Default Modulus Cargo unit/integration/doc gate, excluding ignored receipt, and two CPU public-API observations with controls. WGSL source read; no shader compilation or GPU execution.'}
(HERE/'modulus_probe_receipt.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
print((HERE/'modulus_default_suite.log').read_text()[-1800:]); print(ran.stdout)
assert ran.returncode==0,ran.stderr
