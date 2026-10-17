"""Run three unchanged chrome modules, their original tests and two audit cases."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, re, subprocess
HERE = Path(__file__).resolve().parent; ROOT = HERE.parents[2]
TARGET = Path('C:/t/cargo-targets/mere').resolve(); NAME = 'chrome_navigation_probe'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
load = lambda n: json.loads((HERE / n).read_text(encoding='utf-8-sig'))
assert not (HERE / (NAME + '_receipt.json')).exists()
libs = {'kernel': load('forme_probe_receipt.json')['kernel_import_dependency'], 'url': load('nematic_probe_receipt.json')['libraries']['url']}
assert all(sha(Path(v['path'])) == v['sha256'] for v in libs.values())
sources = {p: sha(ROOT / p) for p in ['crates/shell/chrome/src/nav.rs', 'crates/shell/chrome/src/omnibar.rs', 'crates/shell/chrome/src/suggest.rs']}
binary = TARGET / 'authority-success-chrome-navigation.exe'; assert not list(TARGET.glob(binary.stem + '.*'))
command = ['rustc', '--edition=2024', '--test', str(HERE / (NAME + '.rs')), '-L', 'dependency=' + str(TARGET / 'build/debug/deps'), '-C', 'debuginfo=0']
rows = [json.loads(s) for s in (HERE / 'djinn_settings_probe_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if s.startswith('{')]
for p in sorted({p for r in rows if r.get('reason') == 'build-script-executed' for p in r.get('linked_paths', [])}): command += ['-L', p]
for n, v in libs.items(): command += ['--extern', n + '=' + v['path']]
command += ['-o', str(binary)]
build = run = None; removed = []
try:
    build = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=60)
    with (HERE / (NAME + '_compile.log')).open('x', encoding='utf-8') as f: f.write(build.stdout + build.stderr)
    if build.returncode == 0:
        run = subprocess.run([str(binary), '--test-threads=1'], cwd=ROOT, capture_output=True, text=True, timeout=30)
        with (HERE / (NAME + '_output.log')).open('x', encoding='utf-8') as f: f.write(run.stdout + run.stderr)
        print(run.stdout[-500:] + run.stderr)
    else: print(build.stdout + build.stderr)
finally:
    for p in sorted(TARGET.glob(binary.stem + '.*')):
        assert p.resolve().parent == TARGET; data = p.read_bytes()
        assert ((p.suffix == '.exe' and data.startswith(b'MZ')) or (p.suffix == '.pdb' and data.startswith(b'Microsoft C/C++ MSF'))
            or (p.suffix == '.lib' and data.startswith(b'!<arch>\n')) or (p.suffix in ['.o', '.exp'] and data.startswith(b'\x64\x86')))
        removed.append({'path': str(p), 'bytes': len(data), 'sha256': sha(p)}); p.unlink()
checks = 0 if run is None or run.returncode else int(re.search(r'test result: ok\. (\d+) passed;', run.stdout)[1])
assert all(sha(ROOT / p) == h for p, h in sources.items())
assert all(sha(Path(v['path'])) == v['sha256'] for v in libs.values())
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(), 'basis_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
    'source_sha256': sources, 'libraries': libs, 'command': command, 'build_exit': build.returncode,
    'run_exit': None if run is None else run.returncode, 'checks': checks, 'original_tests': max(checks - 2, 0), 'audit_cases': 2,
    'runner_sha256': sha(Path(__file__)), 'probe_sha256': sha(HERE / (NAME + '.rs')), 'removed_outputs': removed,
    'generated_output_removed': not list(TARGET.glob(binary.stem + '.*')),
    'qualification': 'Three complete unchanged modules nav/omnibar/suggest compiled together over two exact retained kernel/url suppliers. Their original module tests plus two audit cases exercise pure CPU classification and resolution. This is not the complete Chrome crate/unit suite, a fresh Cargo graph, a browser or live host/provider request. The suggestion helper explicitly produces DuckDuckGo/default-sigil rows today; the cases establish that its public shared match input ignores other provider tags and that custom classification settings cannot reach this suggestion API.'}
with (HERE / (NAME + '_receipt.json')).open('x', encoding='utf-8') as f: json.dump(receipt, f, indent=2); f.write('\n')
print(json.dumps({k: receipt[k] for k in ['build_exit', 'run_exit', 'checks', 'original_tests', 'audit_cases', 'generated_output_removed']}))
assert build.returncode == 0 and run is not None and run.returncode == 0 and checks > 2
