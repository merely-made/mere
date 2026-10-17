"""Select a coherent retained supplier set from Cargo fingerprint identities."""
from pathlib import Path
import hashlib,json
HERE=Path(__file__).resolve().parent
DEBUG=Path('C:/t/cargo-targets/mere/build/debug')
FP=DEBUG/'.fingerprint'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
output=HERE/'commons_supplier_selection.json'
assert not output.exists()
prefixes=('mere-proofs-','personae-','identity-','insigne-','p2panda-core-','p2panda-encryption-','p2panda-store-','p2panda-net-','mere-p2panda-net-','muniment-','serde-','thiserror-','servitor-','stickleback-','chartulary-','gemot-')
index=[]
for folder in FP.iterdir():
    if not folder.is_dir() or not folder.name.startswith(prefixes):continue
    suffix=folder.name.rsplit('-',1)[1]
    for p in folder.glob('lib-*.json'):
        crate=p.name[4:-5];marker=folder/('lib-'+crate)
        library=DEBUG/'deps'/('lib'+crate+'-'+suffix+'.rlib')
        if not marker.exists() or not library.exists():continue
        value=marker.read_text().strip()
        if len(value)!=16:continue
        index.append({'crate':crate,'fingerprint_value':int.from_bytes(bytes.fromhex(value),'little'),'path':str(library),'sha256':sha(library),'fingerprint_path':str(p),'fingerprint_sha256':sha(p),'metadata':json.loads(p.read_text())})
def choose(name,value):
    choices=[r for r in index if r['fingerprint_value']==value and r['crate'] in ({'identity','personae'} if name=='identity' else {name})]
    assert len(choices)==1,(name,value,len(choices))
    return choices[0]
gemot=[r for r in index if Path(r['path']).name=='libgemot-d1280ffacb26c7cf.rlib']
assert len(gemot)==1
gemot=gemot[0];deps={r[1]:r[3] for r in gemot['metadata']['deps']}
libs={'gemot':gemot}
for name in ['proofs','insigne','muniment','servitor','identity','serde','thiserror','stickleback','p2panda_core','p2panda_encryption','p2panda_store']:
    libs['personae' if name=='identity' else name]=choose(name,deps[name])
stickle_deps={r[1]:r[3] for r in libs['stickleback']['metadata']['deps']}
libs['p2panda_net']=choose('p2panda_net',stickle_deps['p2panda_net'])
charts=[]
for row in index:
    if row['crate']!='chartulary':continue
    chart_deps={r[1]:r[3] for r in row['metadata']['deps']}
    if all(chart_deps.get(n)==deps[n] for n in ['serde','muniment']):charts.append(row)
assert charts
libs['chartulary']=sorted(charts,key=lambda r:r['path'])[0]
for row in libs.values():
    assert row['metadata']['rustc']==gemot['metadata']['rustc']
    assert row['metadata']['compile_kind']==0
payload={'libraries':libs,'compatible_chartulary_candidates':[r['path'] for r in charts],'qualification':'Cargo fingerprint dependency values select a retained coherent family. The lexicographically first chartulary artifact sharing exact serde/muniment fingerprints is used. These are retained binaries, not fresh supplier source builds or a current Cargo resolution.','selector_sha256':sha(Path(__file__))}
with output.open('x',encoding='utf-8') as f:json.dump(payload,f,indent=2);f.write('\n')
print(json.dumps({'selection':output.name,'libraries':len(libs),'compatible_chartulary_candidates':len(charts)}))
