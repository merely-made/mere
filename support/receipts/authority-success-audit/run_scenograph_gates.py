"""Bounded package gates for the Scenograph audit repairs."""
from pathlib import Path
import hashlib
import json
import re
import subprocess
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
mode = sys.argv[1]
assert mode in ('red', 'green')
common = ['--locked', '--offline', '-j', '1', '--target-dir', 'C:/t/cargo-targets/mere',
          '--config', 'build.build-dir="C:/t/cargo-targets/mere/build"']
if mode == 'red':
    gates = [
        ('layout', ['test', '-p', 'scenograph', '--lib', 'compatibility_summaries_do_not_change_the_default_layout']),
        ('scripts', ['test', '-p', 'script-rhai', '--test', 'scenograph_rules', 'composed_script_failures_leave_the_whole_rule_unmatched']),
    ]
else:
    gates = [
        ('default', ['test', '-p', 'scenograph', '--lib', '--tests']),
        ('scripts', ['test', '-p', 'scenograph', '--lib', '--tests', '--features', 'scripts']),
        ('rhai', ['test', '-p', 'script-rhai', '--lib', '--tests']),
        ('compiler', ['test', '-p', 'scenomise', '--lib', '--tests', '--features', 'authoring']),
    ]
sources = ['crates/cambium/scenes/scenograph/src/link.rs',
           'crates/cambium/scenes/scenograph/src/presentation.rs',
           'crates/cambium/scenes/scenograph/src/presentation/scripts.rs',
           'crates/cambium/scenes/scenomise/src/opened_link.rs',
           'crates/script/rhai/tests/scenograph_rules.rs']
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
receipt = {'mode': mode, 'basis_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
           'source_sha256': {p: sha(ROOT/p) for p in sources}, 'gates': []}
for name, args in gates:
    log = HERE/f'scenograph_{mode}_{name}.log'
    command = ['cargo', *args, *common]
    print(f'Running {mode}: {name}', flush=True)
    with log.open('w', encoding='utf-8') as out:
        ran = subprocess.run(command, cwd=ROOT, stdout=out, stderr=subprocess.STDOUT, timeout=1800)
    content = log.read_text(encoding='utf-8', errors='replace')
    counts = re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed;', content)
    result = {'name': name, 'command': command, 'exit': ran.returncode,
              'test_results': counts, 'log': log.name, 'sha256': sha(log)}
    receipt['gates'].append(result)
    (HERE/f'scenograph_{mode}_gates.json').write_text(json.dumps(receipt, indent=2)+'\n', encoding='utf-8')
    print(json.dumps(result), flush=True)
    if mode == 'green' and ran.returncode:
        sys.exit(ran.returncode)
