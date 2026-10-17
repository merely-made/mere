from pathlib import Path
import json,hashlib
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[2]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
modules=[ROOT/('crates/intel/esp/src/infer/'+n+'.rs') for n in ['provider','stub']]
body='mod infer {\n'+''.join('#[path='+json.dumps(p.as_posix())+'] pub mod '+p.stem+';\n' for p in modules)+'}\n'
body+='''
#[cfg(test)] mod audit {
 use crate::infer::provider::{InferenceProvider,GenerationRequest};
 use crate::infer::stub::StubInferenceProvider;
 use std::{cell::Cell,ops::ControlFlow};
 fn run(cancel_at:usize)->(String,Vec<String>){
  let p=StubInferenceProvider::new().with_response("q","alpha beta gamma");
  let request=GenerationRequest{prompt:"q".into(),..Default::default()};
  let polls=Cell::new(0);let mut fragments=Vec::new();
  let text=p.generate_streaming_controlled(&request,&mut |s|{fragments.push(s.into());ControlFlow::Continue(())},&mut ||{polls.set(polls.get()+1);polls.get()>=cancel_at}).unwrap();
  (text,fragments)
 }
 #[test] fn cancellation_before_first_delivery_returns_undelivered_text(){
  let(text,fragments)=run(2);assert_eq!(text,"alpha");assert!(fragments.is_empty());
 }
 #[test] fn cancellation_after_first_delivery_returns_extra_fragment(){
  let(text,fragments)=run(3);assert_eq!(text,"alpha beta");assert_eq!(fragments,vec!["alpha"]);
  assert_ne!(text,fragments.concat());
 }
}
'''
with (HERE/'infer_cancellation_probe.rs').open('x',encoding='utf-8') as f:f.write(body)
manifest={'source_sha256':{p.relative_to(ROOT).as_posix():sha(p) for p in modules},'qualification':'Two unchanged complete current portable inference provider and stub modules. Public generate_streaming_controlled API only. No actor, session wrapper, model, decoder or GPU execution.'}
with (HERE/'infer_cancellation_source.json').open('x',encoding='utf-8') as f:json.dump(manifest,f,indent=2);f.write('\n')
r=(HERE/'run_decoder_text_math_probe_v2.py').read_text()
first=r.index("token=TARGET/");last=r.index("binary=TARGET/",first)
r=r[:first]+r[last:]
r=r.replace("NAME='decoder_text_math_probe_v2'","NAME='infer_cancellation_probe'").replace("HERE/'decoder_text_math_source.json'","HERE/'infer_cancellation_source.json'").replace("authority-success-decoder-text-math-v2.exe","authority-success-infer-cancellation.exe").replace("'audit_cases':6","'audit_cases':2").replace("'original_tests':checks-6","'original_tests':checks-2").replace('checks>=6','checks>=2')
with (HERE/'run_infer_cancellation_probe.py').open('x',encoding='utf-8') as f:f.write(r)
