"""Compile only the audit probe against a fresh named Cargo build artifact."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
TARGET = Path('C:/t/cargo-targets/mere').resolve()
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
receipt_path = HERE / 'inference_crash_probe_receipt.json'
assert not receipt_path.exists()
sources = {n: sha(ROOT / n) for n in ['ports/distillery/src/inference.rs',
    'ports/distillery/src/inference/durable.rs', 'crates/intel/esp/src/infer/actor.rs',
    'crates/armillary/src/actor.rs', 'ports/distillery/Cargo.toml', 'Cargo.toml', 'Cargo.lock']}
rows = [json.loads(line) for line in (HERE / 'inference_crash_probe_build.jsonl')
        .read_text(encoding='utf-8-sig').splitlines() if line.startswith('{')]
assert any(r.get('reason') == 'build-finished' and r.get('success') for r in rows)
libs = {}
for name in ['distillery', 'serde_json']:
    records = [r for r in rows if r.get('reason') == 'compiler-artifact'
               and r['target']['name'] == name and any(p.endswith('.rlib') for p in r['filenames'])]
    assert len(records) == 1, (name, len(records))
    record = records[0]
    path = next(Path(p) for p in record['filenames'] if p.endswith('.rlib'))
    if path.parent == TARGET / 'debug':
        matches = [p for p in (TARGET / 'build/debug/deps').glob('lib' + name + '-*.rlib') if sha(p) == sha(path)]
        assert matches
        path = matches[0]
    libs[name] = {'path': str(path), 'sha256': sha(path), 'features': record['features'],
                  'package_id': record['package_id'], 'target_source': record['target']['src_path']}
binary = TARGET / 'authority-success-inference-crash-probe.exe'
pdb = binary.with_suffix('.pdb')
assert binary.parent == TARGET and not binary.exists() and not pdb.exists()
command = ['rustc', '--edition=2024', '--test', str(HERE / 'inference_crash_probe.rs'),
           '-L', 'dependency=' + str(TARGET / 'build/debug/deps'), '-C', 'debuginfo=0', '-o', str(binary)]
for name, lib in libs.items(): command += ['--extern', name + '=' + lib['path']]
native = sorted({p for r in rows if r.get('reason') == 'build-script-executed' for p in r.get('linked_paths', [])})
for path in native: command += ['-L', path]
built = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
(HERE / 'inference_crash_probe_compile.log').write_text(built.stdout + built.stderr, encoding='utf-8')
assert built.returncode == 0, built.stderr
digest = sha(binary)
try:
    ran = subprocess.run([str(binary), '--nocapture', '--test-threads=1'], cwd=ROOT,
                         capture_output=True, text=True, timeout=30)
    (HERE / 'inference_crash_probe_output.log').write_text(ran.stdout + ran.stderr, encoding='utf-8')
    print(ran.stdout)
finally:
    assert binary.read_bytes().startswith(b'MZ')
    binary.unlink()
    if pdb.exists():
        assert pdb.read_bytes().startswith(b'Microsoft C/C++ MSF')
        pdb.unlink()
assert ran.returncode == 0, ran.stderr
assert all(sha(ROOT / n) == h for n, h in sources.items())
assert all(sha(Path(lib['path'])) == lib['sha256'] for lib in libs.values())
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'source_sha256': sources,
    'libraries': libs, 'native_search_paths': native, 'probe_build_command': command,
    'probe_build_exit': built.returncode, 'native_binary_sha256': digest, 'tests': 3,
    'test_exit': ran.returncode, 'generated_output_removed': not binary.exists() and not pdb.exists(),
    'scope': 'Two public InferenceLane observations inject a gated provider-construction panic and use an in-memory RunJournal. Ordinary returned model-error control passes. No GPU, real model, disk journal, resident process, broker or network. Demonstrates ignored dead actor command delivery, queued state left by a rejected ask, successful pause/cancel/close despite stranded work, and successful queued resume to a disconnected actor. Lock waiting is source-qualified only; no lock-hang probe ran.'}
receipt_path.write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
print(json.dumps({'receipt': receipt_path.name, 'tests': 3, 'test_exit': ran.returncode,
                  'generated_output_removed': receipt['generated_output_removed']}))
