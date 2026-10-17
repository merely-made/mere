"""Compile current lexical library unit suites over exact retained suppliers."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, os, re, subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
TARGET = Path('C:/t/cargo-targets/mere').resolve()
NAME = 'lexical_original_tests'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE/(NAME+'_receipt.json')).exists()
sources = {p.relative_to(ROOT).as_posix(): sha(p) for area in ['crates/intel/reference-data', 'crates/intel/mora-cmudict'] for p in (ROOT/area).rglob('*') if p.is_file() and (p.suffix == '.rs' or p.name == 'Cargo.toml')}
runtime = {n: sha(ROOT/n) for n in ['crates/intel/reference-data/fixtures/bilingual.json', 'crates/intel/mora-cmudict/data/cmudict.dict']}
rows = [json.loads(line) for line in (HERE/'capture_graphshell_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if line.startswith('{')]
libs = {}
for name in ['serde', 'serde_json', 'blake3']:
    matches = [r for r in rows if r.get('reason') == 'compiler-artifact' and r['target']['name'] == name and any(f.endswith('.rlib') for f in r['filenames'])]
    if name == 'serde':
        matches = [r for r in matches if 'rc' in r['features']]
    assert len(matches) == 1, (name, len(matches))
    row = matches[0]
    path = Path(next(f for f in row['filenames'] if f.endswith('.rlib')))
    libs[name] = {'path': str(path), 'sha256': sha(path), 'features': row['features'], 'package_id': row['package_id']}
mora = TARGET/'build/debug/deps/libmora-66153f59195438cf.rlib'
fingerprint = TARGET/'build/debug/.fingerprint/mora-66153f59195438cf/lib-mora.json'
libs['mora'] = {'path': str(mora), 'sha256': sha(mora), 'fingerprint_path': str(fingerprint), 'fingerprint_sha256': sha(fingerprint), 'fingerprint': json.loads(fingerprint.read_text()), 'package_id': 'Cargo.lock registry mora0.1.0; retained matching fingerprint; no fresh dependency build'}
fixture_root = TARGET/'authority-success-reference-fixtures'
assert not fixture_root.exists()
fixture_root.mkdir()
marker = fixture_root/'audit-owned-fixtures.json'
marker.write_text(json.dumps({'runner': str(Path(__file__).resolve()), 'purpose': 'temporary original reference-data test roots'}), encoding='utf-8')
marker_hash = sha(marker)
env = os.environ.copy()
env['TEMP'] = str(fixture_root)
env['TMP'] = str(fixture_root)
base = ['rustc', '--edition=2024', '-L', 'dependency='+str(TARGET/'build/debug/deps'), '-C', 'debuginfo=0', '--test']
results = {}
removed = []
for crate, area, names in [('reference_data', 'reference-data', ['serde', 'serde_json', 'blake3']), ('mora_cmudict', 'mora-cmudict', ['mora'])]:
    source = ROOT/'crates/intel'/area/'src/lib.rs'
    binary = TARGET/('authority-success-'+area+'.exe')
    assert not list(TARGET.glob(binary.stem+'.*'))
    command = base+['--crate-name', crate, str(source), '-o', str(binary)]
    for name in names:
        command += ['--extern', name+'='+libs[name]['path']]
    build = None
    run = None
    try:
        build = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True, timeout=60)
        with (HERE/(NAME+'_'+area+'_compile.log')).open('x', encoding='utf-8') as f:
            f.write(build.stdout+build.stderr)
        if build.returncode == 0:
            run = subprocess.run([str(binary), '--test-threads=1'], cwd=ROOT, env=env, capture_output=True, text=True, timeout=45)
            with (HERE/(NAME+'_'+area+'_output.log')).open('x', encoding='utf-8') as f:
                f.write(run.stdout+run.stderr)
            print(run.stdout+run.stderr, flush=True)
        else:
            print(build.stderr, flush=True)
    finally:
        for path in sorted(TARGET.glob(binary.stem+'.*')):
            assert path.resolve().parent == TARGET
            data = path.read_bytes()
            assert (path.suffix == '.exe' and data.startswith(b'MZ')) or (path.suffix == '.pdb' and data.startswith(b'Microsoft C/C++ MSF')) or (path.suffix == '.lib' and data.startswith(b'!<arch>\n')) or (path.suffix == '.exp' and data.startswith(b'd\x86'))
            removed.append({'path': str(path), 'bytes': len(data), 'sha256': sha(path)})
            path.unlink()
    match = re.search(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', run.stdout) if run is not None else None
    results[area] = {'command': command, 'build_exit': None if build is None else build.returncode, 'run_exit': None if run is None else run.returncode, 'passed': int(match[1]) if match else None, 'failed': int(match[2]) if match else None, 'ignored': int(match[3]) if match else None}
assert sha(marker) == marker_hash and list(fixture_root.iterdir()) == [marker]
marker.unlink()
fixture_root.rmdir()
assert all(sha(ROOT/n) == h for n, h in {**sources, **runtime}.items())
assert all(sha(Path(value['path'])) == value['sha256'] for value in libs.values())
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(), 'source_sha256': sources, 'runtime_input_sha256': runtime, 'libraries': libs, 'results': results, 'runner_sha256': sha(Path(__file__)), 'removed_outputs': removed, 'generated_output_removed': all(not Path(value['path']).exists() for value in removed) and not fixture_root.exists(), 'qualification': 'Fresh compilation and execution of unchanged current reference-data and mora-cmudict library unit suites, with exact retained serde/serde_json/blake3/mora suppliers. Full shipped CMUdict parsed by original test. Native Windows CPU/filesystem and subprocess lock exit only; no fresh Cargo graph, Unix symlink/directory-sync path, physical power-cut, host import UI or source authenticity acceptance.'}
with (HERE/(NAME+'_receipt.json')).open('x', encoding='utf-8') as f:
    json.dump(receipt, f, indent=2)
    f.write('\n')
print(json.dumps({'results': results, 'generated_output_removed': receipt['generated_output_removed']}))
assert all(value['build_exit'] == value['run_exit'] == 0 and value['failed'] == 0 for value in results.values())
