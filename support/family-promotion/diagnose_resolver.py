# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Bound one resolver trace; retain only its tail and rejection counts."""
from collections import Counter, deque
import json
import os
from pathlib import Path
import subprocess
import threading

here = Path(__file__).resolve().parent
env = os.environ.copy()
env['CARGO_LOG'] = 'cargo=trace'
env['CARGO_TARGET_DIR'] = r'C:\t\cargo-targets\mere'
env['CARGO_BUILD_BUILD_DIR'] = r'C:\t\cargo-targets\mere\build'
p = subprocess.Popen(['cargo', 'generate-lockfile', '--offline'], cwd=here.parents[1],
                     env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                     text=True, errors='replace')
timer = threading.Timer(40, lambda: subprocess.run(['taskkill', '/PID', str(p.pid), '/T', '/F'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL))
timer.start()
tail = deque(maxlen=120)
counts = Counter()
for line in p.stdout:
    tail.append(line)
    if 'conflict' in line.lower() or 'reject' in line.lower():
        counts[line.strip().split('cargo::core::resolver:', 1)[-1]] += 1
code = p.wait()
timer.cancel()
report = {'exit_code': code, 'bounded_seconds': 40, 'frequent_rejections': counts.most_common(15), 'tail': ''.join(tail)[-50000:]}
(here / 'resolver-diagnostic.json').write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
print(json.dumps(report, indent=2)[-16000:])
