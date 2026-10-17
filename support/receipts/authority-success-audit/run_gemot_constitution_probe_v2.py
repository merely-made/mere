"""Fresh four-module constitutional diagnostic over one retained Cargo family."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
TARGET = Path('C:/t/cargo-targets/mere').resolve()
DEBUG = TARGET / 'build/debug'
FP = DEBUG / '.fingerprint'
NAME = 'gemot_constitution_probe_v2'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
assert not (HERE / (NAME + '_receipt.json')).exists()
gemot = FP / 'gemot-2a2dd68ccc68833a/lib-gemot.json'
deps = {d[1]: d[3] for d in json.loads(gemot.read_text())['deps']}
prefixes = ('mere-proofs-', 'personae-', 'identity-', 'p2panda-core-', 'p2panda-store-',
            'muniment-', 'serde-', 'thiserror-', 'stickleback-', 'mien-', 'mooting-')
index = []
for folder in FP.iterdir():
    if not folder.is_dir() or not folder.name.startswith(prefixes): continue
    suffix = folder.name.rsplit('-', 1)[1]
    for metadata in folder.glob('lib-*.json'):
        crate = metadata.name[4:-5]
        marker = folder / ('lib-' + crate)
        artifact = DEBUG / 'deps' / ('lib' + crate + '-' + suffix + '.rlib')
        if not marker.is_file() or not artifact.is_file(): continue
        encoded = marker.read_text().strip()
        if len(encoded) != 16: continue
        index.append((crate, int.from_bytes(bytes.fromhex(encoded), 'little'), artifact, metadata, marker))
libs = {}
for name in ['proofs', 'identity', 'p2panda_core', 'p2panda_store', 'muniment', 'serde', 'thiserror', 'stickleback', 'mien', 'mooting']:
    aliases = {'identity', 'personae'} if name == 'identity' else {name}
    choices = [r for r in index if r[0] in aliases and r[1] == deps[name]]
    assert len(choices) == 1, (name, deps[name], len(choices))
    crate, value, artifact, metadata, marker = choices[0]
    libs[name] = {'path': str(artifact), 'sha256': sha(artifact), 'fingerprint_value': value,
                  'metadata_path': str(metadata), 'metadata_sha256': sha(metadata), 'marker_sha256': sha(marker)}
sources = ['crates/moot/gemot/src/moot/constitution/' + name + '.rs' for name in ['event', 'fold', 'wire', 'store']]
source_hashes = {p: sha(ROOT / p) for p in sources}
rows = [json.loads(s) for s in (HERE / 'djinn_settings_probe_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if s.startswith('{')]
native = sorted({p for r in rows if r.get('reason') == 'build-script-executed' for p in r.get('linked_paths', [])})
binary = TARGET / 'authority-success-gemot-constitution.exe'
assert not list(TARGET.glob(binary.stem + '.*'))
command = ['rustc', '--edition=2024', str(HERE / 'gemot_constitution_probe.rs'), '-L', 'dependency=' + str(DEBUG / 'deps'),
           '-C', 'debuginfo=0', '-o', str(binary)]
for path in native: command += ['-L', path]
for name, item in libs.items(): command += ['--extern', name + '=' + item['path']]
build = run = None
removed = []
try:
    build = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=60)
    with (HERE / (NAME + '_compile.log')).open('x', encoding='utf-8') as out: out.write(build.stdout + build.stderr)
    if build.returncode == 0:
        run = subprocess.run([str(binary)], cwd=ROOT, capture_output=True, text=True, timeout=30)
        with (HERE / (NAME + '_output.log')).open('x', encoding='utf-8') as out: out.write(run.stdout + run.stderr)
        print(run.stdout + run.stderr)
    else: print(build.stderr)
finally:
    for path in sorted(TARGET.glob(binary.stem + '.*')):
        assert path.resolve().parent == TARGET
        data = path.read_bytes()
        assert (path.suffix == '.exe' and data.startswith(b'MZ')) or (path.suffix == '.pdb' and data.startswith(b'Microsoft C/C++ MSF')) or (path.suffix == '.lib' and data.startswith(b'!<arch>\n')) or (path.suffix in ['.o', '.exp'] and data.startswith(b'\x64\x86'))
        removed.append({'path': str(path), 'bytes': len(data), 'sha256': sha(path)})
        path.unlink()
assert all(sha(ROOT / p) == h for p, h in source_hashes.items())
assert all(sha(Path(item['path'])) == item['sha256'] for item in libs.values())
checks = 0 if run is None or run.returncode else sum(s.startswith('PASS ') for s in run.stdout.splitlines())
receipt = {'date_utc': datetime.now(timezone.utc).isoformat(),
    'basis_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
    'source_sha256': source_hashes, 'libraries': libs,
    'supplier_family': {'path': str(gemot), 'sha256': sha(gemot)}, 'native_link_searches': native,
    'command': command, 'build_exit': build.returncode, 'run_exit': None if run is None else run.returncode,
    'checks': checks, 'audit_cases': 3, 'original_tests': 0, 'runner_sha256': sha(Path(__file__)),
    'probe_sha256': sha(HERE / 'gemot_constitution_probe.rs'), 'removed_outputs': removed,
    'generated_output_removed': not list(TARGET.glob(binary.stem + '.*')),
    'qualification': 'Four unchanged complete current constitutional event/fold/wire/store modules freshly compiled over ten exact retained suppliers selected by the named Gemot Cargo fingerprint family. Real signatures, OperationProcessor and MemoryBackend; independent eligible member authors each use sequence zero, no same-signer fork. Sequential and missing-parent/retry controls plus opposite-arrival competing amendment counterexample. No whole Gemot/current Cargo graph, live LogSync, aggregate service, disk, child, headed or physical carrier execution.'}
with (HERE / (NAME + '_receipt.json')).open('x', encoding='utf-8') as out: json.dump(receipt, out, indent=2); out.write('\n')
print(json.dumps({k: receipt[k] for k in ['build_exit', 'run_exit', 'checks', 'generated_output_removed']}))
assert build.returncode == 0 and run is not None and run.returncode == 0 and checks == 3
