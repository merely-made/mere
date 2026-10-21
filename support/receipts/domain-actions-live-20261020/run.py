#!/usr/bin/env python3
"""Real resident/native control/browser acceptance, using existing host harnesses.

The profile and vault are private temporary data. Public evidence contains only
synthetic graph contents, native captures, process exits and a local HTTP visit.
No browser or protocol request is substituted for the native Open control.
"""
import argparse
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import plistlib
import platform
import shutil
import subprocess
import sys
import tempfile
import threading
import time
import uuid


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + '\n')


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--repo', type=Path, required=True)
    parser.add_argument('--artifacts', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    repo, artifacts, out = args.repo.resolve(), args.artifacts.resolve(), args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    sys.path.insert(0, str(repo / 'ports/graphshell/docs/receipts/coop_20261020'))
    from resident import Resident
    sys.path.insert(0, str(repo / 'ports/graphshell/docs/receipts/coop_mac_thinkpad_20261020'))
    from machine import desktop
    _, state = desktop()
    assert state['console_present'] and not state['console_locked'], 'real desktop must be unlocked'
    assert platform.system() == 'Darwin', 'this acceptance packet currently qualifies macOS'
    runtime = Path(tempfile.mkdtemp(prefix='moa-', dir='/tmp'))
    resident = Resident(artifacts / 'djinn', runtime / 'owner', 'LiveOwnerActions', out / 'resident')
    settings = Path(resident.environment['XDG_DATA_HOME']) / 'graphshell/settings/LiveOwnerActions.json'
    owner_settings = {'reservoir': {'persona': str(uuid.uuid4()), 'open_web_addresses': True}}
    write(settings, owner_settings)
    write(out / 'owner-settings.json', owner_settings)
    visits = []
    proof_path = '/owner-action/' + uuid.uuid4().hex

    class Page(BaseHTTPRequestHandler):
        def do_GET(self):
            if self.path == proof_path:
                visits.append({'path': self.path, 'at_ns': time.time_ns(),
                               'user_agent': self.headers.get('User-Agent', '')})
                write(out / 'browser-visits.json', visits)
            body = b'<!doctype html><title>Mere owner action</title><h1>Opened by Graphshell</h1><p>The native control handed this local page to the system browser.</p>'
            self.send_response(200)
            self.send_header('Content-Type', 'text/html; charset=utf-8')
            self.send_header('Content-Length', str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *args):
            pass

    server = ThreadingHTTPServer(('127.0.0.1', 0), Page)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    address = f'http://127.0.0.1:{server.server_port}{proof_path}'
    domain = 'owner-actions-acceptance'
    phases = []
    native = None
    binary = artifacts / 'graphshell_mere'
    helper = artifacts / 'examples/coop_existing_graph'
    bundle = runtime / 'GraphshellOwnerActions.app'
    executable = bundle / 'Contents/MacOS/graphshell_mere'
    executable.parent.mkdir(parents=True)
    shutil.copy2(binary, executable)
    with (bundle / 'Contents/Info.plist').open('wb') as f:
        plistlib.dump({'CFBundleIdentifier': 'made.merely.graphshell.owner-actions.acceptance',
                      'CFBundleName': 'Graphshell owner actions',
                      'CFBundleExecutable': 'graphshell_mere', 'CFBundlePackageType': 'APPL',
                      'CFBundleVersion': '1', 'NSHighResolutionCapable': True}, f)
    subprocess.run(['/usr/bin/codesign', '--force', '--sign', '-', '--timestamp=none', str(bundle)], check=True)
    subprocess.run(['/usr/bin/codesign', '--verify', '--strict', str(bundle)], check=True)

    def prepare(*arguments):
        result = subprocess.run([str(helper), *arguments], env=resident.environment,
                                capture_output=True, text=True, timeout=90, check=True)
        return result.stdout

    def read(name, chosen):
        value = json.loads(prepare('read', domain, chosen))
        write(out / (name + '.json'), value)
        return value

    def history(value, node):
        return value['graphs'][0]['facets']['nodes'].get(node, {}).get(
            'graphshell.access-history/v1', {}).get('records', [])

    def launch(name, scenario):
        nonlocal native
        path = out / (name + '.scn')
        path.write_text(scenario)
        env = resident.environment | {
            'GRAPHSHELL_MERE_SCENARIO': str(path),
            'GRAPHSHELL_MERE_RECEIPT': str(out / (name + '.json')),
            'GRAPHSHELL_MERE_JSON_RECEIPT': '1',
            'GRAPHSHELL_MERE_PRESENTATION_TIMEOUT_SECS': '60', 'RUST_LOG': 'warn',
        }
        with (out / (name + '.log')).open('w') as log:
            native = subprocess.Popen([str(executable), domain], env=env, stdout=log, stderr=log)
            pid = native.pid
            native.wait(timeout=180)
        assert native.returncode == 0, (name, native.returncode)
        receipt = json.loads((out / (name + '.json')).read_text())
        assert receipt['ok'] and not receipt['errors'], receipt
        assert int(receipt['final']['pid']) == pid
        assert receipt['final']['selected'] == node
        assert receipt['captures'] and all(c['width'] > 200 and c['height'] > 200 for c in receipt['captures'])
        phases.append({'name': name, 'pid': pid, 'exit_code': native.returncode})
        native = None
        print(name, 'passed', flush=True)

    try:
        resident.start()
        graphs = json.loads(prepare('seed', domain))
        chosen = graphs['chosen']
        prepare('ordinary-add', domain, chosen, address)
        before = read('before', chosen)
        node = next(n['node_id'] for n in before['graphs'][0]['graph']['nodes'] if n['url'] == address)
        assert history(before, node) == [] and not visits
        selection = (f'await {{"status":"ready","attached":"{chosen}"}}\nwait 30000\n'
                     f'click role:button @data-key={node}\n'
                     f'await {{"selected":"{node}"}}\nwait 30000\n'
                     'wheel 0 10000 px on .mere-record\nsettle 2\n')
        launch('open', selection + f'capture {out}/offered.png\n'
               'click role:button @data-owner-action=graphshell.address.open/system.default\n'
               'await {"notice":"Owner accepted · browser handoff requested."}\nwait 30000\n'
               'wheel 0 10000 px on .mere-record\nsettle 2\n'
               f'capture {out}/accepted.png\n')
        deadline = time.monotonic() + 20
        while not visits and time.monotonic() < deadline:
            time.sleep(.1)
        assert len(visits) == 1, visits
        assert 'Mozilla/' in visits[0]['user_agent'], visits
        after = read('after', chosen)
        saved_history = history(after, node)
        assert len(saved_history) == 1 and saved_history[0]['address'] == address
        assert saved_history[0]['handler'] == 'system.default'
        assert 0 < saved_history[0]['at_ms'] <= visits[0]['at_ns'] // 1_000_000
        resident.stop()
        resident.start()
        cold = read('cold', chosen)
        assert history(cold, node) == saved_history
        launch('reopen', selection + f'capture {out}/reopened.png\n')
        assert len(visits) == 1, 'opening or selecting a record retried its browser effect'
        resident.stop()
        owner_settings['reservoir']['open_web_addresses'] = False
        write(settings, owner_settings)
        resident.start()
        unconfigured = read('unconfigured-owner', chosen)
        for offer in unconfigured['snapshot']['presentation']['offers'].get('mere:' + node, []):
            assert not offer['semantics']['actions']
        launch('disabled', selection + f'capture {out}/disabled.png\n')
        assert len(visits) == 1 and history(read('final', chosen), node) == saved_history
        resident.stop()
        inputs = subprocess.check_output(['git', '-C', str(repo), 'diff', '--name-only'], text=True).splitlines()
        inputs = [p for p in inputs if Path(p).suffix in {'.rs', '.toml'} or Path(p).name == 'Cargo.lock']
        inputs += ['ports/graphshell/mere/src/browser.rs', str(Path(__file__).resolve().relative_to(repo))]
        write(out / 'qualification.json', {
            'status': 'passed', 'source_commit': subprocess.check_output(['git', '-C', str(repo), 'rev-parse', 'HEAD'], text=True).strip(),
            'source_state': 'working source qualified by hashes; not a published commit',
            'source_sha256': {p: sha(repo / p) for p in sorted(set(inputs))},
            'binaries': {'native_artifact': sha(binary), 'native_signed_bundle': sha(executable),
                         'preparation_helper': sha(helper), 'resident': sha(artifacts / 'djinn')},
            'platform': platform.platform(), 'architecture': platform.machine(),
            'real_unlocked_console': True, 'phases': phases, 'resident_starts': resident.starts,
            'address': address, 'node': node, 'session': chosen, 'browser_visits': visits,
            'checks': ['real native retained control', 'exact accepted owner request',
                       'one system browser visit', 'one persisted access-history entry',
                       'cold native and resident reopen without redispatch',
                       'disabled owner setting removes offer without changing access history'],
            'limits': ['macOS only; no Linux or Windows runtime qualification',
                       'retained native harness activation; no physical pointer, keyboard or screen-reader qualification',
                       'system handoff and observed local browser visit; no general page-load or application completion contract'],
        })
        print('Live owner actions acceptance passed', flush=True)
    finally:
        if native is not None and native.poll() is None:
            native.terminate()
            native.wait(timeout=15)
        resident.terminate_owned()
        server.shutdown()
        server.server_close()
        shutil.rmtree(runtime)


if __name__ == '__main__':
    main()
