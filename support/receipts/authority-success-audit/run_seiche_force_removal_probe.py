"""Build current Seiche in the reusable target and run bounded native observations."""
from pathlib import Path
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
TARGET = Path('C:/t/cargo-targets/mere').resolve()
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
inventory = json.loads((HERE / 'source_inventory.json').read_text())['files']
sources = {p: sha(ROOT / p) for p in inventory if p.startswith('crates/conatus/seiche/')}
build = ['cargo', 'build', '-p', 'seiche', '--features', 'serde', '--lib',
    '--target-dir', str(TARGET), '--config', 'build.build-dir="C:/t/cargo-targets/mere/build"',
    '--message-format=json']
log = HERE / 'seiche_library_build.jsonl'
with log.open('w', encoding='utf-8') as stdout, (HERE / 'seiche_library_build_stderr.log').open('w', encoding='utf-8') as stderr:
    built = subprocess.run(build, cwd=ROOT, stdout=stdout, stderr=stderr)
assert built.returncode == 0, 'Seiche build failed; inspect seiche_library_build_stderr.log'
libraries, native = {}, []
for line in log.read_text(encoding='utf-8-sig').splitlines():
    try:
        item = json.loads(line)
    except ValueError:
        continue
    if item.get('reason') == 'compiler-artifact' and item.get('target', {}).get('name') in ['seiche', 'euclid']:
        for p in item['filenames']:
            if p.endswith('.rlib'):
                libraries[item['target']['name']] = {'path': p, 'sha256': sha(Path(p))}
    if item.get('reason') == 'build-script-executed':
        native.extend(p for p in item.get('linked_paths', []) if p.startswith('native='))
assert set(libraries) == {'seiche', 'euclid'}
assert all(sha(ROOT / p) == v for p, v in sources.items())
binary = TARGET / 'authority-success-seiche-force-removal-probe.exe'
pdb = binary.with_suffix('.pdb')
assert binary.parent == TARGET and not binary.exists() and not pdb.exists()
source = HERE / 'seiche_force_removal_probe.rs'
command = ['rustc', '--edition=2024', '--test', str(source), '-L',
    'dependency=' + str(TARGET / 'build/debug/deps'), '-C', 'debuginfo=0', '-o', str(binary)]
for name, value in libraries.items():
    command.extend(['--extern', name + '=' + value['path']])
for path in sorted(set(native)):
    command.extend(['-L', path])
compiled = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
(HERE / 'seiche_force_removal_probe_build.log').write_text(compiled.stdout + compiled.stderr, encoding='utf-8')
assert compiled.returncode == 0, compiled.stdout + compiled.stderr
digest = sha(binary)
try:
    ran = subprocess.run([str(binary), '--nocapture', '--test-threads=1'], cwd=ROOT,
        capture_output=True, text=True, timeout=60)
    (HERE / 'seiche_force_removal_probe_output.log').write_text(ran.stdout + ran.stderr, encoding='utf-8')
finally:
    assert binary.read_bytes().startswith(b'MZ')
    binary.unlink()
    if pdb.exists():
        assert pdb.read_bytes().startswith(b'Microsoft C/C++ MSF')
        pdb.unlink()
receipt = {'test_exit': ran.returncode, 'tests': 3, 'harness_sha256': sha(source),
    'source_sha256': sources, 'library_sha256': libraries,
    'build_command': build, 'build_exit': built.returncode, 'library_build_log_sha256': sha(log),
    'probe_build_command': command, 'probe_build_exit': compiled.returncode,
    'native_binary_sha256': digest, 'generated_output_removed': not binary.exists() and not pdb.exists(),
    'scope': 'Two native persistent-force removal observations with zero-force controls, and one near-body quadtree primitive observation with a minimum-cell control. Current Seiche CPU library, serde/default actor features; no actor execution, GPU, renderer or package test suite. Production unchanged.'}
assert all(sha(ROOT / p) == value for p, value in sources.items())
(HERE / 'seiche_force_removal_probe_receipt.json').write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
print(ran.stdout)
assert ran.returncode == 0, ran.stderr
