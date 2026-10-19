# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Resolve consumer locks and assert one canonical source per promoted crate."""
from pathlib import Path
import json
import os
import subprocess

HERE = Path(__file__).resolve().parent
mapping = json.loads((HERE / 'package-map.json').read_text())
revisions = json.loads((HERE / 'supplier-revisions.json').read_text())
seeds = json.loads((HERE / 'consumer-lock-seeds.json').read_text())
roots = {Path(row['root']) / Path(row['lock']).parent for row in seeds}
roots.add(HERE.parents[1])
# The collision worktree was retired after publication; replay on its integrated primary.
retired = HERE.parents[3] / 'worktrees/isocosm-family-promotion'
primary = HERE.parents[2] / 'isocosm'
roots = {primary / p.relative_to(retired) if p.is_relative_to(retired) and not p.exists() else p for p in roots}
env = os.environ.copy()
env['CARGO_TARGET_DIR'] = r'C:\t\cargo-targets\mere'
env['CARGO_BUILD_BUILD_DIR'] = r'C:\t\cargo-targets\mere\build'
report_path = HERE / 'consumer-source-validation.json'
previous = json.loads(report_path.read_text()) if os.environ.get('FAMILY_PROMOTION_RETRY_FAILED') and report_path.exists() else []
results = [r for r in previous if not (r['exit_code'] or r.get('locked_exit_code') or r.get('source_issues'))]
if previous:
    roots = {Path(r['root']) for r in previous if r not in results}
for root in sorted(roots, key=lambda p: (p != HERE.parents[1], str(p))):
    manifest = root / 'Cargo.toml'
    command = ['cargo', 'metadata', '--manifest-path', str(manifest), '--format-version', '1', '--offline']
    print('Resolving ' + str(root), flush=True)
    p = subprocess.run(command, cwd=root, env=env, capture_output=True, text=True, errors='replace')
    if p.returncode and ('offline' in p.stderr or 'HTTP request' in p.stderr):
        print('Warming missing supplier sources online', flush=True)
        command.remove('--offline')
        p = subprocess.run(command, cwd=root, env=env, capture_output=True, text=True, errors='replace')
    record = {'root': str(root), 'exit_code': p.returncode, 'diagnostics': p.stderr[-18000:]}
    if p.returncode == 0:
        data = json.loads(p.stdout)
        sources = {}
        for package in data['packages']:
            if package['name'] in mapping:
                sources.setdefault(package['name'], set()).add(package.get('source') or 'path:' + package['manifest_path'])
        issues = []
        for name, values in sources.items():
            family = mapping[name][0]
            if len(values) != 1 or not all(f'https://github.com/merely-made/{family}.git?rev={revisions[family]}#' in value for value in values):
                issues.append({'package': name, 'sources': sorted(values)})
        record.update(packages=len(data['packages']), promoted_sources={k: sorted(v) for k, v in sources.items()}, source_issues=issues)
        locked = subprocess.run(command + ['--locked'], cwd=root, env=env, capture_output=True, text=True, errors='replace')
        record['locked_exit_code'] = locked.returncode
        record['locked_diagnostics'] = locked.stderr[-8000:]
        print(f"Resolved {len(data['packages'])} packages; source issues {len(issues)}; locked exit {locked.returncode}", flush=True)
    else:
        print('Resolution failed: ' + p.stderr[-1600:], flush=True)
    results.append(record)
    (HERE / 'consumer-source-validation.json').write_text(json.dumps(results, indent=2) + '\n', encoding='utf-8')
failed = [r for r in results if r['exit_code'] or r.get('locked_exit_code') or r.get('source_issues')]
print(f'Consumer roots: {len(results)}; unresolved roots: {len(failed)}', flush=True)
raise SystemExit(bool(failed))
