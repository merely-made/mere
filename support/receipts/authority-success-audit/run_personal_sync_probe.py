"""Three product/transfer checks against the successful current library build."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,struct
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
dest=HERE/'personal_sync_probe_receipt.json';assert not dest.exists()
terminal=json.loads((HERE/'personal_sync_build_terminal.json').read_text());assert terminal['exit_code']==0 and terminal['build_finished']['success']
sources={n:sha(ROOT/n) for n in ['ports/graphshell/src/personal_sync.rs','ports/graphshell/src/personal_sync/assertions.rs','crates/graph/graph-kernel/src/graph/apply.rs','crates/graph/graph-kernel/src/graph/resource_tags.rs']}
rows=[json.loads(l) for l in (HERE/'personal_sync_build.jsonl').read_text(encoding='utf-8-sig').splitlines() if l.startswith('{')]
assert any(r.get('reason')=='build-finished' and r.get('success') for r in rows)
libs={}
for name in ['graphshell','mere','eidetic','muniment','pollster','serde_json','uuid','p2panda_core']:
 matches=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']==name and any(f.endswith('.rlib') for f in r['filenames'])]
 if name=='pollster':matches=[r for r in matches if r['package_id'].endswith('#pollster@1.0.1')]
 assert len(matches)==1,(name,len(matches))
 r=matches[0];path=next(Path(f) for f in r['filenames'] if f.endswith('.rlib'))
 if path.parent==TARGET/'debug':
  candidates=[p for p in (TARGET/'build/debug/deps').glob('lib'+name+'-*.rlib') if sha(p)==sha(path)];assert candidates;path=candidates[0]
 libs[name]={'path':str(path),'sha256':sha(path),'features':r['features'],'package_id':r['package_id'],'target_source':r['target']['src_path']}
macro_rows=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']=='async_trait' and 'proc-macro' in r['target']['kind']]
assert len(macro_rows)==1,len(macro_rows)
r=macro_rows[0];path=next(Path(f) for f in r['filenames'] if f.endswith('.dll'))
libs['async_trait']={'path':str(path),'sha256':sha(path),'features':r['features'],'package_id':r['package_id'],'target_source':r['target']['src_path']}
binary=TARGET/'authority-success-personal-sync-probe.exe';assert not list(TARGET.glob(binary.stem+'.*'))
command=['rustc','--edition=2024','--test',str(HERE/'personal_sync_probe.rs'),'-L','dependency='+str(TARGET/'build/debug/deps'),'-C','debuginfo=0','-o',str(binary)]
for name,lib in libs.items():command+=['--extern',name+'='+lib['path']]
native=sorted({p for r in rows if r.get('reason')=='build-script-executed' for p in r.get('linked_paths',[])})
for p in native:command+=['-L',p]
built=subprocess.run(command,cwd=ROOT,capture_output=True,text=True)
(HERE/'personal_sync_probe_compile.log').write_text(built.stdout+built.stderr,encoding='utf-8');assert built.returncode==0,built.stderr
digest=sha(binary);removed=[]
try:
 ran=subprocess.run([str(binary),'--nocapture','--test-threads=1'],cwd=ROOT,capture_output=True,text=True,timeout=45)
 (HERE/'personal_sync_probe_output.log').write_text(ran.stdout+ran.stderr,encoding='utf-8');print(ran.stdout)
finally:
 for p in sorted(TARGET.glob(binary.stem+'.*')):
  assert p.resolve().parent==TARGET;data=p.read_bytes()
  if p.suffix=='.exe':assert data.startswith(b'MZ')
  elif p.suffix=='.pdb':assert data.startswith(b'Microsoft C/C++ MSF')
  elif p.suffix=='.lib':assert data.startswith(b'!<arch>\n') and binary.name.encode() in data
  elif p.suffix=='.exp':assert data[:2]==b'\x64\x86' and 0<struct.unpack_from('<H',data,2)[0]<128 and binary.name.encode() in data
  else:raise AssertionError('unknown generated output '+str(p))
  removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
assert ran.returncode==0,ran.stderr
assert all(sha(ROOT/n)==h for n,h in sources.items());assert all(sha(Path(v['path']))==v['sha256'] for v in libs.values())
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'source_sha256':sources,'libraries':libs,'native_search_paths':native,'probe_build_command':command,'probe_build_exit':built.returncode,'native_binary_sha256':digest,'test_exit':ran.returncode,'tests':5,'removed_outputs':removed,'generated_output_removed':not list(TARGET.glob(binary.stem+'.*')),'scope':'Actual public PersonalGraphReplica with fresh current personal-sync-feature artifacts, synthetic signed operations and in-memory stores. Tests check tag stratum, changed-versus-identical access facts, missing-endpoint admission/materialization and valid endpoint control. No installed profile, network, physical disk, custody or headed run.'}

dest.write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8');print(json.dumps({'receipt':dest.name,'tests':5,'exit':ran.returncode,'generated_output_removed':receipt['generated_output_removed']}))
