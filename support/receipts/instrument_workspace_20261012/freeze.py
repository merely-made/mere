# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Freeze already-built candidates; this does not build or execute acceptance."""
from pathlib import Path
import hashlib
import json
import os
import shutil
import subprocess

repo = Path(__file__).resolve().parents[3]
base = Path(os.environ.get('INSTRUMENT_PACKAGE', '/Users/markik/Code/output/instrument-workspace-20261012'))
target = Path('/Users/markik/Code/targets/knot-tabard-apps')
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=repo, text=True).strip()
assert head == os.environ['INSTRUMENT_BUILD_SOURCE'], 'Supply the actual successful build source'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
previous = json.loads((base / 'candidate.json').read_text())
for source, destination in [('candidate.json', 'candidate-r5.json'),
                            ('native-candidate.json', 'native-candidate-r3.json'),
                            ('native-commands.json', 'native-commands-r3.json'),
                            ('native-release.json', 'native-release-r3.json')]:
    assert not (base / destination).exists(), 'Do not overwrite an earlier frozen receipt'
    shutil.copy2(base / source, base / destination)
subprocess.run(['/Users/markik/.cargo/bin/wasm-bindgen', '--target', 'web', '--remove-name-section',
                '--out-name', 'graphshell_web', '--out-dir', str(base / 'web/pkg'),
                str(target / 'wasm32-unknown-unknown/debug/graphshell_web.wasm')], check=True)
files = subprocess.check_output(['git', 'ls-files', 'crates/cambium', 'ports/graphshell/src'], cwd=repo, text=True).splitlines()
paths = set(previous['source_sha256']) | {p for p in files if Path(p).suffix in ['.rs', '.css', '.wgsl']
                                        or Path(p).name in ['Cargo.toml', 'Cargo.lock']}
assert not subprocess.check_output(['git', 'diff', 'HEAD', '--', *sorted(paths)], cwd=repo), 'Compiled source must be committed'
candidate = {**previous, 'source': head, 'build_source': head, 'post_build_changes': 'No compiled runtime input changes',
    'wasm_bindgen': subprocess.check_output(['/Users/markik/.cargo/bin/wasm-bindgen', '--version'], text=True).strip(),
    'source_sha256': {p: sha(repo / p) for p in sorted(paths)},
    'package_sha256': {str(p.relative_to(base)): sha(p) for p in sorted((base / 'web').rglob('*')) if p.is_file()},
    'browser_executed': False, 'native_executed': False}
(base / 'candidate.json').write_text(json.dumps(candidate, indent=2) + '\n')
binary = base / 'frozen/tabard-desktop'
shutil.copy2(target / 'debug/tabard-desktop', binary)
files = subprocess.check_output(['git', 'ls-files', 'crates/cambium', 'ports/tabard', 'crates/genet',
    'Cargo.toml', 'Cargo.lock', 'scripts/run_macos_scenario.py'], cwd=repo, text=True).splitlines()
paths = [p for p in files if (repo / p).is_file() and (Path(p).suffix in ['.rs', '.css', '.js', '.html', '.wgsl', '.scn', '.py']
                                                    or Path(p).name in ['Cargo.toml', 'Cargo.lock'])]
assert not subprocess.check_output(['git', 'diff', 'HEAD', '--', *paths], cwd=repo), 'Native inputs must be committed'
(base / 'native-candidate.json').write_text(json.dumps({'source': head, 'build_source': head,
    'compiler': 'rustc 1.97.1', 'repo': str(repo), 'binary': str(binary), 'binary_sha256': sha(binary),
    'source_sha256': {p: sha(repo / p) for p in paths}, 'native_executed': False}, indent=2) + '\n')
print(json.dumps({'source': head, 'browser_inputs': len(candidate['source_sha256']), 'native_hash': sha(binary)}))
