mod infer {
#[path="C:/Users/mark_/Code/repos/mere/crates/intel/esp/src/infer/provider.rs"] pub mod provider;
#[path="C:/Users/mark_/Code/repos/mere/crates/intel/esp/src/infer/stub.rs"] pub mod stub;
}

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
