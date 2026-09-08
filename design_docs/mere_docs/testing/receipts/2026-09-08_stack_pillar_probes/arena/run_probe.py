from pathlib import Path
import subprocess, os, sys, json, hashlib, time
base = Path(__file__).resolve().parent
mode = sys.argv[1] if len(sys.argv) > 1 else 'debug'
env = os.environ.copy()
env['CARGO_HOME'] = str(base / 'cargo-home')
command = ['cargo', 'test', '--offline', '--manifest-path', str(base / 'probe/Cargo.toml'), '--target-dir', str(base / 'target')]
if mode == 'release':
    command.append('--release')
command.extend(['--', '--nocapture'])
started = time.time()
with (base / (mode + '.log')).open('w', encoding='utf-8') as log:
    result = subprocess.run(command, cwd=base / 'probe', env=env, stdout=log, stderr=subprocess.STDOUT)
record = {'command': command, 'exit_code': result.returncode, 'elapsed_seconds': time.time() - started,
          'cargo_home': env['CARGO_HOME'], 'engine': 'Rust arena only', 'renderer': 'none', 'server': 'none'}
for name in ['probe/Cargo.toml', 'probe/Cargo.lock', 'probe/src/lib.rs', 'source-manifest.json', 'run_probe.py']:
    p = base / name
    if p.exists():
        record[name + '_sha256'] = hashlib.sha256(p.read_bytes()).hexdigest()
(base / (mode + '-run.json')).write_text(json.dumps(record, indent=2))
print(json.dumps(record, indent=2), flush=True)
print((base / (mode + '.log')).read_text()[-8000:], flush=True)
sys.exit(result.returncode)
