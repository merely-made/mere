use chirograph::{ResourceAssembly,ResourceChunkResponse,AssemblyError};
fn frame(name:&str)->ResourceChunkResponse {
 let fixture:serde_json::Value=serde_json::from_str(include_str!("resource_chunk_fixture.json")).unwrap();
 serde_json::from_value(fixture["frames"][name].clone()).unwrap()
}
#[test]
fn shared_owner_rejects_the_corrupt_chunk_the_js_copy_accepts(){
 assert_eq!(ResourceAssembly::new().accept(&frame("bad_chunk")),Err(AssemblyError::CorruptChunk));
}
#[test]
fn shared_owner_rejects_the_wrong_resource_the_js_copy_accepts(){
 assert_eq!(ResourceAssembly::new().accept(&frame("bad_resource")),Err(AssemblyError::WrongResource));
}
#[test]
fn valid_fixture_control_is_accepted_by_the_shared_owner(){
 assert_eq!(ResourceAssembly::new().accept(&frame("valid")).unwrap().unwrap(),b"original");
}
