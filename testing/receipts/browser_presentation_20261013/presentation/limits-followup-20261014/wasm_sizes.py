#!/usr/bin/env python3
"""Attribute wasm code-section bytes to demangled function names.

Usage: wasm_sizes.py FILE.wasm [--top N] [--json OUT]
Reads the code section (per-function body size) and the `name` custom
section (function names), demangles Rust v0/legacy names with rustfilt if
available, and prints totals by crate, by generic family (name with
generic arguments removed) and the largest functions.
"""
import json, re, shutil, subprocess, sys
from collections import Counter, defaultdict


def leb(data, i):
    result = shift = 0
    while True:
        b = data[i]; i += 1
        result |= (b & 0x7F) << shift
        if b < 0x80:
            return result, i
        shift += 7


def parse(path):
    data = open(path, 'rb').read()
    assert data[:4] == b'\0asm'
    i = 8
    imported_funcs = 0
    bodies = []
    names = {}
    while i < len(data):
        sid = data[i]; i += 1
        size, i = leb(data, i)
        end = i + size
        if sid == 2:  # imports
            n, j = leb(data, i)
            for _ in range(n):
                l, j = leb(data, j); j += l
                l, j = leb(data, j); j += l
                kind = data[j]; j += 1
                if kind == 0:
                    _, j = leb(data, j); imported_funcs += 1
                elif kind == 1:
                    j += 1; flags, j = leb(data, j); _, j = leb(data, j)
                    if flags & 1: _, j = leb(data, j)
                elif kind == 2:
                    flags, j = leb(data, j); _, j = leb(data, j)
                    if flags & 1: _, j = leb(data, j)
                elif kind == 3:
                    j += 2
                elif kind == 4:
                    j += 1; _, j = leb(data, j)
        elif sid == 10:  # code
            n, j = leb(data, i)
            for _ in range(n):
                s, j = leb(data, j)
                bodies.append(s); j += s
        elif sid == 0:
            l, j = leb(data, i)
            name = data[j:j + l].decode(); j += l
            if name == 'name':
                while j < end:
                    sub = data[j]; j += 1
                    ss, j = leb(data, j)
                    send = j + ss
                    if sub == 1:
                        n, k = leb(data, j)
                        for _ in range(n):
                            idx, k = leb(data, k)
                            l2, k = leb(data, k)
                            names[idx] = data[k:k + l2].decode('utf-8', 'replace'); k += l2
                    j = send
        i = end
    return imported_funcs, bodies, names


def demangle(names):
    tool = shutil.which('rustfilt')
    if not tool:
        return names
    keys = list(names)
    out = subprocess.run([tool], input='\n'.join(names[k] for k in keys), capture_output=True, text=True).stdout.split('\n')
    return {k: out[n] if n < len(out) else names[k] for n, k in enumerate(keys)}


def crate_of(name):
    name = re.sub(r'^<', '', name)
    m = re.match(r'([A-Za-z_][A-Za-z0-9_]*)::', name)
    return m.group(1) if m else '?'


def family(name):
    out = []; depth = 0
    for ch in name:
        if ch == '<':
            depth += 1
            if depth == 1: out.append('<…>')
        elif ch == '>':
            depth = max(0, depth - 1)
        elif depth == 0:
            out.append(ch)
    return re.sub(r'::h[0-9a-f]{16}$', '', ''.join(out))


def main():
    path = sys.argv[1]
    top = int(sys.argv[sys.argv.index('--top') + 1]) if '--top' in sys.argv else 40
    imported, bodies, names = parse(path)
    names = demangle(names)
    rows = []
    for n, size in enumerate(bodies):
        idx = imported + n
        rows.append((size, names.get(idx, f'func[{idx}]')))
    total = sum(s for s, _ in rows)
    crates = Counter(); fams = Counter(); famcount = Counter()
    for s, n in rows:
        crates[crate_of(n)] += s
        f = family(n); fams[f] += s; famcount[f] += 1
    report = {
        'functions': len(rows), 'code_bytes': total,
        'crates': crates.most_common(),
        'families': [(f, s, famcount[f]) for f, s in fams.most_common(400)],
        'largest': sorted(rows, reverse=True)[:400],
    }
    print(f'{len(rows)} functions, {total} code bytes')
    print('\n# crates')
    for c, s in crates.most_common(top):
        print(f'{s:>10} {s * 100 / total:5.1f}% {c}')
    print('\n# families (instances)')
    for f, s, c in report['families'][:top]:
        print(f'{s:>10} {c:>5} {f[:160]}')
    if '--json' in sys.argv:
        json.dump(report, open(sys.argv[sys.argv.index('--json') + 1], 'w'), indent=1)


if __name__ == '__main__':
    main()
