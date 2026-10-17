#[path="C:/Users/mark_/Code/repos/mere/crates/canvas/pictograph/src/canvas/resolved_image_cache.rs"] mod cache;

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
