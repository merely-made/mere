"""Run actual portable Moot source/tests and compile the unchanged stale fixture."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, re, struct, subprocess
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
TARGET = Path('C:/t/cargo-targets/mere').resolve()
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
dest = HERE / 'community_identity_probe_receipt.json'
assert not dest.exists()
source_names = ['ports/moot/src/coop.rs', 'ports/moot/src/resource_coop.rs',
    'ports/moot/src/resource_coop/control.rs', 'ports/moot/src/resource_coop/tests.rs',
    'ports/graphshell/src/moot_resources.rs']
sources = {n: sha(ROOT / n) for n in source_names}
terminal = json.loads((HERE / 'capture_graphshell_terminal.json').read_text())
assert terminal['exit_code'] == 0
rows = [json.loads(l) for l in (HERE / 'capture_graphshell_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')]
assert any(r.get('reason') == 'build-finished' and r.get('success') for r in rows)
libs = {}
for name in ['serde', 'serde_json']:
    matches = [r for r in rows if r.get('reason') == 'compiler-artifact'
        and r['target']['name'] == name and any(f.endswith('.rlib') for f in r['filenames'])]
    if name == 'serde':
        matches = [r for r in matches if 'rc' in r['features']]
    assert len(matches) == 1, (name, len(matches))
    r = matches[0]
    path = next(Path(f) for f in r['filenames'] if f.endswith('.rlib'))
    libs[name] = {'path': str(path), 'sha256': sha(path), 'features': r['features'], 'package_id': r['package_id']}
base = ['rustc', '--edition=2024', '-A', 'dead_code', '-L', 'dependency=' + str(TARGET / 'build/debug/deps'), '-C', 'debuginfo=0']
for name, lib in libs.items():
    base += ['--extern', name + '=' + lib['path']]
binary = TARGET / 'authority-success-community-identity-probe.exe'
assert not list(TARGET.glob(binary.stem + '.*'))
command = base + ['--test', str(HERE / 'community_identity_probe.rs'), '-o', str(binary)]
built = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
(HERE / 'community_identity_probe_compile.log').write_text(built.stdout + built.stderr, encoding='utf-8')
assert built.returncode == 0, built.stderr
digest = sha(binary)
removed = []
try:
    ran = subprocess.run([str(binary), '--nocapture', '--test-threads=1'], cwd=ROOT, capture_output=True, text=True, timeout=30)
    (HERE / 'community_identity_probe_output.log').write_text(ran.stdout + ran.stderr, encoding='utf-8')
    print(ran.stdout)
finally:
    for p in sorted(TARGET.glob(binary.stem + '.*')):
        assert p.resolve().parent == TARGET
        data = p.read_bytes()
        if p.suffix == '.exe': assert data.startswith(b'MZ')
        elif p.suffix == '.pdb': assert data.startswith(b'Microsoft C/C++ MSF')
        elif p.suffix == '.lib': assert data.startswith(b'!<arch>\n') and binary.name.encode() in data
        elif p.suffix == '.exp': assert data[:2] == b'\x64\x86' and 0 < struct.unpack_from('<H', data, 2)[0] < 128 and binary.name.encode() in data
        else: raise AssertionError('Unknown generated output ' + str(p))
        removed.append({'path': str(p), 'bytes': len(data), 'sha256': sha(p)})
        p.unlink()
assert ran.returncode == 0, ran.stderr
summary = re.search(r'test result: ok\. (\d+) passed; 0 failed', ran.stdout)
assert summary
tests = int(summary[1])
assert tests >= 3

# Compile the original fixture body verbatim against the actual current type.
# This checks a compile-time defect; it does not run the Graphshell feature suite.
text = (ROOT / 'ports/graphshell/src/moot_resources.rs').read_text(encoding='utf-8')
start = text.index('    fn read() -> Snapshot {')
end = text.index('    #[test]', start)
fixture = text[start:end]
fixture_path = HERE / 'community_original_fixture_compile.rs'
assert not fixture_path.exists()
fixture_path.write_text('#![allow(dead_code)]\n#[path="../../../ports/moot/src/coop.rs"] mod coop;\n#[path="../../../ports/moot/src/resource_coop.rs"] mod resource_coop;\nuse resource_coop::*;\nuse std::collections::BTreeMap;\n' + fixture + '\nfn main() {}\n', encoding='utf-8')
metadata = TARGET / 'authority-success-community-original-fixture.rmeta'
assert not metadata.exists()
fixture_command = base + ['--emit=metadata', str(fixture_path), '-o', str(metadata)]
failed = subprocess.run(fixture_command, cwd=ROOT, capture_output=True, text=True)
(HERE / 'community_original_fixture_compile.log').write_text(failed.stdout + failed.stderr, encoding='utf-8')
assert failed.returncode != 0 and 'E0063' in failed.stderr and 'shared_by' in failed.stderr, failed.stderr
assert not metadata.exists()
assert all(sha(ROOT / n) == h for n, h in sources.items())
assert all(sha(Path(v['path'])) == v['sha256'] for v in libs.values())
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'source_sha256': sources,
    'libraries': libs, 'probe_build_command': command, 'probe_build_exit': built.returncode,
    'test_exit': ran.returncode, 'tests': tests, 'new_counterexample_and_controls': 3,
    'original_portable_tests': tests - 3, 'native_binary_sha256': digest,
    'fixture': {'source': 'ports/graphshell/src/moot_resources.rs', 'start_char': start, 'end_char': end,
        'verbatim_fixture_sha256': hashlib.sha256(fixture.encode()).hexdigest(), 'generated_source_sha256': sha(fixture_path),
        'compile_command': fixture_command, 'exit': failed.returncode, 'expected_diagnostic': 'E0063 missing shared_by'},
    'removed_outputs': removed, 'generated_output_removed': not list(TARGET.glob(binary.stem + '.*')),
    'scope': 'Actual current portable coop/resource_coop/control source and original embedded tests compiled directly with qualified serde libraries. Three public Pane counterexample/control tests. Original Graphshell fixture body compiled verbatim against actual current ResourceRead type and correctly fails E0063. No full Graphshell moot-resources feature build, browser, peer, signer, store, clock/revocation or hosting trial.'}
dest.write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
print(json.dumps({'receipt': dest.name, 'tests': tests, 'new_checks': 3,
    'original_portable_tests': tests - 3, 'fixture_compile': 'expected E0063',
    'generated_output_removed': receipt['generated_output_removed']}))
