"""Run the unchanged complete net helper against one exact retained URL parser."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
TARGET = Path('C:/t/cargo-targets/mere').resolve()
NAME = 'document_net_gate_probe'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE / (NAME + '_receipt.json')).exists()
prior = json.loads((HERE / 'nematic_probe_receipt.json').read_text())
supplier = prior['libraries']['url']
assert sha(Path(supplier['path'])) == supplier['sha256']
sources = {p: sha(ROOT / p) for p in ['crates/script/document-host/src/net.rs']}
binary = TARGET / 'authority-success-document-net-gate.exe'
assert not list(TARGET.glob(binary.stem + '.*'))
command = ['rustc', '--edition=2024', str(HERE / (NAME + '.rs')), '-L', 'dependency=' + str(TARGET / 'build/debug/deps'),
           '-C', 'debuginfo=0', '--extern', 'url=' + supplier['path'], '-o', str(binary)]
build = run = None
removed = []
try:
    build = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=60)
    with (HERE / (NAME + '_compile.log')).open('x', encoding='utf-8') as f: f.write(build.stdout + build.stderr)
    if build.returncode == 0:
        run = subprocess.run([str(binary)], cwd=ROOT, capture_output=True, text=True, timeout=30)
        with (HERE / (NAME + '_output.log')).open('x', encoding='utf-8') as f: f.write(run.stdout + run.stderr)
        print(run.stdout + run.stderr)
    else: print(build.stderr)
finally:
    for p in sorted(TARGET.glob(binary.stem + '.*')):
        assert p.resolve().parent == TARGET
        data = p.read_bytes()
        assert ((p.suffix == '.exe' and data.startswith(b'MZ')) or
                (p.suffix == '.pdb' and data.startswith(b'Microsoft C/C++ MSF')) or
                (p.suffix == '.lib' and data.startswith(b'!<arch>\n')) or
                (p.suffix in ['.o', '.exp'] and data.startswith(b'\x64\x86')))
        removed.append({'path': str(p), 'bytes': len(data), 'sha256': sha(p)})
        p.unlink()
assert all(sha(ROOT / p) == h for p, h in sources.items())
assert sha(Path(supplier['path'])) == supplier['sha256']
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(),
    'basis_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
    'source_sha256': sources, 'supplier': supplier, 'command': command,
    'build_exit': build.returncode, 'run_exit': None if run is None else run.returncode,
    'checks': 2 if run is not None and run.returncode == 0 else 0, 'original_tests': 0, 'audit_cases': 2,
    'runner_sha256': sha(Path(__file__)), 'probe_sha256': sha(HERE / (NAME + '.rs')),
    'removed_outputs': removed, 'generated_output_removed': not list(TARGET.glob(binary.stem + '.*')),
    'qualification': 'Complete unchanged document-host net.rs compiled directly with one exact hash-verified retained url2.5.8 library. A normal allow/deny control and raw backslash request show gate/parser disagreement; normalizing before the gate refuses it. This executes actual helper code, not ScriptHost/Wasmtime, a supplied production NetFetcher, current Cargo graph or live network. Host-only scheme/port policy is observed separately and not counted as another bypass. No HTTP request or credential transfer.'}
with (HERE / (NAME + '_receipt.json')).open('x', encoding='utf-8') as f:
    json.dump(receipt, f, indent=2); f.write('\n')
print(json.dumps({k: receipt[k] for k in ['build_exit', 'run_exit', 'checks', 'generated_output_removed']}))
assert build.returncode == 0 and run is not None and run.returncode == 0
