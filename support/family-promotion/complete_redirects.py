# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Root-only redirects for immutable pre-promotion Mere consumers."""
from pathlib import Path
import json
import subprocess
import tomllib

HERE = Path(__file__).resolve().parent
revisions = json.loads((HERE / 'supplier-revisions.json').read_text())
mapping = json.loads((HERE / 'package-map.json').read_text())
seeds = json.loads((HERE / 'consumer-lock-seeds.json').read_text())
candidates = {Path(row['root']) / Path(row['lock']).parent / 'Cargo.toml' for row in seeds}
candidates.add(HERE.parents[1] / 'Cargo.toml')
roots = set()
for manifest in candidates:
    p = subprocess.run(['cargo', 'locate-project', '--workspace', '--manifest-path', str(manifest), '--message-format', 'json'], cwd=manifest.parent, capture_output=True, text=True)
    if p.returncode:
        raise RuntimeError(p.stderr)
    roots.add(Path(json.loads(p.stdout)['root']))


def inline(owner):
    family = owner[0]
    return '{ git = "https://github.com/merely-made/' + family + '.git", rev = "' + revisions[family] + '" }'


changes = []
for manifest in sorted(roots):
    before = manifest.read_text(encoding='utf-8-sig')
    text = before
    # Cargo does not inherit patch tables from a dependency's workspace.
    for source in ('crates-io', 'https://github.com/merely-made/mere.git', 'https://www.github.com/merely-made/mere.git'):
        data = tomllib.loads(text)
        patch = data.get('patch', {}).get(source, {})
        desired = {name: owner for name, owner in mapping.items() if source == 'crates-io' or owner[0] != 'errand' or name == 'errand'}
        missing = {name: owner for name, owner in desired.items() if name not in patch}
        header = '[patch.crates-io]' if source == 'crates-io' else '[patch."' + source + '"]'
        block = ''.join(name + ' = ' + inline(owner) + '\n' for name, owner in sorted(missing.items()))
        if block:
            if header in text:
                text = text.replace(header + '\n', header + '\n# Promoted sources shared with immutable older Mere suppliers.\n' + block, 1)
            else:
                text += '\n' + header + '\n' + block
    if text != before:
        manifest.write_text(text, encoding='utf-8', newline='\n')
        changes.append(str(manifest))
(HERE / 'completed-root-redirects.json').write_text(json.dumps(changes, indent=2)+'\n', encoding='utf-8')
print('Completed root redirects:', len(changes))
