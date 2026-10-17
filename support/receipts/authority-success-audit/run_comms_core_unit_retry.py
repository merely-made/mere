"""Retry only original core tests with the missing retained Windows import lib."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, os, re, subprocess
HERE = Path(__file__).resolve().parent; ROOT = HERE.parents[2]
TARGET = Path('C:/t/cargo-targets/mere').resolve(); NAME = 'comms_core_unit_retry'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
old_path = HERE / 'comms_routing_probe_receipt.json'; old = json.loads(old_path.read_text())
assert not (HERE / (NAME + '_receipt.json')).exists()
assert old['results']['unit_compile']['exit'] == 1 and old['audit_cases'] == 2
assert old['runner_sha256'] == sha(HERE / 'run_comms_routing_probe.py') and old['probe_sha256'] == sha(HERE / 'comms_routing_probe.rs')
assert old['generated_output_removed'] and all(sha(ROOT / p) == h for p, h in old['source_sha256'].items())
assert all(sha(Path(v['path'])) == v['sha256'] for v in old['libraries'].values())
native = Path('C:/Users/mark_/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/windows_x86_64_msvc-0.53.1/lib')
import_lib = native / 'windows.0.53.0.lib'; assert import_lib.is_file()
binary = TARGET / 'authority-success-comms-unit.exe'; assert not list(TARGET.glob(binary.stem + '.*'))
command = old['results']['unit_compile']['command'] + ['-L', 'native=' + str(native)]
env = dict(os.environ); env['CARGO_PKG_VERSION'] = old['CARGO_PKG_VERSION']
build = run = None; removed = []
try:
    build = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True, timeout=60)
    with (HERE / (NAME + '_compile.log')).open('x', encoding='utf-8') as f: f.write(build.stdout + build.stderr)
    if build.returncode == 0:
        run = subprocess.run([str(binary), '--test-threads=1'], cwd=ROOT, capture_output=True, text=True, timeout=30)
        with (HERE / (NAME + '_output.log')).open('x', encoding='utf-8') as f: f.write(run.stdout + run.stderr)
        print(run.stdout[-400:] + run.stderr)
    else: print(build.stdout + build.stderr)
finally:
    for p in sorted(TARGET.glob(binary.stem + '.*')):
        assert p.resolve().parent == TARGET; data = p.read_bytes()
        assert ((p.suffix == '.exe' and data.startswith(b'MZ')) or (p.suffix == '.pdb' and data.startswith(b'Microsoft C/C++ MSF'))
            or (p.suffix == '.lib' and data.startswith(b'!<arch>\n')) or (p.suffix in ['.o', '.exp'] and data.startswith(b'\x64\x86')))
        removed.append({'path': str(p), 'bytes': len(data), 'sha256': sha(p)}); p.unlink()
tests = 0 if run is None or run.returncode else int(re.search(r'test result: ok\. (\d+) passed;', run.stdout)[1])
assert all(sha(ROOT / p) == h for p, h in old['source_sha256'].items())
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': old['basis_commit'],
    'prior_receipt': {'path': old_path.name, 'sha256': sha(old_path)}, 'source_sha256': old['source_sha256'],
    'libraries': old['libraries'], 'native_library': {'path': str(import_lib), 'sha256': sha(import_lib)},
    'command': command, 'build_exit': build.returncode, 'run_exit': None if run is None else run.returncode,
    'original_core_tests': tests, 'audit_cases_this_attempt': 0, 'runner_sha256': sha(Path(__file__)),
    'removed_outputs': removed, 'generated_output_removed': not list(TARGET.glob(binary.stem + '.*')),
    'qualification': 'Original unchanged portable Comms core unit suite only, with optional Misfin/Murm features disabled. Adds an exact retained Windows import-library path omitted from the first direct compiler invocation. No production source change or fresh Cargo graph. The two public in-memory routing cases already passed in the preserved first attempt and were not repeated.'}
with (HERE / (NAME + '_receipt.json')).open('x', encoding='utf-8') as f: json.dump(receipt, f, indent=2); f.write('\n')
print(json.dumps({'build_exit': build.returncode, 'run_exit': receipt['run_exit'], 'original_core_tests': tests, 'generated_output_removed': receipt['generated_output_removed']}))
assert build.returncode == 0 and run is not None and run.returncode == 0 and tests > 0
