"""Run exact current facet presentation functions over real retained card/store types."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve()
NAME='route_facet_probe';out=HERE/(NAME+'_receipt.json');assert not out.exists()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
original=ROOT/'crates/cambium/mere-overmap/src/route.rs';raw=original.read_bytes();text=raw.decode('utf-8')
start=text.index('pub fn add_node_facets(');end=text.index('\n#[cfg(test)]',start);functions=text[start:end]
body=r'''
fn card() -> PortableCardV1 { PortableCardV1 { title: "Selected node".into(),
 values: vec![chirograph::CardValueV1 {label:"Address".into(),value:"https://owner.test/selected".into()}],badges:vec![],media:vec![] } }
fn main() {
 let id=Uuid::from_u128(1);let key=id.to_string();
 let first=|cards:&HashMap<String,PortableCardV1>| cards[&key].values.iter().find(|v|v.label=="Address").unwrap().value.clone();
 let mut cards=HashMap::from([(key.clone(),card())]);
 add_node_facets(&mut cards,&kernel::graph::NodeFacetStore::new()).unwrap();
 assert_eq!(first(&cards),"https://owner.test/selected");
 println!("PASS no-facet control keeps owner address first");
 let mut facets=kernel::graph::NodeFacetStore::new();
 facets.set(id,chartulary::FacetId::new("product.record"),serde_json::json!({"Address":"https://other.test/facet"}),&chartulary::AcceptAll).unwrap();
 let mut cards=HashMap::from([(key.clone(),card())]);add_node_facets(&mut cards,&facets).unwrap();
 assert_eq!(first(&cards),"https://other.test/facet");
 assert!(cards[&key].values.iter().any(|v|v.label=="Address"&&v.value=="https://owner.test/selected"));
 println!("PASS facet Address shadows the still-present owner address for first-label consumers");
 let mut facets=kernel::graph::NodeFacetStore::new();
 facets.set(id,chartulary::FacetId::new("product.record"),serde_json::json!({"address":"https://other.test/facet"}),&chartulary::AcceptAll).unwrap();
 let mut cards=HashMap::from([(key.clone(),card())]);add_node_facets(&mut cards,&facets).unwrap();
 assert_eq!(first(&cards),"https://owner.test/selected");
 println!("PASS lowercase address control does not collide with the exact Address label");
}
'''
probe=HERE/(NAME+'.rs')
with probe.open('x',encoding='utf-8',newline='') as f:f.write('use std::collections::HashMap;\nuse uuid::Uuid;\nuse chirograph::PortableCardV1;\n'+functions+body)
rows=[json.loads(l) for l in (HERE/'capture_graphshell_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')]
assert any(r.get('reason')=='build-finished' and r.get('success') for r in rows)
libs={}
for name in ['kernel','chartulary','uuid','chirograph','serde_json']:
 matches=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']==name and any(f.endswith('.rlib') for f in r['filenames'])]
 assert len(matches)==1,(name,len(matches))
 r=matches[0];path=Path(next(f for f in r['filenames'] if f.endswith('.rlib')))
 libs[name]={'path':str(path),'sha256':sha(path),'package_id':r['package_id'],'features':r['features']}
source_names=['crates/cambium/mere-overmap/src/route.rs','crates/cambium/mere-overmap/src/route_session.rs','ports/graphshell/src/bin/graphshell_mere.rs']
sources={n:sha(ROOT/n) for n in source_names}
binary=TARGET/'authority-success-route-facet-probe.exe';assert not list(TARGET.glob(binary.stem+'.*'))
command=['rustc','--edition=2024',str(probe),'-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0','-o',str(binary)]
for name,lib in libs.items():command+=['--extern',name+'='+lib['path']]
native=sorted({p for r in rows if r.get('reason')=='build-script-executed' for p in r.get('linked_paths',[])})
for p in native:command+=['-L',p]
built=subprocess.run(command,cwd=ROOT,capture_output=True,text=True,timeout=60)
with (HERE/(NAME+'_compile.log')).open('x',encoding='utf-8') as f:f.write(built.stdout+built.stderr)
ran=None;removed=[]
try:
 if built.returncode==0:
  ran=subprocess.run([str(binary)],cwd=ROOT,capture_output=True,text=True,timeout=20)
  with (HERE/(NAME+'_output.log')).open('x',encoding='utf-8') as f:f.write(ran.stdout+ran.stderr)
  print(ran.stdout)
finally:
 for p in sorted(TARGET.glob(binary.stem+'.*')):
  assert p.resolve().parent==TARGET;data=p.read_bytes()
  assert (p.suffix=='.exe' and data.startswith(b'MZ')) or (p.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF')),str(p)
  removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
assert all(sha(ROOT/n)==d for n,d in sources.items()) and original.read_bytes()==raw
assert all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'source_sha256':sources,'libraries':libs,
 'verbatim_functions':{'start':start,'end':end,'sha256':hashlib.sha256(functions.encode()).hexdigest()},
 'probe_sha256':sha(probe),'runner_sha256':sha(Path(__file__)),'command':command,'native_search_paths':native,
 'build_exit':built.returncode,'run_exit':None if ran is None else ran.returncode,'checks':0 if ran is None else ran.stdout.count('PASS '),
 'removed_outputs':removed,'generated_output_removed':not list(TARGET.glob(binary.stem+'.*')),
 'scope':'Exact current add_node_facets and record_values functions extracted verbatim; retained real card/facet-store types, one shadowing observation and two controls. No complete MereRouteSession, native viewer, owner SetView invocation, persistence, network or GUI executed. Dependency bytes retained from an earlier successful build; not a full-current Cargo gate.'}
with out.open('x',encoding='utf-8') as f:f.write(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({'receipt':out.name,'build_exit':built.returncode,'run_exit':receipt['run_exit'],'checks':receipt['checks']}))
assert built.returncode==0,built.stderr
assert ran.returncode==0,ran.stderr
