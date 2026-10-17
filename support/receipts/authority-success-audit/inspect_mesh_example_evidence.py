"""Qualify current example check failures and compare literal provenance to Cargo."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,re,subprocess,tomllib
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
paths=['crates/mesh/mesh/examples/inventory_ingress/main.rs','crates/mesh/mesh/examples/retained_receive/identity.rs']
missing=[]
for name in paths:
    source=ROOT/name
    for match in re.finditer(r'include_(?:str|bytes)!\("([^"\n]+)"\)',source.read_text(encoding='utf-8')):
        relative=match.group(1)
        if '/murm/transport/' not in relative:continue
        resolved=(source.parent/relative).resolve()
        assert not resolved.exists()
        missing.append({'source':name,'line':source.read_text(encoding='utf-8')[:match.start()].count('\n')+1,
                        'include':relative,'resolved':str(resolved),'exists':False})
assert len(missing)==6
for name,count in [('mesh_inventory_example_check.log',2),('mesh_retained_example_check.log',4)]:
    log=(HERE/name).read_text(encoding='utf-8-sig')
    assert log.count("error: couldn't read")==count,(name,log[-3000:])
    assert 'could not compile `mere-mesh`' in log
positive=(HERE/'mesh_example_positive_check.log').read_text(encoding='utf-8-sig')
assert 'Finished `dev` profile' in positive and '\nerror' not in positive
carrier=ROOT/'crates/mesh/mesh/examples/retained_receive/carrier.rs'
literal=re.search(r'"p2panda_net_revision":"([0-9a-f]{40})"',carrier.read_text(encoding='utf-8')).group(1)
lock=ROOT/'Cargo.lock';packages=tomllib.loads(lock.read_text(encoding='utf-8'))['package']
supplier=next(p for p in packages if p['name']=='mere-p2panda-net')
revision=supplier['source'].rsplit('#',1)[1]
assert revision!=literal
metadata=subprocess.run(['cargo','metadata','--locked','--offline','--format-version','1'],cwd=ROOT,capture_output=True,text=True,encoding='utf-8')
(HERE/'mesh_examples_metadata_stderr.log').write_text(metadata.stderr,encoding='utf-8');assert metadata.returncode==0,metadata.stderr
graph=json.loads(metadata.stdout)
selected=[p for p in graph['packages'] if p['name']=='mere-p2panda-net'];assert len(selected)==1
assert selected[0]['source']==supplier['source']
transports=[p for p in graph['packages'] if p['name']=='mere-transport'];assert len(transports)==1
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'missing_includes':missing,
 'inventory_example_check':{'shell_exit':1,'rust_errors':2,'kind':'missing compile-time source includes'},
 'retained_example_check':{'shell_exit':1,'rust_errors':4,'kind':'missing compile-time source includes'},
 'positive_check':{'shell_exit':0,'targets':['mere-mesh admitted-ingress library','mesh-peer example']},
 'hardcoded_p2panda_revision':literal,'actual_p2panda_revision':revision,
 'current_supplier':{'id':selected[0]['id'],'source':selected[0]['source'],'manifest_path':selected[0]['manifest_path']},
 'current_transport':{'id':transports[0]['id'],'source':transports[0]['source'],'manifest_path':transports[0]['manifest_path']},
 'provenance_line':carrier.read_text(encoding='utf-8')[:carrier.read_text(encoding='utf-8').index('"p2panda_net_revision"')].count('\n')+1,
 'trace_sha256':{str(p):sha(p) for p in [lock,carrier,ROOT/'Cargo.toml',Path(selected[0]['manifest_path']),Path(transports[0]['manifest_path'])]},
 'scope':'Two actual fresh Cargo check failures and one positive check. Literal revision disagreement is source/locked resolved Cargo metadata evidence; failed examples were not executed and no newly emitted false report is claimed. No Retinue subprocess, network endpoint, durable store, worker execution or physical carrier ran.'}
(HERE/'mesh_examples_evidence_receipt.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'missing_include_count':len(missing),'inventory_errors':2,'retained_errors':4,'positive_check':0,'literal_revision':literal,'actual_revision':revision}))
