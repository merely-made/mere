#!/usr/bin/env python3
"""Use the qualified native freeze, adding exact shared-controller provenance."""
import importlib.util
import json
from pathlib import Path
import sys


def value(flag):
    return Path(sys.argv[sys.argv.index(flag) + 1]).resolve()


if __name__ == '__main__':
    root, out, harness = value('--repo'), value('--output'), value('--harness')
    shared = root / 'ports/graphshell/docs/receipts/coop_mac_thinkpad_20261020'
    spec = importlib.util.spec_from_file_location('native_freeze', shared / 'freeze.py')
    base = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(base)
    base.main()
    path = out / 'build-inputs.json'
    result = json.loads(path.read_text())
    prefix = 'ports/graphshell/docs/receipts/coop_relay_20261020/'
    result['harness'] = {prefix + n: base.sha(harness / n) for n in ['machine.py', 'run.py', 'freeze.py']}
    for name in ['machine.py', 'run.py', 'freeze.py']:
        p = shared / name
        result['harness'][str(p.relative_to(root))] = base.sha(p)
    for name in ['relay/Cargo.toml', 'relay/Cargo.lock', 'relay/src/main.rs']:
        result['harness'][prefix + name] = base.sha(harness / name)
    p = root / 'ports/graphshell/docs/receipts/coop_20261020/resident.py'
    result['harness'][str(p.relative_to(root))] = base.sha(p)
    path.write_text(json.dumps(result, indent=2) + '\n')
