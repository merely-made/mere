"""Retry the public consumer with Issue imported; retain the passing unit run."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
TARGET = Path('C:/t/cargo-targets/mere').resolve()
NAME = 'servitor_owner_probe_v2'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE / (NAME + '_receipt.json')).exists()
old_path = HERE / 'servitor_owner_probe_receipt.json'
old = json.loads(old_path.read_text())
assert old['results']['owner_compile']['exit'] == old['results']['unit_compile']['exit'] == old['results']['unit_output']['exit'] == 0
assert old['results']['consumer_compile']['exit'] == 1 and old['original_tests'] == 98 and old['audit_cases'] == 0
assert old['generated_output_removed']
assert old['runner_sha256'] == sha(HERE / 'run_servitor_owner_probe.py')
assert old['probe_sha256'] == sha(HERE / 'servitor_owner_probe.rs')
assert all(sha(ROOT / p) == h for p, h in old['source_sha256'].items())
assert all(sha(Path(v['path'])) == v['sha256'] for v in old['libraries'].values())
binary = TARGET / 'authority-success-servitor-owner.exe'
library = TARGET / 'libauthority_success_servitor_owner.rlib'
assert not library.exists() and not list(TARGET.glob(binary.stem + '.*'))
owner_cmd = old['results']['owner_compile']['command']
consumer_cmd = [str(HERE / (NAME + '.rs')) if s == str(HERE / 'servitor_owner_probe.rs') else s
                for s in old['results']['consumer_compile']['command']]
results = {}; removed = []
def execute(label, command, seconds=60):
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=seconds)
    with (HERE / (NAME + '_' + label + '.log')).open('x', encoding='utf-8') as f: f.write(result.stdout + result.stderr)
    results[label] = {'command': command, 'exit': result.returncode}
    if result.returncode: print(result.stdout + result.stderr)
    return result
checks = 0
try:
    owner = execute('owner_compile', owner_cmd)
    if owner.returncode == 0:
        consumer = execute('consumer_compile', consumer_cmd)
        if consumer.returncode == 0:
            run = execute('output', [str(binary)], 30)
            print(run.stdout + run.stderr)
            if run.returncode == 0: checks = sum(s.startswith('PASS ') for s in run.stdout.splitlines())
finally:
    for p in sorted(TARGET.glob(binary.stem + '.*')) + ([library] if library.exists() else []):
        assert p.resolve().parent == TARGET
        data = p.read_bytes()
        assert ((p.suffix == '.exe' and data.startswith(b'MZ')) or
                (p.suffix == '.pdb' and data.startswith(b'Microsoft C/C++ MSF')) or
                (p.suffix in ['.lib', '.rlib'] and data.startswith(b'!<arch>\n')) or
                (p.suffix in ['.o', '.exp'] and data.startswith(b'\x64\x86')))
        removed.append({'path': str(p), 'bytes': len(data), 'sha256': sha(p)}); p.unlink()
assert all(sha(ROOT / p) == h for p, h in old['source_sha256'].items())
assert all(sha(Path(v['path'])) == v['sha256'] for v in old['libraries'].values())
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(),
    'basis_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
    'source_sha256': old['source_sha256'], 'libraries': old['libraries'], 'supplier_metadata': old['supplier_metadata'],
    'native_search_paths': old['native_search_paths'], 'results': results,
    'original_tests_this_attempt': 0, 'prior_original_tests': {'path': old_path.name, 'sha256': sha(old_path), 'tests_passed': 98,
        'qualification': 'Same unchanged complete owner and exact suppliers. First attempt passed all98original tests, then its new consumer failed due to missing Issue trait import. Original tests were not rerun for this audit-only import fix.'},
    'audit_cases': checks, 'runner_sha256': sha(Path(__file__)), 'probe_sha256': sha(HERE / (NAME + '.rs')),
    'removed_outputs': removed, 'generated_output_removed': not library.exists() and not list(TARGET.glob(binary.stem + '.*')),
    'qualification': old['qualification'] + ' This retry reruns whole-owner and public consumer compilation only. All98original unit tests passed in the preserved first attempt; its consumer compile failure was not a production failure. V2 adds the required identity::delegation::Issue import.'}
with (HERE / (NAME + '_receipt.json')).open('x', encoding='utf-8') as f:
    json.dump(receipt, f, indent=2); f.write('\n')
print(json.dumps({'exit_codes': {k: v['exit'] for k, v in results.items()}, 'audit_cases': checks, 'prior_original_tests': 98, 'generated_output_removed': receipt['generated_output_removed']}))
assert len(results) == 3 and all(v['exit'] == 0 for v in results.values()) and checks == 4
