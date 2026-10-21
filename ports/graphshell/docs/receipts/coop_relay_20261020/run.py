#!/usr/bin/env python3
"""Run the existing seven native phases through one owned, observed relay."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import select
import shlex
import subprocess
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / 'coop_mac_thinkpad_20261020'))
import run as base


def option(args, name, default=None):
    return args[args.index(name) + 1] if name in args else default


def main():
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument('--relay-binary', required=True)
    parser.add_argument('--relay-sha256', required=True)
    parser.add_argument('--relay-bind', default='192.168.4.32:0')
    extra, remaining = parser.parse_known_args()
    out = Path(option(remaining, '--output')).resolve()
    host = option(remaining, '--guest-host', 'thinkpad-l14-f')
    ssh = ['ssh', '-S', option(remaining, '--ssh-control'), host]
    check = ['python3', '-c', 'import hashlib,sys;print(hashlib.sha256(open(sys.argv[1],"rb").read()).hexdigest())', extra.relay_binary]
    actual = subprocess.check_output(ssh + [shlex.join(check)], text=True).strip()
    assert actual == extra.relay_sha256
    log_path = out.parent / (out.name + '-relay.log')
    events = []
    with log_path.open('w') as log:
        relay = subprocess.Popen(ssh + [shlex.join([extra.relay_binary, extra.relay_bind])],
                                 stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=log,
                                 text=True, bufsize=1)
        def until(event):
            while True:
                assert select.select([relay.stdout], [], [], 60)[0], 'relay RPC timeout'
                line = relay.stdout.readline()
                assert line, 'relay stopped before ' + event
                item = json.loads(line)
                events.append(item)
                if item['event'] == event:
                    return item
        accepted = False
        try:
            ready = until('ready')
            os.environ['GRAPHSHELL_RELAY_ACCEPTANCE_URL'] = ready['url']
            relay.stdin.write('snapshot\n')
            relay.stdin.flush()
            until('snapshot')
            sys.argv = [str(HERE / 'run.py'), *remaining]
            base.HERE = HERE
            worker_type = base.Worker
            class RelayWorker(worker_type):
                def __init__(self, role, command, log, trace):
                    if role == 'guest':
                        command = [*command[:-1], shlex.join(['env',
                            'GRAPHSHELL_RELAY_ACCEPTANCE_URL=' + ready['url']]) + ' ' + command[-1]]
                    super().__init__(role, command, log, trace)
            base.Worker = RelayWorker
            base.main()
            accepted = True
        finally:
            if relay.poll() is None:
                relay.stdin.write('stop\n')
                relay.stdin.flush()
                until('stopped')
            relay.stdin.close()
            relay.wait(timeout=30)
            result = {'exit': relay.returncode, 'sha256': actual, 'events': events,
                      'bind': extra.relay_bind, 'binary': extra.relay_binary}
            if out.exists():
                (out / 'relay-server.json').write_text(json.dumps(result, indent=2) + '\n')
                if accepted:
                    assert relay.returncode == 0
                    (out / 'accepted/relay-server.json').write_text(json.dumps(result, indent=2) + '\n')
                    (out / 'accepted/relay-server.log').write_bytes(log_path.read_bytes())


if __name__ == '__main__':
    main()
