"""Link geometry observations to the verified current Seiche library."""
from pathlib import Path
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
TARGET = Path('C:/t/cargo-targets/mere').resolve()
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
previous = json.loads((HERE / 'seiche_force_removal_probe_receipt.json').read_text())
assert previous['test_exit'] == 0
assert all(sha(ROOT / p) == h for p, h in previous['source_sha256'].items())
libraries = previous['library_sha256']
assert all(sha(Path(v['path'])) == v['sha256'] for v in libraries.values())
build = HERE / 'seiche_library_build.jsonl'
assert sha(build) == previous['library_build_log_sha256']
native = []
for line in build.read_text(encoding='utf-8-sig').splitlines():
    try:
        item = json.loads(line)
    except ValueError:
        continue
    if item.get('reason') == 'build-script-executed':
        native.extend(p for p in item.get('linked_paths', []) if p.startswith('native='))
binary = TARGET / 'authority-success-seiche-geometry-probe.exe'
pdb = binary.with_suffix('.pdb')
assert binary.parent == TARGET and not binary.exists() and not pdb.exists()
source = HERE / 'seiche_geometry_probe.rs'
command = ['rustc', '--edition=2024', '--test', str(source), '-L',
    'dependency=' + str(TARGET / 'build/debug/deps'), '-C', 'debuginfo=0', '-o', str(binary)]
for name, value in libraries.items():
    command.extend(['--extern', name + '=' + value['path']])
for path in sorted(set(native)):
    command.extend(['-L', path])
compiled = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
(HERE / 'seiche_geometry_probe_build.log').write_text(compiled.stdout + compiled.stderr, encoding='utf-8')
assert compiled.returncode == 0, compiled.stdout + compiled.stderr
digest = sha(binary)
try:
    ran = subprocess.run([str(binary), '--nocapture', '--test-threads=1'], cwd=ROOT,
        capture_output=True, text=True, timeout=60)
    (HERE / 'seiche_geometry_probe_output.log').write_text(ran.stdout + ran.stderr, encoding='utf-8')
finally:
    assert binary.read_bytes().startswith(b'MZ')
    binary.unlink()
    if pdb.exists():
        assert pdb.read_bytes().startswith(b'Microsoft C/C++ MSF')
        pdb.unlink()
receipt = {'test_exit': ran.returncode, 'tests': 2, 'harness_sha256': sha(source),
    'source_sha256': previous['source_sha256'], 'library_sha256': libraries,
    'library_build_log_sha256': sha(build), 'probe_build_command': command,
    'probe_build_exit': compiled.returncode, 'native_binary_sha256': digest,
    'generated_output_removed': not binary.exists() and not pdb.exists(),
    'scope': 'Native CPU public collider admission with panic capture and Ball/Square controls; fluid tick with resized pinned ball and size/tangibility controls. No renderer, GPU, actor execution or package suite. Production unchanged.'}
(HERE / 'seiche_geometry_probe_receipt.json').write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
print(ran.stdout)
assert ran.returncode == 0, ran.stderr
