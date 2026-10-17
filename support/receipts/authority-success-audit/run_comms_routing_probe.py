"""Compile unchanged portable Comms core, original core tests and routing cases."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, os, re, subprocess, tomllib
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
TARGET = Path('C:/t/cargo-targets/mere').resolve(); DEBUG = TARGET / 'build/debug'
NAME = 'comms_routing_probe'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8-sig'))
assert not (HERE / (NAME + '_receipt.json')).exists()
prior = load('gemot_owner_probe_receipt.json')
meta = Path(prior['libraries']['muniment']['metadata_path'])
assert sha(meta) == prior['libraries']['muniment']['metadata_sha256']
deps = {r[1]: r[3] for r in json.loads(meta.read_text())['deps']}
def supplier(name, folder_prefix, proc=False):
    choices = []
    for p in (DEBUG / '.fingerprint').glob(folder_prefix + '-*/lib-' + name + '.json'):
        marker = p.with_suffix(''); suffix = p.parent.name.rsplit('-', 1)[1]
        artifact = DEBUG / 'deps' / ((name + '-' + suffix + '.dll') if proc else ('lib' + name + '-' + suffix + '.rlib'))
        if not marker.is_file() or not artifact.is_file(): continue
        value = int.from_bytes(bytes.fromhex(marker.read_text().strip()), 'little')
        if value == deps[name]: choices.append({'path': str(artifact), 'sha256': sha(artifact), 'fingerprint_value': value,
            'metadata_path': str(p), 'metadata_sha256': sha(p)})
    assert len(choices) == 1, (name, len(choices))
    return choices[0]
libs = {'serde': supplier('serde', 'serde'), 'async_trait': supplier('async_trait', 'async-trait', True),
        'serde_json': supplier('serde_json', 'serde_json'), 'tokio': load('fetch_probe_receipt.json')['libraries']['tokio']}
assert all(sha(Path(v['path'])) == v['sha256'] for v in libs.values())
files = [p for p in load('source_inventory.json')['files'] if p.startswith('crates/shell/comms/')]
sources = {p: sha(ROOT / p) for p in files}; assert len(files) == 10
binary = TARGET / 'authority-success-comms-routing.exe'; unit = TARGET / 'authority-success-comms-unit.exe'
library = TARGET / 'libauthority_success_comms_core.rlib'
assert not library.exists() and not list(TARGET.glob(binary.stem + '.*')) and not list(TARGET.glob(unit.stem + '.*'))
common = ['-L', 'dependency=' + str(DEBUG / 'deps'), '-C', 'debuginfo=0']
externs = lambda names: [s for n in names for s in ['--extern', n + '=' + libs[n]['path']]]
owner_cmd = ['rustc', '--edition=2024', '--crate-name', 'comms_audit', '--crate-type', 'rlib', '--cfg', 'feature="test-support"',
    str(ROOT / 'crates/shell/comms/src/lib.rs'), *common, *externs(['serde', 'async_trait']), '-o', str(library)]
unit_cmd = ['rustc', '--edition=2024', '--test', '--crate-name', 'comms_audit_tests', str(ROOT / 'crates/shell/comms/src/lib.rs'),
    *common, *externs(['serde', 'async_trait', 'serde_json', 'tokio']), '-o', str(unit)]
consumer_cmd = ['rustc', '--edition=2024', str(HERE / (NAME + '.rs')), *common, '--extern', 'comms_audit=' + str(library), '-o', str(binary)]
env = dict(os.environ); env['CARGO_PKG_VERSION'] = tomllib.loads((ROOT / 'Cargo.toml').read_text(encoding='utf-8-sig'))['workspace']['package']['version']
results = {}; removed = []; tests = checks = 0
def run(label, command, seconds=60):
    r = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True, timeout=seconds)
    with (HERE / (NAME + '_' + label + '.log')).open('x', encoding='utf-8') as f: f.write(r.stdout + r.stderr)
    results[label] = {'command': command, 'exit': r.returncode}
    if r.returncode: print(r.stdout + r.stderr)
    return r
try:
    owner = run('owner_compile', owner_cmd)
    if owner.returncode == 0:
        test_build = run('unit_compile', unit_cmd)
        if test_build.returncode == 0:
            test_run = run('unit_output', [str(unit), '--test-threads=1'])
            if test_run.returncode == 0: tests = int(re.search(r'test result: ok\. (\d+) passed;', test_run.stdout)[1])
        consumer = run('consumer_compile', consumer_cmd)
        if consumer.returncode == 0:
            outcome = run('output', [str(binary)], 30); print(outcome.stdout + outcome.stderr)
            if outcome.returncode == 0: checks = sum(s.startswith('PASS ') for s in outcome.stdout.splitlines())
finally:
    for p in sorted(TARGET.glob(binary.stem + '.*')) + sorted(TARGET.glob(unit.stem + '.*')) + ([library] if library.exists() else []):
        assert p.resolve().parent == TARGET; data = p.read_bytes()
        assert ((p.suffix == '.exe' and data.startswith(b'MZ')) or (p.suffix == '.pdb' and data.startswith(b'Microsoft C/C++ MSF'))
            or (p.suffix in ['.lib', '.rlib'] and data.startswith(b'!<arch>\n')) or (p.suffix in ['.o', '.exp'] and data.startswith(b'\x64\x86')))
        removed.append({'path': str(p), 'bytes': len(data), 'sha256': sha(p)}); p.unlink()
assert all(sha(ROOT / p) == h for p, h in sources.items())
assert all(sha(Path(v['path'])) == v['sha256'] for v in libs.values())
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
    'source_sha256': sources, 'libraries': libs, 'supplier_metadata': {'path': str(meta), 'sha256': sha(meta)}, 'results': results,
    'CARGO_PKG_VERSION': env['CARGO_PKG_VERSION'], 'original_core_tests': tests, 'audit_cases': checks,
    'runner_sha256': sha(Path(__file__)), 'probe_sha256': sha(HERE / (NAME + '.rs')), 'removed_outputs': removed,
    'generated_output_removed': not library.exists() and not list(TARGET.glob(binary.stem + '.*')) and not list(TARGET.glob(unit.stem + '.*')),
    'qualification': 'Unchanged complete portable Comms core freshly compiled with test-support and two exact retained direct suppliers. Original core tests additionally use exact retained Tokio/serde_json suppliers; optional Misfin/Murm adapters are source-reviewed but disabled and unexecuted. Two public routing cases use the supplied InMemoryAdapter, including its deterministic fake-send IDs and immediate ready futures; no real message authoring/delivery, external account, live inbox, network, hardware or fresh Cargo dependency graph.'}
with (HERE / (NAME + '_receipt.json')).open('x', encoding='utf-8') as f: json.dump(receipt, f, indent=2); f.write('\n')
print(json.dumps({'exits': {k: v['exit'] for k, v in results.items()}, 'original_core_tests': tests, 'audit_cases': checks, 'generated_output_removed': receipt['generated_output_removed']}))
assert len(results) == 5 and all(v['exit'] == 0 for v in results.values()) and tests > 0 and checks == 2
