#!/usr/bin/env python3
"""Reuse the qualified native machine owner; constrain only the Mac custodian."""
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / 'coop_mac_thinkpad_20261020'))
import machine as base

POLICY = '(version 1)(allow default)(deny network-outbound (remote udp "*:*"))'
PROBE = '''import json,socket,sys
from urllib.parse import urlsplit
r={}
for name,family,address in [('ipv4',socket.AF_INET,('127.0.0.1',9)),('ipv6',socket.AF_INET6,('::1',9))]:
 s=socket.socket(family,socket.SOCK_DGRAM)
 try:s.sendto(b'relay-constraint-probe',address);r[name]='sent'
 except PermissionError:r[name]='denied'
 finally:s.close()
u=urlsplit(sys.argv[1])
with socket.create_connection((u.hostname,u.port),timeout=5):r['relay_tcp']='connected'
assert r=={'ipv4':'denied','ipv6':'denied','relay_tcp':'connected'},r
print(json.dumps(r))
'''


class RelayMachine(base.Machine):
    def __init__(self, args):
        super().__init__(args)
        if args.role == 'owner':
            assert sys.platform == 'darwin'
            relay = os.environ['GRAPHSHELL_RELAY_ACCEPTANCE_URL']
            path = Path(self.resident.environment['XDG_DATA_HOME']) / 'graphshell/settings/CrossGraphOwner.json'
            settings = json.loads(path.read_text())
            settings['coop_relay_urls'] = [relay]
            base.write(path, settings)
            base.write(self.public / 'owner-settings.json', settings)
            wrapper = self.private / 'custodian-launch'
            wrapper.write_text('#!/bin/sh\nexec /usr/bin/sandbox-exec -p ' + shlex.quote(POLICY)
                               + ' ' + shlex.quote(str(args.djinn.resolve())) + ' "$@"\n')
            wrapper.chmod(0o700)
            self.resident.binary = wrapper
        base.write(self.public / (args.role + '-machine.json'), self.info())

    def info(self):
        result = super().info()
        result['base_worker_sha256'] = result['worker_sha256']
        result['worker_sha256'] = base.sha(Path(__file__))
        if self.args.role == 'owner':
            result['co_op_relay'] = os.environ['GRAPHSHELL_RELAY_ACCEPTANCE_URL']
            result['custodian_network_policy'] = POLICY
        return result

    def start(self):
        if self.args.role == 'owner':
            probe = subprocess.run(['/usr/bin/sandbox-exec', '-p', POLICY, sys.executable,
                                    '-c', PROBE, os.environ['GRAPHSHELL_RELAY_ACCEPTANCE_URL']],
                                   capture_output=True, text=True, check=True)
        result = super().start()
        if self.args.role == 'owner':
            base.write(self.public / f'owner-network-constraint-{self.resident.starts}.json',
                       {'policy': POLICY, 'probe': json.loads(probe.stdout),
                        'custodian_pid': self.resident.process.pid,
                        'launcher_sha256': base.sha(self.resident.binary)})
        return result

    def finish(self, name):
        result = super().finish(name)
        if self.args.role == 'guest':
            for capture in result['captures']:
                update = json.loads(capture['fields']['coop'])
                paths = [p for p in update['connection'] if p.startswith('active ')]
                assert all(p.startswith('active relay ') for p in paths), paths
                if update['data'] and Path(capture['path']).stem != 'guest-start':
                    assert paths, 'no observed relay path for an admitted guest capture'
        return result


base.Machine = RelayMachine
if __name__ == '__main__':
    base.main()
