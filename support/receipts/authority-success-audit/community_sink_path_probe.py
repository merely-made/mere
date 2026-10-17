"""Exercise the exact receipt Handler class with in-memory path/write captures."""
import ast
import base64
from email.message import Message
from hashlib import sha256
from http.server import SimpleHTTPRequestHandler
from io import BytesIO, StringIO
import json
from pathlib import Path
from types import SimpleNamespace

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
OUT = HERE / 'community_sink_path_probe_receipt.json'
assert not OUT.exists()
path = ROOT / 'ports/graphshell/web/tests/receipt_sink.py'
source = path.read_text(encoding='utf-8')
tree = ast.parse(source)
main = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == 'main')
handler = next(n for n in main.body if isinstance(n, ast.ClassDef) and n.name == 'Handler')
module = ast.Module(body=[handler], type_ignores=[])
tests = []
for name, origin in [('ordinary', 'http://127.0.0.1:8731'), ('../escaped', 'http://127.0.0.1:8731'),
                     ('../escaped', 'http://refused.invalid')]:
    writes = []
    output = (HERE / 'synthetic-output-never-created').resolve()
    class CapturePath:
        def __init__(self, p): self.p = p
        def __truediv__(self, name): return CapturePath(self.p / name)
        def write_bytes(self, data): writes.append((str(self.p.resolve()), len(data)))
        def write_text(self, data, **kwargs): writes.append((str(self.p.resolve()), len(data)))
    env = {'SimpleHTTPRequestHandler': SimpleHTTPRequestHandler, 'base64': base64, 'json': json,
           'sys': SimpleNamespace(stderr=StringIO()), 'allowed_origin': 'http://127.0.0.1:8731',
           'out': CapturePath(output)}
    exec(compile(module, str(path), 'exec'), env)
    instance = object.__new__(env['Handler'])
    instance.path = '/scenario-receipt'
    instance.headers = Message()
    instance.headers['Origin'] = origin
    instance.headers['Content-Type'] = 'application/json'
    body = json.dumps({'captures': [{'name': name, 'dataUrl': 'data:image/png;base64,AA=='}],
                       'scenario': {'result': {'result': 'ok'}}}).encode()
    instance.headers['Content-Length'] = str(len(body))
    instance.rfile = BytesIO(body)
    statuses = []
    instance.send_response = statuses.append
    instance.send_error = statuses.append
    instance.end_headers = lambda: None
    instance.do_POST()
    escapes = [p for p, _ in writes if not Path(p).is_relative_to(output)]
    if origin == 'http://refused.invalid': assert statuses == [403] and not writes
    elif name == '../escaped': assert statuses == [204] and len(escapes) == 1
    else: assert statuses == [204] and not escapes
    tests.append({'name': name, 'origin': origin, 'statuses': statuses,
                  'attempted_writes': writes, 'paths_outside_output': escapes})
    print(('COUNTEREXAMPLE' if escapes else 'CONTROL') + ': ' + name + ' -> ' + str(statuses))
OUT.write_text(json.dumps({
    'tests': len(tests), 'checks': tests,
    'source_sha256': {'ports/graphshell/web/tests/receipt_sink.py': sha256(path.read_bytes()).hexdigest()},
    'exact_class_lines': [handler.lineno, handler.end_lineno],
    'diagnostic_sha256': sha256(Path(__file__).read_bytes()).hexdigest(),
    'scope': 'Exact unchanged current nested Handler AST, executed with its outer values supplied. Synthetic headers and payload; in-memory path/write intercepts only, no files written by the Handler and no server/network. Shows an allowed-origin capture name escapes --out; wrong-origin control still refuses.',
}, indent=2) + '\n')
