"""Source-reading receipts. Reading is acknowledged separately from inventorying.

Coverage records characters actually emitted for review, bound to content hashes.
Scanning, grep, compilation, and a previous partial read do not count as coverage.
"""
from pathlib import Path
import argparse
import hashlib
import json
import subprocess
import sys

sys.stdout.reconfigure(encoding='utf-8', newline='\n')

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
SOURCE = {'.rs', '.py', '.js', '.mjs', '.cjs', '.ts', '.tsx', '.jsx', '.wgsl',
          '.glsl', '.vert', '.frag', '.html', '.css', '.wit', '.rhai', '.c',
          '.h', '.cpp', '.hpp', '.toml', '.yaml', '.yml', '.sh', '.ps1',
          '.bat', '.cmd', '.scn', '.scene', '.patch', '.swift', '.mu', '.scroll', '.in'}
SOURCE_NAMES = {'package.json', 'manifest.json', 'deno.json', 'deno.jsonc',
                'tsconfig.json', 'Makefile', 'Dockerfile', 'justfile', '.gitattributes'}

def is_source(path):
    if path.suffix.lower() in SOURCE or path.name in SOURCE_NAMES:
        return True
    if path.suffix.lower() in {'.example', '.orig'}:
        return Path(path.stem).suffix.lower() in SOURCE
    return not path.suffix and path.read_bytes().startswith(b'#!')

def save(name, value):
    (HERE / name).write_text(json.dumps(value, indent=2) + '\n', encoding='utf-8')

def load(name, default):
    p = HERE / name
    return json.loads(p.read_text(encoding='utf-8')) if p.exists() else default

def contents(path):
    raw = (ROOT / path).read_bytes()
    return hashlib.sha256(raw).hexdigest(), raw.decode('utf-8-sig', errors='replace')

def inventory():
    result = {}
    paths = subprocess.check_output(
        ['git', 'ls-files', '--cached', '--others', '--exclude-standard', '-z'],
        cwd=ROOT).decode().split('\0')
    for path in paths:
        # The audit must cover non-ignored product WIP as well as committed
        # source. Its own tools and preserved reproductions stay outside the
        # product denominator, so new evidence cannot recursively grow scope.
        if path.startswith('support/receipts/authority-success-audit/'):
            continue
        p = ROOT / path
        if not p.is_file():
            continue
        if not is_source(p):
            continue
        digest, text = contents(path)
        result[path] = {'sha256': digest, 'chars': len(text), 'lines': len(text.splitlines())}
    save('source_inventory.json', {'commit': subprocess.check_output(
        ['git', 'rev-parse', 'HEAD'], cwd=ROOT).decode().strip(), 'files': result,
        'scope': 'Tracked and non-ignored untracked product source, manifests, workflow code, scenarios and patches, including source in existing receipts. This audit directory is evidence/tooling outside the product denominator. Binary assets, logs, lockfiles and JSON evidence/data are not source coverage.'})
    print(json.dumps({'files': len(result), 'lines': sum(v['lines'] for v in result.values()),
                      'chars': sum(v['chars'] for v in result.values())}))

def acknowledge():
    pending = load('pending_read.json', [])
    coverage = load('read_coverage.json', {})
    for item in pending:
        path, digest, end = item['path'], item['sha256'], item['end']
        actual, _ = contents(path)
        if actual != digest:
            raise RuntimeError(f'Changed during review: {path}')
        before = coverage.get(path, {})
        expected = before.get('through', 0) if before.get('sha256') == digest else 0
        if expected != item['start']:
            raise RuntimeError(f'Non-contiguous acknowledgment: {path}')
        coverage[path] = {'sha256': digest, 'through': end}
    save('read_coverage.json', coverage)
    save('pending_read.json', [])
    print(f'Acknowledged {len(pending)} source chunks')

def read(prefix, limit):
    if load('pending_read.json', []):
        raise RuntimeError('Acknowledge or discard the preceding read before continuing')
    files = load('source_inventory.json', {})['files']
    coverage = load('read_coverage.json', {})
    emitted, pending = 0, []
    for path in sorted(files):
        if prefix and not path.startswith(prefix):
            continue
        digest, text = contents(path)
        held = coverage.get(path, {})
        start = held.get('through', 0) if held.get('sha256') == digest else 0
        if start >= len(text):
            continue
        if emitted >= limit:
            break
        end = min(len(text), start + limit - emitted)
        if end < len(text):
            boundary = text.rfind('\n', start, end)
            if boundary > start:
                end = boundary + 1
        line = text.count('\n', 0, start) + 1
        print(f'\n=== {path} | sha256 {digest} | chars {start}:{end}/{len(text)} | starts line {line} ===')
        print(text[start:end].replace('\r\n', '\n'), end='\n')
        pending.append({'path': path, 'sha256': digest, 'start': start, 'end': end})
        emitted += end - start
        if end < len(text):
            break
    save('pending_read.json', pending)
    print(f'\nEMITTED {emitted} characters in {len(pending)} chunks; acknowledgment pending')

def status():
    files = load('source_inventory.json', {})['files']
    coverage = load('read_coverage.json', {})
    complete, chars, changed = 0, 0, []
    for path, info in files.items():
        held = coverage.get(path, {})
        if held.get('sha256') == info['sha256']:
            through = min(info['chars'], held['through'])
            chars += through
            complete += through == info['chars']
        elif path in coverage:
            changed.append(path)
    print(json.dumps({'complete_files': complete, 'total_files': len(files),
                      'read_chars': chars, 'total_chars': sum(v['chars'] for v in files.values()),
                      'changed_since_inventory': changed, 'pending_chunks': len(load('pending_read.json', []))}))

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('action', choices=['inventory', 'read', 'ack', 'status'])
    parser.add_argument('--prefix', default='')
    parser.add_argument('--chars', type=int, default=18000)
    args = parser.parse_args()
    if args.action == 'inventory': inventory()
    elif args.action == 'ack': acknowledge()
    elif args.action == 'read': read(args.prefix, args.chars)
    else: status()
