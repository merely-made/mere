"""Fresh complete Servitor, original unit tests, and public diagnostic consumer."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, re, subprocess
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
TARGET = Path('C:/t/cargo-targets/mere').resolve()
DEBUG = TARGET / 'build/debug'
NAME = 'servitor_owner_probe'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8-sig'))
assert not (HERE / (NAME + '_receipt.json')).exists()
prior = load('gemot_owner_probe_receipt.json')
family = Path(prior['libraries']['servitor']['metadata_path'])
assert sha(family) == prior['libraries']['servitor']['metadata_sha256']
deps = {r[1]: r[3] for r in json.loads(family.read_text())['deps']}
libs = {n: prior['libraries'][n] for n in ['insigne', 'identity']}
assert all(v['fingerprint_value'] == deps[n] and sha(Path(v['path'])) == v['sha256'] for n, v in libs.items())
choices = []
for meta in (DEBUG / '.fingerprint').glob('chartulary-*/lib-chartulary.json'):
    marker = meta.with_suffix('')
    artifact = DEBUG / 'deps' / ('libchartulary-' + meta.parent.name.rsplit('-', 1)[1] + '.rlib')
    if not marker.is_file() or not artifact.is_file(): continue
    value = int.from_bytes(bytes.fromhex(marker.read_text().strip()), 'little')
    if value == deps['chartulary']:
        choices.append({'path': str(artifact), 'sha256': sha(artifact), 'fingerprint_value': value,
                        'metadata_path': str(meta), 'metadata_sha256': sha(meta)})
assert len(choices) == 1
libs['chartulary'] = choices[0]
files = [p for p in load('source_inventory.json')['files'] if p.startswith('crates/servitor/')]
sources = {p: sha(ROOT / p) for p in files}; assert len(files) == 12
rows = [json.loads(s) for s in (HERE / 'djinn_settings_probe_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if s.startswith('{')]
native = sorted({p for r in rows if r.get('reason') == 'build-script-executed' for p in r.get('linked_paths', [])})
binary = TARGET / 'authority-success-servitor-owner.exe'
unit = TARGET / 'authority-success-servitor-unit.exe'
library = TARGET / 'libauthority_success_servitor_owner.rlib'
assert not library.exists() and not list(TARGET.glob(binary.stem + '.*')) and not list(TARGET.glob(unit.stem + '.*'))
common = ['-L', 'dependency=' + str(DEBUG / 'deps'), '-C', 'debuginfo=0']
for p in native: common += ['-L', p]
externs = [v for n, lib in libs.items() for v in ['--extern', n + '=' + lib['path']]]
owner_cmd = ['rustc', '--edition=2024', '--crate-name', 'servitor_audit', '--crate-type', 'rlib',
             str(ROOT / 'crates/servitor/src/lib.rs'), *common, *externs, '-o', str(library)]
unit_cmd = ['rustc', '--edition=2024', '--test', '--crate-name', 'servitor_audit_tests',
            str(ROOT / 'crates/servitor/src/lib.rs'), *common, *externs, '-o', str(unit)]
consumer_cmd = ['rustc', '--edition=2024', str(HERE / (NAME + '.rs')), *common, *externs,
                '--extern', 'servitor_audit=' + str(library), '-o', str(binary)]
results = {}; removed = []
def execute(label, command, seconds=60):
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=seconds)
    with (HERE / (NAME + '_' + label + '.log')).open('x', encoding='utf-8') as f: f.write(result.stdout + result.stderr)
    results[label] = {'command': command, 'exit': result.returncode}
    if result.returncode: print(result.stdout + result.stderr)
    return result
original_tests = checks = 0
try:
    owner = execute('owner_compile', owner_cmd)
    if owner.returncode == 0:
        unit_build = execute('unit_compile', unit_cmd)
        if unit_build.returncode == 0:
            unit_run = execute('unit_output', [str(unit), '--test-threads=1'])
            if unit_run.returncode == 0:
                original_tests = int(re.search(r'test result: ok\. (\d+) passed;', unit_run.stdout)[1])
                print('Original Servitor unit tests passed:', original_tests)
        consumer = execute('consumer_compile', consumer_cmd)
        if consumer.returncode == 0:
            run = execute('output', [str(binary)], 30)
            print(run.stdout + run.stderr)
            if run.returncode == 0: checks = sum(s.startswith('PASS ') for s in run.stdout.splitlines())
finally:
    owned = sorted(TARGET.glob(binary.stem + '.*')) + sorted(TARGET.glob(unit.stem + '.*')) + ([library] if library.exists() else [])
    for p in owned:
        assert p.resolve().parent == TARGET
        data = p.read_bytes()
        assert ((p.suffix == '.exe' and data.startswith(b'MZ')) or
                (p.suffix == '.pdb' and data.startswith(b'Microsoft C/C++ MSF')) or
                (p.suffix in ['.lib', '.rlib'] and data.startswith(b'!<arch>\n')) or
                (p.suffix in ['.o', '.exp'] and data.startswith(b'\x64\x86')))
        removed.append({'path': str(p), 'bytes': len(data), 'sha256': sha(p)}); p.unlink()
assert all(sha(ROOT / p) == h for p, h in sources.items())
assert all(sha(Path(v['path'])) == v['sha256'] for v in libs.values())
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(),
    'basis_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
    'source_sha256': sources, 'libraries': libs, 'supplier_metadata': {'path': str(family), 'sha256': sha(family)},
    'native_search_paths': native, 'results': results, 'original_tests': original_tests, 'audit_cases': checks,
    'runner_sha256': sha(Path(__file__)), 'probe_sha256': sha(HERE / (NAME + '.rs')),
    'removed_outputs': removed, 'generated_output_removed': not library.exists() and not list(TARGET.glob(binary.stem + '.*')) and not list(TARGET.glob(unit.stem + '.*')),
    'qualification': 'Complete unchanged current Servitor freshly compiled directly, with all original source unit tests and a separate public consumer over three exact retained Cargo-fingerprint-matched suppliers. The signed namespace case uses actual identity signatures, shared certificate check and Gate/GraphLog commit. Subject collision case uses two distinct raw public Subject values with equal first four bytes, not generated colliding cryptographic keys or identity authentication. Watch cases are in-memory public APIs with explicit clock and labels. No fresh Cargo dependency graph, live consumer/host, network, product scheduling, physical trial or persisted migration.'}
with (HERE / (NAME + '_receipt.json')).open('x', encoding='utf-8') as f:
    json.dump(receipt, f, indent=2); f.write('\n')
print(json.dumps({'results': results, 'original_tests': original_tests, 'audit_cases': checks, 'generated_output_removed': receipt['generated_output_removed']}, indent=2))
assert len(results) == 5 and all(v['exit'] == 0 for v in results.values()) and original_tests > 0 and checks == 4
