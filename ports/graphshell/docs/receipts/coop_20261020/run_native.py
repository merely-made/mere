#!/usr/bin/env python3
"""Two real custody identities, one holder, exact graph and cold-return checks.

Pass a frozen graphshell-coop and a current-source Djinn built with passphrase-fd.
Stores, vaults, grants and passwords remain in the private run directory.
"""
import argparse
import hashlib
import json
import os
import plistlib
import shutil
import sys
import uuid
from pathlib import Path
import subprocess
import time
from resident import Resident


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + '\n')


def wait(path, processes=(), seconds=150):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        if path.exists():
            return json.loads(path.read_text())
        for p in processes:
            if p.poll() is not None:
                raise RuntimeError(f'process {p.pid} exited {p.returncode} before {path}')
        time.sleep(.1)
    raise TimeoutError(str(path))


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', required=True, type=Path)
    parser.add_argument('--djinn', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    out = args.output.resolve()
    assert not out.exists(), 'use a fresh run directory'
    out.mkdir(parents=True)
    os.chmod(out, 0o700)
    private = out / 'private'
    public = out / 'evidence'
    public.mkdir()
    owner = Resident(args.djinn, private / 'owner', 'CoopOwner', public / 'owner-resident')
    guest = Resident(args.djinn, private / 'guest', 'CoopGuest', public / 'guest-resident')
    processes = []
    logs = []
    binary = str(args.binary.resolve())
    store = private / 'graphs.redb'
    owner_home = private / 'owner-handoff'
    guest_home = private / 'guest-handoff'
    invitation = owner_home / 'invitation.json'
    contact = owner_home / 'contact.json'
    phases = []
    apps = {}

    def activate(role):
        if sys.platform == 'darwin':
            subprocess.run(['/usr/bin/open', '-a', str(apps[role])], check=True, timeout=10, capture_output=True)

    def command(resident, *arguments):
        return subprocess.run([binary, *map(str, arguments)], env=resident.environment,
                              text=True, capture_output=True, timeout=60, check=True).stdout.strip()

    def step(name, **kw):
        return dict(capture=str(public / f'{name}.png'), **kw)

    def launch(name, resident, role, steps, graph=None):
        handoff = private / name
        write(public / f'{name}-scenario.json', steps)
        cmd = [binary, role, '--handoff', str(handoff), '--scenario', str(public / f'{name}-scenario.json')]
        if role == 'owner':
            cmd += ['--store', str(store), '--graph', str(graph)]
        else:
            cmd += ['--invitation', str(invitation), '--contact', str(contact)]
        stream = (public / f'{name}.log').open('w')
        logs.append(stream)
        environment = resident.environment | {'RUST_LOG': 'warn'}
        if sys.platform == 'darwin':
            app = private / 'apps' / (role + '.app')
            executable = app / 'Contents' / 'MacOS' / 'graphshell-coop'
            if not executable.exists():
                executable.parent.mkdir(parents=True)
                shutil.copy2(binary, executable)
                with (app / 'Contents' / 'Info.plist').open('wb') as file:
                    plistlib.dump({'CFBundleIdentifier': 'made.merely.graphshell.coop.' + uuid.uuid4().hex,
                                  'CFBundleName': 'Graphshell co-op ' + role,
                                  'CFBundleExecutable': 'graphshell-coop',
                                  'CFBundlePackageType': 'APPL', 'CFBundleVersion': '1',
                                  'NSHighResolutionCapable': True}, file)
            assert sha(executable) == sha(Path(binary))
            apps[role] = app
            native_log = public / (name + '-native.log')
            launch = ['/usr/bin/open', '-n', '-W', '--stdout', str(native_log), '--stderr', str(native_log)]
            for key, value in environment.items():
                if key.startswith(('XDG_', 'MERE_', 'GRAPHSHELL_', 'TURNSTONE_')) or key in {'LOCALAPPDATA', 'TMPDIR', 'RUST_LOG'}:
                    launch += ['--env', key + '=' + value]
            cmd = launch + [str(app), '--args', *cmd[1:]]
            write(public / (name + '-launch.json'), {'method': 'LaunchServices', 'binary_sha256': sha(executable)})
        p = subprocess.Popen(cmd, env=environment, stdout=stream, stderr=subprocess.STDOUT)
        processes.append(p)
        return p, handoff

    def finish(name, process, handoff):
        result = wait(handoff / 'complete.json', [process])
        process.wait(timeout=30)
        assert process.returncode == 0
        phases.append(dict(name=name, **result))
        write(public / f'{name}-result.json', result)
        return result

    try:
        owner.start()
        guest.start()
        command(guest, 'contact', '--handoff', guest_home)
        chosen = command(owner, 'new-graph', '--store', store)
        sibling = command(owner, 'new-graph', '--store', store)
        assert chosen != sibling
        # Create the unshared sibling through the same ordinary native control.
        p, h = launch('private-baseline', owner, 'owner', [step('private-baseline', set=['https://private.invalid/owner-only', 'Private sentinel'], click='add', nodes=1, status='Saved')], sibling)
        private_before = finish('private-baseline', p, h)['state']['data']
        # Main owner holds its window open until the guest has cold-returned.
        p_owner, h_owner = launch('owner-live', owner, 'owner', [
            step('owner-created', set=['https://shared.invalid/owner', 'Owner source'], click='add', nodes=1, status='Saved'),
            step('owner-invited', set=['', '', '', str(guest_home / 'identity.json'), str(invitation)], click='invite-editor', nodes=1, status='Invitation saved'),
            step('owner-saw-guest', wait_file=str(public / 'guest-added.json'), click='refresh', nodes=2),
            step('owner-live-append', set=['https://shared.invalid/owner-live', 'Owner live source'], click='add', nodes=3, status='Saved'),
            step('owner-ended-access', wait_file=str(public / 'revoke-now.json'), click='revoke', nodes=3, access='revoked', status='Guest access ended'),
        ], chosen)
        # Reuse authority-free current contact, while invitation stays separate.
        wait(h_owner / 'contact.json', [p_owner])
        contact.parent.mkdir(parents=True, exist_ok=True)
        contact.write_bytes((h_owner / 'contact.json').read_bytes())
        wait(public / 'owner-invited.json', [p_owner])
        p_guest, h_guest = launch('guest-live', guest, 'guest', [
            step('guest-joined', nodes=1),
            step('guest-added', set=['https://shared.invalid/guest', 'Guest source'], click='add', nodes=2, status='Saved by holder'),
            step('guest-saw-push', wait_file=str(public / 'owner-live-append.json'), nodes=3),
        ])
        wait(public / 'guest-added.json', [p_guest, p_owner])
        activate('owner')
        wait(public / 'owner-live-append.json', [p_guest, p_owner])
        activate('guest')
        first_guest = finish('guest-live', p_guest, h_guest)
        wait(public / 'owner-saw-guest.json', [p_owner])
        # A new OS process, same custody identity, before any revocation.
        guest_node = next(n['id'] for n in first_guest['state']['data']['nodes'] if n['address'].endswith('/guest'))
        p_guest, h_guest = launch('guest-return', guest, 'guest', [
            step('guest-reopened', nodes=3),
            step('guest-selected', click='select-' + guest_node, nodes=3),
            step('guest-renamed', set=['', 'Guest source revised'], click='rename', nodes=3, status='Saved by holder'),
            step('guest-access-ended', wait_file=str(public / 'owner-ended-access.json'), status='Unavailable'),
        ])
        wait(public / 'guest-renamed.json', [p_guest, p_owner])
        activate('owner')
        write(public / 'revoke-now.json', {'go': True})
        finish('owner-live', p_owner, h_owner)
        ended = finish('guest-return', p_guest, h_guest)
        assert ended['state']['data'] is None, 'withdrawn client retains no mounted graph'
        # Both apps and BOTH residents disappear. No in-process identity survives.
        owner.stop()
        guest.stop()
        owner.start()
        guest.start()
        p_owner, h_owner = launch('owner-cold', owner, 'owner', [
            step('owner-cold-retained', nodes=3, access='revoked'),
            step('owner-cold-new-edit', set=['https://shared.invalid/after-revoke', 'Owner after revoke'], click='add', nodes=4, status='Saved'),
            step('owner-cold-refused-guest', wait_file=str(public / 'guest-cold-denied.json'), nodes=4, access='revoked'),
        ], chosen)
        wait(h_owner / 'contact.json', [p_owner])
        contact.write_bytes((h_owner / 'contact.json').read_bytes())
        p_guest, h_guest = launch('guest-cold', guest, 'guest', [step('guest-cold-denied', status='Unavailable')])
        denied = finish('guest-cold', p_guest, h_guest)
        cold = finish('owner-cold', p_owner, h_owner)
        assert denied['state']['data'] is None
        # Observe the sibling again, still through its own explicit owner launch.
        p, h = launch('private-return', owner, 'owner', [step('private-unchanged', nodes=1)], sibling)
        private_after = finish('private-return', p, h)['state']['data']
        assert private_before['nodes'] == private_after['nodes']
        assert private_before['authors'] == private_after['authors']
        assert private_before['revision'] == private_after['revision']
        owner_root = first_guest['state']['data']['owner']
        guest_root = first_guest['state']['root']
        assert owner_root != guest_root
        assert any(a['id'] == 'personae:' + guest_root for a in cold['state']['data']['authors'])
        assert any(a['id'] == 'personae:' + owner_root for a in cold['state']['data']['authors'])
        urls = [n['address'] for n in cold['state']['data']['nodes']]
        assert len(urls) == 4 and all(u.startswith('https://shared.invalid/') for u in urls)
        assert next(n['title'] for n in cold['state']['data']['nodes'] if n['id'] == guest_node) == 'Guest source revised'
        owner.stop()
        guest.stop()
        result = {'result': 'ok', 'binary_sha256': sha(args.binary), 'djinn_sha256': sha(args.djinn),
                  'chosen_graph': chosen, 'private_graph': sibling, 'owner_root': owner_root, 'guest_root': guest_root,
                  'phases': phases, 'cold_reopen': {'native_processes_stopped': True, 'residents_stopped': True,
                                                   'same_owner': cold['state']['data']['owner'] == owner_root,
                                                   'revoked_guest_refused': True, 'private_sibling_unchanged': True}}
        write(public / 'qualification.json', result)
        print(json.dumps({k: v for k, v in result.items() if k != 'phases'}, indent=2))
    finally:
        # Stop only executables inside this run's owned bundles. `open -W`
        # is a launcher process, so stopping it alone would leave a window.
        if apps:
            listing = subprocess.check_output(['ps', '-axo', 'pid=,command='], text=True)
            owned = [str((app / 'Contents/MacOS/graphshell-coop').resolve()) for app in apps.values()]
            for line in listing.splitlines():
                pid, _, command_line = line.strip().partition(' ')
                command_line = command_line.lstrip()
                if any(command_line == executable or command_line.startswith(executable + ' ') for executable in owned):
                    try:
                        os.kill(int(pid), 15)
                    except ProcessLookupError:
                        pass
        for p in processes:
            if p.poll() is None:
                p.terminate()
                p.wait(timeout=15)
        owner.terminate_owned()
        guest.terminate_owned()
        for stream in logs:
            stream.close()


if __name__ == '__main__':
    main()
