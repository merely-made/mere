"""Capture the small current Nisus library gate using the shared stable target."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
inventory = json.loads((HERE/'source_inventory.json').read_text())['files']
sources = {p:v['sha256'] for p,v in inventory.items() if p.startswith('crates/conatus/nisus/')}
assert len(sources) == 2 and all(sha(ROOT/p)==h for p,h in sources.items())
command = ['cargo','--config','build.build-dir="C:/t/cargo-targets/mere/build"',
    'test','-p','nisus','--lib','--target-dir','C:/t/cargo-targets/mere','--','--test-threads=1']
with (HERE/'nisus_library_suite.log').open('w',encoding='utf-8') as out:
    ran = subprocess.run(command,cwd=ROOT,stdout=out,stderr=subprocess.STDOUT,timeout=180)
assert all(sha(ROOT/p)==h for p,h in sources.items())
receipt = {'date_utc':datetime.now(timezone.utc).isoformat(),'command':command,'exit':ran.returncode,
    'source_sha256':sources,'log_sha256':sha(HERE/'nisus_library_suite.log'),
    'scope':'Current Nisus library tests only. No product, collider, GPU or durable storage consumer execution.',
    'retained_isolation':[]}
(HERE/'nisus_library_suite_receipt.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
print((HERE/'nisus_library_suite.log').read_text(encoding='utf-8')[-2200:])
assert ran.returncode == 0
