"""Bounded native observation using the actual production Pool module."""
from pathlib import Path
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
TARGET = Path('C:/t/cargo-targets/mere').resolve()
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
production = ROOT / 'crates/armillary/src/pool.rs'
before = sha(production)
source = HERE / 'armillary_pool_probe.rs'
binary = TARGET / 'authority-success-armillary-pool-probe.exe'
pdb = binary.with_suffix('.pdb')
assert binary.parent == TARGET and not binary.exists() and not pdb.exists()
command = ['rustc', '--edition=2024', '--test', str(source), '-C', 'debuginfo=0', '-o', str(binary)]
compiled = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
(HERE / 'armillary_pool_probe_build.log').write_text(compiled.stdout + compiled.stderr, encoding='utf-8')
assert compiled.returncode == 0, compiled.stdout + compiled.stderr
digest = sha(binary)
try:
    ran = subprocess.run([str(binary), 'audit_', '--nocapture', '--test-threads=1'], cwd=ROOT,
        capture_output=True, text=True, timeout=30)
    (HERE / 'armillary_pool_probe_output.log').write_text(ran.stdout + ran.stderr, encoding='utf-8')
finally:
    assert binary.read_bytes().startswith(b'MZ')
    binary.unlink()
    if pdb.exists():
        assert pdb.read_bytes().startswith(b'Microsoft C/C++ MSF')
        pdb.unlink()
assert sha(production) == before
receipt = {'test_exit': ran.returncode, 'tests': 2, 'harness_sha256': sha(source),
    'source_sha256': {'crates/armillary/src/pool.rs': before},
    'probe_build_command': command, 'probe_build_exit': compiled.returncode,
    'native_binary_sha256': digest, 'generated_output_removed': not binary.exists() and not pdb.exists(),
    'scope': 'Public Pool.submit with long-lived jobs; bounded attempts to observe idle-worker starvation and a busy-worker control. Both jobs released and finished each attempt. Original package tests are filtered out. No product host, shutdown or panic execution.'}
(HERE / 'armillary_pool_probe_receipt.json').write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
print(ran.stdout)
assert ran.returncode == 0, ran.stderr
