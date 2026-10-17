"""Bounded native Canvas observations linked to fresh Cargo JSON artifacts."""
from pathlib import Path
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
TARGET = Path('C:/t/cargo-targets/mere').resolve()
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
libraries, native, finished = {}, [], False
names = {'pictograph', 'cartography', 'kernel', 'scenograph', 'curation'}
for line in (HERE / 'pictograph_library_build.jsonl').read_text(encoding='utf-8-sig', errors='replace').splitlines():
    try: item = json.loads(line)
    except ValueError: continue
    if item.get('reason') == 'build-finished': finished = item.get('success', False)
    if item.get('reason') == 'build-script-executed':
        native.extend(p for p in item.get('linked_paths', []) if p.startswith('native='))
    if item.get('reason') == 'compiler-artifact' and item['target']['name'] in names:
        files = [Path(p) for p in item['filenames'] if p.endswith('.rlib')]
        assert len(files) == 1
        libraries[item['target']['name']] = files[0]
assert finished and set(libraries) == names
binary = TARGET / 'authority-success-pictograph-boundary-probe.exe'
pdb = binary.with_suffix('.pdb')
assert binary.parent.resolve() == TARGET and not binary.exists() and not pdb.exists()
source = HERE / 'pictograph_boundary_probe.rs'
command = ['rustc', '--edition=2024', '--test', str(source), '-L', 'dependency=' + str(TARGET / 'build/debug/deps'), '-C', 'debuginfo=0', '-o', str(binary)]
for name, path in libraries.items(): command.extend(['--extern', name + '=' + str(path)])
for path in sorted(set(native)): command.extend(['-L', path])
built = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
(HERE / 'pictograph_boundary_probe_build.log').write_text(built.stdout + built.stderr, encoding='utf-8')
assert built.returncode == 0, built.stdout + built.stderr
digest = sha(binary)
ran = subprocess.run([str(binary), '--nocapture', '--test-threads=1'], cwd=ROOT, capture_output=True, text=True, timeout=60)
(HERE / 'pictograph_boundary_probe_output.log').write_text(ran.stdout + ran.stderr, encoding='utf-8')
assert binary.read_bytes().startswith(b'MZ')
binary.unlink()
if pdb.exists():
    assert pdb.read_bytes().startswith(b'Microsoft C/C++ MSF')
    pdb.unlink()
paths = ['crates/canvas/pictograph/src/' + s for s in ['canvas/cartography.rs', 'canvas/camera_host.rs', 'canvas/controls.rs', 'canvas/input.rs', 'canvas/coupling_paint.rs', 'canvas/fold_paint.rs', 'canvas/fields.rs', 'canvas/selection.rs', 'canvas/dynamics_spec.rs', 'canvas/dynamics_inspection.rs', 'canvas/relationship_topology.rs', 'canvas/lifecycle.rs', 'canvas/palette.rs', 'canvas/build.rs', 'canvas/view.rs', 'canvas/strategy.rs', 'canvas/underlay.rs']]
receipt = {'test_exit': ran.returncode, 'tests': 13, 'harness_sha256': sha(source), 'source_sha256': {p: sha(ROOT / p) for p in paths},
    'library_sha256': {n: {'path': str(p), 'sha256': sha(p)} for n, p in libraries.items()},
    'build_command': command, 'build_exit': built.returncode, 'native_binary_sha256': digest,
    'generated_output_removed': not binary.exists() and not pdb.exists(),
    'scope': 'Generated native graphs and public Canvas/paint/command APIs. Twelve diagnostics characterize eleven defects (two hide paths share one fold-routing defect); unsupported weighted-root control qualifies shared refusal. One CPU Canvas frame, no headed UI, network, storage, GPU rasterization or source writes.'}
(HERE / 'pictograph_boundary_probe_receipt.json').write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
print(ran.stdout)
assert ran.returncode == 0, ran.stderr
