from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent
source=HERE/'run_tinct_original_suite_probe.py'
receipt={'runner_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'build_exits':[],'run_exit':None,'checks':0,'generated_output_removed':True,'failure':'Supplier preflight asserted one serde artifact; capture contains two feature variants. No compiler or tests started. A later attempted shell inline preparation had a quoting SyntaxError and created nothing; this saved preparation is the first completed v2 setup.'}
with (HERE/'tinct_original_suite_probe_preflight_receipt.json').open('x',encoding='utf-8') as f:f.write(json.dumps(receipt,indent=2)+'\n')
text=source.read_text().replace("NAME='tinct_original_suite_probe'","NAME='tinct_original_suite_v2_probe'").replace("r['target']['name']=='serde' and any","r['target']['name']=='serde' and 'rc' in r['features'] and any").replace('authority-success-tinct-original-unit.exe','authority-success-tinct-original-v2-unit.exe')
with (HERE/'run_tinct_original_suite_v2_probe.py').open('x',encoding='utf-8') as f:f.write(text)
