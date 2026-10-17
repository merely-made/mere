"""Run unchanged current headed-driver main with bounded synthetic dependencies."""
from contextlib import redirect_stdout
from hashlib import sha256
from io import StringIO
import json
from pathlib import Path
import sys
from types import SimpleNamespace
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
OUT = HERE / 'community_headed_exit_probe_receipt.json'
assert not OUT.exists()
source = ROOT / 'ports/graphshell/web/tests/run-headed-receipt.py'
ns = {'__name__': 'audit_headed_driver', '__file__': str(source)}
exec(compile(source.read_bytes(), str(source), 'exec'), ns)
fixture = HERE / 'community_headed_exit_fixture'
fixture.mkdir(exist_ok=False)
web = fixture / 'web'
(web / 'pkg').mkdir(parents=True)
(web / 'scenarios').mkdir()
(web / 'pkg/graphshell_web_bg.wasm').write_bytes(b'synthetic-not-wasm')
(web / 'scenarios/fixture.scn').write_text('synthetic fixture only\n')
tests = []
for verdict in ('fail', 'ok'):
    out = fixture / ('scenario-' + verdict)
    created_processes = []
    calls = []

    class Process:
        pid = 987654
        def __init__(self, args, **kwargs):
            created_processes.append(args)
            if len(created_processes) == 1:
                result = {'scenario': {'result': {'result': verdict}, 'errors': [], 'gate_failures': []}}
                (out / 'result.json').write_text(json.dumps(result))
                (out / 'scenario.done').write_text('RESULT ' + verdict + '\n')
        def poll(self): return None
        def wait(self, timeout): return 0

    class Socket:
        def __enter__(self): return self
        def __exit__(self, *args): pass
        def bind(self, address): calls.append(('bind', address))

    class Response:
        status = 200
        def __enter__(self): return self
        def __exit__(self, *args): pass

    ns['subprocess'] = SimpleNamespace(Popen=Process, run=lambda *a, **k: calls.append(('capture', a)),
                                       TimeoutExpired=TimeoutError)
    ns['socket'] = SimpleNamespace(socket=Socket)
    ns['os'] = SimpleNamespace(environ={}, getloadavg=lambda: [0, 0, 0], getuid=lambda: 1000,
                                killpg=lambda *a: calls.append(('synthetic signal', a)))
    ns['urllib'] = SimpleNamespace(request=SimpleNamespace(urlopen=lambda *a, **k: Response()))
    ns['cpu_ticks'] = lambda: (100, 50)
    ns['time'] = SimpleNamespace(sleep=lambda _: None, monotonic=__import__('time').monotonic)
    argv = ['driver', '--web', str(web), '--out', str(out), '--scenario', 'fixture',
            '--query', '', '--node', str(fixture / 'synthetic-node')]
    stdout = StringIO()
    with patch.object(sys, 'argv', argv), redirect_stdout(stdout):
        returned = ns['main']()
    environment = json.loads((out / 'environment.json').read_text())
    assert returned is None and environment['result'] == verdict
    assert len(created_processes) == 3
    tests.append({'scenario_result': verdict, 'main_returned_normally': True,
                  'stdout': stdout.getvalue(), 'metadata_result': environment['result'],
                  'synthetic_process_count': len(created_processes)})
    print(('COUNTEREXAMPLE' if verdict == 'fail' else 'CONTROL') + ': ' + verdict + ' returns normally')
receipt = {
    'tests': len(tests), 'checks': tests,
    'source_sha256': {'ports/graphshell/web/tests/run-headed-receipt.py': sha256(source.read_bytes()).hexdigest()},
    'diagnostic_sha256': sha256(Path(__file__).read_bytes()).hexdigest(),
    'fixture_files': {p.relative_to(HERE).as_posix(): {'sha256': sha256(p.read_bytes()).hexdigest(), 'bytes': p.stat().st_size}
                      for p in fixture.rglob('*') if p.is_file()},
    'scope': 'Unchanged actual current driver main, synthetic file receipts and substituted process/socket/HTTP/clock/OS dependencies. No browser, process launch, signal, capture, GPU, network or actual headed qualification. Proves failed-scenario status is recorded but does not make this main fail.',
}
OUT.write_text(json.dumps(receipt, indent=2) + '\n')
