"""Compile whole current Stickleback over its exact retained dependency family."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess
HERE = Path(__file__).resolve().parent; ROOT = HERE.parents[2]
TARGET = Path('C:/t/cargo-targets/mere').resolve(); DEBUG = TARGET / 'build/debug'
NAME = 'stickleback_export_probe'; sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8-sig'))
assert not (HERE / (NAME + '_receipt.json')).exists()
old = load('gemot_owner_probe_receipt.json'); family = Path(old['libraries']['stickleback']['metadata_path'])
assert sha(family) == old['libraries']['stickleback']['metadata_sha256']
deps = {r[1]: r[3] for r in json.loads(family.read_text())['deps']}
libs = {}
for name, expected in deps.items():
    aliases = ['identity', 'personae'] if name == 'identity' else [name]
    choices = []
    for alias in aliases:
        for meta in (DEBUG / '.fingerprint').glob('*/lib-' + alias + '.json'):
            marker = meta.with_suffix(''); suffix = meta.parent.name.rsplit('-', 1)[1]
            artifact = DEBUG / 'deps' / ('lib' + alias + '-' + suffix + '.rlib')
            if not marker.is_file() or not artifact.is_file(): continue
            value = int.from_bytes(bytes.fromhex(marker.read_text().strip()), 'little')
            if value == expected: choices.append({'path': str(artifact), 'sha256': sha(artifact), 'fingerprint_value': value,
                'metadata_path': str(meta), 'metadata_sha256': sha(meta)})
    assert len(choices) == 1, (name, len(choices)); libs[name] = choices[0]
assert len(libs) == 17
files = [p for p in load('source_inventory.json')['files'] if p.startswith('crates/stickleback/')]
sources = {p: sha(ROOT / p) for p in files}; assert len(files) == 30
library = TARGET / 'libauthority_success_stickleback_owner.rlib'; binary = TARGET / 'authority-success-stickleback-export.exe'
assert not library.exists() and not list(TARGET.glob(binary.stem + '.*'))
artifacts = ['drop-export-refused-plain.meredrop', 'drop-export-refused-selected.meredrop', 'drop-export-success-control.meredrop']
assert all(not (HERE / p).exists() for p in artifacts)
common = ['-L', 'dependency=' + str(DEBUG / 'deps'), '-C', 'debuginfo=0']
rows = [json.loads(s) for s in (HERE / 'djinn_settings_probe_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if s.startswith('{')]
for p in sorted({p for r in rows if r.get('reason') == 'build-script-executed' for p in r.get('linked_paths', [])}): common += ['-L', p]
owner_cmd = ['rustc', '--edition=2024', '--crate-name', 'stickleback_audit', '--crate-type', 'rlib', str(ROOT / 'crates/stickleback/src/lib.rs'), *common, '-o', str(library)]
for n, v in libs.items(): owner_cmd += ['--extern', n + '=' + v['path']]
consumer_cmd = ['rustc', '--edition=2024', str(HERE / (NAME + '.rs')), *common, '--extern', 'stickleback_audit=' + str(library), '-o', str(binary)]
for n in ['muniment', 'p2panda_core', 'serde']: consumer_cmd += ['--extern', n + '=' + libs[n]['path']]
results = {}; removed = []; checks = 0
def execute(label, command, timeout=60):
    r = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=timeout)
    with (HERE / (NAME + '_' + label + '.log')).open('x', encoding='utf-8') as f: f.write(r.stdout + r.stderr)
    results[label] = {'command': command, 'exit': r.returncode}
    if r.returncode: print(r.stdout + r.stderr)
    return r
try:
    owner = execute('owner_compile', owner_cmd)
    if owner.returncode == 0:
        consumer = execute('consumer_compile', consumer_cmd)
        if consumer.returncode == 0:
            run = execute('output', [str(binary), str(HERE)], 30); print(run.stdout + run.stderr)
            if run.returncode == 0: checks = sum(s.startswith('PASS ') for s in run.stdout.splitlines())
finally:
    for p in sorted(TARGET.glob(binary.stem + '.*')) + ([library] if library.exists() else []):
        assert p.resolve().parent == TARGET; data = p.read_bytes()
        assert ((p.suffix == '.exe' and data.startswith(b'MZ')) or (p.suffix == '.pdb' and data.startswith(b'Microsoft C/C++ MSF'))
            or (p.suffix in ['.lib', '.rlib'] and data.startswith(b'!<arch>\n')) or (p.suffix in ['.o', '.exp'] and data.startswith(b'\x64\x86')))
        removed.append({'path': str(p), 'bytes': len(data), 'sha256': sha(p)}); p.unlink()
assert all(sha(ROOT / p) == h for p, h in sources.items())
assert all(sha(Path(v['path'])) == v['sha256'] for v in libs.values())
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
    'source_sha256': sources, 'libraries': libs, 'family': {'path': str(family), 'sha256': sha(family)}, 'results': results,
    'audit_cases': checks, 'original_tests': 0, 'runner_sha256': sha(Path(__file__)), 'probe_sha256': sha(HERE / (NAME + '.rs')),
    'fixtures': {p: {'sha256': sha(HERE / p), 'bytes': (HERE / p).stat().st_size} for p in artifacts if (HERE / p).is_file()},
    'removed_outputs': removed, 'generated_output_removed': not library.exists() and not list(TARGET.glob(binary.stem + '.*')),
    'qualification': 'Complete unchanged current Stickleback library and public consumer freshly compiled over17exact retained fingerprint-matched suppliers. Two CPU/MemoryBackend/file cases use one genuinely signed operation admitted by the actual processor. The two refusal fixtures and successful control are new audit-owned files, never user exports; their final bytes are retained. Protected-export variant is source-reviewed but not executed. No full original suite, fresh Cargo graph, live carriage, RF, real user-file loss or domain authorization bypass. Hashing/compiling all30files does not claim all30have been source-read.'}
with (HERE / (NAME + '_receipt.json')).open('x', encoding='utf-8') as f: json.dump(receipt, f, indent=2); f.write('\n')
print(json.dumps({'exits': {k: v['exit'] for k, v in results.items()}, 'audit_cases': checks, 'fixture_bytes': {k: v['bytes'] for k, v in receipt['fixtures'].items()}, 'generated_output_removed': receipt['generated_output_removed']}))
assert len(results) == 3 and all(v['exit'] == 0 for v in results.values()) and checks == 2
