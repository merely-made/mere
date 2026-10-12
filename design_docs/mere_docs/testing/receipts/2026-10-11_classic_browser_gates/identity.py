#!/usr/bin/env python3
# Copyright 2026 Mark Alan Boykin
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# SPDX-License-Identifier: MPL-2.0
"""Record source before a serialized build; seal only unchanged built sources."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(chunk)
    return h.hexdigest()


def snapshot(root):
    root = root.resolve()
    names = subprocess.check_output(['git', '-C', str(root), 'ls-files', '--cached', '--others', '--exclude-standard', '-z']).decode().split('\0')
    paths = [root / name for name in names if name and (name.endswith('.rs') or name.endswith('Cargo.toml') or name.endswith('Cargo.lock') or name.endswith('.scn') or name.startswith('ports/graphshell/web/') and name.endswith(('.js', '.html', '.css')))]
    paths += [root / 'ports/graphshell/web/scenarios/renderer_acceptance.scn']
    hashes = {str(path.relative_to(root)): digest(path) for path in sorted(set(paths)) if path.is_file()}
    return {'root': str(root), 'head': subprocess.check_output(['git', '-C', str(root), 'rev-parse', 'HEAD']).decode().strip(),
            'source_sha256': hashes, 'fingerprint': hashlib.sha256(json.dumps(hashes, sort_keys=True).encode()).hexdigest()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['preflight', 'seal'])
    parser.add_argument('--repo', type=Path, required=True)
    parser.add_argument('--extra-source', type=Path, action='append', default=[])
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--preflight', type=Path)
    parser.add_argument('--package', type=Path)
    parser.add_argument('--metadata', type=Path)
    parser.add_argument('--build-log', type=Path)
    args = parser.parse_args()
    sources = [snapshot(root) for root in [args.repo, *args.extra_source]]
    result = {'schema': 'graphshell-renderer-browser-identity-v1', 'sealed': False,
              'features': ['product'], 'target': 'wasm32-unknown-unknown', 'sources': sources}
    if args.action == 'seal':
        if not all([args.preflight, args.package, args.metadata, args.build_log]):
            parser.error('seal requires --preflight, --package, --metadata and --build-log')
        before = json.loads(args.preflight.read_text())
        if [(s['root'], s['fingerprint']) for s in before['sources']] != [(s['root'], s['fingerprint']) for s in sources]:
            raise SystemExit('REFUSED: source changed between preflight and package seal')
        metadata = json.loads(args.metadata.read_text())
        known_roots = [Path(item['root']) for item in sources]
        for package in metadata['packages']:
            if package['source'] is None and not any(Path(package['manifest_path']).resolve().is_relative_to(root) for root in known_roots):
                raise SystemExit('REFUSED: unhashed local package ' + package['manifest_path'])
        log = args.build_log.read_text(errors='replace')
        if 'Finished' not in log or 'error: could not compile' in log:
            raise SystemExit('REFUSED: completed build evidence unavailable')
        artifacts = {}
        for name in ['graphshell_web.js', 'graphshell_web_bg.wasm']:
            path = args.package / name
            artifacts[name] = {'bytes': path.stat().st_size, 'sha256': digest(path)}
        result.update(sealed=True, build_sources=before['sources'], package=str(args.package.resolve()), artifacts=artifacts,
                      metadata_sha256=digest(args.metadata), build_log_sha256=digest(args.build_log),
                      dependencies=[{'name': package['name'], 'version': package['version'], 'source': package['source']}
                                    for package in metadata['packages'] if package['name'].startswith(('netrender', 'vello', 'genet'))])
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + '\n')
    print(args.output)


if __name__ == '__main__':
    main()
