"""Execute unchanged dated packagers against an entirely in-memory fixture."""
import builtins
from contextlib import redirect_stdout
from hashlib import sha256
from io import StringIO
import json
from pathlib import Path, PurePosixPath
from types import SimpleNamespace

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
OUT = HERE / 'receipt_packaging_probe_receipt.json'
assert not OUT.exists()
variants = [('main-050e7d542', 11, 68), ('main-8568f42fd', 12, 72),
            ('command-option-name-repair', 3, 72), ('option-mirror-repair', 4, 83),
            ('retired-hover-repair', 5, 88)]
checks = []
sources = {}
for variant, gate_count, claimed_tests in variants:
    actual = ROOT / ('ports/graphshell/docs/receipts/host_presenter_20261012/' + variant + '/package.py')
    source = actual.read_bytes()
    sources[actual.relative_to(ROOT).as_posix()] = sha256(source).hexdigest()
    for condition in ('zero-test-success', 'failed-gate', 'missing-gate'):
        files = {}
        directories = set()
        operations = []

        class MemoryPath:
            def __init__(self, value): self.path = PurePosixPath(str(value))
            def __str__(self): return str(self.path)
            def __truediv__(self, value): return MemoryPath(self.path / str(value))
            @property
            def parent(self): return MemoryPath(self.path.parent)
            @property
            def name(self): return self.path.name
            @property
            def suffix(self): return self.path.suffix
            def is_file(self): return str(self) in files
            def relative_to(self, base): return self.path.relative_to(str(base))
            def read_bytes(self): return files[str(self)]
            def read_text(self, *args, **kwargs): return self.read_bytes().decode()
            def write_text(self, text, *args, **kwargs):
                files[str(self)] = text.encode(); operations.append(('write', str(self)))
            def iterdir(self):
                return iter(MemoryPath(p) for p in list(files) if PurePosixPath(p).parent == self.path)
            def glob(self, pattern):
                assert pattern == '*'
                return self.iterdir()

        def put(path, data): files[str(path)] = data if isinstance(data, bytes) else data.encode()
        def copy2(source, destination):
            put(destination, files[str(source)]); operations.append(('copy', str(source), str(destination)))
        def copytree(source, destination, **kwargs):
            # Empty synthetic scenario/fixture directories have no entries.
            assert str(source).endswith(('/fixtures', '/scenarios'))
            operations.append(('copytree', str(source), str(destination)))
        def run(command, **kwargs):
            assert command[0].endswith('wasm-bindgen')
            destination = MemoryPath(command[command.index('--out-dir') + 1])
            raw = files[command[-1]]
            put(destination / 'graphshell_web_bg.wasm', raw)
            put(destination / 'graphshell_web.js', b'fixture binding for earlier bytes')
            operations.append(('synthetic bindgen', command[-1]))
            return SimpleNamespace(returncode=0)
        real_import = builtins.__import__
        def fixture_import(name, *args, **kwargs):
            if name == 'pathlib': return SimpleNamespace(Path=MemoryPath)
            if name == 'subprocess': return SimpleNamespace(run=run, check_output=lambda *a, **k: 'newer-head-B\n')
            if name == 'shutil': return SimpleNamespace(copy2=copy2, copytree=copytree)
            return real_import(name, *args, **kwargs)
        base = MemoryPath('/Users/markik/Code/output/graphshell-site-presenter-20261011')
        repo = MemoryPath('/Users/markik/Code/worktrees/mere-nested-graph-view')
        out = MemoryPath('/fixture/' + variant)
        records = [{'name': 'fixture-' + str(i), 'exit_code': 0,
                    'tests': ['ok. 0 passed; 0 failed; 0 ignored;']} for i in range(gate_count)]
        if condition == 'failed-gate': records[0]['exit_code'] = 1
        if condition == 'missing-gate': records.pop()
        put(out / 'gates.json', json.dumps(records))
        put(out / 'viewer-build.wasm', b'earlier-built-viewer-A')
        put(out / 'product-build.wasm', b'earlier-built-product-A')
        put(repo / 'tracked.rs', b'newer-source-B\n')
        put(repo / 'ports/graphshell/web/index.html', b'<fixture>newer page B</fixture>')
        put(base / 'candidate.json', json.dumps({'source': 'old-head-A',
             'source_byte_sha256': {'tracked.rs': sha256(b'older-source-A\n').hexdigest()}, 'browser_drivers': []}))
        put(base / 'continuation-state.json', '{}')
        put(base / 'assert-console-unlocked.cjs', b'fixture check')
        namespace = {'__name__': 'audit_packager', '__file__': str(out / 'package.py'),
                     '__builtins__': {**vars(builtins), '__import__': fixture_import}}
        rejected = False
        with redirect_stdout(StringIO()):
            try: exec(compile(source, str(actual), 'exec'), namespace)
            except AssertionError: rejected = True
        if condition == 'zero-test-success':
            assert not rejected
            candidate = json.loads(files[str(base / 'candidate.json')])
            continuation = json.loads(files[str(base / 'continuation-state.json')])
            assert candidate['cpu_tests'] == claimed_tests and candidate['packaging_complete'] is True
            assert candidate['source'] == 'newer-head-B'
            assert candidate['source_byte_sha256']['tracked.rs'] == sha256(b'newer-source-B\n').hexdigest()
            assert files[str(base / 'web-viewer/pkg/graphshell_web_bg.wasm')] == b'earlier-built-viewer-A'
            assert 'CPU/Wasm-qualified' in candidate['acceptance_status']
            checks.append({'variant': variant, 'condition': condition, 'returned_normally': True,
                           'actual_fixture_tests': 0, 'claimed_cpu_tests': candidate['cpu_tests'],
                           'new_source_labels_earlier_binary': True,
                           'continuation_flags': continuation})
        else:
            assert rejected and operations == []
            checks.append({'variant': variant, 'condition': condition, 'refused_before_writes': True})
        print('PASS', variant, condition)
OUT.write_text(json.dumps({'checks': checks, 'tests': len(checks), 'source_sha256': sources,
    'diagnostic_sha256': sha256(Path(__file__).read_bytes()).hexdigest(),
    'scope': 'Five exact current historical packaging scripts executed unchanged. Path/filesystem, bindgen/copy and git-head dependencies substituted entirely in memory. No actual source, package, historical receipt, build, browser or deployment was changed. Shows verification weaknesses when rerun; does not establish historical test totals or packages were false at their original basis.',
}, indent=2) + '\n')
