#!/usr/bin/env python3
"""Native retained page text, headed: keep three local pages in graphshell-mere,
cold-reopen the resident, run a body-only goal through the door.

Isolated real Djinn (passphrase-held vault), the ordinary graphshell_mere window
on the shared Mesquite lane, a local fixture server that records every request.
"""
import argparse, hashlib, http.server, json, os, plistlib, shutil, signal, subprocess, sys, threading, time, uuid
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / 'coop_20261020'))
from resident import Resident  # noqa: E402

PAGES = {
    '/harbour': ('Harbour notes', 'The fog signal sounds two blasts every thirty seconds when visibility drops below a mile.'),
    '/tides': ('Tide tables', 'Spring tides follow the new moon and the full moon; neap tides arrive at the quarters.'),
    '/shoreline': ('Shoreline field guide', 'Bull kelp anchors on rock with a holdfast, and its stipe can reach thirty metres.'),
}
GOAL = 'fog signal blasts'  # in one page's body only; in no title or address


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + '\n')


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def until(path, processes=(), seconds=240):
    end = time.monotonic() + seconds
    while time.monotonic() < end:
        if Path(path).exists():
            return
        for process in processes:
            assert process.poll() is None, f'{process.pid} exited {process.returncode} before {path}'
        time.sleep(.1)
    raise TimeoutError(path)


def await_(**fields):
    return 'await ' + json.dumps(fields) + '\nwait 30000\n'


def click(id_):
    return 'click role:button @id=' + id_ + '\n'


def type_(id_, text):
    return f'click role:textbox @id={id_}\ninput-text {text}\n'


def capture(path):
    return 'settle 3\ncapture ' + str(path) + '\n'


class Fixture(http.server.ThreadingHTTPServer):
    def __init__(self):
        self.log = []
        outer = self

        class Handler(http.server.BaseHTTPRequestHandler):
            def do_GET(self):
                outer.log.append({'path': self.path, 'agent': self.headers.get('User-Agent', '')})
                page = PAGES.get(self.path)
                if page is None:
                    self.send_response(404)
                    self.end_headers()
                    return
                title, body = page
                html = f'<!doctype html><title>{title}</title><style>.x{{color:red}}</style><body><h1>{title}</h1><p>{body}</p><script>var hidden = 1;</script></body>'.encode()
                self.send_response(200)
                self.send_header('Content-Type', 'text/html; charset=utf-8')
                self.send_header('Content-Length', str(len(html)))
                self.end_headers()
                self.wfile.write(html)

            def log_message(self, *args):
                pass

        super().__init__(('127.0.0.1', 0), Handler)
        threading.Thread(target=self.serve_forever, daemon=True).start()

    @property
    def base(self):
        return f'http://127.0.0.1:{self.server_address[1]}'


def main():
    parser = argparse.ArgumentParser()
    for key in ['binary', 'djinn', 'prepare', 'lock', 'output']:
        parser.add_argument('--' + key, required=True, type=Path)
    args = parser.parse_args()
    out = args.output.resolve()
    assert not out.exists()
    out.mkdir(mode=0o700, parents=True)
    public = out / 'evidence'
    public.mkdir()
    private = out / 'private'
    private.mkdir()
    server = Fixture()
    resident = Resident(args.djinn, private / 'owner', 'NativeRetained', public / 'resident')
    # The owner's yes for the two declared applications, written as Djinn writes a yes it was given.
    grants = json.loads(subprocess.run([str(args.lock.resolve()), 'grants'], env=resident.environment,
                                       capture_output=True, text=True, check=True, timeout=60).stdout)
    write(Path(resident.environment['XDG_DATA_HOME']) / 'graphshell/settings/NativeRetained.json',
          {'reservoir': {'enabled': True, 'persona': str(uuid.uuid4())}, 'applications': grants})
    domain = 'browsing'
    processes = []
    app = private / 'owner.app'
    exe = app / 'Contents/MacOS/graphshell_mere'

    def launch(name, script):
        scenario = public / (name + '.scn')
        scenario.write_text(script)
        env = resident.environment | {
            'GRAPHSHELL_MERE_SCENARIO': str(scenario), 'GRAPHSHELL_MERE_RECEIPT': str(public / (name + '.json')),
            'GRAPHSHELL_MERE_JSON_RECEIPT': '1', 'GRAPHSHELL_MERE_PRESENTATION_TIMEOUT_SECS': '60',
            'GRAPHSHELL_MERE_WIDTH': '1280', 'GRAPHSHELL_MERE_HEIGHT': '1180', 'RUST_LOG': 'warn'}
        if not exe.exists():
            exe.parent.mkdir(parents=True)
            shutil.copy2(args.binary, exe)
            with (app / 'Contents/Info.plist').open('wb') as f:
                plistlib.dump({'CFBundleIdentifier': 'made.merely.graphshell.view.kept.' + uuid.uuid4().hex,
                               'CFBundleName': 'Graphshell kept', 'CFBundleExecutable': 'graphshell_mere',
                               'CFBundlePackageType': 'APPL', 'CFBundleVersion': '1', 'NSHighResolutionCapable': True}, f)
        assert sha(exe) == sha(args.binary)
        cmd = ['/usr/bin/open', '-n', '-W', '--stdout', str(public / (name + '.log')), '--stderr', str(public / (name + '.log'))]
        for key, value in env.items():
            if key.startswith(('XDG_', 'MERE_', 'GRAPHSHELL_', 'TURNSTONE_')) or key in {'LOCALAPPDATA', 'TMPDIR', 'RUST_LOG'}:
                cmd += ['--env', key + '=' + value]
        cmd += ['-a', str(app), '--args', domain]
        p = subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        processes.append(p)
        return p

    def finish(name, p):
        until(public / (name + '.json'), (p,))
        p.wait(timeout=30)
        receipt = json.loads((public / (name + '.json')).read_text())
        assert receipt['ok'], receipt
        assert receipt['captures'] and all(c['width'] > 200 and c['height'] > 200 for c in receipt['captures'])
        return receipt

    try:
        resident.start()
        env = resident.environment
        seeded = subprocess.run([str(args.prepare.resolve()), 'seed', domain], env=env, capture_output=True, text=True, timeout=90)
        if seeded.returncode:
            sys.stderr.write(seeded.stderr[-3000:])
        seeded.check_returncode()
        write(public / 'seed.json', json.loads(seeded.stdout))
        urls = [server.base + path for path in PAGES]
        keep = ''.join(type_('kept-address', url) + click('kept-keep') + await_(**{'kept-pages': str(i + 1), 'kept-busy': 'false'}) for i, url in enumerate(urls))
        # Phase 1: nothing kept; the goal finds nothing; keep three pages; the goal cites a body.
        one = (await_(status='ready') + click('kept-open')
               + await_(**{'kept-open': 'true', 'kept-state': '~off. Nothing is being kept.'})
               + capture(public / '01_off_nothing_kept.png')
               + type_('goal-input', GOAL) + click('goal-run') + 'settle 3\n'
               + capture(public / '02_before_keeping_goal_finds_nothing.png')
               + click('kept-toggle') + await_(**{'kept-state': '~on. 0 pages'})
               + keep
               + await_(**{'kept-state': '~on. 3 pages kept.'})
               + capture(public / '03_three_pages_kept.png')
               + click('goal-run') + await_(**{'kept-found': '~/harbour'})
               + capture(public / '04_after_keeping_goal_cites_body.png'))
        p = launch('keep', one)
        finish('keep', p)
        # Phase 1b: lock the vault through its custody door; a fresh viewer shows the pause,
        # and a keep attempted while locked is refused and writes nothing.
        locked = subprocess.run([str(args.lock.resolve()), 'lock'], env=resident.environment, capture_output=True, text=True, timeout=30)
        write(public / 'lock.json', {'returncode': locked.returncode, 'stdout': locked.stdout, 'stderr': locked.stderr[-2000:]})
        locked.check_returncode()
        three = (await_(status='ready') + click('kept-open')
                 + await_(**{'kept-open': 'true', 'kept-state': '~paused'})
                 + capture(public / '05_vault_locked_keeping_paused.png')
                 + type_('kept-address', urls[0]) + click('kept-keep')
                 + await_(**{'kept-busy': 'false', 'kept-notice': '~paused'})
                 + capture(public / '06_locked_keep_refused.png'))
        p = launch('locked', three)
        finish('locked', p)
        resident.stop()
        # Phase 2: a cold resident and a fresh viewer process.
        resident.start()
        two = (await_(status='ready') + click('kept-open')
               + await_(**{'kept-state': '~on. 3 pages kept.', 'kept-busy': 'false'})
               + type_('goal-input', GOAL) + click('goal-run') + await_(**{'kept-found': '~/harbour'})
               + capture(public / '07_cold_reopen_goal_cites_body.png'))
        p = launch('reopen', two)
        finish('reopen', p)
        stores = sorted(str(path.relative_to(private)) for path in (private / 'owner').rglob('retained.redb'))
        stored = [path for path in (private / 'owner').rglob('retained.redb')]
        plaintext = [GOAL.split()[0] in path.read_bytes().decode('latin-1') or 'fog signal' in path.read_bytes().decode('latin-1') for path in stored]
        requested = [entry['path'] for entry in server.log]
        write(public / 'server-requests.json', server.log)
        assert requested == list(PAGES) + ['/harbour'], requested  # three explicit fetches and the one refused while locked; nothing unrequested
        resident.stop()
        keep_receipt = json.loads((public / 'keep.json').read_text())
        reopen_receipt = json.loads((public / 'reopen.json').read_text())
        write(public / 'qualification.json', {
            'result': 'ok', 'goal': GOAL, 'pages': urls, 'server_requests': requested,
            'retained_stores': stores, 'store_holds_plaintext_at_rest': plaintext,
            'native_processes': [int(keep_receipt['final']['pid']), int(reopen_receipt['final']['pid'])],
            'binary_sha256': sha(args.binary), 'djinn_sha256': sha(args.djinn), 'prepare_sha256': sha(args.prepare), 'lock_sha256': sha(args.lock),
            'custodian_launches': resident.starts})
        print(json.dumps({'result': 'ok', 'evidence': str(public)}))
    finally:
        table = subprocess.check_output(['ps', '-axo', 'pid=,command='], text=True)
        for line in table.splitlines():
            fields = line.strip().split(None, 1)
            if len(fields) == 2 and fields[1].startswith(str(exe)):
                try:
                    os.kill(int(fields[0]), signal.SIGTERM)
                except ProcessLookupError:
                    pass
        for p in processes:
            if p.poll() is None:
                p.terminate()
        resident.terminate_owned()
        server.shutdown()


if __name__ == '__main__':
    main()
