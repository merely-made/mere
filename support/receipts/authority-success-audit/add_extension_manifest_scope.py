"""One-shot explicit extension of the source denominator; adds no read credit."""
from pathlib import Path
import hashlib,json
import read_coverage as reader
from refresh_inventory import EXTRA_PATHS
dest=reader.HERE/'extension_manifest_scope_receipt.json';assert not dest.exists()
path=reader.HERE/'source_inventory.json';prior=hashlib.sha256(path.read_bytes()).hexdigest()
inventory=reader.load('source_inventory.json',{});assert len(inventory['files'])==3725
assert not (set(inventory['files']) & EXTRA_PATHS)
for n in sorted(EXTRA_PATHS):
 digest,text=reader.contents(n)
 inventory['files'][n]={'sha256':digest,'chars':len(text),'lines':len(text.splitlines())}
inventory['scope']+=' Explicitly includes the Chromium and Firefox extension manifests as executable browser entry-point/permission configuration. Other JSON evidence/data remains outside this denominator. Use refresh_inventory.py to preserve this classification on future refreshes.'
reader.save('source_inventory.json',inventory)
v={'prior_inventory_sha256':prior,'current_inventory_sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'prior_files':3725,'current_files':len(inventory['files']),'added_source_paths':{n:inventory['files'][n] for n in sorted(EXTRA_PATHS)},'read_credit_added':0,'reason':'Production browser extension manifests select executable entry points, extension identity, permissions and content policy. They are source configuration; they were incorrectly excluded by the general JSON-data filter. The original reader stays unchanged and all prior exact-basis receipts remain historical. Read/ack the new source through the ordinary reader.'}
dest.write_text(json.dumps(v,indent=2)+'\n',encoding='utf-8');print(json.dumps(v,indent=2))
