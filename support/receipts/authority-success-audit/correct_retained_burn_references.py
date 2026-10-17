"""Correct seven shorthand module paths without rewriting the cited checkpoint."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
load=lambda n:json.loads((HERE/n).read_text(encoding='utf-8'))
old=load('retained_burn_review_receipt.json');doc=ROOT/old['document']['path'];prefix=doc.read_bytes();assert sha(doc)==old['document']['sha256']
base='support/patches/burn-cubecl/src/kernel/'
fix={'attention.rs':'attention/','conv/forward.rs':'conv/forward/','conv/data_backward.rs':'conv/backward_data/','conv/weight_backward.rs':'conv/backward_weight/','grid_sample.rs':'grid_sample/','fft.rs':'fft/','cast.rs':'cast/'}
fix={base+k:base+v for k,v in fix.items()};a=load('authority_map.json');count=0
for row in a['entries'][-16:]:
 for n,p in enumerate(row['implementations']):
  if p in fix:row['implementations'][n]=fix[p];count+=1
  assert (ROOT/row['implementations'][n]).exists()
assert count==7
(HERE/'authority_map.json').write_text(json.dumps(a,indent=2)+'\n',encoding='utf-8')
doc.write_bytes(prefix+b'\n\nRetained Burn checkpoint reference correction: seven authority-map references used file-shaped shorthand for directory modules. A1285/A1287/A1289/A1290 now point to the actual attention, forward, backward_data, backward_weight, grid_sample, fft and cast directories. Reading hashes, coverage and findings are unchanged. The original checkpoint and its cited evidence remain byte-preserved.\n')
record=dict(old);record.update({'date_utc':datetime.now(timezone.utc).isoformat(),'prior_receipt':{'path':'retained_burn_review_receipt.json','sha256':sha(HERE/'retained_burn_review_receipt.json')},'reference_corrections':fix,'new_findings':[],'authority_entries':[],'evidence_sha256':{**old['evidence_sha256'],'correct_retained_burn_references.py':sha(Path(__file__))},'document':{'path':old['document']['path'],'sha256':sha(doc),'preserved_prefix_bytes':len(prefix),'preserved_prefix_sha256':hashlib.sha256(prefix).hexdigest()}})
with (HERE/'retained_burn_references_receipt.json').open('x',encoding='utf-8') as f:json.dump(record,f,indent=2);f.write('\n')
assert doc.read_bytes().startswith(prefix)
print(json.dumps({'corrected_module_paths':count,'coverage':old['coverage']}))
