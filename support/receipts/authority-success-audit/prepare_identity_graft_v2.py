from pathlib import Path
import hashlib,json
p=Path(__file__).resolve().parent
old=p/'run_identity_graft_original_suite.py';new=p/'run_identity_graft_original_suite_v2.py'
assert not new.exists()
with (p/'identity_graft_original_suite_preflight_receipt.json').open('x',encoding='utf-8') as f:
    f.write(json.dumps({'runner':old.name,'runner_sha256':hashlib.sha256(old.read_bytes()).hexdigest(),'exit':1,'reason':'serde has two captured feature rows; selector refused ambiguity before any compilation or output creation','builds':0,'tests':0},indent=2)+'\n')
s=old.read_text().replace("NAME = 'identity_graft_original_suite'", "NAME = 'identity_graft_original_suite_v2'")
s=s.replace("    assert len(matches) == 1, (name, len(matches))", "    if name == 'serde':\n        matches = [r for r in matches if 'rc' in r['features']]\n    assert len(matches) == 1, (name, len(matches))")
new.write_text(s,encoding='utf-8')
