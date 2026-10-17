"""Fresh original cache module tests plus shared-byte ownership observations."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,re
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2];TARGET=Path('C:/t/cargo-targets/mere').resolve();NAME='shared_image_cache_probe'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
source=ROOT/'crates/canvas/pictograph/src/canvas/resolved_image_cache.rs';source_hash=sha(source)
probe=HERE/(NAME+'.rs');assert not probe.exists() and not (HERE/(NAME+'_receipt.json')).exists()
body='#[path='+json.dumps(source.as_posix())+'] mod cache;\n'
body+='''
#[cfg(test)] mod audit {
 use super::cache::ResolvedImageCache;use std::sync::Arc;
 #[test] fn frame_handles_share_one_decoded_allocation(){
  let mut c=ResolvedImageCache::new(8);c.insert([1;32],vec![7;8],2,1);
  let a=c.peek(&[1;32]).unwrap().0.clone();let b=c.peek(&[1;32]).unwrap().0.clone();
  assert!(Arc::ptr_eq(&a,&b));assert_eq!(c.used_bytes(),8);assert_eq!(Arc::strong_count(&a),3);
 }
 #[test] fn eviction_releases_cache_ownership_but_preserves_an_existing_frame_handle(){
  let mut c=ResolvedImageCache::new(8);c.insert([1;32],vec![7;8],2,1);
  let frame=c.peek(&[1;32]).unwrap().0.clone();assert_eq!(Arc::strong_count(&frame),2);
  assert_eq!(c.set_limit(0),vec![[1;32]]);assert_eq!(c.used_bytes(),0);assert!(!c.contains(&[1;32]));
  assert_eq!(frame.as_slice(),&[7;8]);assert_eq!(Arc::strong_count(&frame),1);
 }
}
'''
with probe.open('x',encoding='utf-8') as f:f.write(body)
binary=TARGET/'authority-success-shared-image-cache.exe';assert not list(TARGET.glob(binary.stem+'.*'))
cmd=['rustc','--edition=2024','--test',str(probe),'-C','debuginfo=0','-o',str(binary)]
build=subprocess.run(cmd,cwd=ROOT,capture_output=True,text=True,timeout=45)
with (HERE/(NAME+'_compile.log')).open('x',encoding='utf-8') as f:f.write(build.stdout+build.stderr)
run=None;removed=[]
try:
 if build.returncode==0:
  run=subprocess.run([str(binary),'--test-threads=1'],cwd=ROOT,capture_output=True,text=True,timeout=20)
  with (HERE/(NAME+'_output.log')).open('x',encoding='utf-8') as f:f.write(run.stdout+run.stderr)
  print(run.stdout+run.stderr)
 else:print(build.stderr)
finally:
 for p in sorted(TARGET.glob(binary.stem+'.*')):
  assert p.resolve().parent==TARGET;data=p.read_bytes()
  assert (p.suffix=='.exe' and data.startswith(b'MZ')) or (p.suffix=='.pdb' and data.startswith(b'Microsoft C/C++ MSF')) or (p.suffix=='.lib' and data.startswith(b'!<arch>\n')) or (p.suffix=='.exp' and data.startswith(b'd\x86'))
  removed.append({'path':str(p),'bytes':len(data),'sha256':sha(p)});p.unlink()
assert sha(source)==source_hash
checks=0 if run is None or run.returncode else int(re.search(r'test result: ok\. (\d+) passed;',run.stdout).group(1))
receipt={'date_utc':datetime.now(timezone.utc).isoformat(),'basis_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'source_sha256':{source.relative_to(ROOT).as_posix():source_hash},'probe_sha256':sha(probe),'runner_sha256':sha(Path(__file__)),'command':cmd,'build_exit':build.returncode,'run_exit':None if run is None else run.returncode,'checks':checks,'original_tests':2,'audit_cases':2,'removed_outputs':removed,'generated_output_removed':not list(TARGET.glob(binary.stem+'.*')),'qualification':'Fresh unchanged std-only private cache source, two original tests plus two ownership observations. No whole Canvas, paint/compositor dependency build, image store, renderer, GPU or retained-scene lifetime proof.'}
with (HERE/(NAME+'_receipt.json')).open('x',encoding='utf-8') as f:json.dump(receipt,f,indent=2);f.write('\n')
print(json.dumps({k:receipt[k] for k in ['build_exit','run_exit','checks','generated_output_removed']}))
assert build.returncode==0 and run is not None and run.returncode==0 and checks==4
