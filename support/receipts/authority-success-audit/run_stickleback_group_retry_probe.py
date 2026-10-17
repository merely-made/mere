"""Fresh complete unchanged owner, public session/lane save-failure diagnostic."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, os, subprocess
HERE = Path(__file__).resolve().parent; ROOT = HERE.parents[2]
TARGET = Path('C:/t/cargo-targets/mere').resolve(); NAME = 'stickleback_group_retry_probe'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
old = json.loads((HERE / 'stickleback_export_probe_v3_receipt.json').read_text())
assert not (HERE / (NAME + '_receipt.json')).exists()
assert all(sha(ROOT / p) == h for p, h in old['source_sha256'].items())
assert all(sha(Path(v['path'])) == v['sha256'] for v in old['libraries'].values())
assert sha(Path(old['family']['path'])) == old['family']['sha256']
library = TARGET / 'libauthority_success_group_retry_owner.rlib'
binary = TARGET / 'authority-success-group-retry.exe'
assert not library.exists() and not list(TARGET.glob(binary.stem + '.*'))
owner = old['results']['owner_compile']['command'].copy()
owner[owner.index('-o') + 1] = str(library)
common = owner[owner.index('-L'):owner.index('-o')]
consumer = ['rustc', '--edition=2024', str(HERE / (NAME + '.rs')), *common, '--extern',
    'stickleback_audit=' + str(library), '-o', str(binary)]
for n in ['identity', 'muniment']: consumer += ['--extern', n + '=' + old['libraries'][n]['path']]
env = dict(os.environ); env['CARGO_PKG_VERSION'] = old['CARGO_PKG_VERSION']
results = {}; removed = []; checks = 0
try:
    for label, command in [('owner_compile', owner), ('consumer_compile', consumer), ('output', [str(binary)])]:
        run = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True, timeout=60)
        (HERE / (NAME + '_' + label + '.log')).write_text(run.stdout + run.stderr, encoding='utf-8')
        results[label] = {'command': command, 'exit': run.returncode}
        if run.returncode or label == 'output': print(run.stdout + run.stderr)
        if run.returncode: break
        if label == 'output': checks = sum(s.startswith('PASS ') for s in run.stdout.splitlines())
finally:
    for p in sorted(TARGET.glob(binary.stem + '.*')) + ([library] if library.exists() else []):
        assert p.resolve().parent == TARGET; data = p.read_bytes()
        assert ((p.suffix == '.exe' and data.startswith(b'MZ')) or (p.suffix == '.pdb' and data.startswith(b'Microsoft C/C++ MSF'))
            or (p.suffix in ['.lib', '.rlib'] and data.startswith(b'!<arch>\n')) or (p.suffix in ['.o', '.exp'] and data.startswith(b'\x64\x86')))
        removed.append({'path': str(p), 'bytes': len(data), 'sha256': sha(p)}); p.unlink()
assert all(sha(ROOT / p) == h for p, h in old['source_sha256'].items())
assert all(sha(Path(v['path'])) == v['sha256'] for v in old['libraries'].values())
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': subprocess.check_output(['git','rev-parse','HEAD'], cwd=ROOT, text=True).strip(),
    'source_sha256': old['source_sha256'], 'libraries': old['libraries'], 'family': old['family'], 'results': results,
    'audit_cases': checks, 'original_tests': 0, 'runner_sha256': sha(Path(__file__)), 'probe_sha256': sha(HERE / (NAME + '.rs')),
    'removed_outputs': removed, 'generated_output_removed': not library.exists() and not list(TARGET.glob(binary.stem + '.*')),
    'qualification': 'Fresh whole unchanged Stickleback owner over17exact retained suppliers; real identities, signed operation processing, MemoryBackend and real group encryption. Three public-API cases: successful first save, failed save followed by same-live-session retry, and failed save followed by reload-before-retry. Secret snapshots stay in memory and are neither printed nor retained. Injected callback refusal, not real disk failure or host-level data loss; no original suite or live network. Compile source hashes do not grant source read credit.'}
with (HERE / (NAME + '_receipt.json')).open('x', encoding='utf-8') as f: json.dump(receipt, f, indent=2); f.write('\n')
print(json.dumps({'exits': {k:v['exit'] for k,v in results.items()}, 'audit_cases': checks, 'generated_output_removed': receipt['generated_output_removed']}))
assert len(results) == 3 and all(v['exit'] == 0 for v in results.values()) and checks == 3
