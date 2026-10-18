#!/usr/bin/env python3
"""Verify supplier evidence; do not open any resident store."""
from pathlib import Path
import hashlib
import json
import subprocess
packet = Path(__file__).resolve().parent
repo = packet.parents[3]
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
read = lambda p: json.loads((packet / p).read_text())
for path, expected in read('artifact-hashes.json').items():
    assert sha(packet / path) == expected, path
q = read('qualification.json')
assert all(gate['exit'] == 0 and gate['failed'] == 0 for gate in q['gates'])
for path, expected in q['inputs_sha256'].items():
    assert sha(repo / path) == expected, path
    assert hashlib.sha256(subprocess.check_output(['git','show',q['supplier_source']+':'+path],cwd=repo)).hexdigest() == expected
prior = read('mere-preservation-before.json')
for path, expected in prior['files'].items():
    assert sha(repo / path) == expected, path
reader = 'crates/cambium/mere-overmap/src/materialization.rs'
for consumer in q['consumer_sources'].values():
    assert hashlib.sha256(subprocess.check_output(['git','show',consumer['supplier']+':'+reader],cwd=repo)).hexdigest() == q['inputs_sha256'][reader]
assert q['native_two_consumer_status'] == 'Qualified within recorded consumer scopes'
assert sum(x['reviewed_captures'] for x in q['consumer_sources'].values()) == 18
print(f'Supplier evidence verified: {len(prior["files"])} prior files, shared reader identical across two intakes, all scoped gates green and 18 consumer capture reviews referenced')
