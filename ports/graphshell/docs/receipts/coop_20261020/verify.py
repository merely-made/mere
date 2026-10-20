#!/usr/bin/env python3
"""Verify the public co-op packet, exact scope and optional committed sources."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import tomllib

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]


def read(path):
    return json.loads(path.read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify_sources(files, commit=None):
    if not commit:
        for name, expected in files.items():
            assert sha(ROOT / name) == expected, name
        return
    process = subprocess.Popen(['git', 'cat-file', '--batch'], cwd=ROOT,
                               stdin=subprocess.PIPE, stdout=subprocess.PIPE)
    try:
        for name, expected in files.items():
            process.stdin.write((commit + ':' + name + '\n').encode())
            process.stdin.flush()
            header = process.stdout.readline().decode().strip().split()
            assert len(header) == 3 and header[1] == 'blob', name
            blob = process.stdout.read(int(header[2]))
            assert process.stdout.read(1) == b'\n'
            assert hashlib.sha256(blob).hexdigest() == expected, name
    finally:
        process.stdin.close()
        process.wait(timeout=10)
    assert process.returncode == 0


def normalize(value):
    if isinstance(value, dict):
        return {key: normalize(item) for key, item in value.items()}
    if isinstance(value, list):
        return sorted((normalize(item) for item in value), key=repr)
    return value


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--packet', type=Path, default=HERE)
    parser.add_argument('--commit')
    args = parser.parse_args()
    packet = args.packet.resolve()
    hashes = read(packet / 'artifact-sha256.json')
    for name, expected in hashes.items():
        path = (packet / name).resolve()
        assert path.is_relative_to(packet), name
        assert sha(path) == expected, name
    result = read(packet / 'accepted/qualification.json')
    assert result['result'] == 'ok'
    phases = {phase['name']: phase for phase in result['phases']}
    assert set(phases) == {'private-baseline', 'owner-live', 'guest-live', 'guest-return',
                           'owner-cold', 'guest-cold', 'private-return'}
    pids = [p['pid'] for p in phases.values()]
    assert len(set(pids)) == len(pids), 'cold return reused a native process'
    owner, guest = result['owner_root'], result['guest_root']
    assert owner != guest
    chosen, private = result['chosen_graph'], result['private_graph']
    assert chosen != private
    captures = sorted((packet / 'accepted').glob('*.png'))
    assert len(captures) == 18, len(captures)
    for capture in captures:
        assert capture.read_bytes()[:8] == b'\x89PNG\r\n\x1a\n'
        width, height = struct.unpack('>II', capture.read_bytes()[16:24])
        assert width > 200 and height > 200
        record = read(capture.with_suffix('.json'))
        assert record['colors'] > 8 and record['presentation']
        assert record['pid'] in pids
        data = record['state']['data']
        if data is not None:
            if capture.stem.startswith('private-'):
                assert data['graph'] == private
                assert [n['address'] for n in data['nodes']] == ['https://private.invalid/owner-only']
            else:
                assert data['graph'] == chosen
                assert all(n['address'].startswith('https://shared.invalid/') for n in data['nodes'])
    def state(name):
        return read(packet / 'accepted' / (name + '.json'))['state']
    assert len(state('guest-joined')['data']['nodes']) == 1
    assert len(state('guest-added')['data']['nodes']) == 2
    assert len(state('guest-saw-push')['data']['nodes']) == 3
    assert state('guest-reopened')['root'] == guest
    assert state('guest-reopened')['data']['nodes'] == state('guest-saw-push')['data']['nodes']
    revised = state('guest-renamed')['data']
    assert any(n['title'] == 'Guest source revised' for n in revised['nodes'])
    assert revised['authors'][-1]['id'] == 'personae:' + guest
    assert revised['authors'][-1]['via'] == 'graphshell.coop'
    assert state('owner-ended-access')['data']['access'] == 'revoked'
    assert state('guest-access-ended')['data'] is None
    assert state('guest-cold-denied')['data'] is None
    assert state('guest-cold-denied')['root'] == guest
    cold = state('owner-cold-retained')['data']
    assert cold['owner'] == owner and cold['guest'] == guest and cold['access'] == 'revoked'
    assert cold['nodes'] == revised['nodes']
    assert cold['authors'] == revised['authors']
    assert cold['epoch'] > revised['epoch']
    assert len(state('owner-cold-new-edit')['data']['nodes']) == 4
    assert state('owner-cold-new-edit')['data']['authors'][-1]['id'] == 'personae:' + owner
    before, after = state('private-baseline')['data'], state('private-unchanged')['data']
    for key in ['graph', 'nodes', 'authors', 'revision']:
        assert before[key] == after[key], key
    assert all(result['cold_reopen'].values())
    inputs = read(packet / 'build-inputs.json')
    assert inputs['binaries']['graphshell-coop'] == result['binary_sha256']
    assert inputs['binaries']['djinn'] == result['djinn_sha256']
    for name, expected in inputs['harness'].items():
        assert sha(packet / name) == expected, name
    publication_path = packet / 'publication-inputs.json'
    if publication_path.exists():
        publication = read(publication_path)
        assert publication['binary_identical']
        assert publication['frozen_binaries'] == inputs['binaries']
        assert publication['rebuilt_binaries'] == inputs['binaries']
        verify_sources(inputs['files'], publication['tested_source_commit'])
        verify_sources(publication['files'], args.commit)
        assert set(publication['files']) == set(inputs['files'])
        changes = {name for name in inputs['files']
                   if inputs['files'][name] != publication['files'][name]}
        example = 'ports/tabard/desktop/examples/native_shared_recovery.rs'
        assert changes == set(publication['changes']) == {'Cargo.lock', example}
        original_lock = subprocess.check_output(
            ['git', 'show', publication['tested_source_commit'] + ':Cargo.lock'], cwd=ROOT)
        final_lock = (subprocess.check_output(['git', 'show', args.commit + ':Cargo.lock'], cwd=ROOT)
                      if args.commit else (ROOT / 'Cargo.lock').read_bytes())
        assert normalize(tomllib.loads(original_lock.decode())) == normalize(tomllib.loads(final_lock.decode()))
        for name in inputs['binaries']:
            assert 'native_shared_recovery.rs' not in (packet / (name + '-compiler-inputs.d')).read_text()
    else:
        verify_sources(inputs['files'], args.commit)
    review = read(packet / 'capture-review.json')
    assert {r['capture'] for r in review} == {p.name for p in captures}
    assert all(r['reviewed'] and r['observation'] for r in review)
    assert all(sha(packet / 'accepted' / r['capture']) == r['sha256'] for r in review)
    print(json.dumps({'result': 'ok', 'source_files': len(inputs['files']),
                      'public_artifacts': len(hashes), 'headed_captures': len(captures),
                      'native_processes': len(pids), 'commit': args.commit}, indent=2))


if __name__ == '__main__':
    main()
