"""Qualify the immutable v2 runner's nocapture-sensitive test counter."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re
HERE=Path(__file__).resolve().parent
NAME='native_authority_v2_test_count_receipt.json'
assert not (HERE/NAME).exists()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
log=HERE/'native_authority_v2_probe_output.log'
output=log.read_text()
raw=json.loads((HERE/'native_authority_v2_probe_receipt.json').read_text())
match=re.search(r'test result: ok\. (\d+) passed; (\d+) failed;',output)
assert match and tuple(map(int,match.groups()))==(4,0)
tests=[line for line in output.splitlines() if line.startswith('test ') and not line.startswith('test result:')]
assert len(tests)==4 and raw['checks']==3 and raw['run_exit']==0
assert 'actual bare control rect: (4.0, 4.0, 64.0, 48.0)' in output
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'original_receipt':'native_authority_v2_probe_receipt.json','original_receipt_sha256':sha(HERE/'native_authority_v2_probe_receipt.json'),'output_sha256':sha(log),'runner_reported_checks':3,'tests_passed':4,'tests_failed':0,'run_exit':0,'test_start_lines':tests,'qualification':'The --nocapture geometry print interrupts one test line before its final ok, so the runner suffix-count reports three. The unchanged original test runner summary and four named test start lines establish four passed, zero failed. No rerun, overwritten evidence, or extra execution is claimed.','recorder_sha256':sha(Path(__file__))}
with (HERE/NAME).open('x',encoding='utf-8') as f:f.write(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(receipt,indent=2))
