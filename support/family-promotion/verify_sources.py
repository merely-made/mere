# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0

"""Verify every promoted Rust source against its recorded donor bytes."""
import hashlib
import json
from pathlib import Path

BASE = Path(r'C:\Users\mark_\Code\repos')
HERE = BASE / 'mere/support/family-promotion'
inventory = json.loads((HERE / 'source-inventory.json').read_text(encoding='utf-8'))
mapping = json.loads((HERE / 'package-map.json').read_text(encoding='utf-8'))
errors = []
count = 0
for source, sha in inventory['files'].items():
    if not source.endswith('.rs'):
        continue
    owner = next((v for v in mapping.values() if source.startswith(v[1] + '/')), None)
    if owner is None:
        errors.append('unmapped ' + source)
        continue
    family, src, dst, _ = owner
    donor = BASE / 'mere' / source
    if donor.exists():
        raw = donor.read_bytes()
    else:
        import subprocess
        raw = subprocess.check_output(['git', 'show', inventory['source_revision'] + ':' + source], cwd=BASE / 'mere')
    if donor.exists() and hashlib.sha256(raw).hexdigest() != sha:
        errors.append('donor changed ' + source)
    target = BASE / ('smolweb' if family == 'errand' else family) / dst / source[len(src):].lstrip('/')
    if source == 'crates/intel/esp/tests/lexical_ngram_recall.rs':
        target = BASE / 'mere/crates/intel/eidetic-search/tests/lexical_ngram_recall.rs'
        raw = raw.replace(b'cargo test -p esp', b'cargo test -p eidetic-search')
    if source.startswith('crates/moot/gemot/examples/capsule-library'):
        target = BASE / 'mere/ports/moot/examples' / source.split('/examples/', 1)[1]
        raw = raw.replace(b'cargo run -p gemot', b'cargo run -p mere-moot').replace(b'identity::', b'personae::').replace(b'../../../../../script/wit', b'../../../../../crates/script/wit')
    if not target.exists():
        errors.append('missing ' + str(target))
    elif raw.replace(b'\r\n', b'\n') != target.read_bytes().replace(b'\r\n', b'\n'):
        errors.append('changed ' + source)
    count += 1
report = {'source_revision': inventory['source_revision'], 'rust_files_verified': count, 'errors': errors,
          'allowed_changes': ['line-ending normalization', 'integration target receiving package and WIT path']}
(HERE / 'source-equivalence-final.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
print(json.dumps(report))
raise SystemExit(bool(errors))
