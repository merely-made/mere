# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Run one named migration gate with bounded retained diagnostics."""
from collections import deque
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import subprocess
import sys

name, cwd, *command = sys.argv[1:]
if not name.replace('-', '').replace('_', '').isalnum():
    raise SystemExit('Gate names must be simple identifiers')
here = Path(__file__).resolve().parent
receipt = here / (name + '.json')
if receipt.exists():
    raise SystemExit('Receipt exists; use a distinct gate name')
env = os.environ.copy()
repo = env.get('MERE_AUDIT_TARGET_REPO', 'mere')
if repo not in ('mere', 'genet'):
    raise SystemExit('Only approved Mere/Genet target roots are supported')
env['CARGO_TARGET_DIR'] = rf'C:\t\cargo-targets\{repo}'
env['CARGO_BUILD_BUILD_DIR'] = env['CARGO_TARGET_DIR'] + r'\build'
env['CARGO_BUILD_JOBS'] = '4'
started = datetime.now(timezone.utc).isoformat()
tail = deque(maxlen=240)
results = []
process = subprocess.Popen(command, cwd=cwd, env=env, stdout=subprocess.PIPE,
                           stderr=subprocess.STDOUT, text=True, errors='replace')
for line in process.stdout:
    tail.append(line)
    if any(marker in line for marker in ('test result:', 'error:', 'error[', 'Finished ', 'Running ', 'Updating git', 'Blocking waiting', 'has been running for over', 'FAILED')):
        print(line.rstrip(), flush=True)
        results.append(line.strip())
code = process.wait()
receipt.write_text(json.dumps({'command': command, 'cwd': cwd, 'started': started,
    'finished': datetime.now(timezone.utc).isoformat(), 'exit_code': code,
    'results': results[-150:], 'diagnostic_tail': ''.join(tail)[-60000:],
    'target': env['CARGO_TARGET_DIR']}, indent=2) + '\n', encoding='utf-8')
print(f'{name}: exit {code}; compact receipt {receipt}', flush=True)
raise SystemExit(code)
