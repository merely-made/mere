# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Two serial, source-bound native cases; caller must own the runtime lane."""
from pathlib import Path
import hashlib
import json
import os
import plistlib
import subprocess
import time

repo = Path(__file__).resolve().parents[3]
base = Path(os.environ.get('INSTRUMENT_PACKAGE', '/Users/markik/Code/output/instrument-workspace-20261012'))
manifest = json.loads((base / 'native-candidate.json').read_text())
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(Path(manifest['binary'])) == manifest['binary_sha256']
for name, digest in manifest['source_sha256'].items():
    assert sha(repo / name) == digest, name
console = plistlib.loads(subprocess.check_output(['/usr/sbin/ioreg', '-n', 'Root', '-d1', '-a']))
assert not console.get('IOConsoleLocked', False)
assert not any(s.get('CGSSessionScreenIsLocked', False) for s in console.get('IOConsoleUsers', []))
profile = base / 'profile'
profile.mkdir(exist_ok=True)
commands = []
for label, scenario in [('native-instrument-r4', 'instrument'), ('native-usability-r3', 'usability')]:
    library = profile / (label + '.json')
    assert not library.exists(), 'Fresh authored library required'
    commands.append({'label': label, 'command': [
        '/Library/Developer/CommandLineTools/usr/bin/python3', str(repo / 'scripts/run_macos_scenario.py'),
        '--binary', manifest['binary'], '--prefix', 'TABARD',
        '--scenario', str(repo / f'ports/tabard/desktop/scenarios/{scenario}.scn'),
        '--output', str(base / label), '--timeout', '120',
        '--env', 'TABARD_WIDTH=1280', '--env', 'TABARD_HEIGHT=900',
        '--', '--library', str(library)]})
(base / 'native-commands.json').write_text(json.dumps(commands, indent=2) + '\n')
inhibit = subprocess.Popen(['/usr/bin/caffeinate', '-i', '-s', '-t', '300'])
started = time.monotonic()
results = []
try:
    for run in commands:
        with (base / (run['label'] + '-launch.log')).open('w') as log:
            result = subprocess.run(run['command'], stdout=log, stderr=subprocess.STDOUT, timeout=130)
        results.append({'label': run['label'], 'exit': result.returncode})
        print(json.dumps(results[-1]), flush=True)
        if result.returncode:
            break
finally:
    inhibit.terminate()
    inhibit.wait(timeout=5)
    (base / 'native-release.json').write_text(json.dumps({
        'released_at': time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()),
        'elapsed_seconds': round(time.monotonic() - started, 3),
        'inhibitor_closed': True, 'runs': results}, indent=2) + '\n')
raise SystemExit(any(r['exit'] for r in results))
