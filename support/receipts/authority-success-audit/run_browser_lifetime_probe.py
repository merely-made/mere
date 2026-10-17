"""Run the unmodified browser lifetime's native tests, without supplier artifacts."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
TARGET = Path('C:/t/cargo-targets/mere').resolve()
NAME = 'browser_lifetime_probe'
source = ROOT / 'crates/cambium/cambium-genet-web-host/src/lifetime.rs'
binary = TARGET / 'authority-success-browser-lifetime-probe.exe'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE / (NAME + '_receipt.json')).exists()
assert not list(TARGET.glob(binary.stem + '.*'))
before = sha(source)
command = ['rustc', '--edition=2024', '--test', str(source), '-C', 'debuginfo=0', '-o', str(binary)]
ran = None
removed = []
try:
    built = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=90)
    with (HERE / (NAME + '_compile.log')).open('x', encoding='utf-8') as f:
        f.write(built.stdout + built.stderr)
    if built.returncode == 0:
        ran = subprocess.run([str(binary), '--test-threads=1'], cwd=ROOT, capture_output=True, text=True, timeout=30)
        with (HERE / (NAME + '_output.log')).open('x', encoding='utf-8') as f:
            f.write(ran.stdout + ran.stderr)
        print(ran.stdout + ran.stderr)
    else:
        print(built.stderr)
finally:
    for path in sorted(TARGET.glob(binary.stem + '.*')):
        assert path.resolve().parent == TARGET
        data = path.read_bytes()
        assert (path.suffix == '.exe' and data.startswith(b'MZ')) or (path.suffix == '.pdb' and data.startswith(b'Microsoft C/C++ MSF'))
        removed.append({'path': str(path), 'bytes': len(data), 'sha256': sha(path)})
        path.unlink()
assert sha(source) == before
receipt = {
    'date_utc': datetime.now(timezone.utc).isoformat(),
    'source_sha256': {source.relative_to(ROOT).as_posix(): before},
    'runner_sha256': sha(Path(__file__)),
    'commands': [command], 'build_exits': [built.returncode],
    'run_exit': None if ran is None else ran.returncode,
    'checks': 0 if ran is None else sum(line.startswith('test ') and line.endswith(' ... ok') for line in ran.stdout.splitlines()),
    'removed_outputs': removed,
    'generated_output_removed': not list(TARGET.glob(binary.stem + '.*')),
    'scope': 'Five original current lifetime.rs tests, compiled directly with the standard library. Native phase/startup/future/cleanup semantics only; no browser cleanup scheduling, DOM listener removal, WebGPU promise cancellation, or GPU acceptance.'
}
with (HERE / (NAME + '_receipt.json')).open('x', encoding='utf-8') as f:
    f.write(json.dumps(receipt, indent=2) + '\n')
print(json.dumps({key: receipt[key] for key in ['build_exits', 'run_exit', 'checks', 'generated_output_removed']}))
assert built.returncode == 0 and ran.returncode == 0 and receipt['checks'] == 5
