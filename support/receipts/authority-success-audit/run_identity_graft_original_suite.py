"""Fresh original Incipit and Graft tests over qualified retained suppliers."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, os, re, subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
TARGET = Path('C:/t/cargo-targets/mere').resolve()
NAME = 'identity_graft_original_suite'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE / (NAME + '_receipt.json')).exists()
rows = [json.loads(l) for l in (HERE / 'capture_graphshell_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')]
libraries = {}
for name in ['serde', 'serde_json', 'uuid', 'inker', 'tracing', 'accesskit']:
    matches = [r for r in rows if r.get('reason') == 'compiler-artifact' and r['target']['name'] == name and any(f.endswith('.rlib') for f in r['filenames'])]
    assert len(matches) == 1, (name, len(matches))
    r = matches[0]
    p = Path(next(f for f in r['filenames'] if f.endswith('.rlib')))
    libraries[name] = {'path': str(p), 'sha256': sha(p), 'features': r['features'], 'package_id': r['package_id']}
inventory = json.loads((HERE / 'source_inventory.json').read_text())['files']
sources = {n: v['sha256'] for n, v in inventory.items() if n.startswith(('crates/incipit/', 'crates/inker/engines/graft-engine/'))}
assert len(sources) == 6 and all(sha(ROOT / n) == h for n, h in sources.items())
native = sorted({p for r in rows if r.get('reason') == 'build-script-executed' for p in r.get('linked_paths', [])})
commands, builds, runs, removed = [], [], [], []
for crate, source, names, version in [
    ('incipit', 'crates/incipit/src/lib.rs', ['serde', 'serde_json', 'uuid'], '0.2.0'),
    ('graft_engine', 'crates/inker/engines/graft-engine/src/lib.rs', ['inker', 'tracing', 'accesskit'], '0.2.0'),
]:
    binary = TARGET / ('authority-success-' + crate + '-original.exe')
    assert not list(TARGET.glob(binary.stem + '.*'))
    cmd = ['rustc', '--edition=2024', '--test', '--crate-name', crate, str(ROOT / source), '-L', 'dependency=' + str(TARGET / 'build/debug/deps'), '-C', 'debuginfo=0', '-o', str(binary)]
    for name in names:
        cmd += ['--extern', name + '=' + libraries[name]['path']]
    for path in native:
        cmd += ['-L', path]
    commands.append(cmd)
    env = os.environ.copy()
    env['CARGO_PKG_VERSION'] = version
    try:
        build = subprocess.run(cmd, cwd=ROOT, env=env, capture_output=True, text=True, timeout=120)
        with (HERE / (NAME + '_' + crate + '_compile.log')).open('x', encoding='utf-8') as f:
            f.write(build.stdout + build.stderr)
        builds.append({'crate': crate, 'exit': build.returncode})
        if build.returncode == 0:
            ran = subprocess.run([str(binary), '--test-threads=1'], cwd=ROOT, capture_output=True, text=True, timeout=30)
            with (HERE / (NAME + '_' + crate + '_output.log')).open('x', encoding='utf-8') as f:
                f.write(ran.stdout + ran.stderr)
            match = re.search(r'test result: ok\. (\d+) passed;', ran.stdout)
            runs.append({'crate': crate, 'exit': ran.returncode, 'checks': int(match.group(1)) if match else 0})
            print(ran.stdout + ran.stderr)
        else:
            print(build.stderr)
    finally:
        for p in sorted(TARGET.glob(binary.stem + '.*')):
            assert p.resolve().parent == TARGET
            data = p.read_bytes()
            assert (p.suffix == '.exe' and data.startswith(b'MZ')) or (p.suffix == '.pdb' and data.startswith(b'Microsoft C/C++ MSF')), p
            removed.append({'path': str(p), 'sha256': sha(p), 'bytes': len(data)})
            p.unlink()
assert all(sha(ROOT / n) == h for n, h in sources.items())
assert all(sha(Path(v['path'])) == v['sha256'] for v in libraries.values())
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'source_sha256': sources, 'libraries': libraries, 'native_search_paths': native, 'commands': commands, 'builds': builds, 'runs': runs, 'runner_sha256': sha(Path(__file__)), 'removed_outputs': removed, 'generated_output_removed': all(not Path(p['path']).exists() for p in removed), 'qualification': 'Fresh unchanged original Incipit and Graft test sources, using exact retained Cargo-built supplier artifacts. Graft tests use explicit stub/witness surfaces; no Servo, live producer, GPU import, window or product acceptance. No full linked-data build or RDF/query execution.'}
with (HERE / (NAME + '_receipt.json')).open('x', encoding='utf-8') as f:
    f.write(json.dumps(receipt, indent=2) + '\n')
print(json.dumps({'builds': builds, 'runs': runs, 'generated_output_removed': receipt['generated_output_removed']}))
assert all(b['exit'] == 0 for b in builds) and len(runs) == 2 and all(r['exit'] == 0 and r['checks'] > 0 for r in runs)
